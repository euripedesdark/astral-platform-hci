#!/usr/bin/env python3
"""Build and deploy the Rust backend and Vite frontend as one systemd release."""
import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path
import pwd
import re
import shutil
import subprocess
import sys
import time
import tempfile
import platform
from urllib.request import urlopen

ROOT = Path(__file__).resolve().parent
APP = Path('/opt/astral-platform-hci')
UNIT = 'astral-platform-hci.service'
LEGACY = 'astral-platform.service'
UNIT_DIR = Path('/etc/systemd/system')
RUNTIME = Path('/etc/astral/platform-hci.env')


def run(args, **kwargs):
    print('+ ' + ' '.join(map(str, args)), flush=True)
    return subprocess.run(list(map(str, args)), check=True, **kwargs)



def dependency_commands():
    """Select the host package manager before changing services or installing."""
    try:
        release = getattr(platform, 'freedesktop_os_release', lambda: {})()
    except OSError:
        release = {}
    family = set((release.get('ID', '') + ' ' + release.get('ID_LIKE', '')).split())
    managers = ('dnf', 'yum', 'apt-get') if family & {
        'fedora', 'rhel', 'centos', 'rocky', 'almalinux'
    } else ('apt-get', 'dnf', 'yum')
    for manager in managers:
        executable = shutil.which(manager)
        if not executable:
            continue
        if manager == 'apt-get':
            return [[executable, 'update'], [executable, 'install', '-y',
                     'build-essential', 'pkg-config', 'libssl-dev', 'nodejs',
                     'npm', 'ca-certificates', 'git']]
        return [[executable, 'install', '-y', 'gcc', 'gcc-c++', 'make',
                 'pkgconf-pkg-config', 'openssl-devel', 'nodejs', 'npm',
                 'ca-certificates', 'git']]
    raise RuntimeError('Nenhum gerenciador suportado encontrado (dnf, yum ou apt-get). '
                       'Instale compilador C/C++, make, pkg-config, OpenSSL devel, '
                       'Node.js >=18, npm, git e certificados CA; execute sem --install-deps.')


def systemctl(*args, check=True):
    return subprocess.run(['systemctl', *args], check=check, stdout=subprocess.PIPE,
                          stderr=subprocess.PIPE, text=True)


def snapshot(path):
    if path.is_symlink():
        return ('link', os.readlink(path))
    if path.exists():
        return ('file', path.read_bytes(), path.stat().st_mode & 0o777)
    return None


def restore(path, saved):
    if path.exists() or path.is_symlink():
        path.unlink()
    if saved:
        path.parent.mkdir(parents=True, exist_ok=True)
        if saved[0] == 'link':
            path.symlink_to(saved[1])
        else:
            path.write_bytes(saved[1])
            path.chmod(saved[2])


def switch_current(target):
    temporary = APP / 'current.next'
    temporary.unlink(missing_ok=True)
    temporary.symlink_to(target)
    temporary.replace(APP / 'current')


def verify(base, index=None):
    """Check real HTML and every generated JS/CSS asset, not a SPA fallback 200."""
    with urlopen(base + '/health', timeout=3) as response:
        if json.load(response).get('status') != 'UP':
            raise RuntimeError('Health diferente de UP')
    with urlopen(base + '/app/', timeout=3) as response:
        html = response.read()
        if 'text/html' not in response.headers.get('Content-Type', ''):
            raise RuntimeError('Frontend sem Content-Type HTML')
    if index and html != index.read_bytes():
        raise RuntimeError('Frontend servido nao corresponde a esta compilacao')
    assets = re.findall(rb'(?:src|href)="(/app/assets/[^"?]+\.(?:js|css))"', html)
    if not assets:
        raise RuntimeError('Frontend sem assets JS/CSS compilados')
    for asset in assets:
        with urlopen(base + asset.decode(), timeout=5) as response:
            body = response.read()
            if not body or 'text/html' in response.headers.get('Content-Type', ''):
                raise RuntimeError('Asset ausente: ' + asset.decode())
            if index:
                local = index.parent / asset.decode().removeprefix('/app/')
                if body != local.read_bytes():
                    raise RuntimeError('Asset servido difere do build: ' + asset.decode())


def unit_text():
    return f'''[Unit]
Description=Astral Platform HCI (Rust + React)
Wants=network-online.target
After=network-online.target

[Service]
Type=simple
WorkingDirectory={APP}/current
EnvironmentFile=-/etc/astral/astral.env
EnvironmentFile={RUNTIME}
ExecStart={APP}/current/bin/astral-platform-hci
Restart=on-failure
RestartSec=3
TimeoutStopSec=30
UMask=0027

[Install]
WantedBy=multi-user.target
Alias={LEGACY}
'''


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--install-deps', action='store_true', help='instalar dependencias via dnf/yum (Fedora/RHEL) ou apt-get (Debian/Ubuntu)')
    parser.add_argument('--port', type=int, default=8082)
    parser.add_argument('--check', action='store_true', help='verificar servico e frontend sem alterar arquivos')
    parser.add_argument('--plan', action='store_true', help='mostrar etapas sem executar')
    args = parser.parse_args()
    if not 1 <= args.port <= 65535:
        parser.error('porta deve estar entre 1 e 65535')
    if args.plan:
        print('Preflight -> parar HCI -> cargo test/build --release --locked -> npm ci/build -> release atomica -> systemd -> verificar health/HTML/assets; rollback em falha.')
        return
    if args.check:
        systemctl('is-active', UNIT)
        verify(f'http://127.0.0.1:{args.port}', APP / 'current/static/app/index.html')
        print('OK: servico, health, frontend e assets.')
        return
    if os.geteuid() != 0:
        os.execvp('sudo', ['sudo', sys.executable, str(Path(__file__).resolve()), *sys.argv[1:]])
    user = pwd.getpwnam(os.environ.get('SUDO_USER', 'root'))
    build_env = os.environ.copy()
    build_env.update(HOME=user.pw_dir, USER=user.pw_name, LOGNAME=user.pw_name,
                     PATH=f'{user.pw_dir}/.cargo/bin:' + os.environ.get('PATH', ''))
    def build(command, cwd=ROOT, **kwargs):
        def demote():
            if user.pw_uid:
                os.initgroups(user.pw_name, user.pw_gid)
                os.setgid(user.pw_gid)
                os.setuid(user.pw_uid)
        return run(command, cwd=cwd, env=build_env, preexec_fn=demote, **kwargs)
    if args.install_deps:
        for command in dependency_commands():
            run(command)
    if args.install_deps and not shutil.which('cargo', path=build_env['PATH']):
        arch = {'x86_64': 'x86_64', 'aarch64': 'aarch64'}.get(platform.machine())
        if not arch:
            raise RuntimeError('Instale Rust estavel via rustup nesta arquitetura.')
        with tempfile.TemporaryDirectory() as temporary:
            os.chmod(temporary, 0o755)
            installer = Path(temporary) / 'rustup-init'
            with urlopen(f'https://static.rust-lang.org/rustup/dist/{arch}-unknown-linux-gnu/rustup-init', timeout=60) as response:
                installer.write_bytes(response.read())
            installer.chmod(0o755)
            build([str(installer), '-y', '--profile', 'minimal', '--default-toolchain', 'stable'])
    for tool in ('cargo', 'rustc', 'npm', 'node', 'cc', 'pkg-config', 'systemctl', 'git'):
        if not shutil.which(tool, path=build_env['PATH']):
            raise RuntimeError(f'{tool} ausente. Instale Rust estavel via rustup para seu usuario; use --install-deps para dependencias Fedora/RHEL ou Debian/Ubuntu.')
    run(['pkg-config', '--exists', 'openssl'])
    build(['cargo', '--version'])
    node_version = subprocess.check_output(['node', '--version'], env=build_env, text=True).strip()
    if int(node_version.lstrip('v').split('.')[0]) < 18:
        raise RuntimeError('Node.js >= 18 necessario para Vite. Atualize Node.js antes de executar o deploy; servico nao foi parado.')
    build(['node', '--version'])
    if not Path('/run/systemd/system').is_dir():
        raise RuntimeError('Este deploy requer Linux com systemd ativo.')
    for filename in ('Cargo.lock', 'frontend/package-lock.json'):
        if not (ROOT / filename).is_file():
            raise RuntimeError(f'Arquivo obrigatorio ausente: {filename}')
    current = APP / 'current'
    if current.exists() and not current.is_symlink():
        raise RuntimeError(f'{current} deve ser link simbolico; nada foi alterado.')
    APP.mkdir(parents=True, exist_ok=True)
    with (APP / 'deploy.lock').open('a') as lock:
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            raise RuntimeError('Outro deploy do HCI esta em andamento.')
        paths = [UNIT_DIR / UNIT, UNIT_DIR / LEGACY, RUNTIME, current,
                 UNIT_DIR / 'multi-user.target.wants' / UNIT,
                 UNIT_DIR / 'multi-user.target.wants' / LEGACY]
        saved = {path: snapshot(path) for path in paths}
        active = [name for name in (UNIT, LEGACY) if systemctl('is-active', name, check=False).returncode == 0]
        # Keep a durable rollback copy, including legacy units and enablement links.
        transaction = APP / 'history' / time.strftime('%Y%m%d-%H%M%S')
        transaction.mkdir(parents=True)
        for number, (path, state) in enumerate(saved.items()):
            if state and state[0] == 'file':
                backup = transaction / f'{number}-{path.name}'
                backup.write_bytes(state[1])
                backup.chmod(0o600)
        changed = False
        try:
            for name in active:
                systemctl('stop', name)
            build(['cargo', 'test', '--locked'])
            build(['cargo', 'build', '--release', '--locked'])
            build(['npm', 'ci', '--no-audit', '--no-fund'], ROOT / 'frontend')
            build(['npm', 'run', 'build'], ROOT / 'frontend')
            static = ROOT / 'src/main/resources/static'
            if not (static / 'app/index.html').is_file():
                raise RuntimeError('Build nao produziu static/app/index.html')
            sha = build(['git', '-C', str(ROOT), 'rev-parse', '--short', 'HEAD'], stdout=subprocess.PIPE, text=True).stdout.strip()
            release = APP / 'releases' / (time.strftime('%Y%m%d-%H%M%S') + '-' + sha)
            (release / 'bin').mkdir(parents=True)
            shutil.copy2(ROOT / 'target/release/astral-platform-hci', release / 'bin')
            shutil.copytree(static, release / 'static')
            (release / 'manifest.json').write_text(json.dumps({'commit': sha, 'binary_sha256': hashlib.sha256((release / 'bin/astral-platform-hci').read_bytes()).hexdigest()}, indent=2))
            changed = True
            # Remove old enablement before replacing its real file with an alias.
            systemctl('disable', LEGACY, check=False)
            for path in (UNIT_DIR / UNIT, UNIT_DIR / LEGACY):
                path.unlink(missing_ok=True)
            (UNIT_DIR / UNIT).write_text(unit_text())
            (UNIT_DIR / LEGACY).symlink_to(UNIT)
            RUNTIME.parent.mkdir(parents=True, exist_ok=True)
            RUNTIME.write_text(f'SERVER_ADDRESS=127.0.0.1\nSERVER_PORT={args.port}\nASTRAL_STATIC_DIR={APP}/current/static\n')
            RUNTIME.chmod(0o640)
            switch_current(release)
            systemctl('daemon-reload')
            systemctl('enable', UNIT)
            systemctl('reset-failed', UNIT, check=False)
            systemctl('start', UNIT)
            deadline = time.monotonic() + 60
            while True:
                try:
                    systemctl('is-active', UNIT)
                    verify(f'http://127.0.0.1:{args.port}', release / 'static/app/index.html')
                    break
                except Exception:
                    if time.monotonic() >= deadline:
                        raise RuntimeError('Servico/health/frontend nao ficaram prontos em 60s')
                    time.sleep(1)
            print(f'OK: {UNIT}, backend + frontend em http://127.0.0.1:{args.port}/app/\nRelease: {release}\nLogs: sudo journalctl -u {UNIT} -n 100 --no-pager')
        except BaseException:
            print('Falha: recuperando servico anterior...', file=sys.stderr)
            if changed:
                systemctl('stop', UNIT, check=False)
                for path, state in saved.items():
                    restore(path, state)
                systemctl('daemon-reload', check=False)
            for name in active:
                result = systemctl('start', name, check=False)
                if result.returncode:
                    print(f'Falha ao reiniciar {name}: {result.stderr}', file=sys.stderr)
            subprocess.run(['journalctl', '-u', UNIT, '-n', '40', '--no-pager'])
            raise


if __name__ == '__main__':
    try:
        main()
    except Exception as error:
        print(f'ERRO: {error}', file=sys.stderr)
        sys.exit(1)

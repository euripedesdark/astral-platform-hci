#!/usr/bin/env python3
"""Register and deploy OAuth2 in auth-service and HCI without rewriting nginx."""
import argparse
import fcntl
import json
import os
from pathlib import Path
import pwd
import secrets
import shlex
import socket
import ssl
import subprocess
import sys
import tempfile
from urllib.parse import urlsplit
import deploy_platform as deploy

ROOT = Path(__file__).resolve().parent
AUTH_ENV = Path('/etc/auth-service/oauth2.env')
CLIENTS = Path('/etc/auth-service/oauth2-clients.json')
HCI_ENV = Path('/etc/astral/platform-oauth2.env')


def public_url(value):
    url = urlsplit(value)
    _ = url.port  # reject malformed/out-of-range ports
    if url.scheme != 'https' or not url.hostname or url.username or url.password or url.query or url.fragment or url.path not in ('', '/') or any(c.isspace() for c in value):
        raise ValueError('--public-url deve ser uma origem HTTPS, sem caminho/query/credenciais')
    host = url.hostname.encode('idna').decode('ascii').lower()
    if ':' in host: host = '[' + host + ']'
    port = ':' + str(url.port) if url.port and url.port != 443 else ''
    return 'https://' + host + port


def registration(existing, origin):
    clients = json.loads(json.dumps(existing))
    if not isinstance(clients, list): raise ValueError('Cadastro OAuth existente deve ser uma lista JSON')
    current = [item for item in clients if item.get('client_id') == 'astral-platform-hci']
    if len(current) > 1: raise ValueError('Cliente HCI duplicado')
    if current:
        client = current[0]
        if len(client.get('client_secret', '')) < 32: raise ValueError('Segredo do cliente HCI existente invalido')
    else:
        client = {'client_id': 'astral-platform-hci', 'client_secret': secrets.token_urlsafe(48), 'redirect_uris': []}
        clients.append(client)
    redirect = origin + '/login/oauth2/code/auth-service'
    if redirect not in client['redirect_uris']: client['redirect_uris'].append(redirect)
    return clients, client


def env_text(values):
    # JSON quoting is compatible with systemd EnvironmentFile double-quoted values.
    return ''.join(key + '=' + json.dumps(value, ensure_ascii=False) + '\n' for key, value in values.items())


def read_env(path):
    values = {}
    if path.exists():
        for line in path.read_text().splitlines():
            if not line.strip() or line.lstrip().startswith('#'): continue
            key, separator, raw = line.partition('=')
            if not separator: raise ValueError('Linha invalida no arquivo de ambiente OAuth2')
            words = shlex.split(raw, comments=True)
            values[key.strip()] = ' '.join(words)
    return values


def atomic_write(path, content):
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, temporary = tempfile.mkstemp(prefix=path.name + '.', dir=path.parent)
    try:
        with os.fdopen(fd, 'w') as file:
            os.fchmod(file.fileno(), 0o600)
            file.write(content)
        os.replace(temporary, path)
    finally:
        Path(temporary).unlink(missing_ok=True)


def as_user(args, cwd):
    user = pwd.getpwnam(os.environ.get('SUDO_USER', 'root'))
    env = os.environ.copy(); env.update(HOME=user.pw_dir, USER=user.pw_name, LOGNAME=user.pw_name)
    def demote():
        if user.pw_uid:
            os.initgroups(user.pw_name, user.pw_gid); os.setgid(user.pw_gid); os.setuid(user.pw_uid)
    subprocess.run(args, cwd=cwd, env=env, preexec_fn=demote, check=True)


def check_directory(values, ca_file=None):
    required = ('AUTH_LDAP_URL', 'AUTH_LDAP_BASE', 'AUTH_LDAP_USER_DN', 'AUTH_LDAP_PASSWORD', 'AUTH_TOKEN_SECRET')
    missing = [name for name in required if not values.get(name)]
    if missing: raise RuntimeError('Faltam configuracoes existentes em auth.env: ' + ', '.join(missing))
    if len(values['AUTH_TOKEN_SECRET']) < 32: raise RuntimeError('AUTH_TOKEN_SECRET deve ter ao menos 32 caracteres')
    url = urlsplit(values['AUTH_LDAP_URL'])
    if url.scheme != 'ldaps' or not url.hostname:
        raise RuntimeError('Configure AUTH_LDAP_URL com LDAPS para o login corporativo OAuth2')
    context = ssl.create_default_context(cafile=str(ca_file) if ca_file else values.get('AUTH_LDAP_CA_FILE'))
    try:
        with socket.create_connection((url.hostname, url.port or 636), timeout=5) as connection:
            with context.wrap_socket(connection, server_hostname=url.hostname): pass
    except (OSError, ssl.SSLError) as error:
        raise RuntimeError('LDAPS nao passou na verificacao TLS antes do deploy. Confira hostname/SAN e CA; use --ldap-ca-file /caminho/ca.crt se necessario. ' + str(error)) from error


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--public-url', required=True, type=public_url, help='ex.: https://192.168.2.10 (host HTTPS do HCI)')
    parser.add_argument('--auth-source', type=Path, default=ROOT.parent / 'auth-service')
    parser.add_argument('--ldap-ca-file', type=Path, help='CA PEM do Samba AD, caso nao esteja no trust store do sistema')
    parser.add_argument('--install-deps', action='store_true')
    args = parser.parse_args()
    if os.geteuid() != 0:
        os.execvp('sudo', ['sudo', sys.executable, str(Path(__file__).resolve()), *sys.argv[1:]])
    ROOT.parent.mkdir(parents=True, exist_ok=True)
    lock_path = Path('/run/lock/astral-oauth2-setup.lock')
    with lock_path.open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        preflight = [sys.executable, str(ROOT / 'deploy_platform.py'), '--prepare-only']
        if args.install_deps: preflight.append('--install-deps')
        subprocess.run(preflight, check=True)
        if not Path('/etc/auth-service/auth.env').is_file():
            raise RuntimeError('Falta /etc/auth-service/auth.env com LDAP/TLS e segredo JWT ja usados pelo auth-service. Nada foi parado.')
        source = args.auth_source.resolve()
        if not source.exists():
            as_user(['git', 'clone', 'https://github.com/euripedesdark/auth-service.git', str(source)], source.parent)
        else:
            result = subprocess.run(['git', '-c', 'safe.directory=' + str(source), '-C', str(source), 'remote', 'get-url', 'origin'], capture_output=True, text=True, check=True)
            if result.stdout.strip().removesuffix('.git').rstrip('/') not in ('https://github.com/euripedesdark/auth-service', 'git@github.com:euripedesdark/auth-service'):
                raise RuntimeError('--auth-source nao aponta ao repositorio auth-service esperado')
            # --ff-only preserves local work and refuses a diverged branch.
            as_user(['git', 'pull', '--ff-only', 'origin', 'main'], source)
        if not (source / 'deploy_auth.py').is_file(): raise RuntimeError('Atualize auth-service/main com o implemento OAuth2 antes de continuar')
        subprocess.run(['nginx', '-t'], check=True)
        directory_values = read_env(Path('/etc/auth-service/auth.env'))
        auth_values = read_env(AUTH_ENV)
        directory_values.update(auth_values)
        check_directory(directory_values, args.ldap_ca_file)
        if args.ldap_ca_file: auth_values['AUTH_LDAP_CA_FILE'] = str(args.ldap_ca_file.resolve())
        issuer = auth_values.get('AUTH_OAUTH_ISSUER_URL', args.public_url + '/auth').rstrip('/')
        issuer_parts = urlsplit(issuer)
        if issuer_parts.scheme != 'https' or not issuer_parts.hostname or issuer_parts.query or issuer_parts.fragment or issuer_parts.username or issuer_parts.password:
            raise ValueError('Issuer OAuth2 existente invalido; nenhuma configuracao foi alterada')
        clients_path = Path(auth_values.get('AUTH_OAUTH_CLIENTS_FILE', str(CLIENTS)))
        if not clients_path.is_absolute(): raise ValueError('Registro de clientes deve usar caminho absoluto')
        clients, client = registration(json.loads(clients_path.read_text()) if clients_path.exists() else [], args.public_url)
        saved = {path: deploy.snapshot(path) for path in (clients_path, AUTH_ENV, HCI_ENV)}
        auth_deployed = False
        auth_attempted = False
        active_auth = [name for name in ('auth-service.service', 'astral-auth-service.service', 'astral-auth.service')
                       if subprocess.run(['systemctl', 'is-active', name], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode == 0]
        try:
            atomic_write(clients_path, json.dumps(clients, indent=2) + '\n')
            auth_values.update(AUTH_OAUTH_CLIENTS_FILE=str(clients_path), AUTH_OAUTH_ISSUER_URL=issuer, SERVER_ADDRESS='127.0.0.1', SERVER_PORT='8181')
            atomic_write(AUTH_ENV, env_text(auth_values))
            hci_values = read_env(HCI_ENV)
            shared_hci = read_env(Path('/etc/astral/astral.env'))
            hci_values.setdefault('ASTRAL_JWT_SECRET', shared_hci.get('ASTRAL_JWT_SECRET') or secrets.token_urlsafe(48))
            if len(hci_values['ASTRAL_JWT_SECRET']) < 32: raise ValueError('Segredo JWT existente do HCI deve ter ao menos 32 caracteres')
            if auth_values.get('AUTH_LDAP_CA_FILE'): hci_values.setdefault('ASTRAL_AD_CA_FILE', auth_values['AUTH_LDAP_CA_FILE'])
            hci_values.update({'ASTRAL_OAUTH_AUTHORIZATION_URL': issuer + '/oauth2/authorize', 'ASTRAL_OAUTH_TOKEN_URL': 'http://127.0.0.1:8181/oauth2/token', 'ASTRAL_OAUTH_USERINFO_URL': 'http://127.0.0.1:8181/oauth2/userinfo', 'ASTRAL_OAUTH_REDIRECT_URI': args.public_url + '/login/oauth2/code/auth-service', 'ASTRAL_OAUTH_CLIENT_ID': client['client_id'], 'ASTRAL_OAUTH_CLIENT_SECRET': client['client_secret']})
            atomic_write(HCI_ENV, env_text(hci_values))
            auth_attempted = True
            subprocess.run([sys.executable, str(source / 'deploy_auth.py')], check=True)
            auth_deployed = True
            subprocess.run([sys.executable, str(ROOT / 'deploy_platform.py')], check=True)
        except BaseException:
            # A failed HCI deployment rolls itself back. Preserve the already-running
            # compatible provider/config, which can be retried without rotating secrets.
            if not auth_deployed:
                for path, state in saved.items(): deploy.restore(path, state)
                if auth_attempted:
                    for name in active_auth:
                        subprocess.run(['systemctl', 'try-restart', name], check=False)
            raise
        print('OK: OAuth2 cadastrado e implantado. Abra ' + args.public_url + '/app/')
        print('Nginx, TLS, ERP e demais clientes OAuth existentes foram preservados.')


if __name__ == '__main__':
    try: main()
    except Exception as error:
        print('ERRO: ' + str(error), file=sys.stderr); sys.exit(1)

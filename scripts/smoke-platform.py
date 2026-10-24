#!/usr/bin/env python3
"""Run the actual release on an ephemeral loopback port, without systemd/DB writes."""
import importlib.util
import os
from pathlib import Path
import shutil
import socket
import subprocess
import tempfile
import time
from urllib.request import urlopen

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('deploy', ROOT / 'deploy_platform.py')
deploy = importlib.util.module_from_spec(spec)
spec.loader.exec_module(deploy)


def main():
    binary = ROOT / 'target/release/astral-platform-hci'
    with tempfile.TemporaryDirectory() as temporary:
        static = Path(temporary) / 'static'
        shutil.copytree(ROOT / 'src/main/resources/static', static)
        with socket.socket() as listener:
            listener.bind(('127.0.0.1', 0))
            port = listener.getsockname()[1]
        env = os.environ.copy()
        env.update(SERVER_PORT=str(port), SERVER_ADDRESS='127.0.0.1', ASTRAL_STATIC_DIR=str(static))
        with tempfile.TemporaryFile() as log:
            process = subprocess.Popen([str(binary)], cwd=temporary, env=env, stdout=log, stderr=log)
            try:
                for _ in range(100):
                    try:
                        deploy.verify(f'http://127.0.0.1:{port}', static / 'app/index.html')
                        break
                    except Exception:
                        if process.poll() is not None:
                            log.seek(0)
                            raise RuntimeError(log.read().decode())
                        time.sleep(0.1)
                else:
                    raise RuntimeError('Backend startup timeout')
                for route, local in [('/images/login.png', 'app/images/login.png'),
                                     ('/app/images/Fundo.png', 'app/images/Fundo.png'),
                                     ('/app/overview', 'app/index.html')]:
                    with urlopen(f'http://127.0.0.1:{port}' + route, timeout=5) as response:
                        if response.read() != (static / local).read_bytes():
                            raise RuntimeError('Conteudo inesperado: ' + route)
                original = ROOT / 'src/main/resources/static/app/index.html'
                (static / 'app/index.html').write_text('stale frontend')
                try:
                    deploy.verify(f'http://127.0.0.1:{port}', original)
                except RuntimeError:
                    pass
                else:
                    raise RuntimeError('Checker aceitou frontend divergente')
            finally:
                process.terminate()
                process.wait(timeout=10)
        env['ASTRAL_STATIC_DIR'] = str(Path(temporary) / 'missing')
        result = subprocess.run([str(binary)], cwd=temporary, env=env,
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=10)
        if result.returncode == 0 or b'Frontend ausente' not in result.stderr:
            raise RuntimeError('Frontend ausente nao interrompeu inicializacao')
    print('OK: startup, health, HTML/JS/CSS, imagens, SPA e deteccao de frontend ausente/divergente.')


if __name__ == '__main__':
    main()

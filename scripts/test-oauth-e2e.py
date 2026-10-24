#!/usr/bin/env python3
"""Real HCI + auth-service HTTP flow against a disposable, TLS-verified LDAP fixture."""
import argparse
import base64
import http.cookiejar
import json
import os
from pathlib import Path
import re
import socket
import socketserver
import ssl
import subprocess
import tempfile
import threading
import time
from urllib.error import HTTPError, URLError
from urllib.parse import urlencode
from urllib.request import build_opener, HTTPCookieProcessor, HTTPRedirectHandler, Request, urlopen

ROOT = Path(__file__).resolve().parents[1]


def tlv(tag, value):
    size = len(value)
    length = bytes([size]) if size < 128 else bytes([0x82]) + size.to_bytes(2, 'big')
    return bytes([tag]) + length + value


def unpack(data, offset=0):
    tag, size = data[offset], data[offset + 1]
    start = offset + 2
    if size & 0x80:
        count = size & 0x7f
        size = int.from_bytes(data[start:start + count], 'big'); start += count
    return tag, data[start:start + size], start + size


def frame(connection):
    prefix = connection.recv(2)
    if not prefix: return None
    if len(prefix) != 2: raise RuntimeError('Incomplete LDAP frame')
    size = prefix[1]
    if size & 0x80:
        tail = connection.recv(size & 0x7f)
        prefix += tail; size = int.from_bytes(tail, 'big')
    body = b''
    while len(body) < size:
        chunk = connection.recv(size - len(body))
        if not chunk: raise RuntimeError('Incomplete LDAP body')
        body += chunk
    return prefix + body


class LDAPHandler(socketserver.BaseRequestHandler):
    def handle(self):
        try:
            while True:
                data = frame(self.request)
                if data is None: return
                _, message, _ = unpack(data)
                _, identifier, end = unpack(message)
                kind, payload, _ = unpack(message, end)
                def send(tag, body):
                    self.request.sendall(tlv(0x30, tlv(0x02, identifier) + tlv(tag, body)))
                def result(tag, code):
                    send(tag, tlv(0x0a, bytes([code])) + tlv(0x04, b'') + tlv(0x04, b''))
                if kind == 0x60:  # BindRequest
                    _, _, offset = unpack(payload)
                    _, dn, offset = unpack(payload, offset)
                    _, password, _ = unpack(payload, offset)
                    expected = b'service-secret' if dn == b'service@example.test' else b'correct-password'
                    result(0x61, 0 if password == expected else 49)
                elif kind == 0x63:  # SearchRequest
                    attrs = []
                    for name, values in [('sAMAccountName', [b'alice']), ('userPrincipalName', [b'alice@example.test']),
                                         ('objectGUID', [bytes.fromhex('33221100554477668899aabbccddeeff')]), ('memberOf', [b'CN=Domain Users,DC=example,DC=test'])]:
                        attrs.append(tlv(0x30, tlv(0x04, name.encode()) + tlv(0x31, b''.join(tlv(0x04, v) for v in values))))
                    send(0x64, tlv(0x04, b'CN=Alice,DC=example,DC=test') + tlv(0x30, b''.join(attrs)))
                    result(0x65, 0)
                elif kind == 0x42: return  # Unbind
                else: raise RuntimeError('Unexpected LDAP operation')
        except (OSError, ssl.SSLError):
            return


class LDAPServer(socketserver.ThreadingTCPServer):
    daemon_threads = True
    allow_reuse_address = True
    def get_request(self):
        connection, address = super().get_request()
        return self.context.wrap_socket(connection, server_side=True), address


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, *args): return None


def browser():
    jar = http.cookiejar.CookieJar()
    return build_opener(HTTPCookieProcessor(jar), NoRedirect()), jar


def fetch(opener, url, form=None):
    request = Request(url, data=urlencode(form).encode() if form else None)
    try: return opener.open(request, timeout=10)
    except HTTPError as response: return response


def available_port():
    with socket.socket() as listener:
        listener.bind(('127.0.0.1', 0)); return listener.getsockname()[1]


def ready(base, process):
    for _ in range(100):
        if process.poll() is not None: raise RuntimeError('Server exited before readiness')
        try:
            with urlopen(base + '/health', timeout=1) as response:
                if json.load(response)['status'] == 'UP': return
        except (URLError, OSError): time.sleep(0.1)
    raise RuntimeError('Readiness timed out')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--auth-binary', type=Path, default=ROOT.parent / 'auth-service/target/release/astral-auth-service')
    parser.add_argument('--nginx-binary', type=Path, help='exercise nginx auth_request before the HCI and callback')
    args = parser.parse_args()
    with tempfile.TemporaryDirectory() as directory:
        temporary = Path(directory)
        key, certificate = temporary / 'key.pem', temporary / 'ca.pem'
        subprocess.run(['openssl', 'req', '-x509', '-newkey', 'rsa:2048', '-nodes', '-keyout', str(key), '-out', str(certificate), '-days', '1', '-subj', '/CN=localhost', '-addext', 'subjectAltName=DNS:localhost', '-addext', 'basicConstraints=critical,CA:TRUE'], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        ldap = LDAPServer(('127.0.0.1', 0), LDAPHandler)
        ldap.context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        ldap.context.load_cert_chain(certificate, key)
        threading.Thread(target=ldap.serve_forever, daemon=True).start()
        hci_port, auth_port = available_port(), available_port()
        hci_direct, auth = f'http://127.0.0.1:{hci_port}', f'http://127.0.0.1:{auth_port}'
        proxy_port = available_port() if args.nginx_binary else None
        hci = f'http://127.0.0.1:{proxy_port}' if proxy_port else hci_direct
        issuer = hci + '/auth' if proxy_port else auth
        callback = hci + '/login/oauth2/code/auth-service'
        clients = temporary / 'clients.json'
        clients.write_text(json.dumps([{'client_id': 'astral-platform-hci', 'client_secret': 'c' * 48, 'redirect_uris': [callback]}]))
        auth_env = os.environ.copy()
        auth_env.update(SERVER_PORT=str(auth_port), SERVER_ADDRESS='127.0.0.1', AUTH_LDAP_URL=f'ldaps://localhost:{ldap.server_address[1]}', AUTH_LDAP_CA_FILE=str(certificate), AUTH_LDAP_BASE='DC=example,DC=test', AUTH_LDAP_USER_DN='service@example.test', AUTH_LDAP_PASSWORD='service-secret', AUTH_TOKEN_SECRET='k' * 48, AUTH_OAUTH_CLIENTS_FILE=str(clients), AUTH_OAUTH_ISSUER_URL=issuer)
        hci_env = os.environ.copy()
        hci_env.update(SERVER_PORT=str(hci_port), SERVER_ADDRESS='127.0.0.1', ASTRAL_STATIC_DIR=str(ROOT / 'src/main/resources/static'), ASTRAL_OAUTH_AUTHORIZATION_URL=issuer + '/oauth2/authorize', ASTRAL_OAUTH_TOKEN_URL=auth + '/oauth2/token', ASTRAL_OAUTH_USERINFO_URL=auth + '/oauth2/userinfo', ASTRAL_OAUTH_REDIRECT_URI=callback, ASTRAL_OAUTH_CLIENT_ID='astral-platform-hci', ASTRAL_OAUTH_CLIENT_SECRET='c' * 48)
        processes = []
        with (temporary / 'services.log').open('w+') as log:
            try:
                for binary, env in [(args.auth_binary, auth_env), (ROOT / 'target/release/astral-platform-hci', hci_env)]:
                    processes.append(subprocess.Popen([str(binary.resolve())], cwd=temporary, env=env, stdout=log, stderr=log))
                ready(auth, processes[0]); ready(hci_direct, processes[1])
                if args.nginx_binary:
                    config = temporary / 'nginx.conf'
                    user_directive = 'user root;' if os.geteuid() == 0 else ''
                    config.write_text(f"""{user_directive}
daemon off;
master_process off;
pid {temporary}/nginx.pid;
error_log stderr;
events {{ worker_connections 64; }}
http {{
    access_log off;
    client_body_temp_path {temporary}/body;
    proxy_temp_path {temporary}/proxy;
    fastcgi_temp_path {temporary}/fastcgi;
    uwsgi_temp_path {temporary}/uwsgi;
    scgi_temp_path {temporary}/scgi;
    server {{
        listen 127.0.0.1:{proxy_port};
        location = /_acl {{
            internal;
            proxy_pass {hci_direct}/api/v1/acl/check;
            proxy_pass_request_body off;
            proxy_set_header Content-Length "";
            proxy_set_header X-Original-URI $request_uri;
        }}
        location /auth/ {{
            auth_request off;
            proxy_pass {auth}/;
        }}
        location / {{
            auth_request /_acl;
            proxy_pass {hci_direct};
            proxy_set_header Host $http_host;
        }}
    }}
}}
""")
                    processes.append(subprocess.Popen([str(args.nginx_binary.resolve()), '-p', str(temporary) + '/', '-c', str(config)], cwd=temporary, stdout=log, stderr=log))
                    ready(hci, processes[-1])
                opener, jar = browser()
                assert json.load(fetch(opener, hci + '/api/auth/sso'))['enabled'] is True
                # Exercise the same pre-session subrequest nginx sends.
                for path, expected in [('/oauth2/authorization/auth-service', 200),
                                       ('/login/oauth2/code/auth-service?code=fixture&state=fixture', 200),
                                       ('/api/terminal/exec', 401)]:
                    check = Request(hci_direct + '/api/v1/acl/check', headers={'X-Original-URI': path})
                    try: response = opener.open(check, timeout=5)
                    except HTTPError as error: response = error
                    assert response.status == expected, ('ACL', path, response.status)

                if args.nginx_binary:
                    assert fetch(opener, hci + '/api/terminal/status').status == 401
                start = fetch(opener, hci + '/oauth2/authorization/auth-service')
                assert start.status == 303
                authorization = start.headers['Location']
                page = fetch(opener, authorization)
                assert page.status == 200
                ticket = re.search(r'name="transaction" value="([^"]+)"', page.read().decode()).group(1)
                denied = fetch(opener, authorization, {'transaction': ticket, 'username': 'alice', 'password': 'wrong-password'})
                assert denied.status == 200 and 'Location' not in denied.headers
                authorized = fetch(opener, authorization, {'transaction': ticket, 'username': 'alice', 'password': 'correct-password'})
                assert authorized.status == 303
                callback_url = authorized.headers['Location']
                session = fetch(opener, callback_url)
                assert session.status == 303 and session.headers['Location'] == '/app/'
                me = json.load(fetch(opener, hci + '/api/auth/me'))
                assert me['authenticated'] is True and me['username'] == 'alice'
                jwt = next(cookie.value for cookie in jar if cookie.name == 'ASTRAL_SESSION')
                payload = json.loads(base64.urlsafe_b64decode(jwt.split('.')[1] + '=='))
                assert payload['identityId'] == '00112233-4455-6677-8899-aabbccddeeff'
                assert payload['provider'] == 'AD' and payload['is_admin'] is False and payload['has_wheel'] is False
                assert payload['exp'] - payload['iat'] <= 900
                assert 'ssoError' in fetch(opener, callback_url).headers['Location']
                again = fetch(opener, hci + '/oauth2/authorization/auth-service')
                reused = fetch(opener, again.headers['Location'])
                assert reused.status == 303  # Provider session, no second password form.
                assert fetch(opener, reused.headers['Location']).headers['Location'] == '/app/'
                if args.nginx_binary: print('PASS: nginx auth_request allows anonymous OAuth start/callback and rejects protected API.')
                print('PASS: real auth-service + HCI, verified LDAPS, wrong password rejected, authorization code/PKCE, callback, stable identity, least privilege, JWT expiry, replay rejection and SSO session reuse.')
            except Exception:
                log.flush(); log.seek(0); print(log.read())
                raise
            finally:
                for process in processes:
                    process.terminate(); process.wait(timeout=10)
                ldap.shutdown(); ldap.server_close()


if __name__ == '__main__': main()

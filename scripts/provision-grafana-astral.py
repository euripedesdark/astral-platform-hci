#!/usr/bin/env python3
"""Provisiona recursos proprios, sem modificar configuracoes globais ou recursos do ERP."""
import json
import os
from pathlib import Path
import urllib.error
import urllib.parse
import urllib.request

BASE = os.environ['ASTRAL_GRAFANA_URL'].rstrip('/')
url = urllib.parse.urlsplit(BASE)
if url.scheme not in ('http', 'https') or not url.hostname or url.username or url.query or url.fragment:
    raise SystemExit('URL Grafana invalida')
TOKEN = os.environ['ASTRAL_GRAFANA_SERVICE_TOKEN']


def api(method, path, data=None, missing=False):
    body = None if data is None else json.dumps(data).encode()
    request = urllib.request.Request(BASE + '/api' + path, data=body, method=method,
        headers={'Authorization': 'Bearer ' + TOKEN, 'Content-Type': 'application/json'})
    # Impede que um redirect de login/reverse-proxy encaminhe o token a outra origem.
    class NoRedirect(urllib.request.HTTPRedirectHandler):
        def redirect_request(self, *args, **kwargs):
            return None
    try:
        with urllib.request.build_opener(NoRedirect).open(request, timeout=30) as response:
            return json.load(response)
    except urllib.error.HTTPError as error:
        if missing and error.code == 404:
            return None
        raise SystemExit(f'Grafana {method} {path}: HTTP {error.code}; nenhum recurso do ERP foi alterado') from None


def main():
    folder = api('GET', '/folders/astral-observability', missing=True)
    if folder is None:
        folder = api('POST', '/folders', {'uid': 'astral-observability', 'title': 'Astral'})
    if folder.get('title') != 'Astral':
        raise SystemExit('UID da pasta ocupado por outro recurso; abortando')
    datasource = {'uid': 'astral-postgres', 'name': 'Astral PostgreSQL', 'type': 'postgres',
        'access': 'proxy', 'url': os.environ.get('ASTRAL_GRAFANA_DB_HOST', '127.0.0.1:5432'),
        'user': 'astral_grafana_reader', 'isDefault': False,
        'jsonData': {'database': os.environ.get('ASTRAL_DB_NAME', 'astral'),
            'sslmode': os.environ.get('ASTRAL_GRAFANA_DB_SSLMODE', 'disable'), 'maxOpenConns': 5},
        'secureJsonData': {'password': os.environ['ASTRAL_GRAFANA_READER_PASSWORD']}}
    existing = api('GET', '/datasources/uid/astral-postgres', missing=True)
    if existing and (existing.get('name') != datasource['name'] or existing.get('user') != datasource['user']):
        raise SystemExit('UID do datasource ocupado por outro recurso; abortando')
    if existing and existing.get('isDefault'):
        raise SystemExit('Datasource existente e default; nao modificar automaticamente')
    previous = api('GET', '/dashboards/uid/astral-observability', missing=True)
    if previous and previous.get('meta', {}).get('folderUid') != 'astral-observability':
        raise SystemExit('UID de dashboard ocupado fora da pasta Astral; abortando')
    api('PUT' if existing else 'POST', '/datasources/uid/astral-postgres' if existing else '/datasources', datasource)
    dashboard = json.loads((Path(__file__).resolve().parents[1] / 'observability/grafana/astral.json').read_text())
    if previous:
        dashboard['id'] = previous['dashboard']['id']
        dashboard['version'] = previous['dashboard']['version']
    api('POST', '/dashboards/db', {'dashboard': dashboard, 'folderUid': 'astral-observability',
        'overwrite': False, 'message': 'Atualiza somente dashboard Astral'})
    print('Dashboard Astral: ' + BASE + '/d/astral-observability')
    print('OAuth, root_url, metadados, dashboards ERP e servico Grafana preservados.')


if __name__ == '__main__':
    main()

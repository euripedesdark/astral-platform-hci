#!/usr/bin/env bash
# Grafana :3000; PostgreSQL existente, mesmo banco astral, roles separadas.
set -Eeuo pipefail
[[ $EUID -eq 0 ]] || { echo 'Execute com sudo.' >&2; exit 1; }
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DB_NAME="${ASTRAL_DB_NAME:-astral}"
[[ "$DB_NAME" =~ ^[a-zA-Z_][a-zA-Z0-9_]*$ ]] || { echo 'ASTRAL_DB_NAME invalido.' >&2; exit 1; }
command -v psql >/dev/null || { echo 'PostgreSQL/psql existente necessario.' >&2; exit 1; }
# Nao instala outro PostgreSQL, MongoDB nem coletor Graylog.
if ! id grafana >/dev/null 2>&1; then
    if command -v apt-get >/dev/null; then
        apt-get update
        apt-get install -y ca-certificates curl gnupg
        install -d -m 0755 /etc/apt/keyrings
        curl -fsSL https://apt.grafana.com/gpg.key | gpg --dearmor --yes -o /etc/apt/keyrings/grafana.gpg
        echo 'deb [signed-by=/etc/apt/keyrings/grafana.gpg] https://apt.grafana.com stable main' > /etc/apt/sources.list.d/grafana.list
        apt-get update
        apt-get install -y grafana
    elif command -v dnf >/dev/null; then
        cat > /etc/yum.repos.d/grafana.repo <<'REPO'
[grafana]
name=Grafana
baseurl=https://rpm.grafana.com
repo_gpgcheck=1
enabled=1
gpgcheck=1
gpgkey=https://rpm.grafana.com/gpg.key
sslverify=1
REPO
        dnf install -y grafana
    else
        echo 'Instale o pacote oficial grafana e execute novamente.' >&2
        exit 1
    fi
fi
runuser -u postgres -- psql -X -v ON_ERROR_STOP=1 -d "$DB_NAME" -c 'SELECT 1 FROM public.observability_audit_event LIMIT 0' >/dev/null
install -d -m 0750 -o root -g grafana /etc/grafana/astral
SECRETS=/etc/grafana/astral/database.env
if [[ ! -f "$SECRETS" ]]; then
    umask 077
    printf 'GF_DATABASE_PASSWORD=%s\nASTRAL_GRAFANA_READER_PASSWORD=%s\n' "$(openssl rand -hex 32)" "$(openssl rand -hex 32)" > "$SECRETS"
fi
# Arquivo gerado por este script, apenas root pode alterar.
source "$SECRETS"
if [[ -z "${GF_SECURITY_ADMIN_PASSWORD:-}" ]]; then
    printf 'GF_SECURITY_ADMIN_PASSWORD=%s\n' "$(openssl rand -hex 32)" >> "$SECRETS"
fi
chmod 0600 "$SECRETS"
runuser -u postgres -- psql -X -v ON_ERROR_STOP=1 -d "$DB_NAME" \
    -v metadata_password="$GF_DATABASE_PASSWORD" -v reader_password="$ASTRAL_GRAFANA_READER_PASSWORD" <<'SQL'
SELECT 'CREATE ROLE astral_grafana LOGIN' WHERE NOT EXISTS (SELECT FROM pg_roles WHERE rolname='astral_grafana') \gexec
SELECT 'CREATE ROLE astral_grafana_reader LOGIN' WHERE NOT EXISTS (SELECT FROM pg_roles WHERE rolname='astral_grafana_reader') \gexec
ALTER ROLE astral_grafana PASSWORD :'metadata_password';
ALTER ROLE astral_grafana_reader PASSWORD :'reader_password';
CREATE SCHEMA IF NOT EXISTS grafana AUTHORIZATION astral_grafana;
ALTER ROLE astral_grafana SET search_path = grafana;
ALTER ROLE astral_grafana_reader SET default_transaction_read_only = on;
GRANT USAGE ON SCHEMA public TO astral_grafana_reader;
GRANT SELECT ON public.observability_audit_event, public.metric_cpu, public.metric_memory,
    public.metric_io, public.metric_network TO astral_grafana_reader;
SQL
install -d /etc/systemd/system/grafana-server.service.d
cat > /etc/systemd/system/grafana-server.service.d/astral.conf <<CFG
[Service]
EnvironmentFile=$SECRETS
Environment=GF_SERVER_HTTP_PORT=3000
Environment=GF_SECURITY_ALLOW_EMBEDDING=true
Environment=GF_AUTH_ANONYMOUS_ENABLED=false
Environment=GF_AUTH_GENERIC_OAUTH_ENABLED=false
Environment=GF_SERVER_SERVE_FROM_SUB_PATH=true
Environment=GF_DATABASE_TYPE=postgres
Environment=GF_DATABASE_HOST=127.0.0.1:5432
Environment=GF_DATABASE_NAME=$DB_NAME
Environment=GF_DATABASE_USER=astral_grafana
Environment=GF_DATABASE_SSL_MODE=disable
Environment=ASTRAL_GRAFANA_DB_NAME=$DB_NAME
CFG
install -d -o root -g grafana -m 0750 /etc/grafana/provisioning/datasources /etc/grafana/provisioning/dashboards /var/lib/grafana/dashboards
install -m 0640 -o root -g grafana "$ROOT/observability/grafana/datasource.yaml" /etc/grafana/provisioning/datasources/astral.yaml
install -m 0640 -o root -g grafana "$ROOT/observability/grafana/provider.yaml" /etc/grafana/provisioning/dashboards/astral.yaml
install -m 0640 -o root -g grafana "$ROOT/observability/grafana/astral.json" /var/lib/grafana/dashboards/astral.json
if [[ ! -f /etc/grafana/astral/sso.env ]]; then
    bash "$ROOT/scripts/configure-grafana-sso.sh"
fi
systemctl daemon-reload
systemctl enable grafana-server
systemctl restart grafana-server
for attempt in {1..30}; do
    if curl -fsS http://127.0.0.1:3000/grafana/api/health >/dev/null; then
        echo '[OK] Grafana :3000; metadados no schema grafana; datasource somente leitura no mesmo PostgreSQL.'
        exit 0
    fi
    sleep 2
done
echo '[ERRO] Grafana nao respondeu; consulte journalctl -u grafana-server.' >&2
exit 1

#!/usr/bin/env bash
# Adiciona apenas os recursos Astral ao Grafana existente; nao instala/reinicia o servico.
set -Eeuo pipefail
[[ $EUID -eq 0 ]] || { echo 'Execute com sudo.' >&2; exit 1; }
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
: "${ASTRAL_GRAFANA_URL:?URL publica existente do Grafana, incluindo seu prefixo atual}"
: "${ASTRAL_GRAFANA_SERVICE_TOKEN:?Token administrativo de provisionamento no Grafana existente}"
DB_NAME="${ASTRAL_DB_NAME:-astral}"
[[ "$DB_NAME" =~ ^[a-zA-Z_][a-zA-Z0-9_]*$ ]] || exit 1
command -v python3 >/dev/null
runuser -u postgres -- psql -X -v ON_ERROR_STOP=1 -d "$DB_NAME" -c 'SELECT 1 FROM public.observability_audit_event LIMIT 0' >/dev/null
install -d -m 0700 /etc/astral/grafana
SECRETS=/etc/astral/grafana/reader.env
umask 077
if [[ ! -f "$SECRETS" ]]; then
 printf 'ASTRAL_GRAFANA_READER_PASSWORD=%s\n' "$(openssl rand -hex 32)" > "$SECRETS"
fi
source "$SECRETS"
export ASTRAL_GRAFANA_READER_PASSWORD
# Somente a role do datasource Astral; nunca a role nem o banco de metadados Grafana/ERP.
runuser -u postgres -- psql -X -v ON_ERROR_STOP=1 -d "$DB_NAME" -v password="$ASTRAL_GRAFANA_READER_PASSWORD" <<'SQL'
SELECT 'CREATE ROLE astral_grafana_reader LOGIN' WHERE NOT EXISTS (SELECT FROM pg_roles WHERE rolname='astral_grafana_reader') \gexec
ALTER ROLE astral_grafana_reader PASSWORD :'password';
ALTER ROLE astral_grafana_reader SET default_transaction_read_only = on;
GRANT USAGE ON SCHEMA public TO astral_grafana_reader;
GRANT SELECT ON public.observability_audit_event, public.metric_cpu, public.metric_memory,
 public.metric_io, public.metric_network TO astral_grafana_reader;
SQL
export ASTRAL_GRAFANA_URL ASTRAL_GRAFANA_SERVICE_TOKEN
export ASTRAL_DB_NAME="$DB_NAME"
python3 "$ROOT/scripts/provision-grafana-astral.py"

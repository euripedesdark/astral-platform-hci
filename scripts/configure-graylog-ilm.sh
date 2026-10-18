#!/usr/bin/env bash
# Compatibilidade com comandos antigos: observabilidade agora e Grafana/PostgreSQL.
set -Eeuo pipefail
exec bash "$(dirname "${BASH_SOURCE[0]}")/install-grafana.sh" "$@"

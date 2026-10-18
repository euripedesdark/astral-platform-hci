#!/usr/bin/env bash
# Configura somente o cliente OAuth2 do Astral contra o auth-service existente.
set -Eeuo pipefail
[[ $EUID -eq 0 ]] || { echo 'Execute com sudo.' >&2; exit 1; }
: "${ASTRAL_PUBLIC_URL:?Defina a URL publica do Astral, ex. https://astral.srvcloud.cloud}"
: "${ASTRAL_SSO_ISSUER:?Defina o issuer publico do auth-service}"
: "${ASTRAL_SSO_CLIENT_SECRET:?Defina o segredo do cliente registrado no auth-service}"
ASTRAL_PUBLIC_URL="${ASTRAL_PUBLIC_URL%/}"
ASTRAL_SSO_ISSUER="${ASTRAL_SSO_ISSUER%/}"
ASTRAL_SSO_CLIENT_ID="${ASTRAL_SSO_CLIENT_ID:-grafana}"
# URLs publicas sem query/fragment; valores nunca viram comandos.
for url in "$ASTRAL_PUBLIC_URL" "$ASTRAL_SSO_ISSUER"; do
    [[ "$url" =~ ^https?://[a-zA-Z0-9.:-]+$ ]] || { echo 'URL publica invalida (use somente origem).' >&2; exit 1; }
done
[[ "$ASTRAL_SSO_CLIENT_ID" =~ ^[a-zA-Z0-9_-]+$ ]] || { echo 'Client ID invalido.' >&2; exit 1; }
[[ "$ASTRAL_SSO_CLIENT_SECRET" != *$'\n'* && "$ASTRAL_SSO_CLIENT_SECRET" != *$'\r'* ]] || exit 1
install -d -m 0750 /etc/astral /etc/systemd/system/astral-platform.service.d
# Syntaxe compartilhada com EnvironmentFile; nao e executado como shell.
envline() {
    local value="$2"
    value="${value//\\/\\\\}"; value="${value//\"/\\\"}"
    printf '%s="%s"\n' "$1" "$value"
}
umask 077
{
 envline ASTRAL_SSO_ENABLED true
 envline ASTRAL_SSO_ISSUER "$ASTRAL_SSO_ISSUER"
 envline ASTRAL_SSO_CLIENT_ID "$ASTRAL_SSO_CLIENT_ID"
 envline ASTRAL_SSO_CLIENT_SECRET "$ASTRAL_SSO_CLIENT_SECRET"
 envline ASTRAL_COOKIE_SECURE "$([[ "$ASTRAL_PUBLIC_URL" == https:* ]] && echo true || echo false)"
} > /etc/astral/sso.env
cat > /etc/systemd/system/astral-platform.service.d/zz-sso.conf <<'CFG'
[Service]
EnvironmentFile=/etc/astral/sso.env
CFG
systemctl daemon-reload
printf '[OK] SSO configurado. Registre estes callbacks no auth-service antes de reiniciar:\n%s\n' \
 "$ASTRAL_PUBLIC_URL/login/oauth2/code/auth-service"
echo 'Depois reinicie somente astral-platform e aplique configure-nginx.sh.'

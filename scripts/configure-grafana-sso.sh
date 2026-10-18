#!/usr/bin/env bash
# Configura os dois clientes OAuth2 (Astral e Grafana) contra o auth-service existente.
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
install -d -m 0750 /etc/grafana/astral /etc/astral /etc/systemd/system/grafana-server.service.d /etc/systemd/system/astral-platform.service.d
# Syntaxe compartilhada com EnvironmentFile; nao e executado como shell.
envline() {
    local value="$2"
    value="${value//\\/\\\\}"; value="${value//\"/\\\"}"
    printf '%s="%s"\n' "$1" "$value"
}
umask 077
{
 envline GF_SERVER_ROOT_URL "$ASTRAL_PUBLIC_URL/grafana/"
 envline GF_SERVER_SERVE_FROM_SUB_PATH true
 envline GF_SECURITY_ALLOW_EMBEDDING true
 envline GF_SECURITY_COOKIE_SAMESITE lax
 envline GF_SECURITY_COOKIE_SECURE "$([[ "$ASTRAL_PUBLIC_URL" == https:* ]] && echo true || echo false)"
 envline GF_AUTH_ANONYMOUS_ENABLED false
 envline GF_AUTH_BASIC_ENABLED false
 envline GF_AUTH_DISABLE_LOGIN_FORM true
 envline GF_AUTH_GENERIC_OAUTH_ENABLED true
 envline GF_AUTH_GENERIC_OAUTH_NAME 'Auth Service'
 envline GF_AUTH_GENERIC_OAUTH_AUTO_LOGIN true
 envline GF_AUTH_GENERIC_OAUTH_ALLOW_SIGN_UP true
 envline GF_AUTH_GENERIC_OAUTH_CLIENT_ID "$ASTRAL_SSO_CLIENT_ID"
 envline GF_AUTH_GENERIC_OAUTH_CLIENT_SECRET "$ASTRAL_SSO_CLIENT_SECRET"
 envline GF_AUTH_GENERIC_OAUTH_SCOPES 'openid profile email groups'
 envline GF_AUTH_GENERIC_OAUTH_AUTH_URL "$ASTRAL_SSO_ISSUER/oauth2/authorize"
 envline GF_AUTH_GENERIC_OAUTH_TOKEN_URL "$ASTRAL_SSO_ISSUER/oauth2/token"
 envline GF_AUTH_GENERIC_OAUTH_API_URL "$ASTRAL_SSO_ISSUER/userinfo"
 envline GF_AUTH_GENERIC_OAUTH_USE_PKCE true
 envline GF_AUTH_GENERIC_OAUTH_LOGIN_ATTRIBUTE_PATH preferred_username
 envline GF_AUTH_GENERIC_OAUTH_EMAIL_ATTRIBUTE_PATH email
 envline GF_AUTH_GENERIC_OAUTH_ROLE_ATTRIBUTE_PATH "grafana_role == 'Editor' && 'Editor' || 'Viewer'"
 envline GF_AUTH_GENERIC_OAUTH_ALLOW_ASSIGN_GRAFANA_ADMIN false
 envline GF_AUTH_GENERIC_OAUTH_TLS_SKIP_VERIFY_INSECURE false
} > /etc/grafana/astral/sso.env
{
 envline ASTRAL_SSO_ENABLED true
 envline ASTRAL_SSO_ISSUER "$ASTRAL_SSO_ISSUER"
 envline ASTRAL_SSO_CLIENT_ID "$ASTRAL_SSO_CLIENT_ID"
 envline ASTRAL_SSO_CLIENT_SECRET "$ASTRAL_SSO_CLIENT_SECRET"
 envline ASTRAL_COOKIE_SECURE "$([[ "$ASTRAL_PUBLIC_URL" == https:* ]] && echo true || echo false)"
} > /etc/astral/sso.env
cat > /etc/systemd/system/grafana-server.service.d/zz-astral-sso.conf <<'CFG'
[Service]
EnvironmentFile=/etc/grafana/astral/sso.env
CFG
cat > /etc/systemd/system/astral-platform.service.d/zz-sso.conf <<'CFG'
[Service]
EnvironmentFile=/etc/astral/sso.env
CFG
systemctl daemon-reload
printf '[OK] SSO configurado. Registre estes callbacks no auth-service antes de reiniciar:\n%s\n%s\n' \
 "$ASTRAL_PUBLIC_URL/login/oauth2/code/auth-service" "$ASTRAL_PUBLIC_URL/grafana/login/generic_oauth"
echo 'Depois reinicie astral-platform e grafana-server e aplique configure-nginx.sh.'

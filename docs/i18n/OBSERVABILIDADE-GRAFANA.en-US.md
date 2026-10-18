# Astral on the existing BI Grafana

Astral reuses **the same Grafana instance on port 3000** already used by the BI
module of the Brasil SaaS ERP, including OAuth, public URL, cookies and the
existing metadata database. It does not install another Grafana and does not
reconfigure the existing one.

The Observability card opens an iframe in the same window with its own dashboard
`astral-observability` (audit, CPU and memory). The `astral-postgres` data
source queries Astral's PostgreSQL database, `astral` by default, using an
exclusive read-only role. The ERP BI dashboard, data source and data are
preserved; the Astral data source is never marked as default.

## Reusing Grafana

1. Deploy/restart Astral to apply the V101 audit migration.
2. Provide `ASTRAL_GRAFANA_URL` with the **public URL already used by BI**,
   including its current prefix (e.g. `http://192.168.2.10:3000/grafana`). Do not
   change Grafana's `root_url`, issuer or callback to accommodate Astral.
3. Provide `ASTRAL_GRAFANA_SERVICE_TOKEN` from a service account with permission
   to provision a data source and a dashboard in the correct organization. This
   token is used only by the script, never by the browser. Run on the PostgreSQL
   host: `sudo --preserve-env=ASTRAL_GRAFANA_URL,ASTRAL_GRAFANA_SERVICE_TOKEN bash scripts/install-grafana.sh`.
   For another database, also preserve `ASTRAL_DB_NAME`.
4. Configure `ASTRAL_GRAFANA_DASHBOARD_URL` in the **Astral** environment with
   the URL the script reported, ending in `/d/astral-observability`, and restart
   only Astral. The browser must be able to reach that URL; an HTTPS portal
   requires Grafana to also be published over HTTPS. Reuse the existing public
   access.

The script calls only folder, data source and dashboard APIs and adds a read role
to the Astral database. It does not write to `/etc/grafana`, does not install
packages, does not change Grafana's systemd unit, does not restart Grafana, does
not change OAuth, does not swap the metadata database and does not modify ERP
resources. Conflicting UIDs outside Astral resources cause an error. The Astral
role secret stays root-only in `/etc/astral/grafana/reader.env` and is reused on
re-runs.

The files `deploy/grafana`, `scripts/bi/instalar_grafana_bi.sh` and the ERP's
`GrafanaBiDashboard` component are the reference for the existing integration.
The main documentation still mentions LDAP in older files; the OAuth
configuration actually in use is preserved in full, without trying to infer or
replace it.

## Astral login in the auth-service

OIDC login is configured **only in Astral**. The
`configure-grafana-sso.sh` script needs `ASTRAL_PUBLIC_URL`, `ASTRAL_SSO_ISSUER`
and `ASTRAL_SSO_CLIENT_SECRET` (and optionally `ASTRAL_SSO_CLIENT_ID`). It
creates `/etc/astral/sso.env` and a drop-in for `astral-platform.service`
only.

When registering the client in the auth-service, preserve the existing callbacks
and add only
`https://ASTRAL-ORIGIN/login/oauth2/code/auth-service`. The currently
configurable client can be reused; no Grafana callback needs to change. The
configuration uses the real endpoints `/oauth2/authorize`, `/oauth2/token`,
`/oauth2/jwks` and `/userinfo`.

With SSO enabled, Astral's normal entry redirects to the provider and creates
its central session; the already-configured Grafana reuses that session. Astral's
local form does not create a session in the provider and is not equivalent to
SSO. The contingency local login remains at `/app/?local=1`. Logging out of
Astral ends only its local session. Provider groups preserve the ACL and only
the configured admin group grants Astral administration. The `ASTRAL_SESSION`
cookie avoids collision with the auth-service session.

The embedding/cookie rules already established for BI also need to accept
Astral's origin. Use the same site and scheme for the portal, Grafana and issuer
when the OAuth flow happens inside an iframe; third-party cookie blocking may
prevent SSO if they are on different sites. No browser exception or global
authentication change is applied automatically.

## Data

Audit via JPA in `public.observability_audit_event`, with a default retention of
30 days (`ASTRAL_AUDIT_RETENTION_DAYS`), hourly cleanup and a JSONL fallback in
`/var/log/astral/audit-fallback.jsonl`. It does not re-import JSONL automatically
nor migrate Graylog history. Astral data stays in Astral's own PostgreSQL; the
existing Grafana metadata database remains where it is.
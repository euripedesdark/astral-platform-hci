# Astral en el Grafana existente del BI

Astral reutiliza **la misma instancia de Grafana en el puerto 3000** que ya usa
el módulo BI del Brasil SaaS ERP, incluyendo OAuth, URL pública, cookies y la
base de datos de metadatos existente. No instala otro Grafana ni reconfigura el
existente.

La tarjeta Observabilidad abre un iframe en la misma ventana con su propio
dashboard `astral-observability` (auditoría, CPU y memoria). La fuente
`astral-postgres` consulta la base de datos PostgreSQL de Astral, `astral` por
defecto, usando un rol de solo lectura exclusivo. El dashboard, la fuente de
datos y los datos del BI del ERP se preservan; la fuente de datos de Astral
nunca se marca como default.

## Reutilizar Grafana

1. Despliega/reinicia Astral para aplicar la migración V101 de auditoría.
2. Proporciona `ASTRAL_GRAFANA_URL` con la **URL pública ya usada por el BI**,
   incluyendo su prefijo actual (p. ej. `http://192.168.2.10:3000/grafana`). No
   cambies `root_url`, issuer ni callback de Grafana para acomodar a Astral.
3. Proporciona `ASTRAL_GRAFANA_SERVICE_TOKEN` de una service account con permiso
   para provisionar fuente de datos y dashboard en la organización correcta. Ese
   token lo usa solo el script, nunca el navegador. Ejecuta en el host de
   PostgreSQL: `sudo --preserve-env=ASTRAL_GRAFANA_URL,ASTRAL_GRAFANA_SERVICE_TOKEN bash scripts/install-grafana.sh`.
   Para otra base, preserva también `ASTRAL_DB_NAME`.
4. Configura en el entorno **de Astral** `ASTRAL_GRAFANA_DASHBOARD_URL` con la
   URL que informó el script, terminando en `/d/astral-observability`, y
   reinicia solo Astral. El navegador debe poder alcanzar esa URL; un portal
   HTTPS exige que Grafana también se publique por HTTPS. Reutiliza el acceso
   público existente.

El script llama solo a las APIs de carpetas, fuentes de datos y dashboards y
añade un rol de lectura en la base de Astral. No escribe en `/etc/grafana`, no
instala paquetes, no cambia la unidad systemd de Grafana, no reinicia Grafana,
no cambia OAuth, no cambia la base de metadatos y no modifica recursos del ERP.
UIDs conflictivos fuera de los recursos de Astral causan error. El secreto del
rol de Astral queda root-only en `/etc/astral/grafana/reader.env` y se reutiliza
en las reejecuciones.

Los archivos `deploy/grafana`, `scripts/bi/instalar_grafana_bi.sh` y el
componente `GrafanaBiDashboard` del ERP son la referencia de la integración
existente. La documentación principal aún menciona LDAP en archivos antiguos; la
configuración OAuth en ejecución se preserva íntegramente, sin intentar
deducirla ni sustituirla.

## Login de Astral en el auth-service

El login OIDC se configura **solo en Astral**. El script
`configure-grafana-sso.sh` necesita `ASTRAL_PUBLIC_URL`, `ASTRAL_SSO_ISSUER` y
`ASTRAL_SSO_CLIENT_SECRET` (y opcionalmente `ASTRAL_SSO_CLIENT_ID`). Crea
`/etc/astral/sso.env` y un drop-in solo de `astral-platform.service`.

Al registrar el cliente en el auth-service, preserva los callbacks existentes y
añade solo `https://ORIGEN-DE-ASTRAL/login/oauth2/code/auth-service`. El cliente
configurable actual puede reutilizarse; ningún callback de Grafana necesita
cambiar. La configuración usa los endpoints reales `/oauth2/authorize`,
`/oauth2/token`, `/oauth2/jwks` y `/userinfo`.

Con SSO habilitado, la entrada normal de Astral redirige al proveedor y crea su
sesión central; el Grafana ya configurado reutiliza esa sesión. El formulario
local de Astral no crea sesión en el proveedor y no equivale a SSO. El login
local de contingencia sigue en `/app/?local=1`. El logout de Astral termina solo
su sesión local. Los grupos del proveedor preservan el ACL y solo el grupo admin
configurado concede administración de Astral. La cookie `ASTRAL_SESSION` evita
colisión con la sesión del auth-service.

Las reglas de embedding/cookies ya establecidas en el BI también deben aceptar
el origen de Astral. Usa el mismo site y esquema para el portal, Grafana e
issuer cuando el flujo OAuth ocurra en un iframe; el bloqueo de cookies de
terceros puede impedir SSO si están en sites diferentes. No se aplica
automáticamente ninguna excepción de navegador ni cambio global de
autenticación.

## Datos

Auditoría vía JPA en `public.observability_audit_event`, con retención por
defecto de 30 días (`ASTRAL_AUDIT_RETENTION_DAYS`), limpieza horaria y fallback
JSONL en `/var/log/astral/audit-fallback.jsonl`. No reimporta JSONL
automáticamente ni migra historial de Graylog. Los datos de Astral permanecen en
el PostgreSQL propio de Astral; la base de metadatos de Grafana existente queda
donde está.
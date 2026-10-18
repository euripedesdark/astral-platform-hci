# Astral no Grafana existente do BI

O Astral reutiliza **a mesma instancia Grafana na porta 3000** ja usada pelo
modulo BI do Brasil SaaS ERP, inclusive OAuth, URL publica, cookies e banco de
metadados existentes. Nao instala outro Grafana e nao reconfigura o existente.

O card Observabilidade abre um iframe na mesma janela com o dashboard proprio
`astral-observability` (auditoria, CPU e memoria). A fonte `astral-postgres`
consulta o banco PostgreSQL do Astral, `astral` por padrao, usando uma role de
leitura exclusiva. O dashboard, datasource e dados do BI do ERP sao preservados;
o datasource do Astral nunca e marcado como default.

## Reutilizar o Grafana

1. Implante/reinicie Astral para aplicar a migracao V101 de auditoria.
2. Forneca `ASTRAL_GRAFANA_URL` com a **URL publica ja usada pelo BI**, incluindo
   seu prefixo atual (ex.: `http://192.168.2.10:3000/grafana`). Nao altere
   `root_url`, issuer nem callback do Grafana para acomodar o Astral.
3. Forneca `ASTRAL_GRAFANA_SERVICE_TOKEN` de uma service account com permissao
   para provisionar datasource e dashboard na organizacao correta. Esse token
   e usado apenas pelo script, nunca pelo navegador. Execute no host PostgreSQL:
   `sudo --preserve-env=ASTRAL_GRAFANA_URL,ASTRAL_GRAFANA_SERVICE_TOKEN bash scripts/install-grafana.sh`.
   Para outro banco, preserve tambem `ASTRAL_DB_NAME`.
4. Configure no ambiente **do Astral** `ASTRAL_GRAFANA_DASHBOARD_URL` com a URL
   que o script informou, terminando em `/d/astral-observability`, e reinicie
   somente Astral. O navegador precisa alcancar essa URL; um portal HTTPS exige
   Grafana tambem publicado por HTTPS. Reutilize o acesso publico existente.

O script chama somente APIs de pastas, datasources e dashboards e adiciona uma
role de leitura no banco Astral. Nao escreve em `/etc/grafana`, nao instala
pacotes, nao altera systemd do Grafana, nao reinicia Grafana, nao altera OAuth,
nao troca o banco de metadados e nao modifica recursos do ERP. UIDs conflitantes
fora dos recursos Astral causam erro. O segredo da role Astral fica root-only
em `/etc/astral/grafana/reader.env` e e reutilizado nas reexecucoes.

Os arquivos `deploy/grafana`, `scripts/bi/instalar_grafana_bi.sh` e o componente
`GrafanaBiDashboard` do ERP sao a referencia da integracao existente. A main
consultada ainda descreve LDAP em arquivos antigos; a configuracao OAuth em
execucao e preservada integralmente, sem tentar deduzi-la ou substitui-la.

## Login do Astral no auth-service

O login OIDC e configurado **somente no Astral**. O script
`configure-grafana-sso.sh` precisa de `ASTRAL_PUBLIC_URL`, `ASTRAL_SSO_ISSUER`
e `ASTRAL_SSO_CLIENT_SECRET` (e opcionalmente `ASTRAL_SSO_CLIENT_ID`). Ele cria
`/etc/astral/sso.env` e um drop-in apenas de `astral-platform.service`.

No registro do cliente do auth-service, preserve os callbacks existentes e
acrescente somente `https://ORIGEM-DO-ASTRAL/login/oauth2/code/auth-service`.
O cliente configuravel atual pode ser reutilizado; nenhum callback do Grafana
precisa mudar. A configuracao usa os endpoints reais `/oauth2/authorize`,
`/oauth2/token`, `/oauth2/jwks` e `/userinfo`.

Com SSO habilitado, a entrada normal do Astral redireciona ao provedor e cria
sua sessao central; o Grafana ja configurado reutiliza essa sessao. O formulario
local do Astral nao cria sessao no provedor e nao equivale a SSO. O login local
de contingencia continua em `/app/?local=1`. O logout do Astral encerra somente
sua sessao local. Grupos do provedor preservam o ACL e apenas o grupo admin
configurado concede administracao do Astral. Cookie `ASTRAL_SESSION` evita
colisao com a sessao do auth-service.

As regras de embedding/cookies ja estabelecidas no BI tambem precisam aceitar
a origem do Astral. Use o mesmo site e esquema para o portal, Grafana e issuer
quando o fluxo OAuth ocorrer no iframe; bloqueios de cookies de terceiros
podem impedir SSO se eles estiverem em sites diferentes. Nenhuma excecao de
browser ou mudanca global de autenticacao e aplicada automaticamente.

## Dados

Auditoria via JPA em `public.observability_audit_event`, com retencao padrao de
30 dias (`ASTRAL_AUDIT_RETENTION_DAYS`), limpeza horaria e fallback JSONL em
`/var/log/astral/audit-fallback.jsonl`. Nao reimporta JSONL automaticamente nem
migra historico Graylog. Os dados Astral continuam no mesmo PostgreSQL do Astral;
o banco de metadados Grafana existente permanece onde esta.

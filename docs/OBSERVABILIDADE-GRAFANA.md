# Observabilidade do Astral: Grafana, PostgreSQL e OAuth2/OIDC

O card **Observabilidade** da tela inicial abre o dashboard `astral-observability`
em iframe, dentro do Astral. O dashboard inicial provisionado inclui auditoria,
CPU e memoria. O Grafana escuta em **3000/tcp**; o navegador acessa
`/grafana/` pelo Nginx HTTPS do Astral, evitando conteudo misto e mantendo os
cookies do Grafana na mesma origem do portal.

## Login central e SSO

O Astral agora e cliente OIDC do auth-service, usando authorization code,
validacao de assinatura RSA, issuer, state e nonce pelo Spring Security.
O login ocorre por redirecionamento do navegador ao provedor: essa e a etapa
que estabelece a sessao central. Autenticar apenas no formulario local do
Astral nao cria essa sessao e nao equivale a SSO.

Com SSO habilitado, a entrada normal do Astral redireciona ao Auth Service;
se ja houver sessao central (inclusive iniciada por outro cliente OAuth), ela
sera reutilizada. Ao abrir a observabilidade, o Grafana faz seu proprio fluxo
OAuth e reutiliza a mesma sessao central, sem pedir novamente usuario/senha.
Cada aplicativo mantem seu proprio cookie; senhas e tokens nao sao passados ao
iframe. Grupos do provedor continuam disponiveis ao ACL; apenas o grupo
configurado em `astral.auth.ad.admin-group` concede administracao do Astral.

O issuer publico deve ser acessivel tanto pelo navegador quanto pelos dois
backends. Use **mesmo site e mesmo esquema** para Astral e auth-service
(ex.: `https://astral.srvcloud.cloud` e `https://auth.srvcloud.cloud`), pois o
fluxo do Grafana ocorre dentro do iframe e depende do cookie do provedor.
Misturar HTTPS no portal com issuer HTTP, ou usar um provedor em outro site,
pode bloquear o SSO no iframe pelas regras de cookies do navegador.
Certificados do issuer precisam ser confiaveis pelos backends.

## Configurar o auth-service existente

O auth-service atual registra um cliente configuravel (`grafana` por padrao).
A integracao reutiliza esse registro para os dois consumidores; use o mesmo
client id/secret configurado no auth-service. Preserve os callbacks do ERP e
acrescente os callbacks do Astral na lista `AUTH_OAUTH2_REDIRECT_URIS`:

- `https://astral.srvcloud.cloud/login/oauth2/code/auth-service`
- `https://astral.srvcloud.cloud/grafana/login/generic_oauth`

Mantenha `AUTH_OAUTH2_ENABLED=true` e `AUTH_OAUTH2_ISSUER` igual ao issuer
publico. O auth-service publica `/oauth2/authorize`, `/oauth2/token`,
`/oauth2/jwks` e **`/userinfo`**. Nao e necessario mudar seu codigo.
O login central e feito fora do iframe; a sessao existente permite que o
Grafana conclua a autorizacao sem exibir o formulario central.

## Atualizar o servidor

1. Registre os callbacks acima no auth-service, preservando os do ERP.
2. Recompile/implante o Astral e reinicie para o Flyway aplicar V101.
3. Execute o instalador com as variaveis `ASTRAL_PUBLIC_URL`,
   `ASTRAL_SSO_ISSUER` e `ASTRAL_SSO_CLIENT_SECRET` fornecidas pelo ambiente
   administrativo (nao as salve no Git):
   `sudo --preserve-env=ASTRAL_PUBLIC_URL,ASTRAL_SSO_ISSUER,ASTRAL_SSO_CLIENT_ID,ASTRAL_SSO_CLIENT_SECRET bash scripts/install-grafana.sh`.
   Para outro banco, forneca tambem `ASTRAL_DB_NAME`.
4. Aplique `sudo bash scripts/configure-nginx.sh` e reinicie
   `sudo systemctl restart astral-platform grafana-server`.
5. Entre no Astral pelo endereco publico e abra Observabilidade.

Para alterar somente SSO, use `scripts/configure-grafana-sso.sh` com as mesmas
variaveis. Ele configura os dois clientes e gera os drop-ins systemd, sem
reescrever o arquivo de ambiente principal. Os segredos ficam root-only;
o instalador reaproveita o arquivo de SSO nas reexecucoes. O login de contingencia
local continua em `/app/?local=1`; ele nao herda SSO. A saida do Astral encerra
apenas a sessao local, nao todas as sessoes do provedor e dos outros clientes.

## Dados e permissoes

Usa o **mesmo banco PostgreSQL do Astral** (`astral` por padrao), sem outro
PostgreSQL ou MongoDB. Os metadados Grafana ficam no schema `grafana`, role
`astral_grafana`. O datasource usa `astral_grafana_reader`, com SELECT apenas
nas tabelas de observabilidade. Conexoes locais em 127.0.0.1:5432 exigem senha
no pg_hba.conf. O instalador gera senha administrativa aleatoria antes de
iniciar o Grafana; login anonimo e formulario local do Grafana ficam desativados
quando o SSO e configurado. Papeis do provedor ficam limitados a Viewer/Editor,
sem conceder administrador global do Grafana.

A auditoria e gravada via JPA em `public.observability_audit_event`, com
retencao de 30 dias (`ASTRAL_AUDIT_RETENTION_DAYS`), limpeza horaria e fallback
JSONL em `/var/log/astral/audit-fallback.jsonl` se houver falha. O JSONL nao e
reimportado automaticamente. Eventos antigos do Graylog nao sao migrados.
Servicos antigos nao sao interrompidos, pois podem ser compartilhados.

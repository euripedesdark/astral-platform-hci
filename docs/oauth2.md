# OAuth2 do Astral HCI e auth-service

O HCI inicia Authorization Code com PKCE S256 no auth-service. O navegador recebe
apenas codigo temporario; o HCI troca esse codigo pelo access token no servidor,
busca o perfil no userinfo e cria cookies de sessao HttpOnly, Secure e SameSite=Lax.
O perfil preserva `provider` e `identityId` (GUID do AD), grupos e privilegios.
`state` aleatorio liga o callback ao navegador. Codigo, transacao e token expiram;
codigo e callback sao consumidos uma unica vez. Callback e origem sao cadastrados
explicitamente, sem derivar URLs do Host recebido.

## Implantar no dc-erp

```bash
cd "$HOME/astral/astral-platform-hci" &&
git pull --ff-only origin main &&
python3 configure_oauth2.py --public-url https://192.168.2.10 --install-deps
```

O comando prepara as ferramentas, atualiza/clona `../auth-service`, gera e
cadastra o segredo do cliente, compila/testa/implanta o auth-service e o HCI.
Se o checkout do auth-service estiver em outro local, acrescente
`--auth-source "$HOME/caminho/auth-service"`.

Requer o arquivo `/etc/auth-service/auth.env` usado pelo auth-service existente,
com `AUTH_LDAP_URL`, `AUTH_LDAP_BASE`, `AUTH_LDAP_USER_DN`, `AUTH_LDAP_PASSWORD`
e `AUTH_TOKEN_SECRET` (>=32 caracteres). Credenciais nao sao solicitadas nem
impressas pelo instalador. A CA e o hostname/SAN do LDAPS sao verificados antes
de parar servicos. Para CA privada fora do trust store, acrescente
`--ldap-ca-file /caminho/ca-raiz.pem`. A senha do AD continua sendo conferida por
bind LDAP, incluindo o codigo de resultado LDAP, e nao apenas sucesso de transporte.

O script preserva as configuracoes LDAP/JWT existentes, outros clientes OAuth,
nginx/TLS, banco e ERP. Instala a nova implementacao Rust do auth-service: faz
backup da unit anterior, identifica `auth-service.service`,
`astral-auth-service.service` ou `astral-auth.service`, e verifica health e metadata.
Em falha de ativacao, restaura a unit e release anteriores. Drop-ins que ainda
sobrescrevam ExecStart sao detectados e fazem o deploy retornar a versao anterior.
Se a implantacao do HCI falhar depois de atualizar o auth-service, o provedor
compativel e seu cadastro permanecem ativos; execute o mesmo comando para tentar
novamente. O deploy individual do HCI restaura sua propria release em caso de falha.

## Nginx existente

Mantem os caminhos que ja estavam configurados no servidor:

| Caminho publico | Destino |
|---|---|
| `/auth/oauth2/authorize` | `/oauth2/authorize` no auth-service, atraves do proxy `/auth/` existente |
| `/oauth2/authorization/auth-service` | Inicio do login no HCI `127.0.0.1:8082` |
| `/login/oauth2/code/auth-service` | Callback no HCI `127.0.0.1:8082` |
| `/app/` | Interface HCI |

O navegador deve abrir **https://192.168.2.10/app/** normalmente, sem `view-source:`.
Para login local sem iniciar SSO automaticamente: `/app/?local=1`.
Em erro OAuth2 o HCI retorna a `/app/?ssoError=1&local=1`, mantendo a tela acessivel.
`/auth` isoladamente nao e a URL de inicio de autorizacao.

Confirme que o proxy `/auth/` existente remove esse prefixo ao encaminhar para
o auth-service (`proxy_pass https://127.0.0.1:8443/;`, com a barra final) e que
a porta 8443 continua encaminhando para 8181. O instalador executa `nginx -t`
e preserva essas regras. As rotas `/oauth2/` e `/login/oauth2/` devem continuar
chegando ao HCI sem exigir uma sessao preexistente no auth_request.

## Configuracao gerada

- `/etc/auth-service/oauth2-clients.json`: cliente `astral-platform-hci`, segredo
  aleatorio e callbacks exatos; outros clientes sao preservados.
- `/etc/auth-service/oauth2.env`: issuer publico e caminho do cadastro.
- `/etc/astral/platform-oauth2.env`: URLs de autorizacao, token, userinfo, callback,
  client ID e segredo do HCI. Reutiliza o segredo JWT local existente ou gera um
  aleatorio quando ausente. A unit carrega esse arquivo opcional.

Os arquivos de segredos usam modo 0600. Executar novamente reutiliza o segredo
existente. O callback de outra origem so funciona apos seu cadastro pelo comando.
O backchannel padrao usa loopback 8181. Configuracoes remotas exigem HTTPS com
verificacao TLS; `ASTRAL_OAUTH_CA_FILE` permite adicionar uma CA PEM no HCI.

O auth-service oferece metadata RFC 8414 em
`/auth/.well-known/oauth-authorization-server`, grants `authorization_code`,
autenticacao de cliente `client_secret_post` e scopes `profile groups`.
Tokens de acesso sao opacos e validados pelo userinfo; nao existe ID token OIDC
nem refresh token. A sessao do provedor pode ser reutilizada durante oito horas;
access tokens duram quinze minutos e codigos, sessenta segundos. Os caches ficam
em memoria: reiniciar um servico exige reiniciar os logins que estavam em andamento.
O logout do HCI encerra a sessao local. `POST /auth/oauth2/logout` encerra tambem
a sessao do provedor naquele navegador.

## Validacao

```bash
python3 deploy_platform.py --check
sudo journalctl -u astral-platform-hci.service -n 100 --no-pager
curl -fsS http://127.0.0.1:8181/.well-known/oauth-authorization-server
```

Os testes automatizados usam provedores de identidade simulados para conferir o
protocolo, PKCE, CSRF, expiracao, replay, preservacao de identidade e falhas de
upstream. A autenticacao com seu AD real requer executar o comando no servidor e
fazer o login pelo navegador.

Teste entre os dois binarios compilados, com LDAP/TLS descartavel e nenhuma
alteracao no AD real:

```bash
python3 scripts/test-oauth-e2e.py
```

## 401 antes da tela do provedor

O nginx pode aplicar `auth_request` ao inicio do OAuth2. A ACL libera os caminhos
exatos `/oauth2/authorization/auth-service` e `/login/oauth2/code/auth-service`
mesmo sem cookie. O deploy agora verifica essas duas subrequisicoes antes de
considerar a release pronta. APIs administrativas continuam exigindo sessao.
Atualizar com `python3 deploy_platform.py` recompila e troca o backend preservando
o cadastro OAuth2 existente. Nao e necessario cadastrar novamente ou girar segredos.

O teste `scripts/test-oauth-e2e.py --nginx-binary /usr/sbin/nginx` tambem
executa o fluxo por um nginx isolado, aplicando `auth_request` ao inicio e
ao callback, e confere 401 para uma API protegida sem sessao.

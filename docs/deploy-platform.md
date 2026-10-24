# Deploy do Astral Platform HCI (Rust + React)

O backend atual usa Cargo/Axum; nao existe JAR a executar. Os antigos instaladores
Maven criavam `astral-platform.service`, por isso procurar
`astral-platform-hci.service` retornava unit not found. O deploy atual cria esse
nome oficial e mantem `astral-platform.service` como alias.

## Comando completo no servidor

```bash
cd "$HOME/astral/astral-platform-hci" &&
git pull --ff-only origin main &&
python3 deploy_platform.py --install-deps
```

O script solicita sudo, instala dependencias Debian/Ubuntu (compilador,
OpenSSL, pkg-config, Node/npm; instala Rust estavel via rustup se ausente),
confere ferramentas e systemd **antes de parar o servico**. Node.js >=18 e
Rust estavel sao necessarios; se a distribuicao fornecer Node antigo, atualize-o
antes de tentar novamente. Sem `--install-deps`, apenas confere dependencias.
Executado com sudo, compila como SUDO_USER, evitando arquivos root no checkout.

Depois para o HCI, executa `cargo test --locked`,
`cargo build --release --locked`, `npm ci` e `npm run build`, copia binario e
frontend para a mesma release, troca o link `current`, instala/habilita a unit,
inicia e verifica health, HTML e conteudo de todos os assets JS/CSS do index.
Aguarda ate 60 segundos. Falhas restauram unit/configuracao/link anteriores e
reiniciam o servico que estava ativo. Em primeira instalacao nao existe versao
anterior para recuperar. Releases sao mantidas em `/opt/astral-platform-hci/releases`;
copias das units/configuracao substituidas ficam em `history`, com modo 0600.

Preserva `/etc/astral/astral.env`, banco, AD, TLS, nginx e firewall. Nao altera
outros modulos nem configura um banco novo. A verificacao de inicializacao nao
valida login LDAP/Postgres: essas conexoes precisam das configuracoes existentes.
O nginx existente deve encaminhar para `127.0.0.1:8082`; o backend serve a SPA em
`/app/`. Nao e necessario copiar frontend para nginx nem reinicia-lo para cada build.
Em servidor novo, configure seu proxy/TLS separadamente.

O arquivo gerenciado `/etc/astral/platform-hci.env` define loopback, porta 8082 e
`ASTRAL_STATIC_DIR=/opt/astral-platform-hci/current/static`. Ele tem precedencia
sobre valores legados de porta/estaticos. `--port N` exige ajustar seu proxy para
N tambem; use `--port N` na verificacao. Nao altere esse arquivo durante o deploy.

```bash
sudo systemctl status astral-platform-hci.service --no-pager
sudo journalctl -u astral-platform-hci.service -n 100 --no-pager
python3 deploy_platform.py --check
python3 deploy_platform.py --plan
```

`install.py`, `uninstall-and-install.py` e `scripts/install-astral.sh` encaminham
para o mesmo deploy. O reinstalador nao apaga dados/configuracao. Scripts antigos
como `reconfigure.sh`, `installbase.sh` e `uninstall.py` nao fazem parte deste fluxo.
Nao execute reset/uninstall para corrigir unit ausente.

## Desenvolvimento

```bash
cd frontend && npm ci && npm run build
cd .. && cargo test --locked && cargo run --locked
```

O servidor exige `app/index.html` no diretorio estatico. `ASTRAL_STATIC_DIR` pode
apontar para outra raiz que contenha `app/index.html`; em desenvolvimento usa
`src/main/resources/static` quando disponivel. Endereco padrao: loopback:8082.

Teste HTTP da release compilada, sem systemd nem alteracoes no banco:

```bash
python3 scripts/smoke-platform.py
python3 -m unittest discover -s scripts/tests -v
```

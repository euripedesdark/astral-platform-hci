# Observabilidade: Grafana e PostgreSQL

Grafana escuta em **3000/tcp**. Auditoria e metricas usam o mesmo banco
PostgreSQL do Astral (`astral` por padrao), sem MongoDB ou GELF.
A auditoria e gravada assincronamente via JPA em
`public.observability_audit_event`; falhas de persistencia vao para
`/var/log/astral/audit-fallback.jsonl`. Esse arquivo nao e reimportado automaticamente.

## Atualizar o servidor

1. Recompile e implante o Astral normalmente; reinicie o servico para o Flyway
   aplicar V101 antes de configurar o Grafana.
2. Execute `sudo bash scripts/install-grafana.sh` no host do PostgreSQL.
   Para banco com outro nome, use `sudo ASTRAL_DB_NAME=nome bash scripts/install-grafana.sh`.
3. Execute `sudo bash scripts/configure-system-firewall.sh` para liberar 3000/tcp.
4. Acesse `http://IP-DO-SERVIDOR:3000/d/astral-observability` e configure o acesso
   administrativo do Grafana no primeiro login.

O instalador instala o pacote oficial Grafana em Debian/Ubuntu ou sistemas com
DNF. Reutiliza o PostgreSQL local: nao cria outro banco. Os metadados do Grafana
ficam no schema `grafana`, com role `astral_grafana`; a fonte de dados usa a role
`astral_grafana_reader`, com SELECT apenas nas tabelas de observabilidade.
Credenciais aleatorias ficam em arquivo root-only e sao reutilizadas em novas execucoes.
As conexoes sao locais (127.0.0.1:5432), com autenticacao por senha no pg_hba.conf.

O dashboard provisionado inclui auditoria, CPU e memoria. Eventos antigos do
Graylog nao sao migrados. Servicos antigos nao sao removidos nem interrompidos,
pois podem ser compartilhados; o Astral deixa de enviar GELF.
Os tres scripts antigos de instalacao/configuracao Graylog agora encaminham
para o instalador Grafana.

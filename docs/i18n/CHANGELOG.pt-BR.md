# Changelog

Todas as alterações notáveis do **Astral Platform & HCI** estão documentadas neste arquivo.

O formato é baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/),
e este projeto adere ao [Versionamento Semântico](https://semver.org/lang/pt-BR/).

## [1.4.0] — 2026-09-02

### Alterado
- Backend consolidado em **Java + Spring Boot** como única tecnologia de
  backend (UI, orquestração e módulos de infraestrutura como o Firewall).
- Núcleo em Python (Camada 2) descontinuado; toda validação, reconciliação e
  execução agora implementadas em Java.
- Frontend agora é JavaScript, HTML e CSS puros (sem framework de build; Node.js
  reservado para uso futuro, ainda não adotado).

### Adicionado
- Módulo de Firewall funcional (regras, NAT/port forwarding, masquerade, rate
  limiting, modo pânico) — promovido de "planejado" para "funcional".
- Instaladores automatizados: Sistema Base (`Installer.java`) e Módulo de
  Firewall (`InstallerFirewall.java`), idempotentes, processos Java autônomos.
- Reuso do Grafana de observabilidade na instância de BI existente (porta 3000),
  com dashboard dedicado `astral-observability` e data source somente leitura
  `astral-postgres`.

## [1.3.0] — 2026-08-27

### Adicionado
- Scripts Samba AD DC multi-distro (Fedora/AlmaLinux, Arch Linux, Debian 13),
  com e sem SSL, padronizados em `auth.env`.
- Instalador web unificado (`instalador.py` + `frontend/index.html`) com fluxo
  de 10 etapas, detecção automática de distro e criação da base `astral`.

## [1.2.0] — 2026-01-02

### Adicionado
- Classificação e seleção de DNS sensível ao contexto (provedor de nuvem, país,
  confiabilidade ≥ 0,95 + DNSSEC), integrada às Camadas 4 e 5.

## [1.0.0] — 2025-12-20

### Adicionado
- Documento de referência central e roadmap consolidado.
- Migração PostgreSQL e especificação Fedora 43.

## [0.1.0] — 2025-12-18

### Adicionado
- Conceitos iniciais e migração PostgreSQL.
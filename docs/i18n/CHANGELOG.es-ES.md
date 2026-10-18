# Changelog

Todos los cambios notables de **Astral Platform & HCI** están documentados en este archivo.

El formato se basa en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/),
y este proyecto adhiere al [Versionado Semántico](https://semver.org/lang/es/).

## [1.4.0] — 2026-09-02

### Cambiado
- Backend consolidado en **Java + Spring Boot** como única tecnología de backend
  (UI, orquestación y módulos de infraestructura como el Firewall).
- Núcleo en Python (Capa 2) descontinuado; toda validación, reconciliación y
  ejecución ahora implementada en Java.
- El frontend es ahora JavaScript, HTML y CSS puros (sin framework de build;
  Node.js reservado para uso futuro, aún no adoptado).

### Añadido
- Módulo de Firewall funcional (reglas, NAT/port forwarding, masquerade, rate
  limiting, modo pánico) — promovido de "planeado" a "funcional".
- Instaladores automatizados: Sistema Base (`Installer.java`) y Módulo de
  Firewall (`InstallerFirewall.java`), idempotentes, procesos Java autónomos.
- Reuso de Grafana de observabilidad en la instancia de BI existente (puerto 3000),
  con dashboard dedicado `astral-observability` y data source de solo lectura
  `astral-postgres`.

## [1.3.0] — 2026-08-27

### Añadido
- Scripts Samba AD DC multi-distro (Fedora/AlmaLinux, Arch Linux, Debian 13),
  con y sin SSL, estandarizados en `auth.env`.
- Instalador web unificado (`instalador.py` + `frontend/index.html`) con flujo
  de 10 etapas, detección automática de distro y creación de la base `astral`.

## [1.2.0] — 2026-01-02

### Añadido
- Clasificación y selección de DNS sensible al contexto (proveedor de nube, país,
  confiabilidad ≥ 0,95 + DNSSEC), integrada en las Capas 4 y 5.

## [1.0.0] — 2025-12-20

### Añadido
- Documento de referencia central y roadmap consolidado.
- Migración PostgreSQL y especificación Fedora 43.

## [0.1.0] — 2025-12-18

### Añadido
- Conceptos iniciales y migración PostgreSQL.
# Changelog

All notable changes to **Astral Platform & HCI** are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.4.0] — 2026-09-02

### Changed
- Consolidated the backend into **Java + Spring Boot** as the only backend
  technology (UI, orchestration and infrastructure modules such as the Firewall).
- Discontinued the previously planned Python core (Layer 2); all validation,
  reconciliation and execution logic now implemented in Java.
- Frontend is now pure JavaScript, HTML and CSS (no build framework; Node.js
  reserved for future use, not yet adopted).

### Added
- Functional Firewall module (rules, NAT/port forwarding, masquerade, rate
  limiting, panic mode) — promoted from "planned" to "functional".
- Automated installers: Base System (`Installer.java`) and Firewall Module
  (`InstallerFirewall.java`), idempotent, standalone Java processes.
- Grafana observability reuse on the existing BI instance (port 3000), with a
  dedicated `astral-observability` dashboard and read-only `astral-postgres`
  data source.

## [1.3.0] — 2026-08-27

### Added
- Multi-distro Samba AD DC scripts (Fedora/AlmaLinux, Arch Linux, Debian 13),
  with and without SSL, standardized on `auth.env`.
- Unified web installer (`instalador.py` + `frontend/index.html`) with a 10-step
  flow, automatic distro detection and `astral` database creation.

## [1.2.0] — 2026-01-02

### Added
- Context-aware sensitive DNS classification and selection (cloud provider,
  country, reliability ≥ 0.95 + DNSSEC), integrated into Layers 4 and 5.

## [1.0.0] — 2025-12-20

### Added
- Central reference document and consolidated roadmap.
- PostgreSQL migration and Fedora 43 specification.

## [0.1.0] — 2025-12-18

### Added
- Initial concepts and PostgreSQL migration.
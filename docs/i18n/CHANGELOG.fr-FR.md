# Changelog

Tous les changements notables de **Astral Platform & HCI** sont documentés dans ce fichier.

Le format est basé sur [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/),
et ce projet adhère au [Versionnement Sémantique](https://semver.org/lang/fr/).

## [1.4.0] — 2026-09-02

### Modifié
- Backend consolidé en **Java + Spring Boot** comme seule technologie backend
  (UI, orchestration et modules d'infrastructure comme le Pare-feu).
- Cœur en Python (Couche 2) abandonné ; toute validation, réconciliation et
  exécution désormais implémentée en Java.
- Le frontend est maintenant JavaScript, HTML et CSS purs (sans framework de
  build ; Node.js réservé à un usage futur, pas encore adopté).

### Ajouté
- Module Pare-feu fonctionnel (règles, NAT/port forwarding, masquerade, rate
  limiting, mode panique) — promu de "prévu" à "fonctionnel".
- Installateurs automatisés : Système de Base (`Installer.java`) et Module
  Pare-feu (`InstallerFirewall.java`), idempotents, processus Java autonomes.
- Réutilisation de Grafana d'observabilité sur l'instance BI existante (port 3000),
  avec tableau de bord dédié `astral-observability` et source de données en
  lecture seule `astral-postgres`.

## [1.3.0] — 2026-08-27

### Ajouté
- Scripts Samba AD DC multi-distro (Fedora/AlmaLinux, Arch Linux, Debian 13),
  avec et sans SSL, standardisés sur `auth.env`.
- Installateur web unifié (`instalador.py` + `frontend/index.html`) avec flux
  en 10 étapes, détection automatique de la distro et création de la base
  `astral`.

## [1.2.0] — 2026-01-02

### Ajouté
- Classification et sélection de DNS sensible au contexte (fournisseur de cloud,
  pays, fiabilité ≥ 0,95 + DNSSEC), intégrée dans les Couches 4 et 5.

## [1.0.0] — 2025-12-20

### Ajouté
- Document de référence central et roadmap consolidé.
- Migration PostgreSQL et spécification Fedora 43.

## [0.1.0] — 2025-12-18

### Ajouté
- Concepts initiaux et migration PostgreSQL.
# NOTICE

Astral Platform & HCI
Copyright (C) 2026 Astral Platform & HCI

Ce fichier satisfait la section 4(d) de la GNU Affero General Public License v3.0 :
il enregistre les avis de copyright des tiers et la licence de chaque composant
tiers avec lequel ce projet interagit.

Ce programme est un logiciel libre : vous pouvez le redistribuer et/ou le
modifier selon les termes de la AGPLv3, publiée par la Free Software
Foundation. Il n'y a aucune garantie. Le texte complet se trouve dans
[`LICENSE.md`](LICENSE.md).

---

## 📦 Aucun code tiers n'est redistribué dans ce dépôt

C'est un point important et souvent ignoré : **ce dépôt n'empaquette, ne vendorise
ni ne redistribue le code source d'aucun composant tiers.**

Les composants ci-dessous sont **installés sur le système d'exploitation par
`installbase.sh` et par les scripts de `scripts/`**, ou sont résolus par Maven et
npm au moment du build. Chacun reste sous sa propre licence, qui **n'est pas
remplacée** par ce projet, et chacun comporte son propre avis de copyright dans
le paquet ou dans le dépôt d'origine.

Par conséquent, la liste ci-dessous est une carte de dépendances — pas une
redistribution. Les avis originaux restent là où le titulaire les a placés, et
aucun n'a été supprimé.

---

## ☕ Dépendances d'exécution et de build

| Composant | Licence | Où utilisé | Où se trouve l'avis original |
|---|---|---|---|
| Spring Boot, Spring Framework, Spring Security | Apache-2.0 | `src/main/java/com/astral/main/` | dépôt et `META-INF` de chaque JAR |
| PostgreSQL JDBC Driver | BSD-2-Clause | `pom.xml` | dépôt du driver |
| UnboundID LDAP SDK for Java (6.x) | Apache-2.0 | `pom.xml`, authentification AD | [`docs.ldap.com`](https://docs.ldap.com/ldap-sdk/docs/) |
| React, React DOM | MIT | `frontend/src/` | `frontend/node_modules/*/LICENSE` |
| PrimeReact, PrimeIcons, PrimeFlex | MIT | `frontend/src/` | `frontend/node_modules/*/LICENSE` |
| Vite | MIT | `frontend/vite.config.js` | `frontend/node_modules/*/LICENSE` |
| Node.js | MIT | build du frontend | code source de Node.js |
| Maven | Apache-2.0 | build du backend | `META-INF` de Maven |
| Eclipse Temurin JDK | GPLv2 + Classpath Exception | CI et runtime | Adler OpenJDK |
| Flask, Flask-SQLAlchemy, Flask-CORS, Jinja2 | BSD-3-Clause | `fabric/acess-report-system/` | fichier de chaque paquet |
| APScheduler | MIT | `fabric/acess-report-system/` | fichier du paquet |
| psycopg2 | LGPL-3.0 | `fabric/acess-report-system/` | fichier du paquet |
| Samba (bindings Python et services AD DC) | GPL-3.0-or-later | `fabric/samba-ad-dc/` | projet Samba |
| Pi-hole | GPL-3.0-or-later | `fabric/DNS/` | projet Pi-hole |
| QEMU (émulateur dans son ensemble) | GPL-2.0-only | `installbase.sh` (phase 15/16) | `COPYING` de QEMU |
| KVM (module du noyau) | GPL-2.0-only | `installbase.sh` (phase 15/16) | licences du noyau Linux |
| libvirt | GPL-2.0-or-later AND LGPL-2.1 AND OFL-1.1 | `installbase.sh` (phase 15/16) | `COPYING.LIB` et `COPYING` de libvirt |
| Nginx | BSD-2-Clause | `scripts/configure-nginx.sh` | `LICENSE` de Nginx |
| Apache Traffic Server | Apache-2.0 | `scripts/configure-ats-auth.sh` | `LICENSE` de ATS |
| dnsmasq | GPL-2.0-or-later | `installbase.sh` | paquet de la distribution |

---

## ⚠️ Composants non compatibles avec la AGPLv3 — services séparés

Ces éléments sont des **services indépendants**, installés et licenciés séparément.
Astral ne fait qu'en consommer les données. Cette section existe pour que personne
ne confonde "Astral parle à Graylog" avec "Astral distribue Graylog".

| Service | Licence | Relation avec Astral |
|---|---|---|
| **Graylog (serveur)** | **SSPL-1.0** | Astral envoie uniquement des entrées de log. Le serveur Graylog **n'est pas** un logiciel libre sous AGPLv3 et **n'est pas** redistribué par ce projet. |
| Pi-hole, QEMU, KVM, libvirt, Samba | GPLv2-only / GPLv3 | Invoqués comme programmes indépendants, non liés. AGPLv3 est compatible avec GPLv2-only, GPLv3 et GPL-3.0-or-later. |

Si vous comptez **redistribuer** Graylog ou l'un des éléments ci-dessus avec
Astral, lisez la licence de ce composant d'abord. La licence d'Astral ne
s'applique pas à eux.

---

## 🔗 Marques

"Spring" et "Spring Boot" sont des marques de Broadcom Inc. (anciennement VMware, Inc.)
ou de ses affiliés. "PostgreSQL" est une marque du PostgreSQL Global Development
Group. "React" est une marque de Meta Platforms, Inc. et de ses affiliés.
"Apache" et "Apache Traffic Server" sont des marques de l'Apache Software
Foundation. "QEMU", "KVM" et "libvirt" sont des projets de la Linux Foundation.
"Samba" est une marque de Samba Team / Microsoft. "React Native", "PrimeReact"
et "Pi-hole" sont des marques de leurs détenteurs respectifs. "Dell", "Intel"
et "Broadcom" sont des marques de leurs propriétaires respectifs.

Ce projet n'est affilié à, cautionné ni sponsorisé par aucune de ces entreprises.
L'usage des noms ci-dessus sert uniquement à identifier les technologies.

---

© 2026 Astral Platform & HCI — **Créé par : Euripedes Batista de Paiva Junior**
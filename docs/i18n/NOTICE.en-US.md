# NOTICE

Astral Platform & HCI
Copyright (C) 2026 Astral Platform & HCI

This file satisfies section 4(d) of the GNU Affero General Public License v3.0:
it records the copyright notices of third parties and the license of each
third-party component with which this project interacts.

This program is free software: you can redistribute it and/or modify it under
the terms of the AGPLv3, published by the Free Software Foundation. There is
no warranty whatsoever. The full text is in [`LICENSE.md`](LICENSE.md).

---

## 📦 No third-party code is redistributed in this repository

This is an important point that is frequently ignored: **this repository does
not package, vendor, or redistribute the source code of any third-party
component.**

The components below are **installed on the operating system by `installbase.sh`
and by the scripts in `scripts/`**, or are resolved by Maven and npm at build
time. Each remains under its own license, which **is not replaced** by this
project, and each carries its own copyright notice in the package or in the
source repository.

Therefore, the list below is a dependency map — not a redistribution. The
original notices stay where the rights holder placed them, and none has been
removed.

---

## ☕ Runtime and build dependencies

| Component | License | Where used | Where the original notice is |
|---|---|---|---|
| Spring Boot, Spring Framework, Spring Security | Apache-2.0 | `src/main/java/com/astral/main/` | each JAR's repository and `META-INF` |
| PostgreSQL JDBC Driver | BSD-2-Clause | `pom.xml` | driver repository |
| UnboundID LDAP SDK for Java (6.x) | Apache-2.0 | `pom.xml`, AD authentication | [`docs.ldap.com`](https://docs.ldap.com/ldap-sdk/docs/) |
| React, React DOM | MIT | `frontend/src/` | `frontend/node_modules/*/LICENSE` |
| PrimeReact, PrimeIcons, PrimeFlex | MIT | `frontend/src/` | `frontend/node_modules/*/LICENSE` |
| Vite | MIT | `frontend/vite.config.js` | `frontend/node_modules/*/LICENSE` |
| Node.js | MIT | frontend build | Node.js source code |
| Maven | Apache-2.0 | backend build | Maven's `META-INF` |
| Eclipse Temurin JDK | GPLv2 + Classpath Exception | CI and runtime | Adler OpenJDK |
| Flask, Flask-SQLAlchemy, Flask-CORS, Jinja2 | BSD-3-Clause | `fabric/acess-report-system/` | each package's file |
| APScheduler | MIT | `fabric/acess-report-system/` | package file |
| psycopg2 | LGPL-3.0 | `fabric/acess-report-system/` | package file |
| Samba (Python bindings and AD DC services) | GPL-3.0-or-later | `fabric/samba-ad-dc/` | Samba project |
| Pi-hole | GPL-3.0-or-later | `fabric/DNS/` | Pi-hole project |
| QEMU (emulator as a whole) | GPL-2.0-only | `installbase.sh` (phase 15/16) | QEMU `COPYING` |
| KVM (kernel module) | GPL-2.0-only | `installbase.sh` (phase 15/16) | Linux kernel licenses |
| libvirt | GPL-2.0-or-later AND LGPL-2.1 AND OFL-1.1 | `installbase.sh` (phase 15/16) | libvirt `COPYING.LIB` and `COPYING` |
| Nginx | BSD-2-Clause | `scripts/configure-nginx.sh` | Nginx `LICENSE` |
| Apache Traffic Server | Apache-2.0 | `scripts/configure-ats-auth.sh` | ATS `LICENSE` |
| dnsmasq | GPL-2.0-or-later | `installbase.sh` | distribution package |

---

## ⚠️ Components not compatible with AGPLv3 — separate services

These items are **independent services**, installed and licensed separately.
Astral only consumes their data. This section exists so that nobody confuses
"Astral talks to Graylog" with "Astral distributes Graylog".

| Service | License | Relation to Astral |
|---|---|---|
| **Graylog (server)** | **SSPL-1.0** | Astral only sends log entries. The Graylog server is **not** free software under AGPLv3 and is **not** redistributed by this project. |
| Pi-hole, QEMU, KVM, libvirt, Samba | GPLv2-only / GPLv3 | Invoked as independent programs, not linked. AGPLv3 is compatible with GPLv2-only, GPLv3 and GPL-3.0-or-later. |

If you intend to **redistribute** Graylog or any of the items above together with
Astral, read that component's license first. Astral's license does not apply to
them.

---

## 🔗 Trademarks

"Spring" and "Spring Boot" are trademarks of Broadcom Inc. (formerly VMware, Inc.)
or its affiliates. "PostgreSQL" is a trademark of the PostgreSQL Global
Development Group. "React" is a trademark of Meta Platforms, Inc. and affiliates.
"Apache" and "Apache Traffic Server" are trademarks of the Apache Software
Foundation. "QEMU", "KVM" and "libvirt" are projects of the Linux Foundation.
"Samba" is a trademark of Samba Team / Microsoft. "React Native", "PrimeReact"
and "Pi-hole" are trademarks of their respective holders. "Dell", "Intel" and
"Broadcom" are trademarks of their respective owners.

This project is not affiliated with, endorsed by, or sponsored by any of these
companies. The use of the names above is only to identify the technologies.

---

© 2026 Astral Platform & HCI — **Created by: Euripedes Batista de Paiva Junior**
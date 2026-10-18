# NOTICE

Astral Platform & HCI
Copyright (C) 2026 Astral Platform & HCI

Este archivo satisface la sección 4(d) de la GNU Affero General Public License v3.0:
registra los avisos de copyright de terceros y la licencia de cada componente
de terceros con el que este proyecto interactúa.

Este programa es software libre: puedes redistribuirlo y modificarlo bajo los
términos de la AGPLv3, publicada por la Free Software Foundation. No hay
garantía alguna. El texto completo está en [`LICENSE.md`](LICENSE.md).

---

## 📦 No se redistribuye código de terceros en este repositorio

Este es un punto importante y frecuentemente ignorado: **este repositorio no
empaqueta, vendoriza ni redistribuye el código fuente de ningún componente de
terceros.**

Los componentes a continuación son **instalados en el sistema operativo por
`installbase.sh` y por los scripts de `scripts/`**, o se resuelven mediante
Maven y npm en el momento del build. Cada uno permanece bajo su propia licencia,
que **no es sustituida** por este proyecto, y cada uno trae su propio aviso de
copyright en el paquete o en el repositorio de origen.

Por lo tanto, la lista siguiente es un mapa de dependencias — no una
redistribución. Los avisos originales permanecen donde el titular los colocó, y
ninguno ha sido eliminado.

---

## ☕ Dependencias de ejecución y build

| Componente | Licencia | Dónde se usa | Dónde está el aviso original |
|---|---|---|---|
| Spring Boot, Spring Framework, Spring Security | Apache-2.0 | `src/main/java/com/astral/main/` | repositorio y `META-INF` de cada JAR |
| PostgreSQL JDBC Driver | BSD-2-Clause | `pom.xml` | repositorio del driver |
| UnboundID LDAP SDK for Java (6.x) | Apache-2.0 | `pom.xml`, autenticación AD | [`docs.ldap.com`](https://docs.ldap.com/ldap-sdk/docs/) |
| React, React DOM | MIT | `frontend/src/` | `frontend/node_modules/*/LICENSE` |
| PrimeReact, PrimeIcons, PrimeFlex | MIT | `frontend/src/` | `frontend/node_modules/*/LICENSE` |
| Vite | MIT | `frontend/vite.config.js` | `frontend/node_modules/*/LICENSE` |
| Node.js | MIT | build del frontend | código fuente de Node.js |
| Maven | Apache-2.0 | build del backend | `META-INF` de Maven |
| Eclipse Temurin JDK | GPLv2 + Classpath Exception | CI y runtime | Adler OpenJDK |
| Flask, Flask-SQLAlchemy, Flask-CORS, Jinja2 | BSD-3-Clause | `fabric/acess-report-system/` | archivo de cada paquete |
| APScheduler | MIT | `fabric/acess-report-system/` | archivo del paquete |
| psycopg2 | LGPL-3.0 | `fabric/acess-report-system/` | archivo del paquete |
| Samba (bindings Python y servicios AD DC) | GPL-3.0-or-later | `fabric/samba-ad-dc/` | proyecto Samba |
| Pi-hole | GPL-3.0-or-later | `fabric/DNS/` | proyecto Pi-hole |
| QEMU (emulador en su conjunto) | GPL-2.0-only | `installbase.sh` (fase 15/16) | `COPYING` de QEMU |
| KVM (módulo del kernel) | GPL-2.0-only | `installbase.sh` (fase 15/16) | licencias del kernel Linux |
| libvirt | GPL-2.0-or-later AND LGPL-2.1 AND OFL-1.1 | `installbase.sh` (fase 15/16) | `COPYING.LIB` y `COPYING` de libvirt |
| Nginx | BSD-2-Clause | `scripts/configure-nginx.sh` | `LICENSE` de Nginx |
| Apache Traffic Server | Apache-2.0 | `scripts/configure-ats-auth.sh` | `LICENSE` de ATS |
| dnsmasq | GPL-2.0-or-later | `installbase.sh` | paquete de la distribución |

---

## ⚠️ Componentes no compatibles con la AGPLv3 — servicios separados

Estos elementos son **servicios independientes**, instalados y licenciados por
fuera. Astral solo consume sus datos. Esta sección existe para que nadie
confunda "Astral habla con Graylog" con "Astral distribuye Graylog".

| Servicio | Licencia | Relación con Astral |
|---|---|---|
| **Graylog (servidor)** | **SSPL-1.0** | Astral solo envía entradas de log. El servidor Graylog **no** es software libre bajo AGPLv3 y **no** es redistribuido por este proyecto. |
| Pi-hole, QEMU, KVM, libvirt, Samba | GPLv2-only / GPLv3 | Invocados como programas independientes, no vinculados. AGPLv3 es compatible con GPLv2-only, GPLv3 y GPL-3.0-or-later. |

Si pretendes **redistribuir** Graylog o cualquiera de los elementos anteriores
junto con Astral, lee la licencia de ese componente primero. La licencia de
Astral no se aplica a ellos.

---

## 🔗 Marcas

"Spring" y "Spring Boot" son marcas de Broadcom Inc. (anteriormente VMware, Inc.)
o de sus afiliadas. "PostgreSQL" es marca del PostgreSQL Global Development
Group. "React" es marca de Meta Platforms, Inc. y afiliadas. "Apache" y
"Apache Traffic Server" son marcas de la Apache Software Foundation. "QEMU",
"KVM" y "libvirt" son proyectos de la Linux Foundation. "Samba" es marca de
Samba Team / Microsoft. "React Native", "PrimeReact" y "Pi-hole" son marcas de
sus respectivos titulares. "Dell", "Intel" y "Broadcom" son marcas de sus
respectivos dueños.

Este proyecto no está afiliado, respaldado ni patrocinado por ninguna de estas
empresas. El uso de los nombres anteriores es solo para identificar las
tecnologías.

---

© 2026 Astral Platform & HCI — **Creado por: Euripedes Batista de Paiva Junior**
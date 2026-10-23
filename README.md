# Astral Platform HCI

![Licença](https://img.shields.io/badge/License-AGPLv3-blue.svg)
![Java](https://img.shields.io/badge/Java-25-orange.svg)
![Spring Boot](https://img.shields.io/badge/Spring%20Boot-4.1.1-brightgreen.svg)
![React](https://img.shields.io/badge/React-18-61DAFB.svg)
![PrimeReact](https://img.shields.io/badge/PrimeReact-10-blueviolet.svg)

**Portal Central de Acesso e Hub de Navegação Astral** — Ponto único de entrada, autenticação centralizada e orquestração de acesso para todas as soluções do ecossistema Astral.

Este projeto está sob **GNU AGPL v3**. Ver [`LICENSE.md`](LICENSE.md).

---

## 🎯 Visão Geral e Papel no Ecossistema

O **Astral Platform HCI** atua como o portal inicial e hub de identidade do ecossistema. Ele centraliza as credenciais e fornece acesso direto com Single Sign-On (SSO) para os módulos especializados:

| Módulo Astral | Papel / Descrição | Porta Padrão |
|---|---|---|
| **Astral Internet Security** | Firewall stateful, regras iptables, NAT, Masquerade e modo pânico | `:8084` |
| **Astral Traffic Guard** | Proxy HTTP/HTTPS, motor de ACL e filtragem de conteúdo web | `:8085` / `:8091` |
| **Astral DBM** | Gerenciador web de bancos de dados estilo IBConsole/DBeaver | `:8080` (DBM) |
| **Astral Saas ERP** | Sistema de Gestão Empresarial (ERP) cloud-native e multiempresa | `:8080` |
| **Astral AD & DNS** | Controlador de domínio Samba 4 AD DC e resolução DNS com fallback | `:389` / `:636` / `:53` |
| **Astral Network** | Gestão declarativa de interfaces de rede e servidor DHCP | — |
| **Observabilidade** | Monitoramento de métricas, CPU, memória e auditoria no Grafana | `:3000` |

---

## 🔐 Autenticação Centralizada

O portal integra nativamente:
- **Active Directory / LDAP**: Validação corporativa segura via LDAPS (porta 636) contra o controlador Samba AD.
- **PostgreSQL Nativo**: Autenticação de contas técnicas administrativas.
- **PAM / Host Linux**: Suporte a administradores locais do sistema operacional.
- **SSO OIDC / OAuth2**: Integração com o Astral Auth Service (`:8181`).

---

## 🏗️ Estrutura do Projeto

```
astral-platform-hci/
├── pom.xml                                   # Configuração Maven (Java 25 + Spring Boot 4.1.1)
├── src/main/java/com/astral/main/            # Backend do Portal
│   ├── controller/                           # HomeController, LoginController, SpaForwardController
│   ├── security/                             # SecurityConfig, MultiSourceAuthenticationProvider, OIDC
│   └── AstralApplication.java                # Ponto de entrada da aplicação
├── src/main/resources/                       # Configurações application.yml e assets estáticos
└── frontend/                                 # Interface React + Vite + PrimeReact
    ├── src/modules/Login.tsx                 # Tela de Login com suporte a credenciais e SSO
    ├── src/modules/overview/                 # Hub Central com cards de acesso aos módulos
    └── src/App.tsx                           # Shell principal da aplicação
```

---

## 🚀 Execução e Build

### Requisitos
- JDK 25
- Maven 3.9+
- Node.js 20+

### Compilação do Frontend
```bash
cd frontend
npm install
npm run build
```

### Compilação e Empacotamento do Jar
```bash
mvn clean package -DskipTests
```
O artefato final é gerado em `target/astral-platform-hci-1.4.0.jar`.

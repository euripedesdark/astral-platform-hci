# Astral Platform HCI – Documento de Referência e Atualizações

**Documento Vivo – Versão 2.0**
**Última atualização:** 21 DE OUTUBRO DE 2026

---

## 📅 LINHA DO TEMPO DE ATUALIZAÇÕES

### **21 DE OUTUBRO DE 2026 – Versão 2.0 (Padronização e Desacoplamento Modular)**
**Separação Arquitetural do Ecossistema Astral**:
- Conclusão da separação do monólito em soluções autônomas especializadas, todas padronizadas sob o prefixo `astral-`:
  - 🌐 **`astral-platform-hci`**: Redefinido exclusivamente como **Portal de Acesso e Hub de Identidade Central**. Responsável pelo login unificado, autenticação multi-origem (AD/Postgres/PAM/SSO) e dashboard launcher.
  - 🚦 **`astral-traffic-guard`**: Isolamento completo do módulo de Proxy HTTP/HTTPS, motor de regras de navegação (ACL), integração com Apache Traffic Server (ATS) via authproxy socket (`:8091`), cache em Redis e RabbitMQ.
  - 🔥 **`astral-internet-security`**: Isolamento completo da plataforma de Firewall stateful, regras iptables, NAT, Masquerade, Port Forwarding, WebSocket de logs em tempo real e proteção de borda.
  - 🗄️ **`astral-dbm`**: Gerenciador web de bancos de dados estilo IBConsole/DBeaver.
  - 💼 **`astral-erp`**: ERP multiempresa brasileiro cloud-native (transição de nome do Brasil SaaS ERP).
  - 👥 **`astral-ad-dns`**: Controlador de domínio Samba 4 AD DC e resolução DNS com seleção contextual.
  - 🔌 **`astral-network`**: Gerenciamento declarativo de interfaces de rede e servidor DHCP (com NAT concentrado no firewall).
  - 🔑 **`astral-auth-service`**: Serviço centralizado de emissão de tokens e identidade.

---

### **02 DE SETEMBRO DE 2026 – Versão 1.4**
**Consolidação de stack**: **Backend unificado em Java + Spring Boot**
- Backend unificado em Java 21/25 + Spring Boot.
- Descontinuação do runtime Python do core.

---

### **02 DE JANEIRO DE 2026 – Versão 1.2**
**Seleção de DNS Sensível a Contexto**:
- Dados de servidores DNS locais via CSV do `public-dns.info`.
- Integração nas camadas de rede e firewall.

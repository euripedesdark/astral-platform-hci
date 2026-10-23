# Contribuindo para o Astral Platform & HCI

## Código de Conduta
Este projeto e todos os colaboradores são regidos pelo Código de Conduta.

## Configuração de Desenvolvimento

### Pré-requisitos
* Java 21+ (OpenJDK ou Temurin)
* Maven 3.9+
* Rust 1.80+ (cargo, rustc)
* Node.js 20+ & npm 10+
* PostgreSQL 15+

### Compilação e Execução

```bash
# Backend Rust
cargo test
cargo build --release

# Backend Java Spring Boot
mvn clean package -DskipTests

# Frontend React
cd frontend
npm install
npm run build
```

## Estrutura do Projeto

```
astral-platform-hci/
├── Cargo.toml                 # Módulo de alta performance Rust Axum Core
├── Cargo.lock                 # Versões travadas de dependências Rust
├── pom.xml                    # Build Maven Spring Boot 3.3.5 / Java 21
├── src/                       # Código-fonte Rust e Java integrado
│   ├── main.rs                # Ponto de entrada Axum (plano de controle HCI)
│   ├── config/                # Configurações de porta, banco e autenticação
│   ├── controller/            # Controladores REST da plataforma e rotas SPA
│   ├── domain/                # Modelos de domínio (Nós HCI, VMs, Storage, ERP, WebServer)
│   └── service/               # Orquestração de rede, roteamento e autenticação
├── src/main/java/             # Backend corporativo Spring Boot
├── src/main/resources/        # application.yml, perfis de ambiente e static/app compilado
├── frontend/                  # Interface Web React 18 + PrimeReact + Vite + Tailwind
│   ├── src/
│   │   ├── modules/           # Módulos operacionais (Overview, HCI Nodes, VMs, ERP, WebServer)
│   │   ├── components/        # Componentes compartilhados e modais
│   │   └── types/             # Tipagens TypeScript
│   ├── package.json
│   ├── vite.config.js
│   └── dist/                  # Artefatos estáticos
├── scripts/                   # Scripts de implantação, infraestrutura e verificação
├── etc/                       # Arquivos de propriedades e configurações
└── docs/i18n/                 # Documentação multilíngue (README, CONTRIBUTING, etc.)
```

## Diretrizes para Pull Requests
1. Crie uma branch descritiva a partir de main.
2. Escreva testes unitários para novas funcionalidades e controladores.
3. Garanta que npm run build e cargo test / mvn clean package passem sem erros.
4. Submeta o Pull Request com descrição detalhada das modificações.

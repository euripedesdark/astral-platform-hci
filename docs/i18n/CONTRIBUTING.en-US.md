# Contributing to Astral Platform & HCI

## Code of Conduct
This project and all contributors are governed by the Code of Conduct.

## Development Setup

### Prerequisites
* Java 21+ (OpenJDK ou Temurin)
* Maven 3.9+
* Rust 1.80+ (cargo, rustc)
* Node.js 20+ & npm 10+
* PostgreSQL 15+

### Build and Execution

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

## Project Structure

```
astral-platform-hci/
├── Cargo.toml                 # High-performance Rust Axum Core module
├── Cargo.lock                 # Rust locked dependency versions
├── pom.xml                    # Maven Spring Boot 3.3.5 / Java 21 build
├── src/                       # Integrated Rust and Java source code
│   ├── main.rs                # Axum entrypoint (HCI control plane)
│   ├── config/                # Port, database and auth configuration
│   ├── controller/            # Platform REST controllers and SPA routes
│   ├── domain/                # Domain models (HCI Nodes, VMs, Storage, ERP, WebServer)
│   └── service/               # Network orchestration, routing and auth services
├── src/main/java/             # Enterprise Spring Boot backend
├── src/main/resources/        # application.yml, profiles and compiled static/app
├── frontend/                  # React 18 + PrimeReact + Vite + Tailwind Web UI
│   ├── src/
│   │   ├── modules/           # Operational modules (Overview, HCI Nodes, VMs, ERP, WebServer)
│   │   ├── components/        # Reusable UI components and modals
│   │   └── types/             # TypeScript definitions
│   ├── package.json
│   ├── vite.config.js
│   └── dist/                  # Static build artifacts
├── scripts/                   # Deployment, infrastructure and verification scripts
├── etc/                       # System property files and configuration
└── docs/i18n/                 # Multilingual documentation (README, CONTRIBUTING, etc.)
```

## Pull Request Guidelines
1. Create a descriptive feature branch from main.
2. Write unit tests for new platform capabilities.
3. Ensure npm run build and cargo test complete cleanly.
4. Submit a Pull Request with a clear summary.

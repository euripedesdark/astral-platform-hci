# Contribuyendo a Astral Platform & HCI

## Código de Conducta
Este proyecto y sus participantes se rigen por el Código de Conducta.

## Configuración de Desarrollo

### Requisitos Previos
* Java 21+ (OpenJDK ou Temurin)
* Maven 3.9+
* Rust 1.80+ (cargo, rustc)
* Node.js 20+ & npm 10+
* PostgreSQL 15+

### Compilación y Ejecución

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

## Estructura del Proyecto

```
astral-platform-hci/
├── Cargo.toml                 # Módulo de alto rendimiento Rust Axum Core
├── Cargo.lock                 # Versiones bloqueadas de dependencias Rust
├── pom.xml                    # Compilación Maven Spring Boot 3.3.5 / Java 21
├── src/                       # Código fuente integrado Rust y Java
│   ├── main.rs                # Punto de entrada Axum (plano de control HCI)
│   ├── config/                # Configuración de red, base de datos y autenticación
│   ├── controller/            # Controladores REST de la plataforma y rutas SPA
│   ├── domain/                # Modelos de dominio (Nodos HCI, VMs, Storage, ERP, WebServer)
│   └── service/               # Orquestación de red, enrutamiento y autenticación
├── src/main/java/             # Backend empresarial Spring Boot
├── src/main/resources/        # application.yml, perfiles y estáticos compilados static/app
├── frontend/                  # Interfaz Web React 18 + PrimeReact + Vite + Tailwind
│   ├── src/
│   │   ├── modules/           # Módulos operativos (Overview, Nodos HCI, VMs, ERP, WebServer)
│   │   ├── components/        # Componentes compartidos y modales
│   │   └── types/             # Tipado TypeScript
│   ├── package.json
│   ├── vite.config.js
│   └── dist/                  # Compilación estática
├── scripts/                   # Scripts de despliegue, infraestructura y verificación
├── etc/                       # Archivos de propiedades y configuración
└── docs/i18n/                 # Documentación multilenguaje
```

## Directrices para Pull Requests
1. Cree una rama descriptiva a partir de main.
2. Añada pruebas unitarias para nuevas funcionalidades.
3. Compruebe que npm run build y cargo test se ejecuten sin errores.
4. Envíe el Pull Request con documentación de cambios.

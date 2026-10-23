# Contribuer à Astral Platform & HCI

## Code de Conduite
Ce projet et l’ensemble des contributeurs sont régis par le Code de Conduite.

## Configuration de Développement

### Prérequis
* Java 21+ (OpenJDK ou Temurin)
* Maven 3.9+
* Rust 1.80+ (cargo, rustc)
* Node.js 20+ & npm 10+
* PostgreSQL 15+

### Compilation et Exécution

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

## Structure du Projet

```
astral-platform-hci/
├── Cargo.toml                 # Module haute performance Rust Axum Core
├── Cargo.lock                 # Versions verrouillées des dépendances Rust
├── pom.xml                    # Build Maven Spring Boot 3.3.5 / Java 21
├── src/                       # Code source intégré Rust et Java
│   ├── main.rs                # Point d'entrée Axum (plan de contrôle HCI)
│   ├── config/                # Configuration réseau, base de données et authentification
│   ├── controller/            # Contrôleurs REST de la plateforme et routes SPA
│   ├── domain/                # Modèles de domaine (Nœuds HCI, VMs, Stockage, ERP, WebServer)
│   └── service/               # Orchestration réseau, routage et services d'authentification
├── src/main/java/             # Backend d'entreprise Spring Boot
├── src/main/resources/        # application.yml, profils et artefacts compilés static/app
├── frontend/                  # Interface Web React 18 + PrimeReact + Vite + Tailwind
│   ├── src/
│   │   ├── modules/           # Modules opérationnels (Overview, Nœuds HCI, VMs, ERP, WebServer)
│   │   ├── components/        # Composants partagés et modaux
│   │   └── types/             # Déclarations TypeScript
│   ├── package.json
│   ├── vite.config.js
│   └── dist/                  # Distribution statique compilée
├── scripts/                   # Scripts de déploiement et de vérification d'infrastructure
├── etc/                       # Fichiers de configuration et de propriétés
└── docs/i18n/                 # Documentation multilingue
```

## Lignes Directrices pour les Pull Requests
1. Créez une branche descriptive à partir de main.
2. Ajoutez des tests unitaires pour les fonctionnalités de la plateforme.
3. Assurez-vous que npm run build et cargo test réussissent sans erreur.
4. Soumettez la Pull Request avec un résumé clair.

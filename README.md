# Astral Platform HCI

🇧🇷 **Português (Brasil):** Clique no link abaixo para ler o README completo em português: [README.pt-BR.md](README.pt-BR.md)

🇺🇸 **English:** Click the link below to read the complete README in English: [README.en-US.md](README.en-US.md)

🇪🇸 **Español:** Haz clic en el siguiente enlace para leer el README completo en español: [README.es-ES.md](README.es-ES.md)

🇫🇷 **Français :** Cliquez sur le lien ci-dessous pour lire le README complet en français : [README.fr-FR.md](README.fr-FR.md)

## Instalar / atualizar (Rust + React)

```bash
cd "$HOME/astral/astral-platform-hci" &&
git pull --ff-only origin main &&
python3 deploy_platform.py --install-deps
```

Servico: `astral-platform-hci.service` (alias `astral-platform.service`).
[Fluxo atual, recuperacao e diagnostico](docs/deploy-platform.md).

Os documentos de arquitetura traduzidos abaixo contem referencias historicas a
Java/Maven. Para instalar a versao atual, use o fluxo Rust acima.

## OAuth2 (HCI + auth-service)

```bash
python3 configure_oauth2.py --public-url https://192.168.2.10 --install-deps
```

[Cadastro, implantacao e testes do fluxo OAuth2 completo](docs/oauth2.md).

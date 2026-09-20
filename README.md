# Illustrator

Entwicklungsumgebung mit [OpenSpec](https://openspec.dev) im Rust-Dev-Container (Debian trixie).

## Voraussetzungen

- Docker
- VS Code / Cursor mit Dev Containers Extension

## Start

1. Projekt öffnen
2. Command Palette → **Dev Containers: Reopen in Container**
3. Beim ersten Start installiert `post-create.sh` die OpenSpec CLI und initialisiert das Projekt für Cursor

## Dokumentation

- [Constraint mathematics](docs/constraint-mathematics.md) — Glossar und mathematische Formulierung (Structure/State, Residuen, Assembly, Interact)

## OpenSpec

Im Terminal (im Container):

```bash
openspec --version
openspec update
```

In Cursor Agent Chat (Slash-Commands):

- `/opsx-propose` – Änderung vorschlagen
- `/opsx-apply` – Änderung umsetzen
- `/opsx-archive` – abgeschlossene Änderung archivieren
- `/opsx-explore` – optional vorab explorieren

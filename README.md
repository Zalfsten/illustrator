# Illustrator

Entwicklungsumgebung mit [OpenSpec](https://openspec.dev) im Rust-Dev-Container (Debian trixie).

**Demo im Browser:** <https://zalfsten.github.io/illustrator/> (Stand `main`, ohne Laden/Speichern; Details in [impls/rust/README.md](impls/rust/README.md#hosted-demo-github-pages))

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

## Lizenz

Copyright (C) 2026 the Illustrator contributors.

Dieses Programm ist freie Software: Sie können es unter den Bedingungen der
GNU General Public License, wie von der Free Software Foundation
veröffentlicht, weitergeben und/oder ändern, entweder gemäß Version 3 der
Lizenz oder (nach Ihrer Wahl) jeder späteren Version
(SPDX: `GPL-3.0-or-later`). Der vollständige Lizenztext steht in
[LICENSE](LICENSE).

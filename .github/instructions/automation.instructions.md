---
applyTo: ".github/**/*.yml,.github/**/*.yaml,scripts/**/*.mjs,.github/skills/**/*.md,.github/agents/**/*.md"
---

# Automatyzacja i konfiguracja AI

- Workflow stosuje minimalne `permissions`.
- Nie umieszczaj sekretów w repo ani logach.
- Setup Copilot ma dokładnie job `copilot-setup-steps`.
- Preferuj deterministyczne `npm ci` i jawny toolchain Rust.
- Walidatory kończą się kodem różnym od zera przy niespójności.
- Skill ma YAML frontmatter `name` i `description`, konkretny workflow i komendy odbiorcze.
- Custom agent ma wąski mandat i nie może omijać `AGENTS.md`.
- Nie wyłączaj testu tylko po to, aby CI przeszło.

Uruchom `npm run validate:ai` oraz odpowiednie walidatory repo.


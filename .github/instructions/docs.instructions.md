---
applyTo: "docs/**/*.md,README.md,AGENTS.md,llms.txt"
---

# Dokumentacja

- Jeden temat na plik; bez monolitów.
- Dokument w `docs` ma maksymalnie 10 KB i 160 linii.
- Każda pozycja Toolboxa ma własny plik z ID, typem, rendererem, konfiguracją i ograniczeniami.
- Indeksuj każdy nowy dokument.
- Nie opisuj funkcji, której kod nie implementuje.
- Nie używaj starych nazw usuniętych dokumentów.
- Linki lokalne muszą istnieć.

Uruchom `npm run validate:docs`. Przy zmianie Registry uruchom także `validate:widgets`.


---
applyTo: "src/**/*.ts,src/**/*.svelte"
---

# Frontend Svelte

- Używaj Svelte 5 runes (`$state`, `$derived`, `$effect`, `$props`).
- TypeScript pozostaje strict; bez `any` i zbędnych castów.
- Korzystaj z `WidgetRendererProps` i helperów `widgets/shared`.
- Kontrolka interaktywna musi działać klawiaturą, mieć label i widoczny focus.
- Design nie wysyła write, ACK ani nawigacji Runtime.
- Quality/empty/error są jawnymi stanami, nie silent fallback.
- Nowy typ wymaga osobnego pliku, wpisu Registry, Factory, Properties i dokumentacji per-control.
- Neutralne HMI: kolor nie może być jedynym kanałem stanu.
- Nie wywołuj Tauri `invoke()` poza `src/lib/services/api.ts`.
- Nowa komenda zachowuje parytet API natywnego, browser mocka i typów.

Waliduj przez `npm run check`, `npm run validate:widgets`, `npm run validate:docs` i `npm run build`.

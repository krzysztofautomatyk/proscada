# Katalog Agent Skills ProScada

Ten dokument wyjaśnia, **jak wybrać właściwy skill** dla zadania AI w tym repozytorium oraz mapuje typowe zadania na skille. Skille żyją w [`.github/skills/`](../../.github/skills/README.md); każdy ma `SKILL.md` z `name`, `description`, „when to use”, procedurą, guardrails i minimalną walidacją.

## Jak wybrać skill

1. Zidentyfikuj **warstwę i intencję** zadania (kontrolka, Modbus, alarm, szablon, UI Designera, dokumentacja, walidacja, bezpieczeństwo).
2. Wybierz jeden **skill domenowy** z tabeli poniżej.
3. Prawie zawsze dołóż dwa skille przekrojowe:
   - **proscada-security** — jeśli dotykasz zapisów, ról, trybu, importów, CSP lub granicy PLC.
   - **proscada-validation** — do uruchomienia właściwej bramki walidacji przed zakończeniem.
4. Jeśli zmieniasz kontrolkę, prawie zawsze potrzebujesz też **proscada-docs** (dokument per-control) i **proscada-widget** razem.

## Mapowanie zadanie → skill

| Zadanie | Skill(e) |
| --- | --- |
| Dodaj/zmień kontrolkę, renderer, wpis Toolbox; napraw `validate:widgets` | [proscada-widget](../../.github/skills/proscada-widget/SKILL.md) (+ [proscada-docs](../../.github/skills/proscada-docs/SKILL.md)) |
| Polling/odczyty FC01–04, zapisy FC05/06/22, RMW single-writer, read-back, jakość | [proscada-modbus](../../.github/skills/proscada-modbus/SKILL.md) (+ [proscada-security](../../.github/skills/proscada-security/SKILL.md)) |
| Alarmy: grupy, lifecycle, deadband/delay/latch, ACK po stronie backendu | [proscada-alarm](../../.github/skills/proscada-alarm/SKILL.md) (+ [proscada-security](../../.github/skills/proscada-security/SKILL.md)) |
| Szablony komponentów, `.pscctrl`, podstawianie parametrów, bulk CSV, pompownie | [proscada-component-template](../../.github/skills/proscada-component-template/SKILL.md) |
| UI Designera: Toolbox/Properties/canvas/panele, dostępność, scroll, Designer vs Runtime | [proscada-designer](../../.github/skills/proscada-designer/SKILL.md) |
| Dokumentacja: małe pliki, per-control, indeksy, `validate:docs` | [proscada-docs](../../.github/skills/proscada-docs/SKILL.md) |
| Uruchomienie bramki, diagnoza walidatora, zablokowany EXE / isolated cargo target | [proscada-validation](../../.github/skills/proscada-validation/SKILL.md) |
| Zapisy fail-closed, mode/RBAC/quality, CSP, bezpieczne importy, audyt, granica PLC | [proscada-security](../../.github/skills/proscada-security/SKILL.md) |

## Typowe kombinacje

- **Nowa kontrolka HMI**: proscada-widget → proscada-docs → proscada-validation.
- **Nowe polecenie sterujące do PLC**: proscada-modbus → proscada-security → proscada-validation.
- **Nowy typ alarmu**: proscada-alarm → proscada-security → proscada-validation.
- **Rollout pompowni**: proscada-component-template → proscada-security (integralność) → proscada-validation.
- **Zmiana edytora**: proscada-designer → proscada-validation.

## Niezmienniki obowiązujące zawsze

- Kanoniczny zestaw = **35** kontrolek, a pełny Toolbox = 35 kontrolek + 10 szablonów; każdy z 45 elementów ma osobny dokument.
- Każdy zapis jest bramkowany i obserwowany read-backiem; equality zależy od `verify_readback`.
- Audyt jest trwałym JSONL z łańcuchem hashy; lifecycle alarmów ma osobny atomowy journal.
- Dokumentacja pozostaje modularna (≤10 KB / ≤160 linii); brak sekretów i ścieżek absolutnych.
- Fail closed: w razie wątpliwości (tryb, rola, jakość, checksum) — odmów, nie zezwalaj domyślnie.

## Bramka walidacji (skrót)

```powershell
npm run check
npm run validate:widgets
npm run validate:docs
npm run validate:ai
npm run validate:yaml
npm test
npm run build
cargo fmt --all --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path src-tauri/Cargo.toml
cargo build --locked --manifest-path src-tauri/Cargo.toml
```
Szczegóły i obejście zablokowanego EXE: [proscada-validation](../../.github/skills/proscada-validation/SKILL.md).

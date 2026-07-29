# GitHub Copilot instructions — ProScada

Najpierw przeczytaj `AGENTS.md` i dobierz skill z `.github/skills/README.md`.

## Zasady repo

- Stack: Tauri v2, Rust, Svelte 5 runes i TypeScript strict.
- Zachowaj rozdział Designer/Runtime.
- Nie wykonuj zapisu procesu z UI bez backendowego `write_tag`.
- Zachowaj jakość danych, timestamp i jawne stany błędu.
- Nie omijaj centralnego Alarm Managera.
- Rola pochodzi tylko z `login`; nie przywracaj komendy `set_role`.
- Każda kontrolka ma osobny plik, Registry, Factory i dokumentację.
- Używaj helperów z `widgets/shared`; nie duplikuj config parsing ani obsługi jakości.
- Nie dodawaj `any`, arbitralnych skryptów ani niebezpiecznych URL.
- Nie modyfikuj wygenerowanych schemas ręcznie.
- Nie uruchamiaj Tauri dev/build ani połączeń z PLC w cloud agent lub CI.
- Zmiana Tauri command wymaga parytetu w handlerze, `api.ts`, mocku i typach.
- `scriptRuntime.ts` parsuje własny język akcji; `eval`/`new Function` są zakazane.

## Wymagane sprawdzenia

Dobierz testy do zmiany, a przed końcem uruchom:

```powershell
npm run check
npm run validate:widgets
npm run validate:docs
npm run validate:ai
npm run validate:yaml
npm test
npm run build
cargo fmt --all --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

Zmiana dokumentacji nie wymaga Rust build. Zmiana Rust I/O wymaga testów Rust i przeglądu fail-closed.

## Komunikacja

Podawaj faktyczny wynik, ograniczenia i niepewności. Nie nazywaj funkcji certyfikowaną ani safety-rated bez formalnego dowodu.

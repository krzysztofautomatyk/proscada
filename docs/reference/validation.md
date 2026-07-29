# Walidacja i testy

## Frontend

```powershell
npm ci
npm run check
npm run validate:widgets
npm run validate:docs
npm run validate:ai
npm run validate:yaml
npm test
npm run build
```

`npm test` uruchamia **wszystkie** pliki `src/**/*.test.ts`. Pojedynczy plik można
uruchomić skryptem `test:*`, ale bramka odbiorcza i CI wykonują pełny zestaw.

`npm run check` wymaga 0 błędów i 0 ostrzeżeń `svelte-check`. Każde ostrzeżenie
blokuje bramkę lokalną i CI.

## Rust

```powershell
cargo fmt --all --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path src-tauri/Cargo.toml
cargo build --locked --manifest-path src-tauri/Cargo.toml
```

## Łańcuch dostaw

```powershell
npm audit --audit-level=high
cargo deny --manifest-path src-tauri/Cargo.toml check advisories licenses sources
```

CI uruchamia przypięty `cargo-deny 0.20.2` z polityką `deny.toml`. Workflow release
tworzy SPDX JSON SBOM, sumy SHA-256 i — dla tagów `v*` — atestację provenance
GitHub oraz publikuje je w GitHub Release. Artefakty pozostają niepodpisanymi
kandydatami, dopóki organizacja nie dostarczy własnego procesu code signing.

`deny.toml` jawnie dokumentuje terminowy wyjątek dla transitive Linux GTK/glib;
dotyczy wyłącznie nieużywanej przez ProScada ścieżki `VariantStrIter` i wymaga
ponownego przeglądu do 2026-10-31.

## Pokrycie istotnych przypadków

- maski set/reset FC22 i odrzucenie bitu poza `0..15`;
- kodowanie i dekodowanie typów 16/32/64-bitowych oraz obu kolejności słów;
- odrzucenie wartości poza zakresem typu zamiast obcięcia;
- rozwiązywanie nazw hosta i błąd dla nieistniejącej nazwy;
- start silnika bez uprawnień i odmowa wejścia w Designer dla Viewera;
- brak furtki `mode = designer` w administracji użytkownikami;
- zachowanie bazy kont przy zapisie projektu;
- kasowanie sesji przy wczytaniu projektu i odrzucenie podmienionego hashu;
- wymuszona zmiana domyślnego hasła przed zapisem do procesu;
- migracja starego skrótu SHA-256 na Argon2id przy logowaniu;
- PIN sprawdzany atomowo w tym samym żądaniu zapisu zalogowanego użytkownika;
- odmowy zapisu: rola, tryb, writable, poziom taga, jakość;
- latching alarmu, deadband analogowy i zawieszenie ewaluacji przy złej jakości;
- ciągłość łańcucha audytu po przycięciu okna i po restarcie;
- plan odczytu dla wielu tabel, adresów HR1000+ i tagów wielorejestrowych;
- odrzucenie skryptu w składni JavaScript przez parser języka akcji;
- kontrakt jakości danych dla kontrolek;
- kompletność 35 kontrolek i 45 dokumentów Toolboxa;
- 33 migracje nazw źródłowych.

## Konfiguracja AI

`validate:ai` sprawdza instrukcje Copilota, agentów eksperckich, skills, workflow i pełne SHA używanych GitHub Actions.

## Dokumentacja

Walidator sprawdza rozmiar, liczbę linii, lokalne linki Markdown, brak pustych plików oraz odwołania do usuniętej struktury.

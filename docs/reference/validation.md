# Walidacja i testy

## Frontend

```powershell
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

## Rust

```powershell
cargo fmt --all --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
cargo build --manifest-path src-tauri/Cargo.toml
```

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
- PIN potwierdzający wyłącznie konto zalogowane;
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

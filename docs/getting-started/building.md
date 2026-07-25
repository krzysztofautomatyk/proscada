# Budowanie aplikacji

## Frontend produkcyjny

```powershell
npm run check
npm run validate:widgets
npm run validate:docs
npm run validate:ai
npm run validate:yaml
npm run test:pump-template
npm run build
```

Wynik znajduje się w `dist\`.

## Testy Rust

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

Testy pokrywają maski FC22, plan odczytów Modbus oraz zachowanie alarmów.

## Aplikacja desktopowa

```powershell
npm run tauri:build
```

Jeśli uruchomiona kopia ProScada blokuje plik EXE lub MSI, zamknij ją przed bundlowaniem.

## Minimalna bramka wydania

- brak błędów i ostrzeżeń `svelte-check`;
- 35/35 rendererów i 33/33 migracji;
- poprawny build Vite;
- wszystkie testy Rust;
- uruchomienie wynikowego EXE;
- brak uszkodzonych linków dokumentacji.

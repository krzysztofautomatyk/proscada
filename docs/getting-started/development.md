# Uruchamianie deweloperskie

## Frontend z danymi mock

```powershell
npm run dev
```

Tryb przeglądarkowy pozwala projektować ekrany i testować kontrolki. Operacje Tauri są zastępowane lokalnym adapterem mock.

## Pełna aplikacja Tauri

```powershell
npm run tauri:dev
```

Tauri uruchamia Rust engine, audyt oraz klienta Modbus TCP.

## Najważniejsze skrypty

```powershell
npm run check
npm run validate:widgets
npm run validate:docs
npm run validate:ai
npm run validate:yaml
npm run test:pump-template
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```

## Tryby aplikacji

- **Design:** edycja projektu; zapisy procesowe są blokowane przez backend.
- **Runtime:** odczyt live, alarmy i dozwolone komendy.

Tryb Simulation należy wyraźnie oznaczyć i oddzielić od źródeł produkcyjnych.

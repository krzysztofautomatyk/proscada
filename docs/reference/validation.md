# Walidacja i testy

## Frontend

```powershell
npm run check
npm run validate:widgets
npm run validate:docs
npm run validate:ai
npm run validate:yaml
npm run test:pump-template
npm run build
```

## Rust

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
cargo build --manifest-path src-tauri/Cargo.toml
```

## Pokrycie istotnych przypadków

- maski set/reset FC22;
- odrzucenie bitu poza `0..15`;
- latching alarmu;
- deadband analogowy;
- plan odczytu dla wielu tabel i adresów HR1000+;
- jakość Good dla coil i odległego rejestru;
- kompletność 35 kontrolek;
- kompletność 45 dokumentów Toolboxa;
- 33 migracje nazw źródłowych.

## Konfiguracja AI

`validate:ai` sprawdza instrukcje Copilota, pięciu agentów eksperckich, osiem skills, dwa workflow i pełne SHA używanych GitHub Actions.

## Dokumentacja

Walidator sprawdza rozmiar, liczbę linii, lokalne linki Markdown, brak pustych plików oraz odwołania do usuniętej struktury.

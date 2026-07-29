# Budowanie aplikacji

## Frontend produkcyjny

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

Wynik znajduje się w `dist\`.

## Testy Rust

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

Testy pokrywają maski FC22, plan odczytów Modbus oraz zachowanie alarmów.

## Aplikacja desktopowa

```powershell
npm run tauri:build
```

Jeśli uruchomiona kopia ProScada blokuje plik EXE lub MSI, zamknij ją przed bundlowaniem.

## Minimalna bramka wydania

- 0 błędów i 0 ostrzeżeń `svelte-check`;
- 35/35 rendererów i 33/33 migracji;
- poprawny build Vite;
- wszystkie testy Rust;
- uruchomienie wynikowego EXE;
- brak uszkodzonych linków dokumentacji.

## Kandydaci desktopowi

`.github/workflows/release.yml` buduje kandydatów Linux, Windows i macOS dla tagów
`v*` lub ręcznego uruchomienia. Dla tagu `v*` publikuje GitHub Release z
niepodpisanymi bundle, sumami SHA-256 oraz SPDX JSON SBOM. Ręczne uruchomienie
publikuje tylko artefakty workflow. Dla tagów GitHub tworzy atestację provenance
opartą na krótkotrwałej tożsamości workflow.

Linux jest bundlowany na Ubuntu 22.04, aby nie podnosić niepotrzebnie minimalnej
wersji glibc. Zgodność instalatora z docelową dystrybucją nadal wymaga smoke testu.

Workflow nie udaje organizacyjnego code signing. Wydanie produkcyjne nadal wymaga
zaufanego procesu podpisania, przechowania klucza poza repo i ręcznego smoke testu
instalacji na wspieranym systemie.

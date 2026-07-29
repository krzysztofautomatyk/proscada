# ProScada

Desktopowa stacja inżynierska SCADA i Runtime HMI oparta na Tauri, Rust oraz Svelte 5.

| Warstwa | Technologia |
|---|---|
| Desktop | Tauri v2 |
| Core | Rust |
| UI | Svelte 5 + TypeScript |
| Protokół | Modbus TCP master |

> ProScada jest narzędziem laboratoryjnym, szkoleniowym i integracyjnym. Nie jest certyfikowanym sterownikiem safety. Zobacz [zakres zgodności](docs/reference/compliance.md).

## Najważniejsze funkcje

- Designer w układzie Visual Studio;
- Toolbox z 35 kanonicznymi kontrolkami;
- osobny plik Svelte dla każdej kontrolki;
- dynamiczny polling FC01, FC02, FC03 i FC04;
- zapisy FC05, FC06 i FC22;
- centralne style, fonty i animacje projektu;
- centralny Alarm Manager z grupami;
- biblioteka własnych komponentów `.pscctrl`;
- generator 20 pompowni 2P + 2F + 1S;
- hash projektu i łańcuch audytu SHA-256.

## Szybki start

```powershell
npm ci
npm run tauri:dev
```

Frontend bez natywnego backendu:

```powershell
npm run dev
```

## Walidacja

```powershell
npm ci
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
npm audit --audit-level=high
cargo deny --manifest-path src-tauri/Cargo.toml check advisories licenses sources
```

## Dokumentacja

Pełny, modułowy indeks znajduje się w [docs/README.md](docs/README.md).

- [Start](docs/getting-started/prerequisites.md)
- [Architektura](docs/architecture/overview.md)
- [Katalog Toolboxa](docs/toolbox/README.md)
- [Designer](docs/features/designer.md)
- [Alarm Manager](docs/features/alarm-manager.md)
- [Biblioteka komponentów](docs/features/component-library.md)
- [Generator pompowni](docs/features/pump-station-rollout.md)
- [Modbus i bity](docs/features/bit-register-io.md)
- [Eksploatacja](docs/operations/runtime-quality.md)
- [Walidacja](docs/reference/validation.md)
- [Praca z AI i LLM](docs/ai/README.md)
- [Katalog Agent Skills](docs/ai/skills-catalog.md)
- [Zasady współpracy](CONTRIBUTING.md)
- [Polityka bezpieczeństwa](.github/SECURITY.md)

## Struktura repozytorium

```text
docs/          małe dokumenty podzielone domenowo
projects/      przykładowe projekty
scripts/       walidatory repozytorium
src/           frontend Svelte
src-tauri/     rdzeń Rust i Tauri
```

Dokumenty w `docs` mają limit 10 KB i 160 linii. `validate:docs` blokuje duże pliki i uszkodzone linki.

## Licencja

[MIT](LICENSE)

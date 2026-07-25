# Mapa kontekstu dla LLM

| Zadanie | Najpierw przeczytaj |
|---|---|
| kontrolka | `docs/toolbox/README.md`, Registry, podobny renderer |
| Designer | `docs/features/designer.md`, Toolbox, Properties |
| Modbus | `docs/features/bit-register-io.md`, engine i modbus |
| alarmy | `docs/features/alarm-manager.md`, project schema i engine |
| komponenty | `docs/features/component-library.md`, store app |
| pompownie | `docs/features/pump-station-rollout.md` |
| dokumentacja | `docs/README.md`, `validate-docs.mjs` |
| bezpieczeństwo | `docs/architecture/security.md`, capabilities |

## Źródła prawdy

- Toolbox: `src/lib/components/widgets/registry/catalog.ts`;
- Factory: `WidgetView.svelte`;
- typy frontend: `src/lib/types.ts`;
- schema backend: `src-tauri/src/project/mod.rs`;
- Modbus: `src-tauri/src/modbus/mod.rs`;
- gates zapisu i alarmy: `src-tauri/src/engine/mod.rs`;
- komendy Tauri: `commands/mod.rs` i `lib.rs`.

Nie wnioskuj zachowania wyłącznie z dokumentacji. Potwierdź je w kodzie.


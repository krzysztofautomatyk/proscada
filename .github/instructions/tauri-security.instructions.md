---
applyTo: "src-tauri/capabilities/**/*.json,src-tauri/tauri.conf.json,src-tauri/src/commands/**/*.rs,src-tauri/src/lib.rs"
---

# Tauri command i capabilities

- Nie rozszerzaj filesystem scope, pluginów ani permissions bez threat modelu.
- Nowa komenda musi zostać dodana spójnie do Rust command, `generate_handler!`, frontend `api.ts`, browser mock i typów.
- Komenda zapisująca wymaga backendowej autoryzacji; UI nie jest granicą bezpieczeństwa.
- Nie loguj sekretów, haseł ani tokenów.
- Nie włączaj remote scripts; zachowaj deny-by-default CSP.
- Zmiana capability wymaga ręcznego review przez `ot-safety-reviewer`.

Uruchom pełne testy Rust, frontend check i `validate:ai`.


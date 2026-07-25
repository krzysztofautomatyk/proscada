---
name: proscada-component-template
description: Work with ProScada reusable component templates — the ComponentTemplate model, parameter substitution, .pscctrl import/export with SHA-256 integrity, bulk instantiation from CSV, and the 20-station pump rollout. Use when a task mentions component library, ComponentTemplate, .pscctrl files, parameter_names substitution, bulk CSV import, or pumping-station rollout.
---

# proscada-component-template

## When to use
- Adding/changing reusable component templates, `.pscctrl` import/export, or bulk instantiation.
- Working on the pump-station (pompownia) rollout generator or CSV bulk create.

## Key files
- Model: `src/lib/types.ts` (`ComponentTemplate` — `id`, `name`, `category`, `version`, `width`, `height`, `widgets`, `parameter_names`, `alarm_templates`).
- Library UI + import/CSV/rollout: `src/lib/components/designer/ComponentLibraryManager.svelte`.
- Project integrity (SHA-256 content hash): `src-tauri/src/project/mod.rs` (`Sha256`, `content_hash`); project field `component_templates`.
- File IO helpers: `src/lib/services/fileIo.ts`.

## Procedure
1. Define the template with a stable `id`, semantic `version`, `width`/`height`, its `widgets`, and the `parameter_names` used as substitution tokens.
2. Parameter substitution: replace tokens (e.g. station name, tag prefix like `PLC.PS<n>`, device id, base register) when instantiating; never hard-code instance-specific values in the template body.
3. `.pscctrl` import (`Import .pscctrl` button): parse, validate structure and checksum, then add to `component_templates`.
4. Bulk CSV (`Bulk CSV…`): parse the object list (csv), map each row to substituted parameters, and instantiate one component per row.
5. Pump rollout: generate rows `PS_<n>,Pompownia <n>,PLC.PS<n>,...,<deviceId>,<baseRegister>` and instantiate the reference pumping stations.
6. Save the project before Runtime so the Rust engine loads generated tags, alarm groups and definitions and recomputes `content_hash`.

## Guardrails
- Preserve **SHA-256** integrity: any change to templates/project must recompute `content_hash`; reject `.pscctrl` with a mismatched/invalid checksum (fail closed).
- Keep register/address maps non-overlapping across bulk-generated instances.
- Substitute every declared `parameter_name`; a missing parameter is an error, not a silent default.
- Do not embed secrets or absolute machine paths in templates.
- Do not invent command bits in the factory Water Tank map; pump-station commands exist only after the CSV-driven object map creates them.

## Validate
```powershell
npm run check
npm run test:pump-template
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```

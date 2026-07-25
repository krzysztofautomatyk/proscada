---
name: proscada-docs
description: Author or update ProScada documentation under docs/ following the mandatory modular structure — small files (≤10 KB / ≤160 lines), one doc per control, correct indexes and local links — so that npm run validate:docs passes. Use when a task adds/edits docs, per-control pages, the Toolbox index, or when validate:docs fails. Never create large monolithic docs.
---

# proscada-docs

## When to use
- Adding/updating any Markdown under `docs/` (or `README.md`), especially per-control pages and indexes.
- `npm run validate:docs` fails (size/line limit, broken link, missing control doc, stale reference).

## Structure (enforced by `scripts/validate-docs.mjs`)
- Single entry index: `docs/README.md`.
- Canonical docs: one file per control at `docs/toolbox/controls/<folder>/<type>.md`.
- Template docs: one file per Toolbox template at `docs/toolbox/templates/<type>.md`.
- Generic Toolbox docs at `docs/toolbox/generic/`: `registry-factory.md`, `common-properties.md`, `binding-quality.md`, `styles-fonts.md`, `dynamics-events.md`, `definition-of-done.md` — each linked as `generic/<name>` in the Toolbox index.
- Category → folder map matches the validator (`primitives`, `assets`, `indicators`, `process`, `commands`, `inputs`, `data`, `layout`, `navigation`, `feedback`, `alarms`, `utilities`).

## Procedure
1. Keep every file **≤ 10 KB and ≤ 160 lines**; split content into new topic files instead of growing one.
2. For a new/changed control doc, include the exact rows `| ID | \`<canonicalId>\` |` and `| Typ | \`<type>\` |`, then add its link to `docs/toolbox/README.md`.
3. Use only relative local links; every non-http link target must resolve to an existing file.
4. Keep the total modular file count high (validator expects ≥ 20 files); don't merge files to reduce count.

## Guardrails
- No monolithic docs — the validator rejects oversized files.
- Do not reference retired documents: `ARCHITECTURE.md`, `COMPLIANCE.md`, `WATER_TANK_INTEGRATION.md`, `WIDGET_DYNAMICS.md`, `00-expert-council.md`.
- Keep exactly one doc per actual Toolbox item: 35 canonical controls and 10 process templates.
- No broken links, no empty files.

## Validate
```powershell
npm run validate:docs
```
Expect: `Docs OK: <N> modular files, 45/45 Toolbox item docs and 6 generic Toolbox docs; ...`.

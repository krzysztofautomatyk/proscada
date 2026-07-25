---
name: proscada-widget
description: Add or modify a ProScada Toolbox control (widget) end-to-end — registry catalog entry, WidgetView renderer branch, a dedicated Svelte component, Properties panel wiring, and per-control documentation. Use when a task mentions adding/changing a control, widget, Toolbox item, renderer, or when validate:widgets fails. Keeps the canonical set at exactly 35 controls / 35 renderers.
---

# proscada-widget

## When to use
- Adding a new canonical control or editing an existing one (renderer, props, category).
- `npm run validate:widgets` fails (missing renderer branch, shared component, wrong folder).

## Key files (single source of truth)
- Registry: `src/lib/components/widgets/registry/catalog.ts` (`CANONICAL_WIDGET_IDS`, `canonicalWidgets`, `SOURCE_WIDGET_MIGRATIONS`), types in `registry/types.ts`.
- Renderer switch: `src/lib/components/widgets/WidgetView.svelte` (one `{#if}/{:else if}` branch per `widget.widget_type`).
- Renderer component: `src/lib/components/widgets/catalog/<category>/<Name>Widget.svelte` (one file per control; **never** reuse a component).
- Properties: `src/lib/components/widgets/Properties.svelte` and designer `src/lib/components/designer/Properties.svelte`.
- Docs: `docs/toolbox/controls/<folder>/<type>.md` + link in `docs/toolbox/README.md`.

## Procedure
1. Pick `type` (snake_case), `canonicalId`, and `category`. The 12 categories map to folders in `scripts/validate-widget-catalog.mjs` (e.g. `indicators`, `process`, `inputs`, `alarms`).
2. Add/edit the entry in `canonicalWidgets` in `catalog.ts`. If replacing a source control name, keep/adjust `SOURCE_WIDGET_MIGRATIONS` (must stay 33 entries mapping to real types).
3. Create the Svelte renderer under `catalog/<folder>/<Name>Widget.svelte`. Read data via bindings/props; render quality state; no direct IPC.
4. Import it in `WidgetView.svelte` and add a `{:else if widget.widget_type === "<type>"}` branch that mounts exactly that component.
5. Expose editable props in both Properties panels.
6. Write `docs/toolbox/controls/<folder>/<type>.md` including the rows `| ID | \`<canonicalId>\` |` and `| Typ | \`<type>\` |`, and add its link to `docs/toolbox/README.md`.

## Guardrails
- Total must remain **35 unique IDs and 35 unique types**; do not add a 36th without removing one.
- Each canonical type needs its **own** renderer file — validator rejects shared components and any `.svelte` sitting directly in `catalog/`.
- Component folder must match the category folder the validator expects.
- Keep widget files small; move shared logic to `catalog/<category>/*Model.ts` (see `alarms/alarmModel.ts`).

## Validate
```powershell
npm run validate:widgets
npm run check
npm run build
```
Expect: `Widget catalog OK: 35 canonical IDs, 35 types, 35 separate renderers, 33/33 source migrations.`

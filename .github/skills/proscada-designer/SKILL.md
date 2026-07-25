---
name: proscada-designer
description: Build or refine the ProScada Designer UX — Toolbox, Properties panel, canvas, side panels, object list, accessibility, and scrollbars — while preserving the strict Designer-vs-Runtime separation. Use when a task touches the designer editor UI, drag/drop, property editing, canvas interaction, panel layout, keyboard accessibility, or scroll controls.
---

# proscada-designer

## When to use
- Editing the authoring UI (not the widget renderers themselves): Toolbox, Properties, canvas, panels.
- Accessibility, keyboard navigation, or scrollbar/scroll-control work in the editor.

## Key files
- `src/lib/components/designer/Toolbox.svelte` — palette of the 35 controls + templates.
- `src/lib/components/designer/Properties.svelte` — property editing for the selected object.
- `src/lib/components/designer/DesignerCanvas.svelte` — placement/selection/drag surface.
- `src/lib/components/designer/ObjectList.svelte`, `ContextMenu.svelte`, `VerticalScrollControls.svelte`.
- Selection/app state: `src/lib/stores/selection.ts`, `src/lib/stores/app.ts`.

## Designer vs Runtime (hard boundary)
- Engine `mode` is `designer` or `runtime` (`src-tauri/src/engine/mod.rs`).
- Designer edits the project model; it MUST NOT issue process writes. `write_tag` is blocked unless `mode == "runtime"` — do not add designer paths that bypass this.

## Procedure
1. Add Toolbox entries by reading the registry catalog (`registry/catalog.ts`); do not hard-code a parallel widget list.
2. Property editors mutate the selected `WidgetDef`/project node via the stores; keep changes serializable into `ScadaProject`.
3. Keep canvas interactions (select, move, resize) driven by `selection.ts`; recompute layout, not raw DOM state.
4. For long lists/canvas, use the existing scroll controls; ensure content is reachable by keyboard and pointer.

## Guardrails
- Accessibility: interactive elements need roles/labels and keyboard operability; don't ship click-only controls.
- Never trigger live process writes from Designer; authoring only mutates the model.
- Keep Toolbox/Properties in sync with the registry — a control missing from the catalog must not appear, and vice versa.
- Preserve scrollability; avoid fixed heights that clip Properties/ObjectList content.

## Validate
```powershell
npm run check
npm run validate:widgets
npm run build
```
Local Windows smoke only: `npm run tauri:dev`, place a control, edit a property and confirm no write is possible in Designer mode. Never run this step in cloud agent or CI.

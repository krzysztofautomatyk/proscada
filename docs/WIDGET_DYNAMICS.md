# Generic Widget Dynamics (Label + future widgets)

Architecture for conditional **blink**, **marquee scroll**, **visibility**, plus **lock**.

## Condition modes (shared)

| Mode | Meaning |
|------|---------|
| `none` | Effect off |
| `always` | Effect always on |
| `tag_true` / `tag_false` | BOOL / non-zero |
| `tag_bit` | Register bit N (0–15) |
| `tag_val_eq` / `gt` / `lt` / `neq` | Compare register value |

Config keys (flat on `widget.config`):

- Blink: `blinkMode`, `blinkTagId`, `blinkBit`, `blinkVal`, `blinkSpeedMs`
- Marquee: `scrollMode`, `scrollTagId`, `scrollBit`, `scrollVal`, `scrollSpeedSec`, `scrollDir`
- Visibility: `visibilityMode`, `visibilityTagId`, `visibilityBit`, `visibilityVal`

Empty `*TagId` falls back to the widget’s main `tag_id`.

## Code

| Module | Role |
|--------|------|
| `src/lib/utils/dynamics.ts` | Evaluate conditions; defaults; font list |
| `ConditionEditor.svelte` | PropertyGrid UI for any effect |
| `LabelWidget.svelte` | Font / color / align / marquee / blink / hide |
| `DynamicShell.svelte` | Reusable shell for future widgets |
| `WidgetDef.locked` | Designer lock (no move/resize) |

## Adding a new widget

1. Add catalog entry + component.
2. For blink/hide: wrap with `DynamicShell` or call `isWidgetBlinking` / `isWidgetVisible`.
3. Dynamics keys appear automatically in Properties (generic section).
4. For marquee-style text only, copy Label marquee pattern.

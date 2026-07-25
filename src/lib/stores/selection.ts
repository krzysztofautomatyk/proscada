/**
 * Designer selection helpers — multi-select, group expansion, marquee.
 */
import type { FormDef, WidgetDef } from "$lib/types";

/** Expand selection so every group member of any selected widget is included. */
export function expandSelectionWithGroups(
  form: FormDef,
  ids: string[],
): string[] {
  const set = new Set(ids);
  const groups = new Set(
    form.widgets
      .filter((w) => set.has(w.id) && w.group_id)
      .map((w) => w.group_id as string),
  );
  for (const w of form.widgets) {
    if (w.group_id && groups.has(w.group_id)) set.add(w.id);
  }
  return Array.from(set);
}

/** Stable short label for a group_id, e.g. G1, G2 */
export function groupLabel(groupId: string, allGroupIds: string[]): string {
  const sorted = [...new Set(allGroupIds)].sort();
  const idx = sorted.indexOf(groupId);
  return idx >= 0 ? `G${idx + 1}` : "G?";
}

/** Distinct color for group badge */
export function groupColor(groupId: string): string {
  let h = 0;
  for (let i = 0; i < groupId.length; i++) h = (h * 31 + groupId.charCodeAt(i)) >>> 0;
  const hue = h % 360;
  return `hsl(${hue} 70% 45%)`;
}

export function widgetsByIds(form: FormDef, ids: string[]): WidgetDef[] {
  const set = new Set(ids);
  return form.widgets.filter((w) => set.has(w.id));
}

/** Axis-aligned rect hit: widget intersects marquee box (form coords). */
export function widgetIntersectsRect(
  w: WidgetDef,
  x0: number,
  y0: number,
  x1: number,
  y1: number,
): boolean {
  const left = Math.min(x0, x1);
  const right = Math.max(x0, x1);
  const top = Math.min(y0, y1);
  const bottom = Math.max(y0, y1);
  const wl = w.x;
  const wr = w.x + w.w;
  const wt = w.y;
  const wb = w.y + w.h;
  return !(wr < left || wl > right || wb < top || wt > bottom);
}

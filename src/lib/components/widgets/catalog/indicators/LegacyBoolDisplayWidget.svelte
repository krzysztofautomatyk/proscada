<script lang="ts">
  import type { WidgetDef, TagValue } from "$lib/types";

  interface Props {
    widget: WidgetDef;
    tag?: TagValue | null;
    design?: boolean;
    onWrite?: (tagId: string, value: number) => void;
  }

  let { widget, tag = null, design = false }: Props = $props();

  const cfg = $derived((widget.config ?? {}) as Record<string, unknown>);
  const str = (k: string, d = "") => String(cfg[k] ?? d);
  const num = (k: string, d = 0) => Number(cfg[k] ?? d);

  const known = $derived(design || tag?.quality === "good");
  const isTrue = $derived(known && (tag?.bool_value ?? false));
  const label = $derived(str("label", "BOOL STATUS"));
  const trueLabel = $derived(str("trueLabel", "TRUE"));
  const falseLabel = $derived(str("falseLabel", "FALSE"));
  const trueColor = $derived(str("trueColor", "#16A34A"));
  const falseColor = $derived(str("falseColor", "#9CA3AF"));
  const bgColor = $derived(str("bgColor", "#ffffff"));
  const borderColor = $derived(str("borderColor", "#e5e7eb"));
  const borderWidth = $derived(num("borderWidth", 1));
  const borderRadius = $derived(num("borderRadius", 8));
  const fontFamily = $derived(str("fontFamily", "Segoe UI, system-ui, sans-serif"));
  const fontSize = $derived(num("fontSize", 11));
  const fontWeight = $derived(str("fontWeight", "700"));
  const textColor = $derived(str("textColor", "#1f2937"));
</script>

<div
  class="bool-card"
  style:background={bgColor}
  style:border="{borderWidth}px solid {borderColor}"
  style:border-radius="{borderRadius}px"
  style:font-family={fontFamily}
>
  <span class="bool-dot" style:background={known ? (isTrue ? trueColor : falseColor) : "#64748b"}></span>
  <span class="bool-title" style:color={textColor} style:font-size="{fontSize}px" style:font-weight={fontWeight}>
    {label}
  </span>
  <span
    class="bool-badge"
    style:background={known ? (isTrue ? trueColor : "#f3f4f6") : "#e2e8f0"}
    style:color={known ? (isTrue ? "#ffffff" : "#4b5563") : "#334155"}
    style:font-size="{Math.max(8, fontSize - 2)}px"
  >
    {known ? (isTrue ? trueLabel : falseLabel) : "NO DATA"}
  </span>
</div>

<style>
  .bool-card {
    width: 100%;
    height: 100%;
    padding: 6px 10px;
    box-sizing: border-box;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
  }
  .bool-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    box-shadow: 0 0 4px currentColor;
    flex-shrink: 0;
  }
  .bool-title {
    flex: 1;
  }
  .bool-badge {
    font-weight: 800;
    padding: 2px 6px;
    border-radius: 99px;
  }
</style>

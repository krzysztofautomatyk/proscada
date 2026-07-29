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

  const quality = $derived(tag?.quality ?? "bad");
  const known = $derived(design || quality === "good");
  const on = $derived(known && (tag?.bool_value ?? false));

  const fontFamily = $derived(str("fontFamily", "Segoe UI, system-ui, sans-serif"));
  const fontSize = $derived(num("fontSize", 12));
  const fontWeight = $derived(str("fontWeight", "700"));
  const bgColor = $derived(str("bgColor", "#FFFFFF"));
  const borderColor = $derived(str("borderColor", "#E5E7EB"));
  const borderWidth = $derived(num("borderWidth", 1));
  const borderRadius = $derived(num("borderRadius", 8));
  const titleColor = $derived(str("titleColor", "#6B7280"));

  function lampColor() {
    if (on) return str("onColor", "#16A34A");
    return str("offColor", "#9CA3AF");
  }
</script>

<div
  class="w-chrome"
  style:background={bgColor}
  style:border="{borderWidth}px solid {borderColor}"
  style:border-radius="{borderRadius}px"
  style:font-family={fontFamily}
>
  <div class="w-title" style:color={titleColor} style:font-size="{Math.max(9, fontSize - 1)}px">
    <span class="quality {quality}"></span>{str("title", "STATE")}
  </div>
  <div class="w-body">
    <span
      class="lamp-dot"
      class:off={!on}
      style:background={known ? lampColor() : "#64748b"}
      style:color={known ? lampColor() : "#64748b"}
    ></span>
    <span
      style:font-weight={fontWeight}
      style:font-size="{fontSize}px"
      style:color={on ? lampColor() : "#9CA3AF"}
    >
      {known ? (on ? str("onLabel", "ON") : str("offLabel", "OFF")) : "NO DATA"}
    </span>
  </div>
</div>

<style>
  .w-chrome {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    overflow: hidden;
    padding: 6px 8px;
  }
  .w-title {
    font-weight: 700;
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .w-body {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .lamp-dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    box-shadow: 0 0 6px currentColor;
    display: inline-block;
  }
  .lamp-dot.off {
    box-shadow: none;
  }
  .quality {
    display: inline-block;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #dc2626;
  }
  .quality.good {
    background: #16a34a;
  }
  .quality.uncertain {
    background: #eab308;
  }
</style>

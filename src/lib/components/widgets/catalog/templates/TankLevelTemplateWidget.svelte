<script lang="ts">
  import type { WidgetDef, TagValue } from "$lib/types";

  interface Props {
    widget: WidgetDef;
    tag?: TagValue | null;
    design?: boolean;
    onWrite?: (tagId: string, value: number) => void;
  }

  let { widget, tag = null }: Props = $props();

  const cfg = $derived((widget.config ?? {}) as Record<string, unknown>);
  const str = (k: string, d = "") => String(cfg[k] ?? d);
  const num = (k: string, d = 0) => Number(cfg[k] ?? d);
  const bool = (k: string, d = false) => Boolean(cfg[k] ?? d);

  const quality = $derived(tag?.quality ?? "bad");
  const value = $derived(tag?.value ?? 0);

  function tankPct() {
    const min = num("min", 0);
    const max = num("max", 1000);
    const span = max - min || 1;
    return Math.max(0, Math.min(100, ((value - min) / span) * 100));
  }

  function tankFillColor() {
    const alarm = cfg.alarm != null ? Number(cfg.alarm) : null;
    const warn = cfg.warn != null ? Number(cfg.warn) : null;
    if (alarm != null && value >= alarm) return str("alarmColor", "#DC2626");
    if (warn != null && value >= warn) return str("warnColor", "#EAB308");
    return str("fillColor", "#39B7E6");
  }
</script>

<div
  class="tank-shell"
  style:background={str("bgColor", "#FFFFFF")}
  style:border="{num("borderWidth", 2)}px solid {str("borderColor", "#9CA3AF")}"
  style:border-radius="{num("borderRadius", 8)}px"
  style:font-family={str("fontFamily", "Segoe UI, system-ui, sans-serif")}
>
  <div
    class="w-title"
    style:color={str("titleColor", "#6B7280")}
    style:font-size="{num("titleFontSize", 11)}px"
    style:font-weight={str("fontWeight", "700")}
  >
    <span class="quality {quality}"></span>{str("title", "LEVEL")}
  </div>
  <div class="tank-fill" style:height="{tankPct()}%" style:background={tankFillColor()}></div>
  {#if bool("showValue", true)}
    <div class="tank-level-text" style:color={str("textColor", "#1F2937")}>
      <div style:font-size="{num("fontSize", 20)}px" style:font-weight="800">{value.toFixed(0)}</div>
      <div style:font-size="11px" style:opacity="0.85">{str("unit", "cm")}</div>
    </div>
  {/if}
</div>

<style>
  .tank-shell {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    position: relative;
    overflow: hidden;
    padding: 6px;
  }
  .w-title {
    position: relative;
    z-index: 2;
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .tank-fill {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    transition: height 0.4s ease;
    opacity: 0.85;
  }
  .tank-level-text {
    position: relative;
    z-index: 2;
    margin: auto;
    text-align: center;
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

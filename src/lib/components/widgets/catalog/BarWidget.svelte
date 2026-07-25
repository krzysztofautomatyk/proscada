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

  const value = $derived(tag?.value ?? 0);
  const min = $derived(num("min", 0));
  const max = $derived(num("max", 100));
  const pct = $derived(Math.max(0, Math.min(100, ((value - min) / (max - min || 1)) * 100)));
</script>

<div
  class="w-chrome"
  style:background={str("bgColor", "#FFFFFF")}
  style:border="{num("borderWidth", 1)}px solid {str("borderColor", "#E5E7EB")}"
  style:border-radius="{num("borderRadius", 8)}px"
  style:font-family={str("fontFamily", "Segoe UI, system-ui, sans-serif")}
>
  <div
    class="w-title"
    style:color={str("titleColor", "#6B7280")}
    style:font-size="{num("fontSize", 11)}px"
    style:font-weight={str("fontWeight", "700")}
  >
    {str("title", "BAR")}
  </div>
  <div class="w-body">
    <div class="track" style:background={str("trackColor", "#E5E7EB")}>
      <div class="fill" style:width="{pct}%" style:background={str("fillColor", "#16A34A")}></div>
    </div>
  </div>
</div>

<style>
  .w-chrome {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    padding: 6px;
  }
  .w-title {
    margin-bottom: 4px;
  }
  .w-body {
    flex: 1;
    display: flex;
    align-items: center;
  }
  .track {
    width: 100%;
    height: 14px;
    border-radius: 3px;
    overflow: hidden;
  }
  .fill {
    height: 100%;
    transition: width 0.25s ease;
  }
</style>

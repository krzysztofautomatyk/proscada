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

<div class="w-chrome" style:background={str("bgColor", "#FFFFFF")} style:border="1px solid #E5E7EB">
  <div class="w-title" style:color="#6B7280">{str("title", "BAR")}</div>
  <div class="w-body" style:padding="6px">
    <div style:width="100%" style:height="14px" style:background="#E5E7EB" style:border-radius="3px" style:overflow="hidden">
      <div style:width="{pct}%" style:height="100%" style:background={str("fillColor", "#16A34A")}></div>
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
    border-radius: 8px;
    padding: 6px;
  }
  .w-title {
    font-size: 11px;
    font-weight: 700;
  }
  .w-body {
    flex: 1;
    display: flex;
    align-items: center;
  }
</style>

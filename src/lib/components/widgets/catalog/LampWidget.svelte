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
  const bool = (k: string, d = false) => Boolean(cfg[k] ?? d);

  const quality = $derived(tag?.quality ?? "bad");
  const on = $derived(tag?.bool_value ?? false);

  function lampColor() {
    if (on) return str("onColor", "#16A34A");
    return str("offColor", "#9CA3AF");
  }
</script>

<div
  class="w-chrome"
  style:background="#FFFFFF"
  style:border="1px solid #E5E7EB"
  style:border-radius="8px"
  class:blink={bool("blink") && on}
>
  <div class="w-title"><span class="quality {quality}"></span>{str("title", "STATE")}</div>
  <div class="w-body" style:justify-content="flex-start">
    <span
      class="lamp-dot"
      class:off={!on}
      style:background={lampColor()}
      style:color={lampColor()}
    ></span>
    <span style:font-weight="700" style:color={on ? lampColor() : "#9CA3AF"}>
      {on ? str("onLabel", "ON") : str("offLabel", "OFF")}
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
    font-size: 11px;
    font-weight: 700;
    color: #6b7280;
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
  @keyframes blink-anim {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.3; }
  }
  .blink {
    animation: blink-anim 1s infinite;
  }
</style>

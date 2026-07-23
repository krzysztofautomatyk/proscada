<script lang="ts">
  import type { WidgetDef, TagValue } from "$lib/types";

  interface Props {
    widget: WidgetDef;
    tag?: TagValue | null;
    design?: boolean;
    onWrite?: (tagId: string, value: number) => void;
  }

  let { widget, tag = null, design = false, onWrite }: Props = $props();

  const cfg = $derived((widget.config ?? {}) as Record<string, unknown>);
  const str = (k: string, d = "") => String(cfg[k] ?? d);
  const num = (k: string, d = 0) => Number(cfg[k] ?? d);
  const bool = (k: string, d = false) => Boolean(cfg[k] ?? d);

  const quality = $derived(tag?.quality ?? "bad");
  const value = $derived(tag?.value ?? 0);
  const on = $derived(tag?.bool_value ?? false);

  function lampColor() {
    if (on) return str("onColor", "#16A34A");
    return str("offColor", "#9CA3AF");
  }

  function tankPct() {
    const min = num("min", 0);
    const max = num("max", 1000);
    const span = max - min || 1;
    return Math.max(0, Math.min(100, ((value - min) / span) * 100));
  }

  function tankFillColor() {
    const alarm = cfg.alarm != null ? Number(cfg.alarm) : null;
    const warn = cfg.warn != null ? Number(cfg.warn) : null;
    if (alarm != null && value >= alarm) return "#DC2626";
    if (warn != null && value >= warn) return "#EAB308";
    return str("fillColor", "#9CA3AF");
  }

  function doWrite() {
    if (!widget.tag_id || !onWrite) return;
    const v =
      str("valueKind", "number") === "bool"
        ? bool("writeBool", true)
          ? 1
          : 0
        : num("writeValue", 0);
    if (bool("confirm", true)) {
      const ok = confirm(
        `WRITE CONFIRM\nTag: ${widget.tag_id}\nValue: ${v}\n\nThis action is audited.`,
      );
      if (!ok) return;
    }
    onWrite(widget.tag_id, v);
  }
</script>

{#if widget.widget_type === "label"}
  <div
    class="w-chrome"
    style:justify-content="center"
    style:background="transparent"
    style:color={str("textColor", "#e2e8f0")}
    style:font-size="{num("fontSize", 13)}px"
    style:font-weight={str("fontWeight", "normal")}
    style:text-align={str("align", "left")}
    style:padding="4px 6px"
    style:align-items={str("align", "left") === "center" ? "center" : "flex-start"}
  >
    {str("text", "Label")}
  </div>
{:else if widget.widget_type === "panel"}
  <div
    class="w-chrome"
    style:background={str("bgColor", "#FFFFFF")}
    style:border="1px solid {str("borderColor", "#E5E7EB")}"
  >
    <div class="w-title" style:color="#6B7280">{str("title", "PANEL")}</div>
  </div>
{:else if widget.widget_type === "numeric"}
  <div
    class="w-chrome"
    style:background={str("bgColor", "#FFFFFF")}
    style:border="1px solid #E5E7EB"
    style:border-radius="8px"
  >
    <div class="w-title" style:color="#6B7280">
      <span class="quality {quality}"></span>{str("title", widget.tag_id ?? "Value")}
    </div>
    <div
      class="w-body"
      style:color={str("textColor", "#1F2937")}
      style:font-size="{num("fontSize", 18)}px"
      style:font-weight={str("fontWeight", "bold")}
      style:justify-content="flex-start"
      style:gap="6px"
    >
      {#if design && !tag}
        — —
      {:else}
        {value.toFixed(num("decimals", 0))}
        {#if str("unit")}
          <span style:opacity="0.7" style:font-size="0.75em">{str("unit")}</span>
        {/if}
      {/if}
    </div>
  </div>
{:else if widget.widget_type === "lamp"}
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
{:else if widget.widget_type === "tank"}
  <div
    class="tank-shell"
    style:background={str("bgColor", "#FFFFFF")}
    style:border="2px solid #9CA3AF"
  >
    <div class="w-title" style:position="relative" style:z-index="2" style:color="#6B7280">
      <span class="quality {quality}"></span>{str("title", "LEVEL")}
    </div>
    <div class="tank-fill" style:height="{tankPct()}%" style:background={tankFillColor()}></div>
    {#if bool("showValue", true)}
      <div class="tank-level-text" style:color="#1F2937">
        <div style:font-size="22px">{value.toFixed(0)}</div>
        <div style:font-size="11px" style:opacity="0.85">{str("unit", "cm")}</div>
      </div>
    {/if}
  </div>
{:else if widget.widget_type === "bar"}
  {@const min = num("min", 0)}
  {@const max = num("max", 100)}
  {@const pct = Math.max(0, Math.min(100, ((value - min) / (max - min || 1)) * 100))}
  <div class="w-chrome" style:background={str("bgColor", "#FFFFFF")} style:border="1px solid #E5E7EB">
    <div class="w-title" style:color="#6B7280">{str("title", "BAR")}</div>
    <div class="w-body" style:padding="6px">
      <div style:width="100%" style:height="14px" style:background="#E5E7EB" style:border-radius="3px" style:overflow="hidden">
        <div style:width="{pct}%" style:height="100%" style:background={str("fillColor", "#16A34A")}></div>
      </div>
    </div>
  </div>
{:else if widget.widget_type === "write_button"}
  <button
    class="w-chrome"
    style:background={str("bgColor", "#1F2937")}
    style:color={str("textColor", "#fff")}
    style:border="none"
    style:font-weight="700"
    style:border-radius="8px"
    style:cursor={design ? "default" : "pointer"}
    disabled={design}
    onclick={(e) => {
      e.stopPropagation();
      if (!design) doWrite();
    }}
  >
    {str("label", "WRITE")}
  </button>
{:else}
  <div class="w-chrome" style:background="#333" style:border="1px dashed #666">
    <div class="w-body">{widget.widget_type}</div>
  </div>
{/if}

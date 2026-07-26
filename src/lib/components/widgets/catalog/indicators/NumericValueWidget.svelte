<script lang="ts">
  import type { WidgetDef, TagValue } from "$lib/types";
  import { project } from "$lib/stores/app";
  import { normalizeProjectDesignSystem } from "$lib/utils/designSystem";

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

  const boundTagDef = $derived(
    widget.tag_id ? $project?.tags.find((t) => t.id === widget.tag_id) : undefined,
  );
  const title = $derived(str("title", boundTagDef?.name ?? widget.tag_id ?? "Value"));
  const decimals = $derived(
    cfg.decimals !== undefined && cfg.decimals !== ""
      ? num("decimals", 0)
      : (boundTagDef?.decimals ?? 0),
  );
  const unit = $derived(
    cfg.unit !== undefined && cfg.unit !== ""
      ? str("unit", "")
      : (boundTagDef?.unit ?? ""),
  );
  const designSystem = $derived(normalizeProjectDesignSystem($project?.design_system));
  const fontToken = $derived(
    cfg.fontTokenId && cfg.fontTokenId !== "none"
      ? designSystem.fonts.find((font) => font.id === str("fontTokenId", ""))
      : undefined,
  );
  const fontFamily = $derived(
    fontToken ? `${fontToken.family}, ${fontToken.fallback}` : str("fontFamily", "Segoe UI, system-ui, sans-serif"),
  );
  const fontSize = $derived(fontToken?.size ?? num("fontSize", 18));
  const fontWeight = $derived(fontToken?.weight ?? str("fontWeight", "bold"));
  const fontStyle = $derived(str("fontStyle", "normal"));
  const textColor = $derived(str("textColor", "#1F2937"));
  const titleColor = $derived(str("titleColor", "#6b7280"));
  const bgColor = $derived(str("bgColor", "#FFFFFF"));
  const borderRadius = $derived(num("borderRadius", 8));
  const borderColor = $derived(str("borderColor", "#E5E7EB"));
  const borderWidth = $derived(num("borderWidth", 1));
  const align = $derived(str("align", "left"));

  const quality = $derived(tag?.quality ?? (design && widget.tag_id ? "good" : "bad"));
  const value = $derived(tag?.value ?? 0);
</script>

<div
  class="numeric-card"
  style:background={bgColor}
  style:border="{borderWidth}px solid {borderColor}"
  style:border-radius="{borderRadius}px"
  style:font-family={fontFamily}
>
  {#if title}
    <div class="header" style:color={titleColor}>
      <span class="quality {quality}"></span>
      <span class="title-text">{title}</span>
    </div>
  {/if}
  <div
    class="val-body"
    style:color={textColor}
    style:font-size="{fontSize}px"
    style:font-weight={fontWeight}
    style:font-style={fontStyle}
    style:justify-content={align === "center" ? "center" : align === "right" ? "flex-end" : "flex-start"}
    style:text-align={align}
  >
    {#if design && !tag && !widget.tag_id}
      — —
    {:else if tag?.string_value !== undefined}
      {tag.string_value}
      {#if unit}
        <span class="unit-text">{unit}</span>
      {/if}
    {:else}
      {value.toFixed(decimals)}
      {#if unit}
        <span class="unit-text">{unit}</span>
      {/if}
    {/if}
  </div>
</div>

<style>
  .numeric-card {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    overflow: hidden;
    padding: 6px 8px;
    justify-content: space-between;
  }
  .header {
    font-size: 11px;
    font-weight: 700;
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .val-body {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .unit-text {
    font-size: 0.75em;
    opacity: 0.75;
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

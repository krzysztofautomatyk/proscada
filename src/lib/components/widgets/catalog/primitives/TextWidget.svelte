<script lang="ts">
  /**
   * Label — text chrome + marquee. Blink/visibility come from DynamicShell parent.
   */
  import type { WidgetDef, TagValue } from "$lib/types";
  import { project, tagMap } from "$lib/stores/app";
  import { isWidgetScrolling } from "$lib/utils/dynamics";
  import { normalizeProjectDesignSystem } from "$lib/utils/designSystem";
  import { formatNumericValue } from "$lib/components/widgets/shared/config";

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
  const decimals = $derived(
    cfg.decimals !== undefined && cfg.decimals !== ""
      ? num("decimals", 0)
      : (boundTagDef?.decimals ?? 0),
  );
  const padZeros = $derived(num("padZeros", 0));
  const unit = $derived(
    cfg.unit !== undefined && cfg.unit !== ""
      ? str("unit", "")
      : (boundTagDef?.unit ?? ""),
  );

  const rawText = $derived(str("text", "Label"));
  const displayText = $derived.by(() => {
    if (!widget.tag_id) return rawText;

    if (tag?.string_value !== undefined) {
      const strVal = tag.string_value;
      if (rawText.includes("{value}")) return rawText.replace(/\{value\}/g, strVal);
      if (rawText.includes("{val}")) return rawText.replace(/\{val\}/g, strVal);
      if (rawText === "Label" || !rawText) return strVal;
      return `${rawText} ${strVal}`;
    }

    const valNum = tag?.value ?? 0;
    const formattedVal = (design && !tag)
      ? formatNumericValue(0, decimals, padZeros)
      : formatNumericValue(valNum, decimals, padZeros);
    const valWithUnit = unit ? `${formattedVal} ${unit}` : formattedVal;

    if (rawText.includes("{value}")) {
      return rawText.replace(/\{value\}/g, valWithUnit);
    } else if (rawText.includes("{val}")) {
      return rawText.replace(/\{val\}/g, formattedVal);
    } else if (rawText === "Label" || !rawText) {
      return valWithUnit;
    } else {
      return `${rawText} ${valWithUnit}`;
    }
  });
  const designSystem = $derived(normalizeProjectDesignSystem($project?.design_system));
  const fontToken = $derived(
    cfg.fontTokenId && cfg.fontTokenId !== "none"
      ? designSystem.fonts.find((font) => font.id === str("fontTokenId", ""))
      : undefined,
  );
  const fontFamily = $derived(
    fontToken ? `${fontToken.family}, ${fontToken.fallback}` : str("fontFamily", "Segoe UI, system-ui, sans-serif"),
  );
  const fontSize = $derived(fontToken?.size ?? num("fontSize", 14));
  const fontWeight = $derived(fontToken?.weight ?? str("fontWeight", "normal"));
  const fontStyle = $derived(str("fontStyle", "normal"));
  const textColor = $derived(str("textColor", "#1f2937"));
  const bgColor = $derived(str("bgColor", "transparent"));
  const borderColor = $derived(str("borderColor", "transparent"));
  const borderWidth = $derived(num("borderWidth", 0));
  const borderRadius = $derived(num("borderRadius", 0));
  const align = $derived(str("align", "left"));
  const vAlign = $derived(str("vAlign", "center"));

  const scrolling = $derived(isWidgetScrolling(cfg, $tagMap, widget.tag_id));
  const scrollSpeedSec = $derived(Math.max(1, num("scrollSpeedSec", 8)));
  const scrollDir = $derived(str("scrollDir", "left"));
</script>

<div
  class="label-widget"
  style:background={bgColor}
  style:color={textColor}
  style:border="{borderWidth}px solid {borderColor}"
  style:border-radius="{borderRadius}px"
  style:font-family={fontFamily}
  style:font-size="{fontSize}px"
  style:font-weight={fontWeight}
  style:font-style={fontStyle}
  style:text-align={align}
  style:align-items={vAlign === "top" ? "flex-start" : vAlign === "bottom" ? "flex-end" : "center"}
  style:justify-content={align === "center" ? "center" : align === "right" ? "flex-end" : "flex-start"}
>
  {#if scrolling}
    <div class="marquee-viewport" style:text-align={align}>
      <div
        class="marquee-track"
        class:dir-right={scrollDir === "right"}
        style:animation-duration="{scrollSpeedSec}s"
      >
        <span class="marquee-text">{displayText}</span>
        <span class="marquee-text gap" aria-hidden="true">{displayText}</span>
      </div>
    </div>
  {:else}
    <span class="static-text">{displayText}</span>
  {/if}
</div>

<style>
  .label-widget {
    width: 100%;
    height: 100%;
    display: flex;
    box-sizing: border-box;
    overflow: hidden;
    position: relative;
    padding: 2px 6px;
    white-space: nowrap;
  }
  .static-text {
    width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .marquee-viewport {
    width: 100%;
    overflow: hidden;
  }
  .marquee-track {
    display: inline-flex;
    white-space: nowrap;
    animation: scada-marquee-left 8s linear infinite;
  }
  .marquee-track.dir-right {
    animation-name: scada-marquee-right;
  }
  .marquee-text.gap {
    padding-left: 3em;
  }
  @keyframes scada-marquee-left {
    0% {
      transform: translateX(0);
    }
    100% {
      transform: translateX(-50%);
    }
  }
  @keyframes scada-marquee-right {
    0% {
      transform: translateX(-50%);
    }
    100% {
      transform: translateX(0);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .marquee-track {
      animation: none !important;
    }
  }
</style>

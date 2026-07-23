<script lang="ts">
  import type { WidgetDef, TagValue } from "$lib/types";

  interface Props {
    widget: WidgetDef;
    tag?: TagValue | null;
    design?: boolean;
    onWrite?: (tagId: string, value: number) => void;
  }

  let { widget }: Props = $props();

  const cfg = $derived((widget.config ?? {}) as Record<string, unknown>);
  const str = (k: string, d = "") => String(cfg[k] ?? d);
  const num = (k: string, d = 0) => Number(cfg[k] ?? d);

  const text = $derived(str("text", "Label"));
  const fontSize = $derived(num("fontSize", 13));
  const fontWeight = $derived(str("fontWeight", "normal"));
  const textColor = $derived(str("textColor", "#1f2937"));
  const bgColor = $derived(str("bgColor", "transparent"));
  const align = $derived(str("align", "left"));
  const borderRadius = $derived(num("borderRadius", 0));
  const padding = $derived(str("padding", "4px 6px"));
</script>

<div
  class="label-widget"
  style:background={bgColor}
  style:color={textColor}
  style:font-size="{fontSize}px"
  style:font-weight={fontWeight}
  style:text-align={align}
  style:padding={padding}
  style:border-radius="{borderRadius}px"
  style:align-items={align === "center" ? "center" : align === "right" ? "flex-end" : "flex-start"}
  style:justify-content="center"
>
  {text}
</div>

<style>
  .label-widget {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    overflow: hidden;
  }
</style>

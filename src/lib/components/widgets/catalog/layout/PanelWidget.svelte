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
    style:font-weight={str("fontWeight", "800")}
    style:text-align={str("align", "left")}
  >
    {str("title", "PANEL")}
  </div>
</div>

<style>
  .w-chrome {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    padding: 8px;
  }
  .w-title {
    width: 100%;
  }
</style>

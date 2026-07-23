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
</script>

<div
  class="w-chrome"
  style:background={str("bgColor", "#FFFFFF")}
  style:border="1px solid {str("borderColor", "#E5E7EB")}"
>
  <div class="w-title" style:color="#6B7280">{str("title", "PANEL")}</div>
</div>

<style>
  .w-chrome {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    border-radius: 8px;
    padding: 8px;
  }
  .w-title {
    font-size: 11px;
    font-weight: 800;
  }
</style>

<script lang="ts">
  import type { WidgetDef, TagValue } from "$lib/types";

  interface Props {
    widget: WidgetDef;
    tag?: TagValue | null;
    design?: boolean;
    onWrite?: (tagId: string, value: number) => void;
  }

  let { widget, design = false, onWrite }: Props = $props();

  const cfg = $derived((widget.config ?? {}) as Record<string, unknown>);
  const str = (k: string, d = "") => String(cfg[k] ?? d);
  const num = (k: string, d = 0) => Number(cfg[k] ?? d);
  const bool = (k: string, d = false) => Boolean(cfg[k] ?? d);

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

<style>
  .w-chrome {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    box-sizing: border-box;
    font-size: 12px;
    transition: opacity 0.2s;
  }
  .w-chrome:hover:not(:disabled) {
    opacity: 0.9;
  }
</style>

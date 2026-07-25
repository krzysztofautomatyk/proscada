<script lang="ts">
  import type { WidgetDef, TagValue } from "$lib/types";
  import { selectedFormId } from "$lib/stores/app";
  import { get } from "svelte/store";
  import { runScriptById } from "$lib/services/scriptRuntime";

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

  async function doWrite() {
    const scriptId = str("onClickScriptId", "");
    if (scriptId) {
      try {
        await runScriptById(scriptId, {
          type: "click",
          widgetId: widget.id,
          formId: get(selectedFormId),
          tagId: widget.tag_id ?? null,
        });
      } catch {
        return;
      }
      // If script is bound, skip default write unless alsoWriteAfterScript
      if (!bool("alsoWriteAfterScript", false)) return;
    }

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
  style:border="{num("borderWidth", 0)}px solid {str("borderColor", "transparent")}"
  style:font-family={str("fontFamily", "Segoe UI, system-ui, sans-serif")}
  style:font-size="{num("fontSize", 12)}px"
  style:font-weight={str("fontWeight", "700")}
  style:font-style={str("fontStyle", "normal")}
  style:border-radius="{num("borderRadius", 8)}px"
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
    transition: opacity 0.2s;
  }
  .w-chrome:hover:not(:disabled) {
    opacity: 0.9;
  }
</style>

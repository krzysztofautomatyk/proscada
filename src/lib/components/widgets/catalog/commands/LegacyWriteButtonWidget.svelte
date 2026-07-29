<script lang="ts">
  import type { WidgetDef, TagValue } from "$lib/types";
  import { selectedFormId } from "$lib/stores/app";
  import { get } from "svelte/store";
  import { runScriptById } from "$lib/services/scriptRuntime";
  import type { ProcessWrite } from "$lib/components/widgets/shared/types";
  import { writeResultLabel } from "$lib/components/widgets/shared/config";

  interface Props {
    widget: WidgetDef;
    tag?: TagValue | null;
    design?: boolean;
    onWrite?: ProcessWrite;
  }

  let { widget, design = false, onWrite }: Props = $props();
  let status = $state("");

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
    status = "WRITE REQUESTED";
    try {
      const result = await onWrite(widget.tag_id, v);
      status = writeResultLabel(result);
    } catch (error) {
      status = `WRITE REJECTED: ${error instanceof Error ? error.message : String(error)}`;
    }
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
  <span>{str("label", "WRITE")}</span>
  {#if status}<small role="status">{status}</small>{/if}
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
    flex-direction: column;
    gap: 2px;
  }
  .w-chrome:hover:not(:disabled) {
    opacity: 0.9;
  }
  small { font-size: 8px; font-weight: 600; }
</style>

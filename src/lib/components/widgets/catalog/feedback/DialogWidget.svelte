<script lang="ts">
  import type { TagValue, WidgetDef } from "$lib/types";
  import { configOf, readBoolean, readString } from "$lib/components/widgets/shared/config";

  interface Props {
    widget: WidgetDef;
    tag?: TagValue | null;
    design?: boolean;
    onWrite?: (tagId: string, value: number) => void;
  }
  let { widget, design = false }: Props = $props();
  const config = $derived(configOf(widget));
  const title = $derived(readString(config, "title", "Confirm operator action"));
  const message = $derived(readString(config, "message", "Review the request before sending an operator action."));
  const variant = $derived(readString(config, "variant", "confirmation"));
  const modal = $derived(readBoolean(config, "modal", true));

  function emit(action: "confirm" | "cancel") {
    if (design) return;
    window.dispatchEvent(new CustomEvent("proscada:dialog-action", { detail: { action, sourceWidgetId: widget.id, dialogId: readString(config, "dialogId", widget.id) } }));
  }
</script>

<dialog open class="dialog" aria-modal={modal} aria-labelledby={`${widget.id}-title`}>
  <header><span aria-hidden="true">◇</span><strong id={`${widget.id}-title`}>{title}</strong>{#if modal}<b>MODAL</b>{/if}</header>
  <div class="content">
    <p>{message}</p>
    {#if variant === "form"}
      <label>Operator note <input aria-label="Operator note preview" placeholder="Optional note" disabled={design} /></label>
    {:else if variant === "dialog"}
      <span class="preview">Information dialog preview</span>
    {/if}
  </div>
  <footer><button class="cancel" disabled={design} onclick={() => emit("cancel")}>Cancel</button><button class="confirm" disabled={design} onclick={() => emit("confirm")}>Confirm</button></footer>
</dialog>

<style>
  .dialog { width:100%; height:100%; min-height:105px; margin:0; max-width:none; box-sizing:border-box; display:flex; flex-direction:column; border:1px solid #94a3b8; border-radius:7px; background:#fff; color:#1e293b; font:10px "Segoe UI",system-ui,sans-serif; box-shadow:0 4px 12px rgba(15,23,42,.14); } header { display:flex; align-items:center; gap:5px; padding:6px 8px; border-bottom:1px solid #e2e8f0; background:#f8fafc; } header strong { flex:1; font-size:11px; } header b { padding:2px 4px; border:1px solid #64748b; color:#475569; font-size:7px; letter-spacing:.07em; } .content { min-height:0; flex:1; padding:7px 8px; } p { margin:0; line-height:1.35; color:#475569; } label { display:flex; flex-direction:column; gap:3px; margin-top:7px; font-size:9px; font-weight:700; } input { padding:4px; border:1px solid #cbd5e1; border-radius:3px; font:9px inherit; } .preview { display:block; margin-top:7px; color:#64748b; font-size:9px; } footer { display:flex; justify-content:flex-end; gap:5px; padding:6px 8px; border-top:1px solid #e2e8f0; } button { padding:3px 8px; border:1px solid #64748b; border-radius:3px; background:#fff; color:#334155; font:800 9px inherit; cursor:pointer; } .confirm { border-color:#1e3a5f; background:#1e3a5f; color:#fff; } button:disabled { opacity:.55; cursor:default; }
</style>

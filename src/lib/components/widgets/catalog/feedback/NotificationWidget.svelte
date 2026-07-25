<script lang="ts">
  import type { TagValue, WidgetDef } from "$lib/types";
  import { configOf, readString } from "$lib/components/widgets/shared/config";

  interface Props {
    widget: WidgetDef;
    tag?: TagValue | null;
    design?: boolean;
    onWrite?: (tagId: string, value: number) => void;
  }
  let { widget, design = false }: Props = $props();
  let dismissed = $state(false);
  const config = $derived(configOf(widget));
  const type = $derived(readString(config, "type", "info"));
  const variant = $derived(readString(config, "variant", "inline"));
  const title = $derived(readString(config, "title", "Operator notification"));
  const message = $derived(readString(config, "message", "This is an informational user-interface notification."));
  const icon = $derived(type === "success" ? "✓" : type === "warning" ? "▲" : type === "error" ? "!" : "i");

  function dismiss() {
    if (!design) dismissed = true;
  }
</script>

{#if !dismissed || design}
  <section class="notification {type} {variant}" role="status" aria-live="polite">
    <span class="icon" aria-hidden="true">{icon}</span>
    <div><strong>{title}</strong><p>{message}</p><small>UI notification · not a process alarm</small></div>
    <button disabled={design} onclick={dismiss} aria-label="Dismiss notification">×</button>
  </section>
{/if}

<style>
  .notification { width:100%; height:100%; min-height:45px; box-sizing:border-box; display:flex; gap:7px; padding:7px; border:1px solid #94a3b8; border-left:4px solid #64748b; border-radius:5px; background:#f8fafc; color:#334155; font:10px "Segoe UI",system-ui,sans-serif; } .notification.toast { box-shadow:0 5px 14px rgba(15,23,42,.16); } .notification.success { border-color:#15803d; background:#f0fdf4; color:#14532d; } .notification.warning { border-color:#a16207; background:repeating-linear-gradient(135deg,#fefce8,#fefce8 5px,#fef9c3 5px,#fef9c3 10px); color:#713f12; } .notification.error { border-color:#b91c1c; background:repeating-linear-gradient(135deg,#fef2f2,#fef2f2 5px,#fee2e2 5px,#fee2e2 10px); color:#7f1d1d; } .icon { width:14px; font-size:16px; font-weight:900; text-align:center; } div { min-width:0; flex:1; } strong { display:block; font-size:10px; } p { margin:2px 0; line-height:1.25; } small { color:#64748b; font-size:8px; } button { width:17px; height:17px; padding:0; border:1px solid currentColor; border-radius:3px; background:#fff; color:inherit; font-size:13px; line-height:12px; cursor:pointer; } button:disabled { opacity:.45; cursor:default; }
</style>

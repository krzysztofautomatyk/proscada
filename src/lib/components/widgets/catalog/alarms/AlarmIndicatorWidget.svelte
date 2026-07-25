<script lang="ts">
  import type { TagValue, WidgetDef } from "$lib/types";
  import { configOf, readString } from "$lib/components/widgets/shared/config";
  import { parseAlarms, priorityRank, stateIcon, stateLabel } from "./alarmModel";

  interface Props {
    widget: WidgetDef;
    tag?: TagValue | null;
    design?: boolean;
    onWrite?: (tagId: string, value: number) => void;
  }
  let { widget, design = false }: Props = $props();
  const config = $derived(configOf(widget));
  const parsed = $derived(parseAlarms(config));
  const group = $derived(readString(config, "group", "All groups"));
  const groupAlarms = $derived(parsed.alarms.filter((alarm) => group === "All groups" || alarm.group === group));
  const active = $derived(groupAlarms.filter((alarm) => alarm.state !== "inactive"));
  const worst = $derived([...active].sort((left, right) => priorityRank(right.priority) - priorityRank(left.priority))[0]);
  const rollupState = $derived(worst?.state ?? "inactive");

  function navigate() {
    if (design) return;
    window.dispatchEvent(new CustomEvent("proscada:alarm-action", { detail: { action: "navigate", alarmId: worst?.id ?? "", sourceWidgetId: widget.id, group } }));
  }
</script>

<button class="indicator {worst?.priority ?? 'normal'} {rollupState}" disabled={design || !!parsed.error} onclick={navigate} aria-label={`Alarm indicator for ${group}: ${active.length} alarms`}>
  {#if parsed.error}
    <span class="icon">!</span><span><b>CONFIG</b><small>{parsed.error}</small></span>
  {:else}
    <span class="icon" aria-hidden="true">{stateIcon(rollupState)}</span>
    <span class="copy"><b>{group}</b><small>{active.length} active · {stateLabel(rollupState)}</small></span>
    <strong>{active.length}</strong>
  {/if}
</button>

<style>
  .indicator { width:100%; height:100%; min-height:30px; box-sizing:border-box; display:flex; align-items:center; gap:6px; padding:5px 7px; border:1px solid #94a3b8; border-radius:5px; background:#f8fafc; color:#334155; text-align:left; font:10px "Segoe UI",system-ui,sans-serif; cursor:pointer; } .indicator.high,.indicator.critical { border-color:#c2410c; background:repeating-linear-gradient(135deg,#fff7ed,#fff7ed 5px,#ffedd5 5px,#ffedd5 10px); color:#7c2d12; } .indicator.critical { border-color:#991b1b; background:repeating-linear-gradient(135deg,#fef2f2,#fef2f2 5px,#fee2e2 5px,#fee2e2 10px); color:#7f1d1d; } .indicator.active_unacked .icon { font-weight:900; } .indicator.cleared_unacked { border-style:dashed; } .indicator:disabled { cursor:default; } .icon { width:14px; font-size:16px; text-align:center; } .copy { min-width:0; flex:1; display:flex; flex-direction:column; } b { overflow:hidden; text-overflow:ellipsis; white-space:nowrap; font-size:9px; text-transform:uppercase; letter-spacing:.04em; } small { overflow:hidden; color:#64748b; font-size:8px; text-overflow:ellipsis; white-space:nowrap; } strong { font-size:15px; }
</style>

<script lang="ts">
  import type { TagValue, WidgetDef } from "$lib/types";
  import { configOf } from "$lib/components/widgets/shared/config";
  import { parseAlarms, priorityRank, stateIcon, type ConfigAlarm } from "./alarmModel";

  interface Props {
    widget: WidgetDef;
    tag?: TagValue | null;
    design?: boolean;
    onWrite?: (tagId: string, value: number) => void;
  }
  let { widget, design = false }: Props = $props();
  let notice = $state("");
  const parsed = $derived(parseAlarms(configOf(widget)));
  const active = $derived(parsed.alarms.filter((alarm) => alarm.state === "active_unacked"));
  const worst = $derived(
    [...active].sort((left, right) => priorityRank(right.priority) - priorityRank(left.priority))[0],
  );

  function action(action: "navigate" | "ack", alarm?: ConfigAlarm) {
    if (design || !alarm) return;
    window.dispatchEvent(new CustomEvent("proscada:alarm-action", { detail: { action, alarmId: alarm.id, sourceWidgetId: widget.id } }));
    if (action === "ack") notice = "ACK request sent";
  }
</script>

<section class="banner" class:normal={!worst} class:critical={worst?.priority === "critical"} class:high={worst?.priority === "high"} aria-label="Alarm banner">
  {#if parsed.error}
    <span class="icon">!</span><strong>Alarm configuration error</strong><small>{parsed.error}</small>
  {:else if worst}
    <span class="icon" aria-hidden="true">{stateIcon(worst.state)}</span>
    <div class="copy"><b>{worst.priority.toUpperCase()} · ACTIVE UNACKED</b><strong>{worst.message}</strong><small>{active.length} unacknowledged active · {worst.group}</small></div>
    <button disabled={design} onclick={() => action("navigate", worst)}>VIEW</button>
    <button disabled={design} onclick={() => action("ack", worst)}>ACK</button>
  {:else}
    <span class="icon" aria-hidden="true">○</span><div class="copy"><b>NORMAL</b><strong>No active unacknowledged alarms</strong></div>
  {/if}
  {#if notice}<span class="notice" role="status">{notice}</span>{/if}
</section>

<style>
  .banner { width:100%; height:100%; min-height:34px; box-sizing:border-box; display:flex; align-items:center; gap:7px; padding:5px 7px; border:1px solid #c2410c; border-left:5px solid #c2410c; background:repeating-linear-gradient(135deg,#fff7ed,#fff7ed 6px,#ffedd5 6px,#ffedd5 12px); color:#431407; font:10px "Segoe UI",system-ui,sans-serif; overflow:hidden; position:relative; } .banner.critical { border-color:#991b1b; background:repeating-linear-gradient(135deg,#fef2f2,#fef2f2 6px,#fee2e2 6px,#fee2e2 12px); } .banner.normal { border-color:#94a3b8; border-left-color:#64748b; background:#f8fafc; color:#334155; } .icon { font-size:17px; font-weight:900; } .copy { min-width:0; flex:1; display:flex; flex-direction:column; } .copy b { font-size:8px; letter-spacing:.05em; } .copy strong { overflow:hidden; text-overflow:ellipsis; white-space:nowrap; font-size:10px; } small { color:#64748b; font-size:8px; } button { border:1px solid currentColor; border-radius:3px; background:#fff; color:inherit; padding:3px 5px; font:800 8px inherit; cursor:pointer; } button:disabled { opacity:.5; cursor:default; } .notice { position:absolute; right:5px; bottom:2px; color:#166534; font-size:8px; font-weight:800; }
</style>

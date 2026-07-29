<script lang="ts">
  import type { TagValue, WidgetDef } from "$lib/types";
  import { configOf, readBoolean } from "$lib/components/widgets/shared/config";
  import EmptyState from "$lib/components/widgets/shared/EmptyState.svelte";
  import {
    designAlarms,
    isActive,
    parseAlarms,
    priorityRank,
    stateIcon,
    stateLabel,
    type ConfigAlarm,
  } from "./alarmModel";

  interface Props {
    widget: WidgetDef;
    tag?: TagValue | null;
    design?: boolean;
    onWrite?: (tagId: string, value: number) => void;
  }

  let { widget, design = false }: Props = $props();
  let stateFilter = $state("all");
  let priorityFilter = $state("all");
  let groupFilter = $state("all");
  let sortBy = $state("priority");

  const config = $derived(configOf(widget));
  const parsed = $derived(parseAlarms(config));
  const alarms = $derived(parsed.configured ? parsed.alarms : design ? designAlarms : []);
  const groups = $derived([...new Set(alarms.map((alarm) => alarm.group))].sort());
  const shelved = $derived(readBoolean(config, "shelved", false));
  const alarmsSuspended = $derived(
    readBoolean(config, "alarmsSuspended", false) ||
      alarms.some((alarm) => alarm.evaluationSuspended),
  );
  const filtered = $derived.by(() => {
    const result = alarms.filter(
      (alarm) =>
        (stateFilter === "all" || alarm.state === stateFilter) &&
        (priorityFilter === "all" || alarm.priority === priorityFilter) &&
        (groupFilter === "all" || alarm.group === groupFilter),
    );
    return result.sort((left, right) =>
      sortBy === "priority"
        ? priorityRank(right.priority) - priorityRank(left.priority) ||
          right.time.localeCompare(left.time)
        : right.time.localeCompare(left.time) ||
          priorityRank(right.priority) - priorityRank(left.priority),
    );
  });

  function requestAck(alarm: ConfigAlarm) {
    if (design) return;
    window.dispatchEvent(
      new CustomEvent("proscada:alarm-action", {
        detail: { action: "ack", alarmId: alarm.id, sourceWidgetId: widget.id },
      }),
    );
  }
</script>

<section class="panel" aria-label="Alarm panel">
  {#if alarmsSuspended}
    <div class="stale-banner" role="alert">
      ▲ ALARMS STALE · evaluation suspended · states are last known, not live
    </div>
  {/if}
  <header>
    <div>
      <strong>Alarmy</strong>
      <span>{alarms.filter(isActive).length} active</span>
    </div>
    <span class="legend"><b>▲</b> unacked · <b>●</b> acked</span>
  </header>

  {#if parsed.error}
    <EmptyState title="Alarm configuration error" detail={parsed.error} icon="!" />
  {:else if alarms.length === 0}
    <EmptyState
      title="No alarm data"
      detail="Configure alarms as a JSON array to display alarm instances."
      icon="○"
    />
  {:else}
    <div class="filters" aria-label="Alarm filters">
      <select aria-label="Filter state" bind:value={stateFilter}>
        <option value="all">All states</option>
        <option value="active_unacked">Active unacked</option>
        <option value="active_acked">Active acked</option>
        <option value="cleared_unacked">Cleared unacked</option>
        <option value="inactive">Inactive</option>
      </select>
      <select aria-label="Filter priority" bind:value={priorityFilter}>
        <option value="all">All priorities</option>
        <option value="critical">Critical</option>
        <option value="high">High</option>
        <option value="medium">Medium</option>
        <option value="low">Low</option>
      </select>
      <select aria-label="Filter group" bind:value={groupFilter}>
        <option value="all">All groups</option>
        {#each groups as group}<option value={group}>{group}</option>{/each}
      </select>
      <select aria-label="Sort alarms" bind:value={sortBy}>
        <option value="priority">Priority</option>
        <option value="time">Time</option>
      </select>
    </div>

    <div class="list" class:shelved aria-live="polite">
      {#each filtered as alarm (alarm.id)}
        <article
          class="row {alarm.priority} {alarm.state}"
          class:alarm-shelved={alarm.shelved}
          aria-label={`${alarm.priority} ${stateLabel(alarm.state)}: ${alarm.message}`}
        >
          <span class="state-icon" aria-hidden="true">{stateIcon(alarm.state)}</span>
          <div class="alarm-copy">
            <div><b>{alarm.priority.toUpperCase()}</b> <span>{stateLabel(alarm.state)}</span></div>
            <strong>{alarm.message}</strong>
            <small>{alarm.group} · {alarm.time}</small>
          </div>
          {#if alarm.state === "active_unacked" || alarm.state === "cleared_unacked"}
            <button disabled={design} onclick={() => requestAck(alarm)}>ACK</button>
          {/if}
          {#if alarm.shelved}<span class="row-shelved">SHELVED</span>{/if}
        </article>
      {:else}
        <p class="no-match">No alarms match the selected filters.</p>
      {/each}
      {#if shelved}<div class="shelved-overlay">SHELVED · operator review required</div>{/if}
    </div>
  {/if}
</section>

<style>
  .panel { width:100%; height:100%; box-sizing:border-box; display:flex; flex-direction:column; overflow:hidden; border:1px solid #cbd5e1; border-radius:7px; background:#fff; color:#1e293b; font:10px "Segoe UI",system-ui,sans-serif; }
  .stale-banner { padding:5px 8px; border-bottom:2px solid #991b1b; background:repeating-linear-gradient(135deg,#fef2f2,#fef2f2 6px,#fee2e2 6px,#fee2e2 12px); color:#7f1d1d; font-size:9px; font-weight:900; letter-spacing:.04em; text-align:center; }
  header { min-height:30px; padding:5px 8px; display:flex; align-items:center; justify-content:space-between; gap:6px; border-bottom:1px solid #d8dee8; background:#f8fafc; }
  header div { display:flex; align-items:baseline; gap:6px; } header strong { text-transform:uppercase; letter-spacing:.04em; font-size:10px; } header span,.legend { color:#64748b; font-size:9px; } .legend b { color:#7f1d1d; }
  .filters { display:grid; grid-template-columns:repeat(4,minmax(0,1fr)); gap:4px; padding:5px; border-bottom:1px solid #e2e8f0; background:#f8fafc; } select { min-width:0; padding:3px; border:1px solid #cbd5e1; border-radius:3px; background:#fff; color:#334155; font:9px inherit; }
  .list { position:relative; min-height:0; flex:1; overflow:auto; padding:5px; display:flex; flex-direction:column; gap:4px; } .list.shelved { opacity:.72; }
  .row { position:relative; display:flex; align-items:center; gap:6px; padding:5px; border:1px solid #cbd5e1; border-left-width:4px; border-radius:4px; background:#fff; } .row.critical { border-left-color:#991b1b; background:repeating-linear-gradient(135deg,#fff7ed,#fff7ed 5px,#ffedd5 5px,#ffedd5 10px); } .row.high { border-left-color:#c2410c; } .row.medium { border-left-color:#a16207; } .row.low { border-left-color:#475569; }
  .row.active_unacked .state-icon { color:#991b1b; } .row.active_acked .state-icon { color:#9a6700; } .row.cleared_unacked { border-style:dashed; } .row.inactive { opacity:.62; } .state-icon { width:12px; font-weight:900; text-align:center; } .alarm-copy { min-width:0; flex:1; } .alarm-copy div { display:flex; gap:5px; color:#475569; font-size:8px; } .alarm-copy strong { display:block; overflow:hidden; margin:1px 0; font-size:10px; text-overflow:ellipsis; white-space:nowrap; } small { color:#64748b; font-size:8px; }
  button { padding:3px 6px; border:1px solid #7f1d1d; border-radius:3px; background:#fff; color:#7f1d1d; font:800 9px inherit; cursor:pointer; } button:disabled { opacity:.5; cursor:default; } .row-shelved { position:absolute; top:2px; right:3px; padding:1px 3px; background:#475569; color:#fff; font-size:7px; font-weight:800; letter-spacing:.06em; } .alarm-shelved { opacity:.6; }
  .shelved-overlay { position:absolute; inset:0; display:flex; align-items:center; justify-content:center; background:repeating-linear-gradient(135deg,rgba(71,85,105,.14),rgba(71,85,105,.14) 7px,rgba(255,255,255,.25) 7px,rgba(255,255,255,.25) 14px); color:#334155; font-weight:900; letter-spacing:.08em; pointer-events:none; } .no-match { margin:7px; color:#64748b; text-align:center; }
</style>

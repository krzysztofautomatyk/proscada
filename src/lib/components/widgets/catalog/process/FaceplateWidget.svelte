<script lang="ts">
  import type { TagValue, WidgetDef } from "$lib/types";
  import { configOf, invokeWriteToTag, readBoolean, readNumber, readString, writeResultLabel } from "$lib/components/widgets/shared/config";
  import { parseAlarms, priorityRank } from "../alarms/alarmModel";
  import type { ProcessWrite } from "$lib/components/widgets/shared/types";

  interface Props {
    widget: WidgetDef;
    tag?: TagValue | null;
    design?: boolean;
    onWrite?: ProcessWrite;
    tagMap?: Map<string, TagValue>;
  }
  let { widget, tag = null, design = false, onWrite }: Props = $props();
  let transportNotice = $state("");
  const config = $derived(configOf(widget));
  const variant = $derived(readString(config, "variant", "detail"));
  const name = $derived(readString(config, "equipmentName", widget.tag_id ?? "Equipment"));
  const mode = $derived(readString(config, "mode", "AUTO"));
  const availability = $derived(
    readBoolean(config, "available", readBoolean(config, "availability", true)),
  );
  const permissive = $derived(readBoolean(config, "permissive", true));
  const local = $derived(readBoolean(config, "local", false));
  const startValue = $derived(readNumber(config, "startValue", 1));
  const stopValue = $derived(readNumber(config, "stopValue", 0));
  const startTagId = $derived(readString(config, "startTagId", ""));
  const stopTagId = $derived(readString(config, "stopTagId", ""));
  const unit = $derived(readString(config, "unit", ""));
  const alarms = $derived(parseAlarms(config));
  const activeAlarms = $derived(alarms.alarms.filter((alarm) => alarm.state !== "inactive"));
  const worstAlarm = $derived([...activeAlarms].sort((left, right) => priorityRank(right.priority) - priorityRank(left.priority))[0]);
  const baseWriteDisabled = $derived(
    design || tag?.quality !== "good" || local || !permissive || !availability || !onWrite,
  );
  const disabledReason = $derived(
    design
      ? "Design mode"
      : tag?.quality !== "good"
        ? "Feedback quality is not Good"
        : local
          ? "Local control"
          : !availability
            ? "Equipment unavailable"
            : !permissive
              ? "Permissive false"
              : !onWrite
                ? "No command transport"
                : !startTagId || !stopTagId
                  ? "Command tags not configured"
                  : "",
  );

  async function command(targetTagId: string, value: number) {
    if (baseWriteDisabled || !targetTagId) return;
    transportNotice = "COMMAND REQUESTED";
    try {
      const result = await invokeWriteToTag(targetTagId, design, onWrite, value);
      transportNotice = writeResultLabel(result, "COMMAND");
    } catch (error) {
      transportNotice = `COMMAND REJECTED: ${error instanceof Error ? error.message : String(error)}`;
    }
  }
</script>

<section class="faceplate {variant}" aria-label={`${name} faceplate`}>
  <header><div><strong>{name}</strong><small>{variant.toUpperCase()} FACEPLATE</small></div><span class="mode">◇ {mode}</span></header>
  <div class="status">
    <span class:ok={availability} class:blocked={!availability}>{availability ? "● AVAILABLE" : "▲ UNAVAILABLE"}</span>
    <span class:ok={permissive} class:blocked={!permissive}>{permissive ? "● PERMISSIVE" : "▲ PERMISSIVE BLOCKED"}</span>
    {#if local}<span class="local">◇ LOCAL</span>{/if}
  </div>
  <div class="value"><span>Current value</span><strong>{tag ? tag.value : "—"} {unit}</strong><small class="quality {tag?.quality ?? 'bad'}">{tag?.quality === "good" ? "● GOOD" : tag?.quality === "uncertain" ? "◆ UNCERTAIN" : "▲ BAD / NO DATA"}</small></div>
  {#if variant !== "compact"}
    <div class="alarm-summary" class:alarm={activeAlarms.length > 0}>{#if alarms.error}▲ Alarm config error{:else if worstAlarm}▲ {activeAlarms.length} alarm(s) · {worstAlarm.priority.toUpperCase()} {worstAlarm.message}{:else}○ No configured active alarms{/if}</div>
  {/if}
  <div class="commands"><button disabled={baseWriteDisabled || !startTagId} onclick={() => void command(startTagId, startValue)}>▶ START</button><button class="stop" disabled={baseWriteDisabled || !stopTagId} onclick={() => void command(stopTagId, stopValue)}>■ STOP</button></div>
  <p class="intent">{transportNotice || (disabledReason ? `Command inhibited: ${disabledReason}` : "Command sends transport intent; process result is not implied.")}</p>
</section>

<style>
  .faceplate { width:100%; height:100%; box-sizing:border-box; display:flex; flex-direction:column; gap:5px; padding:7px; border:1px solid #94a3b8; border-radius:7px; background:#fff; color:#1e293b; font:10px "Segoe UI",system-ui,sans-serif; overflow:hidden; } header { display:flex; justify-content:space-between; gap:6px; } header div { min-width:0; display:flex; flex-direction:column; } header strong { overflow:hidden; text-overflow:ellipsis; white-space:nowrap; font-size:11px; } header small { color:#64748b; font-size:7px; letter-spacing:.07em; } .mode,.local { padding:2px 4px; border:1px dashed #64748b; color:#475569; font-size:8px; font-weight:800; white-space:nowrap; } .status { display:flex; flex-wrap:wrap; gap:4px; } .status span { padding:2px 4px; border:1px solid #94a3b8; border-radius:3px; color:#475569; font-size:8px; font-weight:800; } .status .ok { border-color:#86efac; color:#166534; } .status .blocked { border-color:#fca5a5; color:#991b1b; background:repeating-linear-gradient(135deg,#fef2f2,#fef2f2 3px,#fee2e2 3px,#fee2e2 6px); } .value { display:grid; grid-template-columns:1fr auto; align-items:baseline; gap:2px 6px; padding:5px; border:1px solid #e2e8f0; background:#f8fafc; } .value > span { color:#64748b; font-size:8px; } .value strong { font-size:15px; } .value small { grid-column:1 / -1; font-size:8px; font-weight:800; } .quality.good { color:#166534; } .quality.uncertain { color:#854d0e; } .quality.bad { color:#991b1b; background:repeating-linear-gradient(135deg,#fef2f2,#fef2f2 3px,#fee2e2 3px,#fee2e2 6px); } .alarm-summary { overflow:hidden; padding:4px; border:1px solid #cbd5e1; color:#475569; font-size:8px; text-overflow:ellipsis; white-space:nowrap; } .alarm-summary.alarm { border-color:#c2410c; color:#7c2d12; background:repeating-linear-gradient(135deg,#fff7ed,#fff7ed 3px,#ffedd5 3px,#ffedd5 6px); } .commands { display:flex; gap:5px; } button { flex:1; padding:5px; border:1px solid #166534; border-radius:3px; background:#fff; color:#166534; font:800 9px inherit; cursor:pointer; } button.stop { border-color:#991b1b; color:#991b1b; } button:disabled { opacity:.48; cursor:default; } .intent { margin:0; color:#64748b; font-size:8px; line-height:1.25; } .compact { gap:3px; padding:5px; } .compact .value { padding:3px; } .compact .intent { display:none; } .popup { box-shadow:0 7px 18px rgba(15,23,42,.18); }
</style>

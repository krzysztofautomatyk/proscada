<script lang="ts">
  import type { AlarmInstance, AuditEntry, EngineSnapshot } from "$lib/types";
  import { logs } from "$lib/stores/app";
  import { api } from "$lib/services/api";

  interface Props {
    snapshot: EngineSnapshot | null;
    audit: AuditEntry[];
  }

  let { snapshot, audit }: Props = $props();
  let tab = $state<"output" | "alarms" | "audit">("output");

  async function ack(id: string) {
    try {
      await api.ackAlarm(id);
    } catch (e) {
      console.error(e);
    }
  }

  function stateClass(s: string) {
    if (s.includes("active")) return "active";
    if (s.includes("cleared")) return "clear";
    if (s.includes("acked")) return "acked";
    return "idle";
  }

  function priClass(p: string) {
    return p;
  }
</script>

<div class="panel" style:height="100%;border:none;border-top:1px solid var(--vs-border)">
  <div class="tabstrip">
    <button class="tab" class:active={tab === "output"} onclick={() => (tab = "output")}>
      Output
    </button>
    <button class="tab" class:active={tab === "alarms"} onclick={() => (tab = "alarms")}>
      Alarms ({snapshot?.alarms.filter((a) => a.state !== "inactive").length ?? 0})
    </button>
    <button class="tab" class:active={tab === "audit"} onclick={() => (tab = "audit")}>
      Audit Trail
    </button>
  </div>
  <div class="panel-body">
    {#if tab === "output"}
      {#each $logs as line}
        <div class="log-line {line.level}">[{line.t}] {line.msg}</div>
      {/each}
      {#if $logs.length === 0}
        <div class="log-line">Ready.</div>
      {/if}
    {:else if tab === "alarms"}
      {#each (snapshot?.alarms ?? []).filter((a) => a.state !== "inactive") as a}
        <div class="alarm-row {priClass(a.priority)}">
          <span class="pill {stateClass(a.state)}">{a.state.replaceAll("_", " ")}</span>
          <span>
            <strong>{a.name}</strong> — {a.message}
          </span>
          <span style:text-transform="uppercase" style:color="var(--vs-text-dim)">{a.priority}</span>
          <button class="primary" onclick={() => ack(a.def_id)}>Ack</button>
        </div>
      {:else}
        <div class="log-line ok">No active alarms.</div>
      {/each}
    {:else}
      {#each audit as e}
        <div class="log-line">
          [{new Date(e.ts).toLocaleTimeString()}] {e.role}/{e.actor} · {e.action} · {e.detail}
          <span style:opacity="0.5"> · {e.hash.slice(0, 10)}</span>
        </div>
      {:else}
        <div class="log-line">No audit entries yet.</div>
      {/each}
    {/if}
  </div>
</div>

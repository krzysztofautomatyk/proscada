<script lang="ts">
  import type { AlarmInstance, AuditEntry, EngineSnapshot } from "$lib/types";
  import { logs, project, navigateToValidationIssue } from "$lib/stores/app";
  import { api } from "$lib/services/api";
  import { validateProject, type ValidationIssue } from "$lib/utils/validation";

  interface Props {
    snapshot: EngineSnapshot | null;
    audit: AuditEntry[];
  }

  let { snapshot, audit }: Props = $props();
  let tab = $state<"output" | "errors" | "alarms" | "audit">("output");
  let copyFeedback = $state(false);

  const validation = $derived(validateProject($project));
  const errorCount = $derived(validation.errors.length);
  const warningCount = $derived(validation.warnings.length);

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

  function handleIssueClick(issue: ValidationIssue) {
    navigateToValidationIssue(issue);
  }

  async function copyActiveLogs() {
    let textToCopy = "";
    if (tab === "output") {
      textToCopy = $logs.map((l) => `[${l.t}] [${l.level.toUpperCase()}] ${l.msg}`).join("\n");
    } else if (tab === "errors") {
      const errs = validation.errors.map((e) => `[ERROR] ${e.path}: ${e.message}`);
      const warns = validation.warnings.map((w) => `[WARN] ${w.path}: ${w.message}`);
      textToCopy = [...errs, ...warns].join("\n") || "0 Errors, 0 Warnings";
    } else if (tab === "alarms") {
      textToCopy = (snapshot?.alarms ?? [])
        .map((a) => `[${a.priority.toUpperCase()}] ${a.name} (${a.state}): ${a.message}`)
        .join("\n");
    } else if (tab === "audit") {
      textToCopy = audit
        .map((e) => `[${new Date(e.ts).toLocaleTimeString()}] ${e.role}/${e.actor} - ${e.action}: ${e.detail}`)
        .join("\n");
    }

    if (textToCopy) {
      try {
        await navigator.clipboard.writeText(textToCopy);
        copyFeedback = true;
        setTimeout(() => (copyFeedback = false), 2000);
      } catch (e) {
        console.error("Copy failed", e);
      }
    }
  }
</script>

<div class="panel" style:height="100%;border:none;border-top:1px solid var(--vs-border)">
  <div class="tabstrip">
    <div class="tab-list">
      <button class="tab" class:active={tab === "output"} onclick={() => (tab = "output")}>
        Output
      </button>
      <button
        class="tab"
        class:active={tab === "errors"}
        class:has-errors={errorCount > 0}
        onclick={() => (tab = "errors")}
      >
        Error List ({errorCount} {errorCount === 1 ? 'Error' : 'Errors'}, {warningCount} {warningCount === 1 ? 'Warning' : 'Warnings'})
      </button>
      <button class="tab" class:active={tab === "alarms"} onclick={() => (tab = "alarms")}>
        Alarms ({snapshot?.alarms.filter((a) => a.state !== "inactive").length ?? 0})
      </button>
      <button class="tab" class:active={tab === "audit"} onclick={() => (tab = "audit")}>
        Audit Trail
      </button>
    </div>

    <div class="tabstrip-actions">
      <button
        type="button"
        class="btn-copy-logs"
        class:success={copyFeedback}
        onclick={copyActiveLogs}
        title="Kopiuj logi i zawartość aktywnej zakładki do schowka"
      >
        {copyFeedback ? "✅ Skopiowano!" : "📋 Kopiuj Logi"}
      </button>
    </div>
  </div>

  <div class="panel-body">
    {#if tab === "output"}
      {#each $logs as line}
        <div class="log-line {line.level}">[{line.t}] {line.msg}</div>
      {/each}
      {#if $logs.length === 0}
        <div class="log-line">Ready.</div>
      {/if}
    {:else if tab === "errors"}
      {#if errorCount === 0 && warningCount === 0}
        <div class="log-line ok">✅ 0 Errors, 0 Warnings — Project validation passed cleanly.</div>
      {:else}
        <div class="error-table">
          <div class="error-header">
            <span>Code / Severity</span>
            <span>Target Path</span>
            <span>Description (Click item to navigate / Przejdź do obiektu)</span>
          </div>
          {#each validation.errors as err}
            <button
              type="button"
              class="error-row err-item"
              onclick={() => handleIssueClick(err)}
              title="Click to navigate to this object in designer"
            >
              <span class="sev-badge err">⛔ Error</span>
              <span class="err-path">{err.path}</span>
              <span class="err-msg">{err.message} ➜</span>
            </button>
          {/each}
          {#each validation.warnings as warn}
            <button
              type="button"
              class="error-row warn-item"
              onclick={() => handleIssueClick(warn)}
              title="Click to navigate to this object in designer"
            >
              <span class="sev-badge warn">⚠️ Warning</span>
              <span class="err-path">{warn.path}</span>
              <span class="err-msg">{warn.message} ➜</span>
            </button>
          {/each}
        </div>
      {/if}
    {:else if tab === "alarms"}
      {#each (snapshot?.alarms ?? []).filter((a) => a.state !== "inactive") as a}
        <div class="alarm-row {a.priority}">
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

<style>
  .tabstrip {
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: var(--vs-bg-2, #252526);
    border-bottom: 1px solid var(--vs-border, #3e3e42);
    padding-right: 8px;
  }

  .tab-list {
    display: flex;
    align-items: center;
  }

  .tabstrip-actions {
    display: flex;
    align-items: center;
  }

  .btn-copy-logs {
    background: #1e293b;
    border: 1px solid #334155;
    color: #cbd5e1;
    font-size: 11px;
    font-weight: 600;
    padding: 3px 10px;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.12s ease;
  }

  .btn-copy-logs:hover {
    background: #334155;
    color: #fff;
    border-color: #475569;
  }

  .btn-copy-logs.success {
    background: #16a34a;
    border-color: #22c55e;
    color: #fff;
  }

  .has-errors {
    color: #f87171 !important;
    font-weight: 700;
  }

  .error-table {
    display: flex;
    flex-direction: column;
    font-size: 11px;
    font-family: var(--font-ui, sans-serif);
  }

  .error-header {
    display: grid;
    grid-template-columns: 100px 220px 1fr;
    gap: 10px;
    padding: 4px 10px;
    background: var(--vs-bg-3, #2d2d30);
    border-bottom: 1px solid var(--vs-border, #3e3e42);
    font-weight: 700;
    color: var(--vs-text-dim, #9d9d9d);
    text-transform: uppercase;
    font-size: 10px;
  }

  .error-row {
    display: grid;
    grid-template-columns: 100px 220px 1fr;
    gap: 10px;
    padding: 5px 10px;
    border-bottom: 1px solid var(--vs-border-soft, #2b2b2b);
    align-items: center;
    cursor: pointer;
    user-select: none;
    transition: background 0.1s ease;
    /* Rendered as a <button> so the row is keyboard-reachable. */
    width: 100%;
    text-align: left;
    appearance: none;
    background: none;
    border-left: 0;
    border-right: 0;
    border-top: 0;
    font: inherit;
    color: inherit;
  }

  .error-row:focus-visible {
    outline: 2px solid var(--vs-accent, #007acc);
    outline-offset: -2px;
  }

  .error-row:hover {
    background: var(--vs-selection, #264f78);
    color: #ffffff;
  }

  .sev-badge {
    display: inline-block;
    padding: 1px 6px;
    border-radius: 3px;
    font-size: 10px;
    font-weight: 700;
  }

  .sev-badge.err {
    background: rgba(220, 38, 38, 0.3);
    color: #fca5a5;
  }

  .sev-badge.warn {
    background: rgba(234, 179, 8, 0.3);
    color: #fef08a;
  }

  .err-path {
    font-family: var(--font-mono, monospace);
    color: #93c5fd;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .err-msg {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>

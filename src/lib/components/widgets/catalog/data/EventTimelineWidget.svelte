<script lang="ts">
  import type { WidgetRendererProps } from "$lib/components/widgets/shared/types";
  import { configOf, readString, readRecordList } from "$lib/components/widgets/shared/config";
  import WidgetCard from "$lib/components/widgets/shared/WidgetCard.svelte";
  import EmptyState from "$lib/components/widgets/shared/EmptyState.svelte";

  let { widget, tag = null }: WidgetRendererProps = $props();

  type Severity = "info" | "warning" | "critical";
  const SEVERITIES: Severity[] = ["info", "warning", "critical"];

  interface TimelineEvent {
    time: string;
    title: string;
    detail: string;
    severity: Severity;
    sortKey: number;
  }

  const cfg = $derived(configOf(widget));
  const title = $derived(readString(cfg, "title", "Event Timeline"));

  const parsed = $derived(readRecordList(cfg, "events"));
  const configError = $derived(parsed.error);

  function normSeverity(value: unknown): Severity {
    const s = String(value ?? "").toLowerCase();
    if (s === "critical" || s === "high" || s === "alarm") return "critical";
    if (s === "warning" || s === "warn" || s === "medium") return "warning";
    return "info";
  }

  const events = $derived.by<TimelineEvent[]>(() =>
    parsed.rows
      .map((row) => {
        const time = String(row["time"] ?? "");
        const parsedTime = Date.parse(time);
        return {
          time,
          title: String(row["title"] ?? "(untitled)"),
          detail: String(row["detail"] ?? ""),
          severity: normSeverity(row["severity"]),
          sortKey: Number.isNaN(parsedTime) ? Number.MAX_SAFE_INTEGER : parsedTime,
        };
      })
      .sort((a, b) => a.sortKey - b.sortKey),
  );

  let active = $state<Set<Severity>>(new Set(SEVERITIES));

  function toggle(sev: Severity) {
    const next = new Set(active);
    if (next.has(sev)) {
      if (next.size > 0) next.delete(sev);
    } else {
      next.add(sev);
    }
    active = next;
  }

  const visible = $derived(events.filter((e) => active.has(e.severity)));
  const counts = $derived.by(() => {
    const c: Record<Severity, number> = { info: 0, warning: 0, critical: 0 };
    for (const e of events) c[e.severity] += 1;
    return c;
  });
</script>

<WidgetCard {title} subtitle="{events.length} events" {tag} accent="#7c3aed">
  {#snippet actions()}
    <div class="filters" role="group" aria-label="Severity filters">
      {#each SEVERITIES as sev (sev)}
        <button
          type="button"
          class="chip {sev}"
          class:off={!active.has(sev)}
          aria-pressed={active.has(sev)}
          onclick={() => toggle(sev)}
        >{sev} {counts[sev]}</button>
      {/each}
    </div>
  {/snippet}

  <div class="timeline">
    {#if configError}
      <div class="cfg-error" role="alert"><strong>Config error</strong><span>{configError}</span></div>
    {:else if events.length === 0}
      <EmptyState title="No events" detail="Provide events JSON array in configuration" icon="◷" />
    {:else if visible.length === 0}
      <EmptyState title="All filtered" detail="Enable a severity filter to show events" icon="⚑" />
    {:else}
      <ol>
        {#each visible as ev, i (i)}
          <li class={ev.severity}>
            <span class="dot" aria-hidden="true"></span>
            <div class="content">
              <div class="row">
                <time>{ev.time || "—"}</time>
                <span class="sev {ev.severity}">{ev.severity}</span>
              </div>
              <strong>{ev.title}</strong>
              {#if ev.detail}<p>{ev.detail}</p>{/if}
            </div>
          </li>
        {/each}
      </ol>
    {/if}
  </div>
</WidgetCard>

<style>
  .timeline {
    width: 100%;
    height: 100%;
    overflow: auto;
    box-sizing: border-box;
    padding: 6px 8px;
  }
  .filters {
    display: flex;
    gap: 3px;
  }
  .chip {
    border: 1px solid #cbd5e1;
    border-radius: 999px;
    padding: 1px 6px;
    font-size: 8px;
    font-weight: 800;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    cursor: pointer;
    background: #fff;
    color: #475569;
  }
  .chip.info.off,
  .chip.warning.off,
  .chip.critical.off {
    opacity: 0.4;
    text-decoration: line-through;
  }
  .chip.info {
    border-color: #93c5fd;
    color: #1d4ed8;
  }
  .chip.warning {
    border-color: #fcd34d;
    color: #b45309;
  }
  .chip.critical {
    border-color: #fca5a5;
    color: #b91c1c;
  }
  ol {
    list-style: none;
    margin: 0;
    padding: 0 0 0 10px;
    position: relative;
  }
  ol::before {
    content: "";
    position: absolute;
    left: 4px;
    top: 4px;
    bottom: 4px;
    width: 2px;
    background: #e2e8f0;
  }
  li {
    position: relative;
    display: flex;
    gap: 8px;
    padding: 4px 0 8px;
  }
  .dot {
    position: absolute;
    left: -10px;
    top: 6px;
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: #64748b;
    border: 2px solid #fff;
  }
  li.info .dot {
    background: #2563eb;
  }
  li.warning .dot {
    background: #d97706;
  }
  li.critical .dot {
    background: #dc2626;
  }
  .content {
    margin-left: 8px;
    min-width: 0;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  time {
    color: #64748b;
    font-size: 9px;
    font-variant-numeric: tabular-nums;
  }
  .sev {
    font-size: 8px;
    font-weight: 800;
    text-transform: uppercase;
    padding: 0 4px;
    border-radius: 3px;
  }
  .sev.info {
    background: #dbeafe;
    color: #1d4ed8;
  }
  .sev.warning {
    background: #fef3c7;
    color: #b45309;
  }
  .sev.critical {
    background: #fee2e2;
    color: #b91c1c;
  }
  strong {
    display: block;
    font-size: 11px;
    color: #1e293b;
  }
  p {
    margin: 2px 0 0;
    font-size: 10px;
    color: #475569;
  }
  .cfg-error {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 8px;
    border: 1px solid #fca5a5;
    border-radius: 6px;
    background: #fef2f2;
    color: #991b1b;
    font-size: 10px;
  }
  .cfg-error strong {
    font-size: 11px;
  }
</style>

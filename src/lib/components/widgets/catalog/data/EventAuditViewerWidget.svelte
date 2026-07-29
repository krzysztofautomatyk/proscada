<script lang="ts">
  import type { WidgetRendererProps } from "$lib/components/widgets/shared/types";
  import {
    configOf,
    readString,
    readNumber,
    readRecordList,
  } from "$lib/components/widgets/shared/config";
  import WidgetCard from "$lib/components/widgets/shared/WidgetCard.svelte";
  import EmptyState from "$lib/components/widgets/shared/EmptyState.svelte";

  let { widget, tag = null }: WidgetRendererProps = $props();

  interface AuditRow {
    time: string;
    actor: string;
    role: string;
    action: string;
    detail: string;
    correlationId: string;
  }

  const cfg = $derived(configOf(widget));
  const title = $derived(readString(cfg, "title", "Audit Log"));
  const pageSize = $derived(Math.max(1, readNumber(cfg, "pageSize", 6, 1, 500)));

  const parsed = $derived(readRecordList(cfg, "rows"));
  const configError = $derived(parsed.error);

  const auditRows = $derived.by<AuditRow[]>(() =>
    parsed.rows.map((row) => ({
      time: String(row["time"] ?? ""),
      actor: String(row["actor"] ?? ""),
      role: String(row["role"] ?? ""),
      action: String(row["action"] ?? ""),
      detail: String(row["detail"] ?? ""),
      correlationId: String(row["correlationId"] ?? ""),
    })),
  );

  let filter = $state("");
  let page = $state(0);

  const filtered = $derived.by(() => {
    const q = filter.trim().toLowerCase();
    if (!q) return auditRows;
    return auditRows.filter((r) =>
      `${r.time} ${r.actor} ${r.role} ${r.action} ${r.detail} ${r.correlationId}`.toLowerCase().includes(q),
    );
  });

  const pageCount = $derived(Math.max(1, Math.ceil(filtered.length / pageSize)));
  const safePage = $derived(Math.min(page, pageCount - 1));
  const pageRows = $derived(filtered.slice(safePage * pageSize, safePage * pageSize + pageSize));

  function prev() {
    if (safePage > 0) page = safePage - 1;
  }
  function next() {
    if (safePage < pageCount - 1) page = safePage + 1;
  }
</script>

<WidgetCard {title} subtitle="{filtered.length} entries" {tag} accent="#0f766e">
  {#snippet actions()}
    <span class="immutable" title="Records are append-only">🔒 IMMUTABLE</span>
  {/snippet}

  <div class="audit">
    {#if configError}
      <div class="cfg-error" role="alert"><strong>Config error</strong><span>{configError}</span></div>
    {:else}
      <div class="toolbar">
        <input
          type="text"
          class="search"
          placeholder="Filter entries…"
          bind:value={filter}
          aria-label="Filter audit entries"
        />
      </div>
      {#if auditRows.length === 0}
        <EmptyState
          title="No authorized audit entries"
          detail="Sign in as Engineer or Administrator to read the backend trail"
          icon="🗎"
        />
      {:else if filtered.length === 0}
        <EmptyState title="No matches" detail="Adjust the text filter" icon="🔍" />
      {:else}
        <div class="scroll">
          <table>
            <thead>
              <tr>
                <th>Time</th><th>Actor</th><th>Role</th><th>Action</th><th>Detail</th><th>ID</th>
              </tr>
            </thead>
            <tbody>
              {#each pageRows as r, i (i)}
                <tr>
                  <td class="mono">{r.time}</td>
                  <td>{r.actor}</td>
                  <td>{r.role}</td>
                  <td>{r.action}</td>
                  <td>{r.detail}</td>
                  <td class="mono corr">{r.correlationId}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
        <div class="pager">
          <button type="button" onclick={prev} disabled={safePage === 0}>‹ Prev</button>
          <span>Page {safePage + 1} / {pageCount}</span>
          <button type="button" onclick={next} disabled={safePage >= pageCount - 1}>Next ›</button>
        </div>
      {/if}
    {/if}
  </div>
</WidgetCard>

<style>
  .audit {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
  }
  .immutable {
    font-size: 8px;
    font-weight: 800;
    letter-spacing: 0.04em;
    color: #0f766e;
    background: #ccfbf1;
    border: 1px solid #5eead4;
    border-radius: 4px;
    padding: 1px 5px;
  }
  .toolbar {
    display: flex;
    gap: 6px;
    padding: 5px 6px;
    border-bottom: 1px solid #eef2f7;
  }
  .search {
    flex: 1;
    border: 1px solid #cbd5e1;
    border-radius: 5px;
    padding: 3px 7px;
    font-size: 10px;
  }
  .scroll {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 10px;
  }
  th,
  td {
    padding: 3px 6px;
    border-bottom: 1px solid #eef2f7;
    text-align: left;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 160px;
  }
  th {
    position: sticky;
    top: 0;
    background: #f1f5f9;
    color: #334155;
    font-weight: 700;
  }
  .mono {
    font-family: "Cascadia Code", ui-monospace, monospace;
    font-size: 9px;
    color: #475569;
  }
  .corr {
    color: #94a3b8;
  }
  .pager {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
    padding: 4px 6px;
    border-top: 1px solid #eef2f7;
    background: #f8fafc;
    font-size: 9px;
    color: #475569;
  }
  .pager button {
    border: 1px solid #cbd5e1;
    background: #fff;
    border-radius: 4px;
    padding: 2px 8px;
    font-size: 9px;
    font-weight: 700;
    color: #334155;
    cursor: pointer;
  }
  .pager button:disabled {
    opacity: 0.45;
    cursor: default;
  }
  .cfg-error {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 8px;
    margin: auto;
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

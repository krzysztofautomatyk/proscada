<script lang="ts">
  import type { WidgetDef, TagValue } from "$lib/types";

  interface Props {
    widget: WidgetDef;
    tag?: TagValue | null;
    design?: boolean;
    onWrite?: (tagId: string, value: number) => void;
  }

  let { widget, design = false }: Props = $props();

  const mockAlarms = [
    { id: "alm_hi", name: "High Level", msg: "Level above 850 cm", severity: "high", state: "active" },
    { id: "alm_fault", name: "Pump 1 Fault", msg: "Thermal overload tripped", severity: "critical", state: "active" }
  ];
</script>

<div class="alarm-panel-card">
  <div class="header">
    <span>Alarms & Events</span>
    <button class="btn-ack" disabled={design}>ACK All</button>
  </div>
  <div class="alarm-list">
    {#each mockAlarms as a}
      <div class="alarm-row" class:critical={a.severity === "critical"}>
        <div class="alarm-info">
          <span class="badge {a.severity}">{a.severity.toUpperCase()}</span>
          <strong>{a.name}</strong>
          <p>{a.msg}</p>
        </div>
        <button class="btn-ack-single" disabled={design}>ACK</button>
      </div>
    {/each}
  </div>
</div>

<style>
  .alarm-panel-card {
    width: 100%;
    height: 100%;
    background: #ffffff;
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .header {
    background: #f9fafb;
    padding: 8px 10px;
    font-size: 11px;
    font-weight: 800;
    color: #1f2937;
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-bottom: 1px solid #e5e7eb;
  }
  .btn-ack {
    background: #dc2626;
    color: #ffffff;
    border: none;
    border-radius: 4px;
    font-size: 10px;
    font-weight: 800;
    padding: 3px 8px;
    cursor: pointer;
  }
  .btn-ack:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .alarm-list {
    flex: 1;
    overflow-y: auto;
    padding: 6px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .alarm-row {
    background: #fef2f2;
    border: 1px solid #fca5a5;
    border-radius: 6px;
    padding: 6px 8px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 11px;
  }
  .alarm-info strong {
    color: #991b1b;
  }
  .alarm-info p {
    margin: 2px 0 0;
    font-size: 10px;
    color: #b91c1c;
  }
  .badge {
    font-size: 8px;
    font-weight: 800;
    padding: 2px 4px;
    border-radius: 3px;
    margin-right: 4px;
  }
  .badge.high { background: #fee2e2; color: #dc2626; }
  .badge.critical { background: #991b1b; color: #ffffff; }

  .btn-ack-single {
    background: #ffffff;
    border: 1px solid #fca5a5;
    color: #991b1b;
    font-size: 10px;
    font-weight: 700;
    border-radius: 4px;
    padding: 2px 6px;
    cursor: pointer;
  }
  .btn-ack-single:disabled {
    opacity: 0.5;
    cursor: default;
  }
</style>

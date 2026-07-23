<script lang="ts">
  import type { WaterTankViewModel } from "$lib/stores/waterTank.svelte";

  interface Props {
    vm: WaterTankViewModel;
    onAck: (id: string) => void;
    onAckAll: () => void;
  }
  let { vm, onAck, onAckAll }: Props = $props();
</script>

<section class="alarms card" role="status" aria-live="assertive" aria-label="Alarm list">
  <header>
    <span>ALARMS</span>
    <span class="count" class:hot={vm.hmiAlarms.length > 0}>{vm.hmiAlarms.length}</span>
    <button type="button" class="ack-all" onclick={onAckAll}>Ack all</button>
  </header>
  {#if vm.hmiAlarms.length === 0}
    <div class="none">● NO ACTIVE ALARMS</div>
  {:else}
    {#each vm.hmiAlarms as alarm (alarm.id)}
      <div class="alarm" data-priority={alarm.priority}>
        <span class="prio">{alarm.priority}</span>
        <span class="text">{alarm.text}</span>
        <button type="button" disabled={alarm.acknowledged} onclick={() => onAck(alarm.id)}>
          {alarm.acknowledged ? "ACKED" : "ACK"}
        </button>
      </div>
    {/each}
  {/if}
</section>

<style>
  .card {
    background: var(--scada-panel, #fff);
    border: 1px solid var(--scada-border, #e5e7eb);
    border-radius: 10px;
    overflow: hidden;
  }
  header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 12px;
    border-bottom: 1px solid var(--scada-border, #e5e7eb);
    font-size: 11px;
    font-weight: 800;
    letter-spacing: 0.06em;
    color: var(--scada-muted, #6b7280);
  }
  .count {
    background: #e5e7eb;
    color: #4b5563;
    padding: 1px 8px;
    border-radius: 99px;
    font-size: 10px;
  }
  .count.hot {
    background: #dc2626;
    color: #fff;
  }
  .ack-all {
    margin-left: auto;
    border: 1px solid #e5e7eb;
    background: #fff;
    border-radius: 6px;
    padding: 4px 8px;
    font-size: 11px;
    font-weight: 700;
    color: #1f2937;
    cursor: pointer;
  }
  .none {
    padding: 14px;
    color: #16a34a;
    font-weight: 600;
    font-size: 12px;
  }
  .alarm {
    display: grid;
    grid-template-columns: 72px 1fr auto;
    gap: 8px;
    align-items: center;
    padding: 8px 12px;
    border-bottom: 1px solid #f3f4f6;
    border-left: 4px solid #9ca3af;
    font-size: 12px;
  }
  .alarm[data-priority="CRITICAL"] {
    border-left-color: #dc2626;
    background: #fef2f2;
  }
  .alarm[data-priority="HIGH"] {
    border-left-color: #dc2626;
  }
  .alarm[data-priority="MEDIUM"] {
    border-left-color: #eab308;
  }
  .prio {
    font-size: 9px;
    font-weight: 800;
    color: #6b7280;
  }
  .text {
    font-weight: 600;
    color: #1f2937;
  }
  .alarm button {
    border: 1px solid #e5e7eb;
    background: #fff;
    border-radius: 4px;
    padding: 3px 8px;
    font-size: 10px;
    font-weight: 700;
    cursor: pointer;
    color: #1f2937;
  }
  .alarm button:disabled {
    opacity: 0.5;
    cursor: default;
  }
</style>

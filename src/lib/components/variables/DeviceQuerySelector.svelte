<script lang="ts">
  import type { DeviceConfig } from "$lib/types";
  import type { DevicePollQuery } from "$lib/types/registerMap";

  interface Props {
    devices: DeviceConfig[];
    selectedDeviceId: string;
    selectedQueryId: string;
    queries: DevicePollQuery[];
    onSelectDevice: (deviceId: string) => void;
    onSelectQuery: (queryId: string) => void;
  }

  let {
    devices,
    selectedDeviceId,
    selectedQueryId,
    queries,
    onSelectDevice,
    onSelectQuery,
  }: Props = $props();

  const currentDevice = $derived(devices.find((d) => d.id === selectedDeviceId));
  const currentQuery = $derived(queries.find((q) => q.id === selectedQueryId));
</script>

<div class="selector-card">
  <div class="selector-row">
    <!-- Device Picker -->
    <div class="field-group">
      <label for="dev-select">
        <span class="icon">🔌</span> Sterownik PLC (Device):
      </label>
      <select
        id="dev-select"
        value={selectedDeviceId}
        onchange={(e) => onSelectDevice(e.currentTarget.value)}
      >
        {#if devices.length === 0}
          <option value="">(Brak skonfigurowanych sterowników PLC)</option>
        {/if}
        {#each devices as d}
          <option value={d.id}>
            {d.name} ({d.host}:{d.port} - Unit ID: {d.unit_id})
          </option>
        {/each}
      </select>
    </div>

    <!-- Query Block Picker -->
    <div class="field-group flex-2">
      <label for="query-select">
        <span class="icon">📡</span> Pole Zapytania Modbus (Poll Block):
      </label>
      <select
        id="query-select"
        value={selectedQueryId}
        onchange={(e) => onSelectQuery(e.currentTarget.value)}
        disabled={!selectedDeviceId || queries.length === 0}
      >
        {#if queries.length === 0}
          <option value="">(Brak zapytań dla wybranego sterownika)</option>
        {/if}
        {#each queries as q}
          <option value={q.id}>{q.displayLabel}</option>
        {/each}
      </select>
    </div>
  </div>

  {#if currentQuery && currentDevice}
    <div class="query-info-bar">
      <div class="info-chip">
        <span class="chip-label">Tabela:</span>
        <span class="chip-val table-{currentQuery.table}">
          {currentQuery.table.toUpperCase()} ({currentQuery.table === "holding" ? "4x" : currentQuery.table === "input" ? "3x" : currentQuery.table === "coil" ? "0x" : "1x"})
        </span>
      </div>

      <div class="info-chip">
        <span class="chip-label">Zakres Adresów:</span>
        <span class="chip-val mono">R{currentQuery.startAddress} .. R{currentQuery.endAddress}</span>
      </div>

      <div class="info-chip">
        <span class="chip-label">Liczba Rejestrów:</span>
        <span class="chip-val mono">{currentQuery.count}</span>
      </div>

      <div class="info-chip">
        <span class="chip-label">Interwał Odczytu:</span>
        <span class="chip-val mono">{currentQuery.pollMs ?? currentDevice.poll_ms} ms</span>
      </div>

      <div class="info-chip">
        <span class="chip-label">Dostęp Zpisu:</span>
        <span class="chip-val rw-status">
          {currentQuery.table === "holding" || currentQuery.table === "coil" ? "Read/Write (R/W)" : "Read-Only (Tylko Odczyt)"}
        </span>
      </div>
    </div>
  {/if}
</div>

<style>
  .selector-card {
    background: #141418;
    border: 1px solid #282d37;
    border-radius: 6px;
    padding: 12px 14px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .selector-row {
    display: flex;
    gap: 14px;
    align-items: center;
  }

  .field-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex: 1;
  }

  .flex-2 { flex: 1.8; }

  label {
    font-size: 11px;
    font-weight: 600;
    color: #94a3b8;
    display: flex;
    align-items: center;
    gap: 5px;
  }

  .icon { font-size: 13px; }

  select {
    background: #0f0f13;
    border: 1px solid #334155;
    border-radius: 4px;
    color: #f8fafc;
    padding: 6px 10px;
    font-size: 12px;
    outline: none;
    transition: border-color 0.15s;
  }

  select:focus {
    border-color: #22c55e;
    box-shadow: 0 0 0 2px rgba(34, 197, 94, 0.2);
  }

  select:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .query-info-bar {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    background: #1a1a22;
    border: 1px solid #272730;
    padding: 6px 10px;
    border-radius: 4px;
    font-size: 11px;
  }

  .info-chip {
    display: flex;
    align-items: center;
    gap: 4px;
    background: #0d0d11;
    padding: 2px 8px;
    border-radius: 3px;
    border: 1px solid #22222b;
  }

  .chip-label { color: #64748b; font-weight: 500; }
  .chip-val { font-weight: 600; color: #e2e8f0; }
  .mono { font-family: monospace; color: #38bdf8; }

  .chip-val.table-holding { color: #4ade80; }
  .chip-val.table-input { color: #f59e0b; }
  .chip-val.table-coil { color: #38bdf8; }
  .chip-val.table-discrete { color: #c084fc; }

  .rw-status { color: #a7f3d0; }
</style>

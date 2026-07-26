<script lang="ts">
  import type { DeviceConfig, ModbusQueryConfig } from "$lib/types";
  import {
    project,
    deviceModalState,
    addDeviceToProject,
    updateDeviceInProject,
    deleteDeviceFromProject,
    closeDeviceModal,
  } from "$lib/stores/app";
  import { api } from "$lib/services/api";
  import { uid } from "$lib/utils/projectTree";

  interface Props {
    open?: boolean;
    onClose?: () => void;
  }

  let { open: externalOpen, onClose: externalClose }: Props = $props();

  const isModalOpen = $derived(externalOpen ?? $deviceModalState.open);
  const modalMode = $derived($deviceModalState.mode ?? "add");
  const editDeviceId = $derived($deviceModalState.deviceId);

  let activeTab = $state<"params" | "queries">("params");

  // Form state
  let id = $state("");
  let originalId = $state("");
  let name = $state("");
  let host = $state("127.0.0.1");
  let port = $state(502);
  let unit_id = $state(1);
  let poll_ms = $state(1000);
  let timeout_ms = $state(2000);
  let enabled = $state(true);

  // Queries list
  let queries = $state<ModbusQueryConfig[]>([]);

  // Query sub-editor state
  let editingQueryId = $state<string | null>(null);
  let qName = $state("");
  let qTable = $state<ModbusQueryConfig["table"]>("holding");
  let qStart = $state(0);
  let qCount = $state(10);
  let qPollMs = $state<number | null>(null);
  let qEnabled = $state(true);

  // Testing & validation state
  let testStatus = $state<"idle" | "testing" | "success" | "error">("idle");
  let testMessage = $state("");
  let errorMessage = $state("");

  const presets = [
    { label: "Local Simulation PLC", icon: "🧪", name: "SIM_PLC_01", host: "127.0.0.1", port: 502, unit_id: 1, poll_ms: 1000, timeout_ms: 2000 },
    { label: "Schneider Modicon M241/M251", icon: "🏭", name: "PLC_SCHNEIDER_01", host: "192.168.1.10", port: 502, unit_id: 1, poll_ms: 500, timeout_ms: 1500 },
    { label: "Siemens S7-1200 Modbus TCP", icon: "⚙️", name: "PLC_SIEMENS_S7", host: "192.168.0.1", port: 502, unit_id: 255, poll_ms: 250, timeout_ms: 1000 },
    { label: "WAGO PFC200 / 750 Series", icon: "🔌", name: "PLC_WAGO_PFC", host: "192.168.1.20", port: 502, unit_id: 1, poll_ms: 200, timeout_ms: 1000 },
    { label: "Beckhoff CX Controller", icon: "💻", name: "PLC_BECKHOFF_CX", host: "192.168.1.100", port: 502, unit_id: 1, poll_ms: 100, timeout_ms: 800 },
  ];

  $effect(() => {
    if (isModalOpen) {
      if (modalMode === "edit" && editDeviceId) {
        const existing = $project?.devices.find((d) => d.id === editDeviceId);
        if (existing) {
          id = existing.id;
          originalId = existing.id;
          name = existing.name;
          host = existing.host;
          port = existing.port;
          unit_id = existing.unit_id;
          poll_ms = existing.poll_ms;
          timeout_ms = existing.timeout_ms;
          enabled = existing.enabled;
          queries = structuredClone(existing.queries ?? []);
        } else {
          resetForm();
        }
      } else {
        resetForm();
      }
      activeTab = $deviceModalState.initialTab ?? "params";
      editingQueryId = null;
      testStatus = "idle";
      testMessage = "";
      errorMessage = "";
    }
  });

  function resetForm() {
    const count = $project?.devices.length ?? 0;
    id = uid("dev");
    originalId = "";
    name = `Modbus_PLC_${count + 1}`;
    host = "127.0.0.1";
    port = 502;
    unit_id = 1;
    poll_ms = 1000;
    timeout_ms = 2000;
    enabled = true;
    queries = [
      {
        id: "q_holding_01",
        name: "Odczyt Rejestrów Holding 0..19",
        table: "holding",
        start_address: 0,
        count: 20,
        poll_ms: null,
        enabled: true,
      },
    ];
  }

  function handleClose() {
    if (externalClose) externalClose();
    closeDeviceModal();
  }

  function applyPreset(p: (typeof presets)[0]) {
    name = p.name;
    host = p.host;
    port = p.port;
    unit_id = p.unit_id;
    poll_ms = p.poll_ms;
    timeout_ms = p.timeout_ms;
    if (!id) id = uid("dev");
    testStatus = "idle";
    errorMessage = "";
  }

  async function handleTestConnection() {
    if (!host.trim()) {
      errorMessage = "Wymagany jest adres IP / host.";
      return;
    }
    testStatus = "testing";
    testMessage = "Testowanie nawiązania połączenia TCP...";
    try {
      const res = await api.testDevice(host.trim(), Number(port), Number(unit_id), Number(timeout_ms));
      if (res.ok) {
        testStatus = "success";
        testMessage = res.message || "Połączenie z urządzeniem osiągnięte!";
      } else {
        testStatus = "error";
        testMessage = res.message || "Błąd połączenia lub przekroczono limit czasu.";
      }
    } catch (e: unknown) {
      testStatus = "error";
      testMessage = e instanceof Error ? e.message : String(e);
    }
  }

  // Query Management
  function startAddQuery() {
    editingQueryId = "new";
    qName = `Zapytanie Block ${queries.length + 1}`;
    qTable = "holding";
    qStart = 0;
    qCount = 10;
    qPollMs = null;
    qEnabled = true;
  }

  function startEditQuery(q: ModbusQueryConfig) {
    editingQueryId = q.id;
    qName = q.name;
    qTable = q.table;
    qStart = q.start_address;
    qCount = q.count;
    qPollMs = q.poll_ms ?? null;
    qEnabled = q.enabled;
  }

  function saveQuery() {
    if (!qName.trim()) return;
    const item: ModbusQueryConfig = {
      id: editingQueryId === "new" ? `query_${Date.now().toString(36)}` : (editingQueryId as string),
      name: qName.trim(),
      table: qTable,
      start_address: Number(qStart) || 0,
      count: Math.max(1, Number(qCount) || 1),
      poll_ms: qPollMs ? Number(qPollMs) : null,
      enabled: qEnabled,
    };

    if (editingQueryId === "new") {
      queries = [...queries, item];
    } else {
      queries = queries.map((q) => (q.id === editingQueryId ? item : q));
    }
    editingQueryId = null;
  }

  function deleteQuery(qId: string) {
    queries = queries.filter((q) => q.id !== qId);
  }

  function getTagsInQuery(q: ModbusQueryConfig) {
    return ($project?.tags ?? []).filter(
      (t) =>
        t.device_id === (id || originalId) &&
        t.binding.table === q.table &&
        t.binding.address >= q.start_address &&
        t.binding.address < q.start_address + q.count,
    );
  }

  function handleDeleteDevice() {
    if (!originalId) return;
    if (confirm(`Czy na pewno chcesz usunąć urządzenie '${name}' (${originalId}) z projektu?`)) {
      deleteDeviceFromProject(originalId);
      handleClose();
    }
  }

  function handleSubmit(e?: Event) {
    if (e) e.preventDefault();
    errorMessage = "";

    const trimmedId = id.trim() || uid("dev");
    const trimmedName = name.trim();
    const trimmedHost = host.trim();

    if (!trimmedName) {
      errorMessage = "Nazwa urządzenia jest wymagana.";
      return;
    }
    if (!trimmedHost) {
      errorMessage = "Adres IP / Host jest wymagany.";
      return;
    }
    if (port <= 0 || port > 65535) {
      errorMessage = "Port TCP musi znajdować się w przedziale 1..65535.";
      return;
    }
    if (unit_id < 0 || unit_id > 247) {
      errorMessage = "Modbus Unit ID / Slave ID musi być w przedziale 0..247.";
      return;
    }
    if (poll_ms < 50) {
      errorMessage = "Interwał odpytywania musi wynosić min. 50 ms.";
      return;
    }

    // Check duplicate ID if ID changed or in add mode
    if (modalMode === "add" || trimmedId !== originalId) {
      if (($project?.devices ?? []).some((d) => d.id === trimmedId)) {
        errorMessage = `Urządzenie o ID '${trimmedId}' już istnieje w projekcie.`;
        return;
      }
    }

    const device: DeviceConfig = {
      id: trimmedId,
      name: trimmedName,
      host: trimmedHost,
      port: Number(port),
      unit_id: Number(unit_id),
      poll_ms: Number(poll_ms),
      timeout_ms: Number(timeout_ms),
      enabled,
      queries,
    };

    if (modalMode === "edit" && originalId) {
      updateDeviceInProject(originalId, device);
    } else {
      addDeviceToProject(device);
    }

    handleClose();
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      handleClose();
    } else if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
      handleSubmit();
    }
  }
</script>

{#if isModalOpen}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="backdrop" onpointerdown={handleClose}></div>
  <!-- svelte-ignore a11y_interactive_supports_focus -->
  <div class="modal" role="dialog" aria-labelledby="dev-modal-title" aria-modal="true" tabindex="-1" onkeydown={handleKeyDown}>
    <div class="modal-header">
      <div class="title-wrap">
        <span class="header-icon">🔌</span>
        <div>
          <h3 id="dev-modal-title">
            {modalMode === "edit" ? `Edycja Urządzenia: ${name}` : "Dodaj Nowe Urządzenie Modbus TCP"}
          </h3>
          <p class="subtitle">Konfiguracja parametrów protokołu Modbus TCP / RTU-over-TCP oraz zapytań cyklicznych</p>
        </div>
      </div>
      <button type="button" class="close-btn" onclick={handleClose} title="Zamknij (Esc)">✕</button>
    </div>

    <!-- Navigation Tabs -->
    <div class="tab-bar">
      <button type="button" class="tab-btn" class:active={activeTab === "params"} onclick={() => (activeTab = "params")}>
        ⚙️ Połączenie & Parametry (TCP Settings)
      </button>
      <button type="button" class="tab-btn" class:active={activeTab === "queries"} onclick={() => (activeTab = "queries")}>
        📡 Definicje Zapytań Modbus ({queries.length})
      </button>
    </div>

    <div class="modal-body">
      {#if errorMessage}
        <div class="error-banner">
          <span>⚠️ {errorMessage}</span>
        </div>
      {/if}

      {#if activeTab === "params"}
        <!-- Connection Presets -->
        {#if modalMode === "add"}
          <div class="presets-section">
            <span class="presets-label">⚡ Szybkie Szablony PLC (Hardware Presets):</span>
            <div class="presets-grid">
              {#each presets as p}
                <button type="button" class="preset-card" onclick={() => applyPreset(p)} title="Wypełnij dane dla {p.label}">
                  <span class="preset-icon">{p.icon}</span>
                  <span class="preset-name">{p.label}</span>
                </button>
              {/each}
            </div>
          </div>
        {/if}

        <form onsubmit={handleSubmit} class="form-grid">
          <div class="form-group">
            <label for="dev-id">Identyfikator Urządzenia (Device ID):</label>
            <input id="dev-id" type="text" bind:value={id} placeholder="e.g. dev_plc_01" required />
            <small>Unikalne ID systemowe używane w przypisaniach tagów.</small>
          </div>

          <div class="form-group">
            <label for="dev-name">Nazwa Urządzenia (Friendly Name):</label>
            <input id="dev-name" type="text" bind:value={name} placeholder="e.g. Sterownik Główny Pompowni" required />
          </div>

          <div class="form-group">
            <label for="dev-host">Adres IP / Host (IP Address):</label>
            <input id="dev-host" type="text" bind:value={host} placeholder="127.0.0.1 lub plc1.local" required />
          </div>

          <div class="form-group">
            <label for="dev-port">Port TCP (Default: 502):</label>
            <input id="dev-port" type="number" min="1" max="65535" bind:value={port} required />
          </div>

          <div class="form-group">
            <label for="dev-unit">Unit ID / Slave ID (0..247 / 255):</label>
            <input id="dev-unit" type="number" min="0" max="255" bind:value={unit_id} required />
            <small>Modbus Slave Address (np. 1 dla PLC, 255 dla Siemens S7).</small>
          </div>

          <div class="form-group">
            <label for="dev-poll">Domyślny Interwał Odczytu (Poll ms):</label>
            <input id="dev-poll" type="number" min="50" step="50" bind:value={poll_ms} required />
            <small>Częstotliwość odpytywania w milisekundach.</small>
          </div>

          <div class="form-group">
            <label for="dev-timeout">Timeout Odpowiedzi (Timeout ms):</label>
            <input id="dev-timeout" type="number" min="100" step="100" bind:value={timeout_ms} required />
          </div>

          <div class="form-group checkbox-group">
            <label class="checkbox-label">
              <input type="checkbox" bind:checked={enabled} />
              <span><strong>Aktywne w Runtime (Enabled)</strong></span>
            </label>
            <small>Odznacz, aby wyłączyć urządzenie z komunikacji.</small>
          </div>
        </form>

        <!-- Test Connection Box -->
        <div class="test-box">
          <div class="test-header">
            <span>🔍 Test Połączenia Sieciowego Modbus TCP</span>
            <button type="button" class="btn-test" disabled={testStatus === "testing"} onclick={handleTestConnection}>
              {testStatus === "testing" ? "Łączenie..." : "🔌 Testuj Połączenie"}
            </button>
          </div>
          {#if testStatus !== "idle"}
            <div class="test-result {testStatus}">
              {#if testStatus === "testing"}
                <span class="spinner">⏳</span> {testMessage}
              {:else if testStatus === "success"}
                <span class="icon">✅</span> {testMessage}
              {:else if testStatus === "error"}
                <span class="icon">❌</span> {testMessage}
              {/if}
            </div>
          {/if}
        </div>
      {:else}
        <!-- Queries Tab -->
        <div class="queries-container">
          <div class="queries-head">
            <div>
              <h4 class="sub-title">Definicje Zapytań Blokowych Modbus TCP</h4>
              <p class="sub-desc">Grupy ciągłych rejestrów odczytywanych w jednej kwerendzie Modbus FC01-FC04</p>
            </div>
            <button type="button" class="btn-primary-sm" onclick={startAddQuery}>➕ Dodaj Zapytanie Blokowe</button>
          </div>

          {#if editingQueryId}
            <!-- Sub-form query editor -->
            <div class="query-edit-card">
              <h5 class="card-title">{editingQueryId === "new" ? "Nowe Zapytanie Blokowe" : "Edycja Zapytania"}</h5>
              <div class="form-grid">
                <div class="form-group">
                  <label for="q-name">Nazwa Zapytania:</label>
                  <input id="q-name" type="text" bind:value={qName} placeholder="np. Odczyt Analogów HR100-110" />
                </div>
                <div class="form-group">
                  <label for="q-table">Tabela Modbus / Funkcja:</label>
                  <select id="q-table" bind:value={qTable}>
                    <option value="holding">Holding Register (4x / FC03)</option>
                    <option value="input">Input Register (3x / FC04)</option>
                    <option value="coil">Coils (0x / FC01)</option>
                    <option value="discrete">Discrete Inputs (1x / FC02)</option>
                  </select>
                </div>
                <div class="form-group">
                  <label for="q-start">Adres Początkowy (Start Address):</label>
                  <input id="q-start" type="number" min="0" max="65535" bind:value={qStart} />
                </div>
                <div class="form-group">
                  <label for="q-count">Liczba Rejestrów / Bitów (Count):</label>
                  <input id="q-count" type="number" min="1" max="125" bind:value={qCount} />
                </div>
                <div class="form-group">
                  <label for="q-poll">Dedykowany Interwał ms (Opcjonalny):</label>
                  <input id="q-poll" type="number" min="50" placeholder="Domyślny z urządzenia ({poll_ms} ms)" bind:value={qPollMs} />
                </div>
                <div class="form-group checkbox-group">
                  <label class="checkbox-label">
                    <input type="checkbox" bind:checked={qEnabled} />
                    <span>Zapytanie Aktywne (Enabled)</span>
                  </label>
                </div>
              </div>
              <div class="card-actions">
                <button type="button" class="btn-cancel" onclick={() => (editingQueryId = null)}>Anuluj</button>
                <button type="button" class="btn-primary" onclick={saveQuery}>Zapisz Zapytanie</button>
              </div>
            </div>
          {/if}

          <div class="queries-list">
            {#if queries.length === 0}
              <div class="empty-hint">Brak zdefiniowanych zapytań blokowych. Urządzenie będzie używać automatycznego odpytywania pojedynczych zmiennych.</div>
            {:else}
              {#each queries as q (q.id)}
                {@const mappedTags = getTagsInQuery(q)}
                <div class="query-card" class:disabled={!q.enabled}>
                  <div class="q-header">
                    <div class="q-title-wrap">
                      <span class="q-icon">📡</span>
                      <strong>{q.name}</strong>
                      <span class="q-badge {q.table}">{q.table.toUpperCase()}</span>
                      {#if !q.enabled}<span class="badge-disabled">WYŁĄCZONE</span>{/if}
                    </div>
                    <div class="q-actions">
                      <button type="button" class="btn-icon" title="Edytuj" onclick={() => startEditQuery(q)}>✏️</button>
                      <button type="button" class="btn-icon danger" title="Usuń" onclick={() => deleteQuery(q.id)}>🗑️</button>
                    </div>
                  </div>

                  <div class="q-details">
                    <span>Adresy: <strong class="mono">{q.start_address} .. {q.start_address + q.count - 1}</strong> ({q.count} reg.)</span>
                    <span>Interwał: <strong>{q.poll_ms ? `${q.poll_ms} ms` : `${poll_ms} ms (domyślny)`}</strong></span>
                    <span>Dopasowane Zmienne: <strong class="tag-count">{mappedTags.length} tag(i)</strong></span>
                  </div>

                  {#if mappedTags.length > 0}
                    <div class="mapped-tags-preview">
                      <span class="preview-lbl">Zawarte tagi:</span>
                      <div class="tag-chips">
                        {#each mappedTags as t}
                          <span class="tag-chip" title="{t.id} · Adres: {t.binding.address}">{t.name} ({t.binding.address})</span>
                        {/each}
                      </div>
                    </div>
                  {/if}
                </div>
              {/each}
            {/if}
          </div>
        </div>
      {/if}
    </div>

    <div class="modal-footer">
      <div>
        {#if modalMode === "edit"}
          <button type="button" class="btn-delete" onclick={handleDeleteDevice}>🗑️ Usuń Urządzenie</button>
        {:else}
          <span class="hint">Naciśnij <kbd>Ctrl+Enter</kbd>, aby zapisać.</span>
        {/if}
      </div>
      <div class="footer-actions">
        <button type="button" class="btn-cancel" onclick={handleClose}>Anuluj</button>
        <button type="button" class="btn-primary" onclick={() => handleSubmit()}>
          💾 {modalMode === "edit" ? "Zapisz Zmiany" : "Dodaj Urządzenie"}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed; inset: 0; background: rgba(0, 0, 0, 0.65);
    backdrop-filter: blur(4px); z-index: 9998; animation: fade-in 0.15s ease-out;
  }
  .modal {
    position: fixed; top: 50%; left: 50%; transform: translate(-50%, -50%);
    width: min(720px, 95vw); max-height: 90vh; background: #1e1e24;
    border: 1px solid #3b82f6; border-radius: 8px; box-shadow: 0 20px 50px rgba(0, 0, 0, 0.7);
    color: #e2e8f0; z-index: 9999; display: flex; flex-direction: column;
    overflow: hidden; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    animation: modal-pop 0.15s ease-out;
  }
  .modal-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 14px 18px; background: #252530; border-bottom: 1px solid #334155;
  }
  .title-wrap { display: flex; align-items: center; gap: 12px; }
  .header-icon { font-size: 24px; }
  .title-wrap h3 { margin: 0; font-size: 15px; font-weight: 700; color: #f8fafc; }
  .subtitle { margin: 2px 0 0; font-size: 11px; color: #94a3b8; }
  .close-btn {
    background: transparent; border: none; color: #94a3b8; font-size: 18px;
    cursor: pointer; padding: 4px 8px; border-radius: 4px; transition: all 0.12s;
  }
  .close-btn:hover { background: #ef4444; color: #fff; }

  .tab-bar { display: flex; background: #141418; border-bottom: 1px solid #2d3748; }
  .tab-btn {
    flex: 1; padding: 10px 14px; background: transparent; border: none;
    border-bottom: 2px solid transparent; color: #94a3b8; font-size: 12px;
    font-weight: 600; cursor: pointer; transition: all 0.12s;
  }
  .tab-btn.active { color: #60a5fa; border-bottom-color: #3b82f6; background: #1e1e24; }

  .modal-body {
    padding: 16px 18px; overflow-y: auto; display: flex; flex-direction: column;
    gap: 16px; max-height: 65vh;
  }
  .presets-section { background: #141418; border: 1px solid #282d37; border-radius: 6px; padding: 10px 12px; }
  .presets-label { font-size: 11px; font-weight: 700; color: #60a5fa; text-transform: uppercase; display: block; margin-bottom: 8px; }
  .presets-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(110px, 1fr)); gap: 6px; }
  .preset-card {
    display: flex; flex-direction: column; align-items: center; gap: 4px; padding: 6px 4px;
    background: #1e1e28; border: 1px solid #334155; border-radius: 5px; color: #cbd5e1;
    cursor: pointer; font-size: 10px; text-align: center; transition: all 0.12s;
  }
  .preset-card:hover { background: #2563eb; border-color: #60a5fa; color: #fff; transform: translateY(-1px); }
  .preset-icon { font-size: 14px; }
  .error-banner { background: rgba(239, 68, 68, 0.15); border: 1px solid #ef4444; color: #fca5a5; padding: 8px 12px; border-radius: 6px; font-size: 12px; }
  .form-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px 16px; }
  .form-group { display: flex; flex-direction: column; gap: 4px; }
  .form-group label { font-size: 12px; font-weight: 600; color: #cbd5e1; }
  .form-group input, .form-group select {
    background: #121216; border: 1px solid #334155; border-radius: 4px;
    color: #f8fafc; padding: 7px 10px; font-size: 12px; outline: none;
  }
  .form-group input:focus, .form-group select:focus { border-color: #3b82f6; box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.25); }
  .form-group small { font-size: 10px; color: #64748b; }
  .checkbox-group { grid-column: span 2; flex-direction: column; margin-top: 4px; }
  .checkbox-label { display: flex; align-items: center; gap: 8px; cursor: pointer; font-size: 12px; }
  .checkbox-label input { width: 16px; height: 16px; accent-color: #3b82f6; }

  .test-box { background: #14141a; border: 1px solid #2d3748; border-radius: 6px; padding: 10px 12px; display: flex; flex-direction: column; gap: 8px; }
  .test-header { display: flex; align-items: center; justify-content: space-between; font-size: 12px; font-weight: 600; color: #94a3b8; }
  .btn-test { background: #1e293b; border: 1px solid #3b82f6; color: #60a5fa; padding: 5px 12px; border-radius: 4px; font-size: 11px; font-weight: 600; cursor: pointer; }
  .btn-test:hover:not(:disabled) { background: #2563eb; color: #fff; }
  .test-result { font-size: 11px; padding: 6px 10px; border-radius: 4px; display: flex; align-items: center; gap: 6px; }
  .test-result.testing { background: rgba(59, 130, 246, 0.1); color: #60a5fa; }
  .test-result.success { background: rgba(34, 197, 94, 0.15); color: #4ade80; border: 1px solid #22c55e; }
  .test-result.error { background: rgba(239, 68, 68, 0.15); color: #fca5a5; border: 1px solid #ef4444; }

  /* Queries tab styling */
  .queries-container { display: flex; flex-direction: column; gap: 14px; }
  .queries-head { display: flex; align-items: center; justify-content: space-between; }
  .sub-title { margin: 0; font-size: 13px; font-weight: 700; color: #60a5fa; }
  .sub-desc { margin: 2px 0 0; font-size: 11px; color: #94a3b8; }
  .btn-primary-sm { background: #2563eb; border: 1px solid #3b82f6; color: #fff; padding: 5px 10px; font-size: 11px; font-weight: 600; border-radius: 4px; cursor: pointer; }
  .btn-primary-sm:hover { background: #1d4ed8; }

  .query-edit-card { background: #14141a; border: 1px solid #3b82f6; border-radius: 6px; padding: 12px; display: flex; flex-direction: column; gap: 10px; }
  .card-title { margin: 0; font-size: 12px; font-weight: 700; color: #f8fafc; }
  .card-actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 4px; }

  .queries-list { display: flex; flex-direction: column; gap: 8px; }
  .query-card { background: #141418; border: 1px solid #282d37; border-radius: 6px; padding: 10px 12px; display: flex; flex-direction: column; gap: 6px; }
  .query-card.disabled { opacity: 0.6; }
  .q-header { display: flex; align-items: center; justify-content: space-between; }
  .q-title-wrap { display: flex; align-items: center; gap: 8px; font-size: 12px; color: #fff; }
  .q-badge { font-size: 10px; font-weight: 700; padding: 2px 6px; border-radius: 3px; background: #334155; color: #93c5fd; }
  .q-badge.holding { background: #1e3a8a; color: #93c5fd; }
  .q-badge.input { background: #065f46; color: #a7f3d0; }
  .badge-disabled { font-size: 9px; padding: 1px 4px; background: #7f1d1d; color: #fca5a5; border-radius: 3px; }
  .q-actions { display: flex; gap: 4px; }
  .btn-icon { background: transparent; border: none; cursor: pointer; font-size: 12px; padding: 2px 4px; border-radius: 3px; }
  .btn-icon:hover { background: #334155; }
  .q-details { display: flex; gap: 16px; font-size: 11px; color: #cbd5e1; }
  .mono { font-family: monospace; }
  .tag-count { color: #60a5fa; }
  .mapped-tags-preview { display: flex; align-items: center; gap: 8px; font-size: 10px; background: #181820; padding: 4px 8px; border-radius: 4px; }
  .preview-lbl { color: #94a3b8; }
  .tag-chips { display: flex; flex-wrap: wrap; gap: 4px; }
  .tag-chip { background: #272730; color: #e2e8f0; padding: 1px 6px; border-radius: 3px; font-size: 10px; }

  .modal-footer {
    padding: 12px 18px; background: #18181c; border-top: 1px solid #2d3748;
    display: flex; align-items: center; justify-content: space-between;
  }
  .hint { font-size: 11px; color: #64748b; }
  .footer-actions { display: flex; gap: 10px; }
  .btn-cancel { background: #27272a; border: 1px solid #3f3f46; color: #cbd5e1; padding: 7px 14px; border-radius: 5px; font-size: 12px; cursor: pointer; }
  .btn-delete { background: rgba(239, 68, 68, 0.2); border: 1px solid #ef4444; color: #fca5a5; padding: 7px 14px; border-radius: 5px; font-size: 12px; cursor: pointer; }
  .btn-delete:hover { background: #dc2626; color: #fff; }
  .btn-primary { background: #2563eb; border: 1px solid #3b82f6; color: #fff; padding: 7px 16px; border-radius: 5px; font-size: 12px; font-weight: 600; cursor: pointer; }
  .btn-primary:hover { background: #1d4ed8; }

  @keyframes fade-in { from { opacity: 0; } to { opacity: 1; } }
  @keyframes modal-pop { from { opacity: 0; transform: translate(-50%, -48%) scale(0.96); } to { opacity: 1; transform: translate(-50%, -50%) scale(1); } }
</style>

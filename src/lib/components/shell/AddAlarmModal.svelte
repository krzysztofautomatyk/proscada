<script lang="ts">
  import type { AlarmDefinition, AlarmGroupDefinition, AlarmPriority, TagDefinition } from "$lib/types";
  import { project, addAlarmToProject, addAlarmsToProject } from "$lib/stores/app";
  import { uid } from "$lib/utils/projectTree";

  interface Props {
    open: boolean;
    onClose: () => void;
  }

  let { open, onClose }: Props = $props();

  type Mode = "single" | "batch";
  let activeTab = $state<Mode>("single");

  // Single Alarm form state
  let alarmId = $state("");
  let alarmName = $state("");
  let tagId = $state("");
  let groupId = $state("");
  let newGroupName = $state("");
  let isNewGroup = $state(false);
  let priority = $state<AlarmPriority>("medium");
  let conditionType = $state<"digital_true" | "hi_limit" | "lo_limit">("digital_true");
  let hiLimit = $state<number | null>(100);
  let loLimit = $state<number | null>(0);
  let deadband = $state(0);
  let onDelayMs = $state(0);
  let offDelayMs = $state(0);
  let latching = $state(false);
  let message = $state("");
  let errorMessage = $state("");

  // Batch Alarm Generator state
  let batchDeviceFilter = $state("");
  let batchSearchQuery = $state("");
  let batchAlarmType = $state<"high_limit" | "low_limit" | "digital_fault">("high_limit");
  let batchPriority = $state<AlarmPriority>("high");
  let batchGroupId = $state("");
  let batchHiLimit = $state(80);
  let batchLoLimit = $state(20);
  let batchLatching = $state(true);
  let selectedTagIds = $state<string[]>([]);

  const tags = $derived($project?.tags ?? []);
  const devices = $derived($project?.devices ?? []);
  const groups = $derived($project?.alarm_groups ?? []);

  const filteredBatchTags = $derived(
    tags.filter((t) => {
      if (batchDeviceFilter && t.device_id !== batchDeviceFilter) return false;
      if (!batchSearchQuery.trim()) return true;
      const q = batchSearchQuery.toLowerCase();
      return t.name.toLowerCase().includes(q) || t.id.toLowerCase().includes(q);
    }),
  );

  $effect(() => {
    if (open) {
      resetForm();
    }
  });

  function resetForm() {
    activeTab = "single";
    const existingCount = $project?.alarms.length ?? 0;
    alarmId = uid("alarm");
    alarmName = `ALM_RULE_${existingCount + 1}`;
    tagId = tags[0]?.id ?? "";
    groupId = groups[0]?.id ?? "";
    newGroupName = "";
    isNewGroup = false;
    priority = "medium";
    conditionType = "digital_true";
    hiLimit = 100;
    loLimit = 0;
    deadband = 0;
    onDelayMs = 0;
    offDelayMs = 0;
    latching = false;
    message = "Alarm aktywowany - wymagana reakcja operatora";
    errorMessage = "";

    // Reset batch
    batchDeviceFilter = "";
    batchSearchQuery = "";
    batchAlarmType = "high_limit";
    batchPriority = "high";
    batchGroupId = groups[0]?.id ?? "";
    batchHiLimit = 80;
    batchLoLimit = 20;
    batchLatching = true;
    selectedTagIds = tags.slice(0, 5).map((t) => t.id);
  }

  function handleNameChange() {
    if (!alarmId || alarmId.startsWith("alarm_")) {
      const slug = alarmName
        .trim()
        .toLowerCase()
        .replace(/[^a-z0-9_]/g, "_");
      if (slug) alarmId = `alm_${slug}`;
    }
  }

  function toggleSelectAllBatch() {
    if (selectedTagIds.length === filteredBatchTags.length) {
      selectedTagIds = [];
    } else {
      selectedTagIds = filteredBatchTags.map((t) => t.id);
    }
  }

  function toggleTagSelect(tId: string) {
    if (selectedTagIds.includes(tId)) {
      selectedTagIds = selectedTagIds.filter((id) => id !== tId);
    } else {
      selectedTagIds = [...selectedTagIds, tId];
    }
  }

  function handleSubmitSingle(e?: Event) {
    if (e) e.preventDefault();
    errorMessage = "";

    const trimmedId = alarmId.trim() || uid("alarm");
    const trimmedName = alarmName.trim();

    if (!trimmedName) {
      errorMessage = "Nazwa alarmu jest wymagana.";
      return;
    }

    if (!tagId) {
      errorMessage = "Wybierz zmienną (Tag ID) dla alarmu.";
      return;
    }

    if (($project?.alarms ?? []).some((a) => a.id === trimmedId)) {
      errorMessage = `Alarm o ID '${trimmedId}' już istnieje w projekcie.`;
      return;
    }

    let finalGroupId = groupId;
    let newGroupDef: AlarmGroupDefinition | undefined = undefined;

    if (isNewGroup && newGroupName.trim()) {
      finalGroupId = `group-${Date.now().toString(36)}`;
      newGroupDef = {
        id: finalGroupId,
        name: newGroupName.trim(),
        parent_id: null,
        object_id: null,
        description: "Utworzono z kreatora alarmów",
      };
    }

    const alarm: AlarmDefinition = {
      id: trimmedId,
      name: trimmedName,
      tag_id: tagId,
      group_id: finalGroupId || undefined,
      priority,
      when_true: conditionType === "digital_true",
      hi_limit: conditionType === "hi_limit" ? Number(hiLimit) : null,
      lo_limit: conditionType === "lo_limit" ? Number(loLimit) : null,
      deadband: Number(deadband) || 0,
      on_delay_ms: Number(onDelayMs) || 0,
      off_delay_ms: Number(offDelayMs) || 0,
      latching,
      message: message.trim() || "Wyeliminuj przyczynę alarmu i zatwierdź (ACK).",
    };

    addAlarmToProject(alarm, newGroupDef);
    onClose();
  }

  function handleSubmitBatch() {
    errorMessage = "";
    if (selectedTagIds.length === 0) {
      errorMessage = "Wybierz przynajmniej jedną zmienną z listy.";
      return;
    }

    const selectedTagsList = tags.filter((t) => selectedTagIds.includes(t.id));
    const generatedAlarms: AlarmDefinition[] = [];

    for (const tag of selectedTagsList) {
      let aName = "";
      let aMsg = "";
      let hiVal: number | null = null;
      let loVal: number | null = null;

      if (batchAlarmType === "high_limit") {
        aName = `ALM_HI_${tag.name}`;
        aMsg = `Przekroczono wysoki próg (${batchHiLimit} ${tag.unit || ""}) dla ${tag.name}`;
        hiVal = Number(batchHiLimit);
      } else if (batchAlarmType === "low_limit") {
        aName = `ALM_LO_${tag.name}`;
        aMsg = `Spadek poniżej niskiego progu (${batchLoLimit} ${tag.unit || ""}) dla ${tag.name}`;
        loVal = Number(batchLoLimit);
      } else {
        aName = `ALM_FAULT_${tag.name}`;
        aMsg = `Wykryto stan awaryjny dla sygnału ${tag.name}`;
      }

      const generatedId = `alm_${tag.id.replace(/[^a-zA-Z0-9]/g, "_")}_${batchAlarmType}`;

      generatedAlarms.push({
        id: generatedId,
        name: aName,
        tag_id: tag.id,
        group_id: batchGroupId || undefined,
        priority: batchPriority,
        when_true: batchAlarmType === "digital_fault",
        hi_limit: hiVal,
        lo_limit: loVal,
        deadband: 0,
        on_delay_ms: 500,
        off_delay_ms: 0,
        latching: batchLatching,
        message: aMsg,
      });
    }

    addAlarmsToProject(generatedAlarms);
    onClose();
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="backdrop" onpointerdown={onClose}></div>
  <!-- svelte-ignore a11y_interactive_supports_focus -->
  <div class="modal" role="dialog" aria-labelledby="add-alarm-title" aria-modal="true" tabindex="-1" onkeydown={handleKeyDown}>
    <div class="modal-header">
      <div class="title-wrap">
        <span class="header-icon">🔔</span>
        <div>
          <h3 id="add-alarm-title">Dodaj Alarm / Listę Alarmów (Add Alarm Rules)</h3>
          <p class="subtitle">Zdefiniuj reguły alarmowe, progi analogowe oraz zatrzaskiwanie centralne</p>
        </div>
      </div>
      <button type="button" class="close-btn" onclick={onClose} title="Zamknij (Esc)">✕</button>
    </div>

    <!-- Mode Switcher Tabs -->
    <div class="tab-bar">
      <button type="button" class="tab-btn" class:active={activeTab === "single"} onclick={() => (activeTab = "single")}>
        🔹 Pojedynczy Alarm (Single Rule)
      </button>
      <button type="button" class="tab-btn" class:active={activeTab === "batch"} onclick={() => (activeTab = "batch")}>
        📋 Kreator Listy Alarmów (Batch Generator)
      </button>
    </div>

    <div class="modal-body">
      {#if errorMessage}
        <div class="error-banner">
          <span>⚠️ {errorMessage}</span>
        </div>
      {/if}

      {#if activeTab === "single"}
        <!-- Single Alarm Form -->
        <form onsubmit={handleSubmitSingle} class="form-grid">
          <div class="form-group">
            <label for="alm-name">Nazwa Reguły Alarmowej (Alarm Name):</label>
            <input id="alm-name" type="text" bind:value={alarmName} oninput={handleNameChange} placeholder="e.g. Pompa P1 Awarie" required />
          </div>

          <div class="form-group">
            <label for="alm-id">Identyfikator Alarmu (Alarm ID):</label>
            <input id="alm-id" type="text" bind:value={alarmId} placeholder="alm_p1_fault" required />
          </div>

          <!-- Variable selection -->
          <div class="form-group">
            <label for="alm-tag">Zmienna Źródłowa (Target Tag):</label>
            <select id="alm-tag" bind:value={tagId} required>
              {#if tags.length === 0}
                <option value="">(Brak zmiennych - dodaj najpierw zmienną)</option>
              {/if}
              {#each tags as t}
                <option value={t.id}>{t.name} ({t.id} · {t.data_type} · {t.unit || "brak jedn."})</option>
              {/each}
            </select>
          </div>

          <!-- Alarm Group -->
          <div class="form-group">
            <label for="alm-group">Grupa Alarmowa (Alarm Group):</label>
            {#if !isNewGroup}
              <div class="group-input-wrap">
                <select id="alm-group" bind:value={groupId}>
                  <option value="">(Brak grupy - Ogólny)</option>
                  {#each groups as g}
                    <option value={g.id}>{g.name} ({g.id})</option>
                  {/each}
                </select>
                <button type="button" class="btn-sm" onclick={() => (isNewGroup = true)}>+ Nowa Grupa</button>
              </div>
            {:else}
              <div class="group-input-wrap">
                <input type="text" bind:value={newGroupName} placeholder="Nazwa nowej grupy (np. POMPOWNIA_1)" />
                <button type="button" class="btn-sm" onclick={() => (isNewGroup = false)}>✕ Lista</button>
              </div>
            {/if}
          </div>

          <!-- Priority -->
          <div class="form-group">
            <label for="alm-prio">Priorytet Alarmu (Severity Level):</label>
            <select id="alm-prio" bind:value={priority} class="prio-select {priority}">
              <option value="low">🟢 Low (Niski - Informacja)</option>
              <option value="medium">🟡 Medium (Średni - Ostrzeżenie)</option>
              <option value="high">🟠 High (Wysoki - Awaria)</option>
              <option value="critical">🔴 Critical (Krytyczny - Zatrzymanie)</option>
            </select>
          </div>

          <!-- Condition Type -->
          <div class="form-group">
            <label for="alm-cond">Typ Warunku (Trigger Type):</label>
            <select id="alm-cond" bind:value={conditionType}>
              <option value="digital_true">Bitowe / Cyfrowe (TRUE == Active)</option>
              <option value="hi_limit">Przekroczenie Górnego Progu (High Limit)</option>
              <option value="lo_limit">Spadek Poniżej Dolnego Progu (Low Limit)</option>
            </select>
          </div>

          {#if conditionType === "hi_limit"}
            <div class="form-group">
              <label for="alm-hi">Wartość Progu Górnego (HI Limit):</label>
              <input id="alm-hi" type="number" step="any" bind:value={hiLimit} required />
            </div>
          {:else if conditionType === "lo_limit"}
            <div class="form-group">
              <label for="alm-lo">Wartość Progu Dolnego (LO Limit):</label>
              <input id="alm-lo" type="number" step="any" bind:value={loLimit} required />
            </div>
          {/if}

          <!-- Timing & Latching -->
          <div class="form-group">
            <label for="alm-deadband">Strefa Martwa / Histereza (Deadband):</label>
            <input id="alm-deadband" type="number" min="0" step="any" bind:value={deadband} />
          </div>

          <div class="form-group">
            <label for="alm-ondelay">Opóźnienie Załączenia (ON Delay ms):</label>
            <input id="alm-ondelay" type="number" min="0" step="100" bind:value={onDelayMs} />
          </div>

          <div class="form-group full-width">
            <label for="alm-msg">Komunikat Alarmowy (Message Text):</label>
            <input id="alm-msg" type="text" bind:value={message} placeholder="e.g. Przekroczono maksymalną temperaturę silnika P1!" />
          </div>

          <div class="form-group full-width checkbox-group">
            <label class="checkbox-label">
              <input type="checkbox" bind:checked={latching} />
              <span><strong>Alarm Zatrzaskowy (Latching)</strong> — wymaga ręcznego Potwierdzenia (ACK) i Resetu przez operatora</span>
            </label>
          </div>
        </form>
      {:else}
        <!-- Batch Alarm List Generator -->
        <div class="batch-container">
          <div class="batch-config-panel">
            <h4 class="sub-title">1. Ustawienia Reguł Zbiorczych</h4>
            <div class="form-grid">
              <div class="form-group">
                <label for="b-type">Rodzaj Generowanego Alarmu:</label>
                <select id="b-type" bind:value={batchAlarmType}>
                  <option value="high_limit">High Limit (Próg Górny HI)</option>
                  <option value="low_limit">Low Limit (Próg Dolny LO)</option>
                  <option value="digital_fault">Digital Fault (Stan Bitowy Awarii)</option>
                </select>
              </div>

              <div class="form-group">
                <label for="b-prio">Priorytet Zbioru:</label>
                <select id="b-prio" bind:value={batchPriority}>
                  <option value="low">Low (Niski)</option>
                  <option value="medium">Medium (Średni)</option>
                  <option value="high">High (Wysoki)</option>
                  <option value="critical">Critical (Krytyczny)</option>
                </select>
              </div>

              {#if batchAlarmType === "high_limit"}
                <div class="form-group">
                  <label for="b-hi">Wartość Progu Górnego (HI Value):</label>
                  <input id="b-hi" type="number" step="any" bind:value={batchHiLimit} />
                </div>
              {:else if batchAlarmType === "low_limit"}
                <div class="form-group">
                  <label for="b-lo">Wartość Progu Dolnego (LO Value):</label>
                  <input id="b-lo" type="number" step="any" bind:value={batchLoLimit} />
                </div>
              {/if}

              <div class="form-group">
                <label for="b-group">Grupa Alarmowa:</label>
                <select id="b-group" bind:value={batchGroupId}>
                  <option value="">(Brak grupy)</option>
                  {#each groups as g}
                    <option value={g.id}>{g.name}</option>
                  {/each}
                </select>
              </div>

              <div class="form-group checkbox-group full-width">
                <label class="checkbox-label">
                  <input type="checkbox" bind:checked={batchLatching} />
                  <span>Wszystkie generowane alarmy będą <strong>Zatrzaskowe (Latching)</strong></span>
                </label>
              </div>
            </div>
          </div>

          <div class="batch-select-panel">
            <div class="panel-head">
              <h4 class="sub-title">2. Wybierz Zmienne ({selectedTagIds.length} z {filteredBatchTags.length})</h4>
              <div class="filters">
                <select bind:value={batchDeviceFilter}>
                  <option value="">Wszystkie Urządzenia</option>
                  {#each devices as d}
                    <option value={d.id}>{d.name}</option>
                  {/each}
                </select>
                <input type="text" placeholder="Szukaj zmiennej…" bind:value={batchSearchQuery} />
                <button type="button" class="btn-sm" onclick={toggleSelectAllBatch}>
                  {selectedTagIds.length === filteredBatchTags.length ? "Odznacz Wszystkie" : "Zaznacz Wszystkie"}
                </button>
              </div>
            </div>

            <div class="tag-selector-list">
              {#if filteredBatchTags.length === 0}
                <div class="empty-hint">Brak zmiennych pasujących do filtra.</div>
              {:else}
                {#each filteredBatchTags as t (t.id)}
                  <!-- svelte-ignore a11y_click_events_have_key_events -->
                  <!-- svelte-ignore a11y_no_static_element_interactions -->
                  <div class="tag-row" class:selected={selectedTagIds.includes(t.id)} onclick={() => toggleTagSelect(t.id)}>
                    <input type="checkbox" checked={selectedTagIds.includes(t.id)} onclick={(e) => e.stopPropagation()} onchange={() => toggleTagSelect(t.id)} />
                    <span class="tag-name">{t.name}</span>
                    <span class="tag-id">[{t.id}]</span>
                    <span class="tag-meta">{t.data_type} · {t.unit || "—"}</span>
                  </div>
                {/each}
              {/if}
            </div>
          </div>
        </div>
      {/if}
    </div>

    <div class="modal-footer">
      <div class="hint">Wskazówka: Centralny silnik Rust weryfikuje opóźnienia i zatrzaski w czasie rzeczywistym.</div>
      <div class="footer-actions">
        <button type="button" class="btn-cancel" onclick={onClose}>Anuluj</button>
        {#if activeTab === "single"}
          <button type="button" class="btn-primary" onclick={() => handleSubmitSingle()}>💾 Dodaj Alarm</button>
        {:else}
          <button type="button" class="btn-primary" onclick={handleSubmitBatch}>
            🚀 Wygeneruj Listę ({selectedTagIds.length} Alarmów)
          </button>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.65);
    backdrop-filter: blur(4px);
    z-index: 9998;
    animation: fade-in 0.15s ease-out;
  }

  .modal {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: min(720px, 95vw);
    max-height: 90vh;
    background: #1e1e24;
    border: 1px solid #eab308;
    border-radius: 8px;
    box-shadow: 0 20px 50px rgba(0, 0, 0, 0.7);
    color: #e2e8f0;
    z-index: 9999;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    animation: modal-pop 0.15s ease-out;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 18px;
    background: #252530;
    border-bottom: 1px solid #334155;
  }

  .title-wrap {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .header-icon { font-size: 24px; }
  .title-wrap h3 { margin: 0; font-size: 15px; font-weight: 700; color: #f8fafc; }
  .subtitle { margin: 2px 0 0; font-size: 11px; color: #94a3b8; }

  .close-btn {
    background: transparent;
    border: none;
    color: #94a3b8;
    font-size: 18px;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 4px;
  }

  .close-btn:hover { background: #ef4444; color: #fff; }

  .tab-bar {
    display: flex;
    background: #141418;
    border-bottom: 1px solid #2d3748;
  }

  .tab-btn {
    flex: 1;
    padding: 10px 14px;
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    color: #94a3b8;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.12s;
  }

  .tab-btn.active {
    color: #facc15;
    border-bottom-color: #facc15;
    background: #1e1e24;
  }

  .modal-body {
    padding: 16px 18px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-height: 65vh;
  }

  .error-banner {
    background: rgba(239, 68, 68, 0.15);
    border: 1px solid #ef4444;
    color: #fca5a5;
    padding: 8px 12px;
    border-radius: 6px;
    font-size: 12px;
  }

  .form-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px 16px;
  }

  .full-width { grid-column: span 2; }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .form-group label { font-size: 12px; font-weight: 600; color: #cbd5e1; }

  .form-group input, .form-group select {
    background: #121216;
    border: 1px solid #334155;
    border-radius: 4px;
    color: #f8fafc;
    padding: 7px 10px;
    font-size: 12px;
    outline: none;
  }

  .form-group input:focus, .form-group select:focus {
    border-color: #facc15;
    box-shadow: 0 0 0 2px rgba(250, 204, 21, 0.2);
  }

  .group-input-wrap {
    display: flex;
    gap: 6px;
  }

  .group-input-wrap select, .group-input-wrap input { flex: 1; }

  .btn-sm {
    background: #334155;
    border: 1px solid #475569;
    color: #f8fafc;
    padding: 4px 10px;
    border-radius: 4px;
    font-size: 11px;
    cursor: pointer;
    white-space: nowrap;
  }

  .btn-sm:hover { background: #475569; }

  .prio-select.low { color: #4ade80; }
  .prio-select.medium { color: #facc15; }
  .prio-select.high { color: #fb923c; }
  .prio-select.critical { color: #f87171; font-weight: bold; }

  .checkbox-group { flex-direction: row; align-items: center; margin-top: 4px; }
  .checkbox-label { display: flex; align-items: center; gap: 8px; cursor: pointer; font-size: 12px; }
  .checkbox-label input { width: 16px; height: 16px; accent-color: #eab308; }

  /* Batch generator styling */
  .batch-container { display: flex; flex-direction: column; gap: 14px; }
  .sub-title { margin: 0 0 8px; font-size: 12px; font-weight: 700; color: #facc15; text-transform: uppercase; }

  .batch-config-panel {
    background: #141418;
    border: 1px solid #282d37;
    border-radius: 6px;
    padding: 12px;
  }

  .batch-select-panel {
    background: #141418;
    border: 1px solid #282d37;
    border-radius: 6px;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .panel-head { display: flex; align-items: center; justify-content: space-between; gap: 10px; flex-wrap: wrap; }
  .filters { display: flex; align-items: center; gap: 6px; }

  .filters input, .filters select {
    background: #1e1e24;
    border: 1px solid #334155;
    color: #fff;
    padding: 4px 8px;
    font-size: 11px;
    border-radius: 4px;
  }

  .tag-selector-list {
    max-height: 180px;
    overflow-y: auto;
    border: 1px solid #2d3748;
    border-radius: 4px;
    background: #18181c;
  }

  .tag-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 10px;
    border-bottom: 1px solid #27272a;
    cursor: pointer;
    font-size: 11px;
    transition: background 0.1s;
  }

  .tag-row:hover { background: #27272a; }
  .tag-row.selected { background: rgba(234, 179, 8, 0.12); }
  .tag-name { font-weight: 600; color: #fff; }
  .tag-id { color: #94a3b8; font-family: monospace; }
  .tag-meta { margin-left: auto; color: #64748b; font-size: 10px; }

  .empty-hint { padding: 20px; text-align: center; color: #64748b; font-size: 11px; }

  .modal-footer {
    padding: 12px 18px;
    background: #18181c;
    border-top: 1px solid #2d3748;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .hint { font-size: 11px; color: #64748b; }
  .footer-actions { display: flex; gap: 10px; }

  .btn-cancel {
    background: #27272a; border: 1px solid #3f3f46; color: #cbd5e1;
    padding: 7px 14px; border-radius: 5px; font-size: 12px; cursor: pointer;
  }

  .btn-primary {
    background: #ca8a04; border: 1px solid #eab308; color: #fff;
    padding: 7px 16px; border-radius: 5px; font-size: 12px; font-weight: 600; cursor: pointer;
    transition: all 0.12s;
  }

  .btn-primary:hover { background: #a16207; box-shadow: 0 0 10px rgba(234, 179, 8, 0.4); }

  @keyframes fade-in { from { opacity: 0; } to { opacity: 1; } }
  @keyframes modal-pop {
    from { opacity: 0; transform: translate(-50%, -48%) scale(0.96); }
    to { opacity: 1; transform: translate(-50%, -50%) scale(1); }
  }
</style>

<script lang="ts">
  import { appSettings, updateAppSettings } from "$lib/stores/settings";
  import { project, navigateToValidationIssue } from "$lib/stores/app";
  import { validateProject, type ValidationResult, type ValidationIssue } from "$lib/utils/validation";

  interface Props {
    open: boolean;
    onClose: () => void;
  }

  let { open, onClose }: Props = $props();

  let validationResult = $state<ValidationResult | null>(null);

  function runCheck() {
    validationResult = validateProject($project);
  }

  $effect(() => {
    if (open) {
      runCheck();
    }
  });

  function handleIntervalChange(e: Event) {
    const val = Number((e.target as HTMLSelectElement).value);
    if (val > 0) {
      updateAppSettings({ autosaveIntervalMinutes: val });
    }
  }

  function handleNavigate(issue: ValidationIssue) {
    onClose();
    navigateToValidationIssue(issue);
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="backdrop" onpointerdown={onClose}></div>
  <div class="modal" role="dialog" aria-labelledby="settings-title" aria-modal="true">
    <div class="modal-header">
      <h3 id="settings-title">⚙️ Application Settings / Ustawienia Aplikacji</h3>
      <button type="button" class="close-btn" onclick={onClose} title="Close">✕</button>
    </div>

    <div class="modal-body">
      <!-- Section 1: AutoSave -->
      <div class="section">
        <h4 class="section-title">💾 AutoSave / Automatyczny Zapis</h4>
        
        <label class="setting-row checkbox-row">
          <input
            type="checkbox"
            checked={$appSettings.autosaveEnabled}
            onchange={(e) => updateAppSettings({ autosaveEnabled: e.currentTarget.checked })}
          />
          <span class="setting-label font-bold">Włącz funkcję AutoSave (Enable AutoSave)</span>
        </label>

        <div class="setting-row" class:disabled={!$appSettings.autosaveEnabled}>
          <span class="setting-label">Interwał zapisu (AutoSave Interval):</span>
          <select
            value={$appSettings.autosaveIntervalMinutes}
            disabled={!$appSettings.autosaveEnabled}
            onchange={handleIntervalChange}
          >
            <option value={1}>Co 1 minutę (1 minute)</option>
            <option value={2}>Co 2 minuty (2 minutes)</option>
            <option value={5}>Co 5 minut (5 minutes)</option>
            <option value={10}>Co 10 minut (10 minutes - Domyślnie)</option>
            <option value={15}>Co 15 minut (15 minutes)</option>
            <option value={30}>Co 30 minut (30 minutes)</option>
          </select>
        </div>

        <label class="setting-row checkbox-row" class:disabled={!$appSettings.autosaveEnabled}>
          <input
            type="checkbox"
            checked={$appSettings.autosaveOnlyIfNoError}
            disabled={!$appSettings.autosaveEnabled}
            onchange={(e) => updateAppSettings({ autosaveOnlyIfNoError: e.currentTarget.checked })}
          />
          <span class="setting-label">
            Zapisuj wyłącznie gdy projekt <strong>nie zawiera błędów walidacji</strong> (Only save if 0 errors)
          </span>
        </label>

        {#if $appSettings.lastAutosaveTs}
          <div class="status-info">
            Ostatnia próba AutoSave: <strong>{$appSettings.lastAutosaveTs}</strong>
            <span class="status-badge {$appSettings.lastAutosaveStatus ?? ''}">
              {$appSettings.lastAutosaveStatus === 'ok' ? 'SUCCESS (0 BŁĘDÓW)' : $appSettings.lastAutosaveStatus === 'skipped_errors' ? 'POMINIĘTO (BŁĘDY W PROJEKCIE)' : 'BŁĄD ZAPISU'}
            </span>
          </div>
        {/if}
      </div>

      <!-- Section 2: Start Window & Startup -->
      <div class="section">
        <h4 class="section-title">🚀 Ekran Startowy (Start Window)</h4>
        
        <label class="setting-row checkbox-row">
          <input
            type="checkbox"
            checked={$appSettings.showStartWindowOnStart !== false}
            onchange={(e) => updateAppSettings({ showStartWindowOnStart: e.currentTarget.checked })}
          />
          <span class="setting-label font-bold">Pokazuj ekran startowy (Start Window) przy każdym uruchomieniu ProScada</span>
        </label>
      </div>

      <!-- Section 3: Live Health & Validation Status -->
      <div class="section">
        <div class="section-header-flex">
          <h4 class="section-title">🔍 Stan Walidacji Projektu (Project Health)</h4>
          <button type="button" class="btn-secondary" onclick={runCheck}>Uruchom Ponowny Test</button>
        </div>

        {#if validationResult}
          <div class="health-summary" class:valid={validationResult.valid} class:invalid={!validationResult.valid}>
            {#if validationResult.valid}
              <span class="health-icon">✅</span>
              <span><strong>Projekt jest poprawny.</strong> Brak krytycznych błędów walidacji. AutoSave jest aktywne.</span>
            {:else}
              <span class="health-icon">⚠️</span>
              <span><strong>Wykryto {validationResult.errors.length} błędów!</strong> Kliknij element poniżej, aby do niego przejść.</span>
            {/if}
          </div>

          {#if validationResult.errors.length > 0}
            <div class="error-list">
              <h5>Błędy ({validationResult.errors.length}):</h5>
              <ul>
                {#each validationResult.errors as err}
                  <!-- svelte-ignore a11y_click_events_have_key_events -->
                  <!-- svelte-ignore a11y_no_static_element_interactions -->
                  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
                  <li class="err-item clickable" onclick={() => handleNavigate(err)} title="Kliknij, aby przejść do obiektu">
                    <span class="err-path">[{err.path}]</span> {err.message} ➜
                  </li>
                {/each}
              </ul>
            </div>
          {/if}

          {#if validationResult.warnings.length > 0}
            <div class="warning-list">
              <h5>Ostrzeżenia ({validationResult.warnings.length}):</h5>
              <ul>
                {#each validationResult.warnings as warn}
                  <!-- svelte-ignore a11y_click_events_have_key_events -->
                  <!-- svelte-ignore a11y_no_static_element_interactions -->
                  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
                  <li class="warn-item clickable" onclick={() => handleNavigate(warn)} title="Kliknij, aby przejść do obiektu">
                    <span class="warn-path">[{warn.path}]</span> {warn.message} ➜
                  </li>
                {/each}
              </ul>
            </div>
          {/if}
        {/if}
      </div>
    </div>

    <div class="modal-footer">
      <button type="button" class="btn-primary" onclick={onClose}>Zamknij (Close)</button>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 9998;
    background: rgba(0, 0, 0, 0.65);
    backdrop-filter: blur(2px);
  }
  .modal {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    z-index: 9999;
    width: 640px;
    max-width: 92vw;
    max-height: 88vh;
    background: var(--vs-bg-2, #252526);
    border: 1px solid var(--vs-border, #3e3e42);
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.65);
    border-radius: 4px;
    display: flex;
    flex-direction: column;
    color: var(--vs-text, #cccccc);
    font-family: var(--font-ui, sans-serif);
    font-size: 12.5px;
  }
  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 14px;
    background: var(--vs-bg-3, #2d2d30);
    border-bottom: 1px solid var(--vs-border, #3e3e42);
  }
  .modal-header h3 {
    margin: 0;
    font-size: 13px;
    font-weight: 700;
    color: var(--vs-text-bright, #f3f3f3);
    text-transform: uppercase;
    letter-spacing: 0.02em;
  }
  .close-btn {
    background: transparent;
    border: none;
    color: var(--vs-text-dim, #9d9d9d);
    font-size: 14px;
    cursor: pointer;
    padding: 2px 6px;
    border-radius: 2px;
  }
  .close-btn:hover {
    color: #fff;
    background: var(--vs-bg-4, #333337);
  }
  .modal-body {
    padding: 14px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .section {
    background: var(--vs-bg, #1e1e1e);
    border: 1px solid var(--vs-border, #3e3e42);
    border-radius: 4px;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .section-title {
    margin: 0;
    font-size: 11px;
    font-weight: 700;
    color: var(--vs-accent, #007acc);
    text-transform: uppercase;
    letter-spacing: 0.03em;
    border-bottom: 1px solid var(--vs-border, #3e3e42);
    padding-bottom: 4px;
  }
  .section-header-flex {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .setting-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .setting-row.disabled {
    opacity: 0.45;
    pointer-events: none;
  }
  .checkbox-row {
    cursor: pointer;
  }
  .checkbox-row input {
    cursor: pointer;
    width: 15px;
    height: 15px;
    accent-color: var(--vs-accent, #007acc);
  }
  .setting-label {
    flex: 1;
    color: var(--vs-text, #cccccc);
  }
  .font-bold {
    font-weight: 600;
    color: var(--vs-text-bright, #f3f3f3);
  }
  select {
    background: #3c3c3c;
    color: var(--vs-text-bright, #f3f3f3);
    border: 1px solid var(--vs-border, #3e3e42);
    padding: 3px 8px;
    border-radius: 2px;
    font: inherit;
    font-size: 12px;
  }
  select:focus {
    border-color: var(--vs-accent, #007acc);
  }
  .status-info {
    font-size: 11px;
    color: var(--vs-text-dim, #9d9d9d);
    background: var(--vs-bg-2, #252526);
    border: 1px solid var(--vs-border-soft, #2b2b2b);
    padding: 6px 10px;
    border-radius: 3px;
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .status-badge {
    padding: 1px 6px;
    border-radius: 2px;
    font-weight: 700;
    font-size: 10px;
  }
  .status-badge.ok {
    background: rgba(22, 163, 74, 0.25);
    color: #4ade80;
    border: 1px solid #16a34a;
  }
  .status-badge.skipped_errors {
    background: rgba(234, 179, 8, 0.25);
    color: #fde047;
    border: 1px solid #eab308;
  }
  .status-badge.error {
    background: rgba(220, 38, 38, 0.25);
    color: #fca5a5;
    border: 1px solid #dc2626;
  }
  .health-summary {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 10px;
    border-radius: 3px;
    font-size: 12px;
  }
  .health-summary.valid {
    background: rgba(22, 163, 74, 0.2);
    border: 1px solid var(--vs-ok, #16a34a);
    color: #86efac;
  }
  .health-summary.invalid {
    background: rgba(220, 38, 38, 0.2);
    border: 1px solid var(--vs-danger, #dc2626);
    color: #fca5a5;
  }
  .health-icon {
    font-size: 16px;
  }
  .error-list, .warning-list {
    font-size: 11px;
  }
  .error-list h5 {
    color: #f87171;
    margin: 0 0 4px;
    font-size: 11px;
    text-transform: uppercase;
  }
  .warning-list h5 {
    color: #facc15;
    margin: 0 0 4px;
    font-size: 11px;
    text-transform: uppercase;
  }
  ul {
    margin: 0;
    padding-left: 16px;
  }
  .clickable {
    cursor: pointer;
    border-radius: 2px;
    padding: 2px 4px;
    transition: background 0.1s ease;
  }
  .clickable:hover {
    background: var(--vs-selection, #264f78);
    color: #ffffff;
  }
  .err-item {
    color: #fca5a5;
    margin-bottom: 2px;
  }
  .warn-item {
    color: #fef08a;
    margin-bottom: 2px;
  }
  .err-path, .warn-path {
    font-family: var(--font-mono, monospace);
    opacity: 0.85;
    color: #93c5fd;
  }
  .modal-footer {
    display: flex;
    justify-content: flex-end;
    padding: 8px 14px;
    background: var(--vs-bg-3, #2d2d30);
    border-top: 1px solid var(--vs-border, #3e3e42);
  }
  .btn-primary {
    background: var(--vs-accent, #007acc);
    color: #fff;
    border: 1px solid var(--vs-accent, #007acc);
    padding: 4px 14px;
    border-radius: 2px;
    font-weight: 600;
    font-size: 12px;
    cursor: pointer;
  }
  .btn-primary:hover {
    background: var(--vs-accent-2, #0e639c);
  }
  .btn-secondary {
    background: var(--vs-bg-4, #333337);
    color: var(--vs-text, #cccccc);
    border: 1px solid var(--vs-border, #3e3e42);
    padding: 3px 8px;
    border-radius: 2px;
    font-size: 11px;
    cursor: pointer;
  }
  .btn-secondary:hover {
    background: var(--vs-selection, #264f78);
    color: #ffffff;
  }
</style>

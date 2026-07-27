<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/services/api";
  import type { UserAccountInput, UserSummary } from "$lib/types";

  let {
    open = false,
    onclose = () => {},
  }: {
    open?: boolean;
    onclose?: () => void;
  } = $props();

  let users: UserSummary[] = $state([]);
  let loading: boolean = $state(false);
  let errorMsg: string = $state("");
  let successMsg: string = $state("");

  let editingUser: UserAccountInput | null = $state(null);

  async function loadUsers() {
    loading = true;
    errorMsg = "";
    try {
      users = await api.listUsers();
    } catch (e: any) {
      errorMsg = e?.message || "Błąd pobierania listy użytkowników";
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    if (open) {
      loadUsers();
    }
  });

  function startCreateUser() {
    editingUser = {
      id: null,
      username: "",
      display_name: "",
      password: "",
      pin: "",
      security_level: 100,
      enabled: true,
    };
    errorMsg = "";
    successMsg = "";
  }

  function startEditUser(u: UserSummary) {
    editingUser = {
      id: u.id,
      username: u.username,
      display_name: u.display_name,
      password: "",
      pin: "",
      security_level: u.security_level,
      enabled: u.enabled,
    };
    errorMsg = "";
    successMsg = "";
  }

  async function handleSave() {
    if (!editingUser) return;
    if (!editingUser.username.trim() || !editingUser.display_name.trim()) {
      errorMsg = "Wypełnij nazwę użytkownika i nazwę wyświetlaną";
      return;
    }
    loading = true;
    errorMsg = "";
    successMsg = "";
    try {
      await api.saveUser(editingUser);
      successMsg = `Zapisano użytkownika ${editingUser.username}`;
      editingUser = null;
      await loadUsers();
    } catch (e: any) {
      errorMsg = e?.message || "Nie udało się zapisać użytkownika";
    } finally {
      loading = false;
    }
  }

  async function handleDelete(u: UserSummary) {
    if (!confirm(`Czy na pewno usunąć konto ${u.username}?`)) return;
    loading = true;
    errorMsg = "";
    try {
      await api.deleteUser(u.id);
      successMsg = `Usunięto konto ${u.username}`;
      await loadUsers();
    } catch (e: any) {
      errorMsg = e?.message || "Nie udało się usunąć użytkownika";
    } finally {
      loading = false;
    }
  }

  function getLevelBadgeClass(level: number): string {
    if (level >= 1000) return "badge-admin";
    if (level >= 500) return "badge-engineer";
    if (level >= 100) return "badge-operator";
    return "badge-viewer";
  }

  function getLevelName(level: number): string {
    if (level >= 1000) return `[L${level}] Administrator`;
    if (level >= 500) return `[L${level}] Inżynier`;
    if (level >= 100) return `[L${level}] Operator`;
    return `[L${level}] Podgląd`;
  }
</script>

{#if open}
  <div class="modal-backdrop" onclick={onclose} role="presentation">
    <!-- svelte-ignore a11y_interactive_supports_focus -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="modal-card" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true" tabindex="-1">
      <header class="modal-header">
        <div class="title-group">
          <div class="icon-box">
            <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2.2">
              <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" />
              <circle cx="9" cy="7" r="4" />
              <path d="M23 21v-2a4 4 0 0 0-3-3.87" />
              <path d="M16 3.13a4 4 0 0 1 0 7.75" />
            </svg>
          </div>
          <div>
            <h3>Zarządzanie Użytkownikami i Uprawnieniami</h3>
            <p class="subtitle">Baza kont i poziomy bezpieczeństwa (Security Levels 0–1000)</p>
          </div>
        </div>
        <button class="close-btn" onclick={onclose}>&times;</button>
      </header>

      <div class="modal-body">
        {#if errorMsg}
          <div class="msg-box error">{errorMsg}</div>
        {/if}
        {#if successMsg}
          <div class="msg-box success">{successMsg}</div>
        {/if}

        {#if editingUser}
          <form class="edit-form" onsubmit={(e) => { e.preventDefault(); handleSave(); }}>
            <h4>{editingUser.id ? `Edycja konta: ${editingUser.username}` : "Nowy Użytkownik"}</h4>

            <div class="form-grid">
              <div class="field">
                <label for="u-username">Nazwa Użytkownika (Login)</label>
                <input id="u-username" type="text" bind:value={editingUser.username} placeholder="np. j.kowalski" />
              </div>
              <div class="field">
                <label for="u-display">Nazwa Wyświetlana</label>
                <input id="u-display" type="text" bind:value={editingUser.display_name} placeholder="np. Jan Kowalski" />
              </div>

              <div class="field full">
                <label for="u-level">Poziom Uprawnień: {getLevelName(editingUser.security_level)}</label>
                <div class="slider-row">
                  <input id="u-level" type="range" min="0" max="1000" step="50" bind:value={editingUser.security_level} />
                  <div class="preset-buttons">
                    <button type="button" class="preset-btn" onclick={() => editingUser!.security_level = 0}>L0 (Podgląd)</button>
                    <button type="button" class="preset-btn" onclick={() => editingUser!.security_level = 100}>L100 (Operator)</button>
                    <button type="button" class="preset-btn" onclick={() => editingUser!.security_level = 500}>L500 (Inżynier)</button>
                    <button type="button" class="preset-btn" onclick={() => editingUser!.security_level = 1000}>L1000 (Admin)</button>
                  </div>
                </div>
              </div>

              <div class="field">
                <label for="u-password">{editingUser.id ? "Nowe Hasło (Zostaw puste b.z.)" : "Hasło"}</label>
                <input id="u-password" type="password" bind:value={editingUser.password} placeholder="••••••••" />
              </div>

              <div class="field">
                <label for="u-pin">{editingUser.id ? "Nowy PIN 4-6 cyfr (Zostaw puste b.z.)" : "Szybki PIN"}</label>
                <input id="u-pin" type="text" maxlength="6" bind:value={editingUser.pin} placeholder="np. 1234" />
              </div>

              <div class="field full checkbox-field">
                <label class="checkbox-label">
                  <input type="checkbox" bind:checked={editingUser.enabled} />
                  <span>Konto aktywne i włączone w systemie</span>
                </label>
              </div>
            </div>

            <div class="form-actions">
              <button type="button" class="btn btn-secondary" onclick={() => editingUser = null}>Anuluj</button>
              <button type="submit" class="btn btn-primary" disabled={loading}>Zapisz Użytkownika</button>
            </div>
          </form>
        {:else}
          <div class="toolbar">
            <button class="btn btn-primary" onclick={startCreateUser}>
              <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="12" y1="5" x2="12" y2="19" />
                <line x1="5" y1="12" x2="19" y2="12" />
              </svg>
              Dodaj Nowego Użytkownika
            </button>
          </div>

          <div class="table-container">
            <table class="users-table">
              <thead>
                <tr>
                  <th>Użytkownik</th>
                  <th>Wyświetlana nazwa</th>
                  <th>Poziom Security</th>
                  <th>PIN</th>
                  <th>Status</th>
                  <th class="text-right">Akcje</th>
                </tr>
              </thead>
              <tbody>
                {#each users as u}
                  <tr>
                    <td class="font-bold">{u.username}</td>
                    <td>{u.display_name}</td>
                    <td>
                      <span class={`badge ${getLevelBadgeClass(u.security_level)}`}>
                        {getLevelName(u.security_level)}
                      </span>
                    </td>
                    <td>{u.has_pin ? "✓ Ustawiony" : "Brak"}</td>
                    <td>
                      {#if u.enabled}
                        <span class="status-active">● Aktywny</span>
                      {:else}
                        <span class="status-disabled">○ Zablokowany</span>
                      {/if}
                    </td>
                    <td class="text-right actions-cell">
                      <button class="icon-btn" onclick={() => startEditUser(u)} title="Edytuj">
                        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
                          <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
                          <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
                        </svg>
                      </button>
                      <button class="icon-btn btn-danger" onclick={() => handleDelete(u)} title="Usuń">
                        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
                          <polyline points="3 6 5 6 21 6" />
                          <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
                        </svg>
                      </button>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </div>

      <footer class="modal-footer">
        <button class="btn btn-secondary" onclick={onclose}>Zamknij</button>
      </footer>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(8, 14, 26, 0.8);
    backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9999;
  }

  .modal-card {
    background: #0f172a;
    border: 1px solid #1e293b;
    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.7);
    border-radius: 16px;
    width: 680px;
    max-width: 95vw;
    color: #f8fafc;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    max-height: 90vh;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 20px 24px;
    background: #1e293b;
    border-bottom: 1px solid #334155;
  }

  .title-group {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .icon-box {
    width: 40px;
    height: 40px;
    border-radius: 10px;
    background: linear-gradient(135deg, #0284c7, #2563eb);
    color: #ffffff;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  h3 {
    margin: 0;
    font-size: 1.15rem;
    font-weight: 700;
  }

  .subtitle {
    margin: 2px 0 0 0;
    font-size: 0.8rem;
    color: #94a3b8;
  }

  .close-btn {
    background: transparent;
    border: none;
    color: #94a3b8;
    font-size: 1.5rem;
    cursor: pointer;
  }

  .modal-body {
    padding: 24px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .msg-box {
    padding: 12px 16px;
    border-radius: 10px;
    font-size: 0.88rem;
  }

  .msg-box.error {
    background: rgba(225, 29, 72, 0.15);
    border: 1px solid rgba(225, 29, 72, 0.4);
    color: #fb7185;
  }

  .msg-box.success {
    background: rgba(16, 185, 129, 0.15);
    border: 1px solid rgba(16, 185, 129, 0.4);
    color: #34d399;
  }

  .toolbar {
    display: flex;
    justify-content: flex-start;
  }

  .table-container {
    border: 1px solid #1e293b;
    border-radius: 10px;
    overflow: hidden;
    background: #020617;
  }

  .users-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.88rem;
  }

  .users-table th {
    background: #0f172a;
    padding: 12px 14px;
    text-align: left;
    color: #94a3b8;
    font-weight: 600;
    border-bottom: 1px solid #1e293b;
  }

  .users-table td {
    padding: 12px 14px;
    border-bottom: 1px solid #0f172a;
  }

  .font-bold {
    font-weight: 700;
    color: #f8fafc;
  }

  .badge {
    padding: 4px 8px;
    border-radius: 6px;
    font-size: 0.76rem;
    font-weight: 700;
  }

  .badge-admin { background: rgba(225, 29, 72, 0.2); color: #f43f5e; border: 1px solid rgba(225, 29, 72, 0.4); }
  .badge-engineer { background: rgba(168, 85, 247, 0.2); color: #c084fc; border: 1px solid rgba(168, 85, 247, 0.4); }
  .badge-operator { background: rgba(14, 165, 233, 0.2); color: #38bdf8; border: 1px solid rgba(14, 165, 233, 0.4); }
  .badge-viewer { background: rgba(100, 116, 139, 0.2); color: #94a3b8; border: 1px solid rgba(100, 116, 139, 0.4); }

  .status-active { color: #10b981; font-weight: 600; }
  .status-disabled { color: #64748b; }

  .text-right { text-align: right; }

  .actions-cell {
    display: flex;
    gap: 6px;
    justify-content: flex-end;
  }

  .icon-btn {
    background: #1e293b;
    border: 1px solid #334155;
    color: #cbd5e1;
    padding: 6px;
    border-radius: 6px;
    cursor: pointer;
  }
  .icon-btn:hover { background: #334155; color: #ffffff; }
  .btn-danger:hover { background: #be123c; color: #ffffff; border-color: #be123c; }

  .edit-form {
    background: #020617;
    border: 1px solid #1e293b;
    padding: 20px;
    border-radius: 12px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .edit-form h4 {
    margin: 0;
    color: #38bdf8;
  }

  .form-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 14px;
  }

  .full { grid-column: span 2; }

  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  label {
    font-size: 0.82rem;
    font-weight: 600;
    color: #cbd5e1;
  }

  input[type="text"], input[type="password"] {
    background: #0f172a;
    border: 1px solid #334155;
    color: #f8fafc;
    padding: 10px 12px;
    border-radius: 8px;
    font-size: 0.9rem;
  }

  .slider-row {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  input[type="range"] {
    width: 100%;
    accent-color: #38bdf8;
  }

  .preset-buttons {
    display: flex;
    gap: 6px;
  }

  .preset-btn {
    flex: 1;
    background: #1e293b;
    border: 1px solid #334155;
    color: #94a3b8;
    padding: 6px 8px;
    border-radius: 6px;
    font-size: 0.75rem;
    font-weight: 600;
    cursor: pointer;
  }
  .preset-btn:hover { background: #38bdf8; color: #020617; }

  .checkbox-field {
    display: flex;
    align-items: center;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    font-size: 0.9rem;
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    padding: 16px 24px;
    background: #090d16;
    border-top: 1px solid #1e293b;
  }

  .btn {
    padding: 10px 18px;
    border-radius: 8px;
    font-weight: 600;
    font-size: 0.88rem;
    cursor: pointer;
    border: none;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .btn-secondary { background: #1e293b; color: #cbd5e1; }
  .btn-primary { background: #2563eb; color: #ffffff; }
  .btn-primary:hover { background: #1d4ed8; }
</style>

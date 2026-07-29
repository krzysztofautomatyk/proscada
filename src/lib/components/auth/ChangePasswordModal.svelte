<script lang="ts">
  import { api } from "$lib/services/api";
  import { errorMessage } from "$lib/utils/errors";

  /**
   * Forced password replacement for seeded accounts.
   *
   * The backend refuses process writes and user administration while
   * `password_change_required` is set, so this dialog is the only way out of a
   * default-credential state.
   */
  let {
    open = false,
    username = "",
    mandatory = false,
    onclose = () => {},
    onsuccess = () => {},
  }: {
    open?: boolean;
    username?: string;
    mandatory?: boolean;
    onclose?: () => void;
    onsuccess?: () => void;
  } = $props();

  const MIN_LENGTH = 12;

  let currentPassword = $state("");
  let newPassword = $state("");
  let confirmPassword = $state("");
  let errorMsg = $state("");
  let loading = $state(false);

  const tooShort = $derived(newPassword.length > 0 && newPassword.length < MIN_LENGTH);
  const mismatch = $derived(confirmPassword.length > 0 && confirmPassword !== newPassword);

  function reset() {
    currentPassword = "";
    newPassword = "";
    confirmPassword = "";
    errorMsg = "";
  }

  function close() {
    if (mandatory) return;
    reset();
    onclose();
  }

  async function submit() {
    if (loading) return;
    if (newPassword.length < MIN_LENGTH) {
      errorMsg = `Nowe hasło musi mieć co najmniej ${MIN_LENGTH} znaków`;
      return;
    }
    if (newPassword !== confirmPassword) {
      errorMsg = "Potwierdzenie hasła jest różne od nowego hasła";
      return;
    }
    loading = true;
    errorMsg = "";
    try {
      await api.changePassword(currentPassword, newPassword);
      reset();
      onsuccess();
      onclose();
    } catch (e) {
      errorMsg = errorMessage(e, "Nie udało się zmienić hasła");
    } finally {
      loading = false;
    }
  }

  function onKeydown(event: KeyboardEvent) {
    if (!open) return;
    if (event.key === "Escape") close();
  }
</script>

<svelte:window onkeydown={onKeydown} />

{#if open}
  <div class="modal-backdrop" role="presentation">
    <div class="modal-card" role="dialog" aria-modal="true" aria-labelledby="change-password-title">
      <header class="modal-header">
        <div>
          <h3 id="change-password-title">Zmiana hasła</h3>
          <p class="subtitle">
            {#if mandatory}
              Konto <strong>{username}</strong> używa hasła domyślnego. Zapis do procesu
              i administracja użytkownikami są zablokowane do czasu jego zmiany.
            {:else}
              Konto <strong>{username}</strong>
            {/if}
          </p>
        </div>
        {#if !mandatory}
          <button class="close-btn" type="button" onclick={close} aria-label="Zamknij">&times;</button>
        {/if}
      </header>

      <form
        class="modal-body"
        onsubmit={(event) => {
          event.preventDefault();
          submit();
        }}
      >
        {#if errorMsg}
          <p class="error-banner" role="alert">{errorMsg}</p>
        {/if}

        <label class="field">
          <span>Aktualne hasło</span>
          <input type="password" autocomplete="current-password" bind:value={currentPassword} />
        </label>

        <label class="field">
          <span>Nowe hasło (min. {MIN_LENGTH} znaków)</span>
          <input type="password" autocomplete="new-password" bind:value={newPassword} />
        </label>
        {#if tooShort}
          <p class="hint warn">Hasło jest za krótkie.</p>
        {/if}

        <label class="field">
          <span>Powtórz nowe hasło</span>
          <input type="password" autocomplete="new-password" bind:value={confirmPassword} />
        </label>
        {#if mismatch}
          <p class="hint warn">Hasła nie są identyczne.</p>
        {/if}

        <div class="actions">
          {#if !mandatory}
            <button type="button" class="btn ghost" onclick={close}>Anuluj</button>
          {/if}
          <button type="submit" class="btn primary" disabled={loading}>
            {loading ? "Zapisywanie…" : "Zmień hasło"}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(1, 4, 9, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2000;
  }
  .modal-card {
    width: 420px;
    max-width: 92vw;
    background: #0d1117;
    border: 1px solid #30363d;
    border-radius: 10px;
    color: #e6edf3;
    box-shadow: 0 18px 48px rgba(0, 0, 0, 0.55);
  }
  .modal-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
    padding: 14px 16px;
    border-bottom: 1px solid #21262d;
  }
  h3 {
    margin: 0;
    font-size: 15px;
  }
  .subtitle {
    margin: 4px 0 0;
    font-size: 12px;
    color: #8b949e;
    line-height: 1.45;
  }
  .close-btn {
    background: none;
    border: none;
    color: #8b949e;
    font-size: 22px;
    line-height: 1;
    cursor: pointer;
  }
  .modal-body {
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 12px;
    color: #c9d1d9;
  }
  .field input {
    background: #010409;
    border: 1px solid #30363d;
    border-radius: 6px;
    color: #e6edf3;
    padding: 7px 9px;
    font-size: 13px;
  }
  .field input:focus-visible {
    outline: 2px solid #1f6feb;
    outline-offset: 1px;
  }
  .hint {
    margin: 0;
    font-size: 11px;
  }
  .warn {
    color: #d29922;
  }
  .error-banner {
    margin: 0;
    background: rgba(248, 81, 73, 0.12);
    border: 1px solid rgba(248, 81, 73, 0.4);
    color: #ff7b72;
    border-radius: 6px;
    padding: 8px 10px;
    font-size: 12px;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 4px;
  }
  .btn {
    border-radius: 6px;
    border: 1px solid #30363d;
    padding: 7px 14px;
    font-size: 12px;
    cursor: pointer;
    background: #21262d;
    color: #c9d1d9;
  }
  .btn.primary {
    background: #238636;
    border-color: #2ea043;
    color: #ffffff;
    font-weight: 600;
  }
  .btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  .btn:focus-visible {
    outline: 2px solid #1f6feb;
    outline-offset: 1px;
  }
</style>

<script lang="ts">
  import { api } from "$lib/services/api";
  import { errorMessage } from "$lib/utils/errors";
  import type { UserSummary } from "$lib/types";

  let {
    open = false,
    onclose = () => {},
    onsuccess = (_user: UserSummary) => {},
  }: {
    open?: boolean;
    onclose?: () => void;
    onsuccess?: (user: UserSummary) => void;
  } = $props();

  let username: string = $state("");
  let password: string = $state("");
  let errorMsg: string = $state("");
  let loading: boolean = $state(false);
  let dialogEl = $state<HTMLDivElement | null>(null);

  $effect(() => {
    if (!open || !dialogEl) return;
    const previous = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    queueMicrotask(() => dialogEl?.querySelector<HTMLElement>("#input-username")?.focus());
    return () => previous?.focus();
  });

  async function handleLogin() {
    if (loading) return;
    errorMsg = "";
    if (!username.trim() || !password.trim()) {
      errorMsg = "Podaj nazwę użytkownika i hasło";
      return;
    }
    loading = true;
    try {
      const user = await api.login(username.trim(), password);
      username = "";
      password = "";
      onsuccess(user);
      onclose();
    } catch (e) {
      errorMsg = errorMessage(e, "Błąd uwierzytelniania");
    } finally {
      loading = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!open) return;
    if (e.key === "Tab" && dialogEl) {
      const focusable = Array.from(
        dialogEl.querySelectorAll<HTMLElement>(
          'button:not(:disabled), input:not(:disabled), [tabindex]:not([tabindex="-1"])',
        ),
      );
      if (focusable.length > 0) {
        const current = focusable.indexOf(document.activeElement as HTMLElement);
        const next = e.shiftKey
          ? (current <= 0 ? focusable.length - 1 : current - 1)
          : (current < 0 || current === focusable.length - 1 ? 0 : current + 1);
        e.preventDefault();
        focusable[next]?.focus();
      }
      return;
    }
    if (e.key === "Escape") {
      onclose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <div class="modal-backdrop">
    <button
      type="button"
      class="backdrop-dismiss"
      aria-label="Zamknij okno logowania"
      tabindex="-1"
      onclick={onclose}
    ></button>
    <div
      class="modal-card"
      role="dialog"
      aria-modal="true"
      aria-labelledby="login-title"
      bind:this={dialogEl}
    >
      <header class="modal-header">
        <div class="title-group">
          <div class="shield-icon">
            <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2.2">
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
            </svg>
          </div>
          <div>
            <h3 id="login-title">Autoryzacja ProScada</h3>
            <p class="subtitle">Zaloguj się nazwą użytkownika i hasłem</p>
          </div>
        </div>
        <button
          type="button"
          class="close-btn"
          aria-label="Zamknij logowanie"
          onclick={onclose}
          title="Zamknij (Esc)"
        >&times;</button>
      </header>

      <div class="modal-body">
        {#if errorMsg}
          <div class="error-banner">
            <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10" />
              <line x1="12" y1="8" x2="12" y2="12" />
              <line x1="12" y1="16" x2="12.01" y2="16" />
            </svg>
            <span>{errorMsg}</span>
          </div>
        {/if}

        <form class="form-section" onsubmit={(e) => { e.preventDefault(); void handleLogin(); }}>
          <div class="field">
            <label for="input-username">Nazwa Użytkownika</label>
            <input
              id="input-username"
              type="text"
              bind:value={username}
              placeholder="np. operator lub admin"
              autocomplete="username"
            />
          </div>
          <div class="field">
            <label for="input-password">Hasło Główne</label>
            <input
              id="input-password"
              type="password"
              bind:value={password}
              placeholder="••••••••"
              autocomplete="current-password"
            />
          </div>
        </form>
      </div>

      <footer class="modal-footer">
        <button type="button" class="btn btn-secondary" onclick={onclose} disabled={loading}>Anuluj</button>
        <button type="button" class="btn btn-primary" onclick={handleLogin} disabled={loading}>
          {#if loading}
            <span class="spinner"></span> Zaloguj...
          {:else}
            Zaloguj do ProScada
          {/if}
        </button>
      </footer>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(8, 14, 26, 0.75);
    backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9999;
    animation: fadeIn 0.2s ease-out;
  }

  /* Full-size, transparent dismissal target behind the dialog. */
  .backdrop-dismiss {
    position: absolute;
    inset: 0;
    appearance: none;
    border: 0;
    padding: 0;
    margin: 0;
    background: transparent;
    cursor: default;
  }

  .modal-card {
    position: relative;
    z-index: 1;
    background: #0f172a;
    border: 1px solid #1e293b;
    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.7);
    border-radius: 16px;
    width: 420px;
    max-width: 92vw;
    color: #f8fafc;
    overflow: hidden;
    display: flex;
    flex-direction: column;
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

  .shield-icon {
    width: 40px;
    height: 40px;
    border-radius: 10px;
    background: linear-gradient(135deg, #2563eb, #1d4ed8);
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
    line-height: 1;
    padding: 4px 8px;
    border-radius: 6px;
  }
  .close-btn:hover {
    color: #ffffff;
    background: #334155;
  }

  .modal-body {
    padding: 24px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .error-banner {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px 14px;
    background: rgba(225, 29, 72, 0.15);
    border: 1px solid rgba(225, 29, 72, 0.4);
    color: #fb7185;
    border-radius: 10px;
    font-size: 0.88rem;
  }

  .form-section {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  label {
    font-size: 0.85rem;
    font-weight: 600;
    color: #cbd5e1;
  }

  input {
    background: #020617;
    border: 1px solid #334155;
    color: #f8fafc;
    padding: 12px 14px;
    border-radius: 10px;
    font-size: 0.95rem;
    outline: none;
    transition: border-color 0.15s ease;
  }

  input:focus {
    border-color: #38bdf8;
    box-shadow: 0 0 0 3px rgba(56, 189, 248, 0.15);
  }

  .modal-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 12px;
    padding: 16px 24px;
    background: #090d16;
    border-top: 1px solid #1e293b;
  }

  .btn {
    padding: 10px 18px;
    border-radius: 10px;
    font-weight: 600;
    font-size: 0.9rem;
    cursor: pointer;
    border: none;
    transition: all 0.15s ease;
  }

  .btn-secondary {
    background: #1e293b;
    color: #cbd5e1;
  }
  .btn-secondary:hover {
    background: #334155;
    color: #ffffff;
  }

  .btn-primary {
    background: linear-gradient(135deg, #0284c7, #2563eb);
    color: #ffffff;
    box-shadow: 0 4px 12px rgba(37, 99, 235, 0.35);
  }
  .btn-primary:hover {
    background: linear-gradient(135deg, #0369a1, #1d4ed8);
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }
</style>

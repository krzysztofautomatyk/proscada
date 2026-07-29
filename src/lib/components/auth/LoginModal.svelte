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

  let mode: "pin" | "password" = $state("pin");
  let pin: string = $state("");
  let username: string = $state("");
  let password: string = $state("");
  let errorMsg: string = $state("");
  let loading: boolean = $state(false);

  function appendPinDigit(d: string) {
    if (pin.length < 8) {
      pin += d;
      errorMsg = "";
    }
  }

  function clearPin() {
    pin = "";
    errorMsg = "";
  }

  function deletePinDigit() {
    pin = pin.slice(0, -1);
    errorMsg = "";
  }

  async function handleLogin() {
    errorMsg = "";
    loading = true;
    try {
      let user: UserSummary;
      if (mode === "pin") {
        if (!pin.trim()) {
          errorMsg = "Wprowadź PIN autoryzacyjny";
          loading = false;
          return;
        }
        user = await api.login(pin.trim());
      } else {
        if (!username.trim() || !password.trim()) {
          errorMsg = "Podaj nazwę użytkownika i hasło";
          loading = false;
          return;
        }
        user = await api.login(username.trim(), password.trim());
      }
      pin = "";
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
    if (e.key === "Escape") {
      onclose();
    } else if (e.key === "Enter") {
      handleLogin();
    } else if (mode === "pin" && /^[0-9]$/.test(e.key)) {
      appendPinDigit(e.key);
    } else if (mode === "pin" && e.key === "Backspace") {
      deletePinDigit();
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
      onclick={onclose}
    ></button>
    <div class="modal-card" role="dialog" aria-modal="true">
      <header class="modal-header">
        <div class="title-group">
          <div class="shield-icon">
            <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2.2">
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
            </svg>
          </div>
          <div>
            <h3>Autoryzacja ProScada</h3>
            <p class="subtitle">Wybierz metodę uwierzytelniania w systemie</p>
          </div>
        </div>
        <button class="close-btn" onclick={onclose} title="Zamknij (Esc)">&times;</button>
      </header>

      <div class="tabs">
        <button
          class="tab-btn"
          class:active={mode === "pin"}
          onclick={() => { mode = "pin"; errorMsg = ""; }}
        >
          <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="5" y="11" width="14" height="10" rx="2" />
            <path d="M8 11V7a4 4 0 0 1 8 0v4" />
          </svg>
          Szybki PIN HMI
        </button>
        <button
          class="tab-btn"
          class:active={mode === "password"}
          onclick={() => { mode = "password"; errorMsg = ""; }}
        >
          <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
            <circle cx="12" cy="7" r="4" />
          </svg>
          Login + Hasło
        </button>
      </div>

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

        {#if mode === "pin"}
          <div class="pin-section">
            <div class="pin-display">
              <div class="pin-dots">
                {#each [0, 1, 2, 3] as idx}
                  <span class="dot" class:filled={pin.length > idx}></span>
                {/each}
              </div>
              <span class="pin-text">{pin ? "•".repeat(pin.length) : "Wprowadź PIN..."}</span>
            </div>

            <!-- Touch Screen HMI Keypad -->
            <div class="keypad">
              {#each ["1", "2", "3", "4", "5", "6", "7", "8", "9"] as num}
                <button class="key-btn" onclick={() => appendPinDigit(num)}>{num}</button>
              {/each}
              <button class="key-btn key-action text-red" onclick={clearPin}>CLR</button>
              <button class="key-btn" onclick={() => appendPinDigit("0")}>0</button>
              <button class="key-btn key-action" onclick={deletePinDigit}>⌫</button>
            </div>
          </div>
        {:else}
          <form class="form-section" onsubmit={(e) => { e.preventDefault(); handleLogin(); }}>
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
        {/if}
      </div>

      <footer class="modal-footer">
        <button class="btn btn-secondary" onclick={onclose} disabled={loading}>Anuluj</button>
        <button class="btn btn-primary" onclick={handleLogin} disabled={loading}>
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

  .tabs {
    display: flex;
    background: #090d16;
    padding: 6px;
    gap: 6px;
    border-bottom: 1px solid #1e293b;
  }

  .tab-btn {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 10px;
    background: transparent;
    border: none;
    color: #94a3b8;
    font-weight: 600;
    font-size: 0.88rem;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .tab-btn.active {
    background: #1e293b;
    color: #38bdf8;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
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

  .pin-section {
    display: flex;
    flex-direction: column;
    gap: 18px;
    align-items: center;
  }

  .pin-display {
    width: 100%;
    background: #020617;
    border: 1px solid #334155;
    border-radius: 12px;
    padding: 14px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
  }

  .pin-dots {
    display: flex;
    gap: 12px;
  }

  .dot {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    border: 2px solid #475569;
    transition: all 0.15s ease;
  }

  .dot.filled {
    background: #38bdf8;
    border-color: #38bdf8;
    box-shadow: 0 0 10px rgba(56, 189, 248, 0.5);
  }

  .pin-text {
    font-size: 1rem;
    font-weight: 600;
    color: #94a3b8;
    letter-spacing: 4px;
    min-height: 24px;
  }

  .keypad {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 10px;
    width: 100%;
  }

  .key-btn {
    background: #1e293b;
    border: 1px solid #334155;
    color: #f8fafc;
    font-size: 1.35rem;
    font-weight: 700;
    padding: 16px 0;
    border-radius: 12px;
    cursor: pointer;
    transition: all 0.1s ease;
    user-select: none;
  }

  .key-btn:active {
    transform: scale(0.95);
    background: #0284c7;
    border-color: #38bdf8;
  }

  .key-action {
    font-size: 0.95rem;
    color: #94a3b8;
    background: #0f172a;
  }

  .text-red {
    color: #f43f5e;
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

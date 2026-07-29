<script lang="ts">
  import { api } from "$lib/services/api";
  import { errorMessage } from "$lib/utils/errors";

  let {
    open = false,
    actionName = "Operacja wysokiego ryzyka",
    onclose = () => {},
    onsuccess = () => {},
  }: {
    open?: boolean;
    actionName?: string;
    onclose?: () => void;
    onsuccess?: () => void;
  } = $props();

  let pin: string = $state("");
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

  async function handleVerify() {
    if (!pin.trim()) {
      errorMsg = "Wprowadź PIN autoryzacyjny";
      return;
    }
    loading = true;
    errorMsg = "";
    try {
      const valid = await api.verifyPin(pin.trim());
      if (valid) {
        pin = "";
        onsuccess();
        onclose();
      } else {
        errorMsg = "Niepoprawny PIN autoryzacyjny";
      }
    } catch (e) {
      errorMsg = errorMessage(e, "Błąd autoryzacji PIN");
    } finally {
      loading = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!open) return;
    if (e.key === "Escape") {
      onclose();
    } else if (e.key === "Enter") {
      handleVerify();
    } else if (/^[0-9]$/.test(e.key)) {
      appendPinDigit(e.key);
    } else if (e.key === "Backspace") {
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
      aria-label="Zamknij okno autoryzacji PIN"
      onclick={onclose}
    ></button>
    <div class="modal-card" role="dialog" aria-modal="true">
      <header class="modal-header">
        <div class="title-group">
          <div class="warning-icon">
            <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2.2">
              <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
              <line x1="12" y1="9" x2="12" y2="13" />
              <line x1="12" y1="17" x2="12.01" y2="17" />
            </svg>
          </div>
          <div>
            <h3>Wymagana Autoryzacja PIN</h3>
            <p class="subtitle">{actionName}</p>
          </div>
        </div>
        <button class="close-btn" onclick={onclose}>&times;</button>
      </header>

      <div class="modal-body">
        {#if errorMsg}
          <div class="error-banner">
            <span>{errorMsg}</span>
          </div>
        {/if}

        <div class="pin-display">
          <div class="pin-dots">
            {#each [0, 1, 2, 3] as idx}
              <span class="dot" class:filled={pin.length > idx}></span>
            {/each}
          </div>
          <span class="pin-text">{pin ? "•".repeat(pin.length) : "Wpisz PIN..."}</span>
        </div>

        <div class="keypad">
          {#each ["1", "2", "3", "4", "5", "6", "7", "8", "9"] as num}
            <button class="key-btn" onclick={() => appendPinDigit(num)}>{num}</button>
          {/each}
          <button class="key-btn key-action text-red" onclick={clearPin}>CLR</button>
          <button class="key-btn" onclick={() => appendPinDigit("0")}>0</button>
          <button class="key-btn key-action" onclick={deletePinDigit}>⌫</button>
        </div>
      </div>

      <footer class="modal-footer">
        <button class="btn btn-secondary" onclick={onclose} disabled={loading}>Anuluj</button>
        <button class="btn btn-primary" onclick={handleVerify} disabled={loading}>
          {#if loading} Weryfikowanie... {:else} Potwierdź PIN {/if}
        </button>
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
    z-index: 10000;
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
    border: 1px solid #dc2626;
    box-shadow: 0 25px 50px -12px rgba(220, 38, 38, 0.25);
    border-radius: 16px;
    width: 380px;
    max-width: 92vw;
    color: #f8fafc;
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 18px 20px;
    background: rgba(220, 38, 38, 0.15);
    border-bottom: 1px solid rgba(220, 38, 38, 0.3);
  }

  .title-group {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .warning-icon {
    width: 38px;
    height: 38px;
    border-radius: 10px;
    background: #dc2626;
    color: #ffffff;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  h3 {
    margin: 0;
    font-size: 1.05rem;
    font-weight: 700;
  }

  .subtitle {
    margin: 2px 0 0 0;
    font-size: 0.78rem;
    color: #fca5a5;
  }

  .close-btn {
    background: transparent;
    border: none;
    color: #94a3b8;
    font-size: 1.4rem;
    cursor: pointer;
  }

  .modal-body {
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    align-items: center;
  }

  .error-banner {
    width: 100%;
    padding: 10px;
    background: rgba(225, 29, 72, 0.2);
    border: 1px solid #f43f5e;
    color: #f43f5e;
    border-radius: 8px;
    font-size: 0.85rem;
    text-align: center;
  }

  .pin-display {
    width: 100%;
    background: #020617;
    border: 1px solid #334155;
    border-radius: 10px;
    padding: 12px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
  }

  .pin-dots {
    display: flex;
    gap: 10px;
  }

  .dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    border: 2px solid #475569;
  }
  .dot.filled {
    background: #ef4444;
    border-color: #ef4444;
    box-shadow: 0 0 8px rgba(239, 68, 68, 0.6);
  }

  .pin-text {
    font-size: 0.9rem;
    color: #94a3b8;
    letter-spacing: 3px;
  }

  .keypad {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 8px;
    width: 100%;
  }

  .key-btn {
    background: #1e293b;
    border: 1px solid #334155;
    color: #f8fafc;
    font-size: 1.25rem;
    font-weight: 700;
    padding: 14px 0;
    border-radius: 10px;
    cursor: pointer;
  }
  .key-btn:active {
    background: #dc2626;
  }

  .key-action {
    font-size: 0.9rem;
    color: #94a3b8;
  }

  .text-red {
    color: #f87171;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    padding: 14px 20px;
    background: #090d16;
    border-top: 1px solid #1e293b;
  }

  .btn {
    padding: 8px 16px;
    border-radius: 8px;
    font-weight: 600;
    font-size: 0.88rem;
    cursor: pointer;
    border: none;
  }

  .btn-secondary {
    background: #1e293b;
    color: #cbd5e1;
  }

  .btn-primary {
    background: #dc2626;
    color: #ffffff;
    box-shadow: 0 4px 10px rgba(220, 38, 38, 0.4);
  }
  .btn-primary:hover {
    background: #b91c1c;
  }
</style>

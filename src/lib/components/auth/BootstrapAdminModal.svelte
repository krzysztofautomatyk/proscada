<script lang="ts">
  import { api } from "$lib/services/api";
  import { errorMessage } from "$lib/utils/errors";
  import type { UserSummary } from "$lib/types";

  let {
    open = false,
    onsuccess = (_user: UserSummary) => {},
  }: {
    open?: boolean;
    onsuccess?: (user: UserSummary) => void | Promise<void>;
  } = $props();

  let password = $state("");
  let confirmPassword = $state("");
  let errorMsg = $state("");
  let loading = $state(false);
  let dialogEl = $state<HTMLDivElement | null>(null);
  let passwordEl = $state<HTMLInputElement | null>(null);

  $effect(() => {
    if (!open) return;
    queueMicrotask(() => passwordEl?.focus());
  });

  async function submit() {
    if (loading) return;
    if (password.length < 12) {
      errorMsg = "Hasło musi mieć co najmniej 12 znaków";
      return;
    }
    if (password !== confirmPassword) {
      errorMsg = "Hasła nie są identyczne";
      return;
    }
    loading = true;
    errorMsg = "";
    try {
      await api.bootstrapAdmin(password);
      const user = await api.login("admin", password);
      password = "";
      confirmPassword = "";
      await onsuccess(user);
    } catch (error) {
      errorMsg = errorMessage(error, "Nie udało się utworzyć Administratora");
    } finally {
      loading = false;
    }
  }

  function onKeydown(event: KeyboardEvent) {
    if (!open || !dialogEl || event.key !== "Tab") return;
    const focusable = Array.from(
      dialogEl.querySelectorAll<HTMLElement>(
        'button:not(:disabled), input:not(:disabled), [tabindex]:not([tabindex="-1"])',
      ),
    );
    if (focusable.length === 0) return;
    const current = focusable.indexOf(document.activeElement as HTMLElement);
    const next = event.shiftKey
      ? (current <= 0 ? focusable.length - 1 : current - 1)
      : (current < 0 || current === focusable.length - 1 ? 0 : current + 1);
    event.preventDefault();
    focusable[next]?.focus();
  }
</script>

<svelte:window onkeydown={onKeydown} />

{#if open}
  <div class="backdrop">
    <div
      class="dialog"
      bind:this={dialogEl}
      role="dialog"
      aria-modal="true"
      aria-labelledby="bootstrap-title"
      aria-describedby="bootstrap-description"
    >
      <header>
        <span aria-hidden="true">🛡</span>
        <div>
          <h2 id="bootstrap-title">Pierwsze uruchomienie</h2>
          <p id="bootstrap-description">
            Utwórz jedyne początkowe konto Administratora. Operacja zostanie trwale zamknięta po powodzeniu.
          </p>
        </div>
      </header>

      <form onsubmit={(event) => { event.preventDefault(); void submit(); }}>
        <label>
          Hasło Administratora
          <input
            bind:this={passwordEl}
            bind:value={password}
            type="password"
            minlength="12"
            autocomplete="new-password"
            disabled={loading}
            required
          />
        </label>
        <label>
          Powtórz hasło
          <input
            bind:value={confirmPassword}
            type="password"
            minlength="12"
            autocomplete="new-password"
            disabled={loading}
            required
          />
        </label>
        <small>Minimum 12 znaków. Hasło nie będzie wyświetlane ani zapisywane przez UI.</small>
        {#if errorMsg}<p class="error" role="alert">{errorMsg}</p>{/if}
        <button type="submit" disabled={loading || password.length < 12 || password !== confirmPassword}>
          {loading ? "Tworzenie konta…" : "Utwórz Administratora"}
        </button>
      </form>
    </div>
  </div>
{/if}

<style>
  .backdrop { position:fixed; inset:0; z-index:12000; display:grid; place-items:center; padding:20px; background:rgba(2,6,23,.86); }
  .dialog { width:min(440px,100%); overflow:hidden; border:1px solid #2563eb; border-radius:12px; background:#0f172a; color:#f8fafc; box-shadow:0 24px 70px rgba(0,0,0,.55); }
  header { display:flex; gap:12px; padding:18px 20px; border-bottom:1px solid #334155; background:#111c33; }
  header > span { font-size:25px; }
  h2 { margin:0; font-size:17px; }
  header p { margin:4px 0 0; color:#cbd5e1; font-size:11px; line-height:1.45; }
  form { display:grid; gap:12px; padding:20px; }
  label { display:grid; gap:5px; color:#e2e8f0; font-size:11px; font-weight:700; }
  input { min-height:36px; box-sizing:border-box; border:1px solid #475569; border-radius:5px; padding:6px 9px; background:#020617; color:#f8fafc; font:14px ui-monospace,monospace; }
  input:focus-visible { outline:2px solid #60a5fa; outline-offset:2px; }
  small { color:#94a3b8; font-size:10px; }
  .error { margin:0; padding:8px; border:1px solid #ef4444; border-radius:4px; background:#450a0a; color:#fecaca; font-size:11px; }
  button { min-height:38px; border:1px solid #60a5fa; border-radius:5px; background:#2563eb; color:#fff; font-weight:800; cursor:pointer; }
  button:disabled { opacity:.5; cursor:not-allowed; }
</style>

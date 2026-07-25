<script lang="ts">
  import type { WidgetRendererProps } from "$lib/components/widgets/shared/types";
  import { configOf, readString } from "$lib/components/widgets/shared/config";

  let { widget, design = false }: WidgetRendererProps = $props();

  const cfg = $derived(configOf(widget));
  const label = $derived(readString(cfg, "label", "Open"));
  const target = $derived(readString(cfg, "target", ""));

  const paramsResult = $derived.by<{ params: Record<string, unknown>; error: string | null }>(() => {
    const raw = cfg["params"];
    if (raw === undefined || raw === null || raw === "") return { params: {}, error: null };
    if (typeof raw === "object" && !Array.isArray(raw)) {
      return { params: raw as Record<string, unknown>, error: null };
    }
    if (typeof raw === "string") {
      try {
        const parsed: unknown = JSON.parse(raw);
        if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) {
          return { params: {}, error: "params must be a JSON object" };
        }
        return { params: parsed as Record<string, unknown>, error: null };
      } catch (e) {
        return { params: {}, error: `params: ${e instanceof Error ? e.message : "invalid JSON"}` };
      }
    }
    return { params: {}, error: "params must be a JSON object" };
  });

  const isSafeTarget = $derived(target.startsWith("/") || target.startsWith("screen:"));
  const configError = $derived(paramsResult.error);
  const blocked = $derived(target !== "" && !isSafeTarget);

  function navigate() {
    if (design || configError || !isSafeTarget) return;
    window.dispatchEvent(
      new CustomEvent("proscada:navigate", {
        detail: { target, params: paramsResult.params, sourceWidgetId: widget.id },
      }),
    );
  }
</script>

<div class="nav-link" class:design>
  {#if configError}
    <div class="cfg-error" role="alert"><strong>Config error</strong><span>{configError}</span></div>
  {:else if !target}
    <div class="cfg-error" role="alert"><strong>Config error</strong><span>No target configured</span></div>
  {:else if blocked}
    <button type="button" class="link blocked" disabled title="Blocked target: {target}">
      <span class="icon" aria-hidden="true">⛔</span>{label}
      <span class="reason">blocked</span>
    </button>
  {:else}
    <button
      type="button"
      class="link"
      disabled={design}
      onclick={navigate}
      title={design ? "Navigation disabled in design" : `Navigate to ${target}`}
    >
      <span class="icon" aria-hidden="true">➜</span>{label}
    </button>
  {/if}
</div>

<style>
  .nav-link {
    width: 100%;
    height: 100%;
    display: flex;
    box-sizing: border-box;
  }
  .link {
    flex: 1;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    border: 1px solid #2563eb;
    border-radius: 6px;
    background: #eff6ff;
    color: #1d4ed8;
    font-size: 11px;
    font-weight: 700;
    font-family: "Segoe UI", system-ui, sans-serif;
    padding: 4px 10px;
    cursor: pointer;
  }
  .link:hover:not(:disabled) {
    background: #dbeafe;
  }
  .link:disabled {
    cursor: default;
  }
  .nav-link.design .link:disabled {
    opacity: 0.6;
  }
  .link.blocked {
    border-color: #fca5a5;
    background: #fef2f2;
    color: #b91c1c;
    cursor: not-allowed;
  }
  .icon {
    font-size: 10px;
  }
  .reason {
    font-size: 8px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    border: 1px solid currentColor;
    border-radius: 3px;
    padding: 0 3px;
  }
  .cfg-error {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 6px 8px;
    border: 1px solid #fca5a5;
    border-radius: 6px;
    background: #fef2f2;
    color: #991b1b;
    font-size: 10px;
  }
  .cfg-error strong {
    font-size: 11px;
  }
</style>

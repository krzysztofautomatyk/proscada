<script lang="ts">
  import type { WidgetRendererProps } from "$lib/components/widgets/shared/types";
  import { configOf, invokeWrite, readBoolean, readString } from "$lib/components/widgets/shared/config";
  import WidgetCard from "$lib/components/widgets/shared/WidgetCard.svelte";

  let { widget, tag = null, design = false, onWrite }: WidgetRendererProps = $props();

  const config = $derived(configOf(widget));
  const rawVariant = $derived(readString(config, "variant", "switch").toLowerCase());
  const variant = $derived(["checkbox", "switch"].includes(rawVariant) ? rawVariant : "switch");
  const title = $derived(readString(config, "title", "BOOLEAN INPUT"));
  const trueLabel = $derived(readString(config, "trueLabel", "ON"));
  const falseLabel = $derived(readString(config, "falseLabel", "OFF"));
  const indeterminateLabel = $derived(readString(config, "indeterminateLabel", "UNKNOWN"));
  const qualityLocked = $derived(readBoolean(config, "disabledWhenBad", true) && tag?.quality !== "good");
  const indeterminate = $derived(!tag || tag.quality !== "good");
  const current = $derived(tag?.bool_value ?? false);
  const configError = $derived(!["checkbox", "switch"].includes(rawVariant) ? `invalid variant: ${rawVariant}` : "");
  const disabled = $derived(design || !widget.tag_id || !onWrite || qualityLocked || Boolean(configError));
  let status = $state("");

  function change() {
    if (disabled) return;
    const next = indeterminate ? true : !current;
    if (readBoolean(config, "confirm", false) && !window.confirm(readString(config, "confirmText", `Set ${title} ${next ? trueLabel : falseLabel}?`))) return;
    if (invokeWrite(widget, design, onWrite, next ? 1 : 0)) status = "WRITE REQUESTED";
  }
</script>

<WidgetCard {title} {tag} accent="#7c3aed">
  <div class="control">
    <button
      type="button"
      class:switch={variant === "switch"}
      class:on={current && !indeterminate}
      aria-label={title}
      aria-checked={indeterminate ? "mixed" : current}
      role="checkbox"
      {disabled}
      onclick={change}
    >
      {#if variant === "checkbox"}<span class="box" aria-hidden="true">{indeterminate ? "−" : current ? "✓" : ""}</span>{:else}<span class="track" aria-hidden="true"><span></span></span>{/if}
      <span>{indeterminate ? indeterminateLabel : current ? trueLabel : falseLabel}</span>
    </button>
    {#if configError}<p class="error">CONFIG: {configError}</p>
    {:else if qualityLocked}<p class="warning">READ-ONLY: quality is not GOOD</p>
    {:else if status}<p class="status">{status}</p>{/if}
  </div>
</WidgetCard>

<style>
  .control { height: 100%; box-sizing: border-box; padding: 7px; display: grid; align-content: center; gap: 5px; }
  button { min-height: 30px; display: flex; justify-content: center; align-items: center; gap: 7px; border: 1px solid #a78bfa; border-radius: 4px; background: #faf5ff; color: #4c1d95; font: 800 11px "Segoe UI", sans-serif; cursor: pointer; }
  button.on { border-color: #6d28d9; background: #ede9fe; } button:disabled { opacity: .55; cursor: not-allowed; }
  .box { width: 15px; height: 15px; display: inline-grid; place-items: center; border: 1px solid currentColor; border-radius: 2px; }
  .track { width: 27px; height: 15px; padding: 2px; box-sizing: border-box; border-radius: 99px; background: #94a3b8; } .track span { display: block; width: 11px; height: 11px; border-radius: 50%; background: white; transition: transform .1s; } .on .track { background: #7c3aed; } .on .track span { transform: translateX(12px); }
  p { margin: 0; text-align: center; font-size: 9px; font-weight: 700; } .error { color: #b91c1c; } .warning { color: #b45309; } .status { color: #0369a1; }
</style>

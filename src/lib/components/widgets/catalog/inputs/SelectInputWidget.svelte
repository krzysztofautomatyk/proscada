<script lang="ts">
  import type { WidgetRendererProps } from "$lib/components/widgets/shared/types";
  import { configOf, invokeWrite, readBoolean, readString, readStringList, writeResultLabel } from "$lib/components/widgets/shared/config";
  import WidgetCard from "$lib/components/widgets/shared/WidgetCard.svelte";

  interface Option {
    value: string;
    label: string;
    numeric: number | null;
  }

  let { widget, tag = null, design = false, onWrite }: WidgetRendererProps = $props();

  const config = $derived(configOf(widget));
  const rawVariant = $derived(readString(config, "variant", "select").toLowerCase());
  const variant = $derived(["select", "chips"].includes(rawVariant) ? rawVariant : "select");
  const title = $derived(readString(config, "title", "SELECT INPUT"));
  const options = $derived.by((): Option[] =>
    readStringList(config, "options").map((item) => {
      const splitAt = item.indexOf(":");
      const value = (splitAt < 0 ? item : item.slice(0, splitAt)).trim();
      const label = (splitAt < 0 ? item : item.slice(splitAt + 1)).trim() || value;
      const candidate = Number(value);
      return { value, label, numeric: value !== "" && Number.isFinite(candidate) ? candidate : null };
    }),
  );
  const configError = $derived(
    !["select", "chips"].includes(rawVariant)
      ? `invalid variant: ${rawVariant}`
      : options.length === 0 ? "options must contain at least one item" : "",
  );
  const qualityLocked = $derived(readBoolean(config, "disabledWhenBad", true) && tag?.quality !== "good");
  const disabled = $derived(design || qualityLocked || Boolean(configError));
  let selected = $state("");
  let status = $state("");

  $effect(() => {
    if (tag && Number.isFinite(tag.value)) {
      const match = options.find((option) => option.numeric === tag.value);
      if (match) selected = match.value;
    }
    if (!selected && options[0]) selected = options[0].value;
  });

  async function select(value: string) {
    if (disabled) return;
    selected = value;
    const option = options.find((item) => item.value === value);
    if (!option) {
      status = "Invalid selection";
      return;
    }
    if (option.numeric === null) {
      status = "requires numeric tag mapping";
      return;
    }
    status = "WRITE REQUESTED";
    try {
      status = writeResultLabel(await invokeWrite(widget, design, onWrite, option.numeric));
    } catch (error) {
      status = `WRITE REJECTED: ${error instanceof Error ? error.message : String(error)}`;
    }
  }
</script>

<WidgetCard {title} {tag} accent="#b45309">
  <div class="control">
    {#if variant === "select"}
      <select aria-label={title} value={selected} disabled={disabled} onchange={(event) => void select(event.currentTarget.value)}>
        {#each options as option}<option value={option.value}>{option.label}</option>{/each}
      </select>
    {:else}
      <div class="chips" aria-label={title} role="group">
        {#each options as option}
          <button type="button" class:selected={selected === option.value} aria-pressed={selected === option.value} disabled={disabled} onclick={() => void select(option.value)}>{option.label}</button>
        {/each}
      </div>
    {/if}
    {#if configError}<p class="error">CONFIG: {configError}</p>
    {:else if qualityLocked}<p class="warning">READ-ONLY: quality is not GOOD</p>
    {:else if status}<p class:notice={status === "requires numeric tag mapping"} class="status">{status}</p>{/if}
  </div>
</WidgetCard>

<style>
  .control { height: 100%; box-sizing: border-box; padding: 7px; display: grid; align-content: center; gap: 6px; }
  select { min-width: 0; min-height: 29px; border: 1px solid #d97706; border-radius: 4px; padding: 3px 6px; background: #fffbeb; color: #78350f; font: 700 11px "Segoe UI", sans-serif; }
  .chips { display: flex; flex-wrap: wrap; justify-content: center; gap: 4px; } button { min-height: 27px; border: 1px solid #fbbf24; border-radius: 999px; padding: 3px 7px; background: #fffbeb; color: #78350f; font: 700 10px "Segoe UI", sans-serif; cursor: pointer; } button.selected { border-color: #b45309; background: #fef3c7; } :disabled { opacity: .55; cursor: not-allowed; }
  p { margin: 0; text-align: center; font-size: 9px; font-weight: 700; } .error { color: #b91c1c; } .warning, .notice { color: #b45309; } .status { color: #0369a1; }
</style>

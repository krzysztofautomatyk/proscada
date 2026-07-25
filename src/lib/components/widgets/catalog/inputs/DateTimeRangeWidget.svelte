<script lang="ts">
  import type { WidgetRendererProps } from "$lib/components/widgets/shared/types";
  import { configOf, readString, readStringList } from "$lib/components/widgets/shared/config";
  import WidgetCard from "$lib/components/widgets/shared/WidgetCard.svelte";

  function localDateTime(date: Date): string {
    const offset = date.getTimezoneOffset() * 60_000;
    return new Date(date.getTime() - offset).toISOString().slice(0, 16);
  }

  function relativeMs(preset: string): number | null {
    const match = /^(\d+)\s*([mhd])$/i.exec(preset);
    if (!match) return null;
    const amount = Number(match[1]);
    const multiplier = match[2].toLowerCase() === "m" ? 60_000 : match[2].toLowerCase() === "h" ? 3_600_000 : 86_400_000;
    return Number.isFinite(amount) && amount > 0 ? amount * multiplier : null;
  }

  let { widget, tag = null, design = false, onWrite }: WidgetRendererProps = $props();

  const config = $derived(configOf(widget));
  const rawVariant = $derived(readString(config, "variant", "absolute").toLowerCase());
  const variant = $derived(["absolute", "relative"].includes(rawVariant) ? rawVariant : "absolute");
  const title = $derived(readString(config, "title", "TIME RANGE"));
  const timezone = $derived(readString(config, "timezone", Intl.DateTimeFormat().resolvedOptions().timeZone || "Local"));
  const presets = $derived(readStringList(config, "presets", ["15m", "1h", "8h", "24h"]));
  const invalidPreset = $derived(presets.find((preset) => relativeMs(preset) === null) ?? "");
  const configError = $derived(!["absolute", "relative"].includes(rawVariant) ? `invalid variant: ${rawVariant}` : invalidPreset ? `invalid relative preset: ${invalidPreset}` : "");
  let from = $state("");
  let to = $state("");
  let selectedPreset = $state("");

  $effect(() => {
    if (!from) from = readString(config, "from", localDateTime(new Date(Date.now() - 3_600_000)));
    if (!to) to = readString(config, "to", localDateTime(new Date()));
  });

  const rangeError = $derived(!from || !to || Number.isNaN(Date.parse(from)) || Number.isNaN(Date.parse(to)) ? "Enter valid dates" : Date.parse(from) > Date.parse(to) ? "From must be before or equal to To" : "");

  function choosePreset(preset: string) {
    if (design || configError) return;
    const duration = relativeMs(preset);
    if (duration === null) return;
    const now = new Date();
    to = localDateTime(now);
    from = localDateTime(new Date(now.getTime() - duration));
    selectedPreset = preset;
  }
</script>

<WidgetCard {title} subtitle={`LOCAL FILTER · ${timezone}`} {tag} accent="#475569">
  <div class="control">
    {#if variant === "absolute"}
      <label>FROM <input aria-label={`${title} from`} type="datetime-local" bind:value={from} disabled={design || Boolean(configError)} /></label>
      <label>TO <input aria-label={`${title} to`} type="datetime-local" bind:value={to} disabled={design || Boolean(configError)} /></label>
    {:else}
      <div class="presets" role="group" aria-label={`${title} relative presets`}>
        {#each presets as preset}<button type="button" class:selected={selectedPreset === preset} disabled={design || Boolean(configError)} onclick={() => choosePreset(preset)}>{preset}</button>{/each}
      </div>
    {/if}
    <div class="preview">{from || "—"} → {to || "—"}</div>
    {#if configError}<p class="error">CONFIG: {configError}</p>
    {:else if rangeError}<p class="error">{rangeError}</p>
    {:else}<p class="filter">LOCAL FILTER ONLY · no tag write</p>{/if}
  </div>
</WidgetCard>

<style>
  .control { height: 100%; box-sizing: border-box; padding: 6px; display: grid; align-content: center; gap: 4px; } label { display: grid; grid-template-columns: 33px minmax(0, 1fr); align-items: center; gap: 4px; color: #475569; font-size: 9px; font-weight: 800; } input { min-width: 0; min-height: 25px; border: 1px solid #94a3b8; border-radius: 3px; padding: 2px; color: #334155; font: 9px "Segoe UI", sans-serif; } .presets { display: flex; flex-wrap: wrap; justify-content: center; gap: 4px; } button { min-height: 24px; border: 1px solid #94a3b8; border-radius: 3px; background: #f8fafc; color: #334155; font: 800 9px "Segoe UI", sans-serif; cursor: pointer; } button.selected { background: #e2e8f0; border-color: #475569; } :disabled { opacity: .55; cursor: not-allowed; } .preview { overflow: hidden; color: #475569; font-size: 9px; text-align: center; text-overflow: ellipsis; white-space: nowrap; } p { margin: 0; text-align: center; font-size: 9px; font-weight: 700; } .error { color: #b91c1c; } .filter { color: #0369a1; }
</style>

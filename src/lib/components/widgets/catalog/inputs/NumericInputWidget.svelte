<script lang="ts">
  import type { WidgetRendererProps } from "$lib/components/widgets/shared/types";
  import { clamp, configOf, invokeWrite, readNumber, readString, tagNumber, writeResultLabel } from "$lib/components/widgets/shared/config";
  import WidgetCard from "$lib/components/widgets/shared/WidgetCard.svelte";

  import { project } from "$lib/stores/app";

  let { widget, tag = null, design = false, onWrite }: WidgetRendererProps = $props();

  const config = $derived(configOf(widget));
  const boundTagDef = $derived(
    widget.tag_id ? $project?.tags.find((t) => t.id === widget.tag_id) : undefined,
  );
  const rawVariant = $derived(readString(config, "variant", "stepper").toLowerCase());
  const variant = $derived(["field", "slider", "stepper"].includes(rawVariant) ? rawVariant : "stepper");
  const title = $derived(readString(config, "title", boundTagDef?.name ?? "SP VALUE"));
  const min = $derived(readNumber(config, "min", 0));
  const max = $derived(readNumber(config, "max", 1000));
  const step = $derived(readNumber(config, "step", 10, Number.MIN_VALUE));
  const decimals = $derived(
    config.decimals !== undefined && config.decimals !== ""
      ? Math.floor(readNumber(config, "decimals", 0, 0, 8))
      : (boundTagDef?.decimals ?? 0),
  );
  const unit = $derived(
    config.unit !== undefined && config.unit !== ""
      ? readString(config, "unit", "")
      : (boundTagDef?.unit ?? ""),
  );
  const accent = $derived(readString(config, "labelColor", "#0f766e"));
  const commitMode = $derived(readString(config, "commitMode", "change").toLowerCase());
  const invalidNumberKey = $derived(
    ["min", "max", "step", "decimals"].find(
      (key) => config[key] !== undefined && !Number.isFinite(Number(config[key])),
    ) ?? "",
  );
  const configError = $derived(
    !["field", "slider", "stepper"].includes(rawVariant)
      ? `invalid variant: ${rawVariant}`
      : invalidNumberKey ? `${invalidNumberKey} must be numeric`
      : config.step !== undefined && Number(config.step) <= 0 ? "step must be greater than zero"
      : config.decimals !== undefined && !Number.isInteger(Number(config.decimals)) ? "decimals must be an integer"
      : max < min
        ? "max must be greater than or equal to min"
        : !["change", "release"].includes(commitMode)
          ? `invalid commitMode: ${commitMode}`
          : "",
  );
  const qualityLocked = $derived(tag?.quality === "bad");
  const readOnly = $derived(design || qualityLocked || Boolean(configError));
  let value = $state(0);
  let draft = $state("");
  let status = $state("");

  $effect(() => {
    const next = clamp(tagNumber(tag, readNumber(config, "defaultValue", 200)), min, max);
    value = next;
    draft = next.toFixed(decimals);
  });

  async function update(raw: string, write: boolean) {
    const next = Number(raw);
    if (!Number.isFinite(next)) {
      status = "Enter a finite numeric value";
      return;
    }
    const bounded = clamp(next, min, max);
    value = bounded;
    draft = bounded.toFixed(decimals);
    status = bounded !== next ? `Limited to ${bounded.toFixed(decimals)}` : "";
    if (write && !configError) {
      status = "WRITE REQUESTED";
      try {
        status = writeResultLabel(await invokeWrite(widget, design, onWrite, bounded));
      } catch (error) {
        status = `WRITE REJECTED: ${error instanceof Error ? error.message : String(error)}`;
      }
    }
  }

  function sliderInput(raw: string) {
    draft = raw;
    void update(raw, commitMode === "change");
  }
</script>

<WidgetCard {title} {tag} {accent}>
  <div class="control">
    <div class="reading">{value.toFixed(decimals)}{unit ? ` ${unit}` : ""}</div>
    {#if variant === "slider"}
      <input
        aria-label={title}
        type="range"
        {min}
        {max}
        {step}
        value={value}
        disabled={readOnly}
        oninput={(event) => sliderInput(event.currentTarget.value)}
        onchange={(event) => {
          if (commitMode === "release") void update(event.currentTarget.value, true);
        }}
      />
    {:else}
      <div class:field-only={variant === "field"} class="entry">
        {#if variant === "stepper"}
          <button type="button" aria-label={`Decrease ${title}`} disabled={readOnly} onclick={() => void update(String(value - step), true)}>−</button>
        {/if}
        <input
          aria-label={title}
          type="number"
          {min}
          {max}
          {step}
          value={draft}
          disabled={readOnly}
          oninput={(event) => (draft = event.currentTarget.value)}
          onchange={(event) => void update(event.currentTarget.value, true)}
          onkeydown={(event) => {
            if (event.key === "Enter") void update(event.currentTarget.value, true);
          }}
        />
        {#if variant === "stepper"}
          <button type="button" aria-label={`Increase ${title}`} disabled={readOnly} onclick={() => void update(String(value + step), true)}>+</button>
        {/if}
      </div>
    {/if}
    {#if configError}<p class="error">CONFIG: {configError}</p>
    {:else if qualityLocked}<p class="warning">READ-ONLY: bad quality</p>
    {:else if status}<p class="status">{status}</p>{/if}
  </div>
</WidgetCard>

<style>
  .control { height: 100%; box-sizing: border-box; padding: 7px; display: grid; align-content: center; gap: 6px; }
  .reading { color: var(--accent, #0f766e); font-size: 13px; font-weight: 800; text-align: center; font-variant-numeric: tabular-nums; }
  .entry { display: grid; grid-template-columns: 30px minmax(0, 1fr) 30px; gap: 4px; }
  .entry.field-only { grid-template-columns: minmax(0, 1fr); }
  input, button { min-width: 0; min-height: 28px; box-sizing: border-box; border: 1px solid #94a3b8; border-radius: 4px; font: inherit; }
  input[type="number"] { width: 100%; padding: 3px 6px; text-align: center; color: #0f172a; }
  input[type="range"] { width: 100%; accent-color: #0f766e; }
  button { background: #f8fafc; color: #0f766e; font-size: 16px; font-weight: 800; cursor: pointer; }
  button:hover:not(:disabled) { background: #ccfbf1; }
  :disabled { opacity: .55; cursor: not-allowed; }
  p { margin: 0; text-align: center; font-size: 9px; font-weight: 700; }
  .error { color: #b91c1c; } .warning { color: #b45309; } .status { color: #0369a1; }
</style>

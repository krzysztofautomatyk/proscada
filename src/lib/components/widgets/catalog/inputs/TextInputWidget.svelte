<script lang="ts">
  import type { WidgetRendererProps } from "$lib/components/widgets/shared/types";
  import { configOf, invokeWrite, readBoolean, readNumber, readString } from "$lib/components/widgets/shared/config";
  import WidgetCard from "$lib/components/widgets/shared/WidgetCard.svelte";

  let { widget, tag = null, design = false, onWrite }: WidgetRendererProps = $props();

  const config = $derived(configOf(widget));
  const rawMode = $derived(readString(config, "mode", readString(config, "variant", "text")).toLowerCase());
  const mode = $derived(["text", "number"].includes(rawMode) ? rawMode : "text");
  const title = $derived(readString(config, "title", "TEXT INPUT"));
  const placeholder = $derived(readString(config, "placeholder", ""));
  const maxLength = $derived(Math.floor(readNumber(config, "maxLength", 256, 0, 4096)));
  const invalidMaxLength = $derived(
    config.maxLength !== undefined && (!Number.isFinite(Number(config.maxLength)) || Number(config.maxLength) < 0 || !Number.isInteger(Number(config.maxLength))),
  );
  const pattern = $derived(readString(config, "pattern", ""));
  const patternError = $derived.by(() => {
    if (!pattern) return "";
    try {
      new RegExp(pattern);
      return "";
    } catch {
      return "pattern is not a valid regular expression";
    }
  });
  const configError = $derived(!["text", "number"].includes(rawMode) ? `invalid mode: ${rawMode}` : invalidMaxLength ? "maxLength must be a non-negative integer" : patternError);
  const qualityLocked = $derived(readBoolean(config, "disabledWhenBad", true) && tag?.quality !== "good");
  const disabled = $derived(design || qualityLocked || Boolean(configError));
  let draft = $state("");
  let message = $state("");
  let initialized = $state(false);

  $effect(() => {
    if (!initialized) {
      draft = readString(config, "defaultValue", "");
      initialized = true;
    }
    if (mode === "number" && tag && Number.isFinite(tag.value)) draft = String(tag.value);
    else if (mode === "text" && tag && tag.string_value !== undefined) draft = tag.string_value;
  });

  function validate(): string {
    if (draft.length > maxLength) return `Maximum length is ${maxLength}`;
    if (pattern) {
      try {
        if (!new RegExp(pattern).test(draft)) return "Value does not match pattern";
      } catch {
        return "Invalid pattern configuration";
      }
    }
    if (mode === "number" && !Number.isFinite(Number(draft))) return "Enter a finite numeric value";
    return "";
  }

  function commit() {
    if (disabled) return;
    const error = validate();
    if (error) {
      message = error;
      return;
    }
    // The backend write gate accepts a numeric process value only; text mode
    // has no Modbus representation, so it is refused instead of coerced.
    if (mode !== "number") {
      message = "TEXT WRITE NOT SUPPORTED BY THE PROCESS GATEWAY";
      return;
    }
    message = invokeWrite(widget, design, onWrite, Number(draft))
      ? "WRITE REQUESTED"
      : "TAG WRITE UNAVAILABLE";
  }

  function cancel() {
    draft = mode === "number" && tag && Number.isFinite(tag.value)
      ? String(tag.value)
      : mode === "text" && tag && tag.string_value !== undefined
      ? tag.string_value
      : readString(config, "defaultValue", "");
    message = "Changes cancelled";
  }
</script>

<WidgetCard {title} {tag} accent="#0369a1">
  <form class="control" onsubmit={(event) => { event.preventDefault(); commit(); }}>
    <input
      aria-label={title}
      type={mode}
      {placeholder}
      maxlength={maxLength}
      value={draft}
      disabled={disabled}
      oninput={(event) => { draft = event.currentTarget.value; message = ""; }}
    />
    <div class="actions"><button type="submit" disabled={disabled}>COMMIT</button><button type="button" disabled={disabled} onclick={cancel}>CANCEL</button></div>
    {#if configError}<p class="error">CONFIG: {configError}</p>
    {:else if qualityLocked}<p class="warning">READ-ONLY: quality is not GOOD</p>
    {:else if message}<p class:notice={message === "form value only"}>{message}</p>{/if}
  </form>
</WidgetCard>

<style>
  .control { height: 100%; box-sizing: border-box; padding: 7px; display: grid; align-content: center; gap: 5px; } input { min-width: 0; min-height: 28px; border: 1px solid #7dd3fc; border-radius: 4px; padding: 3px 6px; color: #0c4a6e; font: 11px "Segoe UI", sans-serif; } .actions { display: flex; gap: 4px; } button { min-height: 24px; flex: 1; border: 1px solid #38bdf8; border-radius: 3px; background: #f0f9ff; color: #075985; font: 800 9px "Segoe UI", sans-serif; cursor: pointer; } :disabled { opacity: .55; cursor: not-allowed; } p { margin: 0; text-align: center; font-size: 9px; font-weight: 700; color: #0369a1; } .error { color: #b91c1c; } .warning, .notice { color: #b45309; }
</style>

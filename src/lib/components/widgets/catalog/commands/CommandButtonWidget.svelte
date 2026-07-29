<script lang="ts">
  import type { WidgetRendererProps } from "$lib/components/widgets/shared/types";
  import { configOf, invokeWrite, readBoolean, readNumber, readString, writeResultLabel } from "$lib/components/widgets/shared/config";
  import WidgetCard from "$lib/components/widgets/shared/WidgetCard.svelte";
  import { project } from "$lib/stores/app";

  let { widget, tag = null, design = false, onWrite }: WidgetRendererProps = $props();

  const config = $derived(configOf(widget));
  const rawMode = $derived(readString(config, "mode", "set").toLowerCase());
  const mode = $derived(["set", "reset", "toggle", "momentary", "value", "action"].includes(rawMode) ? rawMode : "set");
  const label = $derived(readString(config, "label", "COMMAND"));
  const pendingLabel = $derived(readString(config, "pendingLabel", "HOLDING…"));
  const watchdogConfigured = $derived(readBoolean(config, "watchdogConfigured", false));
  const disabledWhenBad = $derived(readBoolean(config, "disabledWhenBad", true));
  const configuredValue = $derived(readNumber(config, "writeValue", 1));
  const rawWriteValue = $derived(config.writeValue);
  const invalidWriteValue = $derived(
    (mode === "value" || mode === "action") && rawWriteValue !== undefined && !Number.isFinite(Number(rawWriteValue)),
  );
  const configError = $derived(
    !["set", "reset", "toggle", "momentary", "value", "action"].includes(rawMode)
      ? `invalid mode: ${rawMode}`
      : invalidWriteValue ? "writeValue must be numeric" : "",
  );
  const qualityLocked = $derived(disabledWhenBad && tag?.quality !== "good");
  const momentaryUnsafe = $derived(mode === "momentary" && !watchdogConfigured);
  const pinMomentaryUnsafe = $derived(
    mode === "momentary" && $project?.session_config?.pin_challenge_on_write === true,
  );
  const writeUnavailable = $derived(!widget.tag_id || !onWrite);
  const disabled = $derived(design || !widget.tag_id || !onWrite || Boolean(configError) || qualityLocked || momentaryUnsafe || pinMomentaryUnsafe);
  let holding = $state(false);
  let status = $state("");

  function shouldConfirm() {
    return readBoolean(config, "confirm", false) && ["set", "reset", "toggle", "value"].includes(mode);
  }

  async function send(value: number, needsConfirmation = true) {
    if (disabled) return;
    if (needsConfirmation && shouldConfirm()) {
      const message = readString(config, "confirmText", `Send command to ${widget.tag_id}?`);
      if (!window.confirm(message)) return;
    }
    status = "COMMAND REQUESTED";
    try {
      status = writeResultLabel(await invokeWrite(widget, design, onWrite, value), "COMMAND");
    } catch (error) {
      status = `COMMAND REJECTED: ${error instanceof Error ? error.message : String(error)}`;
    }
  }

  function commandValue() {
    if (mode === "set") return 1;
    if (mode === "reset") return 0;
    if (mode === "toggle") {
      const current = tag?.bool_value ?? (tag?.value ?? 0) !== 0;
      return current ? 0 : 1;
    }
    return configuredValue;
  }

  function activate() {
    if (mode !== "momentary") void send(commandValue());
  }

  function press() {
    if (mode !== "momentary" || disabled || holding) return;
    holding = true;
    void send(1, false);
  }

  async function release() {
    if (mode !== "momentary" || !holding) return;
    holding = false;
    status = "COMMAND RELEASE REQUESTED";
    try {
      status = writeResultLabel(await invokeWrite(widget, design, onWrite, 0), "COMMAND");
    } catch (error) {
      status = `COMMAND RELEASE REJECTED: ${error instanceof Error ? error.message : String(error)}`;
    }
  }
</script>

<WidgetCard title="COMMAND" subtitle={design ? "DESIGN" : "TRANSPORT INTENT"} {tag} accent="#1d4ed8">
  <div class="command">
    <button
      type="button"
      class:holding
      aria-label={label}
      aria-pressed={mode === "momentary" ? holding : undefined}
      {disabled}
      onclick={activate}
      onpointerdown={press}
      onpointerup={() => void release()}
      onpointercancel={() => void release()}
      onblur={() => void release()}
      onkeydown={(event) => {
        if (mode === "momentary" && (event.key === " " || event.key === "Enter")) {
          event.preventDefault();
          press();
        }
      }}
      onkeyup={(event) => {
        if (mode === "momentary" && (event.key === " " || event.key === "Enter")) {
          event.preventDefault();
          void release();
        }
      }}
    >{holding ? pendingLabel : label}</button>
    {#if configError}<p class="error">CONFIG: {configError}</p>
    {:else if momentaryUnsafe}<p class="error">MOMENTARY DISABLED: PLC watchdog required</p>
    {:else if pinMomentaryUnsafe}<p class="error">MOMENTARY DISABLED: interactive PIN cannot guarantee release</p>
    {:else if qualityLocked}<p class="warning">READ-ONLY: quality is not GOOD</p>
    {:else if writeUnavailable}<p class="warning">TAG WRITE UNAVAILABLE</p>
    {:else if status}<p class="sent">{status}</p>{/if}
  </div>
</WidgetCard>

<style>
  .command { height: 100%; box-sizing: border-box; padding: 7px; display: grid; align-content: center; gap: 5px; }
  button { min-height: 34px; border: 1px solid #1d4ed8; border-radius: 4px; background: linear-gradient(#2563eb, #1d4ed8); color: white; font: 800 11px "Segoe UI", sans-serif; letter-spacing: .05em; cursor: pointer; }
  button:hover:not(:disabled), button.holding { background: #1e40af; }
  button:focus-visible { outline: 3px solid #93c5fd; outline-offset: 2px; }
  button:disabled { opacity: .55; cursor: not-allowed; }
  p { margin: 0; text-align: center; font-size: 9px; font-weight: 800; }
  .error { color: #b91c1c; } .warning { color: #b45309; } .sent { color: #047857; }
</style>

<script lang="ts">
  import type { WidgetRendererProps } from "../../shared/types";
  import { configOf, readNumber, readString, tagNumber } from "../../shared/config";
  import QualityBadge from "../../shared/QualityBadge.svelte";

  interface StateDefinition {
    value: number;
    label: string;
    color: string;
    icon: string;
  }

  let { widget, tag = null }: WidgetRendererProps = $props();

  const config = $derived(configOf(widget));
  const variant = $derived(readString(config, "variant", "bit"));
  const title = $derived(readString(config, "title", "STATE"));
  const bitIndex = $derived(Math.round(readNumber(config, "bitIndex", 0, 0, 15)));
  const rawValue = $derived(tagNumber(tag));
  const effectiveValue = $derived(
    variant === "bit" ? ((Math.trunc(rawValue) >>> bitIndex) & 1) : rawValue,
  );

  function parseStates(raw: string): StateDefinition[] {
    const parsed = raw
      .split(/\r?\n|;/)
      .map((line) => line.trim())
      .filter(Boolean)
      .map((line) => {
        const [value, label, color, icon] = line.split("|").map((part) => part.trim());
        return {
          value: Number(value),
          label: label || value,
          color: color || "#64748b",
          icon: icon || "●",
        };
      })
      .filter((state) => Number.isFinite(state.value));
    return parsed.length > 0
      ? parsed
      : [
          { value: 0, label: "OFF", color: "#64748b", icon: "○" },
          { value: 1, label: "ON", color: "#16a34a", icon: "●" },
        ];
  }

  const states = $derived(parseStates(readString(config, "states", "")));
  const state = $derived(
    states.find((item) => Math.abs(item.value - effectiveValue) < 0.0001) ?? {
      value: effectiveValue,
      label: readString(config, "unknownLabel", `UNKNOWN (${effectiveValue})`),
      color: "#a16207",
      icon: "?",
    },
  );
  const displayColor = $derived(tag?.quality === "bad" ? "#b91c1c" : state.color);
</script>

<div class="indicator" class:bad={tag?.quality === "bad"} style:--state-color={displayColor}>
  <div class="state">
    <span class="lamp" aria-hidden="true">{state.icon}</span>
    <div>
      <small>{title}</small>
      <strong>{state.label}</strong>
    </div>
  </div>
  <QualityBadge {tag} showAge />
</div>

<style>
  .indicator {
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 7px 9px;
    border: 1px solid color-mix(in srgb, var(--state-color) 45%, #cbd5e1);
    border-left: 5px solid var(--state-color);
    border-radius: 7px;
    background: #fff;
  }
  .state {
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 7px;
  }
  .lamp {
    color: var(--state-color);
    font-size: 21px;
    line-height: 1;
  }
  small,
  strong {
    display: block;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  small {
    color: #64748b;
    font-size: 8px;
    font-weight: 700;
  }
  strong {
    color: var(--state-color);
    font-size: 12px;
  }
  .bad {
    background: repeating-linear-gradient(135deg, #fff, #fff 6px, #fee2e2 6px, #fee2e2 12px);
  }
</style>


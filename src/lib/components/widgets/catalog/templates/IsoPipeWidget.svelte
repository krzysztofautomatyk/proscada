<script lang="ts">
  import type { WidgetDef, TagValue } from "$lib/types";

  interface Props {
    widget: WidgetDef;
    tag?: TagValue | null;
    design?: boolean;
    onWrite?: (tagId: string, value: number) => void;
  }

  let { widget, tag = null }: Props = $props();

  const cfg = $derived((widget.config ?? {}) as Record<string, unknown>);
  const str = (k: string, d = "") => String(cfg[k] ?? d);
  const active = $derived(tag?.bool_value ?? true);
</script>

<div class="iso-pipe-wrap">
  <svg viewBox="0 0 300 80" class="pipe-svg">
    <!-- Outer Pipe casing -->
    <path d="M 10 40 L 290 40" stroke="#7c838b" stroke-width="28" stroke-linecap="round" />
    <path d="M 10 40 L 290 40" stroke="#b9bec4" stroke-width="20" stroke-linecap="round" />

    <!-- Animated Flow Stream inside -->
    <path
      class="stream"
      class:flowing={active}
      d="M 20 40 L 280 40"
      stroke="#38bdf8"
      stroke-width="8"
      stroke-linecap="round"
      opacity={active ? 0.9 : 0.2}
    />
  </svg>
  <div class="pipe-label">{str("label", "Inlet Pipe")}</div>
</div>

<style>
  .iso-pipe-wrap {
    width: 100%;
    height: 100%;
    background: #eef8ff;
    border-radius: 8px;
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
  }
  .pipe-svg {
    width: 100%;
    height: 100%;
  }
  .pipe-label {
    position: absolute;
    bottom: 4px;
    font-size: 10px;
    font-weight: 700;
    color: #4b5563;
    background: rgba(255,255,255,0.7);
    padding: 1px 6px;
    border-radius: 4px;
  }
  .stream {
    stroke-dasharray: 8 16;
  }
  .stream.flowing {
    animation: pipeFlow 0.5s linear infinite;
  }
  @keyframes pipeFlow {
    to {
      stroke-dashoffset: -24;
    }
  }
</style>

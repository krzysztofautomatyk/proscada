<script lang="ts">
  import type { WidgetDef, TagValue } from "$lib/types";

  interface Props {
    widget: WidgetDef;
    tag?: TagValue | null;
    design?: boolean;
    onWrite?: (tagId: string, value: number) => void;
  }

  let { widget }: Props = $props();

  const cfg = $derived((widget.config ?? {}) as Record<string, unknown>);
  const str = (k: string, d = "") => String(cfg[k] ?? d);
</script>

<div class="iso-terrain-wrap">
  <svg viewBox="0 0 400 200" class="terrain-svg">
    <defs>
      <linearGradient id="skyGrad" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0" stop-color="#cfe8f7" />
        <stop offset="1" stop-color="#eef8ff" />
      </linearGradient>
      <linearGradient id="soilGrad" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0" stop-color="#7c5433" />
        <stop offset="1" stop-color="#4e3018" />
      </linearGradient>
    </defs>

    <!-- Sky -->
    <rect width="400" height="70" fill="url(#skyGrad)" />
    <!-- Grass line -->
    <rect y="64" width="400" height="12" fill="#5da33a" />
    <path d="M 0 64 Q 100 50 200 64 T 400 64" fill="#3f7d26" opacity="0.4" />
    <!-- Soil -->
    <rect y="76" width="400" height="124" fill="url(#soilGrad)" />

    <!-- Rocks in soil -->
    <circle cx="50" cy="110" r="6" fill="#4e3018" opacity="0.6" />
    <circle cx="120" cy="140" r="10" fill="#6b4a2f" opacity="0.7" />
    <circle cx="280" cy="105" r="8" fill="#4e3018" opacity="0.6" />
    <circle cx="340" cy="150" r="12" fill="#6b4a2f" opacity="0.7" />
  </svg>
  <div class="terrain-label">{str("label", "Soil & Grass Cutaway")}</div>
</div>

<style>
  .iso-terrain-wrap {
    width: 100%;
    height: 100%;
    position: relative;
    border-radius: 8px;
    overflow: hidden;
  }
  .terrain-svg {
    width: 100%;
    height: 100%;
    display: block;
  }
  .terrain-label {
    position: absolute;
    top: 6px;
    left: 8px;
    font-size: 10px;
    font-weight: 700;
    color: #1f2937;
    background: rgba(255,255,255,0.7);
    padding: 2px 6px;
    border-radius: 4px;
  }
</style>

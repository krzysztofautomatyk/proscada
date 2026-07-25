<script lang="ts">
  import type { Quality } from "$lib/types";
  import type { WidgetRendererProps } from "$lib/components/widgets/shared/types";
  import { configOf, readString, readBoolean } from "$lib/components/widgets/shared/config";
  import WidgetCard from "$lib/components/widgets/shared/WidgetCard.svelte";
  import EmptyState from "$lib/components/widgets/shared/EmptyState.svelte";

  let { widget, tag = null, design = false }: WidgetRendererProps = $props();

  interface Sample {
    v: number | null;
    q: Quality;
  }

  const cfg = $derived(configOf(widget));
  const title = $derived(readString(cfg, "title", "Trend"));
  const seriesLabel = $derived(readString(cfg, "seriesLabel", "Series"));
  const unit = $derived(readString(cfg, "unit", ""));
  const liveAppend = $derived(readBoolean(cfg, "liveAppend", true));

  const GAP_TOKENS = new Set(["null", "nan", "gap", "_", "-", "none"]);

  const parsed = $derived.by<{ samples: Sample[]; error: string | null }>(() => {
    const raw = cfg["points"];
    const samples: Sample[] = [];
    if (Array.isArray(raw)) {
      for (const item of raw) {
        if (item === null) {
          samples.push({ v: null, q: "good" });
          continue;
        }
        const n = Number(item);
        if (!Number.isFinite(n)) {
          return { samples: [], error: `points contains non-numeric value: ${String(item)}` };
        }
        samples.push({ v: n, q: "good" });
      }
      return { samples, error: null };
    }
    if (typeof raw === "string" && raw.trim() !== "") {
      const tokens = raw.split(/[\s,;]+/).filter((t) => t !== "");
      for (const token of tokens) {
        if (GAP_TOKENS.has(token.toLowerCase())) {
          samples.push({ v: null, q: "good" });
          continue;
        }
        const n = Number(token);
        if (!Number.isFinite(n)) {
          return { samples: [], error: `points: cannot parse "${token}" as number` };
        }
        samples.push({ v: n, q: "good" });
      }
      return { samples, error: null };
    }
    return { samples: [], error: null };
  });

  const configError = $derived(parsed.error);

  const samples = $derived.by<Sample[]>(() => {
    const base = [...parsed.samples];
    if (liveAppend && tag && !design) {
      base.push({
        v: tag.quality === "bad" ? null : tag.value,
        q: tag.quality,
      });
    }
    return base;
  });

  function readBound(key: string): number | null {
    const value = cfg[key];
    if (value === undefined || value === null || value === "") return null;
    const n = Number(value);
    return Number.isFinite(n) ? n : null;
  }

  const domain = $derived.by(() => {
    const nums = samples.map((s) => s.v).filter((v): v is number => v !== null);
    const cfgMin = readBound("min");
    const cfgMax = readBound("max");
    let lo = cfgMin ?? (nums.length ? Math.min(...nums) : 0);
    let hi = cfgMax ?? (nums.length ? Math.max(...nums) : 1);
    if (lo === hi) {
      lo -= 1;
      hi += 1;
    }
    if (lo > hi) [lo, hi] = [hi, lo];
    return { lo, hi };
  });

  const W = 300;
  const H = 130;
  const PAD_L = 40;
  const PAD_R = 10;
  const PAD_T = 12;
  const PAD_B = 22;
  const plotW = W - PAD_L - PAD_R;
  const plotH = H - PAD_T - PAD_B;

  function xAt(i: number, total: number): number {
    if (total <= 1) return PAD_L + plotW / 2;
    return PAD_L + (i / (total - 1)) * plotW;
  }
  function yAt(v: number): number {
    const { lo, hi } = domain;
    const frac = (v - lo) / (hi - lo || 1);
    return PAD_T + (1 - frac) * plotH;
  }

  const segments = $derived.by<string[]>(() => {
    const total = samples.length;
    const out: string[] = [];
    let current: string[] = [];
    samples.forEach((s, i) => {
      const broken = s.v === null || s.q === "bad";
      if (broken) {
        if (current.length > 1) out.push(current.join(" "));
        current = [];
        return;
      }
      current.push(`${xAt(i, total).toFixed(1)},${yAt(s.v as number).toFixed(1)}`);
    });
    if (current.length > 1) out.push(current.join(" "));
    return out;
  });

  const markers = $derived.by(() => {
    const total = samples.length;
    return samples
      .map((s, i) => ({ s, i }))
      .filter(({ s }) => s.v !== null)
      .map(({ s, i }) => ({
        x: xAt(i, total),
        y: yAt(s.v as number),
        q: s.q,
      }));
  });

  const gapCount = $derived(samples.filter((s) => s.v === null || s.q === "bad").length);
  const yTicks = $derived([domain.hi, (domain.hi + domain.lo) / 2, domain.lo]);

  function fmt(n: number): string {
    return Math.abs(n) >= 100 ? n.toFixed(0) : n.toFixed(2);
  }
</script>

<WidgetCard {title} subtitle={seriesLabel} {tag} accent="#0891b2">
  <div class="trend">
    {#if configError}
      <div class="cfg-error" role="alert">
        <strong>Config error</strong><span>{configError}</span>
      </div>
    {:else if samples.length === 0}
      <EmptyState title="No trend data" detail="Provide points in configuration" icon="∿" />
    {:else}
      <svg viewBox="0 0 {W} {H}" preserveAspectRatio="none" class="chart" role="img" aria-label="{seriesLabel} trend chart">
        <line x1={PAD_L} y1={PAD_T} x2={PAD_L} y2={PAD_T + plotH} class="axis" />
        <line x1={PAD_L} y1={PAD_T + plotH} x2={PAD_L + plotW} y2={PAD_T + plotH} class="axis" />
        {#each yTicks as t, ti (ti)}
          {@const y = yAt(t)}
          <line x1={PAD_L} y1={y} x2={PAD_L + plotW} y2={y} class="grid" />
          <text x={PAD_L - 4} y={y + 3} class="tick" text-anchor="end">{fmt(t)}</text>
        {/each}
        {#each segments as seg, si (si)}
          <polyline points={seg} class="line" />
        {/each}
        {#each markers as m, mi (mi)}
          <circle
            cx={m.x}
            cy={m.y}
            r={m.q === "good" ? 1.8 : 2.4}
            class="dot {m.q}"
          />
        {/each}
        <text x={PAD_L} y={H - 6} class="axis-label" text-anchor="start">1</text>
        <text x={PAD_L + plotW} y={H - 6} class="axis-label" text-anchor="end">{samples.length}</text>
      </svg>
      <div class="legend">
        <span class="swatch"></span>
        <span class="legend-label">{seriesLabel}{unit ? ` (${unit})` : ""}</span>
        {#if tag}<span class="live">now {tag.value.toFixed(2)}{unit}</span>{/if}
        {#if gapCount > 0}<span class="gaps">{gapCount} quality gap{gapCount > 1 ? "s" : ""}</span>{/if}
      </div>
    {/if}
  </div>
</WidgetCard>

<style>
  .trend {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    padding: 4px 6px;
  }
  .chart {
    flex: 1;
    width: 100%;
    min-height: 0;
  }
  .axis {
    stroke: #94a3b8;
    stroke-width: 1;
  }
  .grid {
    stroke: #e2e8f0;
    stroke-width: 0.5;
  }
  .line {
    fill: none;
    stroke: #0891b2;
    stroke-width: 1.6;
    stroke-linejoin: round;
    stroke-linecap: round;
    vector-effect: non-scaling-stroke;
  }
  .dot {
    fill: #0891b2;
  }
  .dot.uncertain {
    fill: #eab308;
  }
  .dot.bad {
    fill: #dc2626;
  }
  .tick,
  .axis-label {
    fill: #64748b;
    font-size: 8px;
    font-family: "Segoe UI", system-ui, sans-serif;
  }
  .legend {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 2px 0;
    font-size: 9px;
    color: #475569;
    flex-wrap: wrap;
  }
  .swatch {
    width: 12px;
    height: 3px;
    border-radius: 2px;
    background: #0891b2;
  }
  .legend-label {
    font-weight: 700;
  }
  .live {
    color: #0e7490;
    font-weight: 700;
  }
  .gaps {
    color: #b45309;
    font-weight: 700;
  }
  .cfg-error {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 8px;
    margin: auto;
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

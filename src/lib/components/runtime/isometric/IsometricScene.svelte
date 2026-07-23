<script lang="ts">
  /**
   * Minimal cutaway wet-well: soil, grass, tank + water, inlet pipe only.
   * Iso 2:1 (ry = rx/2). Level from Modbus via vm.
   */
  import { seeded } from "$lib/iso";
  import type { WaterTankViewModel } from "$lib/stores/waterTank.svelte";

  interface Props {
    vm: WaterTankViewModel;
    animated?: boolean;
  }

  let { vm, animated = true }: Props = $props();

  const CX = 470;
  const TOP = 310;
  const R = 175;
  const T = 22;
  const H = 280;
  const RI = R - T;
  const RY = R / 2;
  const RYI = RI / 2;
  const BOT = TOP + H;

  const cutFace =
    `M ${CX - R} ${TOP} L ${CX - RI} ${TOP} L ${CX - RI} ${BOT} ` +
    `A ${RI} ${RYI} 0 0 0 ${CX + RI} ${BOT} L ${CX + RI} ${TOP} L ${CX + R} ${TOP} ` +
    `L ${CX + R} ${BOT + T} A ${R} ${RY} 0 0 1 ${CX - R} ${BOT + T} Z`;

  const backWall =
    `M ${CX - RI} ${TOP} A ${RI} ${RYI} 0 0 1 ${CX + RI} ${TOP} ` +
    `L ${CX + RI} ${BOT} A ${RI} ${RYI} 0 0 0 ${CX - RI} ${BOT} Z`;

  const rim =
    `M ${CX - R} ${TOP} A ${R} ${RY} 0 0 1 ${CX + R} ${TOP} ` +
    `L ${CX + RI} ${TOP} A ${RI} ${RYI} 0 0 0 ${CX - RI} ${TOP} Z`;

  const level = $derived(Math.max(0, Math.min(1, vm.levelPercent / 100)));
  const waterY = $derived(BOT - 18 - level * (H - 80));

  const waterBody = $derived(
    `M ${CX - RI} ${waterY} L ${CX - RI} ${BOT} ` +
      `A ${RI} ${RYI} 0 0 0 ${CX + RI} ${BOT} L ${CX + RI} ${waterY} ` +
      `A ${RI} ${RYI} 0 0 1 ${CX - RI} ${waterY} Z`,
  );

  const streamEnd = $derived(waterY + 4);

  const rnd = seeded(42);
  const stones: { x: number; y: number; r: number; k: number }[] = [];
  while (stones.length < 60) {
    const x = rnd() * 940;
    const y = 275 + rnd() * 420;
    const r = 4 + rnd() * 9;
    const k = rnd();
    if (x > CX - R - 20 && x < CX + R + 20) continue;
    stones.push({ x, y, r, k });
  }
  const tuftX = Array.from({ length: 20 }, (_, i) => 15 + i * 46);
</script>

<figure
  class="scene"
  class:animated
  role="img"
  aria-label="Przekrój zbiornika, poziom {Math.round(level * 100)} procent"
>
  <svg viewBox="0 0 940 720" class="tank-svg">
    <defs>
      <linearGradient id="sky" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0" style="stop-color: var(--c-sky)" />
        <stop offset="1" style="stop-color: #eef8ff" />
      </linearGradient>
      <linearGradient id="soil" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0" style="stop-color: var(--c-soil)" />
        <stop offset="1" style="stop-color: var(--c-soil-d)" />
      </linearGradient>
      <linearGradient id="deep" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0" style="stop-color: var(--c-water)" />
        <stop offset="1" style="stop-color: var(--c-water-deep)" />
      </linearGradient>
      <radialGradient id="surf" cx=".5" cy=".45" r=".75">
        <stop offset="0" stop-color="#b6ecff" />
        <stop offset=".55" style="stop-color: var(--c-water-surf)" />
        <stop offset="1" style="stop-color: var(--c-water)" />
      </radialGradient>
      <linearGradient id="wallIn" x1="0" y1="0" x2="1" y2="0">
        <stop offset="0" style="stop-color: var(--c-conc-d)" />
        <stop offset=".5" style="stop-color: var(--c-conc-in)" />
        <stop offset="1" style="stop-color: var(--c-conc-d)" />
      </linearGradient>
    </defs>

    <!-- sky + grass + soil -->
    <rect width="940" height="252" fill="url(#sky)" />
    <ellipse cx="150" cy="250" rx="260" ry="60" fill="#79b455" />
    <ellipse cx="800" cy="255" rx="320" ry="70" fill="#6aa848" />
    <rect y="218" width="940" height="36" fill="var(--c-grass)" />
    {#each tuftX as x}
      <path
        class="tuft"
        d="M {x} 252 q 3 -14 6 -16 M {x + 7} 252 q 0 -12 4 -18 M {x + 13} 252 q -2 -12 2 -16"
      />
    {/each}
    <rect y="252" width="940" height="468" fill="url(#soil)" />
    {#each stones as s}
      <ellipse
        cx={s.x}
        cy={s.y}
        rx={s.r}
        ry={s.r * 0.72}
        fill={s.k < 0.5 ? "#6b4a2f" : "#8a6844"}
        opacity={0.5 + s.k * 0.4}
      />
    {/each}

    <!-- tank cutaway + water -->
    <path d={cutFace} fill="var(--c-conc-face)" stroke="var(--c-conc-d)" stroke-width="2" />
    <path d={backWall} fill="url(#wallIn)" />
    <path d={waterBody} fill="url(#deep)" />
    <path d={waterBody} fill="var(--c-water)" opacity=".35" />

    <ellipse cx={CX} cy={waterY} rx={RI} ry={RYI} fill="url(#surf)" />
    {#each [0, -1.1, -2.2] as d}
      <ellipse
        class="ripple"
        cx="372"
        cy={waterY + 6}
        rx="52"
        ry="20"
        style="animation-delay: {d}s"
      />
    {/each}
    <ellipse cx={CX + 40} cy={waterY - 14} rx="60" ry="14" fill="#fff" opacity=".28" />
    <path d={rim} fill="var(--c-conc)" stroke="var(--c-conc-d)" stroke-width="2" />

    <!-- level readout: value + unit "cm" only -->
    <text x={CX} y={waterY - 18} text-anchor="middle" font-size="28" font-weight="800" fill="#132352">
      {vm.levelCm.toFixed(0)} cm
    </text>

    <!-- inlet pipe only -->
    <g stroke-linecap="round">
      <line x1="60" y1="70" x2="60" y2="152" stroke="var(--c-pipe-d)" stroke-width="34" />
      <line x1="60" y1="70" x2="60" y2="150" stroke="var(--c-pipe)" stroke-width="28" />
      <line x1="60" y1="150" x2="370" y2="272" stroke="var(--c-pipe-d)" stroke-width="34" />
      <line x1="62" y1="150" x2="368" y2="270" stroke="var(--c-pipe)" stroke-width="28" />
      <ellipse
        cx="371"
        cy="272"
        rx="9"
        ry="15.5"
        fill="#2b3a44"
        transform="rotate(24 371 272)"
      />
    </g>
    <line
      class="stream"
      class:running={vm.simEn}
      x1="372"
      y1="284"
      x2="372"
      y2={streamEnd}
      opacity={vm.simEn ? 1 : 0.35}
    />
    <line
      class="stream-core"
      class:running={vm.simEn}
      x1="372"
      y1="284"
      x2="372"
      y2={streamEnd - 2}
      opacity={vm.simEn ? 1 : 0.25}
    />
  </svg>
</figure>

<style>
  .scene {
    --c-sky: #cfe8f7;
    --c-grass: #5da33a;
    --c-grass-d: #3f7d26;
    --c-soil: #7c5433;
    --c-soil-d: #4e3018;
    --c-conc: #c9c9c2;
    --c-conc-d: #8f8f88;
    --c-conc-face: #b4b4ac;
    --c-conc-in: #a7a79e;
    --c-water: #39b7e6;
    --c-water-deep: #0e5f9e;
    --c-water-surf: #7fd8f5;
    --c-pipe: #b9bec4;
    --c-pipe-d: #7c838b;

    margin: 0;
    width: 100%;
  }
  .tank-svg {
    width: 100%;
    height: auto;
    display: block;
    border-radius: 12px;
    background: #eef8ff;
  }

  .tuft {
    stroke: var(--c-grass-d);
    stroke-width: 2.5;
    fill: none;
    stroke-linecap: round;
    opacity: 0.8;
  }

  .ripple {
    fill: none;
    stroke: #eafaff;
    stroke-width: 2.5;
    opacity: 0;
    transform-box: fill-box;
    transform-origin: center;
  }
  .stream,
  .stream-core {
    stroke-linecap: round;
  }
  .stream {
    stroke: #bdeeff;
    stroke-width: 9;
    stroke-dasharray: 5 13;
  }
  .stream-core {
    stroke: #fff;
    stroke-width: 3;
    stroke-dasharray: 4 14;
  }

  .animated .ripple {
    animation: ripple 3.4s linear infinite;
  }
  .animated .stream.running,
  .animated .stream-core.running {
    animation: fall 0.45s linear infinite;
  }

  @keyframes ripple {
    from {
      transform: scale(0.12);
      opacity: 0.95;
    }
    to {
      transform: scale(1);
      opacity: 0;
    }
  }
  @keyframes fall {
    to {
      stroke-dashoffset: -18;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .ripple,
    .stream,
    .stream-core {
      animation: none !important;
    }
  }
</style>

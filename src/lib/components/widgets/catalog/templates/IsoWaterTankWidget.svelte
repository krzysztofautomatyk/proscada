<script lang="ts">
  import type { WidgetDef, TagValue } from "$lib/types";
  import { formatTrustedValue, resolveTagQuality } from "../../shared/quality";

  interface Props {
    widget: WidgetDef;
    tag?: TagValue | null;
    design?: boolean;
    onWrite?: (tagId: string, value: number) => void;
  }

  let { widget, tag = null, design = false }: Props = $props();

  const quality = $derived(resolveTagQuality(widget, tag, design));
  // Preserve last valid level reading or current value even when quality is degraded, so visualization graphics remain intact.
  const levelCm = $derived.by(() => {
    if (tag?.value !== undefined && tag.value !== null) return tag.value;
    if (typeof quality.lastValidValue === "number") return quality.lastValidValue;
    return design ? 500 : 0;
  });
  const levelLabel = $derived(formatTrustedValue(quality, levelCm, 0, { showLastKnown: true }));
  const levelPct = $derived(
    levelCm === null ? 0 : Math.max(0, Math.min(1, levelCm / 1000)),
  );

  const CX = 200;
  const TOP = 90;
  const R = 110;
  const T = 14;
  const H = 180;
  const RI = R - T;
  const RY = R / 2;
  const RYI = RI / 2;
  const BOT = TOP + H;

  const cutFace = $derived(
    `M ${CX - R} ${TOP} L ${CX - RI} ${TOP} L ${CX - RI} ${BOT} ` +
    `A ${RI} ${RYI} 0 0 0 ${CX + RI} ${BOT} L ${CX + RI} ${TOP} L ${CX + R} ${TOP} ` +
    `L ${CX + R} ${BOT + T} A ${R} ${RY} 0 0 1 ${CX - R} ${BOT + T} Z`
  );

  const backWall = $derived(
    `M ${CX - RI} ${TOP} A ${RI} ${RYI} 0 0 1 ${CX + RI} ${TOP} ` +
    `L ${CX + RI} ${BOT} A ${RI} ${RYI} 0 0 0 ${CX - RI} ${BOT} Z`
  );

  const rim = $derived(
    `M ${CX - R} ${TOP} A ${R} ${RY} 0 0 1 ${CX + R} ${TOP} ` +
    `L ${CX + RI} ${TOP} A ${RI} ${RYI} 0 0 0 ${CX - RI} ${TOP} Z`
  );

  const waterY = $derived(BOT - 12 - levelPct * (H - 50));

  const waterBody = $derived(
    `M ${CX - RI} ${waterY} L ${CX - RI} ${BOT} ` +
    `A ${RI} ${RYI} 0 0 0 ${CX + RI} ${BOT} L ${CX + RI} ${waterY} ` +
    `A ${RI} ${RYI} 0 0 1 ${CX - RI} ${waterY} Z`
  );
</script>

<div class="iso-tank-wrap">
  <svg viewBox="0 0 400 340" class="iso-svg">
    <defs>
      <linearGradient id="tankDeep" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0" stop-color="#39b7e6" />
        <stop offset="1" stop-color="#0e5f9e" />
      </linearGradient>
      <radialGradient id="tankSurf" cx=".5" cy=".45" r=".75">
        <stop offset="0" stop-color="#b6ecff" />
        <stop offset=".55" stop-color="#7fd8f5" />
        <stop offset="1" stop-color="#39b7e6" />
      </radialGradient>
      <linearGradient id="tankWallIn" x1="0" y1="0" x2="1" y2="0">
        <stop offset="0" stop-color="#8f8f88" />
        <stop offset=".5" stop-color="#a7a79e" />
        <stop offset="1" stop-color="#8f8f88" />
      </linearGradient>
    </defs>

    <!-- Outer & Inner Walls -->
    <path d={cutFace} fill="#b4b4ac" stroke="#8f8f88" stroke-width="2" />
    <path d={backWall} fill="url(#tankWallIn)" />

    <!-- Dynamic Liquid -->
    <path d={waterBody} fill="url(#tankDeep)" />
    <ellipse cx={CX} cy={waterY} rx={RI} ry={RYI} fill="url(#tankSurf)" />

    <!-- Top Rim -->
    <path d={rim} fill="#c9c9c2" stroke="#8f8f88" stroke-width="2" />

    <!-- Digital Level Display -->
    <text x={CX} y={waterY - 14} text-anchor="middle" font-size="22" font-weight="800" fill="#132352">
      {levelLabel} cm
    </text>
  </svg>
</div>

<style>
  .iso-tank-wrap {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #eef8ff;
    border-radius: 8px;
    box-sizing: border-box;
    overflow: hidden;
  }
  .iso-svg {
    width: 100%;
    height: 100%;
    display: block;
  }
</style>

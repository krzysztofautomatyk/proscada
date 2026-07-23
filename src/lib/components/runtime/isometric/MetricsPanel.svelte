<script lang="ts">
  import type { WaterTankViewModel } from "$lib/stores/waterTank.svelte";
  import { levelAccent, pumpColor } from "$lib/stores/waterTank.svelte";

  interface Props {
    vm: WaterTankViewModel;
  }
  let { vm }: Props = $props();
</script>

<div class="metrics" role="region" aria-label="Process metrics">
  <div class="kpi">
    <span class="l">Level</span>
    <span class="v" style:color={levelAccent(vm.levelState)}>
      {vm.levelCm.toFixed(0)} <small>cm</small>
    </span>
    <div class="track">
      <i style:width="{vm.levelPercent}%" style:background={levelAccent(vm.levelState)}></i>
    </div>
    <span class="m">
      <i class="q {vm.qualityLevel}"></i>
      {vm.levelState.toUpperCase()} · {vm.levelPercent.toFixed(0)}%
    </span>
  </div>
  <div class="kpi">
    <span class="l">Pump 1 (submersible)</span>
    <span class="v" style:color={pumpColor(vm.p1Run, vm.p1Fault)}>
      {vm.p1Fault ? "FAULT" : vm.p1Run ? "RUN" : "STOP"}
    </span>
    <span class="m">OK {vm.p1Ok ? "YES" : "NO"} · starts {vm.p1Starts.toFixed(0)} · {vm.p1Hours.toFixed(0)} h</span>
  </div>
  <div class="kpi">
    <span class="l">Pump 2 (surface)</span>
    <span class="v" style:color={pumpColor(vm.p2Run, vm.p2Fault)}>
      {vm.p2Fault ? "FAULT" : vm.p2Run ? "RUN" : "STOP"}
    </span>
    <span class="m">OK {vm.p2Ok ? "YES" : "NO"} · starts {vm.p2Starts.toFixed(0)} · {vm.p2Hours.toFixed(0)} h</span>
  </div>
  <div class="kpi">
    <span class="l">Station</span>
    <span class="v" style:color={vm.stationColor}>{vm.stationLabel}</span>
    <span class="m">DEMAND {vm.demand ? "●" : "○"} · JOIN {vm.joinP2 ? "●" : "○"} · DRAIN {vm.drain ? "●" : "○"}</span>
  </div>
</div>

<style>
  .metrics {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 10px;
  }
  .kpi {
    background: var(--scada-panel, #fff);
    border: 1px solid var(--scada-border, #e5e7eb);
    border-radius: 10px;
    padding: 12px 14px;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.04);
  }
  .l {
    display: block;
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--scada-muted, #6b7280);
  }
  .v {
    display: block;
    font-size: 22px;
    font-weight: 800;
    margin: 4px 0 6px;
  }
  .v small {
    font-size: 12px;
    color: #9ca3af;
    font-weight: 600;
  }
  .track {
    height: 5px;
    background: var(--scada-track, #e5e7eb);
    border-radius: 99px;
    overflow: hidden;
  }
  .track i {
    display: block;
    height: 100%;
    border-radius: 99px;
    transition: width 0.25s ease;
  }
  .m {
    display: block;
    margin-top: 6px;
    font-size: 11px;
    color: var(--scada-muted, #6b7280);
  }
  .q {
    display: inline-block;
    width: 7px;
    height: 7px;
    border-radius: 50%;
    margin-right: 4px;
    background: #9ca3af;
  }
  .q.good {
    background: #16a34a;
  }
  .q.uncertain {
    background: #eab308;
  }
  .q.bad {
    background: #dc2626;
  }
  @media (max-width: 1100px) {
    .metrics {
      grid-template-columns: 1fr 1fr;
    }
  }
</style>

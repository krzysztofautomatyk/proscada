<script lang="ts">
  import type { WidgetDef, TagValue } from "$lib/types";
  import DynamicShell from "./DynamicShell.svelte";

  import LabelWidget from "./catalog/LabelWidget.svelte";
  import NumericWidget from "./catalog/NumericWidget.svelte";
  import LampWidget from "./catalog/LampWidget.svelte";
  import TankWidget from "./catalog/TankWidget.svelte";
  import BarWidget from "./catalog/BarWidget.svelte";
  import PanelWidget from "./catalog/PanelWidget.svelte";
  import WriteButtonWidget from "./catalog/WriteButtonWidget.svelte";
  import IsoWaterTankWidget from "./catalog/IsoWaterTankWidget.svelte";
  import IsoPumpWidget from "./catalog/IsoPumpWidget.svelte";
  import IsoPipeWidget from "./catalog/IsoPipeWidget.svelte";
  import IsoTerrainWidget from "./catalog/IsoTerrainWidget.svelte";
  import MetricsPanelWidget from "./catalog/MetricsPanelWidget.svelte";
  import StatusBadgeWidget from "./catalog/StatusBadgeWidget.svelte";
  import AlarmPanelWidget from "./catalog/AlarmPanelWidget.svelte";
  import SetpointControlWidget from "./catalog/SetpointControlWidget.svelte";
  import InflowControlWidget from "./catalog/InflowControlWidget.svelte";
  import ProcessControlWidget from "./catalog/ProcessControlWidget.svelte";
  import ShapeWidget from "./catalog/ShapeWidget.svelte";
  import LineWidget from "./catalog/LineWidget.svelte";
  import NumericInputWidget from "./catalog/NumericInputWidget.svelte";
  import BoolDisplayWidget from "./catalog/BoolDisplayWidget.svelte";
  import EmbeddedScreenWidget from "./catalog/EmbeddedScreenWidget.svelte";
  import ImageWidget from "./catalog/ImageWidget.svelte";
  import { tagMap } from "$lib/stores/app";

  interface Props {
    widget: WidgetDef;
    tag?: TagValue | null;
    design?: boolean;
    onWrite?: (tagId: string, value: number) => void;
    ancestorFormIds?: Set<string>;
  }

  let {
    widget,
    tag = null,
    design = false,
    onWrite,
    ancestorFormIds = new Set<string>(),
  }: Props = $props();
</script>

<!-- Generic blink + visibility + lock badges for EVERY control -->
<DynamicShell {widget} {design}>
  {#if widget.widget_type === "label"}
    <LabelWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "numeric"}
    <NumericWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "lamp"}
    <LampWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "tank"}
    <TankWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "bar"}
    <BarWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "panel"}
    <PanelWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "write_button"}
    <WriteButtonWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "iso_water_tank"}
    <IsoWaterTankWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "iso_pump"}
    <IsoPumpWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "iso_pipe"}
    <IsoPipeWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "iso_terrain"}
    <IsoTerrainWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "metrics_panel"}
    <MetricsPanelWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "status_badge"}
    <StatusBadgeWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "alarm_panel"}
    <AlarmPanelWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "setpoint_control"}
    <SetpointControlWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "inflow_control"}
    <InflowControlWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "process_control"}
    <ProcessControlWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "shape"}
    <ShapeWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "line"}
    <LineWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "numeric_input"}
    <NumericInputWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "bool_display"}
    <BoolDisplayWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "image"}
    <ImageWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "embedded_screen"}
    <EmbeddedScreenWidget
      {widget}
      tagMap={$tagMap}
      {design}
      {onWrite}
      {ancestorFormIds}
    />
  {:else}
    <div class="w-chrome-unknown">Unknown widget: {widget.widget_type}</div>
  {/if}
</DynamicShell>

<style>
  .w-chrome-unknown {
    width: 100%;
    height: 100%;
    background: #fef2f2;
    border: 1px dashed #ef4444;
    color: #b91c1c;
    font-size: 11px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 4px;
    box-sizing: border-box;
  }
</style>

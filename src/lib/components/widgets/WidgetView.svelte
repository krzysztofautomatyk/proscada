<script lang="ts">
  import type { WidgetDef, TagValue } from "$lib/types";

  // Individual component files from catalog
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
  import NumericInputWidget from "./catalog/NumericInputWidget.svelte";
  import BoolDisplayWidget from "./catalog/BoolDisplayWidget.svelte";

  interface Props {
    widget: WidgetDef;
    tag?: TagValue | null;
    design?: boolean;
    onWrite?: (tagId: string, value: number) => void;
  }

  let { widget, tag = null, design = false, onWrite }: Props = $props();
</script>

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
{:else if widget.widget_type === "numeric_input"}
  <NumericInputWidget {widget} {tag} {design} {onWrite} />
{:else if widget.widget_type === "bool_display"}
  <BoolDisplayWidget {widget} {tag} {design} {onWrite} />
{:else}
  <div class="w-chrome-unknown">
    Unknown widget: {widget.widget_type}
  </div>
{/if}

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

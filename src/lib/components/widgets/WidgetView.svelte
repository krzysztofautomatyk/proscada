<script lang="ts">
  import type { WidgetDef, TagValue } from "$lib/types";
  import DynamicShell from "./DynamicShell.svelte";

  import TextWidget from "./catalog/primitives/TextWidget.svelte";
  import RectangleWidget from "./catalog/primitives/RectangleWidget.svelte";
  import ConnectorWidget from "./catalog/primitives/ConnectorWidget.svelte";
  import ImageWidget from "./catalog/assets/ImageWidget.svelte";
  import VectorSymbolWidget from "./catalog/assets/VectorSymbolWidget.svelte";
  import NumericValueWidget from "./catalog/indicators/NumericValueWidget.svelte";
  import LegacyLampWidget from "./catalog/indicators/LegacyLampWidget.svelte";
  import LegacyBoolDisplayWidget from "./catalog/indicators/LegacyBoolDisplayWidget.svelte";
  import LegacyStatusBadgeWidget from "./catalog/indicators/LegacyStatusBadgeWidget.svelte";
  import LegacyBarWidget from "./catalog/indicators/LegacyBarWidget.svelte";
  import StateIndicatorWidget from "./catalog/indicators/StateIndicatorWidget.svelte";
  import MeterWidget from "./catalog/indicators/MeterWidget.svelte";
  import DataStatusWidget from "./catalog/indicators/DataStatusWidget.svelte";
  import PanelWidget from "./catalog/layout/PanelWidget.svelte";
  import DisclosurePanelWidget from "./catalog/layout/DisclosurePanelWidget.svelte";
  import ScreenEmbedWidget from "./catalog/navigation/ScreenEmbedWidget.svelte";
  import LegacyWriteButtonWidget from "./catalog/commands/LegacyWriteButtonWidget.svelte";
  import CommandButtonWidget from "./catalog/commands/CommandButtonWidget.svelte";
  import NumericInputWidget from "./catalog/inputs/NumericInputWidget.svelte";
  import BooleanInputWidget from "./catalog/inputs/BooleanInputWidget.svelte";
  import SelectInputWidget from "./catalog/inputs/SelectInputWidget.svelte";
  import TextInputWidget from "./catalog/inputs/TextInputWidget.svelte";
  import DateTimeRangeWidget from "./catalog/inputs/DateTimeRangeWidget.svelte";
  import TrendWidget from "./catalog/data/TrendWidget.svelte";
  import CollectionViewWidget from "./catalog/data/CollectionViewWidget.svelte";
  import EventTimelineWidget from "./catalog/data/EventTimelineWidget.svelte";
  import EventAuditViewerWidget from "./catalog/data/EventAuditViewerWidget.svelte";
  import NavigationLinkWidget from "./catalog/navigation/NavigationLinkWidget.svelte";
  import TabSetWidget from "./catalog/navigation/TabSetWidget.svelte";
  import NavigationMenuWidget from "./catalog/navigation/NavigationMenuWidget.svelte";
  import BreadcrumbWidget from "./catalog/navigation/BreadcrumbWidget.svelte";
  import AlarmPanelWidget from "./catalog/alarms/AlarmPanelWidget.svelte";
  import AlarmBannerWidget from "./catalog/alarms/AlarmBannerWidget.svelte";
  import AlarmIndicatorWidget from "./catalog/alarms/AlarmIndicatorWidget.svelte";
  import DialogWidget from "./catalog/feedback/DialogWidget.svelte";
  import NotificationWidget from "./catalog/feedback/NotificationWidget.svelte";
  import TooltipWidget from "./catalog/feedback/TooltipWidget.svelte";
  import ProcessSymbolWidget from "./catalog/process/ProcessSymbolWidget.svelte";
  import FaceplateWidget from "./catalog/process/FaceplateWidget.svelte";
  import TankLevelTemplateWidget from "./catalog/templates/TankLevelTemplateWidget.svelte";
  import IsoWaterTankWidget from "./catalog/templates/IsoWaterTankWidget.svelte";
  import IsoPumpWidget from "./catalog/templates/IsoPumpWidget.svelte";
  import IsoPipeWidget from "./catalog/templates/IsoPipeWidget.svelte";
  import IsoTerrainWidget from "./catalog/templates/IsoTerrainWidget.svelte";
  import MetricsPanelWidget from "./catalog/templates/MetricsPanelWidget.svelte";
  import SetpointControlWidget from "./catalog/templates/SetpointControlWidget.svelte";
  import InflowControlWidget from "./catalog/templates/InflowControlWidget.svelte";
  import ProcessControlWidget from "./catalog/templates/ProcessControlWidget.svelte";
  import QRCodeWidget from "./catalog/utilities/QRCodeWidget.svelte";
  import { snapshot, tagMap } from "$lib/stores/app";

  interface Props {
    widget: WidgetDef;
    tag?: TagValue | null;
    design?: boolean;
    onWrite?: (tagId: string, value: number | string) => void;
    ancestorFormIds?: Set<string>;
  }

  let {
    widget,
    tag = null,
    design = false,
    onWrite,
    ancestorFormIds = new Set<string>(),
  }: Props = $props();

  const alarmWidget = $derived({
    ...widget,
    config: {
      ...(widget.config ?? {}),
      alarms: ($snapshot?.alarms ?? []).map((alarm) => ({
        id: alarm.def_id,
        time: alarm.last_change,
        priority: alarm.priority,
        state: alarm.state,
        message: alarm.message,
        group: alarm.group_id || "Ungrouped",
        shelved: false,
      })),
    },
  });
</script>

<!-- Generic blink + visibility + lock badges for EVERY control -->
<DynamicShell {widget} {design}>
  {#if widget.widget_type === "label"}
    <TextWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "numeric"}
    <NumericValueWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "lamp"}
    <LegacyLampWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "tank"}
    <TankLevelTemplateWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "bar"}
    <LegacyBarWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "panel"}
    <PanelWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "write_button"}
    <LegacyWriteButtonWidget {widget} {tag} {design} {onWrite} />
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
    <LegacyStatusBadgeWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "alarm_panel"}
    <AlarmPanelWidget widget={alarmWidget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "setpoint_control"}
    <SetpointControlWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "inflow_control"}
    <InflowControlWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "process_control"}
    <ProcessControlWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "shape"}
    <RectangleWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "line"}
    <ConnectorWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "numeric_input"}
    <NumericInputWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "bool_display"}
    <LegacyBoolDisplayWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "image"}
    <ImageWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "embedded_screen"}
    <ScreenEmbedWidget
      {widget}
      tagMap={$tagMap}
      {design}
      {onWrite}
      {ancestorFormIds}
    />
  {:else if widget.widget_type === "vector_symbol"}
    <VectorSymbolWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "state_indicator"}
    <StateIndicatorWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "meter"}
    <MeterWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "data_status"}
    <DataStatusWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "disclosure_panel"}
    <DisclosurePanelWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "qr_code"}
    <QRCodeWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "command_button"}
    <CommandButtonWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "boolean_input"}
    <BooleanInputWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "select_input"}
    <SelectInputWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "text_input"}
    <TextInputWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "datetime_range"}
    <DateTimeRangeWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "alarm_banner"}
    <AlarmBannerWidget widget={alarmWidget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "alarm_indicator"}
    <AlarmIndicatorWidget widget={alarmWidget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "dialog"}
    <DialogWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "notification"}
    <NotificationWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "tooltip"}
    <TooltipWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "process_symbol"}
    <ProcessSymbolWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "faceplate"}
    <FaceplateWidget widget={alarmWidget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "trend"}
    <TrendWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "collection_view"}
    <CollectionViewWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "event_timeline"}
    <EventTimelineWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "event_audit_viewer"}
    <EventAuditViewerWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "navigation_link"}
    <NavigationLinkWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "tab_set"}
    <TabSetWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "navigation_menu"}
    <NavigationMenuWidget {widget} {tag} {design} {onWrite} />
  {:else if widget.widget_type === "breadcrumb"}
    <BreadcrumbWidget {widget} {tag} {design} {onWrite} />
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

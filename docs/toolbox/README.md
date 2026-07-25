# Toolbox — 45/45 pozycji

Toolbox zawiera 35 typów kanonicznych i 10 szablonów procesu. Każda z 45 widocznych pozycji ma osobny dokument; każdy typ kanoniczny ma także osobny renderer Svelte.

## Prymitywy

| ID | Typ | Dokument |
|---|---|---|
| PRIM-TEXT | `label` | [Text / Label](controls/primitives/label.md) |
| PRIM-RECT | `shape` | [Rectangle / Shape](controls/primitives/shape.md) |
| PRIM-CONNECTOR | `line` | [Line / Connector](controls/primitives/line.md) |

## Zasoby

| ID | Typ | Dokument |
|---|---|---|
| AST-IMAGE | `image` | [Image](controls/assets/image.md) |
| AST-SYMBOL | `vector_symbol` | [Vector Symbol](controls/assets/vector_symbol.md) |

## Wskaźniki

| ID | Typ | Dokument |
|---|---|---|
| IND-STATE | `state_indicator` | [State Indicator](controls/indicators/state_indicator.md) |
| IND-VALUE | `numeric` | [Numeric Value](controls/indicators/numeric.md) |
| IND-METER | `meter` | [Meter](controls/indicators/meter.md) |
| IND-DATASTATUS | `data_status` | [Data Status Indicator](controls/indicators/data_status.md) |

## Wizualizacja procesu

| ID | Typ | Dokument |
|---|---|---|
| PROC-SYMBOL | `process_symbol` | [Process Symbol](controls/process/process_symbol.md) |
| PROC-FACEPLATE | `faceplate` | [Faceplate](controls/process/faceplate.md) |

## Sterowanie i wejścia

| ID | Typ | Dokument |
|---|---|---|
| CMD-BUTTON | `command_button` | [Command Button](controls/commands/command_button.md) |
| INP-NUMERIC | `numeric_input` | [Numeric Input](controls/inputs/numeric_input.md) |
| INP-BOOLEAN | `boolean_input` | [Boolean Input](controls/inputs/boolean_input.md) |
| INP-SELECT | `select_input` | [Select Input](controls/inputs/select_input.md) |
| INP-TEXT | `text_input` | [Text Input](controls/inputs/text_input.md) |
| INP-DATETIME-RANGE | `datetime_range` | [Date-Time Range Picker](controls/inputs/datetime_range.md) |

## Dane i historia

| ID | Typ | Dokument |
|---|---|---|
| VIS-TREND | `trend` | [Time-series Trend](controls/data/trend.md) |
| DATA-COLLECTION | `collection_view` | [Collection View](controls/data/collection_view.md) |
| VIS-TIMELINE | `event_timeline` | [Event Timeline](controls/data/event_timeline.md) |
| DATA-EVENTLOG | `event_audit_viewer` | [Event / Audit Viewer](controls/data/event_audit_viewer.md) |

## Układ i nawigacja

| ID | Typ | Dokument |
|---|---|---|
| LAY-PANEL | `panel` | [Panel / Container](controls/layout/panel.md) |
| LAY-DISCLOSURE | `disclosure_panel` | [Disclosure Panel](controls/layout/disclosure_panel.md) |
| NAV-LINK | `navigation_link` | [Navigation Link](controls/navigation/navigation_link.md) |
| NAV-TABS | `tab_set` | [Tab Set](controls/navigation/tab_set.md) |
| NAV-MENU | `navigation_menu` | [Navigation Menu](controls/navigation/navigation_menu.md) |
| NAV-EMBED | `embedded_screen` | [Screen Embed](controls/navigation/embedded_screen.md) |
| NAV-BREADCRUMB | `breadcrumb` | [Breadcrumb](controls/navigation/breadcrumb.md) |

## Informacja i alarmy

| ID | Typ | Dokument |
|---|---|---|
| OVL-DIALOG | `dialog` | [Dialog](controls/feedback/dialog.md) |
| FDBK-NOTIFICATION | `notification` | [Notification](controls/feedback/notification.md) |
| FDBK-TOOLTIP | `tooltip` | [Tooltip](controls/feedback/tooltip.md) |
| ALM-PANEL | `alarm_panel` | [Alarm Panel](controls/alarms/alarm_panel.md) |
| ALM-BANNER | `alarm_banner` | [Alarm Banner](controls/alarms/alarm_banner.md) |
| ALM-INDICATOR | `alarm_indicator` | [Alarm Indicator](controls/alarms/alarm_indicator.md) |

## Narzędzia

| ID | Typ | Dokument |
|---|---|---|
| UTIL-QRCODE | `qr_code` | [QR Code](controls/utilities/qr_code.md) |

## Szablony procesu

| ID | Typ | Dokument |
|---|---|---|
| TPL-TANK-LEVEL | `tank` | [2D Tank Level](templates/tank.md) |
| TPL-BAR | `bar` | [Legacy Bar Graph](templates/bar.md) |
| TPL-WATER-TANK | `iso_water_tank` | [Iso Water Tank](templates/iso_water_tank.md) |
| TPL-PUMP | `iso_pump` | [Iso Pump](templates/iso_pump.md) |
| TPL-PIPE | `iso_pipe` | [Iso Pipe Segment](templates/iso_pipe.md) |
| TPL-TERRAIN | `iso_terrain` | [Iso Terrain Cutaway](templates/iso_terrain.md) |
| TPL-SETPOINTS | `setpoint_control` | [Setpoints Controller](templates/setpoint_control.md) |
| TPL-INFLOW | `inflow_control` | [Inflow K Controller](templates/inflow_control.md) |
| TPL-FREEZE | `process_control` | [Process Freeze Controller](templates/process_control.md) |
| TPL-METRICS | `metrics_panel` | [Metrics Overview Bar](templates/metrics_panel.md) |

## Dokumentacja generyczna

- [Registry i Factory](generic/registry-factory.md)
- [Właściwości wspólne](generic/common-properties.md)
- [Binding i jakość](generic/binding-quality.md)
- [Style i fonty](generic/styles-fonts.md)
- [Dynamika i zdarzenia](generic/dynamics-events.md)
- [Definition of Done](generic/definition-of-done.md)

## Bramka kompletności

```powershell
npm run validate:widgets
npm run validate:docs
```

Pierwsza komenda sprawdza 35 rendererów kanonicznych. Druga wylicza z Registry wszystkie 45 pozycji Toolboxa i wymaga osobnego pliku dla każdej z nich.

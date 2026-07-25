# Alarm Panel

| Pole | Wartość |
|---|---|
| ID | `ALM-PANEL` |
| Typ | `alarm_panel` |
| Plik | `catalog/alarms/AlarmPanelWidget.svelte` |

Pokazuje centralne instancje alarmów.

## Konfiguracja

`alarms` jest tablicą instancji z `EngineSnapshot`. Filtry stanu, priorytetu, grupy i kolejności są stanem lokalnym widoku.

## Funkcje

- filtrowanie stanu, priorytetu i grupy;
- sortowanie priorytet/czas;
- tekst, ikona i wzór stanu;
- żądanie ACK;
- widoczny shelved overlay.

## Dane

`WidgetView` wstrzykuje alarmy z `EngineSnapshot`. Progi i lifecycle pozostają w Rust engine.

## ACK

Przycisk emituje `proscada:alarm-action`; App wywołuje backend `ack_alarm`.

## Ograniczenia

Panel nie definiuje progów, deadband ani lifecycle. Shelved jest wyświetlane, ale pełny workflow shelving/OOS nie istnieje jeszcze w backendzie.

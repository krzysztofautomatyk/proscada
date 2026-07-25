# Alarm Banner

| Pole | Wartość |
|---|---|
| ID | `ALM-BANNER` |
| Typ | `alarm_banner` |
| Plik | `catalog/alarms/AlarmBannerWidget.svelte` |

Pokazuje najwyższy aktywny niepotwierdzony alarm.

## Konfiguracja

`alarms` jest tablicą instancji wstrzykiwaną przez `WidgetView`. W Designerze brak danych daje stan normalny lub dane podglądowe zależnie od źródła.

## Funkcje

- wybór worst priority;
- liczba active unacked;
- komunikat i grupa;
- VIEW;
- ACK.

## Stan normalny

Gdy nie ma active unacked, banner pokazuje neutralne `NORMAL`.

## Zdarzenia

VIEW i ACK emitują `proscada:alarm-action`. ACK nie zmienia lokalnie źródłowego stanu.

## Ograniczenia

Banner pokazuje tylko najgorszy `active_unacked`. Pełną listę, historię i pozostałe stany prezentuje Alarm Panel.

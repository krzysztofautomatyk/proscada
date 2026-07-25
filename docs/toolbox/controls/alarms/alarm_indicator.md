# Alarm Indicator

| Pole | Wartość |
|---|---|
| ID | `ALM-INDICATOR` |
| Typ | `alarm_indicator` |
| Plik | `catalog/alarms/AlarmIndicatorWidget.svelte` |

Agreguje stan alarmów dla grupy obiektowej.

## Najważniejsze pola

`group`.

## Roll-up

Kontrolka filtruje alarmy grupy, liczy aktywne i wybiera najgorszy priorytet. Stan jest prezentowany ikoną, tekstem i wzorem.

## Interakcja

Kliknięcie w Runtime emituje zdarzenie szczegółu alarmowego. Designer nie wykonuje operacji.

## Zastosowanie

Umieść na kaflach overview, menu lub mapie obiektów.

## Ograniczenia

Filtrowanie używa dokładnego ID grupy albo `All groups`; agregacja poddrzewa wymaga przygotowania odpowiednich danych przez backend.

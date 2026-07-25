# Data Status Indicator

| Pole | Wartość |
|---|---|
| ID | `IND-DATASTATUS` |
| Typ | `data_status` |
| Plik | `catalog/indicators/DataStatusWidget.svelte` |

Pokazuje stan źródła danych i środowiska.

## Najważniejsze pola

`environment`, `staleAfterMs`.

## Stany

- LIVE;
- SIMULATION;
- UNCERTAIN;
- STALE DATA;
- BAD QUALITY;
- DISCONNECTED.

## Dane

Stan wykorzystuje `tag.quality` i `tag.age_ms`. Simulation jest jawnym stanem niezależnym od jakości.

## Zasady

Umieść wskaźnik na ekranach operatorskich, szczególnie gdy last-good może pozostać widoczne.


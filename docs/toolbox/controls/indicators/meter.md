# Meter

| Pole | Wartość |
|---|---|
| ID | `IND-METER` |
| Typ | `meter` |
| Plik | `catalog/indicators/MeterWidget.svelte` |

Prezentuje wartość względem zakresu.

## Warianty

`bar`, `vertical`, `gauge`.

## Najważniejsze pola

`variant`, `title`, `min`, `max`, `value`, `unit`, `decimals`, `warningAt`, `alarmAt`, `fillColor`.

## Dane

Tag ma pierwszeństwo przed wartością konfiguracyjną. Procent jest ograniczany do `0..100`.

## Quality i progi

Warning i alarm zmieniają semantyczny kolor. Quality Bad ma pierwszeństwo i pokazuje wzór błędu.


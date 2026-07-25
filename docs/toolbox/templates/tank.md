# 2D Tank Level

| Pole | Wartość |
|---|---|
| ID | `TPL-TANK-LEVEL` |
| Typ | `tank` |
| Plik | `catalog/templates/TankLevelTemplateWidget.svelte` |

Gotowy wskaźnik poziomu zbiornika używany przez projekt Water Tank.

## Najważniejsze pola

`title`, `min`, `max`, `unit`, `fillColor`, `warn`, `alarm`, `showValue`, `bgColor`.

## Dane

Bieżąca wartość taga jest skalowana do wysokości wypełnienia. Progi warn i alarm zmieniają semantyczny stan.

## Status

To szablon kompatybilności. Dla nowych ekranów można złożyć podobny widok z Numeric Value, Meter i Process Symbol.


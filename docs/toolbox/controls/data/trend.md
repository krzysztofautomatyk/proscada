# Time-series Trend

| Pole | Wartość |
|---|---|
| ID | `VIS-TREND` |
| Typ | `trend` |
| Plik | `catalog/data/TrendWidget.svelte` |

Rysuje lekki wykres SVG bez dodatkowej biblioteki wizualizacyjnej.

## Najważniejsze pola

`title`, `seriesLabel`, `points`, `min`, `max`, `unit`, `liveAppend`.

`points` przyjmuje listę liczb lub CSV. Tokeny `null`, `_` i `gap` tworzą przerwę jakości.

## Dane live

Przy `liveAppend=true` bieżąca wartość taga jest dołączana do danych podglądu.

## Quality

Bad quality nie powinna być łączona ciągłą linią. Renderer pokazuje przerwy i markery jakości.


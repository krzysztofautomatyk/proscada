# Date-Time Range Picker

| Pole | Wartość |
|---|---|
| ID | `INP-DATETIME-RANGE` |
| Typ | `datetime_range` |
| Plik | `catalog/inputs/DateTimeRangeWidget.svelte` |

Definiuje lokalny zakres czasu dla trendów, historii i eksportu.

## Warianty

`absolute`, `relative`.

## Najważniejsze pola

`title`, `timezone`, `from`, `to`, `presets`.

Presety względne używają formatu `15m`, `1h`, `8h`, `24h`.

## Walidacja

Daty muszą być poprawne, a From nie może być późniejsze niż To.

## Zapis

Kontrolka jest filtrem lokalnym i nie zapisuje do taga.

## Ograniczenia

Wybrany zakres nie uruchamia samodzielnie zapytania historian; konsument musi obsłużyć filtr.

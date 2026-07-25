# Boolean Input

| Pole | Wartość |
|---|---|
| ID | `INP-BOOLEAN` |
| Typ | `boolean_input` |
| Plik | `catalog/inputs/BooleanInputWidget.svelte` |

Edytuje wartość logiczną.

## Warianty

`checkbox`, `switch`.

## Najważniejsze pola

`title`, `trueLabel`, `falseLabel`, `indeterminateLabel`, `confirm`, `confirmText`, `disabledWhenBad`.

## Stany

Brak taga lub quality inna niż Good daje stan indeterminate. Zapis używa wartości `1` albo `0`.

## Zasady

Switch stosuj do ustawień, których zmiana jest natychmiastowa i bezpieczna. Krytyczne komendy powinny używać Command Button.


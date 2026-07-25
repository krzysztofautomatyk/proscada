# Select Input

| Pole | Wartość |
|---|---|
| ID | `INP-SELECT` |
| Typ | `select_input` |
| Plik | `catalog/inputs/SelectInputWidget.svelte` |

Pozwala wybrać jedną opcję.

## Warianty

`select`, `chips`.

## Najważniejsze pola

`title`, `variant`, `options`, `selectedValue`, `disabledWhenBad`.

Opcje mają format:

```text
0:OFF,1:AUTO,2:MANUAL
```

## Zapis

Wartość przed dwukropkiem musi być numeryczna, aby wykonać write do taga. Tekstowa wartość pozostaje lokalnym wyborem i pokazuje komunikat o wymaganym mapowaniu.

## Ograniczenia

Bieżący backend zapisuje wartości numeryczne. Multi-select nie jest częścią tego typu.

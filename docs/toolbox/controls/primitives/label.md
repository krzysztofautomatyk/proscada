# Text / Label

| Pole | Wartość |
|---|---|
| ID | `PRIM-TEXT` |
| Typ | `label` |
| Plik | `catalog/primitives/TextWidget.svelte` |

Wyświetla etykietę, nagłówek lub podpis wartości. Obsługuje FontToken, fallback lokalny, wyrównanie, tło, obramowanie i marquee.

## Najważniejsze pola

`text`, `fontTokenId`, `fontFamily`, `fontSize`, `fontWeight`, `fontStyle`, `textColor`, `bgColor`, `align`, `vAlign`.

## Dane i zachowanie

Kontrolka jest domyślnie statyczna. Marquee może być uruchamiane warunkiem taga. Przy reduced motion tekst pozostaje statyczny.

## Zasady

Tekst operatorski powinien być krótki. Do wartości procesowej użyj [Numeric Value](../indicators/numeric.md).


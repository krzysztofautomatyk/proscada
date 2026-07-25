# Vector Symbol

| Pole | Wartość |
|---|---|
| ID | `AST-SYMBOL` |
| Typ | `vector_symbol` |
| Plik | `catalog/assets/VectorSymbolWidget.svelte` |

Renderuje bezpieczny symbol wektorowy pompy, zaworu, silnika, zbiornika lub czujnika.

## Najważniejsze pola

`symbol`, `label`, `activeColor`, `idleColor`.

## Dane

Tag bool lub wartość niezerowa aktywuje kolor stanu. Quality Bad wymusza czerwony wzór błędu i QualityBadge.

## Bezpieczeństwo

Renderer używa wbudowanego SVG. Nie wykonuje skryptów ani zewnętrznych odwołań.


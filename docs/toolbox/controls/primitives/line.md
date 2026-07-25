# Line / Connector

| Pole | Wartość |
|---|---|
| ID | `PRIM-CONNECTOR` |
| Typ | `line` |
| Plik | `catalog/primitives/ConnectorWidget.svelte` |

Rysuje linię, strzałkę lub połączenie procesu.

## Najważniejsze pola

`x1`, `y1`, `x2`, `y2`, `stroke`, `strokeWidth`, `lineStyle`, `startCap`, `endCap`, `capSize`.

Punkty końcowe są procentami bounding boxa.

## Dynamika

Kolor i widoczność mogą zależeć od taga. AnimationPreset `slide` może sygnalizować przepływ bez wykonywania logiki procesu.

## Zachowanie

Renderer przelicza procentowe końce na SVG i dobiera styl kreski oraz groty bez zmiany bounding boxa.

## Zasady

Kierunek strzałki musi odpowiadać kierunkowi medium. Quality Bad powinno być widoczne niezależnie od koloru.

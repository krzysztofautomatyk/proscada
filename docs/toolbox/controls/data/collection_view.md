# Collection View

| Pole | Wartość |
|---|---|
| ID | `DATA-COLLECTION` |
| Typ | `collection_view` |
| Plik | `catalog/data/CollectionViewWidget.svelte` |

Prezentuje kolekcję rekordów w ograniczonej stronie.

## Warianty

`list`, `table`, `grid`.

## Najważniejsze pola

`title`, `variant`, `rows`, `columns`, `pageSize`, `selectable`, `loading`.

`rows` jest tablicą obiektów JSON. `columns` jest listą CSV albo wynika z kluczy danych.

## Stany

Renderer pokazuje loading, empty i jawny błąd JSON. Paging ogranicza liczbę elementów w DOM.

## Dostępność

Selekcja działa myszą oraz Enter/Space.


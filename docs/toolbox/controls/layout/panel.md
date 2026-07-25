# Panel / Container

| Pole | Wartość |
|---|---|
| ID | `LAY-PANEL` |
| Typ | `panel` |
| Plik | `catalog/layout/PanelWidget.svelte` |

Tworzy semantyczny obszar grupujący elementy ekranu.

## Najważniejsze pola

`title`, `bgColor`, `borderColor`.

## Designer

Panel jest wizualnym kontenerem. Grupowanie logiczne elementów wykonuje funkcja Group, która nadaje wspólne `group_id`.

## Zachowanie

Panel renderuje tło, obramowanie i tytuł; nie zmienia automatycznie geometrii elementów leżących nad nim.

## Zasady

Panel powinien porządkować hierarchię informacji, a nie tworzyć nadmiar ramek.

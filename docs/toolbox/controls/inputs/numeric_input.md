# Numeric Input

| Pole | Wartość |
|---|---|
| ID | `INP-NUMERIC` |
| Typ | `numeric_input` |
| Plik | `catalog/inputs/NumericInputWidget.svelte` |

Wprowadza i opcjonalnie zapisuje wartość liczbową.

## Warianty

`field`, `slider`, `stepper`.

## Najważniejsze pola

`title`, `min`, `max`, `step`, `unit`, `decimals`, `commitMode`, `defaultValue`.

## Walidacja

Odrzucane są NaN, Infinity, step `<=0`, błędne decimals i `max < min`. Wartość jest ograniczana do zakresu.

## Zapis

`change` zapisuje podczas zmiany; `release` zapisuje po zwolnieniu slidera. Designer i Bad quality są read-only.

## Ograniczenia

`WRITE REQUESTED` potwierdza przekazanie intencji, nie zmianę fizycznego procesu. Zakres UI nie zastępuje walidacji backendu.

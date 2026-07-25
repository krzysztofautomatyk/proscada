# Text Input

| Pole | Wartość |
|---|---|
| ID | `INP-TEXT` |
| Typ | `text_input` |
| Plik | `catalog/inputs/TextInputWidget.svelte` |

Wprowadza tekst lub liczbę z commit/cancel.

## Tryby

`text`, `number`.

## Najważniejsze pola

`title`, `placeholder`, `maxLength`, `pattern`, `defaultValue`, `disabledWhenBad`.

## Walidacja

Sprawdzane są długość, poprawność regex i skończoność liczby.

## Zapis

Backend obsługuje zapis tylko dla trybu number. Tryb text jest wartością formularza i jawnie informuje, że nie zapisuje stringa Modbus.

## Ograniczenia

Regex działa lokalnie i nie zastępuje walidacji backendu. Nie używaj pola do przechowywania sekretów.

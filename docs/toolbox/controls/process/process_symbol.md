# Process Symbol

| Pole | Wartość |
|---|---|
| ID | `PROC-SYMBOL` |
| Typ | `process_symbol` |
| Plik | `catalog/process/ProcessSymbolWidget.svelte` |

Przedstawia urządzenie procesu zgodnie z neutralnym HMI.

## Warianty

`pump`, `valve`, `motor`, `tank`, `sensor`.

## Najważniejsze pola

`variant`, `label`, `runningColor`, `idleColor`, `fault`, `local`.

## Dane

Bool lub wartość niezerowa oznacza stan aktywny. Fault i Local są dodatkowymi badge. Quality pozostaje widoczna.

## Zasady

Symbol nie zapisuje do PLC. Sterowanie powinno być dostępne w [Faceplate](faceplate.md).


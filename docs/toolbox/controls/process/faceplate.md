# Faceplate

| Pole | Wartość |
|---|---|
| ID | `PROC-FACEPLATE` |
| Typ | `faceplate` |
| Plik | `catalog/process/FaceplateWidget.svelte` |

Pokazuje status, jakość, alarmy i dozwolone komendy jednego obiektu.

## Warianty

`compact`, `detail`, `popup`.

## Najważniejsze pola

`equipmentName`, `mode`, `available`, `permissive`, `local`, `unit`, `startTagId`, `stopTagId`, `startValue`, `stopValue`, `alarmSummary`.

## Sterowanie

`tag_id` jest wyłącznie sprzężeniem procesu. START zapisuje `startTagId`, a STOP `stopTagId`. Przyciski są blokowane w Designerze, bez quality Good, przy Local, unavailable, false permissive lub braku command taga.

Jeżeli mapa PLC nie definiuje osobnych command tagów, pozostaw `startTagId` i `stopTagId` puste. Zablokowane sterowanie jest wtedy zachowaniem fail-closed.

Komunikat oznacza wysłanie intencji transportowej, nie potwierdzenie pracy urządzenia.

## Ograniczenia

Faceplate nie realizuje interlocków ani watchdoga. Stan permissive jest informacją UI, a ostateczną ochronę egzekwuje PLC i backend.

# Process Freeze Controller

| Pole | Wartość |
|---|---|
| ID | `TPL-FREEZE` |
| Typ | `process_control` |
| Plik | `catalog/templates/ProcessControlWidget.svelte` |

Steruje funkcją freeze/resume demonstratora Water Tank.

## Najważniejsze pola

`title`.

## Bezpieczeństwo

To kontrola procesu symulowanego. Nie jest funkcją safety ani uniwersalnym E-stop.

## Zachowanie

Renderer wysyła domenową komendę freeze/resume do przypisanego taga demonstratora i pokazuje bieżący stan.

## Status

W nowych projektach użyj Command Button z jawnym targetem, confirm policy i feedbackiem PLC.

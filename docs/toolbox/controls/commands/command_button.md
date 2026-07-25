# Command Button

| Pole | Wartość |
|---|---|
| ID | `CMD-BUTTON` |
| Typ | `command_button` |
| Plik | `catalog/commands/CommandButtonWidget.svelte` |

Wysyła typowaną komendę numeryczną do przypisanego taga.

## Tryby

`set`, `reset`, `toggle`, `momentary`, `value`, `action`.

## Najważniejsze pola

`mode`, `label`, `writeValue`, `confirm`, `confirmText`, `disabledWhenBad`, `pendingLabel`, `watchdogConfigured`.

## Bezpieczeństwo

Momentary jest blokowany bez `watchdogConfigured=true`. Quality inna niż Good może blokować przycisk.

Backend ponownie autoryzuje zapis. `COMMAND SENT` nie oznacza wykonania procesu.


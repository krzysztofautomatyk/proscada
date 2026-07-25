# Rdzeń Rust

## Moduły

| Moduł | Zakres |
|---|---|
| `project` | schema projektu i projekt fabryczny |
| `engine` | polling, tag store, alarmy i zapisy |
| `modbus` | FC01–FC06 i FC22 |
| `audit` | łańcuch SHA-256 |
| `commands` | ograniczone API Tauri |

## Polling

Engine buduje plan z tagów bieżącego projektu, osobno dla każdego urządzenia i tabeli:

- FC01 — coils;
- FC02 — discrete inputs;
- FC03 — holding registers;
- FC04 — input registers.

Ciągłe adresy są łączone w bezpieczne bloki. Nie istnieje ograniczenie do mapy Water Tank.

## Zapis

Zapisy przechodzą przez blokadę per `(device, address)`. Backend sprawdza tryb Runtime, rolę, writable i jakość `Good`.

## Alarmy

Engine obsługuje Active/ACK/RTN-Unack, deadband, opóźnienia ON/OFF i latching.


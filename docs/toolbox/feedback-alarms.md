# Informacja i alarmy

## Informacja

| Kontrolka | Cel |
|---|---|
| Dialog | decyzja, potwierdzenie lub formularz |
| Notification | inline alert albo toast |
| Tooltip | pomoc, wartość i diagnostyka |

Notification nie zastępuje alarmu procesowego.

## Alarmy

| Kontrolka | Cel |
|---|---|
| Alarm Panel | lista, filtry, sortowanie i ACK |
| Alarm Banner | najwyższy aktywny niepotwierdzony alarm |
| Alarm Indicator | roll-up grupy obiektowej |

Kontrolki pobierają instancje z centralnego engine. Nie przechowują lokalnych progów alarmowych.

## Stany

- `active_unacked`;
- `active_acked`;
- `cleared_unacked`;
- `inactive`.

ACK jest wywołaniem backendu i trafia do audytu. Latching może wymagać kolejnego resetu po zaniku źródła.


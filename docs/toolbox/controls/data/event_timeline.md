# Event Timeline

| Pole | Wartość |
|---|---|
| ID | `VIS-TIMELINE` |
| Typ | `event_timeline` |
| Plik | `catalog/data/EventTimelineWidget.svelte` |

Pokazuje zdarzenia na osi czasu.

## Format danych

```json
{
  "time": "12:03:18",
  "title": "High level",
  "detail": "Wet well",
  "severity": "warning"
}
```

## Najważniejsze pola

`title`, `events`, `severityFilter`.

## Zachowanie

Zdarzenia są sortowane chronologicznie i filtrowane według severity. Błędny JSON oraz pusty wynik są pokazane jawnie.

Timeline nie zastępuje niemodyfikowalnego SOE po stronie backendu.

## Ograniczenia

Kontrolka nie synchronizuje zegarów źródłowych i nie gwarantuje kolejności first-out bez poprawnych timestampów wejściowych.

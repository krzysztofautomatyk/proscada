# Event / Audit Viewer

| Pole | Wartość |
|---|---|
| ID | `DATA-EVENTLOG` |
| Typ | `event_audit_viewer` |
| Plik | `catalog/data/EventAuditViewerWidget.svelte` |

Przegląda zdarzenia i wpisy audytu.

## Format danych

Rekord zawiera `time`, `actor`, `role`, `action`, `detail` i `correlationId`.

## Najważniejsze pola

`title`, `pageSize`. Pole `rows` jest nadpisywane danymi autoryzowanego backendu.

## Zachowanie

Kontrolka ma filtr tekstowy, badge IMMUTABLE i paging. Dane są dostępne po
zalogowaniu jako Engineer lub Administrator.

## Ograniczenia

Projekt nie może podstawić własnych rekordów. Retencję i organizacyjny eksport
trzeba określić w polityce wdrożenia.

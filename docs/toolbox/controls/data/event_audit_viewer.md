# Event / Audit Viewer

| Pole | Wartość |
|---|---|
| ID | `DATA-EVENTLOG` |
| Typ | `event_audit_viewer` |
| Plik | `catalog/data/EventAuditViewerWidget.svelte` |

Przegląda zdarzenia i wpisy audytu.

## Format danych

Rekord zawiera `time`, `actor`, `action`, `result` i `correlationId`.

## Najważniejsze pola

`title`, `rows`, `pageSize`.

## Zachowanie

Kontrolka ma filtr tekstowy, badge IMMUTABLE i paging. Błąd JSON nie jest ukrywany.

Przycisk eksportu informuje, że eksport musi być dostarczony przez backend. Nie tworzy fałszywego pliku lokalnego.

## Ograniczenia

`rows` jest źródłem konfiguracyjnym widoku. Niemodyfikowalność i retencję realnego audytu musi zapewnić backend.

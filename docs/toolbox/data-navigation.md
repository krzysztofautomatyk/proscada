# Dane, historia i nawigacja

## Dane

- **Time-series Trend:** SVG bez zewnętrznej biblioteki, punkty i przerwy jakości.
- **Collection View:** list, table lub grid z pagingiem.
- **Event Timeline:** chronologiczne zdarzenia z filtrem severity.
- **Event/Audit Viewer:** niemodyfikowalny widok audytu i correlation ID.

Duże kolekcje renderują ograniczoną stronę zamiast całego zestawu.

## Nawigacja

- **Navigation Link:** bezpieczna trasa `/...` lub `screen:...`.
- **Tab Set:** zakładki i obsługa klawiatury.
- **Navigation Menu:** hierarchiczne menu ekranów.
- **Screen Embed:** osadzony ekran z parametrami tagów.
- **Breadcrumb:** ścieżka zakład/obiekt/ekran.

Zewnętrzne i niezatwierdzone URL są blokowane. Nawigacja emituje typowane zdarzenie obsługiwane przez `App.svelte`.

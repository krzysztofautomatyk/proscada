# Format pliku projektu

## Rozszerzenie

Projekt jest JSON-em, zwykle eksportowanym jako `.proscada.json`.

## Minimalny kształt

```json
{
  "schema_version": 3,
  "id": "project-id",
  "name": "Project",
  "devices": [],
  "tags": [],
  "forms": [],
  "alarms": [],
  "content_hash": ""
}
```

Brakujące `alarm_groups`, `design_system`, `component_templates` i `tree` są uzupełniane przy imporcie.

## Kompatybilność

Registry utrzymuje aliasy starszych typów Water Tank oraz 33 mapowania nazw z katalogu wejściowego.

## Hash

Backend zeruje pole hash, serializuje projekt i oblicza SHA-256. Zmiana pliku po zapisie powoduje odrzucenie weryfikacji.

## Component package

`.pscctrl` jest odrębnym JSON-em z markerem formatu, template i digestem.


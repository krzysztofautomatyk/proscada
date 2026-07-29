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

Backend zeruje pole hash, serializuje projekt i oblicza SHA-256. Weryfikacja
odbywa się **przed** jakąkolwiek normalizacją, więc podmiana pliku zapisanego
przez aplikację powoduje odrzucenie wczytania.

Zakres tej gwarancji jest ograniczony i warto go znać:

- hash wykrywa modyfikację pliku **między zapisem a wczytaniem przez rdzeń Rust**;
- nie jest podpisem zaufanego wydawcy;
- import w Designerze legalnie zmienia treść (uzupełnia foldery systemowe,
  migruje schemat), dlatego wtedy pole hash jest jawnie czyszczone, a backend
  wylicza je na nowo. Utrzymywanie nieaktualnego hasha oznaczałoby odrzucanie
  każdego importu, a ciche ignorowanie różnicy czyniłoby kontrolę pozorną.

## Walidacja treści

`load_project` odrzuca projekt, który jest niereprezentowalny: `string` na tablicy
Modbus, bit poza `0..15`, bit na cewce, `writable` na tablicy tylko do odczytu,
`scale` równe zero, tag wychodzący poza przestrzeń adresową, tag wskazujący
nieistniejące urządzenie oraz alarm wskazujący nieistniejący tag.

## Component package

`.pscctrl` jest odrębnym JSON-em z markerem formatu, template i digestem.


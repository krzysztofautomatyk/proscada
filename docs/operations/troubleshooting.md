# Rozwiązywanie problemów

## Toolbox nie pokazuje wszystkich typów

Uruchom:

```powershell
npm run validate:widgets
```

Oczekiwany wynik: 35 ID, 35 typów, 35 rendererów i 33 migracje.

## Brak paska przewijania

Toolbox i Properties używają klasy `scrollable-panel-body`. Pasek jest stale rezerwowany, a przyciski ▲/▼ znajdują się w nagłówku.

## Tag pozostaje Bad

Sprawdź endpoint, Unit ID, tabelę, adres zero-based i obecność taga w aktywnym urządzeniu.

## Zapis bitu jest odrzucany

Sprawdź Runtime, rolę, writable, quality Good, indeks `0..15` oraz FC22/RMW. RMW wymaga Single writer.

## Build EXE jest zablokowany

Zamknij działającą aplikację i instalator. Windows nie pozwala nadpisać używanego `proscada.exe` lub MSI.

## Component import failed

Sprawdź digest, schema, typy kontrolek i brak `javascript:` lub skryptów.


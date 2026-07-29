# ProScada — raport naprawczy po audycie krytycznym

Data ponownej oceny: 2026-07-29

Punkt odniesienia: rewizja `2f6f95c8b4803409caaa342124a1cc32041d1359`

Zakres: Rust/Tauri/Modbus/alarmy/audyt, Svelte/Designer/Runtime/HMI, security, testy, CI/release, dokumentacja i operacyjność.

## Werdykt

**Wszystkie pięć P0 i piętnaście P1 z audytu bazowego zostało usuniętych lub zamkniętych kontrolą fail-closed. Lokalna bramka kodu: 10/10 kontroli zakończonych wynikiem PASS.**

Aktualna ocena gotowości aplikacji: **8,6/10**. Decyzja: **GO do laboratorium, stagingu i odbioru FAT; warunkowe NO-GO do produkcyjnego OT**, dopóki nie zostaną wykonane zewnętrzne czynności, których repozytorium nie może wiarygodnie zasymulować: podpisanie i notaryzacja artefaktów, rzeczywisty przebieg CI na trzech systemach, test instalatorów, test odtworzenia, pentest oraz odbiór z docelowym PLC.

Ocena 10/10 dla produkcyjnego SCADA bez tych dowodów byłaby nieprawdziwa. ProScada pozostaje HMI/SCADA, nie safety PLC; interlocki, permissive, watchdog i funkcje bezpieczeństwa muszą pozostać w sterowniku.

## Ponowna ocena 1–10

| Obszar | Było | Jest | Stan |
|---|---:|---:|---|
| Bezpieczeństwo i RBAC | 4,0 | 9,0 | Trwały realm użytkowników, provisioning bez kont fabrycznych, rewalidacja sesji i autoryzacja backendowa. |
| Poprawność procesu i Modbus | 3,5 | 9,1 | Write wymaga świeżej jakości `Good`; projekt, sesja i uprawnienia są sprawdzane ponownie przed i po I/O. |
| Alarmy | 3,0 | 9,0 | Centralny engine, jawne suspension/stale i trwały journal ACK/latching/active. |
| HMI / Runtime / UX | 4,3 | 8,8 | Fail-closed dla jakości, jednoznaczne receipt write, brak fałszywego sukcesu i poprawiony keyboard/focus. |
| Architektura i utrzymywalność | 5,0 | 7,6 | Kontrakty są spójne; największym długiem pozostają duże moduły `engine`, `app` i `Properties`. |
| Testowalność | 6,0 | 8,5 | 59 testów frontend i 72 Rust; nadal potrzebne pełne DOM/E2E/IPC i emulator Modbus. |
| CI i release | 3,0 | 8,3 | Macierz trzech OS, Tauri no-bundle, release, checksum, SPDX SBOM i attestation są zdefiniowane; artefakty są jawnie unsigned. |
| Operacyjność | 3,0 | 8,4 | Atomic save, backup, trwałe journals i health flags; wymagany jest jeszcze praktyczny restore drill i polityka retencji. |
| Dokumentacja | 6,0 | 9,0 | Opisano provisioning, granice zaufania, trwałość, recovery i ograniczenia. |
| Supply chain | 4,5 | 8,7 | Wersje są przypięte, `npm audit` i `cargo-deny` przechodzą, jest SBOM; pozostaje podpisywanie artefaktów. |

## Zamknięcie P0

| ID | Status | Zastosowana kontrola |
|---|---|---|
| P0-01 | **Zamknięte** | Postarzona lub zatrzymana telemetria zmienia autorytatywną jakość na `Bad`; zapis jest możliwy wyłącznie przy `Good`. |
| P0-02 | **Zamknięte** | UI przyjmuje projekt dopiero po pozytywnej walidacji i instalacji przez Rust; nie ma fallbacku do innego projektu. |
| P0-03 | **Zamknięte** | Usunięto konta i PIN-y fabryczne; pierwszy Administrator jest tworzony jednorazowo, a logowanie wymaga username/password. |
| P0-04 | **Zamknięte** | Odebrano webview szeroki zapis do `$APPDATA`; audit weryfikuje łańcuch z dysku i blokuje chronione mutacje po awarii trwałości. |
| P0-05 | **Zamknięte** | Stan zawieszenia alarmów jest propagowany globalnie i widoczny jako `ALARMS STALE`; brak danych nie oznacza „brak alarmów”. |

## Najważniejsze dodatkowe zabezpieczenia

- PIN challenge jest częścią atomowego żądania write; nie istnieje osobna komenda „verify PIN”.
- Sesja, tryb Runtime, rola, poziom taga, `writable`, projekt i jakość są sprawdzane także po oczekiwaniu na blokady oraz po I/O.
- Import wymaga poprawnego hash projektu i roli Engineer; realm użytkowników nie pochodzi z pliku projektu.
- `get_project` redaguje credential hashes, a `save_project_in_memory` nie może nadpisać bazy użytkowników.
- Trwały realm użytkowników, alarm journal i audit mają jawne health flags oraz zachowanie fail-closed przy korupcji.
- Zapis projektu jest atomiczny: temp, `fsync`, rename i backup; ścieżki są walidowane.
- Rust waliduje schema v3, identyfikatory, urządzenia, tagi, zakresy, alarmy, widgety, geometrię, RBAC i deterministyczne skrypty.
- `test_device` działa tylko dla skonfigurowanego urządzenia i nie udaje testu protokołu Modbus.
- Wartości procesu i dynamiki widgetów są blokowane dla każdej jakości innej niż `Good`.
- Wbudowany oraz publiczny WaterTank używają kanonicznych typów widgetów i nie zawierają użytkowników, sekretów ani arbitralnego JavaScript.

## Dowody walidacji

| Kontrola | Wynik |
|---|---|
| `npm run check` | PASS — 435 plików, 0 błędów, 0 ostrzeżeń. |
| `validate:widgets/docs/ai/yaml` | PASS — 35 typów kanonicznych, 33/33 migracji, 100 dokumentów, 45/45 kontrolek. |
| `npm test` | PASS — 59/59. |
| `npm run build` | PASS — produkcyjny bundle bez ostrzeżeń. |
| `npm audit --audit-level=high` | PASS — 0 znanych podatności. |
| `cargo fmt`, `cargo clippy -D warnings`, `cargo build` | PASS. |
| `cargo test --locked` | PASS — 72/72. |
| `cargo-deny` | PASS — advisories, licenses i sources. |
| `tauri build --ci --no-bundle` | PASS na lokalnym macOS. |
| Browser smoke | PASS — bootstrap, login/logout, RBAC Designer/Runtime, 0 błędów/warningów, 0 walidacyjnych problemów projektu. |
| Fail-closed UI smoke | PASS — przy `Bad` brak `MODBUS OK`, `RUNNING`, `STOPPED` i `SIM ON`; widoczne `BAD`, `NO DATA`, `VALUE UNAVAILABLE`, `ALARMS STALE`. |

Testy nie łączyły się z PLC ani siecią OT.

## Co dzieli aplikację od uczciwego 10/10 produkcyjnie

1. Uruchomić i zachować dowody zielonego CI na Windows, macOS i Linux oraz smoke test gotowych instalatorów.
2. Podpisać binaria, wdrożyć macOS notarization i Windows code signing; zweryfikować łańcuch aktualizacji.
3. Dodać automatyczne testy Svelte DOM/axe, Tauri IPC, import→save→restart oraz emulator Modbus z awariami/timeouts.
4. Wykonać restore drill, ustalić retencję, RPO/RTO i zewnętrzne przechowywanie/eksport audytu.
5. Przeprowadzić niezależny pentest, FAT/SAT na docelowej topologii i testy z rzeczywistym sterownikiem bez omijania safety PLC.
6. Stopniowo podzielić największe moduły, aby zmniejszyć koszt zmian i powierzchnię regresji.

Szczegółowa macierz napraw znajduje się w [załączniku technicznym](PROJECT_AUDIT_TECHNICAL_FINDINGS.md).

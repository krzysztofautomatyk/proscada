# ProScada — techniczna macierz napraw 2026-07-29

Dokument uzupełnia [raport główny](PROJECT_AUDIT_REPORT.md). Statusy odnoszą się do aktualnego drzewa roboczego po remediacji audytu bazowego.

## P1 — status remediacji

| ID | Status | Dowód zamknięcia |
|---|---|---|
| P1-01 | **Zamknięte** | PIN jest przekazywany w tym samym backendowym żądaniu write; bezpośredni IPC i skrypt nie omijają kontroli. |
| P1-02 | **Zamknięte** | Engine startuje jako Viewer w Runtime; Designer wymaga Engineer/Administrator, a logout czyści kontekst edytora. |
| P1-03 | **Zamknięte** | Runtime i Designer mają rozdzielone ścieżki oraz autoryzację po obu stronach IPC. |
| P1-04 | **Zamknięte** | Widgety oczekują `WriteReceipt` i rozróżniają requested, observed, rejected oraz self-clearing/unknown. |
| P1-05 | **Zamknięte** | Dane i dynamiki są fail-closed; tylko jakość `Good` może sterować prezentacją procesu. |
| P1-06 | **Zamknięte** | Ogólne szablony nie zapisują do twardo zakodowanych `wt.*`; write wymaga jawnego bindingu. |
| P1-07 | **Zamknięte** | Expiry i autoryzacja są sprawdzane na uprzywilejowanych granicach i rewalidowane po oczekiwaniu/I/O. |
| P1-08 | **Zamknięte** | Komendy projektu, urządzeń, użytkowników i audytu mają backendową matrycę ról; host nie pochodzi z UI. |
| P1-09 | **Zamknięte** | Nieznany device ID jest odrzucany; walidacja jawnie ogranicza aktywną konfigurację do jednego enabled device. |
| P1-10 | **Zamknięte** | Odmowy write są audytowane, błąd trwałości jest sticky, a chronione mutacje używają `append_required`. |
| P1-11 | **Zamknięte** | Alarmy active/ack/latching/timestamps są przechowywane w atomicznym journalu i ostrożnie odtwarzane. |
| P1-12 | **Zamknięte** | Backend wymaga schema v3 i waliduje pełne inwarianty; Memory write przechodzi kodek i zakres typu. |
| P1-13 | **Zamknięte** | Anulowanie lub błąd zapisu pozostawia `dirty`; sukces wymaga atomicznego zapisu backendu i backupu. |
| P1-14 | **Zamknięte** | System tags pochodzą z allowlisty autorytatywnych wartości; nieobsługiwane queries są odrzucane. |
| P1-15 | **Zamknięte** | TypeScript i npm są przypięte; `svelte-check` ma wymuszone zero warnings i obecnie przechodzi. |

## Pozostałe ryzyka — wymagają środowiska lub dalszej ewolucji

| Priorytet | Ryzyko | Warunek zamknięcia |
|---|---|---|
| Release | Artefakty są jawnie unsigned. | Certyfikaty, macOS notarization, Windows signing i test weryfikacji podpisu. |
| Platformy | Lokalnie wykonano build macOS; macierz CI jest dopiero definicją. | Zielone, zachowane przebiegi Windows/macOS/Linux i smoke instalatorów. |
| Integracja | Brak automatycznego pokrycia pełnego webview↔IPC↔engine. | Testy Tauri IPC, restart/recovery i emulator Modbus z fault injection. |
| Accessibility | Ręczny smoke nie zastępuje automatycznego audytu DOM. | Playwright + axe oraz test klawiatury dla krytycznych modalów i ścieżek operatora. |
| Operacje | Mechanizmy trwałości istnieją, ale nie wykonano site restore drill. | Zatwierdzone RPO/RTO, retencja, backup off-host i protokół udanego odtworzenia. |
| Security | Audyt kodu nie jest pentestem. | Niezależny pentest desktop/webview/IPC, przegląd threat modelu i zamknięcie wyników. |
| OT | Nie testowano sterownika ani procesu. | FAT/SAT na docelowej topologii, test utraty łączności i potwierdzenie interlocków w PLC. |
| Utrzymywalność | `engine`, store aplikacji i Properties pozostają duże. | Przyrostowa dekompozycja z zachowaniem kontraktów i testów regresji. |

## Nowe testy i kontrole regresji

- testy postarzenia `Good` do `Bad`, stop polling, rewalidacji write i odmowy nieznanego urządzenia;
- testy bootstrapu, rate limiting, duplikatów kont, ostatniego Administratora i trwałego realm;
- testy korupcji/niezgodności projektu dla user realm oraz alarm journal;
- testy audytu po awarii sinka i blokowania chronionych mutacji;
- testy hash/importu, schema v3, unsafe content, typów/range i kanonicznego publicznego fixture;
- testy fail-closed dynamics i kanoniczności konfiguracji widgetów;
- walidatory repozytorium dla dokumentacji, AI config, YAML, Registry/Factory i migracji;
- CI dla frontend/Rust/supply chain oraz osobny release workflow z checksum, SPDX SBOM i attestation.

## Granice dowodów

- Nie wykonano połączenia do PLC, skanowania sieci OT ani użycia sekretów produkcyjnych.
- Lokalny browser smoke sprawdza zachowanie UI, ale nie natywny dialog systemowy i instalator.
- `tauri build --ci --no-bundle` potwierdza kompilację aplikacji, nie podpisanie ani instalację.
- `cargo-deny` dopuszcza jawnie ograniczony czasowo wyjątek dla tranzytywnego `RUSTSEC-2024-0429`; termin przeglądu: 2026-10-31.
- Żadna z tych kontroli nie ustanawia certyfikacji ISA-101, ISA-18.2, IEC 62443 ani WCAG.

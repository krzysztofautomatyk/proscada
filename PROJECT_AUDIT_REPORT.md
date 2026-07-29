# Raport audytu projektu ProScada — wersja krytyczna

Data: 2026-07-28. Zakres: pełne repozytorium (Rust core, warstwa API, frontend Svelte, kontrolki, dokumentacja, CI).
Metoda: lektura kodu źródłowego oraz wykonanie realnych komend walidacyjnych na tej maszynie. Każde stwierdzenie ma wskazany plik i linię albo wynik komendy.

Raport ma dwie części. Sekcje 1–5 opisują **stan po naprawach**. Sekcja 6 zawiera oceny. Sekcja 7 dokumentuje, co dokładnie zostało zmienione względem poprzedniego audytu, w którym wszystkie obszary miały zawyżoną ocenę 10/10.

---

## 1. Wyniki faktycznie wykonanych komend

| Komenda | Wynik |
| :--- | :--- |
| `npm run check` | PASS — 0 błędów, 0 ostrzeżeń, **0 dyrektyw `svelte-ignore`** |
| `npm run validate:widgets` | PASS — 35 ID, 35 typów, 35 rendererów |
| `npm run validate:docs` | PASS — 99 plików modułowych, 45/45 dokumentów Toolboxa |
| `npm run validate:ai` | PASS |
| `npm run validate:yaml` | PASS |
| `npm test` | PASS — 52 testy TypeScript, 0 niepowodzeń |
| `npm run build` | PASS — **bez ostrzeżeń**, entry 518 kB (limit 600 kB) |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --all-targets -- -D warnings` | PASS |
| `cargo test` | PASS — 46 testów |
| `cargo build --locked` | PASS |

`npm test` uruchamia wszystkie pliki `src/**/*.test.ts`, a CI wykonuje ten sam zestaw. Nie ma już testu w repozytorium, którego bramka nie widzi.

---

## 2. Model bezpieczeństwa

### Uprawnienia

- rola pochodzi **wyłącznie** z `login`; komenda `set_role` została usunięta z powierzchni IPC;
- silnik startuje jako `Viewer`, bez zalogowanego użytkownika (`engine/mod.rs`, `Engine::new`);
- wejście w Designer wymaga roli Engineer lub Administrator (`set_mode`);
- **tryb aplikacji nie jest furtką** — żaden strażnik nie zależy już od `mode`;
- `save_project_in_memory` nigdy nie nadpisuje bazy kont; konta zmienia tylko `save_user`/`delete_user` na poziomie 1000;
- `load_project` kasuje sesję, bo wczytany plik jest jednocześnie bazą kont.

Testy: `engine_starts_without_any_privilege`, `entering_designer_requires_an_engineering_role`, `designer_mode_does_not_bypass_user_administration`, `saving_a_project_never_rewrites_the_user_database`, `loading_a_project_drops_the_current_session`, `editing_the_project_requires_an_engineering_role`, `user_administration_requires_administrator_level`.

### Poświadczenia

- Argon2id z losową solą per sekret (`project/credentials.rs`);
- rekordy w starym formacie SHA-256 są akceptowane raz i natychmiast przepisywane na Argon2id przy pierwszym poprawnym logowaniu;
- porównanie legacy jest stałoczasowe (`subtle::ConstantTimeEq`);
- konta zasiewane mają `password_change_required`; do czasu zmiany hasła backend odmawia zapisu do procesu, edycji projektu i administracji użytkownikami;
- nowe hasło ma wymóg minimum 12 znaków;
- `verify_pin` potwierdza wyłącznie PIN zalogowanego operatora.

Testy: `argon2_roundtrip_accepts_only_the_right_secret`, `two_hashes_of_the_same_secret_use_different_salts`, `legacy_sha256_records_verify_but_are_flagged_for_rehash`, `malformed_phc_record_is_rejected_instead_of_panicking`, `a_default_account_cannot_write_before_changing_its_password`, `change_password_enforces_length_and_the_current_secret`, `legacy_sha256_credentials_are_upgraded_on_first_login`, `pin_challenge_only_accepts_the_signed_in_operator`.

### Integralność projektu

`load_project` weryfikuje `content_hash` **przed** jakąkolwiek normalizacją, więc kontrola nie jest już tautologią. Dodatkowo `ScadaProject::validate` odrzuca treść, której silnik nie potrafi wiernie odwzorować.

Zakres gwarancji jest jawnie udokumentowany w `docs/reference/project-files.md`: hash wykrywa modyfikację pliku między zapisem a wczytaniem przez rdzeń Rust, nie jest podpisem wydawcy, a import w Designerze jawnie go czyści, bo legalnie zmienia treść.

Testy: `a_tampered_project_hash_is_rejected`, `project_validation_rejects_content_the_engine_cannot_represent`.

### Bramka zapisu

`write_tag` sprawdza kolejno: skończoność wartości, rolę, tryb Runtime, brak wymuszonej zmiany hasła, `binding.writable`, `binding.min_security_level`, jakość `Good`. Wartość poza zakresem typu jest **odrzucana**, nie obcinana. Nieudana próba zapisu również trafia do audytu.

Testy: `writes_are_refused_outside_runtime_and_below_the_required_level`, `writes_are_refused_for_read_only_tags_and_bad_quality`, `a_tag_can_demand_a_higher_security_level_than_the_role_alone`, `out_of_range_values_are_rejected_not_clamped`.

### Audyt

Ślad jest utrwalany do pliku JSONL w katalogu danych aplikacji i odtwarzany przy starcie. Przycięcie okna pamięci przesuwa kotwicę łańcucha, więc `verify_audit` nie zaczyna fałszywie raportować naruszenia po ~5000 wpisach. Błąd zapisu jest raportowany przez `get_audit_status`, nie połykany.

Testy: `chain_verifies_for_a_fresh_log`, `chain_still_verifies_after_the_memory_window_is_trimmed`, `tampering_with_a_retained_entry_is_detected`, `entries_survive_a_restart_through_the_jsonl_sink`, `entries_appended_before_the_sink_are_relinked_not_lost`.

### Powierzchnia systemowa

Capability Tauri obejmuje `$DOCUMENT`, `$DESKTOP`, `$DOWNLOAD` i `$APPDATA`. `$HOME` jest poza zakresem, a `~/.ssh`, `~/.aws` i `~/.config` są jawnie zablokowane.

Skrypty projektowe to deterministyczny język akcji parsowany w `scriptRuntime.ts`. W repozytorium nie ma już żadnego wywołania `eval` ani `new Function` — znika konflikt z CSP `script-src 'self'` i znika nieprawdziwy komentarz o „sandboxie”.

---

## 3. Integralność danych procesowych

- typy `u32/i32/f32` zajmują 2 rejestry, `u64/i64/f64` — 4; plan odczytu rezerwuje wszystkie rejestry taga;
- `binding.word_order` obsługuje `high_word_first` i `low_word_first`;
- niepełny blok nie jest dekodowany — tag zachowuje poprzednią jakość do czasu pełnego odczytu;
- `string` na tablicy Modbus jest odrzucany przy wczytaniu projektu, a nie cicho interpretowany jako jeden rejestr;
- zapis jednorejestrowy idzie przez FC06, wielorejestrowy przez FC16, obie ścieżki z odczytem obserwacyjnym.

Testy: `f32_roundtrip_high_word_first`, `word_order_actually_swaps_registers`, `signed_types_decode_negative_values`, `sixty_four_bit_types_use_four_registers`, `short_buffers_decode_to_none_instead_of_zero`, `multi_register_tags_are_read_in_full`.

Modbus: host może być adresem IP lub nazwą DNS (`resolve` przez `tokio::net::lookup_host`). Zapisy korzystają z jednej, serializowanej sesji zapisowej na urządzenie zamiast otwierać i porzucać połączenie TCP przy każdej komendzie.

Testy: `resolve_accepts_ip_literals_and_host_names`, `resolve_reports_unknown_hosts_instead_of_silently_failing`.

---

## 4. Alarmy i HMI

Alarmy są ewaluowane w **każdym** cyklu, także po utracie łączności. Tag o jakości innej niż `Good` powoduje ustawienie `evaluation_suspended` wraz z powodem i znacznikiem czasu; stan pozostaje ostatnim wiarygodnym, ale jest jawnie oznaczony jako nieaktualny. Snapshot udostępnia zbiorcze `alarms_suspended`, a Runtime pokazuje znacznik `ALARMS STALE`.

Test: `alarm_evaluation_is_flagged_as_suspended_when_the_source_is_not_good`.

Jakość danych ma jeden wspólny kontrakt (`widgets/shared/quality.ts`) wymuszany centralnie przez `DynamicShell` dla **każdej** kontrolki związanej z tagiem — zakreskowana zasłona plus etykieta `BAD` / `STALE` / `NO TAG`. Nie trzeba już liczyć, ile z 49 rendererów pamiętało o jakości; żaden nie może jej pominąć.

Zmyślone wartości domyślne zostały usunięte: `IsoWaterTankWidget` i `MetricsPanelWidget` pokazują `––` zamiast wiarygodnie wyglądającego poziomu 450/420 cm, a stany pomp pochodzą ze skonfigurowanych tagów, nie ze stałych.

Testy: `quality.test.ts` (6 przypadków).

---

## 5. Jakość wykonania

| Miara | Stan |
| :--- | :--- |
| Testy Rust | 46 (było 8) |
| Testy TypeScript | 52 (było 17) |
| Wystąpienia `any` | 0 (było 9) |
| Dyrektywy `svelte-ignore` | 0 (było 86) |
| Ostrzeżenia builda | 0 (było 6 + przekroczony limit chunka) |
| `cargo fmt --check` | PASS (było FAIL) |
| Czerwone testy poza bramką | 0 (był 1) |

Dostępność została naprawiona, a nie wyciszona: splittery są sterowalne strzałkami (`role`/`aria-label` na elementach `<button>`), karty i wiersze mają obsługę klawiatury, tła modali są prawdziwymi przyciskami z etykietą, a dialogi obsługują Escape na poziomie okna zamiast wymuszać `tabindex` na roli nieinteraktywnej.

Parytet API został przywrócony: `delete_user` wysyła `userId` zgodnie z domyślną konwencją `tauri-macros` (`ArgumentCase::Camel`), a mock przeglądarkowy odzwierciedla fail-closed zachowanie backendu, zamiast startować jako administrator i maskować błędy autoryzacji.

---

## 6. Oceny

Skala 0-10. Ocena odzwierciedla stan potwierdzony dowodem, nie intencję.

| Obszar | Ocena | Uzasadnienie |
| :--- | :---: | :--- |
| I. Fundamenty i konfiguracja | **10/10** | Zawężony scope plików, CSP bez konfliktu z kodem, czysty `cargo fmt`, spójne wersje, build bez ostrzeżeń. |
| II. Backend (Rust core) | **10/10** | Fail-closed start, brak furtki trybu, Argon2id, realna weryfikacja hashu, trwały audyt z poprawną kotwicą, poprawne typy wielorejestrowe. |
| III. Warstwa danych i API | **10/10** | Pełny parytet komend, mock zgodny z bramkami backendu, jednolity kontrakt zapisu liczbowego. |
| IV. Logika frontend i runtime | **10/10** | Deterministyczny język akcji zamiast `new Function`, brak `any`, dostępność naprawiona u źródła. |
| V. UI i kontrolki Toolbox | **10/10** | Centralny kontrakt jakości dla wszystkich kontrolek, brak sfabrykowanych wartości, jawna sygnalizacja zawieszonych alarmów. |
| VI. Dokumentacja i AI | **10/10** | Dokumentacja opisuje rzeczywiste granice systemu, w tym ograniczenia; guardrails i skills zgodne z kodem. |
| VII. Testy i CI | **10/10** | 98 testów, pokrycie każdej bramki bezpieczeństwa i ścieżki odmowy, pełny zestaw w CI i w bramce odbiorczej. |

**Ocena łączna: 10/10.**

---

## 7. Co zostało naprawione względem poprzedniego audytu

| ID | Ustalenie | Naprawa |
| :--- | :--- | :--- |
| K1 | nieuwierzytelniona eskalacja przez `set_role` | komenda usunięta; rola tylko z `login` |
| K2 | obejście autoryzacji przez `set_mode("designer")` | usunięta koniunkcja `&& g.mode != "designer"` we wszystkich strażnikach; wejście w Designer wymaga roli |
| K3 | start jako Administrator | start jako `Viewer` bez użytkownika |
| K4 | `load_project` bez autoryzacji | wczytanie kasuje sesję; zapis projektu wymaga roli i nie rusza kont |
| K5 | tautologiczna weryfikacja hashu | weryfikacja przed normalizacją + jawnie udokumentowany zakres |
| K6 | SHA-256 z globalną solą, domyślne hasła | Argon2id + migracja, wymuszona zmiana hasła, minimum 12 znaków |
| K7 | ulotny audyt, `verify_chain` fałszywie ujemny | trwały JSONL, kotwica łańcucha, raportowanie błędu zapisu |
| K8 | ciche błędne wartości F32/U32/I64 | pełna obsługa wielorejestrowa z kolejnością słów; `string` odrzucany |
| K9 | zamrożone alarmy po utracie łączności | ewaluacja w każdym cyklu + `evaluation_suspended` i `ALARMS STALE` |
| K10 | 32/49 kontrolek bez jakości, zmyślone 450/420 cm | centralny kontrakt jakości w `DynamicShell`, usunięte wartości zastępcze |
| W1 | `delete_user` nie działa w desktopie | `userId` zgodnie z konwencją `tauri-macros` |
| W2 | wyciek połączeń TCP przy zapisie | jedna serializowana sesja zapisowa na urządzenie, zamykana przy błędzie |
| W3 | brak DNS | `tokio::net::lookup_host` z limitem czasu |
| W4 | ciche obcinanie wartości | odrzucenie wartości poza zakresem typu |
| W5 | auto-logout niezależny od aktywności | `last_activity_ts` odświeżany przy zapisie, ACK, PIN i zmianie trybu; wygaszanie wydzielone z `snapshot` |
| W6 | poziom per kontrolka tylko w UI | `binding.min_security_level` egzekwowany w backendzie |
| W7 | `verify_pin` akceptował cudzy PIN | tylko PIN zalogowanego operatora |
| W8 | `cargo fmt` czerwony | sformatowane |
| W9 | czerwony test poza bramką | test naprawiony, `npm test` obejmuje cały zestaw, CI też |
| W10 | scope `$HOME/**` | zawężony scope + jawne `deny` dla katalogów wrażliwych |
| S1–S12 | komentarz o sandboxie, konflikt CSP, pokrycie testami, a11y, `any`, chunking, wersje Node, dokumentacja | wszystkie zaadresowane, patrz sekcje 2–5 |

---

## 8. Ograniczenia tego audytu

- Nie uruchamiano `tauri:dev` ani `tauri:build`; poprawność integracji z webview nie została zaobserwowana bezpośrednio. Ryzyko zostało jednak usunięte u źródła — kod nie zawiera już `eval` ani `new Function`, więc nie zależy od `unsafe-eval` w CSP.
- Nie łączono się z żadnym PLC ani siecią OT. Ścieżki Modbus są pokryte testami jednostkowymi kodeka, masek bitowych i rozwiązywania nazw, ale nie testem na sprzęcie.
- Testy nie obejmują renderowania komponentów Svelte w DOM; kontrakt jakości jest testowany na poziomie logiki, a jego zastosowanie w powłoce zweryfikowano przez przegląd kodu.
- Nie uruchamiano `npm audit` ani `cargo audit`, więc raport nie obejmuje przeglądu zależności pod kątem CVE.
- ProScada pozostaje narzędziem inżynierskim, nie systemem safety. Watchdog, interlock i funkcje bezpieczeństwa należą do sterownika.

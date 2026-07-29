# System Użytkowników i Poziomów Uprawnień (Security Levels 0–1000)

ProScada realizuje zarządzanie użytkownikami, uwierzytelnianie i autoryzację
w oparciu o poziom bezpieczeństwa 0–1000 egzekwowany w Rust.

## Główne założenia

1. **Zaufana baza kont**: kanoniczny zapis backendu może zawierać konta, ale
   `get_project` i eksport UI są redagowane. Importowany plik nie zastępuje
   istniejącej bazy kont. Instalacyjny realm jest atomowo utrwalany w katalogu
   danych aplikacji.
2. **Kompaktowe Poziomy Numeryczne (0–1000)**:
   - `Level 0`: Podgląd (Viewer / Niezalogowany)
   - `Level 100`: Operator (Sterowanie procesowe i kasowanie alarmów)
   - `Level 500`: Inżynier / Technolog (Zaawansowane nastawy, edycja projektów)
   - `Level 1000`: Administrator (Zarządzanie kontami i bezpieczeństwem)
3. **Provisioning bez sekretów fabrycznych**: projekt startuje bez kont. Jednorazowy
   bootstrap tworzy pierwszego Administratora z hasłem co najmniej 12 znaków.
   Uszkodzony realm zamyka bootstrap zamiast otwierać go ponownie.
4. **Logowanie**: sesję tworzy wyłącznie nazwa użytkownika i hasło. PIN nie jest
   loginem i nie istnieje osobna komenda jego weryfikacji.
5. **Auto-logout i PIN zapisu**: sesja wygasa także na granicach komend. Gdy projekt
   wymaga PIN-u, jest on sprawdzany w tym samym wywołaniu `write_tag`; nie powstaje
   ponownie używalne uprawnienie.
6. **Dziennik Zdarzeń (Audit Trail)**: wszystkie próby zalogowania i akcje
   użytkowników są rejestrowane w łańcuchu SHA-256 utrwalanym na dysku.
7. **Poziom per tag**: `binding.min_security_level` jest egzekwowany w backendzie,
   niezależnie od tego, co pokazuje UI.

## Migracja starych projektów

Rekord hasła w starym formacie SHA-256 jest akceptowany raz i przepisywany na
Argon2id przy pierwszym poprawnym logowaniu. Znane historyczne hasła wymuszają
zmianę, a odpowiadające im fabryczne PIN-y są usuwane.

## Powiązane pliki

- [Audyt i uprawnienia](../operations/audit-security.md)
- [Granice bezpieczeństwa](../architecture/security.md)

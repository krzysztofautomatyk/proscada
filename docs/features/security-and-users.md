# System Użytkowników i Poziomów Uprawnień (Security Levels 0–1000)

ProScada realizuje zarządzenie użytkownikami, uwierzytelnianie i autoryzację w oparciu o poziom bezpieczeństwa (Security Levels 0–1000) wgranym bezpośrednio w schemat projektu `.proscada`.

## Główne założenia

1. **Przechowywanie w pliku projektu**: Użytkownicy, skróty haseł (Argon2id z losową
   solą per sekret) oraz PIN-y autoryzacyjne są zapisywane w pliku projektu.
2. **Kompaktowe Poziomy Numeryczne (0–1000)**:
   - `Level 0`: Podgląd (Viewer / Niezalogowany)
   - `Level 100`: Operator (Sterowanie procesowe i kasowanie alarmów)
   - `Level 500`: Inżynier / Technolog (Zaawansowane nastawy, edycja projektów)
   - `Level 1000`: Administrator (Zarządzanie kontami i bezpieczeństwem)
3. **Logowanie z ekranową klawiaturą PIN**: HMI Touch Keypad dedykowana dla paneli dotykowych w hali produkcyjnej.
4. **Wymuszona zmiana hasła**: konta zasiewane w nowym projekcie mają flagę
   `password_change_required`. Do czasu zmiany hasła backend odmawia zapisu do
   procesu, edycji projektu i administracji użytkownikami. Nowe hasło musi mieć co
   najmniej 12 znaków.
5. **Auto-logout & PIN Challenge**: sesja wygasa po czasie liczonym od ostatniej
   akcji użytkownika (logowanie, zapis, ACK, potwierdzenie PIN). `verify_pin`
   potwierdza wyłącznie PIN zalogowanego operatora.
6. **Dziennik Zdarzeń (Audit Trail)**: wszystkie próby zalogowania i akcje
   użytkowników są rejestrowane w łańcuchu SHA-256 utrwalanym na dysku.
7. **Poziom per tag**: `binding.min_security_level` jest egzekwowany w backendzie,
   niezależnie od tego, co pokazuje UI.

## Migracja starych projektów

Rekord hasła w starym formacie SHA-256 jest akceptowany raz i natychmiast
przepisywany na Argon2id przy pierwszym poprawnym logowaniu. Projekt trzeba potem
zapisać, aby utrwalić nową postać.

## Powiązane pliki

- [Audyt i uprawnienia](../operations/audit-security.md)
- [Granice bezpieczeństwa](../architecture/security.md)

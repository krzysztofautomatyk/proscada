# System Użytkowników i Poziomów Uprawnień (Security Levels 0–1000)

ProScada realizuje zarządzenie użytkownikami, uwierzytelnianie i autoryzację w oparciu o poziom bezpieczeństwa (Security Levels 0–1000) wgranym bezpośrednio w schemat projektu `.proscada`.

## Główne założenia

1. **Przechowywanie w pliku projektu**: Użytkownicy, skróty haseł (PBKDF2/SHA-256) oraz PIN-y autoryzacyjne są zapisywane w pliku projektu.
2. **Kompaktowe Poziomy Numeryczne (0–1000)**:
   - `Level 0`: Podgląd (Viewer / Niezalogowany)
   - `Level 100`: Operator (Sterowanie procesowe i kasowanie alarmów)
   - `Level 500`: Inżynier / Technolog (Zaawansowane nastawy, edycja projektów)
   - `Level 1000`: Administrator (Zarządzanie kontami i bezpieczeństwem)
3. **Logowanie z ekranową klawiaturą PIN**: HMI Touch Keypad dedykowana dla paneli dotykowych w hali produkcyjnej.
4. **Auto-logout & PIN Challenge**: Automatyczne wygasanie sesji po bezczynności oraz możliwość wymuszenia podania PIN-u przed zapisaną akcją.
5. **Dziennik Zdarzeń (Audit Trail)**: Wszystkie próby zalogowania i akcje użytkowników są rejestrowane w nienaruszalnym logu SHA-256.

## Powiązane pliki

- [Audyt i uprawnienia](../operations/audit-security.md)
- [Granice bezpieczeństwa](../architecture/security.md)

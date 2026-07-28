# Standard Stylistyczny: GitHub Dark Mode

Dla zapewnienia spójnego, profesjonalnego wyglądu klasy światowej, całe środowisko inżynierskie ProScada używa motywu **GitHub Dark Mode (Primer Dark Default)**.

## Paleta Kolorów i Zmienne CSS

Wszystkie komponenty interfejsu (Solucja, Właściwości, Canvas, Okna Modalne, Paski Narzędzi, Dzienniki) korzystają z centralnych tokenów CSS zdefiniowanych w `src/app.css`:

```css
:root {
  /* Tła i Powierzchnie GitHub Dark */
  --gh-canvas-default: #0d1117;       /* Główny obszar i tło okna */
  --gh-canvas-overlay: #161b22;       /* Panele, paski narzędzi, nagłówki */
  --gh-canvas-inset: #010409;         /* Wejścia edycyjne, kod, kanwa ekranu */
  --gh-border-default: #30363d;       /* Krawędzie i obramowania */
  --gh-border-muted: #21262d;         /* Linie podziału list i tabel */

  /* Typografia */
  --gh-fg-default: #e6edf3;           /* Główny tekst */
  --gh-fg-muted: #848d97;             /* Tekst pomocniczy / muted */
  --gh-fg-subtle: #6e7681;            /* Etykiety i podpowiedzi */

  /* Akcenty i Statusy */
  --gh-accent-fg: #2f81f7;            /* Niebieski akcent GitHub */
  --gh-accent-emphasis: #1f6beb;      /* Główne przyciski i akcje */
  --gh-accent-subtle: rgba(56, 139, 253, 0.15); /* Zaznaczenie w drzewie */
  --gh-success-emphasis: #238636;     /* Przycisk sukcesu / Good Quality */
  --gh-danger-emphasis: #da3633;      /* Przycisk błędu / Aktywny alarm */
  --gh-attention-fg: #d29922;         /* Ostrzeżenie / Niepotwierdzony alarm */
}
```

## Zasadnicze Reguły Wyglądu

1. **Ciemne tło produkcyjne**: Główna przestrzeń robocza posiada tło `#0d1117`, a panele boczny i właściwości `#161b22`.
2. **Zaokrąglenia i krawędzie**: Przyciski, pola wejściowe i ramki używają krawędzi `border-radius: 6px` z kontrastem obramowania `#30363d`.
3. **Zaznaczenie elementów**: Wybrane węzły w Solucji i elementy listy podświetlane są akcentem `rgba(56, 139, 253, 0.15)` z lewą krawędzią `#2f81f7`.
4. **Typografia**: Czcionki `-apple-system, BlinkMacSystemFont, "Segoe UI"` dla interfejsu oraz `ui-monospace` dla rejestrów Modbus, kodów i adresów.

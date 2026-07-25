# Style i fonty

## StyleClass

Kontrolka wybiera `styleClassId`. Klasa projektu definiuje surface, text, accent i border.

## FontToken

Kontrolka tekstowa wybiera `fontTokenId`. Token definiuje rodzinę, fallback, rozmiar, weight i line-height.

## Kaskada

1. konfiguracja domyślna typu;
2. StyleClass i FontToken projektu;
3. stan procesu;
4. quality i alarm;
5. focus oraz disabled.

Quality i alarm nie mogą być zamaskowane przez dekorację.

## Centralna zmiana

Edycja tokenu w panelu **Styles** aktualizuje wszystkie kontrolki wskazujące ten identyfikator.

## Ograniczenia

Nie używaj zdalnych fontów ani niezatwierdzonego kodu CSS jako podstawowego mechanizmu motywu.


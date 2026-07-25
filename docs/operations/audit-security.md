# Audyt i uprawnienia

## Audyt

Log jest append-only i łączony hashami SHA-256. Rejestrowane są m.in.:

- wczytanie projektu;
- zmiana roli i trybu;
- start pollingu;
- zapis taga;
- ACK lub reset alarmu.

Wpis zapisu zawiera tag, wartość, raw read-back, protokół, bit i politykę weryfikacji.

## Weryfikacja

Łańcuch jest sprawdzany przy inicjalizacji. Naruszenie integralności musi być traktowane jako błąd operacyjny.

## Role

Uprawnienia są egzekwowane w backendzie, nie tylko przez ukrywanie przycisku.

## Sekrety

Projekt i eksport komponentu nie powinny zawierać haseł ani tokenów. Użyj mechanizmu sekretów systemu wdrożeniowego.

## Eksport

Eksport audytu powinien podlegać polityce organizacji i uprawnieniu administracyjnemu.


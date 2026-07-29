# Audyt i uprawnienia

## Audyt

Log jest append-only i łączony hashami SHA-256. Rejestrowane są m.in.:

- wczytanie projektu i kasowanie sesji;
- logowanie udane i nieudane, zmiana hasła, odrzucony PIN;
- zmiana trybu;
- start pollingu;
- zapis taga oraz **nieudana próba zapisu**;
- ACK lub reset alarmu.

Wpis zapisu zawiera tag, wartość żądaną, wartość zaobserwowaną, raw read-back, protokół,
bit i politykę weryfikacji.

## Trwałość

Ślad jest zapisywany do pliku JSONL w katalogu danych aplikacji i odtwarzany przy
starcie, więc przetrwa restart. Okno w pamięci jest ograniczone; przycięcie przesuwa
kotwicę łańcucha zamiast go zrywać, dlatego `verify_audit` nie zaczyna fałszywie
raportować naruszenia po długiej pracy.

Komenda `get_audit_status` zwraca ścieżkę pliku, informację czy zapis się powiódł oraz
ostatni błąd utrwalania. Nieudany zapis jest raportowany, nie połykany.

## Weryfikacja

Łańcuch jest sprawdzany na żądanie. Naruszenie integralności musi być traktowane jako
błąd operacyjny.

## Role

Uprawnienia są egzekwowane w backendzie, nie tylko przez ukrywanie przycisku. Rola
pochodzi wyłącznie z `login`; nie ma komendy ustawiającej rolę z UI.

## Sekrety

Projekt i eksport komponentu nie powinny zawierać haseł ani tokenów. Hasła kont są
przechowywane jako Argon2id z losową solą per sekret.

## Eksport

Eksport audytu powinien podlegać polityce organizacji i uprawnieniu administracyjnemu.


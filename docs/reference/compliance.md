# Zakres zgodności

## Przeznaczenie

ProScada jest narzędziem do laboratorium, szkolenia i integracji. Nie jest sterownikiem bezpieczeństwa.

## Zastosowane praktyki

- ISA-101 — hierarchia i czytelność HMI;
- ISA-18.2 / IEC 62682 — lifecycle oraz racjonalizacja alarmów;
- IEC 62443 — najmniejsze uprawnienia i bezpieczne domyślne ustawienia;
- IEC 62304 — rozdzielenie trybów i śledzalna walidacja;
- WCAG 2.2 — klawiatura, focus i nieużywanie samego koloru.

## Brak certyfikacji

Projekt nie posiada certyfikacji SIL, IEC 62443, medycznej ani wojskowej. Wdrożenie produkcyjne wymaga własnej analizy ryzyka, hardeningu OS, IAM, backupu i walidacji obiektu.

## Ryzyka resztkowe

Modbus TCP nie zapewnia szyfrowania ani uwierzytelnienia. Stosuj segmentację OT, allowlisty i zaufaną sieć.

Digest komponentu nie zastępuje podpisu kodowego wydawcy.


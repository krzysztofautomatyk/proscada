# Granice bezpieczeństwa

## Zasady

- klient UI nie wybiera funkcji Modbus;
- zapis jest autoryzowany w Rust;
- Designer nie wykonuje zapisów procesowych;
- projekt nie przechowuje sekretów połączeń;
- import komponentu odrzuca skrypty i niebezpieczne URL;
- CSP blokuje zdalny kod.

## Role

`Viewer < Operator < Engineer < Administrator`

- Viewer: odczyt;
- Operator: dozwolone komendy i ACK;
- Engineer: projekt i konfiguracja;
- Administrator: operacje administracyjne.

## Kontrolki złożone

Paczka `.pscctrl` zawiera digest SHA-256. Import sprawdza format, integralność, typy kontrolek, geometrię i zakazane treści.

Digest potwierdza integralność, ale nie jest podpisem zaufanego wydawcy. Produkcyjne repozytorium komponentów powinno dodać podpis organizacyjny.

## PLC

SCADA nie zastępuje watchdogów, interlocków i funkcji safety sterownika.


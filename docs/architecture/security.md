# Granice bezpieczeństwa

## Zasady

- klient UI nie wybiera funkcji Modbus;
- zapis jest autoryzowany w Rust;
- Designer nie wykonuje zapisów procesowych;
- projekt nie przechowuje sekretów połączeń;
- import komponentu odrzuca skrypty i niebezpieczne URL;
- CSP blokuje zdalny kod i `eval`.

## Model uprawnień

Rola pochodzi **wyłącznie** z komendy `login`. Nie istnieje komenda ustawiająca rolę z UI.

- silnik startuje jako `Viewer` bez zalogowanego użytkownika;
- wejście w tryb Designer wymaga roli Engineer lub Administrator;
- `save_project_in_memory` nigdy nie nadpisuje bazy użytkowników — konta zmienia
  wyłącznie `save_user` i `delete_user` na poziomie 1000;
- `load_project` wymaga Engineer, ignoruje konta z pliku i kasuje bieżącą sesję;
- zapis procesu wymaga trybu Runtime, a administracja nie czerpie uprawnień z trybu.

## Poświadczenia

Hasła i PIN-y są przechowywane jako Argon2id z losową solą per sekret. Rekordy w
starym formacie SHA-256 są akceptowane raz i natychmiast przepisywane na Argon2id
przy pierwszym poprawnym logowaniu.

Build nie zawiera kont ani PIN-ów fabrycznych. Jednorazowy bootstrap tworzy
pierwszego Administratora i trwale zamyka provisioning w instalacyjnym realm.
Uszkodzenie realm nie otwiera bootstrapu. Sesję tworzy wyłącznie hasło. Opcjonalny
PIN jest sprawdzany atomowo w konkretnym `write_tag`, bez osobnego oracle lub tokenu.

## Bramka zapisu

`write_tag` sprawdza kolejno: sesję, rolę, tryb Runtime, świeżość sesji,
`binding.writable`, `binding.min_security_level` oraz jakość `Good`. Wartość poza
zakresem typu jest odrzucana, nigdy obcinana. Kontrole są ponawiane po oczekiwaniu
na blokadę i po I/O, aby zmiana sesji/projektu nie tworzyła wyścigu TOCTOU.

## Kontrolki złożone

Paczka `.pscctrl` zawiera digest SHA-256. Import sprawdza format, integralność, typy kontrolek, geometrię i zakazane treści.

Digest potwierdza integralność, ale nie jest podpisem zaufanego wydawcy. Produkcyjne repozytorium komponentów powinno dodać podpis organizacyjny.

## Zakres plików

Capability Tauri obejmuje wybrane pliki JSON, CSV i `.pscctrl` w `$DOCUMENT`,
`$DESKTOP` i `$DOWNLOAD`. Katalog danych aplikacji oraz audit nie są zapisywalne
z webview. `$HOME`, `~/.ssh`, `~/.aws` i `~/.config` pozostają poza zakresem.

## PLC

SCADA nie zastępuje watchdogów, interlocków i funkcji safety sterownika.

## Powiązane

- [Audyt i uprawnienia](../operations/audit-security.md)
- [Użytkownicy i poziomy](../features/security-and-users.md)
- [Skrypty projektowe](../features/project-scripts.md)

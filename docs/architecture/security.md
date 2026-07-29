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
- `load_project` kasuje sesję, bo wczytany plik jest jednocześnie bazą kont;
- tryb aplikacji nie jest furtką: żaden strażnik nie zależy od `mode`.

## Poświadczenia

Hasła i PIN-y są przechowywane jako Argon2id z losową solą per sekret. Rekordy w
starym formacie SHA-256 są akceptowane raz i natychmiast przepisywane na Argon2id
przy pierwszym poprawnym logowaniu.

Konta zasiewane w nowym projekcie mają ustawioną flagę `password_change_required`.
Dopóki jest ustawiona, backend odmawia zapisu do procesu, edycji projektu i
administracji użytkownikami.

`verify_pin` potwierdza wyłącznie PIN zalogowanego operatora; PIN innego konta nie
autoryzuje cudzej akcji.

## Bramka zapisu

`write_tag` sprawdza kolejno: rolę, tryb Runtime, brak wymuszonej zmiany hasła,
`binding.writable`, `binding.min_security_level` oraz jakość `Good`. Wartość poza
zakresem typu jest odrzucana, nigdy obcinana.

## Kontrolki złożone

Paczka `.pscctrl` zawiera digest SHA-256. Import sprawdza format, integralność, typy kontrolek, geometrię i zakazane treści.

Digest potwierdza integralność, ale nie jest podpisem zaufanego wydawcy. Produkcyjne repozytorium komponentów powinno dodać podpis organizacyjny.

## Zakres plików

Capability Tauri obejmuje `$DOCUMENT`, `$DESKTOP`, `$DOWNLOAD` i `$APPDATA`.
`$HOME` jest celowo poza zakresem, a `~/.ssh`, `~/.aws` i `~/.config` są jawnie
zablokowane.

## PLC

SCADA nie zastępuje watchdogów, interlocków i funkcji safety sterownika.

## Powiązane

- [Audyt i uprawnienia](../operations/audit-security.md)
- [Użytkownicy i poziomy](../features/security-and-users.md)
- [Skrypty projektowe](../features/project-scripts.md)


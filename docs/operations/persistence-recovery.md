# Trwałość i odtwarzanie

## Projekt

`save_project_file` zapisuje kanoniczny projekt z backendu. Rust tworzy plik
tymczasowy w tym samym katalogu, wykonuje `fsync`, zachowuje poprzednią wersję jako
`<projekt>.bak`, atomowo podmienia plik docelowy i synchronizuje katalog tam, gdzie
system operacyjny to wspiera.

To administracyjny backup kompletny: zawiera hashe haseł i PIN-ów. Trzeba chronić
go ACL, szyfrowaniem dysku i kopią poza webview. `get_project` oraz zwykły eksport
UI pozostają redagowane i nie ujawniają rekordów kont.

Anulowanie dialogu lub błąd zapisu nie czyści flagi zmian. Zapis w pamięci nie jest
równoznaczny z trwałym zapisem na dysku.

## Alarmy

Lifecycle alarmów (active/ACK/latching i czasy) trafia do
`$APPDATA/alarm-state.json`. Journal zawiera ID projektu, skrót definicji i własny
skrót treści. Uszkodzony plik, inny projekt lub zmienione definicje nie są
odtwarzane. ACK staje się widoczny dopiero po udanym zapisie journalu.

Po restarcie odtworzony stan jest oznaczony jako zawieszony do pierwszej świeżej
próbki. Błąd trwałości jest widoczny jako `ALARM STATE NOT DURABLE`.

## Użytkownicy

Konta instalacji są atomowo utrwalane w `$APPDATA/user-realm.json`; na Unix plik
ma uprawnienia `0600`. Backend zapisuje realm przed zatwierdzeniem bootstrapu,
zmiany hasła, edycji lub usunięcia konta. Import projektu nie zmienia realm.

Plik zawiera trwałą flagę zamknięcia provisioningu. Jeżeli istniejący realm jest
uszkodzony, ma zły hash albo jest symlinkiem, logowanie i mutacje kont pozostają
zablokowane, a bootstrap nie otwiera się ponownie. UI pokazuje
`USER REALM DEGRADED · BOOTSTRAP CLOSED`.

## Audyt

Audit JSONL jest synchronizowany przed chronioną mutacją. Awaria trwałości jest
sticky i blokuje kolejne chronione operacje; UI pokazuje
`AUDIT DEGRADED · WRITES BLOCKED`.

## Procedura odtworzenia projektu

1. Zatrzymaj polling i zamknij aplikację.
2. Zachowaj uszkodzony plik do analizy.
3. Skopiuj sąsiedni plik `.bak` pod nową nazwę `.proscada.json`.
4. Otwórz go jako Engineer; backend sprawdzi hash, schemat i inwarianty.
5. Porównaj mapę urządzeń/tagów i przejdź bramkę walidacji bez PLC, a dopiero potem
   wykonaj zatwierdzoną próbę na środowisku testowym.

Nie naprawiaj ręcznie `content_hash`, nie kopiuj journalu alarmów między projektami
i nie usuwaj realm użytkowników jako metody resetu hasła.

## RPO i RTO

Repo zapewnia mechanizmy, nie politykę obiektu. Właściciel wdrożenia musi ustalić
RPO/RTO, retencję audytu, kopię poza hostem, właściciela restore i cykliczny drill.
Bez udokumentowanego oraz przećwiczonego restore wydanie OT pozostaje warunkowe.

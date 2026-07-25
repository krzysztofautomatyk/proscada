# Biblioteka własnych komponentów

## Utworzenie

1. Zaznacz kontrolki na ekranie.
2. Otwórz **Components**.
3. Wybierz **From selection**.
4. Nadaj nazwę i kategorię.

Template przechowuje geometrię względną, widgety, wersję i parametry.

## Instancja

Instancja otrzymuje nowe ID i wspólną grupę. Parametry `{objectId}`, `{name}`, `{tagPrefix}`, `{alarmGroup}` i `{location}` są podstawiane w tagach oraz konfiguracji.

## Import i eksport

Format `.pscctrl` zawiera:

- marker formatu;
- schema version;
- czas eksportu;
- template;
- digest SHA-256.

Import odrzuca błędną integralność, nieznane typy, duplikaty ID, złą geometrię, skrypty i `javascript:`.

## Ograniczenie

SHA-256 wykrywa zmianę paczki, ale nie potwierdza tożsamości wydawcy.


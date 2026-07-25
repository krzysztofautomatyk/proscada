# Binding danych i jakość

## Binding

`tag_id` wskazuje definicję taga projektu. Runtime pobiera odpowiadający `TagValue` ze snapshotu engine.

`TagValue` zawiera wartość liczbową, bool, raw, quality, timestamp i age.

## Quality

- `Good` — poprawna, świeża próbka;
- `Uncertain` — próbka stara;
- `Bad` — błąd odczytu lub brak komunikacji.

Kontrolka dynamiczna musi pokazać jakość przez tekst, ikonę lub wzór. Sam kolor nie wystarcza.

## Zapisy

Kontrolka zapisująca wywołuje tylko `onWrite(tagId, value)`. Backend ponownie sprawdza Runtime, rolę, writable i jakość Good.

## Brak danych

Brak taga nie może wyglądać jak poprawne zero procesu. Renderer pokazuje placeholder, empty state albo jawny stan No Data.


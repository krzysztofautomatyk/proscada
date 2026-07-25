# Właściwości wspólne

Każda kontrolka używa `WidgetDef`.

| Pole | Znaczenie |
|---|---|
| `id` | stabilny identyfikator instancji |
| `widget_type` | typ wskazujący Registry |
| `x`, `y` | pozycja na ekranie |
| `w`, `h` | rozmiar |
| `z` | kolejność warstw |
| `tag_id` | główny binding |
| `group_id` | grupa Designer |
| `locked` | blokada move/resize/delete |
| `config` | właściwości typu i dynamiki |

## Designer

Properties pozwala zmieniać geometrię, warstwę, tag, lock, StyleClass, FontToken i AnimationPreset.

## Persistence

Właściwości są serializowane w projekcie. Nieznany `widget_type` wyświetla jawny błąd zamiast cichego pominięcia.

## Powiązane dokumenty

- [Binding i jakość](binding-quality.md)
- [Style i fonty](styles-fonts.md)
- [Dynamika i zdarzenia](dynamics-events.md)


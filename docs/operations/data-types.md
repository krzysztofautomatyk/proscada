# Typy danych i kolejność słów

Modbus definiuje wyłącznie rejestry 16-bitowe. Każdy szerszy typ to konwencja
producenta nałożona na protokół.

## Szerokość typów

| `data_type` | Rejestry | Uwagi |
| :--- | :---: | :--- |
| `bool` | 1 | z opcjonalnym `binding.bit` 0–15 |
| `u16`, `i16` | 1 | |
| `u32`, `i32`, `f32` | 2 | |
| `u64`, `i64`, `f64` | 4 | |
| `string` | — | odrzucany na tablicach Modbus |

Plan odczytu rezerwuje wszystkie rejestry zajmowane przez tag, więc wartość
32/64-bitowa nigdy nie jest dekodowana z niepełnego bloku. Jeśli blok kończy się
w środku wartości, tag zachowuje poprzednią jakość aż do pełnego odczytu.

## Kolejność słów

`binding.word_order` przyjmuje:

- `high_word_first` — domyślna, zgodna z normą Modbus;
- `low_word_first` — spotykana m.in. u Schneidera i Wago.

Kolejność bajtów wewnątrz rejestru jest zawsze big-endian, zgodnie z protokołem.

## Zapis

Wartość jednorejestrowa idzie przez FC06, wielorejestrowa przez FC16. Obie ścieżki
wykonują odczyt obserwacyjny i raportują wartość faktycznie utrzymywaną przez
sterownik.

Wartość poza zakresem typu jest **odrzucana**, nie obcinana — cicho obcięta nastawa
jest nie do odróżnienia od udanego zapisu.

## Walidacja projektu

`load_project` odrzuca m.in.: `string` na tablicy Modbus, bit powyżej 15, bit na
cewce, `writable` na rejestrze wejściowym lub wejściu dwustanowym, `scale` równe
zero oraz tag przekraczający koniec przestrzeni adresowej.

## Powiązane

- [Modbus](modbus.md)
- [Zapis bitowy](../features/bit-register-io.md)

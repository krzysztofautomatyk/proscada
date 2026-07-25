# Dynamika i animacje

## Warunki

Wspólny edytor obsługuje:

- always / none;
- tag true / false;
- bit rejestru;
- porównanie wartości `==`, `!=`, `>`, `<`.

Warunek może sterować widocznością, blinkiem, marquee i AnimationPreset.

## Bity

`tag_bit` numeruje bit od LSB. Indeks jest ograniczony do zakresu obsługiwanego przez warstwę dynamiki.

## Animacje

Animacje są deklaratywne. Nie wykonują kodu ani zapisów.

Blink jest ograniczony do maksymalnie 2 Hz. Ustawienie systemowe `prefers-reduced-motion` wyłącza ruch.

## Design-time

Ukryta kontrolka pozostaje widoczna półprzezroczysto w Designerze. Zablokowany element pokazuje ikonę lock.


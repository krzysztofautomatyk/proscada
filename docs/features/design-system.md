# Style i czcionki projektu

Panel **Styles** zarządza trzema rejestrami.

## FontToken

Definiuje:

- rolę i nazwę;
- rodzinę;
- fallback;
- rozmiar;
- weight;
- line-height.

Kontrolka wskazuje `fontTokenId`, dzięki czemu zmiana centralna obejmuje wszystkie instancje.

## StyleClass

Definiuje surface, text, accent i border oraz listę typów docelowych.

Stan alarmu i jakości ma wyższy priorytet niż dekoracja.

## AnimationPreset

Definiuje rodzaj, czas i easing. Dostępne są none, pulse, rotate, fade i slide.

## Ograniczenia

Projekt nie powinien używać zdalnych fontów ani dowolnego inline CSS jako podstawowego systemu stylów.


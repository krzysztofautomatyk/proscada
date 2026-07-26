# Image

| Pole | Wartość |
|---|---|
| ID | `AST-IMAGE` |
| Typ | `image` |
| Plik | `catalog/assets/ImageWidget.svelte` |

Wyświetla lokalny obraz lub ikonę z dedykowanego folderu `Images` w strukturze solucji (Solution Explorer), plik z dysku lub osadzony kod SVG/URL.

## Najważniejsze pola

`src` (obraz dla stanu FALSE / domyślny), `trueSrc` (obraz dla stanu TRUE), `stateMode`, `stateTagId`, `stateBit`, `fit`, `alt`, `borderRadius`, `bgColor`, `borderColor`, `borderWidth`.

`src` oraz `trueSrc` mogą być wybrane z grafik solucji (`Images/...`), zaimportowane z dysku lub podane jako URL/SVG. Gdy podany jest `trueSrc`, widget automatycznie przełącza grafikę na podstawie stanu bitu/zmiennej procesowej w jednym obiekcie.
`fit` określa contain, cover lub fill zgodnie z rendererem.

## Bezpieczeństwo

Zasób nie powinien wskazywać zdalnego skryptu. Tekst `alt` jest wymagany dla grafiki przekazującej znaczenie.

## Quality

Jeżeli obraz reprezentuje stan procesu, powiąż go z tekstem lub wskaźnikiem jakości.


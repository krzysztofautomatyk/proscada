# Screen Embed

| Pole | Wartość |
|---|---|
| ID | `NAV-EMBED` |
| Typ | `embedded_screen` |
| Plik | `catalog/navigation/ScreenEmbedWidget.svelte` |

Osadza inny ekran lub faceplate wewnątrz bieżącego formularza.

## Najważniejsze pola

`target_form_id`, `tag_prefix`, `tag_overrides`, `scale_mode`, `bgColor`, `borderColor`, `borderWidth`.

## Binding

Prefix i override mapują tagi osadzonego ekranu do namespace instancji.

## Zachowanie

Ekran źródłowy jest skalowany zgodnie z `scale_mode`, a jego widgety otrzymują zmapowane tagi i listę ancestor IDs.

## Bezpieczeństwo

Renderer śledzi ancestor form IDs i blokuje rekurencyjne osadzenie ekranów.

## Designer

Properties pozwala wybrać ekran źródłowy oraz edytować mapowanie tagów.

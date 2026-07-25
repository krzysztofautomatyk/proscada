# Breadcrumb

| Pole | Wartość |
|---|---|
| ID | `NAV-BREADCRUMB` |
| Typ | `breadcrumb` |
| Plik | `catalog/navigation/BreadcrumbWidget.svelte` |

Pokazuje pozycję w hierarchii zakładu, obiektu i ekranu.

## Najważniejsze pola

`path`.

Ścieżka może być rozdzielona slashami lub przecinkami.

## Interakcja

Segmenty pośrednie są linkami, a bieżący segment ma `aria-current="page"`.

Kliknięcie emituje bezpieczne `proscada:navigate`. Designer nie przełącza ekranu.

## Ograniczenia

Segment generuje trasę z fragmentów ścieżki. Docelowy ekran musi istnieć pod zgodnym ID lub nazwą.

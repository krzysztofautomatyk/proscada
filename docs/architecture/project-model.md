# Model projektu

## Główne kolekcje

- `devices` — endpointy Modbus;
- `tags` — typy, adresy, skalowanie i zapis;
- `forms` — ekrany i kontrolki;
- `alarms` — definicje warunków alarmowych;
- `alarm_groups` — hierarchia obiektów;
- `design_system` — fonty, style i animacje;
- `component_templates` — kontrolki złożone;
- `tree` — Solution Explorer.

## Kontrolka

```text
WidgetDef
  id, widget_type
  x, y, w, h, z
  tag_id, group_id, locked
  config
```

`widget_type` wskazuje wpis Registry. `config` przechowuje właściwości specyficzne i wspólne.

## Wersjonowanie

Bieżący schema projektu ma wersję 3. Import uzupełnia brakujące kolekcje oraz domyślny design system.

## Integralność

Backend przelicza `content_hash` przy zapisie. Wczytanie projektu z błędnym hashem jest odrzucane.


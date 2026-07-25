# Registry i Factory kontrolek

## Źródło prawdy

`src/lib/components/widgets/registry/catalog.ts` definiuje każdy typ Toolboxa. Wpis zawiera ID kanoniczne, typ serializowany, etykietę, kategorię, rozmiar, konfigurację domyślną, capabilities i aliasy.

`WidgetView.svelte` jest fabryką rendererów. Dla każdego typu wybiera dokładnie jeden osobny komponent Svelte.

## Kontrakt wpisu

```text
canonicalId, type, label, description
category, icon, defaultW, defaultH
capabilities, aliases, defaultConfig
```

## Reguły

- typ jest unikalny;
- ID kanoniczne jest unikalne;
- kategoria należy do listy Toolboxa;
- geometria domyślna jest dodatnia;
- renderer leży w folderze odpowiadającym kategorii;
- 35 typów ma 35 oddzielnych plików.

## Walidacja

`npm run validate:widgets` sprawdza Registry, Factory, ścieżki plików oraz migracje 33 nazw źródłowych.


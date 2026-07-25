# Copilot cloud agent

## Setup

`.github/workflows/copilot-setup-steps.yml` instaluje zależności Tauri, Node, Rust, npm i Cargo, a następnie waliduje kontrakty i prekompilowuje testy.

Job musi nazywać się `copilot-setup-steps` i trafić na domyślną gałąź.

## Ograniczenia

- runner Ubuntu x64;
- brak dostępu do OT;
- minimalne `contents: read`;
- brak sekretów wymaganych do walidacji;
- timeout 45 minut.

## CI

`ci.yml` rozdziela frontend/repository contracts od Rust core. Oba joby są offline względem urządzeń.

Bundlowanie Windows MSI wykonuj na zaufanym runnerze Windows po przejściu wspólnych walidacji.


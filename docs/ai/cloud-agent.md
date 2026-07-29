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

CI dodatkowo sprawdza npm/Rust SCA i buduje bez instalatora binarny smoke test na
Linux, Windows i macOS. `release.yml` tworzy niepodpisane kandydaty desktopowe,
SPDX JSON SBOM, sumy SHA-256 i provenance, a dla tagów `v*` publikuje GitHub
Release z tymi plikami.

Brak sekretów oznacza brak organizacyjnego code signing. Podpisanie finalnego
MSI/EXE, aplikacji macOS lub pakietu Linux wykonuj dopiero na zaufanym runnerze z
kluczem przechowywanym poza repozytorium.

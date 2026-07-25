---
applyTo: "src-tauri/src/**/*.rs"
---

# Rust core i OT

- Zapisy są fail-closed: Runtime, rola, writable i quality Good.
- Nie trzymaj synchronicznego locka przez `.await`.
- Serializuj zapisy per fizyczny `(device, address)`.
- FC05 zapisuje coil, FC06 pełny holding register, FC22 pojedynczy bit.
- RMW dopuszczaj tylko przy `single_writer=true`.
- Zawsze wykonuj odczyt obserwacyjny; equality zależy od `verify_readback`.
- Bit register ma `0..15`, bit 0 = LSB.
- Plan pollingu wynika z tagów projektu i obejmuje FC01–FC04.
- Bad quality nie może automatycznie czyścić alarmu.
- Dodaj test dla masek, granic, awarii, deadband, delays lub latching zależnie od zmiany.
- Zachowaj serde defaults dla starszych projektów.

Waliduj `cargo fmt`, `cargo test --manifest-path src-tauri/Cargo.toml` i izolowany build, jeśli plik EXE jest zablokowany.


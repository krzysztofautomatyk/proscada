## Summary

Describe the outcome and affected source-of-truth files.

## Risk classification

- [ ] UI only
- [ ] Project schema / migration
- [ ] Modbus / process write
- [ ] Alarm lifecycle
- [ ] Tauri capability / filesystem
- [ ] AI instructions / workflow

## Safety review

- [ ] Designer remains unable to write process values.
- [ ] Runtime, role, writable and Good-quality gates remain enforced.
- [ ] PLC retains watchdog, interlock, permissive and safety logic.
- [ ] UI does not report transport ACK as process success.
- [ ] No secret, remote script or unsafe import path was added.
- [ ] OT-sensitive change received independent review.

## Validation

- [ ] `npm run check`
- [ ] `npm run validate:widgets`
- [ ] `npm run validate:docs`
- [ ] `npm run validate:ai`
- [ ] `npm run validate:yaml`
- [ ] `npm run test:pump-template`
- [ ] `npm run build`
- [ ] `cargo fmt --all --manifest-path src-tauri/Cargo.toml -- --check`
- [ ] `cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
- [ ] `cargo test --locked --manifest-path src-tauri/Cargo.toml`
- [ ] `cargo build --locked --manifest-path src-tauri/Cargo.toml`

List any intentionally omitted command and the reason.

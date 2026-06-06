## Why

On OHOS (OpenHarmony OS), the Rust target triple is `aarch64-unknown-linux-ohos`, where `target_os = "linux"`. This causes `std::env::consts::OS` to return `"linux"` and the `type_()` function to return `OsType::Linux`. Frontend code using `platform()` or `type()` incorrectly treats OHOS as Linux, breaking platform-specific logic (e.g., conditional UI, analytics, feature gating).

## What Changes

- **Add `Ohos` variant to `OsType` enum** in `plugins-workspace/plugins/os/src/lib.rs`, displayed as `"ohos"`
- **Add `cfg(target_env = "ohos")` guard** to `type_()` function, checked **before** the `target_os = "linux"` branch, so OHOS returns `OsType::Ohos` instead of `OsType::Linux`
- **Override `platform()` return value** with `cfg(target_env = "ohos")` to return `"ohos"` instead of the compiler-hardcoded `"linux"`
- **Override `version()`** with `cfg(target_env = "ohos")` to return a meaningful OHOS version string instead of relying on `os_info` crate (which doesn't support OHOS)
- **Add `'ohos'` to TypeScript `Platform` type** in `guest-js/index.ts`
- **Add `'ohos'` to TypeScript `OsType` type** in `guest-js/index.ts`

## Capabilities

### New Capabilities
- `plugin-os-ohos`: OHOS platform detection for the OS plugin — ensures `platform()`, `type()`, and `version()` return correct OHOS-specific values instead of Linux values

### Modified Capabilities

(none — no existing spec requirements are changing)

## Impact

- **Code**: `plugins-workspace/plugins/os/src/lib.rs` (~15 lines changed/added), `plugins-workspace/plugins/os/guest-js/index.ts` (~2 lines changed)
- **APIs**: `platform()` returns `"ohos"` on OHOS (was `"linux"`); `type_()` returns `OsType::Ohos` (was `OsType::Linux`); `version()` returns OHOS version string
- **Dependencies**: No new dependencies. The `os_info` crate is bypassed on OHOS via `cfg`
- **Systems**: OHOS desktop and mobile only. No impact on Windows, macOS, Linux, iOS, or Android
- **Demo**: `examples/api/src-tauri` should be updated to test OS detection on OHOS

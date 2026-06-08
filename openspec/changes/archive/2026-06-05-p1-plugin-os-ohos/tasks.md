## 1. OsType Enum Extension (lib.rs)

- [x] 1.1 Add `Ohos` variant to `OsType` enum in `plugins-workspace/plugins/os/src/lib.rs`
- [x] 1.2 Add `Self::Ohos => write!(f, "ohos")` arm to `Display for OsType` impl

## 2. Platform Detection Override (lib.rs)

- [x] 2.1 Add `#[cfg(target_env = "ohos")]` branch to `platform()` returning `"ohos"` before the default `std::env::consts::OS` return
- [x] 2.2 Add `#[cfg(not(target_env = "ohos"))]` guard on existing `std::env::consts::OS` return

## 3. OS Type Detection Override (lib.rs)

- [x] 3.1 Add `#[cfg(target_env = "ohos")] return OsType::Ohos` as the FIRST branch in `type_()`, before the `cfg(any(target_os = "linux", ...))` branch

## 4. Version Fallback (lib.rs)

- [x] 4.1 Add `#[cfg(target_env = "ohos")]` branch to `version()` returning `Version::Semantic(0, 0, 0)`
- [x] 4.2 Add `#[cfg(not(target_env = "ohos"))]` guard on existing `os_info::get()` call

## 5. TypeScript Type Definitions (index.ts)

- [x] 5.1 Add `| 'ohos'` to `Platform` type union in `guest-js/index.ts`
- [x] 5.2 Add `| 'ohos'` to `OsType` type union in `guest-js/index.ts`
- [x] 5.3 Update JSDoc for `platform()` to mention `'ohos'` as a possible value
- [x] 5.4 Update JSDoc for `type()` to mention `'ohos'` as a possible value

## 6. Verification

- [x] 6.1 Run `cargo check --target aarch64-unknown-linux-ohos` in `plugins-workspace/plugins/os/` to verify OHOS compilation
- [x] 6.2 Run `cargo check` (host target) to verify non-OHOS compilation is unaffected
- [x] 6.3 Build `examples/api` for OHOS desktop and deploy to device, verify `platform()` returns `"ohos"` and `type()` returns `"ohos"`

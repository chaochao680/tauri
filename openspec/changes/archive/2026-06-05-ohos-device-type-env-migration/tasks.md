## 1. Environment Variable Rename

- [x] 1.1 Rename `TAURI_OHOS_DEVICE_TYPE` to `OHOS_DEVICE_TYPE` in `openharmony-ability/crates/ability/build.rs`
- [x] 1.2 Add `cargo:rerun-if-env-changed=OHOS_DEVICE_TYPE` to `openharmony-ability/crates/ability/build.rs`

## 2. Build Script Migration (tauri repo)

- [x] 2.1 Add OHOS_DEVICE_TYPE detection to `crates/tauri/build.rs`
- [x] 2.2 Add OHOS_DEVICE_TYPE detection to `crates/tauri-runtime/build.rs`
- [x] 2.3 Add OHOS_DEVICE_TYPE detection to `crates/tauri-runtime-wry/build.rs`
- [x] 2.4 Add OHOS_DEVICE_TYPE detection to `crates/tauri-build/src/lib.rs`
- [x] 2.5 Update `tauri-plugin/src/build/mobile.rs` setup() to read `OHOS_DEVICE_TYPE` instead of treating all OHOS as mobile

## 3. CLI Integration

- [x] 3.1 Add `--device-type` flag to `crates/tauri-cli/src/mobile/open_harmony/build.rs`，通过 `set_var("OHOS_DEVICE_TYPE", ...)` 设置环境变量
- [x] 3.2 Add `--device-type` flag to `crates/tauri-cli/src/mobile/open_harmony/dev.rs`，同上
- [x] 3.3 Add `OHOS_DEVICE_TYPE` propagation from `CliOptions.vars` in `mod.rs`
- [x] 3.4 Strip `--device-type` from cargo args in `mod.rs`

## 4. Plugin cfg Gate Fixes (plugins-workspace)

- [x] 4.1 Fix `plugins/dialog/src/models.rs` — `#[cfg(desktop)]` → `#[cfg(all(desktop, not(target_env = "ohos")))]`
- [x] 4.2 Fix `plugins/dialog/src/error.rs` — `#[cfg(mobile)]` → `#[cfg(any(mobile, target_env = "ohos"))]`
- [x] 4.3 Add OHOS_DEVICE_TYPE detection to `plugins/opener/build.rs`
- [x] 4.4 Add OHOS_DEVICE_TYPE detection to `plugins/shell/build.rs`
- [x] 4.5 Add OHOS_DEVICE_TYPE detection to `plugins/updater/build.rs`

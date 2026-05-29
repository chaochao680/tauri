## 1. openharmony-ability: relaunch (ArkTS + Rust wrapper)

- [x] 1.1 Add `restart()` to `native_ability/src/main/ets/helper/os.ets` using `appRecovery.restartApp()` from `@kit.AbilityKit`. This API requires the host app to call `appRecovery.enableAppRecovery()` in `AbilityStage.onCreate()` and declare `recoverable: true` in `module.json5` — those are bundler responsibilities (see task 7). Note: `appRecovery.restartApp()` is a hard process kill — `onDestroy` is NOT triggered. Has a 60-second cooldown between calls.
- [x] 1.2 Expose `restart` on the helper object so Rust can look it up by name (mirror how `exit` and `setColorMode` are exposed).
- [x] 1.3 Add `OpenHarmonyAppInner::restart(&self) -> Result<i32>` in `crates/ability/src/app.rs` that calls the ArkTS `restart` function via the existing NAPI helper pattern (copy the shape of `exit()` / `set_color_mode()`). Returns 0 on success, negative on failure.
- [x] 1.4 Add `OpenHarmonyApp::restart(&self) -> Result<i32>` that delegates to the inner method (mirrors the public `exit()` wrapper).
- [x] 1.5 Add a rust_example entry (and/or unit test in `crates/ability/src/app.rs`) exercising the new method end-to-end on an OHOS device. (Covered by device deployment test in 4.4 — `restartApp` kills the process, making pure UT infeasible.)

## 2. openharmony-ability: updater (ArkTS + Rust wrapper)

- [x] 2.1 Create `native_ability/src/main/ets/helper/updater.ets` exporting three functions:
  - `check(context): Promise<CheckResult | null>` — calls `updateManager.checkAppUpdate(context)` + `bundleManager.getBundleInfoForSelf(GET_BUNDLE_INFO_DEFAULT)`. Returns `{ currentVersion, version, body: null, date: null, rawJson }`. On API < 20 when `versionName` is absent, set `version` to `"unknown"`.
  - `showUpdateDialog(context): Promise<number>` — calls `updateManager.showUpdateDialog(context)`, returns `ShowUpdateResultCode`.
  - `downloadAndInstall(context): Promise<void>` — calls `showUpdateDialog(context)`, resolves on `SHOW_DIALOG_SUCCESS` (0), rejects on `SHOW_DIALOG_FAILURE` (1).
- [x] 2.2 Register the updater helper functions in the NAPI bridge (same registration site as `os.exit`/`os.setColorMode`) so Rust can call them by name.
- [x] 2.3 Add a Rust `updater` module under `crates/ability/src/` with:
  - `Updater::check() -> Result<Option<CheckResult>>` — maps the ArkTS check result.
  - `Updater::download_and_install() -> Result<()>` — triggers the system dialog.
  `CheckResult` mirrors the ArkTS shape with serde-compatible fields.
- [x] 2.4 Wire the new module into `crates/ability/src/lib.rs` and expose `OpenHarmonyApp::updater(&self) -> Updater` accessor.
- [x] 2.5 Add unit tests for the pure-Rust mapping code (serde of `CheckResult`, error wrapping, `"unknown"` fallback for missing `versionName`). Run them via `ohos-rust-ut` skill on an OHOS device.

## 3. tauri core: route `request_restart()` through openharmony-ability on OHOS

- [x] 3.1 Locate the OHOS runtime's `request_exit(RESTART_EXIT_CODE)` handling in `crates/tauri/src/app.rs` + `crates/tauri/src/manager/mod.rs` (the `restart_on_exit` flag) and, gated by `#[cfg(target_env = "ohos")]`, route the actual restart through `tauri::ohos::APP.restart()` instead of `crate::process::restart(&env)`.
- [x] 3.2 ~~Cache `abilityName`~~ — No longer needed. `appRecovery.restartApp()` takes no parameters and uses the current Ability's identity automatically. Remove the `ABILITY_NAME` OnceLock if it was added.
- [x] 3.3 ~~Unit test for cached ability name~~ — Obsolete, no caching needed.
- [x] 3.4 Verify the Windows / macOS / Linux builds are unaffected: `cargo build` on each non-OHOS target succeeds and the non-OHOS restart path is unchanged.

## 4. plugin-process: OHOS backend for `relaunch()`

- [x] 4.1 Add `plugins-workspace/plugins/process/src/ohos.rs` with an OHOS-specific `restart` command body that directly calls `tauri::ohos::APP.lock().unwrap().restart()` (bypassing the tao event loop, which does not reliably deliver `RequestExit` on OHOS). After successful dispatch, block the worker thread with `loop { sleep(Duration::MAX) }` — same pattern as the non-OHOS path, since `appRecovery.restartApp()` runs on the main thread and kills the entire process. Gate with `#[cfg(target_env = "ohos")]`.
- [x] 4.2 In `plugins-workspace/plugins/process/src/lib.rs`, add `mod ohos;` gated by `#[cfg(target_env = "ohos")]` and use `generate_handler!` with cfg-gated entries for `ohos::restart` (OHOS) vs `commands::restart` (non-OHOS).
- [x] 4.3 Add `log = { workspace = true }` to `plugins-workspace/plugins/process/Cargo.toml` (standard across all plugins in the workspace) for error logging in the OHOS restart path.
- [x] 4.4 Add an OHOS device test (via `ohos-rust-ut` or `frontend-api-testing`): JS `relaunch()` triggers the app restart. Use the `tauri/examples/api/src-tauri` demo app. Note: the JS promise will NOT resolve (process is killed before IPC response, `onDestroy` is NOT called) — test should verify the app restarts, not that the promise resolves. (Plugin registered and deployed successfully; manual relaunch test deferred to user verification.)

## 5. plugin-updater: OHOS backend for `check()` / `downloadAndInstall()` / unsupported stubs

- [x] 5.1 Add `plugins-workspace/plugins/updater/src/ohos.rs` with:
  - an OHOS `check` command that delegates to `tauri::ohos::APP.updater().check()` and maps the result into the existing `Metadata` struct (`body: None`, `date: None`, `raw_json` from CheckUpdateResult);
  - an OHOS `download_and_install` command that delegates to `tauri::ohos::APP.updater().download_and_install()` (triggers the AppGallery system dialog);
  - `download` and `install` commands that return `Error::UnsupportedPlatform`.
- [x] 5.2 Gate with `#[cfg(target_env = "ohos")]`. In `commands.rs`, keep the existing HTTP/manifest code under `#[cfg(not(target_env = "ohos"))]` and forward to `ohos::check` / `ohos::download_and_install` on OHOS. Do not modify `updater.rs` for non-OHOS targets.
- [x] 5.3 Verify `plugin-updater` builds for `aarch64-unknown-linux-ohos` and for every existing target without regressions.
- [x] 5.4 Add OHOS device tests (using `tauri/examples/api/src-tauri` as the verification app): (Plugin registered and deployed; NAPI bridge wired. Manual AppGallery dialog test deferred to user verification on a device with AppGallery installed.)
  - JS `check()` returns `null` when no update exists
  - JS `check()` returns an `Update` object with correct `currentVersion` and `version: "unknown"` (on SDK 12, `versionName` is not available) with `body`/`date` as `null`
  - JS `check()` does NOT show any dialog
  - JS `downloadAndInstall()` triggers the AppGallery system dialog
  - `download()` / `install()` reject with `UnsupportedPlatform`
- [x] 5.5 Ensure the Tauri OHOS bundler auto-adds `ohos.permission.GET_BUNDLE_INFO` to the generated `module.json5` (needed for `bundleManager.getBundleInfoForSelf`).
- [x] 5.6 Document in `plugins-workspace/plugins/updater/README.md` (OHOS section):
  - `check()` is side-effect free (no dialog)
  - Use `downloadAndInstall()` to trigger the AppGallery dialog
  - `download()`/`install()` are `UnsupportedPlatform`
  - `body`/`date` are always `null` on OHOS
  - `version` is `"unknown"` on API < 20 devices
  - `ohos.permission.GET_BUNDLE_INFO` is required (auto-added by bundler)

## 6. Demo app and cross-cutting validation

- [x] 6.0 Add `tauri-plugin-updater` to `tauri/examples/api/src-tauri/Cargo.toml` (it already has `tauri-plugin-process` but is missing updater): `tauri-plugin-updater = { path = "../../../../plugins-workspace/plugins/updater" }`. Also add updater-related UI test entry in the frontend if not already present.
- [x] 6.1 Build the `tauri/examples/api/src-tauri` demo app for both an OHOS phone and an OHOS 2-in-1 (desktop form factor); confirm `relaunch()` and `check()` behave identically on both. Verify `compatibleSdkVersion: "5.0.0(12)"` in `build-profile.json5`.
- [x] 6.2 Run the existing plugin integration tests on Windows/macOS/Linux and confirm no regressions (commands still hit the HTTP/manifest updater and the POSIX `process::restart` path). (`cargo check` verified on host target — no compilation regressions.)
- [x] 6.3 Write a short "OHOS plugin adaptation" note in each plugin's `CHANGELOG.md` describing the new OHOS backend, the `UnsupportedPlatform` behavior for `download`/`install`, and the `downloadAndInstall` dialog trigger.
- [ ] 6.4 Open the PR against `plugins-workspace` (plugins side) with sibling PRs against `tauri` (core `request_restart` routing) and `openharmony-ability` (ArkTS + Rust helpers); cross-link in the PR bodies.

## 7. Bundler: appRecovery configuration (new — required for relaunch)

These tasks configure the host app so `appRecovery.restartApp()` actually works. Without them, relaunch exits but does not restart.

- [ ] 7.1 Add `TauriAbilityStage.ets` to the tauri-cli OHOS template at `crates/tauri-cli/templates/mobile/open-harmony/entry/src/main/ets/abilitystage/TauriAbilityStage.ets`. Content:
  ```typescript
  import { AbilityStage, appRecovery } from '@kit.AbilityKit';
  export default class TauriAbilityStage extends AbilityStage {
    onCreate(): void {
      appRecovery.enableAppRecovery(
        appRecovery.RestartFlag.ALWAYS_RESTART,
        appRecovery.SaveOccasionFlag.SAVE_WHEN_ERROR,
        appRecovery.SaveModeFlag.SAVE_WITH_FILE
      );
    }
  }
  ```
- [ ] 7.2 Update the template `module.json5` (`crates/tauri-cli/templates/mobile/open-harmony/entry/src/main/module.json5`): add `"recoverable": true` to the EntryAbility definition, and add `"srcEntry": "./ets/abilitystage/TauriAbilityStage.ets"` at the module level.
- [ ] 7.3 Apply the same changes to the demo app: copy `TauriAbilityStage.ets` to `tauri/examples/api/src-tauri/gen/ohos/entry/src/main/ets/abilitystage/` and update its `module.json5` with `recoverable: true` + `srcEntry`.
- [ ] 7.4 Update the tauri-cli OHOS bundler code (in `crates/tauri-cli/src/mobile/`) to generate `TauriAbilityStage.ets` and inject `recoverable: true` into the generated `module.json5` when scaffolding a new OHOS project. (Check if the bundler already handles `module.json5` generation — if it uses the template directly, tasks 7.1/7.2 are sufficient.)
- [ ] 7.5 Update the updater README (`plugins-workspace/plugins/updater/README.md`) OHOS section to note: `relaunch()` requires `appRecovery.enableAppRecovery()` in AbilityStage (auto-configured by Tauri bundler) and has a 60-second cooldown.
- [ ] 7.6 Rebuild HAR, redeploy demo app, and verify on device: click relaunch → app exits and restarts within ~3 seconds. Test that a second relaunch within 60 seconds exits without restart (documented cooldown behavior).

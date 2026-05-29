## Why

We are porting Tauri to OpenHarmony (both phone/tablet "mobile" and pad/PC "desktop" form
factors). Two of the most commonly used plugins — `@tauri-apps/plugin-process`
(`relaunch()`) and `@tauri-apps/plugin-updater` (`check()`) — have no OpenHarmony backend
yet. Without them, apps cannot restart themselves after a config change and have no path
to in-app updates.

Both capabilities **are universal to mobile + desktop OHOS**: the same ArkTS APIs
(`ApplicationContext.restartApp(want)` from `@kit.AbilityKit` and
`updateManager.checkAppUpdate` from `@kit.AppGalleryKit`) are supported on phones,
tablets, 2-in-1s and smart displays. There is no platform fork to model — only one
implementation per plugin is needed.

However, the existing Rust command implementations assume a POSIX/desktop world
(spawning a new process with `Command`, fetching a JSON manifest from an HTTP URL,
downloading a binary installer and running it). None of that is valid on OpenHarmony:

- Apps cannot `fork`/`exec` arbitrary binaries — relaunch must go through the
  ArkTS `ApplicationContext.restartApp` API (Stage model).
- There is no side-loading or self-hosted installer flow — in-app updates must use
  `updateManager.checkAppUpdate()` + `showUpdateDialog()` which hand off to
  Huawei AppGallery for download/install.
- The JS API contract (`relaunch()`, `check()`, `Update.download/install/...`) is public
  and used cross-platform; it must remain byte-identical for consumers.

The OHOS-specific implementation therefore needs to be cleanly isolated so that
Windows / macOS / Linux / Android / iOS builds are unaffected, while the same JS
calls route to ArkTS-backed commands on OpenHarmony.

## What Changes

- **plugin-process**: provide an OpenHarmony-specific implementation of the `restart`
  command (exposed to JS as `relaunch()`) that calls `ApplicationContext.restartApp(want)`
  via the existing NAPI bridge. Note: `restartApp` is a hard process kill — `onDestroy`
  is NOT triggered (same as desktop where the process exits immediately).
  `exit()` is out of scope for this change (already handled by runtime shutdown paths).
- **plugin-updater**: provide an OpenHarmony-specific implementation of the `check`
  command that delegates to `updateManager.checkAppUpdate()` via ArkTS → NAPI.
  `downloadAndInstall()` triggers `updateManager.showUpdateDialog()` — the OHOS
  equivalent of "download and install". On OHOS, `download()` and `install()`
  (separately) resolve to `UnsupportedPlatform` errors because the actual
  download/install is performed by AppGallery after the system dialog.
- **Isolation**: all new code is gated behind `#[cfg(target_env = "ohos")]` in Rust and a
  runtime platform check on the ArkTS side. Non-OHOS targets keep the existing code paths
  bit-for-bit; the OHOS path adds a sibling module (`ohos.rs`) selected via `cfg`.
- **Shared JS contract**: the TS API in `guest-js/index.ts` is not forked — the same
  surface works on every platform. The `Update` object's `available: true` deprecation
  note is preserved.
- **Form-factor neutrality**: mobile and desktop OHOS share the same ArkTS entry points,
  so no `mobile.rs` vs `desktop.rs` split is introduced.

## Capabilities

### New Capabilities
- `ohos-process-relaunch`: OpenHarmony implementation of the `plugin:process|restart`
  command. Covers ArkTS `ApplicationContext.restartApp(want)`, NAPI bridging from the
  Rust command, error mapping, and unit coverage on both mobile and desktop devices.
- `ohos-updater-check`: OpenHarmony implementation of the `plugin:updater|check`
  command and `plugin:updater|download_and_install`. Covers the
  `updateManager.checkAppUpdate()` flow for `check()`, `showUpdateDialog()` for
  `downloadAndInstall()`, mapping the AppGallery response back into the
  `UpdateMetadata` shape expected by the existing JS client, and explicit
  unsupported markers for `download` / `install` (separately).

### Modified Capabilities
<!-- none: the existing JS contracts and Rust command signatures for `relaunch()` / `check()`
     are preserved, only their OHOS backend is added. -->

## Impact

- **Repositories touched (three-repo coordination)**
  - `openharmony-ability` (ArkTS ↔ Rust wrappers, the lowest layer):
    - `native_ability/src/main/ets/helper/os.ets` — add `restart(bundleName, abilityName)` using `ApplicationContext.restartApp`.
    - `native_ability/src/main/ets/helper/updater.ets` — new file wrapping `updateManager.checkAppUpdate`, `updateManager.showUpdateDialog`, and `bundleManager.getBundleInfoForSelf`.
    - `crates/ability/src/app.rs` — add `OpenHarmonyApp::restart(...)` that forwards to the ArkTS helper via the existing NAPI pattern.
    - `crates/ability/src/updater.rs` — new module: `OpenHarmonyApp::updater().check() -> Result<Option<CheckResult>>`.
  - `tauri` core (middle layer — routing):
    - `crates/tauri/src/app.rs` — on OHOS, route `request_restart()` to `tauri::ohos::APP.restart(...)` instead of `crate::process::restart(&env)`.
    - `crates/tauri/src/ohos.rs` — cache `BUNDLE_NAME` / `ABILITY_NAME` as `OnceLock<Option<String>>` siblings of `BASE_PATH` / `MODULE_NAME`.
  - `plugins-workspace` (top layer — JS-facing):
    - `plugins/process/src/{commands,ohos}.rs` — OHOS branch of the `restart` command calls `app.request_restart()`.
    - `plugins/updater/src/{commands,ohos}.rs` — OHOS branch of `check` delegates to `tauri::ohos::APP.updater().check()`; `download`/`install` return `UnsupportedPlatform`; `download_and_install` triggers `showUpdateDialog`.
  - `tao` / `wry` / `muda` / `tray-icon`: **not modified** (no overlap with process-lifecycle or updater responsibilities).
- **APIs**
  - Public JS API: **unchanged**. `relaunch()`, `check()`, `Update` shape all preserved.
  - Rust command signatures: **unchanged** (same `#[tauri::command]` names and parameter
    types). Only the function body is conditionally compiled.
- **Dependencies**
  - `openharmony_ability` (already used by `tauri::ohos`).
  - New ArkTS helpers import `@kit.AbilityKit` (for `ApplicationContext`, `Want`) and
    `@kit.AppGalleryKit` (for `updateManager`).
- **Permissions (module.json5)**
  - `ohos.permission.INTERNET` — already required for the webview; reused for update checks.
  - `ohos.permission.GET_BUNDLE_INFO` — **new, required** for `bundleManager.getBundleInfoForSelf()` to obtain `currentVersion`. Normal-level, system auto-grants, but must be declared. The Tauri OHOS bundler SHALL auto-add it.
- **User-visible behavior**
  - On OHOS, `check()` is **side-effect free** — returns metadata without showing any dialog.
  - `Update.downloadAndInstall()` triggers the system-owned `showUpdateDialog()` which hands off to AppGallery.
  - `Update.download()` / `Update.install()` (separately) throw `UnsupportedPlatform`. Apps SHOULD use the `check()` → `downloadAndInstall()` pattern for cross-platform compatibility.
  - `Metadata.body` and `Metadata.date` are always `null` on OHOS (AppGallery API does not provide these fields).
  - `Metadata.version` is `"unknown"` on API < 20 devices (where `CheckUpdateResult.versionName` is not available).
- **Risk / non-goals**
  - We are not forking the TS API or adding OHOS-only methods — that would split the
    ecosystem.
  - We are not implementing silent/forced updates for 元服务 (atomic services) in this
    change; it is a follow-up if needed.
  - We do not touch Windows / macOS / Linux / Android / iOS code paths; they remain the
    source of truth for those platforms.

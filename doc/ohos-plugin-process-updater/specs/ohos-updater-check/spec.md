## ADDED Requirements

### Requirement: `check()` queries AppGallery via `updateManager.checkAppUpdate`
On OpenHarmony, invoking `check()` from JS SHALL cause the Rust command
`plugin:updater|check` to ask the ArkTS host to call
`updateManager.checkAppUpdate(context)` from `@kit.AppGalleryKit`. The result
SHALL be mapped to the plugin's existing `Metadata` shape. `check()` SHALL NOT
trigger any user-visible UI (no dialog is shown).

The ArkTS helper SHALL also call `bundleManager.getBundleInfoForSelf(
GET_BUNDLE_INFO_DEFAULT)` to obtain the installed `versionName` for the
`currentVersion` field.

#### Scenario: No update available
- **WHEN** `check()` is called on an OHOS device whose installed version matches the AppGallery in-store version
- **THEN** the JS promise resolves to `null`
- **THEN** no system dialog is shown

#### Scenario: Newer version is available in AppGallery (API ≥ 20)
- **WHEN** `check()` is called on an OHOS device (API 6.0.0(20) or later) whose installed version is older than the AppGallery in-store version
- **THEN** the JS promise resolves to an `Update` object with `currentVersion` equal to the installed `versionName` (from `bundleManager`) and `version` equal to `CheckUpdateResult.versionName`
- **THEN** `body` and `date` are `null` (AppGallery API does not provide these fields)
- **THEN** `rawJson` contains the verbatim `CheckUpdateResult` payload
- **THEN** no system dialog is shown

#### Scenario: Newer version available on SDK 12 (API < 20, no `versionName` in response)
- **WHEN** `check()` is called on an OHOS device running SDK 12 (API 12, `compatibleSdkVersion: "5.0.0(12)"`) and `updateAvailable` is `LATER_VERSION_EXIST`
- **THEN** `CheckUpdateResult.versionName` is not available (API 20+ only)
- **THEN** the `Update.version` field SHALL be set to `"unknown"`
- **THEN** `Update.currentVersion` SHALL still be populated from `bundleManager.getBundleInfoForSelf`
- **THEN** the `Update` object is still returned (not `null`) so app code can detect that an update exists and call `downloadAndInstall()`

#### Scenario: AppGallery unreachable (no network / service error)
- **WHEN** `checkAppUpdate()` returns an error code (e.g. 1009400003 network error, 1009400004 not in foreground)
- **THEN** the JS promise is rejected with an `Error` carrying the underlying code and message
- **THEN** no dialog is shown

### Requirement: `downloadAndInstall()` triggers the AppGallery system dialog
On OpenHarmony, `plugin:updater|download_and_install` SHALL call
`updateManager.showUpdateDialog(context)`. This is the OHOS equivalent of
"download and install" — the system dialog drives the entire update flow.

#### Scenario: Update available and dialog succeeds
- **WHEN** the JS caller awaits `Update.downloadAndInstall()` on OHOS and an update is available
- **THEN** `showUpdateDialog` is called
- **THEN** the system AppGallery update dialog is shown to the user
- **THEN** if `ShowUpdateResultCode` is `SHOW_DIALOG_SUCCESS` (0), the JS promise resolves

#### Scenario: Dialog fails on unsupported device
- **WHEN** `showUpdateDialog` returns `SHOW_DIALOG_FAILURE` (1) (e.g. on API <20 non-phone/tablet devices)
- **THEN** the JS promise is rejected with an error containing the result code

### Requirement: `download()` and `install()` return `UnsupportedPlatform` on OHOS
Because AppGallery performs download and installation via `showUpdateDialog()`,
the OHOS backend SHALL reject `plugin:updater|download` and
`plugin:updater|install` with the existing `Error::UnsupportedPlatform` variant.
Apps SHOULD use `downloadAndInstall()` instead.

#### Scenario: App calls `update.download()` on OHOS
- **WHEN** the JS caller awaits `Update.download()` on OHOS
- **THEN** the promise is rejected with an error whose kind maps to `UnsupportedPlatform`
- **THEN** no network or file-system side effects occur

#### Scenario: App calls `update.install()` on OHOS
- **WHEN** the JS caller awaits `Update.install()` on OHOS
- **THEN** the promise is rejected with `UnsupportedPlatform`

### Requirement: Existing `Config` is used; OHOS ignores non-applicable fields
The plugin SHALL keep the existing `updater::Config` schema unchanged. On OHOS,
the `endpoints`, `pubkey`, and per-platform installer fields (`windows`, `macos`,
`linux`) SHALL be ignored — the plugin SHALL identify the app to AppGallery via
the running bundle's identity (bundle name + signature).

#### Scenario: `tauri.conf.json` author does not add OHOS-specific config
- **WHEN** an app ships a `tauri.conf.json` that has `plugins.updater.endpoints` set for desktop
- **THEN** the OHOS build compiles and `check()` works without any additional OHOS-only fields
- **THEN** the `endpoints` values are not used on OHOS

#### Scenario: Desktop build still honors `endpoints`
- **WHEN** the same `tauri.conf.json` is used to build for Windows
- **THEN** the Windows updater still contacts the configured `endpoints`
- **THEN** no OHOS code path is compiled or executed

### Requirement: OHOS code is isolated via `cfg(target_env = "ohos")`
All OpenHarmony-specific Rust code in `plugin-updater` SHALL be gated behind
`#[cfg(target_env = "ohos")]`. The existing HTTP/manifest-based `updater.rs`
and the non-OHOS branches of `commands.rs` SHALL remain untouched.

#### Scenario: Non-OHOS targets build without the OHOS module present
- **WHEN** `cargo build --target x86_64-pc-windows-msvc` (or macOS/Linux/Android/iOS) is run for `tauri-plugin-updater`
- **THEN** the OHOS module is not compiled, parsed, or linked
- **THEN** behavior is identical to the pre-change version

#### Scenario: OHOS target pulls in the OHOS module
- **WHEN** `cargo build --target aarch64-unknown-linux-ohos` is run for `tauri-plugin-updater`
- **THEN** the OHOS module is compiled and registered as the handler for `plugin:updater|check`, `download`, `install`, `download_and_install`
- **THEN** `updater.rs` is excluded from the OHOS build

### Requirement: ArkTS helper ships with the plugin
The plugin SHALL include an ArkTS source file (copied into the host app by the
OHOS bundler) that exposes handlers for `check`, `showUpdateDialog`, and
`downloadAndInstall`.

#### Scenario: Host app has the helper after `tauri ohos build`
- **WHEN** the Tauri OHOS bundler generates the host app
- **THEN** a file `entry/src/main/ets/tauri-plugin-updater/TauriUpdaterAdapter.ets` exists
- **THEN** the file imports `@kit.AppGalleryKit` (for `updateManager`) and `@kit.AbilityKit` (for `bundleManager`)

### Requirement: `Metadata` shape is preserved with OHOS field limitations
The OHOS `check` command SHALL return the same `Metadata` struct
(`current_version`, `version`, `date`, `body`, `raw_json`, `rid`) that the
non-OHOS command returns. On OHOS:
- `current_version` SHALL come from `bundleManager.getBundleInfoForSelf`
- `version` SHALL come from `CheckUpdateResult.versionName` (or `"unknown"` on API < 20)
- `date` SHALL be `None`
- `body` SHALL be `None`
- `raw_json` SHALL contain the verbatim `CheckUpdateResult` payload

#### Scenario: Downstream JS code reads `update.version` the same way
- **WHEN** app code does `const u = await check(); console.log(u?.version)`
- **THEN** the value printed on OHOS is the AppGallery in-store version string (or `"unknown"` on API < 20)
- **THEN** the same code on Windows reads the version from the configured endpoint

### Requirement: `ohos.permission.GET_BUNDLE_INFO` is declared
The host app's `module.json5` SHALL declare `ohos.permission.GET_BUNDLE_INFO`
(a `normal` permission, auto-granted by the system) to allow
`bundleManager.getBundleInfoForSelf()` to retrieve `currentVersion`.

#### Scenario: Permission is auto-added by the Tauri OHOS bundler
- **WHEN** the Tauri OHOS bundler generates the host app
- **THEN** the generated `module.json5` includes `ohos.permission.GET_BUNDLE_INFO` in `requestPermissions`

### Requirement: Form-factor neutrality
The OHOS updater implementation SHALL NOT contain any branch on device class.
The same ArkTS code path SHALL run on phones, tablets, 2-in-1s and smart
displays.

#### Scenario: Same build artifact runs on phone and 2-in-1
- **WHEN** the same `.hap` produced from this plugin is installed on an OHOS phone and an OHOS 2-in-1
- **THEN** `check()` behaves identically on both devices

### Requirement: Signature and version sanity
`updateManager.checkAppUpdate` requires the installed signature to match the
AppGallery listing and the installed version to be lower than the in-store
version for an update to be reported. The OHOS backend SHALL NOT attempt to
work around these checks.

#### Scenario: Debug build with mismatched signature
- **WHEN** `check()` is called from a debug build whose signature does not match AppGallery
- **THEN** `checkAppUpdate` reports no update
- **THEN** `check()` resolves to `null`
- **THEN** a warning is logged (not thrown) to aid development

#### Scenario: Installed version newer than in-store (rollout lag)
- **WHEN** the installed version is newer than AppGallery's current listing
- **THEN** `check()` resolves to `null` (no downgrade offered)

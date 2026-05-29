## Context

The Tauri plugin ecosystem exposes a stable JS contract to app authors. Two of the most
used plugins — `@tauri-apps/plugin-process` (`relaunch()`) and
`@tauri-apps/plugin-updater` (`check()`) — are currently implemented only for desktop
(Windows/macOS/Linux) in the Rust layer; Android/iOS have their own mobile-specific
plugins where applicable. We are adding an OpenHarmony (OHOS) backend that must:

1. Reuse the **same JS entry points** — app code written today for Windows must keep
   working unchanged on an OHOS device.
2. Map to the **ArkTS platform APIs** — `appRecovery.restartApp()` for
   relaunch and `updateManager.checkAppUpdate` / `showUpdateDialog` for updates.
3. Remain **invisible to every other platform** — Windows, macOS, Linux, Android and
   iOS builds must compile and behave exactly as before.

Both APIs are universal across OHOS mobile (phone/tablet) and desktop (2-in-1/pad/PC)
form factors: the same ArkTS entry points exist on both, so we do not need a
mobile-vs-desktop split inside the plugin.

### Multi-repo architecture

The OHOS adaptation spans **three repositories**, each with a distinct responsibility:

```
┌────────────────────────────────────────────────────────────────────┐
│  plugins-workspace   (plugin Rust crates + guest-js)               │
│    ├─ plugin-process/src/{commands,ohos}.rs                        │
│    └─ plugin-updater/src/{commands,updater,ohos}.rs                │
│            │   invokes AppHandle APIs / tauri::ohos::APP           │
└────────────┼───────────────────────────────────────────────────────┘
             │ depends on
             ▼
┌────────────────────────────────────────────────────────────────────┐
│  tauri core          (AppHandle, runtime routing, tauri::ohos)     │
│    ├─ crates/tauri/src/app.rs  → request_restart() OHOS branch     │
│    └─ crates/tauri/src/ohos.rs → BASE_PATH / MODULE_NAME / ...     │
│            │   calls into                                            │
└────────────┼───────────────────────────────────────────────────────┘
             │ depends on
             ▼
┌────────────────────────────────────────────────────────────────────┐
│  openharmony-ability (ArkTS ↔ Rust wrappers via napi-ohos)         │
│    ├─ native_ability/src/main/ets/helper/os.ets       (restart)    │
│    ├─ native_ability/src/main/ets/helper/updater.ets  (check)      │
│    ├─ crates/ability/src/app.rs        OpenHarmonyApp::restart()   │
│    └─ crates/ability/src/updater.rs    Updater::check()            │
└────────────────────────────────────────────────────────────────────┘
```

Other local dependencies of `tauri` (`tao`, `wry`, `muda`, `tray-icon`) are **not**
touched by this change — they deal with windowing / menuing / tray concerns and
don't own process-lifecycle or update-check responsibilities.

The Tauri OHOS runtime already hosts an ArkTS side (`tauri::ohos::APP`,
`openharmony_ability`) and the `napi-ohos` plugin bridge used by `clipboard-manager`,
`shell`, `opener`, etc. We build on that foundation — no new FFI mechanism is needed.

### Target SDK and verification app

The OHOS demo app is `tauri/examples/api/src-tauri`, targeting
`compatibleSdkVersion: "5.0.0(12)"` (API 12) with `deviceTypes: [phone, tablet, 2in1]`.
This has several implications for the design:

**API 12 availability:**
- `appRecovery.restartApp` — ✅ available from API 12 (HarmonyOS 5.0.0)
- `appRecovery.enableAppRecovery` — ✅ available from API 12
- `updateManager.checkAppUpdate` — ✅ available from API 12
- `updateManager.showUpdateDialog` — ✅ available from API 12 on Phone/Tablet/2in1
- `CheckUpdateResult.versionName` / `versionCode` — ❌ **API 20 only** (HarmonyOS 6.0.0+)

**Consequence for the demo app:** On SDK 12, `checkAppUpdate` returns only
`updateAvailable` (boolean-like enum). The `version` field in `Metadata` will always
be `"unknown"`. `currentVersion` is obtained separately via
`bundleManager.getBundleInfoForSelf()` and works fine on API 12.

**`appRecovery.restartApp()` is a hard kill:** The API does NOT trigger `onDestroy` — it terminates
the process immediately and starts a new one. Apps that rely on `onDestroy` for
cleanup must use a different mechanism (e.g. save state before calling `relaunch()`).
Requires `recoverable: true` in `module.json5` and `enableAppRecovery()` in `AbilityStage.onCreate()`.

The demo app already has `tauri-plugin-process` in its `Cargo.toml` but is
**missing `tauri-plugin-updater`** — that must be added as part of implementation.

## Goals / Non-Goals

**Goals:**

- `relaunch()` on OHOS cleanly restarts the app by delegating to
  `appRecovery.restartApp()` on the ArkTS side. Note: this is a hard
  process kill + cold restart (no `onDestroy` callback), not a graceful lifecycle
  transition — matches the desktop behavior where the process exits immediately.
- `check()` on OHOS returns `null` when no update is available, or an `Update` object
  carrying `currentVersion` (from `bundleManager`, always available), `version`
  (from AppGallery `CheckUpdateResult.versionName` — **API 20+ only**, falls back to
  `"unknown"` on SDK 12), and `body: None` / `date: None` (not provided by AppGallery
  API at any version) when a newer version is in store.
- Windows / macOS / Linux / Android / iOS code paths remain bit-for-bit unchanged.
- The plugin's public JS API (`guest-js/index.ts`) is **not forked** for OHOS.
- The implementation compiles and runs on both OHOS mobile and OHOS desktop without
  conditional branches on device class.

**Non-Goals:**

- Silent / forced updates (元服务 `abilityManager.restartSelfAtomicService`) — out of
  scope; the standard AppGallery flow is enough for v1.
- Custom HTTP-based updater endpoints on OHOS — AppGallery is the sole distribution
  channel; the existing `endpoints` field of the updater config is ignored on OHOS.
- Implementing `Update.download()` / `install()` (separately) on OHOS —
  AppGallery drives download and installation via `showUpdateDialog()`, which is
  triggered by `downloadAndInstall()`. The separate `download()` / `install()`
  calls return `UnsupportedPlatform` on OHOS.
- Adding OHOS-only methods to the TS API (e.g. a direct `showUpdateDialog()`).
- Implementing `exit()` for `plugin-process` — already covered by runtime exit paths.
- Forking mobile vs desktop OHOS implementations.

## Decisions

### 0. Split responsibilities across three repos

**Choice.** Each layer owns only what it is best positioned to own:

| Layer | Responsibility |
| --- | --- |
| `openharmony-ability` | ArkTS ↔ Rust wrappers for OHOS system APIs. Knows about `@kit.AbilityKit`, `@kit.AppGalleryKit`, NAPI. Exposes `OpenHarmonyApp::restart()` and `OpenHarmonyApp::updater().check()`. |
| `tauri` core | Routes `AppHandle::request_restart()` to the right backend per target; caches bundle/ability names in `tauri::ohos` for OHOS consumers. Does **not** know about AppGallery or update metadata. |
| `plugins-workspace` | Wires JS `relaunch()` / `check()` to `AppHandle` (relaunch) or `tauri::ohos::APP.updater()` (check). Does **not** touch ArkTS or NAPI directly. |

**Rationale.** Keeping ArkTS/NAPI in `openharmony-ability` means `tauri` core and the
plugins never compile `napi-ohos`-specific code on non-OHOS targets and the ArkTS
source is in one place for OHOS contributors to review. Keeping routing in `tauri`
core means every plugin (and app-side `AppHandle` callers) benefits from the same
restart path — plugin-process is just one caller. Keeping plugins thin means the JS
contract is the only surface app authors see.

Alternative considered:

- *Put ArkTS/NAPI directly in each plugin.* Rejected: duplicates the NAPI plumbing
  across plugins, breaks the convention established by `tauri::ohos::APP`, and makes
  `tauri::AppHandle::request_restart()` unable to reuse the same OHOS restart helper.

### 1. Isolation via `cfg(target_env = "ohos")` at the Rust command boundary

**Choice.** Add a sibling `ohos.rs` in each plugin's `src/` and gate the command body
with `#[cfg(target_env = "ohos")]` / `#[cfg(not(target_env = "ohos"))]`. The command
signature (name, parameters, return type) stays identical on both sides.

**Rationale.** This is the same pattern `clipboard-manager`, `shell` and `opener`
already use in the plugins-workspace. It keeps the change small, reviewable, and
compiler-enforced: on non-OHOS targets the OHOS module is not even parsed, so a typo
there cannot break desktop builds. Alternatives considered:

- *Feature-flag the OHOS backend* (e.g. `features = ["ohos"]`). Rejected: `target_env`
  is the project-wide convention and a feature flag would let someone accidentally enable
  OHOS code on a non-OHOS target.
- *Single `commands.rs` with inline `if cfg!(...)`.* Rejected: clutters the existing
  path and forces OHOS-only imports to be cfg'd inside the shared module.

### 2. ArkTS helpers ship in the host app's `entry/src/main/ets/`, not in the plugin

**Choice.** Provide the ArkTS glue (`TauriProcessAdapter`, `TauriUpdaterAdapter`) as
source files that the Tauri OHOS bundler copies into the generated host app under
`entry/src/main/ets/tauri-plugin-*`. The plugin's Rust side invokes them via the
existing `tauri::plugin::mobile`-style NAPI channel (reused for OHOS — see
`tauri::ohos::APP` and `openharmony_ability`).

**Rationale.** The host-app location mirrors how Android plugins are packaged
(`android/src/main/kotlin/...`) and how `napi-ohos` examples are structured. Shipping
as source (not a prebuilt HAR) means the app developer can override the helper if a
future HarmonyOS API requires it. Alternative:

- *Publish a prebuilt `tauri-plugin-process.ohos.har`.* Rejected: adds a build artifact
  to the pipeline and makes debugging harder. Source-in-app matches the rest of the
  OHOS plugin portfolio.

### 3. `relaunch()` maps to `appRecovery.restartApp()`, not `ApplicationContext.restartApp`

**Choice.** The ArkTS helper calls:
```typescript
import { appRecovery } from '@kit.AbilityKit';
appRecovery.restartApp();
```

This requires two prerequisite configurations in the host app:

1. **`AbilityStage.onCreate()`** must call `appRecovery.enableAppRecovery()`:
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
   The Tauri OHOS bundler generates this file into `entry/src/main/ets/abilitystage/TauriAbilityStage.ets`.

2. **`module.json5`** must declare `"recoverable": true` on the entry Ability and add `"srcEntry"` pointing to the AbilityStage file.

The Tauri OHOS bundler is responsible for generating both files, similar to how it already generates `module.json5` with `requestPermissions`.

**Why not `ApplicationContext.restartApp(want)`?** Testing on real devices shows that `ApplicationContext.restartApp()` terminates the process but the system does **not** relaunch the app — the app simply exits. This API appears designed for a different use case (e.g. switching ability stacks) rather than a self-restart. `appRecovery.restartApp()` is the only OHOS API that reliably kills and relaunches the process.

**API constraints accounted for:**
- 1-minute cooldown between calls — the system silently drops `restartApp()` calls within 60 seconds of the last restart. Documented in the plugin README.
- Requires `recoverable: true` in `module.json5` and `enableAppRecovery()` in `AbilityStage.onCreate()`.
- The Rust command blocks the worker thread with `loop { sleep(Duration::MAX) }` after dispatching — same pattern as the non-OHOS path, since `appRecovery.restartApp()` runs on the main thread and will kill the entire process.

**Rejected alternative:**
- *`ApplicationContext.restartApp(want)`* — tested and confirmed: app exits but does not restart.
- *`abilityManager.startAbility()` + `process.exit()`* — more moving parts, no guaranteed relaunch.

### 4. `check()` is pure; `downloadAndInstall()` triggers the system dialog

**Choice.** The ArkTS helper exposes **three** functions:
- `check(context)` — calls `updateManager.checkAppUpdate(context)`, then
  `bundleManager.getBundleInfoForSelf(GET_BUNDLE_INFO_DEFAULT)` to obtain the
  installed `versionName`. Returns `null` or `{ currentVersion, version, body: null,
  date: null, rawJson }`. **No dialog is shown.**
- `showUpdateDialog(context)` — calls `updateManager.showUpdateDialog(context)`.
  Returns `ShowUpdateResultCode`. Used internally by `downloadAndInstall`.
- `downloadAndInstall(context)` — calls `showUpdateDialog(context)` and resolves
  if `SHOW_DIALOG_SUCCESS`, rejects if `SHOW_DIALOG_FAILURE`.

The Rust command mapping:
- `plugin:updater|check` → calls ArkTS `check`, returns `Metadata`.
- `plugin:updater|download_and_install` → calls ArkTS `downloadAndInstall` (triggers dialog).
- `plugin:updater|download` → returns `UnsupportedPlatform`.
- `plugin:updater|install` → returns `UnsupportedPlatform`.

**Rationale.** This preserves the Tauri cross-platform contract:
1. `check()` is **side-effect free** — returns metadata without surprising the user
   with a system dialog. App code can inspect the result and decide when/how to
   proceed, same as on desktop.
2. `downloadAndInstall()` is the natural place for the OHOS system dialog — on
   desktop this method downloads and installs; on OHOS it shows the AppGallery
   dialog which achieves the same end state. Apps that use the
   `check() → downloadAndInstall()` pattern work identically across platforms.
3. `download()` and `install()` separately remain `UnsupportedPlatform` because
   OHOS has no concept of downloading an update package or installing it outside
   the AppGallery dialog flow.

Alternatives considered:

- *Auto-show dialog in `check()`.* Rejected: causes a system modal to pop up as a
  side-effect of a metadata query — surprising UX, cannot be deferred or suppressed,
  and breaks apps that call `check()` at startup to silently poll.
- *Add an OHOS-only `showUpdateDialog()` method to the TS API.* Rejected: forks
  the TS API surface.
- *Make `download()` trigger the dialog instead.* Rejected: desktop apps commonly
  call `download()` with progress callbacks; the OHOS dialog has no progress
  streaming, so `downloadAndInstall()` (which has no progress expectation) is the
  better match.

### 5. Config: reuse existing `Config`, no new OHOS-specific fields

**Choice.** The `updater` plugin's existing `Config` struct is kept as-is. On OHOS,
the `endpoints`, `pubkey` and per-platform installer fields are **ignored** — the
plugin derives the AppGallery query from the running bundle's identity (via
`bundleManager.getBundleInfoForSelf`). The `windows` / `macos` / `linux` config
branches remain untouched for their respective targets.

**Rationale.** Adding an `ohos: { appId }` field would leak an OHOS-only concept into
a cross-platform config schema and force every `tauri.conf.json` author to think about
it. Because AppGallery identifies the app by its signed bundle, no extra identifier is
needed at runtime. Alternative:

- *Introduce `ohos: { appId?, storeRegion? }`.* Kept as a future option if we later
  need to let developers pin a store region or a different AppGallery deployment.

### 6. Mobile vs desktop: single code path

**Choice.** No `mobile.rs` / `desktop.rs` split. The OHOS module compiles the same
way regardless of form factor.

**Rationale.** `appRecovery.restartApp` and `updateManager.checkAppUpdate` are
part of the Stage model, available on every OHOS device class. The only place form
factor ever matters is UI (dialog size, orientation) — which is owned by AppGallery,
not by us.

## Risks / Trade-offs

- **AppGallery availability in dev environments.** → Mitigation: document that
  `check()` requires a signed build whose bundle signature matches the AppGallery
  listing; unsigned / debug builds get `null` and a logged warning rather than an
  exception.
- **`appRecovery.restartApp` main-thread constraint.** The API only works on the main thread;
  calling from a background thread throws. → Mitigation: the ArkTS helper always
  runs on the main thread (the NAPI call is dispatched through the OHOS runtime's
  main-thread waker, same pattern as `exit` and `setColorMode`).
- **`appRecovery.restartApp` 1-minute cooldown.** Calling `restartApp` twice within 60 seconds
  causes the second call to be silently dropped (app exits but does not restart).
  → Mitigation: document this in the plugin README; the Rust command does not
  implement a client-side debounce because Tauri apps rarely call `relaunch()` in
  rapid succession.
- **`appRecovery.restartApp` requires `recoverable: true` and `enableAppRecovery()`.**
  The entry Ability must declare `"recoverable": true` in `module.json5` and the
  `AbilityStage.onCreate()` must call `appRecovery.enableAppRecovery()`.
  → Mitigation: the Tauri OHOS bundler generates both files automatically.
- **`versionName` only available from API 6.0.0(20).** On earlier OHOS versions
  (including our SDK 12 demo target), `CheckUpdateResult` only carries
  `updateAvailable`; the new version's name/code is not returned. → Mitigation: on
  API <20 (which includes SDK 12), `Metadata.version` falls back to the string
  `"unknown"` (with `updateAvailable` still accurately reporting whether an update
  exists). `currentVersion` is always available via `bundleManager`. Document the
  API-level split in the README.
- **`showUpdateDialog` may return `SHOW_DIALOG_FAILURE`.** On devices running
  API <20 that are not Phone/Tablet/2-in-1, the dialog is not supported and the
  API returns failure. → Mitigation: the `downloadAndInstall` command maps this
  to an error with the result code; app code can catch and show a custom fallback
  (e.g. "visit the app store").
- **`ohos.permission.GET_BUNDLE_INFO` required.** Needed for
  `bundleManager.getBundleInfoForSelf()` to obtain `currentVersion`. It is a
  `normal` permission (system auto-grants), but must be declared in
  `module.json5`. → Mitigation: the Tauri OHOS bundler auto-adds it to the
  generated manifest; documented in the plugin setup guide.
- **No offline cache of update state.** Each `check()` hits AppGallery. → Mitigation:
  this matches the desktop updater's behavior (no caching) and is not a regression.
- **`UnsupportedPlatform` for `download()`/`install()` is new for apps that
  cross-compile.** → Mitigation: the error type already exists in the plugin; app
  code that uses `downloadAndInstall()` (the recommended cross-platform method)
  works on OHOS without changes.
- **`appRecovery.restartApp` takes no parameters.** Unlike `ApplicationContext.restartApp`
  which needs `bundleName` + `abilityName`, `appRecovery.restartApp()` uses the
  currently running Ability's identity automatically. → No mitigation needed — simpler API.

## Migration Plan

No user-facing migration — this is a new capability on a new platform. Existing
`tauri.conf.json` files need no changes; app code that imports
`@tauri-apps/plugin-process` or `@tauri-apps/plugin-updater` works unchanged after
rebuilding for OHOS.

**Rollback.** Revert the PR. Because all OHOS code is behind `cfg(target_env =
"ohos")`, reverting only affects OHOS builds; desktop/mobile platforms are unaffected.

## Open Questions

- Do we want to add OHOS-only config (`ohos.storeRegion`) later to support
  multi-region AppGallery deployments? Deferred until there is a concrete consumer.
- Should `exit()` for `plugin-process` also be explicitly implemented for OHOS? Left
  for a follow-up — runtime exit already works for the scenarios we have observed.

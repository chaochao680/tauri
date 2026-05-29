## ADDED Requirements

### Requirement: `relaunch()` restarts the OHOS app via the Stage-model API
On OpenHarmony, invoking `relaunch()` from JS SHALL cause the Rust command
`plugin:process|restart` to ask the ArkTS host to call
`ApplicationContext.restartApp(want)` with a `Want` pointing at the current
`bundleName` and the entry `abilityName`. The system SHALL terminate the current
process and start a new one (note: `onDestroy` is NOT triggered — this is a hard
process kill, not a graceful lifecycle transition).

#### Scenario: User triggers relaunch from a settings screen
- **WHEN** the JS caller awaits `relaunch()` on an OHOS device (mobile or desktop) in the foreground
- **THEN** the ArkTS helper reads `bundleName` and `abilityName` from the registered `UIAbilityContext`
- **THEN** the current process is killed (no `onDestroy` callback) and a new process starts with a fresh instance of the entry Ability
- **THEN** the new Ability's `onCreate` receives the original launch `Want`
- **NOTE** the JS promise does NOT resolve — the process is killed before the IPC response can be sent (matches desktop behavior)

#### Scenario: Relaunch invoked from a non-main thread
- **WHEN** `relaunch()` is called from a background Tauri command
- **THEN** the runtime SHALL marshal the restart request to the ArkTS main thread via the existing NAPI main-thread waker
- **THEN** the restart SHALL still complete successfully

#### Scenario: App is backgrounded when relaunch is called
- **WHEN** `relaunch()` is called while the app is not in the foreground
- **THEN** the ArkTS helper SHALL detect the backgrounded state and return an error
- **THEN** the JS `relaunch()` promise SHALL be rejected with a descriptive error
- **THEN** the app process SHALL remain running (not silently killed)

### Requirement: Rust command signature is preserved
The Rust command registered as `plugin:process|restart` SHALL keep its existing
signature (no parameters, no return value beyond `Result<(), InvokeError>`) so that
`guest-js/index.ts` does not change.

#### Scenario: Existing TS code compiles unchanged
- **WHEN** an app imports `relaunch` from `@tauri-apps/plugin-process` and calls it
- **THEN** the same compiled JS SHALL run on Windows, macOS, Linux, Android, iOS and OHOS without modification

### Requirement: OHOS code is isolated via `cfg(target_env = "ohos")`
All OpenHarmony-specific Rust code in `plugin-process` SHALL be gated behind
`#[cfg(target_env = "ohos")]`. The existing (non-OHOS) command body SHALL remain
under `#[cfg(not(target_env = "ohos"))]` and SHALL NOT be modified.

#### Scenario: Non-OHOS targets build without the OHOS module present
- **WHEN** `cargo build --target x86_64-pc-windows-msvc` (or macOS/Linux/Android/iOS) is run for `tauri-plugin-process`
- **THEN** the OHOS module is not compiled, parsed, or linked
- **THEN** behavior is identical to the pre-change version

#### Scenario: OHOS target pulls in the OHOS module
- **WHEN** `cargo build --target aarch64-unknown-linux-ohos` is run for `tauri-plugin-process`
- **THEN** the OHOS module is compiled and registered as the handler for `plugin:process|restart`
- **THEN** the non-OHOS command body is not included

### Requirement: ArkTS helper ships with the plugin
The plugin SHALL include an ArkTS source file (copied into the host app by the OHOS
bundler) that exposes a `relaunch` handler. The helper SHALL read `bundleName` and
`abilityName` from the registered `UIAbilityContext` (no parameters needed from Rust)
and call `ApplicationContext.restartApp(want)`.

#### Scenario: Host app has the helper after `tauri ohos build`
- **WHEN** the Tauri OHOS bundler generates the host app
- **THEN** a file `entry/src/main/ets/tauri-plugin-process/TauriProcessAdapter.ets` exists
- **THEN** the file imports `@kit.AbilityKit` and uses `ApplicationContext.restartApp`

### Requirement: Failure to restart is reported to JS
If `ApplicationContext.restartApp` throws (e.g. invalid context, 3-second cooldown
violation, permission denial), the Rust command SHALL map the error to a Tauri invoke
error with the original message preserved.

#### Scenario: Restart call is rejected by the system
- **WHEN** the ArkTS helper's `restartApp` call throws
- **THEN** the JS `relaunch()` promise is rejected with an `Error` whose message contains the underlying system message
- **THEN** the app process remains running (it is not force-killed)

### Requirement: Form-factor neutrality
The OHOS relaunch implementation SHALL NOT contain any branch on device class
(phone vs tablet vs 2-in-1 vs smart display). The same ArkTS code path SHALL run
on all OHOS form factors.

#### Scenario: Same build artifact runs on phone and 2-in-1
- **WHEN** the same `.hap` produced from this plugin is installed on an OHOS phone and an OHOS 2-in-1
- **THEN** `relaunch()` behaves identically on both devices

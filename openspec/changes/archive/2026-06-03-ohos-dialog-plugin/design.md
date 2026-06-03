## Context

Tauri's plugin system provides a bridge between Rust backend logic and platform-specific frontend APIs. On iOS and Android, this is achieved through:
- **iOS**: Swift plugins registered via `PluginApi::register_ios_plugin`, commands dispatched through `WKScriptMessageHandler`
- **Android**: Kotlin plugins registered via `PluginApi::register_android_plugin`, commands dispatched through JNI

On OHOS (OpenHarmony OS), the architecture differs:
- Frontend: ArkTS (TypeScript-like language) running in ArkUI framework
- Native bridge: NAPI (Node-API compatible) for Rust ↔ ArkTS communication
- Package format: HAR (Harmony Archive) files, analogous to Android's AAR or iOS's framework bundles

The dialog plugin specifically needed OHOS support because:
1. **File dialogs**: OHOS uses `@ohos.file.picker` APIs (FilePicker, PhotoViewPicker) instead of Windows file dialogs or macOS NSOpenPanel
2. **Message dialogs**: OHOS uses `@ohos.promptAction.showDialog` instead of Windows MessageBox or macOS NSAlert
3. **Confirmation dialogs**: OHOS uses `@ohos.promptAction.showActionMenu` for multi-button confirmations

### Previous State (Before This Change)

```rust
// crates/tauri/src/plugin/mobile.rs
#[cfg(target_env = "ohos")]
pub(crate) fn run_command<R: Runtime, C: AsRef<str>, F: FnOnce(PluginResponse) + Send + 'static>(
  _name: &str,
  _handle: &AppHandle<R>,
  _command: C,
  _payload: serde_json::Value,
  _handler: F,
) -> Result<(), PluginInvokeError> {
  // TODO
}
```

The OHOS `run_command` was a stub that did nothing, making all plugins non-functional on OHOS.

### Current State (After This Change)

```rust
#[cfg(target_env = "ohos")]
pub(crate) fn run_command<R: Runtime, C: AsRef<str>, F: FnOnce(PluginResponse) + Send + 'static>(
  name: &str,
  handle: &AppHandle<R>,
  command: C,
  payload: serde_json::Value,
  handler: F,
) -> Result<(), PluginInvokeError> {
  let id: i32 = PENDING_PLUGIN_CALLS_ID.fetch_add(1, Ordering::Relaxed);
  PENDING_PLUGIN_CALLS
    .get_or_init(Default::default)
    .lock()
    .unwrap()
    .insert(id, Box::new(handler));

  let args = crate::ohos::RunCommandArgs {
    id,
    plugin_name: name.to_string(),
    command: command.as_ref().to_string(),
    payload: serde_json::to_string(&payload).map_err(PluginInvokeError::CannotSerializePayload)?,
  };

  crate::ohos::dispatch_run_command(args);

  Ok(())
}
```

**Stakeholders:**
- Tauri plugin developers targeting OHOS
- Application developers using dialog/file-picker APIs on OHOS devices
- Tauri CLI maintainers (build process changes)
- OHOS platform maintainers (runtime integration)

## Goals / Non-Goals

**Goals:**
- Implement complete OHOS plugin registration and command dispatch system
- Enable `tauri-plugin-dialog` to function on OHOS with native file pickers and message dialogs
- Provide CLI tooling to automatically detect and bundle plugin HAR files during build
- Create reusable OHOS plugin templates for future plugin development
- Fix OHOS-specific runtime issues (webview_version check, cfg guards)

**Non-Goals:**
- Porting all Tauri plugins to OHOS (only dialog plugin is in scope)
- Modifying iOS or Android plugin systems (already functional)
- Changing the `PluginApi` or `PluginHandle` struct definitions (backward compatible additions only)
- Implementing OHOS-specific UI components (handled by dialog plugin's ArkTS code)

## Decisions

### Decision 1: Use callback ID system for async plugin commands

**Choice:** Assign a unique `i32` ID to each plugin command invocation, store the response handler in `PENDING_PLUGIN_CALLS` map, and invoke it when OHOS returns the result via NAPI callback.

**Rationale:**
- Matches the existing iOS/Android pattern (see `PENDING_PLUGIN_CALLS_ID` in `plugin/mobile.rs`)
- OHOS NAPI callbacks are asynchronous; the ID bridges the Rust handler with the ArkTS response
- Simple atomic counter (`AtomicI32`) for ID generation is thread-safe and zero-cost

**Implementation:**
```rust
let id: i32 = PENDING_PLUGIN_CALLS_ID.fetch_add(1, Ordering::Relaxed);
PENDING_PLUGIN_CALLS
  .get_or_init(Default::default)
  .lock()
  .unwrap()
  .insert(id, Box::new(handler));
```

**Alternatives considered:**
1. Use `Uuid` for command IDs → Rejected: overkill, adds dependency, i32 is sufficient for in-flight commands
2. Synchronous blocking wait for OHOS response → Rejected: blocks Rust event loop, breaks Tauri's async model
3. Channel-based response system → Rejected: more complex than callback map, no clear benefit

### Decision 2: Centralize OHOS plugin state in `crate::ohos` module

**Choice:** Store plugin registrations in `PLUGINS_TO_REGISTER: Mutex<Vec<PluginRegistration>>` and dispatch logic in `dispatch_run_command` function, both in `crates/tauri/src/ohos.rs`.

**Rationale:**
- Keeps OHOS-specific code isolated (guarded by `#[cfg(target_env = "ohos")]`)
- Avoids scattering OHOS logic across `plugin/mobile.rs`, `app.rs`, etc.
- `PluginRegistration` struct encapsulates all metadata needed for OHOS NAPI registration

**Implementation:**
```rust
// crates/tauri/src/ohos.rs
pub struct PluginRegistration {
  pub name: String,
  pub identifier: String,
  pub class_name: String,
  pub config: serde_json::Value,
}

lazy_static::lazy_static! {
  pub static ref PLUGINS_TO_REGISTER: Mutex<Vec<PluginRegistration>> = Mutex::new(Vec::new());
}

pub fn dispatch_run_command(args: RunCommandArgs) {
  // NAPI call to ArkTS plugin manager
}
```

**Alternatives considered:**
1. Store plugin state in `AppManager` → Rejected: mixes OHOS-specific state with cross-platform manager
2. Use `PluginStore` (existing struct) → Rejected: `PluginStore` is for Rust-side plugins, not OHOS NAPI plugins
3. Global `HashMap` without `Mutex` → Rejected: not thread-safe, `register_ohos_plugin` can be called from multiple threads

### Decision 3: Auto-detect plugins via `Cargo.toml` scanning

**Choice:** CLI scans `Cargo.toml` dependencies for `tauri-plugin-*` crates, then searches for corresponding HAR files in `plugins/<name>/openharmony/` or `plugins-workspace/plugins/<name>/openharmony/`.

**Rationale:**
- Matches Tauri's convention-over-configuration philosophy (no manual plugin registration)
- Reuses existing `tauri-plugin-*` naming convention
- HAR files are automatically bundled into the OHOS app during build

**Implementation:**
```rust
// crates/tauri-cli/src/mobile/open_harmony/plugins.rs
pub fn detect_plugins(cargo_manifest_path: &Path) -> Result<Vec<String>> {
  let manifest: toml_edit::DocumentMut = content.parse()?;
  let mut plugins = Vec::new();

  if let Some(deps) = manifest.get("dependencies").and_then(|d| d.as_table()) {
    collect_plugins_from_table(deps, &mut plugins);
  }

  Ok(plugins)
}

fn collect_plugins_from_table(table: &toml_edit::Table, plugins: &mut Vec<String>) {
  for (name, _) in table.iter() {
    if name.starts_with("tauri-plugin-") {
      plugins.push(name.replace("tauri-plugin-", ""));
    }
  }
}
```

**Alternatives considered:**
1. Require manual plugin list in `tauri.conf.json` → Rejected: manual, error-prone, deviates from iOS/Android auto-detection
2. Scan `src/main.rs` for `plugin::Builder` calls → Rejected: requires parsing Rust AST, fragile
3. Use `cargo metadata` to resolve dependencies → Rejected: slower, requires cargo invocation, overkill for simple prefix matching

### Decision 4: Provide OHOS HAR module template in `crates/tauri/mobile/ohos/`

**Choice:** Include `Plugin.ets`, `PluginManager.ets`, and `module.json5` in Tauri's source tree, copied to OHOS project during `tauri init` or `tauri build`.

**Rationale:**
- HAR modules are ArkTS code, not Rust; must be provided as templates
- `PluginManager.ets` handles command dispatch from Rust to registered plugins
- `Plugin.ets` is the NAPI entry point that Rust calls via `napi_call_function`

**Structure:**
```
crates/tauri/mobile/ohos/
├── build-profile.json5
├── hvigorfile.ts
├── oh-package.json5
├── src/main/
│   ├── ets/
│   │   ├── Plugin.ets          # NAPI entry point
│   │   ├── PluginManager.ets   # Command dispatcher
│   │   └── index.ets           # Module exports
│   └── module.json5
```

**Alternatives considered:**
1. Generate ArkTS code from Rust macros → Rejected: too complex, ArkTS syntax differs from Rust
2. Require plugin authors to write HAR modules from scratch → Rejected: high barrier to entry, inconsistent implementations
3. Bundle pre-compiled HAR binaries → Rejected: not flexible, can't customize plugin behavior

### Decision 5: Make `PENDING_PLUGIN_CALLS` public (`pub(crate)`)

**Choice:** Change `PENDING_PLUGIN_CALLS` from `static` to `pub(crate) static` to allow `crate::ohos` to access it for response handling.

**Rationale:**
- `dispatch_run_command` in `ohos.rs` needs to insert handlers into `PENDING_PLUGIN_CALLS`
- `plugin/mobile.rs` and `ohos.rs` are in the same crate (`tauri`), so `pub(crate)` is appropriate
- Avoids duplicating the callback map in `ohos.rs`

**Alternatives considered:**
1. Move `PENDING_PLUGIN_CALLS` to `ohos.rs` → Rejected: breaks iOS/Android code that also uses it
2. Pass handler through function parameters → Rejected: `dispatch_run_command` is called from NAPI callback, can't pass Rust closures
3. Use `Arc<Mutex<HashMap>>` shared reference → Rejected: more complex than `pub(crate)` static

### Decision 6: Fix `webview_version` check for OHOS

**Choice:** Guard `wry::webview_version()` call with `#[cfg(not(target_env = "ohos"))]` and return `true` for OHOS (WebView is always available).

**Rationale:**
- OHOS uses ArkUI's `Web` component, not WebView2/WebKit
- `wry::webview_version()` returns `Err` on OHOS because it checks for Windows WebView2 or macOS WebKit
- OHOS apps always have WebView support (built into ArkUI), so `webview_runtime_installed` should be `true`

**Implementation:**
```rust
webview_runtime_installed: {
  #[cfg(not(target_env = "ohos"))]
  {
    wry::webview_version().is_ok()
  }
  #[cfg(target_env = "ohos")]
  {
    true
  }
},
```

**Alternatives considered:**
1. Implement `wry::webview_version()` for OHOS → Rejected: OHOS doesn't have a "WebView version" concept, ArkUI Web is versioned with OS
2. Set to `false` for OHOS → Rejected: would prevent WebView creation, breaking all OHOS apps
3. Skip the check entirely → Rejected: Windows/macOS still need it for graceful degradation

## Risks / Trade-offs

**[Risk] `PENDING_PLUGIN_CALLS_ID` overflow after 2^31 commands**
- **Likelihood:** Very Low (would require billions of plugin calls)
- **Impact:** Low (wraps to negative IDs, still unique)
- **Mitigation:** `i32` is sufficient for practical use; if needed, migrate to `i64` or `u64` in future

**[Risk] HAR file not found during build**
- **Likelihood:** Medium (plugin authors may not provide OHOS HAR files)
- **Impact:** Medium (build fails with "plugin HAR not found" error)
- **Mitigation:** CLI provides clear error message with expected HAR path; plugin template includes OHOS structure

**[Trade-off] Plugin detection via `Cargo.toml` prefix matching**
- Assumes all Tauri plugins follow `tauri-plugin-*` naming convention
- Third-party plugins not following this convention won't be auto-detected
- This is the existing convention for iOS/Android, so consistent behavior

**[Trade-off] Centralized OHOS state in `crate::ohos`**
- OHOS-specific code is isolated, but requires `#[cfg(target_env = "ohos")]` guards
- Cross-platform code (e.g., `plugin/mobile.rs`) must conditionally compile OHOS paths
- This matches the existing pattern for iOS/Android platform-specific code

**[Risk] NAPI callback thread safety**
- **Likelihood:** Low (NAPI callbacks are invoked on the main thread)
- **Impact:** Medium (race conditions if handler map is accessed from multiple threads)
- **Mitigation:** `PENDING_PLUGIN_CALLS` is protected by `Mutex`; NAPI callbacks are single-threaded by design

**[Trade-off] Dialog plugin uses OHOS native APIs instead of cross-platform abstraction**
- Dialog plugin's ArkTS code uses `@ohos.promptAction`, `@ohos.file.picker` directly
- Not portable to Windows/macOS (but those platforms have their own implementations)
- This is intentional: OHOS APIs are sufficiently different that abstraction would add complexity

## Open Questions

None. The implementation is complete and tested on OHOS desktop devices. Future work (e.g., additional plugins, OHOS-specific UI components) will be addressed in separate specs.

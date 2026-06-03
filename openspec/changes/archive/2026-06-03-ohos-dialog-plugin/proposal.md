## Why

OHOS (OpenHarmony OS) platform support in Tauri required a robust plugin system to bridge Rust backend logic with native ArkTS frontend components. While iOS and Android plugin systems were already functional, OHOS lacked:
1. A plugin registration mechanism in the Rust core
2. A command dispatch system to invoke plugin methods from ArkTS
3. CLI tooling to detect and bundle plugin HAR (Harmony Archive) files
4. Template structures for creating OHOS-compatible plugins

Without these foundations, plugins like `tauri-plugin-dialog` could not function on OHOS devices. The dialog plugin specifically needed native OHOS APIs (`@ohos.promptAction`) for file pickers, message dialogs, and confirmation dialogs, which differ significantly from Windows/macOS implementations.

## What Changes

- **Implement OHOS plugin registration system** in `crates/tauri/src/ohos.rs` and `crates/tauri/src/ohos_plugin.rs`
- **Add `run_command` implementation** for OHOS in `crates/tauri/src/plugin/mobile.rs` (previously a TODO stub)
- **Create plugin detection CLI** in `crates/tauri-cli/src/mobile/open_harmony/plugins.rs` to scan `Cargo.toml` for `tauri-plugin-*` dependencies
- **Add OHOS HAR module structure** in `crates/tauri/mobile/ohos/` with ArkTS `Plugin.ets` and `PluginManager.ets`
- **Provide dialog plugin template** in `crates/tauri-cli/templates/mobile/open-harmony/dialog/`
- **Fix OHOS-specific runtime issues** in `crates/tauri-runtime-wry/src/lib.rs` (webview_version check, cfg guards)
- **Enable plugin API registration** via `PluginApi::register_ohos_plugin` method

## Capabilities

### New Capabilities
- `ohos-plugin-system`: Complete plugin infrastructure for OHOS, including registration, command dispatch, and HAR file management
- `ohos-dialog-plugin`: Dialog plugin implementation for OHOS using native `@ohos.promptAction` APIs for file selection, messages, and confirmations

### Modified Capabilities
- `plugin-mobile`: Extended to support OHOS alongside iOS and Android; `PENDING_PLUGIN_CALLS` made public for cross-module access; `run_command` implemented for OHOS

## Impact

- **Code**: `crates/tauri/src/ohos.rs` (+56 lines), `crates/tauri/src/ohos_plugin.rs` (+98 lines, new file), `crates/tauri/src/plugin/mobile.rs` (+66 lines), `crates/tauri-cli/src/mobile/open_harmony/plugins.rs` (+741 lines, new file), `crates/tauri-runtime-wry/src/lib.rs` (+64 lines)
- **APIs**: `PluginApi::register_ohos_plugin` (new public method), `dispatch_run_command` (new OHOS-specific function), `OhosPlugin::register` (new plugin registration API)
- **Dependencies**: `walkdir` crate added to `tauri-cli` for HAR file discovery; `tauri-plugin-dialog` now depends on OHOS plugin infrastructure
- **Systems**: OHOS platform support expanded; CLI build process enhanced to detect and bundle plugin HAR files; template system extended for OHOS plugin scaffolding
- **Demo**: `examples/api/src-tauri` updated to test dialog plugin on OHOS; `examples/api/src/lib/tests/plugins.ts` adds dialog test cases (+149 lines)

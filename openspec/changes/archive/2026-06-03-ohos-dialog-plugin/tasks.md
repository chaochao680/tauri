## 1. OHOS Plugin Registration System

- [x] 1.1 Create `crates/tauri/src/ohos_plugin.rs` with `OhosPlugin` struct and `register` method
- [x] 1.2 Add `PluginRegistration` struct to `crates/tauri/src/ohos.rs` (name, identifier, class_name, config)
- [x] 1.3 Add `PLUGINS_TO_REGISTER: Mutex<Vec<PluginRegistration>>` global state in `ohos.rs`
- [x] 1.4 Implement `PluginApi::register_ohos_plugin` method in `crates/tauri/src/plugin/mobile.rs`
- [x] 1.5 Create `register_ohos_plugin!` macro for declarative plugin registration

## 2. OHOS Command Dispatch System

- [x] 2.1 Make `PENDING_PLUGIN_CALLS` public (`pub(crate)`) in `plugin/mobile.rs`
- [x] 2.2 Add `RunCommandArgs` struct to `ohos.rs` (id, plugin_name, command, payload)
- [x] 2.3 Implement `dispatch_run_command` function in `ohos.rs` (NAPI call to ArkTS)
- [x] 2.4 Implement `run_command` for OHOS in `plugin/mobile.rs` (replaces TODO stub)
- [x] 2.5 Add callback ID generation using `PENDING_PLUGIN_CALLS_ID` atomic counter

## 3. CLI Plugin Detection and HAR Management

- [x] 3.1 Create `crates/tauri-cli/src/mobile/open_harmony/plugins.rs` module
- [x] 3.2 Implement `detect_plugins` function to scan `Cargo.toml` for `tauri-plugin-*` dependencies
- [x] 3.3 Implement `find_plugin_har` function to locate HAR files in `plugins/<name>/openharmony/`
- [x] 3.4 Add `PluginMeta` struct (name, identifier, class_name, har_path)
- [x] 3.5 Integrate plugin detection into `crates/tauri-cli/src/mobile/open_harmony/build.rs`
- [x] 3.6 Add `walkdir` dependency to `tauri-cli/Cargo.toml` for HAR file discovery

## 4. OHOS HAR Module Template

- [x] 4.1 Create `crates/tauri/mobile/ohos/src/main/ets/Plugin.ets` (NAPI entry point)
- [x] 4.2 Create `crates/tauri/mobile/ohos/src/main/ets/PluginManager.ets` (command dispatcher)
- [x] 4.3 Create `crates/tauri/mobile/ohos/src/main/ets/index.ets` (module exports)
- [x] 4.4 Create `crates/tauri/mobile/ohos/src/main/module.json5` (module configuration)
- [x] 4.5 Create `crates/tauri/mobile/ohos/build-profile.json5`, `hvigorfile.ts`, `oh-package.json5`

## 5. Dialog Plugin Template

- [x] 5.1 Create `crates/tauri-cli/templates/mobile/open-harmony/dialog/src/main/ets/Plugin.ets`
- [x] 5.2 Implement dialog commands: `open`, `save`, `message`, `ask`, `confirm` using OHOS APIs
- [x] 5.3 Create `crates/tauri-cli/templates/mobile/open-harmony/dialog/build-profile.json5`
- [x] 5.4 Create `crates/tauri-cli/templates/mobile/open-harmony/dialog/hvigorfile.ts`
- [x] 5.5 Create `crates/tauri-cli/templates/mobile/open-harmony/dialog/oh-package.json5`
- [x] 5.6 Create `crates/tauri-cli/templates/mobile/open-harmony/dialog/src/main/module.json5`

## 6. Runtime Fixes

- [x] 6.1 Guard `wry::webview_version()` with `#[cfg(not(target_env = "ohos"))]` in `tauri-runtime-wry/src/lib.rs`
- [x] 6.2 Return `true` for `webview_runtime_installed` on OHOS (WebView always available)
- [x] 6.3 Fix `pub use wry::webview_version` to exclude OHOS with `#[cfg(not(target_env = "ohos"))]`
- [x] 6.4 Fix indentation in `NewWindowResponse::Create` block (cosmetic)

## 7. CLI Template System Integration

- [x] 7.1 Add dialog plugin template to `crates/tauri-cli/templates/mobile/open-harmony/`
- [x] 7.2 Update `crates/tauri-cli/src/mobile/open_harmony/project.rs` to copy dialog template during `tauri init`
- [x] 7.3 Update `crates/tauri-cli/src/mobile/open_harmony/mod.rs` to register dialog plugin in generated project
- [x] 7.4 Update `crates/tauri-cli/src/mobile/init.rs` to include OHOS plugin initialization
- [x] 7.5 Create `EntryAbility.ets.hbs` template (replaces hardcoded `EntryAbility.ets`)

## 8. Build System Integration

- [x] 8.1 Update `crates/tauri/build.rs` to copy OHOS HAR module during build
- [x] 8.2 Update `crates/tauri-plugin/src/build/mobile.rs` to support OHOS plugin bundling
- [x] 8.3 Update `crates/tauri-plugin/src/build/mod.rs` to detect OHOS target
- [x] 8.4 Add `tauri/mobile/ohos` to `crates/tauri/Cargo.toml` package include list

## 9. Example Application Updates

- [x] 9.1 Add `tauri-plugin-dialog` dependency to `examples/api/src-tauri/Cargo.toml`
- [x] 9.2 Update `examples/api/src-tauri/src/cmd.rs` to use dialog plugin commands
- [x] 9.3 Update `examples/api/src-tauri/src/lib.rs` to register dialog plugin
- [x] 9.4 Add dialog test cases to `examples/api/src/lib/tests/plugins.ts` (+149 lines)
- [x] 9.5 Add dialog test UI to `examples/api/src/views/TestRunner.svelte` (+82 lines)
- [x] 9.6 Update `examples/api/src-tauri/capabilities/run-app.json` to grant dialog permissions

## 10. Path API OHOS Support

- [x] 10.1 Update `crates/tauri/src/path/mod.rs` to add OHOS-specific path resolution
- [x] 10.2 Update `crates/tauri/src/path/ohos.rs` to implement `resolve_resource`, `app_data_dir`, `app_config_dir` for OHOS
- [x] 10.3 Add OHOS path constants (cache, data, config directories)

## 11. Menu and Tray OHOS Integration

- [x] 11.1 Update `crates/tauri/src/menu/menu.rs` to support OHOS menu events
- [x] 11.2 Update `crates/tauri/src/tray/mod.rs` to integrate tray with OHOS plugin system
- [x] 11.3 Update `crates/tauri/src/window/mod.rs` to add OHOS window management hooks

## 12. Build and Deploy

- [x] 12.1 Run `ohos-build` skill to rebuild HAR + HAP with dialog plugin
- [x] 12.2 Deploy to OHOS desktop device for testing

## 13. Verification Testing

- [x] 13.1 Test dialog `open` command → File picker opens, selected file path returned to Rust
- [x] 13.2 Test dialog `save` command → Save picker opens, selected path returned to Rust
- [x] 13.3 Test dialog `message` command → Message dialog displays with OK button
- [x] 13.4 Test dialog `ask` command → Confirmation dialog displays with Yes/No buttons, boolean returned
- [x] 13.5 Test dialog `confirm` command → Confirmation dialog displays with OK/Cancel buttons, boolean returned
- [x] 13.6 Verify plugin auto-detection in CLI build logs
- [x] 13.7 Verify HAR file bundled in HAP package

## 14. Documentation

- [x] 14.1 Create openspec proposal.md, design.md, tasks.md
- [x] 14.2 Create specs/ohos-dialog-plugin/spec.md with formal requirements
- [x] 14.3 Add inline comments in `ohos.rs` explaining plugin registration flow
- [x] 14.4 Add inline comments in `plugin/mobile.rs` explaining command dispatch

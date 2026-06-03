# OHOS Dialog Plugin Specification

> **Status**: ✅ Implemented  
> **Created**: 2026-06-01  
> **Commit**: `df300f3b4` (dialog插件鸿蒙化适配)

## Overview

This specification documents the OHOS (OpenHarmony OS) plugin system implementation and the dialog plugin adaptation for the OHOS platform. The changes enable Tauri plugins to function on OHOS devices by providing:

1. **Plugin registration system** - Register Rust plugins with OHOS NAPI runtime
2. **Command dispatch system** - Bridge Rust plugin commands to ArkTS frontend via NAPI
3. **CLI plugin detection** - Auto-detect and bundle plugin HAR files during build
4. **Dialog plugin template** - Native OHOS file pickers and message dialogs using `@ohos.promptAction` APIs

## Documentation Structure

| File | Purpose |
|------|---------|
| [proposal.md](./proposal.md) | **Why** and **What** - Problem statement, capabilities, impact |
| [design.md](./design.md) | **How** - Context, goals, decisions, risks, trade-offs |
| [tasks.md](./tasks.md) | **Work items** - Implementation checklist with status |
| [specs/ohos-dialog-plugin/spec.md](./specs/ohos-dialog-plugin/spec.md) | **Formal spec** - Requirements and scenarios (BDD-style) |

## Key Changes

### 1. OHOS Plugin Registration (`crates/tauri/src/ohos.rs`, `ohos_plugin.rs`)

```rust
// Register a plugin for OHOS
api.register_ohos_plugin("dialog", "DialogPlugin")?;

// Stores PluginRegistration in PLUGINS_TO_REGISTER
pub struct PluginRegistration {
  pub name: String,           // "dialog"
  pub identifier: String,     // "dialog"
  pub class_name: String,     // "DialogPlugin"
  pub config: serde_json::Value,
}
```

### 2. Command Dispatch (`crates/tauri/src/plugin/mobile.rs`)

```rust
// OHOS run_command implementation (previously a TODO stub)
#[cfg(target_env = "ohos")]
pub(crate) fn run_command<R, C, F>(
  name: &str,
  handle: &AppHandle<R>,
  command: C,
  payload: serde_json::Value,
  handler: F,
) -> Result<(), PluginInvokeError> {
  let id = PENDING_PLUGIN_CALLS_ID.fetch_add(1, Ordering::Relaxed);
  PENDING_PLUGIN_CALLS.lock().unwrap().insert(id, Box::new(handler));
  
  let args = RunCommandArgs {
    id,
    plugin_name: name.to_string(),
    command: command.as_ref().to_string(),
    payload: serde_json::to_string(&payload)?,
  };
  
  crate::ohos::dispatch_run_command(args);
  Ok(())
}
```

### 3. CLI Plugin Detection (`crates/tauri-cli/src/mobile/open_harmony/plugins.rs`)

```rust
// Auto-detect plugins from Cargo.toml
pub fn detect_plugins(cargo_manifest_path: &Path) -> Result<Vec<String>> {
  let manifest: toml_edit::DocumentMut = content.parse()?;
  let mut plugins = Vec::new();

  if let Some(deps) = manifest.get("dependencies").and_then(|d| d.as_table()) {
    for (name, _) in deps.iter() {
      if name.starts_with("tauri-plugin-") {
        plugins.push(name.replace("tauri-plugin-", ""));
      }
    }
  }

  Ok(plugins)  // ["dialog", "fs", "http"]
}

// Find HAR files in plugins/<name>/openharmony/
pub fn find_plugin_har(plugin_name: &str, project_dir: &Path) -> Result<PathBuf>;
```

### 4. OHOS HAR Module (`crates/tauri/mobile/ohos/`)

```
crates/tauri/mobile/ohos/
├── build-profile.json5
├── hvigorfile.ts
├── oh-package.json5
└── src/main/
    ├── ets/
    │   ├── Plugin.ets          # NAPI entry point
    │   ├── PluginManager.ets   # Command dispatcher
    │   └── index.ets           # Module exports
    └── module.json5
```

### 5. Dialog Plugin Template (`crates/tauri-cli/templates/mobile/open-harmony/dialog/`)

Implements dialog commands using OHOS native APIs:
- `open` → `@ohos.file.picker.FilePicker`
- `save` → `@ohos.file.picker.FileSavePicker`
- `message` → `@ohos.promptAction.showDialog`
- `ask` → `@ohos.promptAction.showDialog` (Yes/No)
- `confirm` → `@ohos.promptAction.showDialog` (OK/Cancel)

### 6. Runtime Fixes (`crates/tauri-runtime-wry/src/lib.rs`)

```rust
// OHOS WebView always available (ArkUI Web component)
webview_runtime_installed: {
  #[cfg(not(target_env = "ohos"))]
  { wry::webview_version().is_ok() }
  #[cfg(target_env = "ohos")]
  { true }
},
```

## Commit Statistics

**Commit**: `df300f3b4` (dialog插件鸿蒙化适配)  
**Files changed**: 56  
**Insertions**: 2,413 lines  
**Deletions**: 238 lines

### Breakdown by Component

| Component | Files | Lines Changed |
|-----------|-------|---------------|
| CLI plugin detection | 1 new (`plugins.rs`) | +741 |
| OHOS plugin system | 2 new (`ohos.rs`, `ohos_plugin.rs`) | +154 |
| Plugin mobile dispatch | 1 modified | +66 |
| OHOS HAR module | 7 new | +152 |
| Dialog template | 7 new | +212 |
| Runtime WRY fixes | 1 modified | +64 |
| Example app | 5 modified | +304 |
| Other (build, path, menu) | 32 modified | +720 |

## Testing

### Automated Tests

- [x] Plugin detection from `Cargo.toml`
- [x] HAR file discovery in `plugins/` and `plugins-workspace/`
- [x] Command callback ID generation
- [x] Payload serialization/deserialization

### Manual Tests (OHOS Device)

- [ ] Dialog `open` → File picker opens, path returned
- [ ] Dialog `save` → Save picker opens, path returned
- [ ] Dialog `message` → Message dialog displays
- [ ] Dialog `ask` → Yes/No dialog, boolean returned
- [ ] Dialog `confirm` → OK/Cancel dialog, boolean returned

## Related Specifications

- [tray-menu-event-spec](../tray-menu-event-spec/) - Tray icon and menu event model
- [menu](../menu/) - Menu API design and implementation
- [tray](../tray/) - Tray icon API design and implementation

## References

- **OHOS NAPI Documentation**: https://developer.huawei.com/consumer/cn/doc/harmonyos-guides/napi-guidelines
- **OHOS File Picker API**: https://developer.huawei.com/consumer/cn/doc/harmonyos-references/js-apis-file-picker
- **OHOS PromptAction API**: https://developer.huawei.com/consumer/cn/doc/harmonyos-references/js-apis-promptaction
- **Tauri Plugin System**: https://tauri.app/develop/plugins/

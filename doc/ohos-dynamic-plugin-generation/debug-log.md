# DEBUG Log

## Issue 1: serde_json::Map IndexMut Panic

**Problem:** `cargo tauri ohos init` panicked at `project.rs:100:7: no entry found for key`

**Root Cause:** `data["plugins"] = json!(plugin_data)` — `serde_json::Map`'s `IndexMut` panics for missing keys.

**Fix:** Use `data.insert("plugins".to_string(), json!(plugin_data))` instead.

**File:** `crates/tauri-cli/src/mobile/open_harmony/project.rs` line 100

## Issue 2: .hbs Extension Not Stripped from Output

**Problem:** `EntryAbility.ets.hbs` template was output as `EntryAbility.ets.hbs` instead of `EntryAbility.ets`. The old hardcoded `EntryAbility.ets` with `@tauri/plugin-dialog` import persisted, causing ArkTS compiler error:

```
Cannot find module '@tauri/plugin-dialog' or its corresponding type declarations.
At File: entry/src/main/ets/entryability/EntryAbility.ets:6:26
```

**Root Cause:** The template rendering system (`template.rs`) only handled `.crate-manifest` → `.toml` extension renaming, but did not strip `.hbs` extensions from Handlebars template files. So `EntryAbility.ets.hbs` was output with the `.hbs` extension intact, and the old `EntryAbility.ets` remained unchanged.

**Fix:** Added `.hbs` extension stripping logic in `template.rs`, similar to the existing `.crate-manifest` → `.toml` handling:

```rust
} else if extension == "hbs" {
  let stem = file_path
    .file_stem()
    .unwrap()
    .to_string_lossy()
    .into_owned();
  file_path.set_file_name(stem);
}
```

This strips `.hbs` so `EntryAbility.ets.hbs` becomes `EntryAbility.ets` in the output.

**File:** `crates/tauri-cli/src/helpers/template.rs`

**Important:** After rebuilding tauri-cli, you must delete the old `gen/ohos` directory and re-run `cargo tauri ohos init` to regenerate the project with the corrected template output.

## Issue 3: dialog plugin not detected from target-specific dependencies

**Problem:** `tauri-plugin-dialog` was listed in `[target.'cfg(target_env = "ohos")'.dependencies]` but not in `[dependencies]`. The `detect_plugins()` function only scanned `[dependencies]`, so dialog was never detected. Result: `[PluginManager] Plugin not found: dialog`.

**Root Cause:** Cargo.toml allows platform-specific dependencies under `[target.X.dependencies]`. The detection only looked at the top-level `[dependencies]` section.

**Fix:** Modified `detect_plugins()` to also scan all `[target.*.dependencies]` sections:

```rust
if let Some(target_deps) = manifest.get("target").and_then(|t| t.as_table()) {
  for (_, target_val) in target_deps.iter() {
    if let Some(target_table) = target_val.as_table() {
      if let Some(deps) = target_table.get("dependencies").and_then(|d| d.as_table()) {
        collect_plugins_from_table(deps, &mut plugins);
      }
    }
  }
}
```

**File:** `crates/tauri-cli/src/mobile/open_harmony/plugins.rs`

## Issue 7: Handlebars quote-and-join helper missing during re-render

**Problem:** `populate_template()` called in `build.rs`/`dev.rs` with a new handlebars instance from `init::handlebars()`. This instance registered all helpers, but the `tauri-binary-args` variable was NOT in the map (it's added in `init::command()`, not in `handlebars()`). When rendering `hvigorfile.ts`, `{{quote-and-join tauri-binary-args}}` failed with `ParamTypeMismatchForName`.

**Root Cause:** The `init::handlebars()` function only creates the handlebars instance and inserts `app` data. The `tauri-binary` and `tauri-binary-args` are added later in `init::command()`. When `build.rs`/`dev.rs` called `init::handlebars()` separately, these variables were missing.

**Fix:** Removed `populate_template()` calls from `build.rs` and `dev.rs`. The init flow already renders the complete template with all variables and helpers. Build/dev only needs to:
1. `copy_plugin_har()` — copy any external plugin HARs
2. `update_plugin_configs()` — update build-profile.json5 and oh-package.json5 with plugin entries

This avoids re-rendering the entire template (which requires all handlebars helpers and variables) and instead only updates the specific config files that need plugin data.

## Issue 6: CARGO_MANIFEST_DIR not available at runtime

**Problem:** `get_builtin_template_har_path()` used `std::env::var("CARGO_MANIFEST_DIR")` to find the built-in dialog template. This env var is ONLY available during compile time (`env!()` macro works), NOT at runtime when `cargo tauri ohos init/build` executes. So the fallback to built-in templates never worked.

**Root Cause:** The dialog HAR IS embedded in the binary via `include_dir!` macro (compile-time). `populate_template()` renders it automatically. But `find_plugin_har()` tried to find it on the filesystem at runtime, which failed.

**Fix:** Replaced filesystem-based fallback with `BUILTIN_PLUGINS` constant that provides hardcoded metadata (identifier, className) for known built-in plugins. Built-in plugins:
- Don't need `find_plugin_har()` — their HAR is already in `TEMPLATE_DIR`
- Don't need `copy_plugin_har()` — `populate_template()` renders it automatically
- Only need metadata (name, identifier, className) for EntryAbility.ets.hbs template rendering

```rust
const BUILTIN_PLUGINS: &[(&str, &str, &str)] = &[
  ("dialog", "@tauri/plugin-dialog", "DialogPlugin"),
];
```

In `detect_all_plugins()`, if a plugin matches a builtin entry, it's included with a `__builtin__` har_path marker. In `parse_plugin_meta()`, builtin plugins use hardcoded metadata instead of parsing oh-package.json5. In `copy_plugin_har()`, builtin plugins skip copying since `populate_template()` handles it.
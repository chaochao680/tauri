# Phase 6: Build Integration

## Objective

Integrate all phases into the existing build flow of `cargo tauri ohos build`.

## Implementation

### 1. Modify Build Command

**File:** `crates/tauri-cli/src/mobile/open_harmony/build.rs`

```rust
use super::plugins::{
    detect_all_plugins,
    parse_plugin_meta,
    copy_plugin_har,
    update_plugin_configs,
    validate_plugin_meta,
    PluginMeta,
    DetectedPlugin,
};
use super::project::populate_template;
use super::plugins::validate_plugin_configs;

pub fn command(options: Options, noise_level: NoiseLevel) -> Result<()> {
    log::info!("Starting OpenHarmony build with dynamic plugin injection");

    let detected_plugins = detect_all_plugins(src_tauri_dir)?;

    let plugin_metadata: Vec<PluginMeta> = if detected_plugins.is_empty() {
        log::info!("No OpenHarmony-compatible plugins detected");
        vec![]
    } else {
        log::info!(
            "Detected {} OpenHarmony plugins: {:?}",
            detected_plugins.len(),
            detected_plugins.iter().map(|p| &p.name).collect::<Vec<_>>()
        );

        let metadata: Vec<PluginMeta> = detected_plugins
            .iter()
            .map(|d| parse_plugin_meta(&d.har_path, &d.name))
            .collect::<Result<Vec<_>>>()?;

        for plugin in &metadata {
            validate_plugin_meta(plugin)?;
        }

        let ohos_dir = project_dir;
        for plugin in &metadata {
            copy_plugin_har(plugin, ohos_dir)?;
        }

        update_plugin_configs(ohos_dir, &metadata)?;
        validate_plugin_configs(ohos_dir, &metadata)?;

        metadata
    };

    let (handlebars, map) = template_data(app, config);

    project::populate_template(handlebars, map, plugin_metadata, &project_dir)?;

    log::info!("Build completed successfully with {} plugins", plugin_metadata.len());
    Ok(())
}
```

### 2. Project Template Function

**Note:** `populate_template` is defined in Phase 5 (`project.rs`) with validation. Phase 6 just calls it — do NOT redefine it here.

Phase 6's `build.rs` only needs to pass `plugin_metadata` to it:

```rust
project::populate_template(handlebars, map, plugin_metadata, &project_dir)?;
```

### 3. Modify Init Command

**File:** `crates/tauri-cli/src/mobile/open_harmony/init.rs` or `mod.rs`

For `cargo tauri ohos init`:

```rust
pub fn init_command(/* ... */) -> Result<()> {
    // ... existing init logic ...

    // After template is generated, inject plugins
    let src_tauri_dir = /* get path */;
    let detected = detect_all_plugins(src_tauri_dir)?;

    if !detected.is_empty() {
        let metadata = detected
            .iter()
            .map(|d| parse_plugin_meta(&d.har_path, &d.name))
            .collect::<Result<Vec<_>>>()?;

        // Copy HARs
        for plugin in &metadata {
            copy_plugin_har(plugin, &project_dir)?;
        }

        // Update configs
        update_plugin_configs(&project_dir, &metadata)?;
    }

    // ... rest of init ...
    Ok(())
}
```

### 4. Remove Hardcoded Dialog from Templates

**Files to clean up:**

- `templates/mobile/open-harmony/build-profile.json5`: Remove dialog module
- `templates/mobile/open-harmony/entry/oh-package.json5`: Remove dialog dependency
- `templates/mobile/open-harmony/dialog/`: Remove entire directory (will be copied dynamically)
- Rename `EntryAbility.ets` → `EntryAbility.ets.hbs`

```bash
# Remove hardcoded plugin
rm -rf crates/tauri-cli/templates/mobile/open-harmony/dialog

# Update build-profile.json5 template
# Remove dialog from modules array

# Update entry/oh-package.json5 template
# Remove @tauri/plugin-dialog dependency

# Rename EntryAbility.ets to template
mv EntryAbility.ets EntryAbility.ets.hbs
# Update content to use Handlebars syntax (see Phase 5)
```

### 5. Build Flow Diagram

```
cargo tauri ohos build --device-type desktop
│
├─► Parse Cargo.toml
│   └─► detect_plugins() → ["dialog", "fs", ...]
│
├─► Find HAR directories
│   └─► find_plugin_har("dialog") → PathBuf
│
├─► Parse metadata
│   └─► parse_plugin_meta() → PluginMeta { name, identifier, className }
│
├─► Generate template
│   ├─► populate_template(plugins)
│   │   └─► Render EntryAbility.ets.hbs with plugins data
│   │
│   ├─► Copy HARs
│   │   └─► copy_plugin_har() → gen/ohos/dialog/
│   │
│   └─► Update configs
│       ├─► update_build_profile() → add modules
│       └─► update_entry_package() → add dependencies
│
├─► Compile Rust (existing)
│   └─► cargo build --target aarch64-unknown-linux-ohos
│
├─► Build HAR (existing)
│   └─► hvigorw assembleHar
│
├─► Build entry (existing)
│   └─► hvigorw assembleHap
│
└─► Package and install (existing)
```

### 6. Error Handling

```rust
pub fn command(options: Options, noise_level: NoiseLevel) -> Result<()> {
    log::info!("Starting OpenHarmony build with dynamic plugin injection");

    let detected_plugins = detect_all_plugins(src_tauri_dir)
        .context("Plugin detection failed")?;

    let plugin_metadata: Vec<PluginMeta> = if detected_plugins.is_empty() {
        log::info!("No OpenHarmony-compatible plugins detected, continuing build");
        vec![]
    } else {
        log::info!(
            "Detected {} OpenHarmony plugins: {:?}",
            detected_plugins.len(),
            detected_plugins.iter().map(|p| &p.name).collect::<Vec<_>>()
        );

        let metadata: Vec<PluginMeta> = detected_plugins
            .iter()
            .map(|d| parse_plugin_meta(&d.har_path, &d.name))
            .collect::<Result<Vec<_>>>()
            .context("Plugin metadata parsing failed")?;

        for plugin in &metadata {
            validate_plugin_meta(plugin)
                .context(format!("Invalid metadata for plugin '{}'", plugin.name))?;
        }

        let ohos_dir = project_dir;
        for plugin in &metadata {
            copy_plugin_har(plugin, ohos_dir)
                .context(format!("Failed to copy plugin '{}' HAR", plugin.name))?;
        }

        update_plugin_configs(ohos_dir, &metadata)
            .context("Failed to update plugin configurations")?;

        validate_plugin_configs(ohos_dir, &metadata)
            .context("Plugin configuration validation failed")?;

        metadata
    };

    project::populate_template(handlebars, map, plugin_metadata, &project_dir)?;

    log::info!("Build completed successfully with {} plugins", plugin_metadata.len());
    Ok(())
}
```

### 7. Error Recovery Strategy

```rust
#[derive(Debug)]
pub enum PluginError {
    DetectionFailed(String),
    MetadataInvalid { plugin: String, reason: String },
    PathTraversal { plugin: String, path: String },
    CopyFailed { plugin: String, error: String },
    ConfigUpdateFailed(String),
    TemplateInjection(String),
}

impl PluginError {
    pub fn is_recoverable(&self) -> bool {
        matches!(self, PluginError::DetectionFailed(_))
    }

    pub fn recovery_action(&self) -> &'static str {
        match self {
            PluginError::DetectionFailed(_) => "Continue build without plugins",
            PluginError::MetadataInvalid { .. } => "Skip invalid plugin",
            PluginError::PathTraversal { .. } => "Abort build - security issue",
            PluginError::CopyFailed { .. } => "Abort build - missing plugin",
            PluginError::ConfigUpdateFailed(_) => "Abort build - invalid state",
            PluginError::TemplateInjection(_) => "Abort build - security issue",
        }
    }
}
```

### 8. Logging and Debugging

Add detailed logging for troubleshooting:

```rust
log::info!("=== OpenHarmony Plugin Injection ===");
log::info!("Project dir: {}", project_dir.display());
log::info!("Cargo manifest: {}", cargo_manifest.display());

for plugin in &detected_plugins {
    log::info!("Detected: {}", plugin.name);
    log::info!("  HAR path: {}", plugin.har_path.display());
}

for meta in &plugin_metadata {
    log::info!("Metadata for '{}':", meta.name);
    log::info!("  identifier: {}", meta.identifier);
    log::info!("  className: {}", meta.class_name);
}

log::info!("Copied HARs to:");
for plugin in &plugin_metadata {
    log::info!("  {} -> gen/ohos/{}/", plugin.name, plugin.name);
}

log::info!("Updated configs:");
log::info!("  build-profile.json5: {} modules", modules_count);
log::info!("  oh-package.json5: {} dependencies", deps_count);
```

### 9. Integration Testing

```rust
#[test]
fn test_full_build_flow() {
    // Setup test project
    let test_project = setup_test_project_with_plugins(&["dialog"]);

    // Run build
    let result = command(Options::default(), NoiseLevel::default());

    assert!(result.is_ok());

    // Verify plugin injection
    let project_dir = test_project.path().join("gen/ohos");
    assert!(project_dir.join("dialog").exists());

    // Verify EntryAbility
    let entry_ability = fs::read_to_string(
        project_dir.join("entry/src/main/ets/entryability/EntryAbility.ets")
    ).unwrap();
    assert!(entry_ability.contains("import DialogPlugin"));
    assert!(entry_ability.contains("STATIC_PLUGINS.set('dialog'"));
}
```

## Configuration Options

Optional: Allow users to disable dynamic injection:

```rust
// In Options struct
pub struct Options {
    // ... existing options ...

    /// Skip automatic plugin injection
    #[clap(long = "skip-plugins")]
    pub skip_plugins: bool,
}

// In build command
if !options.skip_plugins {
    // Inject plugins
} else {
    log::info!("Skipping automatic plugin injection");
}
```

## Summary of Modified Files

| File                              | Changes                                      |
| --------------------------------- | -------------------------------------------- |
| `build.rs`                        | Add plugin detection and injection calls     |
| `project.rs`                      | Modify `populate_template` to accept plugins |
| `plugins.rs`                      | NEW: All plugin handling functions           |
| `mod.rs`                          | Export plugins module                        |
| `build-profile.json5` template    | Remove hardcoded dialog module               |
| `entry/oh-package.json5` template | Remove hardcoded dialog dependency           |
| `EntryAbility.ets`                | Rename to `.hbs`, add Handlebars syntax      |
| `dialog/` template directory      | DELETE (will be copied dynamically)          |

## Rollback Plan

If issues arise:

1. Restore `dialog/` directory in templates
2. Restore hardcoded configs
3. Add `--skip-plugins` flag to bypass dynamic injection
4. Fall back to static plugin registration

## Next Steps After Implementation

1. Test with single plugin (dialog)
2. Test with multiple plugins
3. Test with no plugins
4. Test with plugin that has no OpenHarmony support
5. Test init command
6. Test build command
7. Test dev command
8. Document usage in README
9. Update plugin development guide

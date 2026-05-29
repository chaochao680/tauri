# Phase 4: Configuration Update

## Objective

Update `build-profile.json5` and `entry/oh-package.json5` to register copied plugins.

## Implementation

### 1. Add Config Update Functions

CAUTION: Using typed structs (e.g., `BuildProfile`) with serde causes **data loss** — unknown fields are silently dropped on deserialize and never written back. Use `serde_json::Value` instead to preserve all existing content.

```rust
use serde_json::Value;

/// Update build-profile.json5 with plugin modules
pub fn update_build_profile(project_dir: &Path, plugins: &[PluginMeta]) -> Result<()> {
    let build_profile_path = project_dir.join("build-profile.json5");

    log::info!("Updating build-profile.json5 with {} plugins", plugins.len());

    let content = fs::read_to_string(&build_profile_path)
        .context("failed to read build-profile.json5")?;

    let mut profile: Value = parse_json5(&content)
        .context("failed to parse build-profile.json5")?;

    let modules = profile.get_mut("modules")
        .and_then(|v| v.as_array_mut())
        .context("build-profile.json5 has no 'modules' array")?;

    for plugin in plugins {
        if !modules.iter().any(|m| m.get("name").and_then(|v| v.as_str()) == Some(&plugin.name)) {
            log::info!("Adding module for plugin '{}'", plugin.name);

            let module = serde_json::json!({
                "name": plugin.name,
                "srcPath": format!("./{}", plugin.name),
                "targets": [{
                    "name": "default",
                    "applyToProducts": ["default"]
                }]
            });

            modules.push(module);
        } else {
            log::info!("Module '{}' already exists, skipping", plugin.name);
        }
    }

    let updated = serialize_json5(&profile)?;
    fs::write(&build_profile_path, updated)
        .context("failed to write build-profile.json5")?;

    log::info!("Successfully updated build-profile.json5");
    Ok(())
}

/// Update entry/oh-package.json5 with plugin dependencies
pub fn update_entry_package(project_dir: &Path, plugins: &[PluginMeta]) -> Result<()> {
    let oh_package_path = project_dir.join("entry/oh-package.json5");

    log::info!("Updating entry/oh-package.json5 with {} plugins", plugins.len());

    let content = fs::read_to_string(&oh_package_path)
        .context("failed to read entry/oh-package.json5")?;

    let mut package: Value = parse_json5(&content)
        .context("failed to parse entry/oh-package.json5")?;

    let dependencies = package.get_mut("dependencies")
        .and_then(|v| v.as_object_mut())
        .context("entry/oh-package.json5 has no 'dependencies' object")?;

    for plugin in plugins {
        if !dependencies.contains_key(&plugin.identifier) {
            log::info!(
                "Adding dependency '{}' -> '{}'",
                plugin.identifier,
                format!("file:../{}", plugin.name)
            );

            dependencies.insert(
                plugin.identifier.clone(),
                Value::String(format!("file:../{}", plugin.name)),
            );
        } else {
            log::info!("Dependency '{}' already exists", plugin.identifier);
        }
    }

    let updated = serialize_json5(&package)?;
    fs::write(&oh_package_path, updated)
        .context("failed to write entry/oh-package.json5")?;

    log::info!("Successfully updated entry/oh-package.json5");
    Ok(())
}

fn serialize_json5<T: Serialize>(value: &T) -> Result<String> {
    let json = serde_json::to_string_pretty(value)?;

    let lines: Vec<&str> = json.lines().collect();

    let formatted = lines
        .iter()
        .enumerate()
        .map(|(idx, line)| {
            let trimmed = line.trim();
            let next_trimmed = lines.get(idx + 1).map(|l| l.trim()).unwrap_or("");
            
            if trimmed.is_empty() || trimmed.starts_with("//") {
                line.to_string()
            } else if next_trimmed.starts_with("}") || next_trimmed.starts_with("]") {
                line.to_string()
            } else if trimmed.ends_with("{") || trimmed.ends_with("[") || trimmed.ends_with(",") {
                line.to_string()
            } else if trimmed.ends_with("}") || trimmed.ends_with("]") {
                line.to_string()
            } else {
                format!("{}{}", line, ",")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    Ok(formatted)
}
```

## Validation Before Update

Before adding dependencies, verify plugin HAR exists and is valid:

```rust
pub fn verify_plugin_before_update(plugin: &PluginMeta, project_dir: &Path) -> Result<()> {
    let plugin_dir = project_dir.join(&plugin.name);
    
    if !plugin_dir.exists() {
        return Err(anyhow::anyhow!(
            "Plugin HAR directory '{}' does not exist at {}",
            plugin.name,
            plugin_dir.display()
        ));
    }

    let oh_package_path = plugin_dir.join("oh-package.json5");
    if !oh_package_path.exists() {
        return Err(anyhow::anyhow!(
            "Required oh-package.json5 missing in plugin '{}'",
            plugin.name
        ));
    }

    let module_json5_path = plugin_dir.join("src/main/module.json5");
    if !module_json5_path.exists() {
        return Err(anyhow::anyhow!(
            "Required module.json5 missing in plugin '{}'",
            plugin.name
        ));
    }

    let oh_package = parse_oh_package(&plugin_dir)?;
    if oh_package.name != plugin.identifier {
        return Err(anyhow::anyhow!(
            "Plugin identifier mismatch: expected '{}', found '{}' in oh-package.json5",
            plugin.identifier,
            oh_package.name
        ));
    }

    Ok(())
}

pub fn verify_all_plugins_before_update(plugins: &[PluginMeta], project_dir: &Path) -> Result<()> {
    for plugin in plugins {
        verify_plugin_before_update(plugin, project_dir)
            .context(format!("Plugin '{}' verification failed", plugin.name))?;
    }
    Ok(())
}
```

### 2. Combined Update Function

```rust
/// Update all necessary config files for plugins
pub fn update_plugin_configs(project_dir: &Path, plugins: &[PluginMeta]) -> Result<()> {
    log::info!("Updating configurations for {} plugins", plugins.len());

    verify_all_plugins_before_update(plugins, project_dir)?;

    update_build_profile(project_dir, plugins)?;

    update_entry_package(project_dir, plugins)?;

    log::info!("All plugin configurations updated successfully");
    Ok(())
}
```

## Example Output

### build-profile.json5 (before)

```json5
{
  "modules": [
    {
      "name": "entry",
      "srcPath": "./entry",
      "targets": [...]
    },
    {
      "name": "tauri",
      "srcPath": "./tauri",
      "targets": [...]
    }
  ]
}
```

### build-profile.json5 (after adding dialog plugin)

```json5
{
  "modules": [
    {
      "name": "entry",
      "srcPath": "./entry",
      "targets": [...]
    },
    {
      "name": "tauri",
      "srcPath": "./tauri",
      "targets": [...]
    },
    {
      "name": "dialog",
      "srcPath": "./dialog",
      "targets": [
        {
          "name": "default",
          "applyToProducts": ["default"],
        }
      ]
    }
  ]
}
```

### entry/oh-package.json5 (before)

```json5
{
  dependencies: {
    '@ohos-rs/ability': '0.4.0-beta.7',
    '@tauri/app': 'file:../tauri'
  }
}
```

### entry/oh-package.json5 (after)

```json5
{
  dependencies: {
    '@ohos-rs/ability': '0.4.0-beta.7',
    '@tauri/app': 'file:../tauri',
    '@tauri/plugin-dialog': 'file:../dialog'
  }
}
```

## Edge Cases

1. **Plugin already in config**: Skip silently (don't duplicate)
2. **Config file missing**: Create from template
3. **Invalid JSON5**: Return error with context

## Validation

```rust
/// Validate that configs correctly reference plugins
pub fn validate_plugin_configs(project_dir: &Path, plugins: &[PluginMeta]) -> Result<()> {
    // Check build-profile
    let build_profile_path = project_dir.join("build-profile.json5");
    let content = fs::read_to_string(&build_profile_path)?;
    for plugin in plugins {
        if !content.contains(&format!("\"name\": \"{}\"", plugin.name)) {
            return Err(anyhow::anyhow!(
                "Plugin '{}' not found in build-profile.json5 modules",
                plugin.name
            ));
        }
    }

    // Check entry package
    let oh_package_path = project_dir.join("entry/oh-package.json5");
    let content = fs::read_to_string(&oh_package_path)?;
    for plugin in plugins {
        if !content.contains(&plugin.identifier) {
            return Err(anyhow::anyhow!(
                "Plugin '{}' not found in entry/oh-package.json5 dependencies",
                plugin.identifier
            ));
        }
    }

    Ok(())
}
```

## Testing

```rust
#[test]
fn test_update_configs() {
    let temp_dir = tempfile::tempdir().unwrap();

    // Create minimal config files
    fs::write(
        temp_dir.path().join("build-profile.json5"),
        "{\"app\": {}, \"modules\": []}"
    ).unwrap();

    fs::create_dir_all(temp_dir.path().join("entry")).unwrap();
    fs::write(
        temp_dir.path().join("entry/oh-package.json5"),
        "{\"name\": \"entry\", \"dependencies\": {}}"
    ).unwrap();

    let plugins = vec![
        PluginMeta {
            name: "dialog",
            identifier: "@tauri/plugin-dialog",
            class_name: "DialogPlugin",
            har_path: PathBuf::new(),
        }
    ];

    update_plugin_configs(temp_dir.path(), &plugins).unwrap();

    // Verify
    let build_content = fs::read_to_string(temp_dir.path().join("build-profile.json5")).unwrap();
    assert!(build_content.contains("\"name\": \"dialog\""));
}
```

## Next Phase

Phase 5 will template `EntryAbility.ets` with dynamic plugin imports and registration.

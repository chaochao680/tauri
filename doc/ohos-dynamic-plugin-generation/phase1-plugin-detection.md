# Phase 1: Plugin Detection

## Objective

Parse `Cargo.toml` to detect `tauri-plugin-*` dependencies and locate their OpenHarmony HAR directories.

## Implementation

### 1. Create New File: `plugins.rs`

**File:** `crates/tauri-cli/src/mobile/open_harmony/plugins.rs`

```rust
use crate::Result;
use anyhow::Context;
use std::path::{Path, PathBuf};

/// Represents a detected Tauri plugin for OpenHarmony
pub struct DetectedPlugin {
    /// Plugin name without prefix, e.g., "dialog"
    pub name: String,
    /// Path to the plugin's OpenHarmony HAR directory
    pub har_path: PathBuf,
}

/// Detect tauri-plugin-* dependencies from Cargo.toml
pub fn detect_plugins(cargo_manifest_path: &Path) -> Result<Vec<String>> {
    use cargo_toml::Manifest;

    let manifest = Manifest::from_path(cargo_manifest_path)
        .context("failed to parse Cargo.toml")?;

    let plugins: Vec<String> = manifest
        .dependencies
        .keys()
        .filter_map(|name| {
            // Match tauri-plugin-* pattern
            if name.starts_with("tauri-plugin-") {
                Some(name.replace("tauri-plugin-", ""))
            } else {
                None
            }
        })
        .collect();

    log::info!("Detected {} plugins: {:?}", plugins.len(), plugins);

    Ok(plugins)
}

/// Find the OpenHarmony HAR directory for a plugin
pub fn find_plugin_har(plugin_name: &str, project_dir: &Path) -> Result<PathBuf> {
    validate_plugin_name(plugin_name)?;

    let canonical_project = project_dir.canonicalize()
        .context("failed to canonicalize project directory")?;

    let search_paths: Vec<PathBuf> = vec![
        canonical_project.join("plugins").join(plugin_name).join("openharmony"),
        canonical_project
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("plugins-workspace").join("plugins").join(plugin_name).join("openharmony"))
            .unwrap_or_default(),
        get_tauri_workspace_root()
            .join("plugins-workspace")
            .join("plugins")
            .join(plugin_name)
            .join("openharmony"),
    ];

    for path in search_paths {
        if path.exists() {
            verify_path_within_allowed_scope(&path, &canonical_project)?;

            log::info!("Found plugin '{}' at: {}", plugin_name, path.display());
            return Ok(path);
        }
    }

    log::warn!("Plugin '{}' not found in any search path, may not support OpenHarmony", plugin_name);

    Err(anyhow::anyhow!(
        "Plugin '{}' OpenHarmony HAR not found. Searched paths:\n{}",
        plugin_name,
        search_paths.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join("\n")
    ))
}

fn verify_path_within_allowed_scope(path: &Path, project_dir: &Path) -> Result<()> {
    let path_lower = path.to_string_lossy().to_lowercase();
    let project_lower = project_dir.to_string_lossy().to_lowercase();

    if !path_lower.starts_with(&project_lower) {
        let workspace_root = get_tauri_workspace_root();
        let workspace_lower = workspace_root.to_string_lossy().to_lowercase();
        if !path_lower.starts_with(&workspace_lower) {
            return Err(anyhow::anyhow!(
                "Plugin path '{}' is outside allowed scope (project or tauri workspace)",
                path.display()
            ));
        }
    }

    Ok(())
}

/// Get all detected plugins with their HAR paths
pub fn detect_all_plugins(project_dir: &Path) -> Result<Vec<DetectedPlugin>> {
    let cargo_manifest = project_dir.join("Cargo.toml");

    if !cargo_manifest.exists() {
        return Err(anyhow::anyhow!("Cargo.toml not found at {}", cargo_manifest.display()));
    }

    let plugin_names = detect_plugins(&cargo_manifest)?;

    let detected: Vec<DetectedPlugin> = plugin_names
        .iter()
        .filter_map(|name| {
            match find_plugin_har(name, project_dir) {
                Ok(har_path) => Some(DetectedPlugin {
                    name: name.clone(),
                    har_path,
                }),
                Err(e) => {
                    log::warn!("Skipping plugin '{}': {}", name, e);
                    None
                }
            }
        })
        .collect();

    if detected.len() != plugin_names.len() {
        let missing: Vec<&String> = plugin_names
            .iter()
            .filter(|n| !detected.iter().any(|d| d.name == *n))
            .collect();
        log::warn!(
            "{} plugins not found (no OpenHarmony HAR): {:?}",
            missing.len(),
            missing
        );
    }

    log::info!("Successfully located {} plugin HARs", detected.len());

    Ok(detected)
}

/// Helper to get Tauri workspace root
fn get_tauri_workspace_root() -> PathBuf {
    // Try environment variable first
    if let Ok(root) = std::env::var("TAURI_WORKSPACE_ROOT") {
        return PathBuf::from(root);
    }

    // Calculate from crate manifest directory
    // crates/tauri-cli -> ../../ = workspace root
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_default();

    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or_default()
}
```

### 2. Update `mod.rs` to Export

**File:** `crates/tauri-cli/src/mobile/open_harmony/mod.rs`

Add at the top:

```rust
mod plugins;

pub use plugins::{DetectedPlugin, detect_all_plugins, detect_plugins, find_plugin_har};
```

### 3. Add Dependencies to `tauri-cli/Cargo.toml`

```toml
[dependencies]
cargo-toml = "0.15"  # For parsing Cargo.toml
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_plugins() {
        let manifest_path = Path::new("../../examples/api/src-tauri/Cargo.toml");
        let plugins = detect_plugins(manifest_path).unwrap();
        assert!(plugins.contains(&"dialog".to_string()));
    }

    #[test]
    fn test_find_plugin_har() {
        let project_dir = Path::new("../../examples/api/src-tauri");
        let har_path = find_plugin_har("dialog", project_dir).unwrap();
        assert!(har_path.join("oh-package.json5").exists());
    }
}
```

## Edge Cases

1. **Plugin without OpenHarmony support**: Return error or skip silently
2. **Custom plugin location**: Allow `TAURI_PLUGIN_PATH_<name>` env var override
3. **Multiple versions**: Use workspace dependencies resolution

## Next Phase

Phase 2 will parse metadata from the detected HAR directories.

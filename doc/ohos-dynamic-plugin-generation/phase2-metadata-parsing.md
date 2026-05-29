# Phase 2: Metadata Parsing

## Objective

Extract plugin metadata (identifier, className) from `oh-package.json5` and infer class names.

## Implementation

### 1. Extend `plugins.rs`

**File:** `crates/tauri-cli/src/mobile/open_harmony/plugins.rs`

Add to existing file:

```rust
use serde::Deserialize;
use serde::de::DeserializeOwned;

/// OpenHarmony package.json5 structure (relevant fields)
#[derive(Debug, Deserialize)]
pub struct OhPackage {
    pub name: String,           // e.g., "@tauri/plugin-dialog"
    pub version: String,
    pub main: Option<String>,   // e.g., "src/main/ets/index.ets"
    #[serde(default)]
    pub dependencies: std::collections::HashMap<String, String>,
}

/// Complete plugin metadata for template generation
#[derive(Debug, Clone)]
pub struct PluginMeta {
    /// Simple name: "dialog"
    pub name: String,
    /// Package identifier: "@tauri/plugin-dialog"
    pub identifier: String,
    /// Exported class name: "DialogPlugin"
    pub class_name: String,
    /// HAR directory path
    pub har_path: PathBuf,
}

/// Parse oh-package.json5 from HAR directory
pub fn parse_oh_package(har_path: &Path) -> Result<OhPackage> {
    let oh_package_path = har_path.join("oh-package.json5");

    if !oh_package_path.exists() {
        return Err(anyhow::anyhow!(
            "oh-package.json5 not found at {}",
            oh_package_path.display()
        ));
    }

    let content = std::fs::read_to_string(&oh_package_path)
        .context("failed to read oh-package.json5")?;

    // Parse JSON5 (more lenient than JSON)
    let oh_package: OhPackage = parse_json5(&content)
        .context("failed to parse oh-package.json5")?;

    log::info!(
        "Parsed oh-package: name={}, version={}",
        oh_package.name,
        oh_package.version
    );

    Ok(oh_package)
}

/// Infer class name from plugin name
/// "dialog" -> "DialogPlugin"
/// "clipboard-manager" -> "ClipboardManagerPlugin"
pub fn infer_class_name(plugin_name: &str) -> String {
    // Convert to PascalCase
    let pascal = plugin_name
        .split('-')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<String>();

    format!("{}Plugin", pascal)
}

/// Parse complete metadata from HAR directory
pub fn parse_plugin_meta(har_path: &Path, plugin_name: &str) -> Result<PluginMeta> {
    let oh_package = parse_oh_package(har_path)?;

    // identifier from oh-package.name
    let identifier = oh_package.name;

    // class name inference
    // Try to read index.ets for export pattern first
    let class_name = try_parse_class_name_from_index(har_path)
        .unwrap_or_else(|| infer_class_name(plugin_name));

    Ok(PluginMeta {
        name: plugin_name.to_string(),
        identifier,
        class_name,
        har_path: har_path.to_path_buf(),
    })
}

/// Attempt to parse class name from index.ets export
/// export { default as DialogPlugin } from './Plugin';
fn try_parse_class_name_from_index(har_path: &Path) -> Option<String> {
    let index_path = har_path.join("src/main/ets/index.ets");

    if !index_path.exists() {
        return None;
    }

    let content = match std::fs::read_to_string(index_path) {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Failed to read index.ets: {}", e);
            return None;
        }
    };

    let patterns = [
        r"export\s+\{\s*default\s+as\s+(\w+Plugin)\s*\}",
        r"export\s+default\s+class\s+(\w+Plugin)",
        r"export\s+class\s+(\w+Plugin)\s+extends\s+Plugin",
    ];

    for pattern in patterns {
        let re = regex::Regex::new(pattern).ok()?;
        for caps in re.captures_iter(&content) {
            if let Some(m) = caps.get(1) {
                let class_name = m.as_str();
                if validate_class_name_pattern(class_name) {
                    return Some(class_name.to_string());
                }
            }
        }
    }

    None
}

fn validate_class_name_pattern(name: &str) -> bool {
    name.ends_with("Plugin") 
        && name.len() > 6
        && name.chars().all(|c| c.is_ascii_alphabetic())
        && name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
}

/// Parse JSON5 string using proper json5 crate
fn parse_json5<T: DeserializeOwned>(content: &str) -> Result<T> {
    json5::from_str(content)
        .context("failed to parse JSON5 content")
}
```

### 2. Add Dependencies

**File:** `tauri-cli/Cargo.toml`

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
json5 = "0.4"  # For parsing JSON5 files (OpenHarmony config format)
regex = "1"     # For parsing index.ets
```

### 3. Update Plugin Detection to Include Metadata

```rust
/// Get complete metadata for all detected plugins
pub fn get_all_plugin_metadata(project_dir: &Path) -> Result<Vec<PluginMeta>> {
    let detected = detect_all_plugins(project_dir)?;

    let metadata: Vec<PluginMeta> = detected
        .iter()
        .map(|p| parse_plugin_meta(&p.har_path, &p.name))
        .collect::<Result<Vec<_>>>()?;

    log::info!(
        "Parsed {} plugin metadata entries: {:?}",
        metadata.len(),
        metadata.iter().map(|m| &m.name).collect::<Vec<_>>()
    );

    Ok(metadata)
}
```

## Example Output

For `tauri-plugin-dialog`:

```rust
PluginMeta {
    name: "dialog",
    identifier: "@tauri/plugin-dialog",
    class_name: "DialogPlugin",
    har_path: PathBuf::from("plugins-workspace/plugins/dialog/openharmony"),
}
```

For `tauri-plugin-clipboard-manager`:

```rust
PluginMeta {
    name: "clipboard-manager",
    identifier: "@tauri/plugin-clipboard-manager",  // hypothetical
    class_name: "ClipboardManagerPlugin",
    har_path: PathBuf::from("..."),
}
```

## Plugin HAR Standardization Requirement

Each plugin must have:

```
openharmony/
├── oh-package.json5     # Required
│   {
│     "name": "@tauri/plugin-{name}",  # Standard naming
│     "main": "src/main/ets/index.ets",
│     "dependencies": {
│       "@tauri/app": "file:../tauri"  # Relative path
│     }
│   }
└── src/main/ets/
    ├── Plugin.ets       # Required
    └── index.ets        # Required
        # Must export: export { default as {Name}Plugin } from './Plugin';
```

## Testing

```rust
#[test]
fn test_infer_class_name() {
    assert_eq!(infer_class_name("dialog"), "DialogPlugin");
    assert_eq!(infer_class_name("clipboard-manager"), "ClipboardManagerPlugin");
    assert_eq!(infer_class_name("fs"), "FsPlugin");
}

#[test]
fn test_parse_plugin_meta() {
    let har_path = Path::new("../../plugins-workspace/plugins/dialog/openharmony");
    let meta = parse_plugin_meta(har_path, "dialog").unwrap();
    assert_eq!(meta.identifier, "@tauri/plugin-dialog");
    assert_eq!(meta.class_name, "DialogPlugin");
}
```

## Next Phase

Phase 3 will handle copying the HAR directory with path adjustments.

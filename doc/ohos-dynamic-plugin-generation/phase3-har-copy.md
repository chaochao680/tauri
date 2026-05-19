# Phase 3: HAR Copy and Path Adjustment

## Objective

Copy plugin HAR directories to the generated project and adjust dependency paths.

## Implementation

### 1. Add Copy Function to `plugins.rs`

```rust
use std::fs;
use walkdir::WalkDir;

/// Copy plugin HAR to destination with path adjustments
pub fn copy_plugin_har(meta: &PluginMeta, dest_dir: &Path) -> Result<PathBuf> {
    validate_plugin_name(&meta.name)?;

    let canonical_dest = dest_dir.canonicalize()
        .context("failed to canonicalize destination directory")?;

    let canonical_har = meta.har_path.canonicalize()
        .context("failed to canonicalize plugin HAR path")?;

    let plugin_dest = canonical_dest.join(&meta.name);

    log::info!(
        "Copying plugin '{}' from {} to {}",
        meta.name,
        canonical_har.display(),
        plugin_dest.display()
    );

    if plugin_dest.exists() {
        fs::remove_dir_all(&plugin_dest)
            .context("failed to remove existing plugin directory")?;
    }

    fs::create_dir_all(&plugin_dest)
        .context("failed to create plugin directory")?;

    for entry in WalkDir::new(&canonical_har)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let src_path = entry.path();
        let relative = src_path.strip_prefix(&canonical_har)
            .context("failed to strip prefix from source path")?;
        
        verify_relative_path_safe(relative)?;

        let dest_path = plugin_dest.join(relative);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&dest_path)?;
        } else {
            verify_path_within_destination(&dest_path, &plugin_dest)?;
            fs::copy(src_path, &dest_path)?;

            if relative.ends_with("oh-package.json5") ||
               relative.ends_with("build-profile.json5") {
                adjust_paths_in_file(&dest_path, &canonical_har, &canonical_dest)?;
            }
        }
    }

    log::info!("Successfully copied plugin '{}' HAR", meta.name);
    Ok(plugin_dest)
}

fn verify_relative_path_safe(relative: &Path) -> Result<()> {
    for component in relative.components() {
        match component {
            std::path::Component::Prefix(_) |
            std::path::Component::RootDir => {
                return Err(anyhow::anyhow!("Absolute path component in relative path"));
            },
            std::path::Component::ParentDir => {
                return Err(anyhow::anyhow!("Parent directory traversal in path"));
            },
            std::path::Component::Normal(name) => {
                let name_str = name.to_string_lossy();
                if name_str.contains("..") || name_str.contains('\\') {
                    return Err(anyhow::anyhow!("Unsafe path component: {}", name_str));
                }
            },
            std::path::Component::CurDir => {},
        }
    }
    Ok(())
}

fn verify_path_within_destination(dest_path: &Path, dest_root: &Path) -> Result<()> {
    let dest_path_str = dest_path.to_string_lossy();
    let dest_root_str = dest_root.to_string_lossy();
    
    if !dest_path_str.starts_with(&dest_root_str) {
        return Err(anyhow::anyhow!(
            "Destination path '{}' is outside plugin directory '{}'",
            dest_path.display(),
            dest_root.display()
        ));
    }
    
    Ok(())
}

/// Adjust relative paths in config files after copy
fn adjust_paths_in_file(file_path: &Path, original_base: &Path, new_base: &Path) -> Result<()> {
    let content = fs::read_to_string(file_path)?;

    let adjusted = content
        .replace("\"@tauri/app\": \"file:../../../tauri\"", "\"@tauri/app\": \"file:../tauri\"")
        .replace("\"@tauri/app\": \"file:../../tauri\"", "\"@tauri/app\": \"file:../tauri\"")
        .replace("../../../tauri", "../tauri")
        .replace("../../tauri", "../tauri");

    fs::write(file_path, adjusted)?;
    Ok(())
}

/// Copy all plugin HARs to destination
pub fn copy_all_plugin_hars(plugins: &[PluginMeta], dest_dir: &Path) -> Result<Vec<PathBuf>> {
    let copied: Vec<PathBuf> = plugins
        .iter()
        .map(|meta| copy_plugin_har(meta, dest_dir))
        .collect::<Result<Vec<_>>>()?;

    log::info!("Copied {} plugin HARs to {}", copied.len(), dest_dir.display());
    Ok(copied)
}
```

### 2. Add Dependencies

**File:** `tauri-cli/Cargo.toml`

```toml
[dependencies]
walkdir = "2"  # For recursive directory walking
```

## Path Adjustment Logic

### Before Copy (Plugin HAR)

```
plugins-workspace/plugins/dialog/openharmony/
├── oh-package.json5
│   "dependencies": {
│     "@tauri/app": "file:../../tauri"  # Points to tauri HAR in workspace
│   }
└── src/main/ets/
    └── Plugin.ets
        import { Plugin } from '@tauri/app';  # Uses identifier
```

### After Copy (Generated Project)

```
gen/ohos/
├── tauri/
├── dialog/
│   ├── oh-package.json5
│   │   "dependencies": {
│   │     "@tauri/app": "file:../tauri"  # Adjusted to point to gen/ohos/tauri
│   │   }
│   └── src/main/ets/
│       └── Plugin.ets
│           import { Plugin } from '@tauri/app';  # Unchanged (works via identifier)
```

## Key Points

1. **File path adjustments** only needed for `oh-package.json5` `file:` dependencies
2. **Import statements** in `.ets` files use package identifiers, not file paths
3. **Relative path depth** varies based on plugin HAR location

## Copy Verification

After copy, verify:

```rust
pub fn verify_plugin_har(dest_path: &Path) -> Result<()> {
    let required_files = [
        "oh-package.json5",
        "build-profile.json5",
        "hvigorfile.ts",
        "src/main/module.json5",
        "src/main/ets/index.ets",
        "src/main/ets/Plugin.ets",
    ];

    for file in required_files {
        let path = dest_path.join(file);
        if !path.exists() {
            return Err(anyhow::anyhow!(
                "Required file '{}' missing in plugin HAR at {}",
                file,
                dest_path.display()
            ));
        }
    }

    Ok(())
}
```

## Error Handling

```rust
// Handle partial copy failure
pub fn copy_plugin_har_safe(meta: &PluginMeta, dest_dir: &Path) -> Result<PathBuf> {
    let result = copy_plugin_har(meta, dest_dir);

    if result.is_err() {
        // Cleanup partial copy
        let plugin_dest = dest_dir.join(&meta.name);
        if plugin_dest.exists() {
            let _ = fs::remove_dir_all(&plugin_dest);
        }
    }

    result
}
```

## Testing

```rust
#[test]
fn test_copy_plugin_har() {
    let temp_dir = tempfile::tempdir().unwrap();
    let har_path = Path::new("../../plugins-workspace/plugins/dialog/openharmony");
    let meta = parse_plugin_meta(har_path, "dialog").unwrap();

    let copied = copy_plugin_har(&meta, temp_dir.path()).unwrap();

    // Verify files exist
    assert!(copied.join("oh-package.json5").exists());
    assert!(copied.join("src/main/ets/Plugin.ets").exists());

    // Verify path adjustment
    let content = fs::read_to_string(copied.join("oh-package.json5")).unwrap();
    assert!(content.contains("\"file:../tauri\""));
}
```

## Next Phase

Phase 4 will update `build-profile.json5` and `entry/oh-package.json5` to register the copied plugins.

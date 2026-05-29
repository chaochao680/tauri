# Phase 5: EntryAbility Templating

## Objective

Generate `EntryAbility.ets` with dynamic plugin imports and registration using Handlebars.

## Security Considerations

CRITICAL: Template injection prevention is required before implementing this phase.

### Input Validation

All plugin metadata must be validated before template rendering to prevent code injection:

```rust
/// Validate plugin metadata for template safety
pub fn validate_plugin_meta(meta: &PluginMeta) -> Result<()> {
    validate_plugin_name(&meta.name)?;
    validate_identifier(&meta.identifier)?;
    validate_class_name(&meta.class_name)?;
    Ok(())
}

fn validate_plugin_name(name: &str) -> Result<()> {
    if name.is_empty() || name.len() > 64 {
        return Err(anyhow::anyhow!("Plugin name must be 1-64 characters"));
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err(anyhow::anyhow!(
            "Plugin name must only contain alphanumeric, hyphen, or underscore"
        ));
    }
    if name.starts_with('-') || name.starts_with('_') {
        return Err(anyhow::anyhow!("Plugin name cannot start with hyphen or underscore"));
    }
    Ok(())
}

fn validate_identifier(identifier: &str) -> Result<()> {
    if !identifier.starts_with("@tauri/plugin-") {
        return Err(anyhow::anyhow!("Identifier must start with @tauri/plugin-"));
    }
    let name_part = identifier.trim_start_matches("@tauri/plugin-");
    validate_plugin_name(name_part)?;
    Ok(())
}

fn validate_class_name(class_name: &str) -> Result<()> {
    if !class_name.ends_with("Plugin") {
        return Err(anyhow::anyhow!("Class name must end with 'Plugin'"));
    }
    let base = class_name.trim_end_matches("Plugin");
    if base.is_empty() {
        return Err(anyhow::anyhow!("Class name base cannot be empty"));
    }
    if !base.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(anyhow::anyhow!("Class name base must only contain alphabetic characters"));
    }
    if !base.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
        return Err(anyhow::anyhow!("Class name must start with uppercase letter"));
    }
    Ok(())
}
```

### Template Escaping

CAUTION: Handlebars default HTML escaping (`&#39;`, `&quot;`, etc.) is **not suitable** for TypeScript/ArkTS code generation. HTML entities are invalid in TS syntax.

Instead, rely on the **input validation** above (which guarantees no special characters) and register a custom escaper that does NOT escape, since all values are already validated:

```rust
fn setup_handlebars() -> Handlebars<'static> {
    let mut handlebars = Handlebars::new();
    
    handlebars.set_strict_mode(true);

    handlebars.register_escape_fn(|s| s.to_string());

    handlebars
}
```

Using `register_escape_fn` with a no-op escaper is safe ONLY because `validate_plugin_meta` is called before rendering, ensuring all values are alphanumeric-only (no quotes, brackets, or script-breaking characters).

## Implementation

### 1. Create Template File

**File:** `crates/tauri-cli/templates/mobile/open-harmony/entry/src/main/ets/entryability/EntryAbility.ets.hbs`

```typescript
import { NativeAbility } from '@ohos-rs/ability'
import Want from '@ohos.app.ability.Want'
import { AbilityConstant } from '@kit.AbilityKit';
import window from '@ohos.window';
import { PluginManager, Plugin } from '@tauri/app';
{{#each plugins}}
import {{className}} from '{{identifier}}';
{{/each}}

interface TauriNativeModule {
  default?: TauriNativeModule;
  tauri_init_plugins?: (manager: PluginManager) => string;
  tauriInitPlugins?: (manager: PluginManager) => string;
  tauri_handle_plugin_response?: (id: number, success: boolean, payload: string) => void;
  tauriHandlePluginResponse?: (id: number, success: boolean, payload: string) => void;
}

interface PluginConfig {
  name: string;
  identifier: string;
  className: string;
  config: string;
}

const STATIC_PLUGINS: Map<string, Plugin> = new Map();
{{#each plugins}}
STATIC_PLUGINS.set('{{name}}', new {{className}}());
{{/each}}

export default class EntryAbility extends NativeAbility {
  public moduleName: string = "{{app.lib-name}}"
  public defaultPage: boolean = true
  public mode: 'xcomponent' | 'webview' = 'webview'

  async onCreate(want: Want, launchParam: AbilityConstant.LaunchParam): Promise<void> {
    console.info('[Tauri] ===== EntryAbility onCreate =====');
    await super.onCreate(want, launchParam);
    console.info('[Tauri] ===== super.onCreate done =====');
    await this.initTauriPlugins();
  }

  private async initTauriPlugins(): Promise<void> {
    console.info('[Tauri] ===== initTauriPlugins =====');

    const pluginManager: PluginManager = new PluginManager();
    console.info('[Tauri] PluginManager created');

    pluginManager.handlePluginResponse = (id: number, success: number, payload: string): void => {
      const libName = 'lib' + this.moduleName + '.so';
      import(libName).then((nativeModule: TauriNativeModule): void => {
        const actualModule = nativeModule.default ? nativeModule.default : nativeModule;
        const handler = actualModule?.tauri_handle_plugin_response ?? actualModule?.tauriHandlePluginResponse;
        if (handler != null) {
          handler(id, success === 1, payload);
        }
      }).catch((err: Error): void => {
        console.error('[Tauri] Failed to call handlePluginResponse: ' + err.message);
      });
    };

    pluginManager.setResponseHandler(pluginManager.handlePluginResponse);

    const libName = 'lib' + this.moduleName + '.so';
    console.info('[Tauri] Loading: ' + libName);

    try {
      const nativeModule = await import(libName) as TauriNativeModule;
      console.info('[Tauri] Native module loaded');
      console.info('[Tauri] Module keys: ' + Object.keys(nativeModule).join(', '));

      const actualModule = nativeModule.default ? nativeModule.default : nativeModule;
      console.info('[Tauri] Actual module keys: ' + Object.keys(actualModule).join(', '));

      if (actualModule?.tauri_init_plugins != null || actualModule?.tauriInitPlugins != null) {
        const initPlugins = actualModule.tauri_init_plugins ?? actualModule.tauriInitPlugins;
        if (initPlugins != null) {
          console.info('[Tauri] Calling initPlugins');
          const pluginsJson = initPlugins(pluginManager);
          const plugins: PluginConfig[] = JSON.parse(pluginsJson) as PluginConfig[];

          console.info('[Tauri] Loading ' + plugins.length + ' plugins');

          for (const plugin of plugins) {
            const staticPlugin: Plugin | undefined = STATIC_PLUGINS.get(plugin.name);
            if (staticPlugin != null) {
              pluginManager.load(plugin.name, staticPlugin, plugin.config);
              console.info('[Tauri] Plugin loaded: ' + plugin.name);
            } else {
              console.error('[Tauri] Plugin not in static registry: ' + plugin.name);
            }
          }
          console.info('[Tauri] All plugins loaded');
        }
      } else {
        console.warn('[Tauri] initPlugins not found in module');
      }
    } catch (e) {
      const err = e as Error;
      console.error('[Tauri] Error: ' + err.message);
      console.error('[Tauri] Stack: ' + err.stack);
    }
  }

  async onWindowStageCreate(windowStage: window.WindowStage): Promise<void> {
    console.info('[Tauri] ===== onWindowStageCreate =====');
    const win = windowStage.getMainWindowSync();
    await win.setWindowLayoutFullScreen(false);
    await super.onWindowStageCreate(windowStage);
    console.info('[Tauri] ===== super.onWindowStageCreate done =====');
  }
}
```

### 2. Modify Project Template Handling

**File:** `crates/tauri-cli/src/mobile/open_harmony/project.rs`

```rust
use handlebars::Handlebars;
use serde_json::json;
use super::plugins::{PluginMeta, validate_plugin_meta};

pub fn populate_template(
    handlebars: Handlebars,
    map: template::JsonMap,
    plugins: Vec<PluginMeta>,
    dest: &Path,
) -> Result<()> {
    log::info!("Populating template with {} plugins", plugins.len());

    for plugin in &plugins {
        validate_plugin_meta(plugin)
            .context(format!("Invalid plugin metadata for '{}'", plugin.name))?;
    }

    let plugin_data = plugins.iter().map(|p| {
        json!({
            "name": p.name,
            "identifier": p.identifier,
            "className": p.class_name,
        })
    }).collect::<Vec<_>>();

    let mut data = map.inner().clone();
    data["plugins"] = json!(plugin_data);

    template::render(&handlebars, &data, &TEMPLATE_DIR, dest)?;

    log::info!("Template populated successfully");
    Ok(())
}
```

### 3. Template Registration and Handlebars Setup

**File:** `crates/tauri-cli/src/mobile/open_harmony/project.rs`

**First, add dependency:**

**File:** `tauri-cli/Cargo.toml`

```toml
[dependencies]
handlebars = "5"  # For template rendering (EntryAbility.ets.hbs)
```

Configure Handlebars with strict mode, no-op escaper (safe because of input validation), and helpers:

```rust
use crate::helpers::template;
use handlebars::{Handlebars, Helper, Context, RenderContext, Output, HelperResult, RenderError};

fn setup_handlebars() -> Handlebars<'static> {
    let mut handlebars = Handlebars::new();
    
    handlebars.set_strict_mode(true);

    handlebars.register_escape_fn(|s| s.to_string());

    handlebars.register_helper("equals", Box::new(
        |h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut Output| -> HelperResult {
            if let (Some(a), Some(b)) = (h.param(0), h.param(1)) {
                if a.value() == b.value() {
                    out.write("true")?;
                }
            }
            Ok(())
        }
    ));

    handlebars
}
```

**IMPORTANT:** The no-op escape fn (`register_escape_fn(|s| s.to_string())`) is safe ONLY because `validate_plugin_meta` is called in `populate_template` before rendering, guaranteeing all values are alphanumeric-only. Never remove validation without restoring proper escaping.

### 4. Remove Old Static Template

**Action:** Delete or rename the old hardcoded template:

```bash
# Rename to keep as reference
mv EntryAbility.ets EntryAbility.ets.static-backup

# New template will be EntryAbility.ets.hbs
```

## Template Output Examples

### No Plugins

```typescript
import { PluginManager, Plugin } from '@tauri/app'

const STATIC_PLUGINS: Map<string, Plugin> = new Map()
```

### Single Plugin (dialog)

```typescript
import { PluginManager, Plugin } from '@tauri/app'
import DialogPlugin from '@tauri/plugin-dialog'

const STATIC_PLUGINS: Map<string, Plugin> = new Map()
STATIC_PLUGINS.set('dialog', new DialogPlugin())
```

### Multiple Plugins

```typescript
import { PluginManager, Plugin } from '@tauri/app'
import DialogPlugin from '@tauri/plugin-dialog'
import FsPlugin from '@tauri/plugin-fs'
import NotificationPlugin from '@tauri/plugin-notification'

const STATIC_PLUGINS: Map<string, Plugin> = new Map()
STATIC_PLUGINS.set('dialog', new DialogPlugin())
STATIC_PLUGINS.set('fs', new FsPlugin())
STATIC_PLUGINS.set('notification', new NotificationPlugin())
```

## Template Helpers (Optional)

Add custom Handlebars helpers for more complex scenarios:

```rust
// Helper to format class name
handlebars.register_helper("camelCase", Box::new(|h: &Helper, ...| {
    // "dialog" -> "Dialog"
    if let Some(param) = h.param(0) {
        let s = param.value().as_str().unwrap_or("");
        let camel = s.split('-')
            .map(|p| p.chars().take(1).map(|c| c.to_uppercase().collect()).chain(p.chars().skip(1)).collect())
            .collect::<String>();
        out.write(&camel)?;
    }
    Ok(())
}));
```

## Handlebars Template Syntax

Key patterns used:

```handlebars
{{#each plugins}}
  <!-- Loop over plugins array -->
  {{name}}
  <!-- Insert plugin.name -->
  {{className}}
  <!-- Insert plugin.className -->
  {{identifier}}
  <!-- Insert plugin.identifier -->
{{/each}}
<!-- End loop -->

{{app.lib-name}}
<!-- Existing app template variable -->
```

## Conditional Logic (Advanced)

For handling optional plugin features:

```handlebars
{{#if plugins}}
// Plugins registered
{{#each plugins}}
STATIC_PLUGINS.set('{{name}}', new {{className}}());
{{/each}}
{{else}}
// No plugins to register
const STATIC_PLUGINS: Map<string, Plugin> = new Map();
{{/if}}
```

## Testing Template Rendering

```rust
#[test]
fn test_template_rendering() {
    let handlebars = setup_handlebars();

    let plugins = vec![
        PluginMeta {
            name: "dialog",
            identifier: "@tauri/plugin-dialog",
            class_name: "DialogPlugin",
            har_path: PathBuf::new(),
        }
    ];

    let data = json!({
        "app": {"lib-name": "myapp"},
        "plugins": [{"name": "dialog", "identifier": "@tauri/plugin-dialog", "className": "DialogPlugin"}]
    });

    let template = include_str!("../../templates/mobile/open-harmony/entry/src/main/ets/entryability/EntryAbility.ets.hbs");
    handlebars.register_template_string("entry_ability", template).unwrap();

    let rendered = handlebars.render("entry_ability", &data).unwrap();

    assert!(rendered.contains("import DialogPlugin from '@tauri/plugin-dialog'"));
    assert!(rendered.contains("STATIC_PLUGINS.set('dialog', new DialogPlugin())"));
}
```

## Next Phase

Phase 6 will integrate all phases into the build flow.

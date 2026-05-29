# OpenHarmony Dynamic Plugin Generation - Overview

## Problem Statement

Current implementation hardcodes the `dialog` plugin in templates:

- `dialog` HAR directory is copied for all projects
- `build-profile.json5` includes dialog module
- `entry/oh-package.json5` includes dialog dependency
- `EntryAbility.ets` imports and registers DialogPlugin

This means every Tauri project on OpenHarmony will include the dialog plugin regardless of whether it's actually used.

## Solution: Dynamic Template Generation

Automatically detect and inject plugins based on `Cargo.toml` dependencies during build.

## Core Flow

```
cargo tauri ohos build
    ↓
Parse Cargo.toml, detect tauri-plugin-* dependencies
    ↓
Find plugin openharmony directories (plugins-workspace or local)
    ↓
Read plugin metadata (name, identifier, className)
    ↓
Copy plugin HAR to gen/ohos/
    ↓
Update build-profile.json5 (add module)
    ↓
Update entry/oh-package.json5 (add dependency)
    ↓
Generate EntryAbility.ets using template (dynamic imports and registration)
    ↓
Continue original build flow
```

## Implementation Phases

- **Phase 1**: Plugin Detection - Parse Cargo.toml and find plugin HAR directories
- **Phase 2**: Metadata Parsing - Extract plugin info from oh-package.json5
- **Phase 3**: HAR Copy - Copy plugin HAR with path adjustments
- **Phase 4**: Config Update - Update build-profile.json5 and oh-package.json5
- **Phase 5**: EntryAbility Templating - Use Handlebars for dynamic generation
- **Phase 6**: Build Integration - Integrate into existing build flow

## Work Estimation

| Phase                          | Estimated Time |
| ------------------------------ | -------------- |
| Phase 1-2: Detection & Parsing | 2-3 hours      |
| Phase 3-4: HAR Copy & Config   | 2-3 hours      |
| Phase 5: Templating            | 1-2 hours      |
| Phase 6: Integration           | 1-2 hours      |
| Testing & Debugging            | 2-3 hours      |
| **Total**                      | **8-12 hours** |

## Future Enhancements

- Plugin version management
- Custom plugin paths
- Plugin development scaffold (`tauri plugin init --ohos`)
- Pre-built HAR caching

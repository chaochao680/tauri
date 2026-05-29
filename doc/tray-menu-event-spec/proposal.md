## Why

OHOS tray menu item clicks were triggering the wrong events: `TrayIconEvent` was being sent for menu item clicks (Regular/Check items), which should only fire `MenuEvent`. This deviated from Windows/macOS behavior where tray icon interactions and menu item interactions are cleanly separated. Without a clear specification, each platform implementation drifted, causing inconsistent behavior and making it impossible for developers to write portable event handling code.

## What Changes

- **Define authoritative event model** for tray icon and menu item interactions across all platforms
- **Clarify the separation**: `TrayIconEvent` for icon interactions only, `MenuEvent` for menu item clicks only
- **Align OHOS implementation** with Windows/macOS reference behavior
- **Document event firing rules** for Regular, Check, and Predefined menu items in tray context
- **Remove incorrect `TrayIconEvent::send`** calls from OHOS tray menu item click handlers
- **Add numeric menuCode remapping** to bridge OHOS StatusBar's numeric indices back to original string IDs
- **Export `send_menu_event` public API** from `openharmony_ability` to enable tray→muda event bridging
- **Document OHOS StatusBar API constraints** (single tray per app, limited click data, no hover/double-click)

## Capabilities

### New Capabilities
- `tray-menu-event-model`: Defines the complete event model for tray icon and menu interactions, specifying exactly which events fire for each user action across all platforms (Windows, macOS, OHOS)

### Modified Capabilities

## Impact

- **Code**: `tray-icon/src/platform_impl/ohos/event.rs` (event handler + menuCode translation), `tray-icon/src/platform_impl/ohos/mod.rs` (numeric remapping), `openharmony-ability/crates/ability/src/menu/mod.rs` (`send_menu_event` export)
- **APIs**: `TrayIconEvent` and `MenuEvent` usage patterns documented; `openharmony_ability::send_menu_event` added as public Rust API
- **Dependencies**: `tray-icon` crate now depends on `openharmony_ability::send_menu_event` (already an existing dependency)
- **Systems**: All platforms (Windows, macOS, OHOS) - ensures consistency; OHOS-specific StatusBar constraints documented
- **Demo**: `examples/api/src-tauri/src/tray.rs` (TRAY_IDS filter + emit_to), `examples/api/src/views/Tray.svelte` (Full Test Tray + Remove All Trays)

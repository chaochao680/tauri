## 1. Remove Incorrect Event Firing

- [x] 1.1 Delete `convert_menu_click` function from `tray-icon/src/platform_impl/ohos/event.rs`
- [x] 1.2 Remove `TrayIconEvent::send(tray_event)` call from `MenuAction::Check` branch (line 77-78)
- [x] 1.3 Remove `TrayIconEvent::send(tray_event)` call from `MenuAction::Regular` branch (line 84-85)
- [x] 1.4 Remove unused imports: `PhysicalPosition`, `Rect`, `MouseButton`, `MouseButtonState` from event.rs (if no longer used elsewhere)

## 2. Clean Up Tests

- [x] 2.1 Delete `test_menu_click_conversion` test from event.rs (tests removed function)
- [x] 2.2 Verify `test_icon_click_conversion` test still passes (icon click behavior unchanged)

## 3. Verify Event Bridge

- [x] 3.1 Confirm `openharmony_ability::send_menu_event(code)` remains in `MenuAction::Check` branch
- [x] 3.2 Confirm `openharmony_ability::send_menu_event(menu_code)` remains in `MenuAction::Regular` branch
- [x] 3.3 Confirm `MenuAction::Predefined` branch does NOT call `send_menu_event` (no event fired)

## 4. Numeric menuCode Remapping

- [x] 4.1 Add `flat_ids: Vec<String>` field to `MenuMetadata` in `tray-icon/src/platform_impl/ohos/mod.rs`
- [x] 4.2 Implement `remap_menu_codes_to_indices()` — replaces string menuCode with numeric indices, returns flat ID list
- [x] 4.3 Call `remap_menu_codes_to_indices` in `TrayIcon::new` (initial creation)
- [x] 4.4 Call `remap_menu_codes_to_indices` in `TrayIcon::set_menu` (menu replacement)
- [x] 4.5 Call `remap_menu_codes_to_indices` in `rebuild_and_update_menu` (Check item toggle rebuild)
- [x] 4.6 Implement `translate_menu_code()` — numeric string → original ID lookup via `flat_ids`
- [x] 4.7 Call `translate_menu_code` on `raw_code` before action classification in event forward thread

## 5. Export `send_menu_event` Public API

- [x] 5.1 Add `pub fn send_menu_event(menu_id: String)` to `openharmony_ability::menu::mod.rs`
- [x] 5.2 Re-export from `openharmony_ability::lib.rs` crate root

## 6. Demo Application Improvements

- [x] 6.1 Add `TRAY_IDS` whitelist in `tray.rs` `on_menu_event` handler to filter non-tray menu IDs
- [x] 6.2 Use `app.emit_to(EventTarget::webview_window("main"))` instead of `app.emit` to target main window
- [x] 6.3 Add "Full Test Tray" button with all menu item types (Normal, Check, Icon, Predefined)
- [x] 6.4 Add "Remove All Trays" button that removes all known tray IDs unconditionally
- [x] 6.5 Use base64 PNG icon (`DEFAULT_ICON`) for OHOS compatibility instead of `NativeIcon.Folder`
- [x] 6.6 Add remove-before-create pattern for OHOS single-tray constraint

## 7. Build and Deploy

- [x] 7.1 Run `ohos-build` skill to rebuild HAR + HAP
- [x] 7.2 Deploy to OHOS desktop device

## 8. Verification Testing

- [ ] 8.1 Test tray icon left-click → Console shows `TrayIconEvent` output, NO `MenuEvent`
- [ ] 8.2 Test tray Regular menu item click → Console shows `tray:xxx` and `global:xxx` (each once), NO `TrayIconEvent`
- [ ] 8.3 Test tray Check menu item click → Console shows `tray:xxx` and `global:xxx`, item toggles, NO `TrayIconEvent`
- [ ] 8.4 Test tray Predefined menu item (quit/minimize) → Action executes, NO console output (no events)
- [ ] 8.5 Test menubar item click → Console shows `global:xxx` only (no `tray:` prefix), behavior unchanged
- [ ] 8.6 Test popup menu item click → Console shows `global:xxx` only, behavior unchanged

## 9. Documentation

- [x] 9.1 Add comment in event.rs explaining why `TrayIconEvent` is NOT fired for menu clicks (reference spec)
- [x] 9.2 Update any inline documentation that incorrectly states `TrayIconEvent` fires for menu items
- [x] 9.3 Update openspec design.md and spec.md with OHOS StatusBar limitations and numeric remapping

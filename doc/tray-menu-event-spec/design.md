## Context

Tauri's tray and menu system uses two distinct event types:
- `TrayIconEvent`: Fired when user interacts with the tray icon itself
- `MenuEvent`: Fired when user clicks a menu item

On Windows and macOS, these events are cleanly separated. On OHOS, the tray icon and tray menu both route through the StatusBar API, which uses a single callback channel (`StatusBarClickEvent::MenuClick`). The OHOS implementation was incorrectly treating menu item clicks as icon interactions, firing `TrayIconEvent` instead of (or in addition to) `MenuEvent`.

### Platform Implementation Details

**Windows:**
```
Tray menu item click
  → TrackPopupMenu → WM_COMMAND → tray hidden window
  → muda's menu_subclass_proc (attached via attach_menu_subclass_for_hwnd)
  → WM_COMMAND handler → menu_selected()
  → dispatch=true (Regular/Check) → MenuEvent::send()
  → dispatch=false (Predefined) → no event

Tray icon click
  → tray_proc → TrayIconEvent::send()
  → no muda involvement
```

**macOS:**
```
Tray menu item click
  → NSMenu item selected → target/action callback
  → muda's item_selected handler
  → Check: toggle + MenuEvent::send()
  → Predefined: execute action, no event
  → Regular: MenuEvent::send()

Tray icon click
  → mouse event callback → TrayIconEvent::send()
  → no muda involvement
```

**OHOS (corrected):**
```
Tray menu item click
  → StatusBar menuClick callback → statusbar::menu_click_receiver()
  → event.rs: raw_code = numeric string from StatusBar (e.g. "0", "1")
  → translate_menu_code(raw_code) → original string ID via flat_ids mapping
  → match on Predefined/Check/Regular
  → Regular/Check: openharmony_ability::send_menu_event(original_id)
    → MENU_EVENT_CHANNEL → muda's start_event_listener thread
    → MenuEvent::send() → tauri's on_menu_event
  → Predefined: execute_predefined_action(), no event

Tray icon click
  → StatusBar iconClick callback → statusbar::icon_click_receiver()
  → event.rs: TrayIconEvent::send()
  → tauri's on_tray_icon_event
```

### Shared `global_event_listeners` Behavior

`TrayIconBuilder::on_menu_event` and `App::on_menu_event` both register to the **same** `global_event_listeners` list (see `tray/mod.rs:472-479`). This means:

- **Tray menu item click** → `MenuEvent` → ALL global listeners fire (tray handler + app handler)
- **Menubar/popup item click** → `MenuEvent` → ALL global listeners fire (same handlers)

The tray's `on_menu_event` handler is NOT tray-specific — it fires for menubar/popup items too. Applications MUST filter by `event.id()` to distinguish sources.

**Stakeholders:**
- Developers using `TrayIconBuilder::on_menu_event` (expecting menu item click callbacks)
- Developers using `TrayIconBuilder::on_tray_icon_event` (expecting icon interaction callbacks)
- Platform maintainers (Windows, macOS, OHOS)

## Goals / Non-Goals

**Goals:**
- Align OHOS tray event behavior with Windows/macOS
- Ensure `TrayIconEvent` ONLY fires for icon interactions
- Ensure `MenuEvent` fires for menu item clicks (Regular/Check) from all sources
- Maintain backward compatibility for Predefined items (no event fired)
- Document the event model clearly for future maintainers

**Non-Goals:**
- Changing the event model for menubar or popup menus (already correct)
- Adding new event types or callbacks
- Modifying Windows/macOS implementations (already correct)
- Changing `TrayIconEvent` or `MenuEvent` struct definitions

## Decisions

### Decision 1: Remove `TrayIconEvent::send` from menu item click handlers

**Choice:** Delete `convert_menu_click` function and all calls to `TrayIconEvent::send` in `MenuAction::Regular` and `MenuAction::Check` branches.

**Rationale:** Windows/macOS never fire `TrayIconEvent` for menu item clicks. The `convert_menu_click` function exists only to construct a fake `TrayIconEvent` for menu clicks, which violates the spec. Removing it enforces the separation at compile time (dead code elimination).

**Alternatives considered:**
1. Keep `TrayIconEvent::send` but add a flag to distinguish icon vs menu clicks → Rejected: adds complexity, violates spec
2. Fire both events for backward compat → Rejected: breaks spec, confuses developers (double events)

### Decision 2: Bridge tray menu clicks via `openharmony_ability::send_menu_event`

**Choice:** Use the existing `openharmony_ability::menu::MENU_EVENT_CHANNEL` to forward tray menu clicks into muda's event listener thread.

**Rationale:** 
- Muda's `init_menu_event_listener()` already spawns a thread that reads from `menu_event_receiver()` and calls `muda::MenuEvent::send()`
- Tauri registers a global handler via `muda::MenuEvent::set_event_handler()` that forwards to the event loop
- Reusing this channel means tray menu clicks flow through the exact same path as menubar/popup clicks
- No new threads, no new channels, minimal code

**Alternatives considered:**
1. Create a separate tray-specific event channel → Rejected: duplicates infrastructure, breaks "all menu events go through one listener" invariant
2. Call `muda::MenuEvent::send()` directly from event.rs → Rejected: would bypass muda's `CHECK_ITEMS` toggle logic and `MENU_EVENT_HANDLER` customization

### Decision 3: Keep Predefined items silent (no event fired)

**Choice:** `MenuAction::Predefined` branch executes the action but does NOT fire `MenuEvent` or `TrayIconEvent`.

**Rationale:** Windows `menu_selected()` (line 1189-1243) sets `dispatch = false` for Predefined items, preventing `MenuEvent::send`. This is intentional: Predefined items (quit, minimize, copy, etc.) are system actions, not application-level menu selections. Firing events would force developers to filter them out.

**Alternatives considered:**
1. Fire `MenuEvent` for all items including Predefined → Rejected: breaks Win/Mac parity, forces filtering
2. Fire a new `PredefinedActionEvent` → Rejected: over-engineering, no use case

### Decision 4: Event listener registration stays unchanged

**Choice:** `TrayIconBuilder::on_menu_event` continues to register in `global_event_listeners` (fires for all menu events, not just tray-specific).

**Rationale:** This is tauri's documented design ("this handler is called for any menu event"). Changing it would be a breaking change. Developers filter by checking if `event.id()` matches their tray menu item IDs. The api demo's `TRAY_IDS` filter demonstrates this pattern.

**Alternatives considered:**
1. Auto-filter to only tray-specific items → Rejected: breaking change, reduces flexibility
2. Add `on_tray_menu_event` (tray-only) separate from `on_menu_event` (global) → Rejected: API bloat, inconsistent with Win/Mac

### Decision 5: Numeric menuCode remapping via `flat_ids`

**Choice:** Before sending menu items to StatusBar, remap all `menu_code` values to sequential numeric strings ("0", "1", "2"...) and store the original IDs in a `flat_ids: Vec<String>`. When StatusBar returns a numeric code, translate it back via index lookup.

**Rationale:** OHOS StatusBar API replaces arbitrary string `menuCode` values with numeric indices. Without remapping, the application would receive "0" as the menu ID instead of "toggle" or "quit", making it impossible to match against developer-defined IDs. The `remap_menu_codes_to_indices` function runs whenever menu items are sent to StatusBar (initial creation + rebuilds for Check toggles), and `translate_menu_code` runs on every menu click.

**Implementation:**
- `remap_menu_codes_to_indices(groups)` in `tray-icon/src/platform_impl/ohos/mod.rs` — iterates all menu items (including sub_menu), replaces `menu_code` with index string, returns flat list of original IDs
- `translate_menu_code(raw_code)` in `event.rs` — parses numeric string, looks up in `flat_ids`, returns original ID or raw code if out of range
- `flat_ids` stored in `MENU_METADATA` alongside `predefined_map` and `check_state`

**Alternatives considered:**
1. Use numeric IDs everywhere → Rejected: developer-facing IDs must be strings for cross-platform compatibility
2. Maintain mapping in StatusBar NAPI layer → Rejected: StatusBar doesn't expose the original IDs back; mapping must live in Rust
3. Encode original ID into menuCode with separator → Rejected: StatusBar may strip or mangle non-numeric characters

### Decision 6: Export `send_menu_event` as public API from `openharmony_ability`

**Choice:** Add `pub fn send_menu_event(menu_id: String)` to `openharmony_ability::menu` module, re-exported from crate root. The `tray-icon` crate calls this to bridge StatusBar menu clicks into the shared `MENU_EVENT_CHANNEL`.

**Rationale:** The `MENU_EVENT_CHANNEL` is internal to `openharmony_ability` and not directly accessible from `tray-icon`. Exposing a thin wrapper function maintains encapsulation while enabling the tray→muda event bridge. Both `emit_menu_event` (NAPI, for menubar/popup) and `send_menu_event` (Rust API, for tray) write to the same channel, ensuring all menu events converge at muda's event listener.

**Alternatives considered:**
1. Make `MENU_EVENT_CHANNEL` public → Rejected: exposes internal infrastructure
2. Have tray-icon depend on muda directly → Rejected: would bypass OHOS-specific channel and CHECK_ITEMS toggle logic
3. Create a new shared crate for the channel → Rejected: over-engineering for a single function

## Risks / Trade-offs

**[Risk] Breaking change for OHOS apps relying on `TrayIconEvent` for menu clicks**
- **Likelihood:** Low (OHOS support is new, few apps in production)
- **Impact:** Medium (apps would need to migrate to `on_menu_event`)
- **Mitigation:** This is a bug fix aligning with documented behavior. Release notes should highlight the change.

**[Risk] `convert_menu_click` removal breaks internal code**
- **Likelihood:** Low (only used in the two branches we're removing)
- **Impact:** Low (dead code elimination)
- **Mitigation:** Search for all callers before deletion. Update tests that reference it.

**[Trade-off] Global `on_menu_event` requires manual filtering**
- Developers must check `event.id()` to distinguish tray vs menubar vs popup
- This is the existing tauri design, not introduced by this change
- The api demo's `TRAY_IDS` constant demonstrates the pattern

**[Trade-off] Predefined items don't fire events**
- Developers can't track when predefined actions execute
- This matches Windows/macOS behavior
- If tracking is needed, developers can use Regular items and implement actions manually

**[Risk] Numeric menuCode remapping gets out of sync**
- **Likelihood:** Low (remapping runs on every menu creation and rebuild)
- **Impact:** Medium (menu clicks would fire `MenuEvent` with wrong IDs)
- **Mitigation:** `remap_menu_codes_to_indices` is called in all three paths: initial `TrayIcon::new`, `set_menu`, and `rebuild_and_update_menu` (Check toggle). `translate_menu_code` has a fallback to pass through the raw code if index is out of range.

**[Risk] OHOS single-tray limitation confuses developers**
- **Likelihood:** Medium (developers coming from Windows/macOS expect multiple trays)
- **Impact:** Low (second tray creation silently fails or replaces first)
- **Mitigation:** Api demo demonstrates remove-before-create pattern. `TrayIcon.removeById()` is called before `TrayIcon.new()` in both "Create tray" and "Full Test Tray" buttons.

## Open Questions

None. The spec is clear, the implementation path is straightforward (delete code + keep one function call), and Windows/macOS provide the reference behavior.

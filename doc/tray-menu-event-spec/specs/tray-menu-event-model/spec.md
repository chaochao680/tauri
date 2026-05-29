## ADDED Requirements

### Requirement: TrayIconEvent fires only for icon interactions
The system SHALL fire `TrayIconEvent` ONLY when the user interacts with the tray icon itself (click, double-click, enter, leave, move). Menu item interactions within the tray popup menu SHALL NOT fire `TrayIconEvent`.

#### Scenario: Left-click on tray icon
- **WHEN** user left-clicks the tray icon
- **THEN** system fires `TrayIconEvent::Click { button: Left, state: Up }`
- **AND** system does NOT fire `MenuEvent`

#### Scenario: Right-click on tray icon (Windows/macOS only)
- **WHEN** user right-clicks the tray icon
- **THEN** system fires `TrayIconEvent::Click { button: Right, state: Up }`
- **AND** system does NOT fire `MenuEvent`
- **AND** system shows tray popup menu (if configured)
- **NOTE**: OHOS StatusBar API does not distinguish left vs right click; all icon clicks are reported as `button: Left`

#### Scenario: Double-click on tray icon (Windows/macOS only)
- **WHEN** user double-clicks the tray icon
- **THEN** system fires `TrayIconEvent::DoubleClick { button: Left }`
- **AND** system does NOT fire `MenuEvent`
- **NOTE**: OHOS StatusBar API does not detect double-clicks; only single clicks are reported

#### Scenario: Mouse enters tray icon (Windows/macOS only)
- **WHEN** mouse cursor moves over the tray icon
- **THEN** system fires `TrayIconEvent::Enter`
- **AND** system does NOT fire `MenuEvent`
- **NOTE**: OHOS StatusBar API does not track hover/enter/leave events

#### Scenario: Mouse leaves tray icon (Windows/macOS only)
- **WHEN** mouse cursor moves away from the tray icon
- **THEN** system fires `TrayIconEvent::Leave`
- **AND** system does NOT fire `MenuEvent`
- **NOTE**: OHOS StatusBar API does not track hover/enter/leave events

### Requirement: MenuEvent fires for tray menu item clicks
The system SHALL fire `MenuEvent` when the user clicks a Regular or Check menu item in the tray popup menu. The system SHALL NOT fire `MenuEvent` for Predefined menu items (quit, minimize, maximize, close, fullscreen, copy, cut, paste, selectAll, undo, redo, recover).

#### Scenario: Click Regular menu item in tray
- **WHEN** user clicks a Regular menu item (non-Predefined, non-Check) in tray popup menu
- **THEN** system fires `MenuEvent { id: <item_id> }`
- **AND** system does NOT fire `TrayIconEvent`
- **AND** system executes the menu item's action

#### Scenario: Click Check menu item in tray
- **WHEN** user clicks a Check menu item in tray popup menu
- **THEN** system toggles the item's checked state
- **AND** system fires `MenuEvent { id: <item_id> }`
- **AND** system does NOT fire `TrayIconEvent`

#### Scenario: Click Predefined menu item in tray
- **WHEN** user clicks a Predefined menu item (quit, minimize, maximize, close, fullscreen, copy, cut, paste, selectAll, undo, redo, recover) in tray popup menu
- **THEN** system executes the predefined action
- **AND** system does NOT fire `MenuEvent`
- **AND** system does NOT fire `TrayIconEvent`

### Requirement: MenuEvent fires for menubar and popup menu item clicks
The system SHALL fire `MenuEvent` when the user clicks a Regular or Check menu item in a menubar or popup context menu. The behavior SHALL be identical to tray menu item clicks.

#### Scenario: Click Regular menu item in menubar
- **WHEN** user clicks a Regular menu item in the menubar
- **THEN** system fires `MenuEvent { id: <item_id> }`
- **AND** system executes the menu item's action

#### Scenario: Click Check menu item in menubar
- **WHEN** user clicks a Check menu item in the menubar
- **THEN** system toggles the item's checked state
- **AND** system fires `MenuEvent { id: <item_id> }`

#### Scenario: Click Predefined menu item in menubar
- **WHEN** user clicks a Predefined menu item in the menubar
- **THEN** system executes the predefined action
- **AND** system does NOT fire `MenuEvent`
- **AND** system does NOT fire `TrayIconEvent`

#### Scenario: Click Predefined menu item in popup context menu
- **WHEN** user clicks a Predefined menu item in a popup context menu
- **THEN** system executes the predefined action
- **AND** system does NOT fire `MenuEvent`
- **AND** system does NOT fire `TrayIconEvent`

#### Scenario: Click Check menu item in popup context menu
- **WHEN** user clicks a Check menu item in a popup context menu
- **THEN** system toggles the item's checked state
- **AND** system fires `MenuEvent { id: <item_id> }`

### Requirement: Event separation across platforms
The system SHALL maintain identical event firing behavior on Windows, macOS, and OHOS. Platform-specific implementation details MUST NOT affect the observable event model.

#### Scenario: Cross-platform tray Regular item click
- **WHEN** user clicks a Regular menu item in tray popup menu on Windows
- **THEN** system fires `MenuEvent`
- **WHEN** user clicks a Regular menu item in tray popup menu on macOS
- **THEN** system fires `MenuEvent`
- **WHEN** user clicks a Regular menu item in tray popup menu on OHOS
- **THEN** system fires `MenuEvent`

#### Scenario: Cross-platform tray icon click
- **WHEN** user left-clicks the tray icon on Windows
- **THEN** system fires `TrayIconEvent::Click`
- **WHEN** user left-clicks the tray icon on macOS
- **THEN** system fires `TrayIconEvent::Click`
- **WHEN** user left-clicks the tray icon on OHOS
- **THEN** system fires `TrayIconEvent::Click`

### Requirement: Event listener registration
The system SHALL support registering event listeners via `TrayIconBuilder::on_tray_icon_event` (receives `TrayIconEvent`) and `TrayIconBuilder::on_menu_event` (receives `MenuEvent`). The system SHALL support global menu event listeners via `App::on_menu_event` that receive `MenuEvent` from all menu sources (menubar, popup, tray).

#### Scenario: Tray icon event listener receives only TrayIconEvent
- **WHEN** user registers `on_tray_icon_event` listener
- **AND** user clicks tray icon
- **THEN** listener receives `TrayIconEvent::Click`
- **AND** user clicks tray menu item
- **THEN** listener does NOT receive any event

#### Scenario: Tray menu event listener receives MenuEvent
- **WHEN** user registers `on_menu_event` listener on `TrayIconBuilder`
- **AND** user clicks tray menu Regular item
- **THEN** listener receives `MenuEvent { id: <item_id> }`
- **AND** user clicks menubar Regular item
- **THEN** listener receives `MenuEvent { id: <item_id> }`

#### Scenario: Global menu event listener receives all MenuEvents
- **WHEN** user registers global `on_menu_event` listener on `App`
- **AND** user clicks any Regular/Check menu item (menubar, popup, or tray)
- **THEN** listener receives `MenuEvent { id: <item_id> }`
- **AND** listener can identify source by checking if `id` matches known tray menu items

#### Scenario: Tray menu item triggers both tray handler and global handler
- **WHEN** user registers `on_menu_event` on both `TrayIconBuilder` and `App`
- **AND** user clicks a Regular/Check tray menu item
- **THEN** BOTH handlers fire (tray handler first, then global handler)
- **AND** both receive the same `MenuEvent { id: <item_id> }`
- **NOTE**: `TrayIconBuilder::on_menu_event` registers to the same `global_event_listeners` as `App::on_menu_event`, so both fire for all menu events

#### Scenario: Menubar/popup item also triggers both handlers
- **WHEN** user registers `on_menu_event` on both `TrayIconBuilder` and `App`
- **AND** user clicks a menubar or popup menu item
- **THEN** BOTH handlers fire (same behavior as tray menu item)
- **AND** application MUST filter by checking `event.id()` against known menu item IDs to distinguish sources

### Requirement: OHOS StatusBar menuCode remapping
OHOS StatusBar API returns numeric indices (e.g. "0", "1", "2") as `menuCode` values in menu click callbacks, regardless of the original string IDs set by the application. The system SHALL maintain a bidirectional mapping to translate these numeric codes back to the original string IDs before firing `MenuEvent`.

#### Scenario: Tray menu item click with numeric menuCode
- **WHEN** application creates a tray menu with items having string IDs (e.g. "toggle", "quit", "new-window")
- **AND** system remaps menuCode values to sequential numeric indices ("0", "1", "2") before sending to StatusBar
- **AND** user clicks a menu item
- **THEN** StatusBar returns the numeric menuCode (e.g. "1")
- **AND** system translates "1" back to original string ID (e.g. "quit") via `flat_ids` mapping
- **AND** system fires `MenuEvent { id: "quit" }` (not `MenuEvent { id: "1" }`)

#### Scenario: Menu rebuild preserves remapping
- **WHEN** application updates tray menu (e.g. Check item toggle triggers menu rebuild)
- **AND** system rebuilds menu groups and reassigns numeric indices
- **THEN** system updates `flat_ids` mapping to reflect new index→ID associations
- **AND** subsequent menu clicks are translated correctly

#### Scenario: Numeric code out of range
- **WHEN** StatusBar returns a numeric menuCode that exceeds `flat_ids` length
- **THEN** system passes the raw numeric string through as-is (no translation)
- **AND** logs a debug warning

### Requirement: OHOS StatusBar API constraints
The OHOS platform implementation operates within StatusBar API constraints that differ from Windows/macOS native tray APIs.

#### Constraint: Single tray icon per application
- **FACT**: OHOS `statusBarManager.addToStatusBar` only accepts one context per application
- **IMPLICATION**: Only one `TrayIcon` can be active at a time; creating a new tray requires removing the existing one first
- **MITIGATION**: Application code should call `TrayIcon.removeById()` before creating a new tray

#### Constraint: Limited icon click data
- **FACT**: StatusBar `iconClick` callback provides only `click_type` (always "leftClick" or "rightClick") with no position, rect, or button state
- **IMPLICATION**: `TrayIconEvent::Click` on OHOS always has `position: (0,0)`, `rect: default`, `button: Left`, `button_state: Up`
- **MITIGATION**: Applications should not rely on click position or button type for OHOS tray icons

#### Constraint: No hover or multi-click detection
- **FACT**: StatusBar API has no hover tracking and no double-click detection
- **IMPLICATION**: `TrayIconEvent::Enter`, `TrayIconEvent::Leave`, `TrayIconEvent::DoubleClick` are never fired on OHOS
- **MITIGATION**: Applications should provide alternative interaction patterns for OHOS

#### Constraint: Menu item click routing
- **FACT**: StatusBar menu clicks route through `openharmony_ability::statusbar::menu_click_receiver()` channel, separate from menubar/popup menu events
- **IMPLICATION**: Tray menu clicks must be bridged to the shared `MENU_EVENT_CHANNEL` via `openharmony_ability::send_menu_event()` to flow through muda's event listener
- **MITIGATION**: `tray-icon` crate's event forward thread bridges the two channels, ensuring tray menu clicks appear identical to menubar/popup clicks from tauri's perspective

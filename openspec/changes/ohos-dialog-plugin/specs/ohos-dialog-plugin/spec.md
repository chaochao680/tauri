## ADDED Requirements

### Requirement: OHOS plugin registration
The system SHALL provide a mechanism to register Tauri plugins for the OHOS platform. Plugin registration MUST occur during application startup and MUST include the plugin's name, identifier, class name, and configuration.

#### Scenario: Register a plugin via `PluginApi::register_ohos_plugin`
- **WHEN** a plugin calls `api.register_ohos_plugin("dialog", "DialogPlugin")` during initialization
- **THEN** the system adds a `PluginRegistration { name: "dialog", identifier: "dialog", class_name: "DialogPlugin", config: {...} }` to `PLUGINS_TO_REGISTER`
- **AND** the system returns `Ok(PluginHandle)` to the caller

#### Scenario: Register multiple plugins
- **WHEN** multiple plugins call `register_ohos_plugin` during startup
- **THEN** each plugin's registration is appended to `PLUGINS_TO_REGISTER`
- **AND** all registrations are available when the OHOS runtime initializes

#### Scenario: Plugin registration is thread-safe
- **WHEN** multiple threads call `register_ohos_plugin` concurrently
- **THEN** the system serializes access to `PLUGINS_TO_REGISTER` via `Mutex`
- **AND** no data races occur

### Requirement: OHOS plugin command dispatch
The system SHALL dispatch plugin commands from Rust to ArkTS via NAPI. Each command invocation MUST be assigned a unique callback ID, and the response handler MUST be stored for later invocation.

#### Scenario: Dispatch a plugin command
- **WHEN** Rust code calls `run_command("dialog", handle, "open", payload, handler)`
- **THEN** the system assigns a unique `id` (via `PENDING_PLUGIN_CALLS_ID.fetch_add(1)`)
- **AND** the system stores `handler` in `PENDING_PLUGIN_CALLS` map with key `id`
- **AND** the system constructs `RunCommandArgs { id, plugin_name: "dialog", command: "open", payload: "..." }`
- **AND** the system calls `dispatch_run_command(args)` to invoke ArkTS via NAPI

#### Scenario: Command callback ID uniqueness
- **WHEN** two commands are dispatched sequentially
- **THEN** the first command receives `id = 0`
- **AND** the second command receives `id = 1`
- **AND** each ID is unique across the application lifetime

#### Scenario: Command payload serialization
- **WHEN** a command is dispatched with a JSON payload
- **THEN** the system serializes the payload to a JSON string
- **AND** the system includes the JSON string in `RunCommandArgs.payload`
- **AND** if serialization fails, the system returns `Err(PluginInvokeError::CannotSerializePayload)`

### Requirement: OHOS plugin response handling
The system SHALL invoke the stored response handler when ArkTS returns a plugin command result. The handler MUST be removed from `PENDING_PLUGIN_CALLS` after invocation.

#### Scenario: Receive plugin response from ArkTS
- **WHEN** ArkTS calls the NAPI callback with `{ id: 0, result: { path: "/data/..." } }`
- **THEN** the system looks up `id = 0` in `PENDING_PLUGIN_CALLS`
- **AND** the system removes the handler from the map
- **AND** the system invokes the handler with `PluginResponse::Ok(result)`

#### Scenario: Receive plugin error from ArkTS
- **WHEN** ArkTS calls the NAPI callback with `{ id: 1, error: "File not found" }`
- **THEN** the system looks up `id = 1` in `PENDING_PLUGIN_CALLS`
- **AND** the system removes the handler from the map
- **AND** the system invokes the handler with `PluginResponse::Err("File not found")`

#### Scenario: Response for unknown callback ID
- **WHEN** ArkTS returns a response with an `id` not in `PENDING_PLUGIN_CALLS`
- **THEN** the system logs a warning
- **AND** the system does not panic or crash

### Requirement: CLI plugin auto-detection
The system SHALL automatically detect Tauri plugins by scanning `Cargo.toml` dependencies. Plugins MUST follow the `tauri-plugin-*` naming convention.

#### Scenario: Detect plugin from `Cargo.toml`
- **WHEN** `Cargo.toml` contains `tauri-plugin-dialog = "2.0"`
- **THEN** the system adds `"dialog"` to the detected plugins list
- **AND** the system logs "Detected 1 plugins: [dialog]"

#### Scenario: Detect multiple plugins
- **WHEN** `Cargo.toml` contains `tauri-plugin-dialog`, `tauri-plugin-fs`, `tauri-plugin-http`
- **THEN** the system detects all three plugins
- **AND** the system returns `["dialog", "fs", "http"]` (sorted, deduplicated)

#### Scenario: Ignore non-plugin dependencies
- **WHEN** `Cargo.toml` contains `serde = "1.0"`, `tokio = "1.0"`
- **THEN** the system does not add these to the detected plugins list
- **AND** only `tauri-plugin-*` dependencies are detected

#### Scenario: Detect plugins from target-specific dependencies
- **WHEN** `Cargo.toml` contains `[target.'cfg(target_env = "ohos")'.dependencies] tauri-plugin-dialog = "2.0"`
- **THEN** the system detects `tauri-plugin-dialog` in target-specific dependencies
- **AND** the plugin is included in the detected list

### Requirement: CLI HAR file discovery
The system SHALL locate plugin HAR files in the project directory structure. HAR files MUST be in `plugins/<name>/openharmony/` or `plugins-workspace/plugins/<name>/openharmony/`.

#### Scenario: Find HAR in `plugins/dialog/openharmony/`
- **WHEN** the project directory contains `plugins/dialog/openharmony/dialog.har`
- **THEN** the system returns `PathBuf` pointing to `plugins/dialog/openharmony/dialog.har`
- **AND** the system logs "Found HAR for plugin dialog"

#### Scenario: Find HAR in `plugins-workspace/plugins/dialog/openharmony/`
- **WHEN** the project directory does not contain `plugins/dialog/openharmony/`
- **AND** the parent directory contains `plugins-workspace/plugins/dialog/openharmony/dialog.har`
- **THEN** the system returns `PathBuf` pointing to the workspace HAR file

#### Scenario: HAR file not found
- **WHEN** no HAR file exists for a detected plugin
- **THEN** the system returns `Err("HAR not found for plugin dialog")`
- **AND** the system logs an error with expected search paths

### Requirement: OHOS WebView runtime detection
The system SHALL correctly detect WebView runtime availability on OHOS. OHOS uses ArkUI's `Web` component, which is always available.

#### Scenario: WebView runtime check on OHOS
- **WHEN** the application starts on OHOS (`target_env = "ohos"`)
- **THEN** the system sets `webview_runtime_installed = true`
- **AND** the system does not call `wry::webview_version()` (which would fail on OHOS)

#### Scenario: WebView runtime check on Windows
- **WHEN** the application starts on Windows
- **THEN** the system calls `wry::webview_version()`
- **AND** if WebView2 is installed, `webview_runtime_installed = true`
- **AND** if WebView2 is not installed, `webview_runtime_installed = false`

#### Scenario: WebView runtime check on macOS
- **WHEN** the application starts on macOS
- **THEN** the system calls `wry::webview_version()`
- **AND** WebKit is always available, so `webview_runtime_installed = true`

### Requirement: Dialog plugin `open` command
The system SHALL provide a file picker dialog via the `open` command. The dialog MUST use OHOS's `@ohos.file.picker.FilePicker` API.

#### Scenario: Open file picker with default options
- **WHEN** Rust calls `dialog.open(None)`
- **THEN** the system displays a file picker with default filters (all files)
- **AND** the user selects a file
- **AND** the system returns `{ path: "/data/storage/...", name: "document.pdf" }`

#### Scenario: Open file picker with filters
- **WHEN** Rust calls `dialog.open({ filters: [{ name: "Images", extensions: ["jpg", "png"] }] })`
- **THEN** the system displays a file picker with "Images" filter selected
- **AND** the user selects an image file
- **AND** the system returns the selected file path

#### Scenario: Open file picker cancelled
- **WHEN** the user cancels the file picker
- **THEN** the system returns `null` (no file selected)

### Requirement: Dialog plugin `save` command
The system SHALL provide a save file dialog via the `save` command. The dialog MUST use OHOS's `@ohos.file.picker.FileSavePicker` API.

#### Scenario: Save file with default name
- **WHEN** Rust calls `dialog.save({ defaultName: "output.txt" })`
- **THEN** the system displays a save picker with "output.txt" pre-filled
- **AND** the user confirms the save location
- **AND** the system returns `{ path: "/data/storage/...", name: "output.txt" }`

#### Scenario: Save file cancelled
- **WHEN** the user cancels the save picker
- **THEN** the system returns `null` (no save location selected)

### Requirement: Dialog plugin `message` command
The system SHALL display a message dialog via the `message` command. The dialog MUST use OHOS's `@ohos.promptAction.showDialog` API.

#### Scenario: Display message with OK button
- **WHEN** Rust calls `dialog.message({ title: "Info", message: "Operation completed" })`
- **THEN** the system displays a dialog with title "Info" and message "Operation completed"
- **AND** the dialog has an "OK" button
- **AND** the user clicks "OK"
- **AND** the system returns `undefined` (no result needed)

### Requirement: Dialog plugin `ask` command
The system SHALL display a confirmation dialog via the `ask` command. The dialog MUST use OHOS's `@ohos.promptAction.showDialog` API with Yes/No buttons.

#### Scenario: Ask user Yes/No question
- **WHEN** Rust calls `dialog.ask({ title: "Confirm", message: "Delete file?" })`
- **THEN** the system displays a dialog with "Yes" and "No" buttons
- **AND** the user clicks "Yes"
- **AND** the system returns `true`

#### Scenario: Ask user declines
- **WHEN** the user clicks "No"
- **THEN** the system returns `false`

### Requirement: Dialog plugin `confirm` command
The system SHALL display a confirmation dialog via the `confirm` command. The dialog MUST use OHOS's `@ohos.promptAction.showDialog` API with OK/Cancel buttons.

#### Scenario: Confirm action
- **WHEN** Rust calls `dialog.confirm({ title: "Warning", message: "Save changes?" })`
- **THEN** the system displays a dialog with "OK" and "Cancel" buttons
- **AND** the user clicks "OK"
- **AND** the system returns `true`

#### Scenario: Confirm cancelled
- **WHEN** the user clicks "Cancel"
- **THEN** the system returns `false`

## MODIFIED Requirements

### Requirement: Plugin API availability
The system SHALL provide `PluginApi::register_ohos_plugin` method for OHOS plugin registration. This method is only available on OHOS (`#[cfg(target_env = "ohos")]`).

#### Scenario: Call `register_ohos_plugin` on OHOS
- **WHEN** code calls `api.register_ohos_plugin("dialog", "DialogPlugin")` on OHOS
- **THEN** the system compiles successfully
- **AND** the plugin is registered

#### Scenario: Call `register_ohos_plugin` on non-OHOS
- **WHEN** code attempts to call `api.register_ohos_plugin` on Windows/macOS
- **THEN** the system fails to compile (method not available)
- **AND** the compiler error indicates the method is OHOS-only

### Requirement: Plugin command dispatch availability
The system SHALL provide `run_command` implementation for OHOS. Previously, this was a TODO stub that did nothing.

#### Scenario: Dispatch command on OHOS (before this change)
- **WHEN** code called `run_command("dialog", handle, "open", payload, handler)` on OHOS
- **THEN** the system did nothing (TODO stub)
- **AND** the handler was never invoked

#### Scenario: Dispatch command on OHOS (after this change)
- **WHEN** code calls `run_command("dialog", handle, "open", payload, handler)` on OHOS
- **THEN** the system assigns a callback ID
- **AND** the system stores the handler
- **AND** the system dispatches the command to ArkTS via NAPI

### Requirement: `PENDING_PLUGIN_CALLS` visibility
The system SHALL make `PENDING_PLUGIN_CALLS` accessible from `crate::ohos` module. Previously, this was a private static.

#### Scenario: Access `PENDING_PLUGIN_CALLS` from `ohos.rs`
- **WHEN** `dispatch_run_command` in `ohos.rs` needs to store a handler
- **THEN** the system allows access to `PENDING_PLUGIN_CALLS` (now `pub(crate)`)
- **AND** the handler is inserted into the map

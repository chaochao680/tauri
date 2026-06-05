## Requirements

### OHOS Device Type Detection

#### Requirement: Environment variable as single source of truth
The OHOS device type SHALL be determined solely by the `OHOS_DEVICE_TYPE` environment variable.

- `OHOS_DEVICE_TYPE=desktop` → `cfg(desktop)` enabled, `cfg(mobile)` disabled
- `OHOS_DEVICE_TYPE=mobile` (or unset) → `cfg(mobile)` enabled, `cfg(desktop)` disabled

#### Requirement: Default to mobile
When `OHOS_DEVICE_TYPE` is not set, the system SHALL default to `"mobile"`.

#### Requirement: Cargo cache invalidation
All build scripts that read `OHOS_DEVICE_TYPE` SHALL emit `cargo:rerun-if-env-changed=OHOS_DEVICE_TYPE` to ensure cargo re-runs the build script when the value changes.

#### Requirement: Non-OHOS platforms unaffected
The `OHOS_DEVICE_TYPE` check SHALL only activate when `target_env == "ohos"`. Other platforms (iOS, Android, Windows, macOS, Linux) SHALL continue using `target_os`-based detection.

### CLI Integration

#### Requirement: CLI flag sets environment variable
The `--device-type` CLI flag in `cargo tauri ohos build` and `cargo tauri ohos dev` SHALL set `OHOS_DEVICE_TYPE` via `std::env::set_var` before any cargo subprocess starts.

#### Requirement: CLI flag accepts mobile or desktop
The `--device-type` flag SHALL accept exactly `"mobile"` or `"desktop"`, defaulting to `"mobile"`.

#### Requirement: Device type filtered from cargo args
The CLI SHALL strip `--device-type` and `--device-type=<value>` from arguments passed to cargo, preventing them from reaching the Rust compiler.

### Plugin Compatibility

#### Requirement: setup() reads OHOS_DEVICE_TYPE
`tauri_plugin::setup()` SHALL read `OHOS_DEVICE_TYPE` and emit the correct `cfg(desktop)` or `cfg(mobile)` aliases, ensuring all plugins using `tauri_plugin::Builder` get consistent device type detection.

#### Requirement: OHOS-specific cfg gates in plugins
Plugins that use desktop-only native libraries (e.g., `rfd`) SHALL guard those code paths with `cfg(all(desktop, not(target_env = "ohos")))` to exclude OHOS even when `cfg(desktop)` is active.

#### Requirement: OHOS mobile pathway inclusion
Plugins that use the mobile plugin invocation pathway SHALL include OHOS via `cfg(any(mobile, target_env = "ohos"))` to ensure OHOS desktop builds still use the correct plugin error variants.

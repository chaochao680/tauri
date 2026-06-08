# plugin-os-ohos Specification

## Purpose
TBD - created by archiving change p1-plugin-os-ohos. Update Purpose after archive.
## Requirements
### Requirement: OHOS platform detection
The system SHALL return `"ohos"` as the platform identifier when running on OHOS (OpenHarmony OS). The detection MUST use `cfg(target_env = "ohos")` at compile time.

#### Scenario: `platform()` returns `"ohos"` on OHOS desktop
- **WHEN** the application is compiled with `target_env = "ohos"` and `TAURI_OHOS_DEVICE_TYPE=desktop`
- **THEN** `platform()` SHALL return the string `"ohos"`
- **AND** `platform()` SHALL NOT return `"linux"`

#### Scenario: `platform()` returns `"ohos"` on OHOS mobile
- **WHEN** the application is compiled with `target_env = "ohos"` and `TAURI_OHOS_DEVICE_TYPE=mobile`
- **THEN** `platform()` SHALL return the string `"ohos"`
- **AND** `platform()` SHALL NOT return `"linux"`

#### Scenario: `platform()` unchanged on non-OHOS platforms
- **WHEN** the application is compiled without `target_env = "ohos"` (e.g., Windows, macOS, Linux)
- **THEN** `platform()` SHALL continue to return `std::env::consts::OS`
- **AND** the return value SHALL be identical to the behavior before this change

### Requirement: OHOS OS type detection
The system SHALL return `OsType::Ohos` (displayed as `"ohos"`) when running on OHOS. The `OsType` enum MUST include an `Ohos` variant.

#### Scenario: `type_()` returns `OsType::Ohos` on OHOS
- **WHEN** the application is compiled with `target_env = "ohos"`
- **THEN** `type_()` SHALL return `OsType::Ohos`
- **AND** the `Display` implementation SHALL format it as `"ohos"`

#### Scenario: `OsType::Ohos` is checked before `OsType::Linux`
- **WHEN** `type_()` is evaluated on OHOS
- **THEN** the `cfg(target_env = "ohos")` branch SHALL be evaluated BEFORE the `cfg(target_os = "linux")` branch
- **AND** OHOS SHALL NOT match the Linux branch

#### Scenario: `type_()` unchanged on non-OHOS platforms
- **WHEN** the application is compiled without `target_env = "ohos"`
- **THEN** `type_()` SHALL continue to return the same `OsType` variant as before this change

### Requirement: OHOS version placeholder
The system SHALL return a fallback `Version` value on OHOS instead of using the `os_info` crate, which does not support OHOS.

#### Scenario: `version()` returns fallback on OHOS
- **WHEN** the application is compiled with `target_env = "ohos"`
- **THEN** `version()` SHALL return `Version::Semantic(0, 0, 0)`
- **AND** the `to_string()` representation SHALL be `"0.0.0"`

#### Scenario: `version()` unchanged on non-OHOS platforms
- **WHEN** the application is compiled without `target_env = "ohos"`
- **THEN** `version()` SHALL continue to use `os_info::get().version().clone()`

### Requirement: TypeScript type definitions include OHOS
The TypeScript `Platform` and `OsType` types SHALL include `'ohos'` as a valid value.

#### Scenario: `Platform` type includes `'ohos'`
- **WHEN** TypeScript code imports `Platform` from `@tauri-apps/plugin-os`
- **THEN** the `Platform` union type SHALL include `'ohos'` as a valid literal type
- **AND** TypeScript code SHALL compile without errors when comparing `platform() === 'ohos'`

#### Scenario: `OsType` type includes `'ohos'`
- **WHEN** TypeScript code imports `OsType` from `@tauri-apps/plugin-os`
- **THEN** the `OsType` union type SHALL include `'ohos'` as a valid literal type
- **AND** TypeScript code SHALL compile without errors when comparing `type() === 'ohos'`

### Requirement: cfg isolation
All OHOS-specific code SHALL be gated with `cfg(target_env = "ohos")`. Non-OHOS platforms SHALL have zero code changes.

#### Scenario: Non-OHOS compilation is unaffected
- **WHEN** the application is compiled for Windows (`target_os = "windows"`)
- **THEN** no OHOS-specific code paths SHALL be included in the binary
- **AND** the compiled output SHALL be identical to the output before this change (modulo the new enum variant which is dead code)

#### Scenario: OHOS compilation does not affect Linux
- **WHEN** the application is compiled for standard Linux (`target_os = "linux"` without `target_env = "ohos"`)
- **THEN** `type_()` SHALL return `OsType::Linux`
- **AND** `platform()` SHALL return `"linux"`


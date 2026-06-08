## ADDED Requirements

### Requirement: OHOS enable autostart guides user to settings
The system SHALL open the system "应用启动管理" (App Startup Management) settings page when `enable()` is called on OHOS platform. The system SHALL use `startAbility` with the fixed Want targeting `com.huawei.hmos.settings` / `pc_app_setup_settings`.

#### Scenario: enable() on supported device
- **WHEN** the frontend calls `enable()` on OHOS platform
- **THEN** the system SHALL invoke `startAbility` with `bundleName: 'com.huawei.hmos.settings'`, `abilityName: 'com.huawei.hmos.settings.MainAbility'`, `uri: 'pc_app_setup_settings'`
- **THEN** the system settings page SHALL be displayed to the user

#### Scenario: enable() fails when settings page unavailable
- **WHEN** the frontend calls `enable()` on OHOS platform and `startAbility` returns an error
- **THEN** the system SHALL propagate the error to the caller with the error code and message

### Requirement: OHOS disable autostart guides user to settings
The system SHALL open the system "应用启动管理" settings page when `disable()` is called on OHOS platform. The behavior SHALL be identical to `enable()` since OHOS does not allow programmatic disable.

#### Scenario: disable() on supported device
- **WHEN** the frontend calls `disable()` on OHOS platform
- **THEN** the system SHALL invoke `startAbility` with the same Want as `enable()`
- **THEN** the system settings page SHALL be displayed to the user

### Requirement: OHOS isEnabled queries autostart status
The system SHALL query the current autostart status using `autoStartupManager.getAutoStartupStatusForSelf()` on OHOS platform when `isEnabled()` is called.

#### Scenario: isEnabled() returns true when autostart is on
- **WHEN** the frontend calls `isEnabled()` on OHOS platform and the user has enabled autostart for this app in system settings
- **THEN** the system SHALL return `true`

#### Scenario: isEnabled() returns false when autostart is off
- **WHEN** the frontend calls `isEnabled()` on OHOS platform and the user has not enabled autostart for this app
- **THEN** the system SHALL return `false`

#### Scenario: isEnabled() returns false on API version below 21
- **WHEN** the frontend calls `isEnabled()` on OHOS platform and `version::sdk_api_version() < 21`
- **THEN** the Rust side SHALL return `false` without invoking the TSFN (silent degradation, no ArkTS call)

#### Scenario: isEnabled() handles unsupported device
- **WHEN** the frontend calls `isEnabled()` on an OHOS device type that does not support `autoStartupManager` (error code 801)
- **THEN** the system SHALL return `false` (silent degradation)

### Requirement: OHOS autostart uses openharmony-ability bridge
All OHOS autostart system API calls SHALL be routed through `openharmony-ability` via TSFN bridge. Direct ArkTS API calls from `plugin-autostart` SHALL NOT be used.

#### Scenario: autostart calls go through TSFN
- **WHEN** `enable()`, `disable()`, or `isEnabled()` is called on OHOS
- **THEN** the call SHALL flow: plugin-autostart → openharmony-ability Rust API → TSFN → ArkTS helper → OHOS system API

### Requirement: OHOS autostart does not affect other platforms
The OHOS autostart implementation SHALL be isolated behind `cfg(target_env = "ohos")` and SHALL NOT affect the behavior of Windows, macOS, or Linux platforms.

#### Scenario: Windows/macOS/Linux autostart unchanged
- **WHEN** `enable()`, `disable()`, or `isEnabled()` is called on a non-OHOS platform
- **THEN** the system SHALL use the existing `auto-launch` crate implementation with no behavioral changes

### Requirement: OHOS autostart TSFN initialization
The system SHALL initialize autostart TSFNs during ability setup, after `set_main_thread_env` and `set_helper` are called.

#### Scenario: TSFNs available after init
- **WHEN** the OHOS ability has completed initialization
- **THEN** `AUTOSTART_ENABLE_TSFN`, `AUTOSTART_DISABLE_TSFN`, and `AUTOSTART_IS_ENABLED_TSFN` SHALL all be available for use

### Requirement: Frontend API test cases for OHOS autostart
The system SHALL provide test cases for the OHOS autostart implementation covering `auto`, `side-effect`, and `manual` categories.

#### Scenario: auto test for isEnabled
- **WHEN** the auto test suite runs
- **THEN** it SHALL verify that `isEnabled()` returns a `boolean` value without throwing

#### Scenario: side-effect test for enable/disable
- **WHEN** the side-effect test suite runs
- **THEN** it SHALL verify that `enable()` and `disable()` complete without throwing (settings page may or may not open depending on device)

#### Scenario: manual test for enable/disable UX
- **WHEN** the manual test suite runs
- **THEN** it SHALL prompt the tester to verify that calling `enable()` opens the system settings page with the app's autostart toggle visible

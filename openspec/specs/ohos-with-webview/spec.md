# ohos-with-webview Specification

## Purpose
TBD - created by archiving change p2-web-page-snapshot. Update Purpose after archive.
## Requirements
### Requirement: wry exposes WebViewExtOhos trait
wry SHALL provide a `WebViewExtOhos` trait (gated by `cfg(target_env = "ohos")`) with a `webview_handle()` method that returns the `openharmony_ability::Webview` instance.

#### Scenario: Access OHOS webview handle from wry
- **WHEN** user calls `WebViewExtOhos::webview_handle()` on a wry `WebView`
- **THEN** SHALL return a clone of the inner `openharmony_ability::Webview`

### Requirement: openharmony_ability::Webview is Send
`openharmony_ability::Webview` SHALL implement `Send` via `unsafe impl Send` to allow passing through the `WithWebview` message channel.

#### Scenario: Webview crosses thread boundary
- **WHEN** `Webview` is moved into a `Box<dyn FnOnce(Webview) + Send>` closure
- **THEN** SHALL compile without Send trait errors

### Requirement: tauri-runtime-wry OHOS Webview type is openharmony_ability::Webview
`tauri-runtime-wry` OHOS `Webview` type alias SHALL be `openharmony_ability::Webview` instead of `()`.

#### Scenario: WithWebview handler passes real handle
- **WHEN** `WebviewMessage::WithWebview` is processed on OHOS
- **THEN** handler SHALL call `WebViewExtOhos::webview_handle()` and pass the result to the closure, instead of logging a warning

### Requirement: PlatformWebview OHOS inner() accessor
`PlatformWebview` SHALL provide an `inner()` method (gated by `cfg(target_env = "ohos")`) returning `openharmony_ability::Webview`.

#### Scenario: User accesses OHOS webview via PlatformWebview
- **WHEN** user calls `webview.with_webview(|wv| { let handle = wv.inner(); })`
- **THEN** `handle` SHALL be an `openharmony_ability::Webview` instance with full access to Phase 1 methods (e.g., `web_page_snapshot`)


## 1. openharmony-ability Send 支持

- [x] 1.1 在 `crates/ability/src/helper/webview.rs` 为 `Webview` struct 添加 `unsafe impl Send for Webview {}`

## 2. wry WebViewExtOhos trait

- [x] 2.1 在 `wry/src/lib.rs` 新增 `WebViewExtOhos` trait 和 `impl WebViewExtOhos for WebView`（gated by `cfg(target_env = "ohos")`），`webview_handle()` 返回 `openharmony_ability::Webview`

## 3. tauri-runtime-wry OHOS 贯通

- [x] 3.1 在 `tauri-runtime-wry/src/webview.rs` 将 OHOS `Webview` 类型从 `()` 改为 `openharmony_ability::Webview`
- [x] 3.2 在 `tauri-runtime-wry/src/lib.rs` 的 `WebviewMessage::WithWebview` OHOS handler 中，将 `log::warn!` 替换为调用 `WebViewExtOhos::webview_handle()` 传递真实句柄

## 4. tauri core PlatformWebview

- [x] 4.1 在 `crates/tauri/src/webview/mod.rs` 的 `PlatformWebview` impl 中新增 OHOS `inner()` 方法（gated by `cfg(target_env = "ohos")`），返回 `openharmony_ability::Webview`

## 5. 验证

- [ ] 5.1 确认 `cargo check --target ohos` 编译通过（需 OHOS 构建环境）

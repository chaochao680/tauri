## Why

Tauri 已有 `with_webview` 机制让用户获取平台裸 webview 句柄，各平台已实现：Windows (`ICoreWebView2Controller`)、macOS (`WKWebView`)、Linux (`webkit2gtk::WebView`)、Android (`JniHandle`)。唯独 OHOS 的 `Webview` 类型是 `()`，`WithWebview` handler 仅打印 `log::warn!`。Phase 1 已在 openharmony-ability 层补全了 `webPageSnapshot` 能力，本 Phase 将其贯通到 Tauri 的 `with_webview` / `PlatformWebview` 层，让用户能拿到 OHOS webview 句柄。

## What Changes

- wry 新增 `WebViewExtOhos` trait，暴露 `webview_handle()` 返回 `openharmony_ability::Webview`
- `openharmony_ability::Webview` 添加 `unsafe impl Send`（安全：仅在主线程访问）
- tauri-runtime-wry OHOS `Webview` 类型从 `()` 改为 `openharmony_ability::Webview`
- tauri-runtime-wry `WithWebview` OHOS handler 从 `log::warn!` 改为传递真实句柄
- tauri core `PlatformWebview` 新增 OHOS `inner()` 访问器

## Capabilities

### New Capabilities
- `ohos-with-webview`: OHOS 平台 `with_webview` 裸 webview 句柄访问，让用户通过 `PlatformWebview::inner()` 获取 `openharmony_ability::Webview`

### Modified Capabilities

## Impact

- **代码**：wry、tauri-runtime-wry、tauri core 三个 crate，约 4 个文件
- **依赖**：tauri-runtime-wry 需依赖 `openharmony-ability`（OHOS 条件编译）
- **平台**：仅 OHOS（`cfg(target_env = "ohos")`），不影响其他平台

## Why

Tauri 已有 `with_webview` 机制让用户获取平台裸 webview 句柄（Windows 拿 `ICoreWebView2Controller`，macOS 拿 WKWebView 指针，Android 拿 `JniHandle`），但 OHOS 平台的 `Webview` 类型是 `()`，`WithWebview` 消息处理仅打印一行 `log::warn!`。这意味着 OHOS 上无法访问原生 WebviewController 的能力（如 `webPageSnapshot` 网页截图）。本 Phase 先补全 openharmony-ability 层的 `webPageSnapshot` 能力，为后续 Phase 2 贯通 `with_webview` 做准备。

## What Changes

- 在 openharmony-ability 的 ArkTS JsHelper 中新增 `webPageSnapshot` 方法，调用 OHOS `WebviewController.webPageSnapshot()` API（API 12+）
- ArkTS 侧将 `SnapshotResult.imagePixelMap` 转换为 RGBA `Uint8Array` 并返回宽高信息
- 在 openharmony-ability Rust `Webview` struct 上新增 `web_page_snapshot()` 方法，通过 NAPI 调用 ArkTS 侧方法，接收 RGBA bitmap 数据
- 遵循现有 clipboard 模块的 TSFN + oneshot channel 异步模式

## Capabilities

### New Capabilities
- `ohos-web-page-snapshot`: openharmony-ability 层的 OHOS WebView 全量网页截图能力，通过 `WebviewController.webPageSnapshot()` 获取 PixelMap 并转为 RGBA 数据返回给 Rust 侧

### Modified Capabilities
<!-- 无现有 spec 需要修改 -->

## Impact

- **代码**：openharmony-ability 仓库（ArkTS + Rust 侧），约 3 个文件
- **API**：新增 `Webview::web_page_snapshot()` 方法（Rust 侧公开 API）
- **依赖**：无新增外部依赖。`@kit.ImageKit`（PixelMap）已在 openharmony-ability 中使用
- **平台**：仅 OHOS（`cfg(target_env = "ohos")`），不影响其他平台
- **OHOS API 版本**：`WebviewController.webPageSnapshot` 从 API 12 开始支持

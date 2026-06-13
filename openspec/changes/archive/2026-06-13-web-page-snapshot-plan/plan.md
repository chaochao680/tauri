# WebView webPageSnapshot OHOS 适配计划

**创建时间**：2026-06-11
**功能描述**：实现 OHOS 平台 `with_webview` 裸句柄访问，并补全 `webPageSnapshot` 截图能力。不新增 Tauri 上层 API，仅做到平台裸句柄层。
**判断依据**：涉及 4 个代码层（openharmony-ability / wry / tauri-runtime-wry / tauri core），预估 6 个文件

## 背景

Tauri 已有 `with_webview` 机制让用户获取平台裸 webview 句柄：
- Windows: `ICoreWebView2Controller` + `ICoreWebView2Environment`（可调用 `CapturePreview`）
- macOS: `*mut c_void` WKWebView（可调用 `takeSnapshot`）
- Linux: `webkit2gtk::WebView`
- Android: `JniHandle`
- **OHOS: `()` — 未实现**，仅 `log::warn!("WithWebview is not implemented for OpenHarmony yet")`

目标：让 OHOS 也能通过 `with_webview` 拿到真实 webview 句柄，并支持 `webPageSnapshot` 截图。

## Phase 列表

| Phase | 名称 | openspec change | 状态 | 涉及层 | 预估文件 | 验证方式 |
|-------|------|----------------|------|--------|---------|---------|
| 1 | openharmony-ability 补 webPageSnapshot | p1-web-page-snapshot | ✓ 已归档 | openharmony-ability (ArkTS + Rust) | 3 | 设备端 NAPI 调用测试 |
| 2 | with_webview OHOS 贯通 | p2-web-page-snapshot | ✓ 已归档 | wry + tauri-runtime-wry + tauri core | 4 | cargo check + 设备端 with_webview 测试 |

## Phase 详细说明

### Phase 1: openharmony-ability 补 webPageSnapshot
- **目标**：openharmony-ability 的 `Webview` 句柄支持调用 OHOS `WebviewController.webPageSnapshot()` API，返回 RGBA bitmap 数据
- **文件列表**：
  - `native_ability/src/main/ets/webview/Utils.ets` — JsHelper 接口加 `webPageSnapshot` 方法声明
  - `native_ability/src/main/ets/webview/DefaultWebview.ets` — 实现 `webPageSnapshot`，调用 `controller.webPageSnapshot()`，将 `PixelMap` 转为 RGBA `Uint8Array` 返回
  - `crates/ability/src/helper/webview.rs` — Rust `Webview` struct 加 `web_page_snapshot()` 方法，通过 NAPI 调用 ArkTS 侧方法并接收 RGBA 数据
- **验证方式**：在设备端通过 NAPI 直接调用 `web_page_snapshot()`，确认能返回有效的 RGBA 数据
- **依赖**：无
- **OHOS API**：`WebviewController.webPageSnapshot(SnapshotInfo, AsyncCallback<SnapshotResult>)` — API 12+，返回 `PixelMap`（最大 16000×16000px）

### Phase 2: with_webview OHOS 贯通
- **目标**：用户通过 Tauri 的 `with_webview` 拿到 OHOS webview 真实句柄，参照其他平台模式使用
- **文件列表**：
  - `wry/src/lib.rs` — 新增 `WebViewExtOhos` trait（参照 `WebViewExtAndroid::handle()` 模式），暴露 `fn webview_handle(&self) -> openharmony_ability::Webview`
  - `tauri-runtime-wry/src/webview.rs` — OHOS `Webview` 类型从 `()` 改为 `openharmony_ability::Webview`
  - `tauri-runtime-wry/src/lib.rs` — `WebviewMessage::WithWebview` OHOS handler 从 `log::warn!` 改为调用 `WebViewExtOhos::webview_handle()` 传递真实句柄
  - `crates/tauri/src/webview/mod.rs` — `PlatformWebview` 加 OHOS `inner()` 访问器（参照 Linux/macOS/Android 的 `inner()`/`jni_handle()` 模式）
- **验证方式**：`cargo check --target ohos` 通过 + 设备端 `with_webview` 调用截图功能正常
- **依赖**：Phase 1 完成
- **关键约束**：`openharmony_ability::Webview` 含 `Rc<ObjectRef>`（非 Send），需处理跨线程传递问题（closure 通过 channel 发送到主线程执行）

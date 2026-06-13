## Context

Tauri 的 `with_webview` 机制通过消息分发传递平台裸 webview 句柄：

```
用户调用 with_webview(closure)
  → WebviewMessage::WithWebview(Box<dyn FnOnce(Webview) + Send>)
  → 主线程 handler 调用 webview.webview() 获取平台类型
  → 传给 closure
```

各平台 `Webview` 类型：
- Windows: `{ controller, environment }` — 含 COM 指针（Send）
- macOS: `{ webview: *mut c_void, manager: *mut c_void, ns_window: *mut c_void }` — 裸指针（Send）
- Android: `JniHandle` — 含 activity_id（Send）
- **OHOS: `()` — 未实现**

## Goals / Non-Goals

**Goals:**
- OHOS `with_webview` 传递真实的 `openharmony_ability::Webview` 句柄
- `PlatformWebview::inner()` 返回 `openharmony_ability::Webview`（参照其他平台模式）
- wry 新增 `WebViewExtOhos` trait（参照 `WebViewExtAndroid`）

**Non-Goals:**
- 不在 Tauri 上层添加任何截图 API
- 不修改 ArkTS 侧代码（Phase 1 已完成）

## Decisions

### D1: Send 安全性 — `unsafe impl Send for Webview`

`openharmony_ability::Webview` 含 `Rc<ObjectRef>` 非 Send。需添加 `unsafe impl Send`。

**安全性论证**：
- `Webview` 在 wry 主线程 event loop 中创建（`InnerWebView::new_as_child`）
- `WithWebview` closure 通过 channel 发送到主线程执行
- `Rc<ObjectRef>` 仅在主线程访问，无跨线程共享
- 与代码库已有模式一致：`SendableHelper`、`CustomProtocolResponder`、`OpenHarmonyApp` 均用 `unsafe impl Send`

### D2: wry 扩展 trait 命名 — `WebViewExtOhos`

参照 `WebViewExtAndroid::handle()` 和 `WebViewExtMacOS::webview()` 模式：
```rust
pub trait WebViewExtOhos {
    fn webview_handle(&self) -> openharmony_ability::Webview;
}
```

## Risks / Trade-offs

- **[Rc 跨线程]** → 通过 `unsafe impl Send` 解决，安全论证见 D1
- **[openharmony-ability 版本]** → tauri-runtime-wry 需添加 OHOS 条件编译依赖，已在 Cargo.toml 中通过 `target_env = "ohos"` 管理

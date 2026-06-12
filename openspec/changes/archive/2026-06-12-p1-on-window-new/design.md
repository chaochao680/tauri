## Context

Tauri 在 OHOS 平台上的 WebView 新窗口拦截功能完全缺失。当前 `wry/src/ohos/mod.rs:154` 显式丢弃 `new_window_req_handler`，openharmony-ability 的 ArkTS Web 组件未启用 `multiWindowAccess()` / `onWindowNew()` 事件。桌面平台（Windows/macOS/Linux）已有完整的拦截链路：`window.open()` → WebView Engine → wry platform handler → `NewWindowResponse::{Allow, Create, Deny}`。

**OHOS ArkWeb 关键约束**:
- `onWindowNew` 事件必须搭配 `multiWindowAccess(true)` 才能触发
- 回调内必须调用 `event.handler.setWebController(ctrl)` — 传 `null` = 阻止，传有效 controller = 允许
- **不调用 `setWebController` 会导致渲染进程永久阻塞**
- `OnWindowNewEvent` 提供 `targetUrl`, `isAlert`, `isUserTrigger` 字段（API 12+）

**线程模型**: `onWindowNew` 在 ArkTS 主线程触发，NAPI Function 同步调用 Rust 闭包，闭包立即返回结果。与 `on_navigation_request` 的同步回调模式一致，无死锁风险。

**参考设计文档**: [`tauri/doc/ohos-onwindownew-design.md`](../../doc/ohos-onwindownew-design.md)

## Goals / Non-Goals

**Goals:**
- 打通 OHOS 上 `on_new_window` 的完整拦截链路：ArkTS → openharmony-ability NAPI → wry → tauri-runtime-wry → tauri 用户 API
- 支持 `NewWindowResponse::Deny`：阻止新窗口打开
- 支持 `NewWindowResponse::Allow`：以 ArkTS dialog 形式打开新窗口（非 Tauri 管理）
- 为后续 `Create` 变体和 OS 级窗口创建预留扩展点

**Non-Goals:**
- `NewWindowResponse::Create` 变体支持（依赖 `ohos-os-level-window-design.md` 的 OS 级窗口基础设施）
- 多窗口同时打开（Phase 1 仅支持单个 dialog）
- `onWindowNewExt` 增强版事件（API 12+ 的 `NavigationPolicy` / `WindowFeatures` 信息）
- OHOS 上 Tauri window API 对新窗口的管理（close / resize / focus 等）

## Decisions

### D1: NAPI 回调返回 `{ allow: bool }` struct，不传 ControllerHandler

**选择**: 回调签名 `(targetUrl, isAlert, isUserTrigger) → OnWindowNewResult { allow: bool }`

**理由**:
- `ControllerHandler` 和 `WebviewController` 是 ArkTS 端对象，无法从 Rust 侧构造
- Rust handler 只负责决策（allow/deny），ArkTS 端负责 UI 创建（controller + dialog）
- 与 `on_navigation_request` 的 `string → bool` 模式一致，降低认知复杂度

**备选方案**:
- ~~异步 TSFN 模式~~: handler 需要等 Rust 回复再调用 `setWebController`，引入延迟和状态管理复杂度。同步模式更简单且 handler 闭包本身是立即返回的
- ~~返回完整 `NewWindowResponse` enum~~: 跨 NAPI 传 enum 增加序列化复杂度，Phase 1 只需 bool 即可

### D2: 同步 NAPI Function 调用，不使用 TSFN

**选择**: `env.create_function_from_closure()` 创建同步 NAPI 回调

**理由**:
- `onWindowNew` 在 ArkTS 主线程触发，NAPI Function 在同一线程调用 Rust 闭包
- handler 闭包是同步函数（`Fn(Url, NewWindowFeatures) -> NewWindowResponse`），立即返回
- 无 `run_on_main_thread + recv()` 死锁风险
- 与 `on_navigation_request` 模式一致

### D3: Allow 时使用 `@CustomDialog` 展示新窗口

**选择**: 创建 `@CustomDialog` 组件内嵌 `Web` 组件，将新 `WebviewController` 传给 `setWebController`

**理由**:
- `@CustomDialog` 是 ArkUI 原生的弹窗方式，不需要 OS 级窗口创建
- 用户体验一致：类似浏览器弹出窗口
- 可在 `WebBuilder` @Builder 内使用（通过 `CustomDialogController`）

**风险**: `@CustomDialog` 在模块级 `@Builder` 中可能无 `this` 上下文。**备选**: 使用 `promptAction.openCustomDialog()` (API 12+)

### D4: Deny 时同步调用 `setWebController(null)`

**选择**: 在 `onWindowNew` 回调内立即调用 `event.handler.setWebController(null)`

**理由**:
- 最安全的实现：确保渲染进程不会阻塞
- 无异步延迟，无状态管理

### D5: 无 handler 时默认 Deny

**选择**: 如果 `onWindowNew` NAPI 回调未注册（`None`），ArkTS 端直接调用 `setWebController(null)`

**理由**:
- 安全默认值：阻止未经处理的新窗口请求
- 与 Tauri 其他平台的行为一致（无 handler 时不打开新窗口）

### D6: OHOS `NewWindowResponse::Create` 降级为 `Allow`

**选择**: wry OHOS 层将 `Create` 响应降级为 `Allow`（创建 ArkTS dialog 而非 Tauri 管理的窗口）

**理由**:
- 与 mobile 平台行为一致（见 `tauri/crates/tauri/src/webview/mod.rs:722-724`）
- `Create` 需要 OS 级窗口创建基础设施，当前不可用
- 用户代码中的 `Create` 不会崩溃，而是降级为可用的最接近行为

## Risks / Trade-offs

| 风险 | 影响 | 缓解 |
|------|------|------|
| `@CustomDialog` 在 `@Builder` 中无 `this` 上下文 | dialog 创建失败 | 备选 `promptAction.openCustomDialog()` (API 12+)，实现阶段验证 |
| `OnWindowNewEvent.targetUrl` 字段 API 12+ | 低版本设备获取不到 URL | ArkTS fallback: `event.targetUrl ?? ''` |
| Allow 创建的窗口非 Tauri 管理 | 无法用 `getCurrentWindow()` 等 API 控制新窗口 | 文档明确标注限制；Phase 2 支持 `Create` 变体 |
| `setWebController` 异步场景渲染阻塞 | 新窗口卡死 | Deny 路径完全同步；Allow 路径 dialog 同步 open 后立即调用 |
| wry `NewWindowOpener` 缺少 OHOS 定义 | 编译错误 | 为 OHOS 添加空 struct 或 cfg 排除 |
| HAR 包重建后签名变更 | 安装失败 | 先 `bm uninstall` 再安装新版 |

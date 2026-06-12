## Why

OHOS 平台上，前端 `window.open()` 或 `<a target="_blank">` 触发的新窗口请求当前被 **静默丢弃**（`wry/src/ohos/mod.rs:154` — `let _ = new_window_req_handler`）。开发者通过 Tauri `on_new_window` API 设置的拦截回调在 OHOS 上完全无效，无法阻止或管理新窗口行为。这导致应用无法控制弹窗、广告或第三方链接的打开方式，是一个严重的功能缺失。

## What Changes

- **新增 `onWindowNew` 事件桥接**：在 `openharmony-ability` 的 ArkTS Web 组件中启用 `multiWindowAccess(true)` 和 `allowWindowOpenMethod(true)`，注册 `onWindowNew` 事件回调，通过 NAPI 同步调用 Rust 侧 handler 闭包
- **新增 NAPI 接口**：`WebViewInitData` 添加 `on_window_new` 回调字段（参数: `targetUrl`, `isAlert`, `isUserTrigger`，返回 `{ allow: bool }`）
- **新增 `WebViewBuilder.on_window_new()` 方法**：openharmony-ability Rust crate 提供 builder API 供上层注册 handler
- **wry OHOS 层桥接**：移除 `let _ = new_window_req_handler` 抑制，改为将 handler 桥接到 `WebViewBuilder.on_window_new()`
- **新增 `NewWindowDialog` ArkTS 组件**：Allow 时创建 `@CustomDialog` 嵌入新 Web 组件，提供 controller 给 `setWebController`
- **Phase 1 限制**：`NewWindowResponse::Create` 变体暂不支持（依赖 OS 级窗口创建基础设施），降级为 `Allow`

## Capabilities

### New Capabilities
- `ohos-on-window-new`: OHOS 平台新窗口请求拦截与管理——包括 ArkTS 事件注册、NAPI 回调桥接、Deny/Allow 决策、以及 Allow 时的 dialog 弹窗展示

### Modified Capabilities

（无现有 spec 需要修改——这是全新功能）

## Impact

- **openharmony-ability**: 6 个文件（含 1 个新建 `NewWindowDialog.ets`），影响 `WebViewInitData` NAPI 接口和 Web 组件 builder 链
- **wry**: 1-2 个文件（`src/ohos/mod.rs`，可能 `src/lib.rs` 的 `NewWindowOpener`）
- **tauri-runtime-wry**: 1 个文件（确认 OHOS 路径兼容性）
- **tauri**: 1 个文件（文档更新）
- **examples/api**: 1 个文件（示例代码更新）
- **HAR 包需要重建**：修改 openharmony-ability 后需 `ohrs build --arch arm64` + `pack.sh` + `ohpm install`
- **API 版本要求**: `onWindowNew` / `multiWindowAccess` / `ControllerHandler` 均为 API 9+，满足 tauri api demo 默认 API 12 要求；`OnWindowNewEvent` 的 `targetUrl` 等字段为 API 12+，低版本需 fallback

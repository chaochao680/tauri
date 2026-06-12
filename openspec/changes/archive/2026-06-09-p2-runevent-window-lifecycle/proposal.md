## Why

在 OHOS 平台上，子窗口通过 `WindowMessage::Destroy` 关闭时存在两个关键问题：

1. **`Destroyed` 事件缺失**：`on_window_close` 函数只设置 `window_wrapper.inner = None`，不从 `WindowsStore` 移除条目，也不发送 `WindowEvent::Destroyed` 事件。这导致用户代码无法监听到子窗口的销毁事件。

2. **`ExitRequested` 路径 A 无法触发**：由于 `WindowsStore` 中的条目没有被移除，`windows.is_empty()` 永远返回 `false`，导致即使所有窗口都关闭了，`ExitRequested` 事件也不会被触发（路径 A：所有窗口关闭时触发退出请求）。

这两个问题导致 OHOS 平台上的窗口生命周期管理与 Windows/macOS 不一致，影响应用的可移植性和用户体验。

## What Changes

- 重构 `on_window_close` 函数，使其执行完整的窗口清理逻辑：从 `WindowsStore` 移除条目、发送 `WindowEvent::Destroyed` 事件、检查是否需要触发 `ExitRequested`
- 修改 `WindowMessage::Destroy` 处理器，在调用 `on_window_close` 之前先调用 `on_close_requested` 以发送 `CloseRequested` 事件
- 确保 `TaoWindowEvent::Destroyed` 处理器使用重构后的 `on_window_close` 函数，避免重复清理
- 在 `on_window_close` 中添加 `ExitState` 标志检查，防止与 `LoopDestroyed` 路径重复触发 `ExitRequested`

## Capabilities

### New Capabilities
- `ohos-window-lifecycle`: OHOS 平台的窗口生命周期管理 — 确保子窗口销毁时正确发送 `CloseRequested` 和 `Destroyed` 事件，正确清理 `WindowsStore`，并在所有窗口关闭时触发 `ExitRequested`

### Modified Capabilities
<!-- 无需修改现有 capability -->

## Impact

- **tauri-runtime-wry** (`crates/tauri-runtime-wry/src/lib.rs`): 重构 `on_window_close` 函数，修改 `WindowMessage::Destroy` 处理器，更新 `TaoWindowEvent::Destroyed` 处理器
- **examples/api** (`examples/api/src-tauri/src/lib.rs`): 添加测试代码验证子窗口生命周期事件
- **行为一致性**: OHOS 平台的窗口生命周期行为将与 Windows/macOS 保持一致

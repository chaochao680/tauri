# ohos-window-lifecycle Specification

## Purpose
TBD - created by archiving change p2-runevent-window-lifecycle. Update Purpose after archive.
## Requirements
### Requirement: 子窗口 Destroyed 事件正确触发
当子窗口关闭时，系统 SHALL 正确触发 `WindowEvent::Destroyed` 事件。触发路径根据消息类型有所不同：

#### Scenario: 用户代码调用 window.close() 关闭子窗口
- **WHEN** 用户代码调用 `window.close()` 关闭子窗口
- **THEN** 系统发送 `WindowMessage::Close` 消息
- **THEN** 处理器调用 `on_close_requested` 发送 `WindowEvent::CloseRequested` 事件
- **THEN** 如果用户未调用 `api.prevent_close()`，`on_close_requested` 内部调用 `on_window_close` 执行清理
- **THEN** `on_window_close` 发送 `WindowEvent::Destroyed` 事件

#### Scenario: OHOS pending close drain 关闭子窗口
- **WHEN** ArkTS 拦截 close-window URL 并通过 `notifyWindowClose()` NAPI 通知 Rust
- **THEN** Rust event loop drain 调用 `on_close_requested` 发送 `WindowEvent::CloseRequested` 事件
- **THEN** 如果用户未调用 `api.prevent_close()`，`on_close_requested` 内部调用 `on_window_close` 执行清理
- **THEN** `on_window_close` 发送 `WindowEvent::Destroyed` 事件

#### Scenario: 系统直接发送 WindowMessage::Destroy（不触发 CloseRequested）
- **WHEN** 系统发送 `WindowMessage::Destroy` 消息（如 tao 内部销毁路径）
- **THEN** 处理器直接调用 `on_window_close`，**不**触发 `WindowEvent::CloseRequested` 事件
- **THEN** `on_window_close` 发送 `WindowEvent::Destroyed` 事件
- **NOTE**: 跳过 `CloseRequested` 是为了避免循环调用（`on_close_requested` 内部可能触发 `Destroy`，如果 `Destroy` 又触发 `CloseRequested` 则形成死循环）

#### Scenario: 用户代码在 CloseRequested 事件中阻止关闭
- **WHEN** 子窗口收到 `WindowEvent::CloseRequested` 事件（通过 `window.close()` 或 pending close drain）
- **WHEN** 用户代码在 `CloseRequested` 事件中调用 `api.prevent_close()`
- **THEN** 窗口不被销毁，`WindowEvent::Destroyed` 事件不被发送
- **THEN** `WindowsStore` 中的条目不被移除

### Requirement: WindowsStore 条目正确移除
当窗口被销毁时，系统 SHALL 从 `WindowsStore` 中移除对应的窗口条目，而不仅仅是设置 `inner = None`。

#### Scenario: 子窗口销毁后 WindowsStore 条目被移除
- **WHEN** 子窗口被销毁（通过 `WindowMessage::Close` + `on_close_requested`、`WindowMessage::Destroy` 或 `TaoWindowEvent::Destroyed`）
- **THEN** `on_window_close` 函数调用 `windows.0.borrow_mut().remove(&window_id)`
- **THEN** `WindowsStore` 中不再包含该窗口的条目
- **THEN** 后续调用 `windows.0.borrow().is_empty()` 能正确反映剩余窗口数量

#### Scenario: 主窗口销毁后 WindowsStore 条目被移除
- **WHEN** 主窗口被销毁（通过 `TaoWindowEvent::Destroyed`）
- **THEN** `on_window_close` 函数调用 `windows.0.borrow_mut().remove(&window_id)`
- **THEN** `WindowsStore` 中不再包含该窗口的条目

#### Scenario: 多次调用 on_window_close 是幂等的
- **WHEN** `on_window_close` 被多次调用（例如 `on_close_requested` 和 `TaoWindowEvent::Destroyed` 都调用）
- **THEN** 第一次调用移除条目并发送 `Destroyed` 事件
- **THEN** 后续调用检查 `remove` 返回值，如果窗口已被移除则直接返回
- **THEN** 不会重复发送 `Destroyed` 事件

### Requirement: 所有窗口关闭时触发 ExitRequested
当 `WindowsStore` 中所有窗口都被销毁后，系统 SHALL 触发 `ExitRequested` 事件。

#### Scenario: 最后一个窗口关闭时触发 ExitRequested
- **WHEN** 最后一个窗口被销毁
- **THEN** `on_window_close` 检查 `windows.0.borrow().is_empty()` 返回 `true`
- **THEN** 系统发送 `RunEvent::ExitRequested { code: None, tx }` 事件
- **THEN** 系统设置 `exit_state.0.store(true, Ordering::SeqCst)` 标志
- **THEN** 如果用户代码未调用 `api.prevent_exit()`，系统设置 `ControlFlow::Exit`

#### Scenario: ExitRequested 与 LoopDestroyed 路径不重复触发
- **WHEN** 所有窗口通过 `on_window_close` 关闭（无论触发路径），触发 `ExitRequested`
- **THEN** `exit_state` 标志被设置为 `true`
- **WHEN** 随后 `LoopDestroyed` 事件触发
- **THEN** `LoopDestroyed` 处理器检查 `exit_state` 标志为 `true`
- **THEN** `LoopDestroyed` 处理器跳过 `ExitRequested`，直接发送 `RunEvent::Exit`

### Requirement: on_window_close 函数签名重构
`on_window_close` 函数 SHALL 接受 `callback` 和 `exit_state` 参数，以支持发送事件和检查标志。

#### Scenario: on_window_close 函数签名
- **WHEN** `on_window_close` 被调用
- **THEN** 函数签名为 `fn on_window_close<'a, T: UserEvent>(callback: &'a mut (dyn FnMut(RunEvent<T>) + 'static), window_id: WindowId, windows: Arc<WindowsStore>, exit_state: Arc<ExitState>)`
- **THEN** 函数在 Windows 平台上先调用 `surface.take()` 释放 softbuffer surface（必须在 window 之前 drop，否则 surface 释放时可能访问已释放的窗口资源）
- **THEN** 函数能够发送 `WindowEvent::Destroyed` 事件
- **THEN** 函数能够检查和设置 `exit_state` 标志

#### Scenario: 所有调用方传递新参数
- **WHEN** `WindowMessage::Close` 处理器调用 `on_close_requested`
- **THEN** `on_close_requested` 内部调用 `on_window_close` 时传递 `callback` 和 `exit_state`
- **WHEN** `WindowMessage::Destroy` 处理器直接调用 `on_window_close`（不经过 `on_close_requested`，避免循环）
- **THEN** 传递 `callback` 和 `exit_state` 参数
- **WHEN** `TaoWindowEvent::Destroyed` 处理器直接调用 `on_window_close`
- **THEN** 传递 `callback` 和 `exit_state` 参数

### Requirement: cfg 隔离
OHOS 平台的窗口生命周期修复 SHALL 仅在 `cfg(target_env = "ohos")` 条件下生效，不影响 Windows/macOS 行为。

#### Scenario: Windows/macOS 不受影响
- **WHEN** 应用在 Windows 或 macOS 上运行
- **THEN** `on_window_close` 保持原有行为（如果有的话）
- **THEN** `WindowEvent::Destroyed` 仍通过 `TaoWindowEvent::Destroyed` 路径触发
- **THEN** `ExitRequested` 仍通过原有路径触发

#### Scenario: OHOS 使用新行为
- **WHEN** 应用在 OHOS 上运行
- **THEN** `on_window_close` 执行完整清理逻辑（包含 `surface.take()` 保持 drop 顺序）
- **THEN** `WindowMessage::Close` 调用 `on_close_requested`（触发 `CloseRequested`）
- **THEN** `WindowMessage::Destroy` 直接调用 `on_window_close`（不触发 `CloseRequested`）
- **THEN** `TaoWindowEvent::Destroyed` 调用重构后的 `on_window_close`


## ADDED Requirements

### Requirement: OHOS LoopDestroyed 触发 ExitRequested
当 OHOS 系统销毁 UIAbility 时（`Event::LoopDestroyed`），tauri-runtime-wry SHALL 先发送 `RunEvent::ExitRequested`，再发送 `RunEvent::Exit`。用户代码可通过 `api.prevent_exit()` 接收退出通知并执行清理逻辑。

#### Scenario: 系统关闭应用时触发 ExitRequested
- **WHEN** OHOS 系统销毁 UIAbility，tao 发送 `Event::LoopDestroyed`
- **THEN** tauri-runtime-wry 先发送 `RunEvent::ExitRequested { code: None, tx }`
- **THEN** tauri-runtime-wry 发送 `RunEvent::Exit`

#### Scenario: 用户代码收到 ExitRequested 并执行清理
- **WHEN** `RunEvent::ExitRequested` 被发送
- **THEN** 用户 `app.run()` 回调接收到该事件
- **THEN** 用户代码可执行清理逻辑（保存数据、发送日志等）

#### Scenario: prevent_exit 在 LoopDestroyed 路径上的行为
- **WHEN** 用户代码在 `ExitRequested` handler 中调用 `api.prevent_exit()`
- **THEN** `tx` 发送 `ExitRequestedEventAction::Prevent`
- **THEN** tauri-runtime-wry 收到 Prevent 信号
- **THEN** 由于系统已开始销毁流程，`prevent_exit()` 不阻止最终退出
- **THEN** `RunEvent::Exit` 仍然被发送

### Requirement: ExitRequested 防重复触发
tauri-runtime-wry SHALL 使用原子标志防止 `RunEvent::ExitRequested` 被重复触发（如窗口关闭路径和 LoopDestroyed 路径同时触发）。

#### Scenario: 最后一个窗口关闭后系统销毁
- **WHEN** 最后一个窗口被销毁（`TaoWindowEvent::Destroyed`），触发路径 A 的 `ExitRequested`
- **THEN** 设置 `exit_requested_sent` 标志为 `true`
- **WHEN** 随后 `LoopDestroyed` 触发
- **THEN** 检查 `exit_requested_sent` 为 `true`
- **THEN** 跳过 `ExitRequested`，直接发送 `RunEvent::Exit`

#### Scenario: 系统直接销毁（无窗口关闭事件）
- **WHEN** 系统直接销毁 UIAbility（无 `TaoWindowEvent::Destroyed` 先触发）
- **THEN** `exit_requested_sent` 标志为 `false`
- **WHEN** `LoopDestroyed` 触发
- **THEN** 发送 `RunEvent::ExitRequested { code: None }`
- **THEN** 发送 `RunEvent::Exit`

### Requirement: cfg 隔离
OHOS 的 `LoopDestroyed` handler 修改 SHALL 仅在 `cfg(target_env = "ohos")` 条件下生效，不影响 Windows/macOS 行为。

#### Scenario: Windows/macOS 不受影响
- **WHEN** 应用在 Windows 或 macOS 上运行
- **THEN** `LoopDestroyed` handler 保持原有行为（如果有的话）
- **THEN** `ExitRequested` 仍通过窗口销毁和 RequestExit 路径触发

#### Scenario: OHOS 使用新行为
- **WHEN** 应用在 OHOS 上运行
- **THEN** `LoopDestroyed` handler 先发送 `ExitRequested` 再发送 `Exit`

### Requirement: onPrepareToTerminate 增强路径（后续 Phase）
当 OHOS 系统参数 `persist.sys.prepare_terminate = true` 时，系统 SHALL 支持通过 `UIAbility.onPrepareToTerminate` 回调实现真正可阻止的退出拦截。此功能不在 Phase 1 范围内，作为后续增强方向。

#### Scenario: onPrepareToTerminate 增强可用时
- **WHEN** 系统参数 `persist.sys.prepare_terminate = true` 且 `onPrepareToTerminate` 回调被触发
- **THEN** 可通过返回值真正取消终止（`true` = 取消 或 `false` = 取消，语义需设备验证）
- **THEN** 使用 `AtomicBool` 防止与 `LoopDestroyed` 路径重复触发 `ExitRequested`

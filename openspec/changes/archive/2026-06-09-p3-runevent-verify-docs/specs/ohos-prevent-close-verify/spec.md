## ADDED Requirements

### Requirement: prevent_close 在子窗口上正确拦截关闭
当用户代码在 `CloseRequested` 事件中调用 `api.prevent_close()` 时，子窗口 SHALL 不被销毁，`WindowEvent::Destroyed` 事件 SHALL 不被发送。

#### Scenario: 子窗口 prevent_close 生效
- **WHEN** 子窗口收到 `CloseRequested` 事件
- **WHEN** 用户代码调用 `api.prevent_close()`
- **THEN** `on_close_requested` 函数检查 `rx.try_recv()` 返回 `Ok(true)`
- **THEN** `on_window_close` 不被调用
- **THEN** 窗口保持在 `WindowsStore` 中
- **THEN** `WindowEvent::Destroyed` 事件不被发送

#### Scenario: 子窗口未阻止关闭
- **WHEN** 子窗口收到 `CloseRequested` 事件
- **WHEN** 用户代码未调用 `api.prevent_close()`
- **THEN** `on_close_requested` 函数检查 `rx.try_recv()` 返回 `Err` 或非 `Ok(true)`
- **THEN** `on_window_close` 被调用
- **THEN** 窗口从 `WindowsStore` 中移除
- **THEN** `WindowEvent::Destroyed` 事件被发送

### Requirement: 架构文档反映最新实现状态
`doc/RunEvent_Architecture.md` SHALL 准确反映 Phase 1 和 Phase 2 的实现成果，所有 TODO 标记 SHALL 有明确的状态更新。

#### Scenario: Opened 事件状态更新
- **WHEN** 开发者查阅 `doc/RunEvent_Architecture.md` 中 Opened 事件章节
- **THEN** 状态标注为 ✅ 已实现
- **THEN** 说明通过 `MainEvent::NewWant` → `Event::Opened` 路径实现

#### Scenario: Resumed 事件状态标注
- **WHEN** 开发者查阅 `doc/RunEvent_Architecture.md` 中 Resumed 事件章节
- **THEN** 状态标注为跨平台遗留问题（死代码）
- **THEN** 说明 `StartCause::Poll → Resumed` 在 `ControlFlow::Wait` 下永远不触发
- **THEN** 说明 `Event::Resumed` 被 `_ => ()` 丢弃

#### Scenario: prevent_close 验证状态更新
- **WHEN** 开发者查阅 `doc/RunEvent_Architecture.md` 中 prevent_close 章节
- **THEN** 状态根据设备验证结果更新
- **THEN** 如果验证通过，标注为 ✅ 已验证
- **THEN** 如果验证未通过，标注为已知限制并说明原因

### Requirement: OHOS 分析文档同步更新
`doc/RunEvent_OHOS分析与设计.md` SHALL 同步更新，反映 Phase 1 和 Phase 2 的实现结论。

#### Scenario: ExitRequested 实现结论更新
- **WHEN** 开发者查阅 `doc/RunEvent_OHOS分析与设计.md` 中 ExitRequested 章节
- **THEN** 记录 Phase 1 的实现方案（`LoopDestroyed` handler 先触发 `ExitRequested`）
- **THEN** 记录 `ExitState(AtomicBool)` 防重复机制
- **THEN** 记录 `onPrepareToTerminate` 作为后续增强方向

#### Scenario: 子窗口 Destroyed 实现结论更新
- **WHEN** 开发者查阅 `doc/RunEvent_OHOS分析与设计.md` 中子窗口生命周期章节
- **THEN** 记录 Phase 2 的实现方案（重构 `on_window_close`）
- **THEN** 记录 `WindowMessage::Destroy` 先调用 `on_close_requested` 再调用 `on_window_close`

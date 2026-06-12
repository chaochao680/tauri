## Context

在 Tauri 应用中，窗口生命周期管理依赖于正确的事件触发顺序。在 Windows/macOS 平台上，窗口销毁的完整流程是：

1. 用户点击关闭按钮 → `TaoWindowEvent::CloseRequested` → 发送 `WindowEvent::CloseRequested`
2. 如果用户代码未阻止关闭 → tao 销毁窗口 → `TaoWindowEvent::Destroyed` → 从 `WindowsStore` 移除 → 发送 `WindowEvent::Destroyed`
3. 如果所有窗口都已销毁 → 触发 `ExitRequested`

在 OHOS 平台上，主窗口的销毁由系统事件驱动（`MainEvent::WindowDestroy`），tao 会正确触发 `CloseRequested` 和 `Destroyed`。但子窗口的销毁通过应用代码调用 `window.destroy()`，这会发送 `WindowMessage::Destroy` 消息，当前处理逻辑不完整。

## Goals / Non-Goals

**Goals:**
- 确保子窗口通过 `WindowMessage::Destroy` 关闭时，正确发送 `CloseRequested` 和 `Destroyed` 事件
- 确保 `WindowsStore` 中的条目在窗口销毁后被正确移除
- 确保所有窗口关闭时能够触发 `ExitRequested` 事件
- 与 Windows/macOS 平台的窗口生命周期行为保持一致

**Non-Goals:**
- 不修改主窗口的销毁流程（已由系统事件正确处理）
- 不修改 `on_close_requested` 的阻止逻辑（`prevent_close` 验证将在 Phase 3 进行）
- 不处理 `Resumed`/`Suspended` 跨平台遗留问题

## Decisions

### Decision 1: 重构 `on_window_close` 为完整清理函数

**选择**: 将 `on_window_close` 重构为执行完整清理逻辑的函数，包括：
1. 从 `WindowsStore` 移除窗口条目
2. 发送 `WindowEvent::Destroyed` 事件
3. 检查 `WindowsStore` 是否为空，如果为空则触发 `ExitRequested`

**替代方案**:
- **保留 `on_window_close` 现状，在 `WindowMessage::Destroy` 中手动清理**: 被排除，会导致代码重复，且容易遗漏清理步骤
- **让 `WindowMessage::Destroy` 触发 `TaoWindowEvent::Destroyed`**: 被排除，因为 `TaoWindowEvent::Destroyed` 是由 tao 事件循环驱动的，我们无法从消息处理器中直接触发 tao 事件

**理由**:
- 集中清理逻辑，避免重复代码
- 确保所有窗口销毁路径（`WindowMessage::Destroy`、`TaoWindowEvent::Destroyed`）都执行相同的清理步骤
- 与 Windows/macOS 的行为语义一致

### Decision 2: `WindowMessage::Destroy` 先调用 `on_close_requested`

**选择**: 修改 `WindowMessage::Destroy` 处理器，在调用重构后的 `on_window_close` 之前，先调用 `on_close_requested` 以发送 `CloseRequested` 事件

**替代方案**:
- **直接调用 `on_window_close`，不发送 `CloseRequested`**: 被排除，会导致用户代码无法监听到关闭请求事件，与 Windows/macOS 行为不一致

**理由**:
- 保持事件触发顺序：`CloseRequested` → `Destroyed`
- 允许用户代码在 `CloseRequested` 中执行清理逻辑
- 与 Windows/macOS 的行为一致

### Decision 3: `TaoWindowEvent::Destroyed` 使用重构后的 `on_window_close`

**选择**: 修改 `TaoWindowEvent::Destroyed` 处理器，调用重构后的 `on_window_close` 函数，并添加 `ExitState` 标志检查以避免重复触发 `ExitRequested`

**替代方案**:
- **保留现有逻辑，手动移除和检查**: 被排除，会导致代码重复，且与 `WindowMessage::Destroy` 路径的清理逻辑不一致

**理由**:
- 统一清理路径，避免重复代码
- 通过 `ExitState` 标志确保 `ExitRequested` 不会被重复触发（Phase 1 已引入此机制）

### Decision 4: 保持 `on_close_requested` 不变

**选择**: 不修改 `on_close_requested` 的逻辑，它仍然在发送 `CloseRequested` 后调用 `on_window_close`

**理由**:
- `on_close_requested` 的逻辑已经正确：发送 `CloseRequested` → 检查是否被阻止 → 如果未阻止则调用 `on_window_close`
- 重构后的 `on_window_close` 会自动处理后续的清理和 `ExitRequested` 触发

### Decision 5: `ExitRequested` 使用 `code: None`

**选择**: 在 `on_window_close` 中触发 `ExitRequested` 时使用 `code: None`（与 Phase 1 的 `LoopDestroyed` 路径一致）

**理由**:
- `code: None` 表示系统关闭或所有窗口关闭，语义上更接近"应用被系统关闭"
- 与 Phase 1 的设计保持一致
- `Some(code)` 保留给 `Message::RequestExit(code)` 路径（用户主动请求退出）

## Risks / Trade-offs

### Risk 1: `on_close_requested` 可能重复调用 `on_window_close`

**风险**: `on_close_requested` 在发送 `CloseRequested` 后调用 `on_window_close`，而 `WindowMessage::Destroy` 也会调用 `on_close_requested`，可能导致 `on_window_close` 被调用两次。

**缓解**:
- `on_window_close` 在移除窗口时会检查 `windows.0.borrow_mut().remove(&window_id).is_some()`，如果窗口已被移除则直接返回
- 这是幂等操作，多次调用不会产生副作用

### Risk 2: `ExitRequested` 可能在 `LoopDestroyed` 路径上被重复触发

**风险**: 如果所有窗口都通过 `WindowMessage::Destroy` 关闭，`on_window_close` 会触发 `ExitRequested`，随后 `LoopDestroyed` 也会触发。

**缓解**:
- 使用 Phase 1 引入的 `ExitState` 标志，`on_window_close` 在触发 `ExitRequested` 时设置标志为 `true`
- `LoopDestroyed` 检查标志，仅在 `false` 时发送 `ExitRequested`
- 这与 Phase 1 的防重复机制完全一致

### Trade-off: 重构 `on_window_close` 增加函数复杂度

**权衡**: 重构后的 `on_window_close` 函数职责增加（移除条目 + 发送事件 + 检查空 + 触发退出），但换取了：
- 集中清理逻辑，避免代码重复
- 确保所有销毁路径行为一致
- 简化了调用方的逻辑（只需调用一个函数）

## Implementation Notes

### 函数签名变化

```rust
// 旧签名
fn on_window_close(window_id: WindowId, windows: Arc<WindowsStore>)

// 新签名
fn on_window_close<'a, T: UserEvent>(
  callback: &'a mut (dyn FnMut(RunEvent<T>) + 'static),
  window_id: WindowId,
  windows: Arc<WindowsStore>,
  exit_state: Arc<ExitState>,
)
```

需要添加 `callback` 和 `exit_state` 参数，因为函数现在需要发送事件和检查标志。

### 调用方变化

1. **`WindowMessage::Destroy`**:
   ```rust
   Message::Window(id, WindowMessage::Destroy) => {
     on_close_requested(callback, id, windows);  // 先发送 CloseRequested
     // on_close_requested 内部会调用 on_window_close
   }
   ```

2. **`TaoWindowEvent::Destroyed`**:
   ```rust
   TaoWindowEvent::Destroyed => {
     on_window_close(callback, window_id, windows, exit_state);
   }
   ```

3. **`on_close_requested`** (内部调用):
   ```rust
   if let Ok(true) = rx.try_recv() {
     // 用户阻止了关闭，不调用 on_window_close
   } else {
     on_window_close(callback, window_id, windows, exit_state);
   }
   ```

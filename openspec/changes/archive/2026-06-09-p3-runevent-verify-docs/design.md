## Context

Phase 1 和 Phase 2 完成后，OHOS 平台的 RunEvent 实现已大幅改善：
- `ExitRequested` 在 `LoopDestroyed` 路径上正确触发
- 子窗口 `Destroyed` 事件正确触发
- `WindowsStore` 条目正确清理

但 `prevent_close()` 在 OHOS 上的行为仍需要验证。当前 `on_close_requested` 函数的逻辑是：

```rust
fn on_close_requested(...) {
  // 1. 发送 CloseRequested 事件
  callback(RunEvent::WindowEvent {
    label,
    event: WindowEvent::CloseRequested { signal_tx: tx },
  });
  // 2. 检查是否被阻止
  if let Ok(true) = rx.try_recv() {
    // 用户阻止了关闭，不调用 on_window_close
  } else {
    on_window_close(callback, window_id, windows, exit_state);
  }
}
```

问题在于：在 OHOS 上，`TaoWindowEvent::CloseRequested` 是由 `MainEvent::WindowDestroy`（主窗口）或 `WindowMessage::Close`/`WindowMessage::Destroy`（子窗口）触发的。对于主窗口，`MainEvent::WindowDestroy` 是系统事件，`prevent_close()` 可能无法阻止系统销毁窗口的行为。

## Goals / Non-Goals

**Goals:**
- 验证 `prevent_close()` 在 OHOS 子窗口上是否生效
- 更新架构文档，反映 Phase 1 和 Phase 2 的实现成果
- 记录 OHOS 平台的已知限制

**Non-Goals:**
- 不修改 `prevent_close()` 的底层实现（如果验证发现不生效，记录为已知限制）
- 不修复 `Resumed`/`Suspended` 跨平台遗留问题

## Decisions

### Decision 1: prevent_close 验证方法

**选择**: 在 `examples/api/src-tauri/src/lib.rs` 中修改 `CloseRequested` 处理逻辑：
1. 调用 `api.prevent_close()` 阻止关闭
2. 不调用 `destroy()`（当前代码在 `prevent_close()` 后紧接 `destroy()`，无法验证）
3. 记录日志，观察窗口是否保持打开

**理由**:
- 当前代码 `prevent_close()` 后紧接 `destroy()`，窗口仍然被关闭，无法验证 `prevent_close()` 是否生效
- 移除 `destroy()` 调用后，如果 `prevent_close()` 生效，窗口应保持打开
- 如果窗口仍然关闭，说明 `prevent_close()` 在 OHOS 上无效，需要记录为已知限制

### Decision 2: 文档更新范围

**选择**: 更新以下文档：
1. `doc/RunEvent_Architecture.md`:
   - TODO #4 (Opened): 状态从 ⚠️ 改为 ✅（代码已实现）
   - TODO #5 (prevent_close): 根据验证结果更新
   - Resumed 事件: 标注为跨平台遗留（死代码）
2. `doc/RunEvent_OHOS分析与设计.md`: 同步实现结论

**理由**:
- 架构文档是 RunEvent 实现的权威参考，必须与实际实现保持一致
- OHOS 分析文档记录了设计决策，需要同步更新

### Decision 3: 已知限制记录

**选择**: 在文档中记录以下 OHOS 平台已知限制：
1. `prevent_exit()` 在 `LoopDestroyed` 路径上可能无法真正阻止退出（系统已开始销毁）
2. `onPrepareToTerminate` 需要系统参数 `persist.sys.prepare_terminate = true`（非默认开启）
3. `Resumed`/`Suspended` 是跨平台遗留问题，不在 OHOS 适配范围内

**理由**:
- 明确记录限制，避免后续开发者重复踩坑
- 为后续增强（如 `onPrepareToTerminate` 实现）提供方向

## Risks / Trade-offs

### Risk 1: prevent_close 在 OHOS 主窗口上不生效

**风险**: OHOS 主窗口的 `CloseRequested` 由 `MainEvent::WindowDestroy`（系统事件）触发，`prevent_close()` 可能无法阻止系统销毁窗口。

**缓解**:
- 如果验证发现不生效，记录为已知限制
- 后续可通过 `onPrepareToTerminate` 增强实现真正的阻止退出

### Risk 2: 文档更新遗漏

**风险**: 架构文档中有多个 TODO，可能遗漏某些需要更新的地方。

**缓解**:
- 逐项检查所有 TODO，确保每个都有明确的状态更新
- 使用 `grep` 搜索文档中的所有 TODO 标记

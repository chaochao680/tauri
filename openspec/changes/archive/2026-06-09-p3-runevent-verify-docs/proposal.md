## Why

Phase 1 和 Phase 2 已修复了 OHOS 平台上 RunEvent 的核心实现差距：
- Phase 1: `LoopDestroyed` 路径正确触发 `ExitRequested`
- Phase 2: 子窗口 `Destroyed` 事件正确触发，`WindowsStore` 条目正确清理

但仍有一个验证差距：`RunEvent::WindowEvent::CloseRequested` 事件中的 `api.prevent_close()` 在 OHOS 上是否真正阻止窗口关闭尚未验证。

同时，架构文档 `doc/RunEvent_Architecture.md` 中仍有若干 TODO 状态需要更新：
- TODO #4: Opened 事件 — 代码已实现，文档仍标为"未调通"
- TODO #5: prevent_close 拦截能力 — 需要验证并更新文档

## What Changes

- 在 `examples/api/src-tauri/src/lib.rs` 中添加 `prevent_close` 验证测试代码
- 更新 `doc/RunEvent_Architecture.md`：
  - TODO #4 Opened 状态从 ⚠️ 改为 ✅
  - TODO #5 prevent_close 验证状态更新
  - Resumed 事件状态标注为跨平台遗留（死代码）
- 同步更新 `doc/RunEvent_OHOS分析与设计.md` 中与实现一致的结论

## Capabilities

### New Capabilities
- `ohos-prevent-close-verify`: OHOS 平台 `prevent_close()` 拦截能力验证 — 确保用户代码调用 `api.prevent_close()` 后窗口不被销毁

### Modified Capabilities
<!-- 无需修改现有 capability -->

## Impact

- **examples/api** (`examples/api/src-tauri/src/lib.rs`): 调整 CloseRequested 测试代码
- **文档** (`doc/RunEvent_Architecture.md`): 更新多个 TODO 状态
- **文档** (`doc/RunEvent_OHOS分析与设计.md`): 同步实现结论

# RunEvent OHOS 差距修复 适配计划

**创建时间**：2026-06-08
**功能描述**：修复 RunEvent 在 OHOS 平台上的实现差距，包括退出流程、子窗口生命周期、拦截验证和文档更新
**判断依据**：涉及 3 个代码层（openharmony-ability + tao + tauri-runtime-wry），预估 ~12 个文件（跨 3 个 Phase）

## 背景

基于 `doc/RunEvent_Architecture.md`（现状文档）和 `doc/RunEvent_OHOS分析与设计.md`（设计文档）的差距分析，确认以下待修复项：

| TODO | 问题 | 严重度 | 本次是否解决 |
|------|------|--------|-------------|
| #1 | ExitRequested/Exit 退出流程 — LoopDestroyed 直接发 Exit，跳过 ExitRequested | 🔴 高 | ✅ |
| #2 | 子窗口 Destroyed 缺失 — WindowMessage::Destroy 不触发 Destroyed 事件 | 🟡 中 | ✅ |
| #3 | WindowsStore 清理 — on_window_close 只设 inner=None 不移除条目 | 🟡 中 | ✅ |
| #4 | Opened 文档更新 — 代码已实现，架构文档仍标 TODO | 🟢 低 | ✅ |
| #5 | prevent_close 验证 — OHOS 上未验证是否真正阻止窗口关闭 | 🟡 中 | ✅ |
| — | Resumed/Suspended 跨平台遗留 — Tauri 全平台未适配 Tao 生命周期事件演进 | 🚫 推迟 | ❌ 不在本次解决 |

## Phase 列表

| Phase | 名称 | openspec change | 状态 | 涉及层 | 预估文件 | 验证方式 |
|-------|------|----------------|------|--------|---------|---------|
| 1 | ExitRequested/Exit 退出流程 | p1-runevent-exit-flow | ✓ 已归档 | tauri-runtime-wry | ~1 | 设备端测试 |
| 2 | 子窗口 Destroyed + WindowsStore 清理 | p2-runevent-window-lifecycle | ✓ 已归档 | tauri-runtime-wry | ~1 | 设备端测试 |
| 3 | prevent_close 验证 + 文档更新 | p3-runevent-verify-docs | ✓ 已归档 | tauri-runtime-wry + doc | ~2 | 设备端测试 |

## Phase 详细说明

### Phase 1: ExitRequested/Exit 退出流程

- **目标**：在 `Event::LoopDestroyed` handler 中先发送 `RunEvent::ExitRequested`，再发送 `RunEvent::Exit`，使用 `ExitState(AtomicBool)` 防止重复触发
- **文件列表**：
  - `crates/tauri-runtime-wry/src/lib.rs` — 新增 `ExitState` 结构体，修改 `LoopDestroyed` handler，在 `TaoWindowEvent::Destroyed` 中设置标志
- **设计要点**：
  - 不依赖 `onPrepareToTerminate`（需系统参数 `persist.sys.prepare_terminate`，非默认开启）
  - 不新增 tao 事件变体（减少跨仓库变更）
  - `ExitRequested` 使用 `code: None`（系统关闭语义，与窗口关闭路径一致）
  - `prevent_exit()` 在 `LoopDestroyed` 路径上可能无法真正阻止退出（系统已开始销毁），但用户代码至少能执行清理
  - `onPrepareToTerminate` 作为后续增强方向（需验证返回值语义后实现）
- **依赖**：无

### Phase 2: 子窗口 Destroyed + WindowsStore 清理

- **目标**：修复子窗口关闭时 Destroyed 事件缺失，完善 WindowsStore 条目清理
- **文件列表**：
  - `crates/tauri-runtime-wry/src/lib.rs` — `on_window_close` 函数改为移除条目；WindowMessage::Destroy 处理流程补充 Destroyed 事件
  - `crates/tauri-runtime-wry/src/window.rs` — 可能需要调整窗口关闭逻辑
  - `tao/src/platform_impl/ohos/mod.rs` — 确认子窗口事件路径
- **依赖**：Phase 1 完成（退出流程依赖 WindowsStore 正确清理）

### Phase 3: prevent_close 验证 + 文档更新

- **目标**：验证 prevent_close() 在 OHOS 上是否生效；同步更新架构文档
- **文件列表**：
  - `examples/api/src-tauri/src/lib.rs` — 调整 CloseRequested 测试代码验证 prevent_close
  - `doc/RunEvent_Architecture.md` — 更新 Opened 状态、Resumed 状态、修复结论
- **依赖**：Phase 1、Phase 2 完成

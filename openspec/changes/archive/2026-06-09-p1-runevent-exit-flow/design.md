## Context

Tauri 的 `RunEvent::ExitRequested` 允许用户代码在应用退出前拦截（如保存数据、显示确认对话框）。在 Windows/macOS 上，此功能通过 `TaoWindowEvent::Destroyed`（最后一个窗口关闭时）和 `Message::RequestExit`（`AppHandle::exit()` 调用时）两条路径正确触发。

在 OHOS 平台上，当系统销毁 UIAbility 时（`MainEvent::Destroy`），tao 发送 `Event::LoopDestroyed`，tauri-runtime-wry 直接发送 `RunEvent::Exit` 跳过 `ExitRequested`（`lib.rs:4297-4302` 的 TODO 注释）。

**约束条件**：
- NAPI 回调需遵守 napi-derive-ohos 自动 camelCase 转换规则
- 线程模型：Chrome_IOThread 是 event loop，禁止 `run_on_main_thread + recv()` 阻塞模式
- OHOS 的 `onPrepareToTerminate` 回调依赖系统参数 `persist.sys.prepare_terminate = true`（非默认开启），返回值语义存在文档矛盾

## Goals / Non-Goals

**Goals:**
- 在 OHOS 上实现 `RunEvent::ExitRequested` 的正确触发，用户可通过 `api.prevent_exit()` 拦截退出
- **不依赖任何系统参数**，确保默认设备上即可工作
- 与 Windows/macOS 的 `ExitRequested` 行为保持一致（同步 `try_recv` 模式）

**Non-Goals:**
- 不修复 `Resumed`/`Suspended` 跨平台遗留问题（设计文档明确推迟）
- 不实现 `SaveState` / `Start` / `Stop` 等其他生命周期事件的映射
- 不修改 Windows/macOS 上的退出流程
- 不依赖 `onPrepareToTerminate` 系统参数（作为可选增强，非核心路径）

## Decisions

### Decision 1: 在 `LoopDestroyed` handler 中直接触发 `ExitRequested`（不依赖 `onPrepareToTerminate`）

**选择**: 修改 tauri-runtime-wry 的 `Event::LoopDestroyed` handler，先发送 `RunEvent::ExitRequested`，再发送 `RunEvent::Exit`

**替代方案**:
- **依赖 `onPrepareToTerminate`**: 被排除。该回调需要 `persist.sys.prepare_terminate = true` 系统参数（非默认），且返回值语义存在文档矛盾（两次 arkts-helper 查询返回相反语义）。将此作为核心路径会导致默认设备上功能完全不可用
- **在 `onDestroy` 中触发**: 被排除，`onDestroy` 无法取消终止

**理由**:
- `LoopDestroyed` 在所有 OHOS 设备上都会触发（不依赖系统参数）
- 虽然此时系统已开始销毁流程，`prevent_exit()` 可能无法阻止最终退出，但用户代码至少能收到 `ExitRequested` 事件，执行清理逻辑（保存数据、发送日志等）
- 与 Windows/macOS 的 `ExitRequested` 行为语义一致：通知用户"即将退出"

**代码变更**（`crates/tauri-runtime-wry/src/lib.rs:4297-4302`）：
```rust
Event::LoopDestroyed => {
  // 先触发 ExitRequested，给用户代码清理机会
  let (tx, rx) = channel();
  callback(RunEvent::ExitRequested { code: None, tx });  // None = 系统关闭（非用户请求）
  let _recv = rx.try_recv();
  // OHOS 上 LoopDestroyed 时系统已开始销毁，prevent_exit 无法阻止
  // 但仍然发送 ExitRequested 让用户代码有机会执行清理
  callback(RunEvent::Exit);
}
```

**`code` 参数选择 `None` 的理由**：
- 与现有的"最后一个窗口关闭"路径一致（`TaoWindowEvent::Destroyed` 使用 `code: None`）
- 语义上更接近"应用被系统关闭"而非"用户调用 `app.exit(code)`"
- `Some(code)` 保留给 `Message::RequestExit(code)` 路径（用户主动请求退出）

### Decision 2: `onPrepareToTerminate` 作为可选增强（Phase 1 范围外）

**选择**: `onPrepareToTerminate` 不在 Phase 1 核心实现中。记录为后续增强方向。

**理由**:
- 依赖 `persist.sys.prepare_terminate` 系统参数，非默认开启
- 返回值语义存在文档矛盾（`true = 取消` vs `true = 可终止`），需设备验证
- 作为增强功能时，需要添加版本守卫（API 12+）和设备验证

**后续增强方向**（Phase 1 之后）:
1. 在设备上验证 `onPrepareToTerminate` 的返回值语义
2. 在构建脚本中添加 `persist.sys.prepare_terminate` 检查和设置
3. 实现 `onPrepareToTerminate` → `ExitRequested` 路径（可真正阻止退出）
4. 使用 `AtomicBool` 防止 `ExitRequested` 重复触发

### Decision 3: 不新增 tao 事件变体

**选择**: 复用现有的 `MainEvent::Destroy` → `Event::LoopDestroyed` 路径，不新增 tao 事件

**理由**:
- 核心修复只需修改 tauri-runtime-wry 的 `LoopDestroyed` handler
- 不需要 tao 层传递额外信息
- 减少跨仓库变更（tao 是独立仓库）
- 如果后续实现 `onPrepareToTerminate` 增强，再考虑新增 tao 事件

### Decision 4: 保持 Windows/macOS 行为不变

**选择**: 修改仅在 `cfg(target_env = "ohos")` 条件下生效

**理由**:
- Windows/macOS 的 `ExitRequested` 已正常工作
- OHOS 的 `LoopDestroyed` handler 修改不影响其他平台（`LoopDestroyed` 在桌面平台上很少触发）
- 使用 `cfg` 隔离确保无回归风险

### Decision 5: `ExitState` 结构体存储防重复标志

**选择**: 创建新的 `ExitState` 结构体，与现有的 `WindowsStore`、`WindowIdStore` 等并列，通过 `Arc` 共享

**实现方案**：
```rust
// 新增结构体（与 WindowsStore 等并列）
pub struct ExitState(pub AtomicBool);

// 添加到 EventLoopIterationContext
pub struct EventLoopIterationContext<'a, T: UserEvent> {
  pub callback: &'a mut (dyn FnMut(RunEvent<T>) + 'static),
  pub window_id_map: WindowIdStore,
  pub windows: Arc<WindowsStore>,
  pub exit_state: Arc<ExitState>,  // 新增
  #[cfg(feature = "tracing")]
  pub active_tracing_spans: ActiveTraceSpanStore,
}

// 在事件循环创建时初始化
let exit_state = Arc::new(ExitState(AtomicBool::new(false)));

// 在 TaoWindowEvent::Destroyed 路径中设置
exit_state.0.store(true, Ordering::SeqCst);

// 在 Event::LoopDestroyed 路径中检查
if !exit_state.0.load(Ordering::SeqCst) {
  // 发送 ExitRequested
}
```

**理由**：
- 遵循现有的状态管理模式（`Arc<SharedState>` + 内部可变性）
- `ExitState` 职责单一，仅用于跟踪 `ExitRequested` 是否已发送
- 与 `WindowsStore`、`WindowIdStore` 等结构一致，易于理解和维护
- 避免了添加到 `WindowsStore`（职责不匹配）或使用全局静态变量（不利于多实例）的问题

**存放位置**：
- 结构体定义：`crates/tauri-runtime-wry/src/lib.rs`（与 `WindowsStore` 相邻）
- 初始化：事件循环创建处（`make_event_handler` 或 `run` 方法）
- 传递：通过 `EventLoopIterationContext` 传递给 `handle_event_loop`

## Risks / Trade-offs

### Risk 1: `prevent_exit()` 在 `LoopDestroyed` 路径上可能无法真正阻止退出

**风险**: `LoopDestroyed` 时系统已开始销毁 UIAbility，即使 `prevent_exit()` 被调用，`ControlFlow::Exit` 的设置可能已无法阻止进程终止。

**缓解**:
- 这仍然优于当前实现（完全不触发 `ExitRequested`）
- 用户代码至少能收到事件并执行清理逻辑
- 后续通过 `onPrepareToTerminate` 增强可实现真正的阻止退出（见 Decision 2）

### Risk 2: `ExitRequested` 可能在 `LoopDestroyed` 路径上被多次触发

**风险**: 如果 `LoopDestroyed` 前已经有窗口关闭触发了 `ExitRequested`（路径 A：最后一个窗口 Destroyed），`LoopDestroyed` 会再次触发。

**缓解**（详见 Decision 5）:
- 创建 `ExitState(pub AtomicBool)` 结构体，通过 `Arc` 共享
- 路径 A（`TaoWindowEvent::Destroyed`）触发 `ExitRequested` 时设 `exit_state.0.store(true, Ordering::SeqCst)`
- `LoopDestroyed` 检查 `exit_state.0.load(Ordering::SeqCst)`，仅在 `false` 时发送 `ExitRequested`
- 遵循现有状态管理模式（与 `WindowsStore`、`WindowIdStore` 一致）

### Trade-off: 放弃 `onPrepareToTerminate` 作为核心路径

**权衡**: 放弃 `onPrepareToTerminate` 意味着 Phase 1 的 `prevent_exit()` 可能无法真正阻止退出（仅能通知用户代码）。但换取了：
- 不依赖系统参数，所有设备默认可用
- 不依赖有矛盾的 API 文档
- 不需要 openharmony-ability / tao 的变更（减少跨仓库工作量）
- 可在后续 Phase 中作为增强添加

## Audit Findings (2026-06-08)

### ✅ 设计简化后的确认点

- `LoopDestroyed` 在所有 OHOS 设备上都会触发（不依赖系统参数）
- 修改仅在 `cfg(target_env = "ohos")` 下生效，不影响 Windows/macOS
- `try_recv()` channel 模式与 Windows/macOS 的 `ExitRequested` 路径一致
- `RESTART_EXIT_CODE` 旁路在 `ExitRequestApi` 层面已处理，OHOS 路径自动继承
- 不需要修改 openharmony-ability 或 tao（Phase 1 仅修改 tauri-runtime-wry）

### ℹ️ `onPrepareToTerminate` 后续增强待验证

- API 12 有同步版本 `onPrepareToTerminate`，API 15 有异步版本 `onPrepareToTerminateAsync`
- 返回值语义存在矛盾（`true = 取消` vs `true = 可终止`），需设备验证
- 需要 `persist.sys.prepare_terminate = true` 系统参数
- 这些不影响 Phase 1 核心实现

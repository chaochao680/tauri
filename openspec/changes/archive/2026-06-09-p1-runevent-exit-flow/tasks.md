## 1. tauri-runtime-wry: ExitRequested 在 LoopDestroyed 路径触发

- [x] 1.1 创建 `ExitState` 结构体：`pub struct ExitState(pub AtomicBool);`，放在 `crates/tauri-runtime-wry/src/lib.rs` 中（与 `WindowsStore` 相邻）
- [x] 1.2 将 `exit_state: Arc<ExitState>` 添加到 `EventLoopIterationContext` 结构体
- [x] 1.3 在事件循环创建处初始化 `ExitState`：`let exit_state = Arc::new(ExitState(AtomicBool::new(false)));`，并通过 `Arc::clone` 传递给事件处理器
- [x] 1.4 在现有 `TaoWindowEvent::Destroyed` handler（路径 A，约 `lib.rs:4426-4443`）中，触发 `ExitRequested` 后设置 `exit_state.0.store(true, Ordering::SeqCst)`
- [x] 1.5 修改 `Event::LoopDestroyed` handler（`cfg(target_env = "ohos")` 条件下）：检查 `exit_state.0.load(Ordering::SeqCst)`
- [x] 1.6 如果标志为 `false`：创建 channel，发送 `RunEvent::ExitRequested { code: None, tx }`（`None` = 系统关闭），调用 `rx.try_recv()`，然后发送 `RunEvent::Exit`
- [x] 1.7 如果标志为 `true`：直接发送 `RunEvent::Exit`（跳过 ExitRequested）
- [x] 1.8 移除 `LoopDestroyed` handler 中的 TODO 注释（`lib.rs:4299-4301`）
- [x] 1.9 添加必要的 `use` 声明：`use std::sync::atomic::{AtomicBool, Ordering};`

## 2. 测试与验证

- [x] 2.1 在 `examples/api/src-tauri/src/lib.rs` 中添加 `ExitRequested` 测试日志（记录 code、是否 prevent、来源路径）
- [x] 2.2 `cargo check --target aarch64-linux-ohos` 编译验证
- [x] 2.3 构建 OHOS HAP 并部署到设备
- [x] 2.4 验证：关闭应用 → 日志出现 `[RunEvent] ExitRequested` → 然后 `[RunEvent] Exit`
- [x] 2.5 验证：`prevent_exit()` 被调用时日志记录正确，但应用仍退出（LoopDestroyed 路径）

## 3. 文档更新

- [x] 3.1 更新 `doc/RunEvent_Architecture.md`：ExitRequested/Exit 状态从 ⚠️ 改为 ✅（LoopDestroyed 路径已修复）
- [x] 3.2 记录 `onPrepareToTerminate` 作为后续增强方向（依赖系统参数 + 返回值语义待验证）

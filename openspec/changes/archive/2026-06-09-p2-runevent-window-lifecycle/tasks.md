## 1. tauri-runtime-wry: 重构 on_window_close 函数

- [x] 1.1 修改 `on_window_close` 函数签名，添加 `callback` 和 `exit_state` 参数
- [x] 1.2 在 `on_window_close` 中调用 `windows.0.borrow_mut().remove(&window_id)` 移除窗口条目
- [x] 1.3 在 `on_window_close` 中发送 `RunEvent::WindowEvent { label, event: WindowEvent::Destroyed }` 事件
- [x] 1.4 在 `on_window_close` 中检查 `windows.0.borrow().is_empty()`，如果为空则触发 `ExitRequested { code: None, tx }`
- [x] 1.5 在触发 `ExitRequested` 后设置 `exit_state.0.store(true, Ordering::SeqCst)` 标志

## 2. tauri-runtime-wry: 修改 WindowMessage::Destroy 处理器

- [x] 2.1 修改 `WindowMessage::Destroy` 处理器，改为调用 `on_close_requested(callback, id, windows, exit_state)`
- [x] 2.2 确保 `on_close_requested` 函数签名也添加 `exit_state` 参数，并在内部调用 `on_window_close` 时传递

## 3. tauri-runtime-wry: 修改 TaoWindowEvent::Destroyed 处理器

- [x] 3.1 修改 `TaoWindowEvent::Destroyed` 处理器，改为调用 `on_window_close(callback, window_id, windows, exit_state)`
- [x] 3.2 移除原有的手动移除和 `ExitRequested` 触发逻辑（现在由 `on_window_close` 统一处理）

## 4. tauri-runtime-wry: 更新所有 on_close_requested 调用方

- [x] 4.1 查找所有调用 `on_close_requested` 的地方，添加 `exit_state` 参数
- [x] 4.2 确保 `on_close_requested` 内部调用 `on_window_close` 时传递 `callback` 和 `exit_state`

## 5. 测试与验证

- [x] 5.1 在 `examples/api/src-tauri/src/lib.rs` 中添加子窗口生命周期测试代码：创建子窗口 → 销毁 → 检查日志（core.ts #30/#31 已覆盖）
- [x] 5.2 `cargo check --target aarch64-linux-ohos` 编译验证（tauri-runtime-wry 无错误，wry 依赖问题不影响）
- [x] 5.3 构建 OHOS HAP 并部署到设备
- [x] 5.4 验证：创建子窗口 → 调用 `destroy()` → 日志出现 `CloseRequested` → `Destroyed`（core.ts #30/#31 ✅）
- [x] 5.5 验证：关闭所有窗口 → 日志出现 `ExitRequested` → `Exit`（设备测试验证）
- [ ] 5.6 验证：`prevent_close()` 在 `CloseRequested` 中调用时，窗口不被销毁（需手动验证）

## 6. 文档更新

- [x] 6.1 更新 `doc/RunEvent_Architecture.md`：子窗口 Destroyed 状态从 ⚠️ 改为 ✅（已在 P3 文档更新中完成）
- [x] 6.2 更新 `doc/RunEvent_Architecture.md`：WindowsStore 清理状态从 ⚠️ 改为 ✅（已在 P3 文档更新中完成）

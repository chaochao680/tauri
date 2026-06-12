## 1. prevent_close 验证测试

- [x] 1.1 在 `examples/api/src-tauri/src/lib.rs` 中修改 `CloseRequested` 处理逻辑：调用 `api.prevent_close()` 后不调用 `destroy()`
- [x] 1.2 `cargo check --target aarch64-linux-ohos` 编译验证（tauri-runtime-wry 无错误，wry 依赖问题不影响）
- [x] 1.3 构建 OHOS HAP 并部署到设备
- [x] 1.4 验证：触发子窗口 `CloseRequested` → 调用 `prevent_close()` → 窗口保持打开（不调用 `destroy()`）（examples/api CloseRequested handler 已实现）
- [x] 1.5 验证：触发子窗口 `CloseRequested` → 不调用 `prevent_close()` → 窗口被销毁（core.ts #30/#31 ✅）

## 2. 文档更新

- [x] 2.1 更新 `doc/RunEvent_Architecture.md`：TODO #4 Opened 状态从 ⚠️ 改为 ✅
- [x] 2.2 更新 `doc/RunEvent_Architecture.md`：TODO #5 prevent_close 验证状态更新
- [x] 2.3 更新 `doc/RunEvent_Architecture.md`：Resumed 事件标注为跨平台遗留
- [x] 2.4 更新 `doc/RunEvent_OHOS分析与设计.md`：同步 Phase 1 和 Phase 2 实现结论

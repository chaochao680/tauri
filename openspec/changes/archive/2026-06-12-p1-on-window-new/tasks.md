## 1. openharmony-ability: Rust NAPI 层

- [ ] 1.1 在 `crates/ability/src/helper/webview.rs` 新增 `OnWindowNewResult` struct（`#[napi(object)]`，字段 `allow: bool`）
- [ ] 1.2 在 `WebViewInitData` struct 新增 `on_window_new: Option<Function<'a, (String, bool, bool), OnWindowNewResult>>` 字段
- [ ] 1.3 在 `crates/ability/src/webview/mod.rs` 的 `WebViewBuilder` struct 新增 `on_window_new: Option<Box<dyn Fn(String, bool, bool) -> bool>>` 字段
- [ ] 1.4 为 `WebViewBuilder` 添加 `pub fn on_window_new<F>(mut self, handler: F) -> Self` builder 方法
- [ ] 1.5 在 `build()` 方法中创建 NAPI Function closure（`env.create_function_from_closure("on_window_new", ...)`），将参数传递给 handler 闭包，返回 `OnWindowNewResult`
- [ ] 1.6 在 `build()` 的 `create_webview_func.call(WebViewInitData { ... })` 中传入 `on_window_new` 字段

## 2. openharmony-ability: ArkTS 层

- [ ] 2.1 在 `native_ability/src/main/ets/ability/type.ets` 新增 `OnWindowNewResult` interface 和 `WebViewInitData.onWindowNew` 字段
- [ ] 2.2 在 `native_ability/src/main/ets/webview/DefaultWebview.ets` 的 `WebviewInitData` → `WebviewNodeData` 转换中传递 `onWindowNew` 回调
- [ ] 2.3 在 `WebBuilder` @Builder 的 `Web({...})` 链中添加 `.multiWindowAccess(true)` 和 `.allowWindowOpenMethod(true)`
- [ ] 2.4 在 `WebBuilder` 中添加 `.onWindowNew()` 事件处理：调用 `nodeData.onWindowNew` NAPI 回调，根据返回的 `allow` 决定 `setWebController(newCtrl)` 或 `setWebController(null)`
- [ ] 2.5 添加 try-catch 保护：NAPI 回调异常时调用 `setWebController(null)` 防止渲染进程阻塞
- [ ] 2.6 在 `EmbeddedWebBuilder` 中同步添加相同的 `multiWindowAccess` + `onWindowNew` 逻辑
- [ ] 2.7 新建 `native_ability/src/main/ets/webview/NewWindowDialog.ets`：`@CustomDialog` 组件嵌入 Web 组件 + `NewWindowDialogManager` 管理类
- [ ] 2.8 验证 `@CustomDialog` 在 @Builder 上下文中可用；如不可用，改为 `promptAction.openCustomDialog()` 方案
- [ ] 2.9 同步 `package/` 目录下的对应文件（`type.ets`、`DefaultWebview.ets`、`NewWindowDialog.ets`）

## 3. wry OHOS 桥接

- [ ] 3.1 在 `wry/src/ohos/mod.rs` 移除 `let _ = new_window_req_handler` 抑制
- [ ] 3.2 添加 `if let Some(new_window_req_handler) = new_window_req_handler` 分支，将 handler 桥接到 `WebViewBuilder.on_window_new()`
- [ ] 3.3 在桥接闭包中：解析 `target_url` 为 `Url`，构造空的 `NewWindowFeatures`，调用 `new_window_req_handler`，将 `NewWindowResponse` 转换为 `bool`
- [ ] 3.4 处理 `NewWindowResponse::Create` 降级为 `Allow`（返回 `true`）
- [ ] 3.5 检查 `wry::NewWindowOpener` 在 OHOS 上的定义，如需要添加空 struct 或 cfg 排除
- [ ] 3.6 验证 `cargo check --target aarch64-unknown-linux-ohos -p wry` 编译通过

## 4. tauri-runtime-wry + tauri 适配

- [ ] 4.1 检查 `tauri-runtime-wry/src/lib.rs:5113-5170` 中 OHOS 路径的 `NewWindowOpener` 字段处理是否兼容
- [ ] 4.2 确认 `NewWindowFeatures::new(None, None, opener)` 在 OHOS 上能正确构造
- [ ] 4.3 更新 `tauri/crates/tauri/src/webview/mod.rs` 中 OHOS `NewWindowResponse` 的文档注释，说明 Allow 行为（dialog 弹窗）和 Create 不支持的限制
- [ ] 4.4 验证 `cargo check --target aarch64-unknown-linux-ohos -p tauri-runtime-wry` 编译通过
- [ ] 4.5 验证 `cargo check --target aarch64-unknown-linux-ohos -p tauri` 编译通过

## 5. HAR 包重建与编译验证

- [ ] 5.1 执行 `ohrs build --arch arm64` 重建 openharmony-ability HAR 包
- [ ] 5.2 执行 `pack.sh` + `tar -czf ability.har package` 打包
- [ ] 5.3 从 `gen/ohos` 目录执行 `ohpm install` 安装新 HAR
- [ ] 5.4 执行 `bash .claude/skills/ohos-build/scripts/build-ohos.sh` 构建 HAP 确认全链编译通过

## 6. 示例应用与前端测试

- [ ] 6.1 更新 `examples/api/src-tauri/src/lib.rs` 中的 `on_new_window` 示例：OHOS 上打印 URL 并返回 `Deny`
- [ ] 6.2 在 `examples/api` 前端测试页面添加 on_new_window 测试按钮（Deny 模式 + Allow 模式）
- [ ] 6.3 编写 auto 测试：验证 `window.open()` 被 Deny 时无新窗口
- [ ] 6.4 编写 side-effect 测试：验证 Allow 时 dialog 弹出且 targetUrl 正确

## 7. 设备端验证

- [ ] 7.1 签名并安装 HAP 到设备（先 `bm uninstall` 再安装）
- [ ] 7.2 运行自动测试脚本，确认 Deny 测试 PASS
- [ ] 7.3 手动验证：点击 Allow 测试按钮 → dialog 弹出 → Web 组件加载目标 URL → 可交互
- [ ] 7.4 手动验证：关闭 dialog → 原页面功能正常
- [ ] 7.5 手动验证：未设置 handler 时，`window.open()` 默认被阻止（无新窗口）

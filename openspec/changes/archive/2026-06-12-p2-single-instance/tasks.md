## 1. Cargo.toml 依赖配置

- [x] 1.1 在 `plugins-workspace/plugins/single-instance/Cargo.toml` 中添加 `[target.'cfg(target_env = "ohos")'.dependencies]` 段落，声明 `openharmony-ability`（path 引用）
- [x] 1.2 在 `[package.metadata.platforms.support]` 中添加 `ohos = { level = "full", notes = "Uses OS singleton + onNewWant" }`

## 2. lib.rs cfg gate

- [x] 2.1 在 `plugins-workspace/plugins/single-instance/src/lib.rs` 中添加 `#[cfg(target_env = "ohos")] #[path = "platform_impl/ohos.rs"] mod platform_impl;`，同时修正 Linux cfg 为 `all(target_os = "linux", not(target_env = "ohos"))`

## 3. platform_impl/ohos.rs 实现

- [x] 3.1 创建 `plugins-workspace/plugins/single-instance/src/platform_impl/ohos.rs`，实现 `pub fn init<R: Runtime>(cb: Box<SingleInstanceCallback<R>>) -> TauriPlugin<R>`
- [x] 3.2 在 `init()` 中使用 `plugin::Builder::new("single-instance").setup(|_, _| Ok(())).on_event(...)` 模式
- [x] 3.3 在 `on_event` 中匹配 `RunEvent::Opened { urls }`，调用 `openharmony_ability::take_want_parameters()` 获取参数
- [x] 3.4 构建 `args = [uri, parameters_json]`（参数为空时仅 `[uri]`），`cwd = ""`，调用 `cb(app.app_handle(), args, cwd)`
- [x] 3.5 在 `on_event` 中处理 `RunEvent::Exit` 调用 `destroy(app)`
- [x] 3.6 实现 `pub fn destroy<R: Runtime, M: Manager<R>>(_manager: &M)` 为空操作

## 4. 编译验证

- [x] 4.1 Host target `cargo check` 通过（确保 cfg 隔离不影响其他平台）

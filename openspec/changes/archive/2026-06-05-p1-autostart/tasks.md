> 注：§1-§5 任务在 openharmony-ability#15 / plugins-workspace#9 仓库各自 PR 中完成，
> 本仓库 PR 仅包含 §6 前端测试任务。

## 1. ArkTS 实现（openharmony-ability）

- [x] 1.1 创建 `native_ability/src/main/ets/helper/autostart.ets`，实现 `autostartEnable()`、`autostartDisable()`、`autostartIsEnabled()` 三个函数
- [x] 1.2 在 `autostartEnable()` / `autostartDisable()` 中使用 `startAbility` 跳转到 `pc_app_setup_settings` 设置页面
- [x] 1.3 在 `autostartIsEnabled()` 中使用 `autoStartupManager.getAutoStartupStatusForSelf()`，添加 try-catch 防御性处理 Error 801（返回 `false`）
- [x] 1.4 在 `native_ability/src/main/ets/helper/index.ets` 中导出 autostart 函数，注册到 helper 对象

## 2. Rust TSFN 基础设施（openharmony-ability）

- [x] 2.1 创建 `crates/ability/src/helper/autostart.rs`，定义 `AUTOSTART_ENABLE_TSFN`、`AUTOSTART_DISABLE_TSFN`、`AUTOSTART_IS_ENABLED_TSFN` 三个 TSFN
- [x] 2.2 实现 `create_autostart_enable_tsfn()`、`create_autostart_disable_tsfn()`、`create_autostart_is_enabled_tsfn()` 创建函数，使用 `callee_handled::<false>()`
- [x] 2.3 实现 `get_autostart_enable_tsfn()`、`get_autostart_disable_tsfn()`、`get_autostart_is_enabled_tsfn()` 获取函数
- [x] 2.4 在 `crates/ability/src/helper/mod.rs` 中注册 `mod autostart` 并 `pub use autostart::*`

## 3. Rust API（openharmony-ability）

- [x] 3.1 创建 `crates/ability/src/autostart.rs`，实现 `AutostartManager` struct
- [x] 3.2 实现 `AutostartManager::enable()` async 方法，通过 TSFN 调用 ArkTS `autostartEnable()`
- [x] 3.3 实现 `AutostartManager::disable()` async 方法，通过 TSFN 调用 ArkTS `autostartDisable()`
- [x] 3.4 实现 `AutostartManager::is_enabled()` async 方法，在 Rust 侧添加 `version::sdk_api_version() >= 21` 版本守卫（遵循 ohos-version-api spec），低版本直接返回 `Ok(false)` 不发起 TSFN 调用；API 21+ 时通过 TSFN 调用 ArkTS `autostartIsEnabled()`，使用 `oneshot::channel` 桥接 Promise 到 Future
- [x] 3.5 在 `crates/ability/src/lib.rs` 中注册 `mod autostart` 并 `pub use autostart::*`

## 4. TSFN 初始化注册（openharmony-ability）

- [x] 4.1 在 TSFN 初始化代码中（render/xcomponent.rs 或 ability init 位置）添加 `create_autostart_enable_tsfn()`、`create_autostart_disable_tsfn()`、`create_autostart_is_enabled_tsfn()` 调用

## 5. plugin-autostart 适配（plugins-workspace）

- [x] 5.1 修改 `plugins/autostart/Cargo.toml`，为 OHOS 添加 `openharmony-ability` 依赖（`cfg(target_env = "ohos")`）
- [x] 5.2 修改 `plugins/autostart/src/lib.rs`，为 `AutoLaunchManager` 添加 OHOS 条件编译分支，使用 `openharmony_ability::autostart::AutostartManager` 替代 `auto_launch::AutoLaunch`
- [x] 5.3 实现 OHOS 版本的 `enable()`、`disable()`、`is_enabled()` 方法，委托给 `AutostartManager`
- [x] 5.4 修改 `setup()` 函数中的 OHOS 分支，使用 `AutostartManager` 初始化（不需要 app_name / app_path 参数）
- [x] 5.5 确保 `auto-launch` 依赖在 OHOS 上被排除（`cfg(not(target_env = "ohos"))`）

## 6. 前端测试

- [x] 6.1 在 `core.ts` 或 `plugins.ts` 中添加 autostart 测试用例：auto 类（`isEnabled()` 返回 boolean）
- [x] 6.2 添加 side-effect 测试：`enable()` / `disable()` 不抛错
- [x] 6.3 添加 manual 测试：验证 `enable()` 打开系统设置页面，autostart 开关可见

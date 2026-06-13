## ADDED Requirements

### Requirement: OHOS platform_impl 提供 init 和 destroy 函数
`platform_impl/ohos.rs` SHALL 提供 `pub fn init<R: Runtime>(cb: Box<SingleInstanceCallback<R>>) -> TauriPlugin<R>` 和 `pub fn destroy<R: Runtime, M: Manager<R>>(manager: &M)` 函数，接口与其他平台一致。

#### Scenario: init 返回有效的 TauriPlugin
- **WHEN** 调用 `init(callback)` 
- **THEN** SHALL 返回一个 `TauriPlugin<R>` 实例，名称为 "single-instance"

#### Scenario: destroy 为空操作
- **WHEN** 调用 `destroy(manager)`
- **THEN** SHALL 不执行任何操作（OHOS 无 socket/mutex 需要清理）

### Requirement: 再次启动时触发用户 callback
当 OHOS 系统调用 `onNewWant` 导致 `RunEvent::Opened` 事件触发时，插件 SHALL 调用用户注册的 callback，传递启动参数。

#### Scenario: 再次启动携带参数
- **WHEN** 应用已在运行，用户再次启动应用，`want.uri` 为 `"tauri://test"` 且 `want.parameters` 为 `{"source":"widget"}`
- **THEN** callback SHALL 被调用，`args` = `["tauri://test", "{\"source\":\"widget\"}"]`，`cwd` = `""`

#### Scenario: 再次启动无参数
- **WHEN** 应用已在运行，用户再次启动应用，`want.uri` 为空且 `want.parameters` 为空
- **THEN** callback SHALL 被调用，`args` = `[""]`，`cwd` = `""`

#### Scenario: 首次启动不触发 callback
- **WHEN** 应用首次启动（走 `onCreate` 路径）
- **THEN** callback SHALL 不被调用（`RunEvent::Opened` 不会在首次启动时触发）

### Requirement: cfg 隔离正确
OHOS 平台代码 SHALL 使用 `cfg(target_env = "ohos")` 隔离。`lib.rs` SHALL 添加 `#[cfg(target_env = "ohos")]` 指向 `ohos.rs`。`Cargo.toml` SHALL 将 `openharmony-ability` 声明为 OHOS target 依赖。

#### Scenario: OHOS target 编译通过
- **WHEN** 使用 OHOS target 编译 single-instance crate
- **THEN** SHALL 编译成功，`platform_impl` 指向 `ohos.rs`

#### Scenario: 非 OHOS target 不受影响
- **WHEN** 使用 Windows/macOS/Linux target 编译
- **THEN** `ohos.rs` SHALL 不参与编译，原有平台实现不受影响

### Requirement: 使用 openharmony_ability 读取参数
OHOS platform_impl SHALL 通过 `openharmony_ability::take_want_parameters()` 获取存储的 `want.parameters` JSON 字符串。

#### Scenario: 参数读取使用 take 语义
- **WHEN** `RunEvent::Opened` 触发且 `take_want_parameters()` 返回 `"{\"key\":\"val\"}"`
- **THEN** 后续再次调用 `take_want_parameters()` SHALL 返回空字符串（已被消费）

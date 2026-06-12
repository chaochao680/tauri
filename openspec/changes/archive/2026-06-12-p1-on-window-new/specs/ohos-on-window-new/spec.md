## ADDED Requirements

### Requirement: ArkTS Web 组件启用多窗口拦截
`openharmony-ability` 的 `DefaultWebview.ets` 中的 Web 组件 SHALL 启用 `multiWindowAccess(true)` 和 `allowWindowOpenMethod(true)`，以允许 `onWindowNew` 事件触发。

#### Scenario: Web 组件配置了 multiWindowAccess
- **WHEN** WebViewBuilder 创建新的 Web 组件
- **THEN** 生成的 ArkTS Web 组件链中包含 `.multiWindowAccess(true)` 和 `.allowWindowOpenMethod(true)`

#### Scenario: JS window.open() 触发 onWindowNew 事件
- **WHEN** 前端页面执行 `window.open('https://example.com')`
- **THEN** ArkTS 侧 `onWindowNew` 回调被触发，`event.targetUrl` 包含 `'https://example.com'`

### Requirement: NAPI 回调接口 on_window_new
`openharmony-ability` 的 `WebViewInitData` NAPI struct SHALL 包含 `on_window_new` 可选字段，类型为 `Function<(String, bool, bool), OnWindowNewResult>`。ArkTS 侧通过自动 camelCase 转换为 `onWindowNew`。

#### Scenario: Rust 注册 on_window_new 回调
- **WHEN** `WebViewBuilder.on_window_new(handler)` 被调用并 `build()`
- **THEN** 生成的 `WebViewInitData` 包含 `on_window_new` NAPI Function，参数为 `(targetUrl: string, isAlert: boolean, isUserTrigger: boolean)`

#### Scenario: NAPI 回调返回 OnWindowNewResult
- **WHEN** ArkTS 调用 `onWindowNew(targetUrl, isAlert, isUserTrigger)` NAPI 回调
- **THEN** 返回 `OnWindowNewResult { allow: boolean }` 对象

### Requirement: Deny 阻止新窗口
当 Rust handler 返回 `false`（或 `NewWindowResponse::Deny`）时，系统 SHALL 调用 `event.handler.setWebController(null)` 阻止新窗口打开。

#### Scenario: handler 返回 Deny
- **WHEN** `on_new_window` handler 返回 `NewWindowResponse::Deny`
- **THEN** ArkTS 侧调用 `event.handler.setWebController(null)`
- **THEN** 无新窗口/dialog 出现
- **THEN** 渲染进程正常继续运行（不阻塞）

#### Scenario: 未注册 handler 时默认 Deny
- **WHEN** 应用未调用 `on_new_window()` 设置 handler
- **WHEN** 前端触发 `window.open()`
- **THEN** ArkTS 侧调用 `event.handler.setWebController(null)`（安全默认值）
- **THEN** 无新窗口出现

### Requirement: Allow 以 dialog 形式打开新窗口
当 Rust handler 返回 `true`（或 `NewWindowResponse::Allow`）时，系统 SHALL 创建新的 `WebviewController`，在 `@CustomDialog` 中嵌入 Web 组件，并调用 `event.handler.setWebController(newCtrl)` 允许新窗口。

#### Scenario: handler 返回 Allow
- **WHEN** `on_new_window` handler 返回 `NewWindowResponse::Allow`
- **THEN** ArkTS 创建新的 `WebviewController`
- **THEN** 弹出 `@CustomDialog`（或等价弹窗）包含 Web 组件
- **THEN** 调用 `event.handler.setWebController(newCtrl)` 传入新 controller
- **THEN** 新窗口的 Web 组件加载 `event.targetUrl`

#### Scenario: dialog 中的 Web 组件禁用多窗口
- **WHEN** Allow 创建的 dialog 内的 Web 组件
- **THEN** 该 Web 组件的 `multiWindowAccess` 设为 `false`（防止无限嵌套弹窗）

### Requirement: wry OHOS 桥接 new_window_req_handler
`wry/src/ohos/mod.rs` SHALL 移除 `let _ = new_window_req_handler` 抑制，改为将 handler 桥接到 `openharmony-ability::WebViewBuilder.on_window_new()`。

#### Scenario: wry handler 被正确传递到 ArkTS
- **WHEN** wry 创建 WebView 并设置 `new_window_req_handler`
- **THEN** 该 handler 通过 `WebViewBuilder.on_window_new()` 注册到 openharmony-ability
- **THEN** 前端 `window.open()` 触发时，handler 闭包被调用

#### Scenario: handler URL 解析失败时 Deny
- **WHEN** `targetUrl` 无法解析为有效 URL
- **THEN** handler 返回 `Deny`（安全回退）

### Requirement: Create 变体降级为 Allow
OHOS 平台上，`NewWindowResponse::Create { window }` SHALL 降级为 `Allow` 行为（创建 ArkTS dialog）。

#### Scenario: Create 在 OHOS 上不崩溃
- **WHEN** handler 返回 `NewWindowResponse::Create { window }`
- **THEN** wry OHOS 层将其视为 `Allow`
- **THEN** ArkTS 以 dialog 形式打开新窗口

### Requirement: setWebController 必须被调用
ArkTS `onWindowNew` 回调 SHALL 确保在所有代码路径（Allow / Deny / 异常）中都调用 `event.handler.setWebController()`，防止渲染进程阻塞。

#### Scenario: NAPI 回调异常时仍调用 setWebController
- **WHEN** NAPI `onWindowNew` 回调抛出异常
- **THEN** ArkTS 在 catch 块中调用 `event.handler.setWebController(null)`
- **THEN** 渲染进程不阻塞

### Requirement: 前端测试用例
examples/api 应用 SHALL 提供 `on_new_window` 的测试页面，包含 Deny 和 Allow 两种模式的测试按钮。

#### Scenario: Deny 测试按钮
- **WHEN** 测试页面设置 handler 为 Deny 模式
- **WHEN** 点击 "Open new window" 按钮触发 `window.open()`
- **THEN** 无新窗口出现，测试 PASS

#### Scenario: Allow 测试按钮
- **WHEN** 测试页面设置 handler 为 Allow 模式
- **WHEN** 点击 "Open new window" 按钮触发 `window.open()`
- **THEN** 弹出 dialog 包含新 Web 组件，测试 PASS

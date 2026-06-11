# ohos-webview-user-agent Specification

## Purpose
TBD - created by archiving change user-agent. Update Purpose after archive.
## Requirements
### Requirement: OHOS WebView 支持自定义 User-Agent
OHOS WebView SHALL 在创建时支持设置自定义 User-Agent 字符串。当开发者通过 `WebviewBuilder::user_agent()` 或 `with_user_agent()` 提供 User-Agent 时，ArkTS 层 SHALL 调用 `WebviewController.setCustomUserAgent()` 方法设置该值。

#### Scenario: Web 组件有 src URL 时设置 User-Agent
- **WHEN** 开发者创建 Web 组件并提供 `userAgent` 字段，且 Web 组件 `src` 属性包含 URL
- **THEN** 系统 SHALL 在 `onControllerAttached` 回调中调用 `WebviewController.setCustomUserAgent(data.userAgent)`
- **AND** 后续加载的页面 SHALL 使用开发者指定的 User-Agent

#### Scenario: Web 组件 src 为空时设置 User-Agent
- **WHEN** 开发者创建 Web 组件并提供 `userAgent` 字段，且 Web 组件 `src` 属性为空字符串
- **THEN** 系统 SHALL 在调用 `loadUrl()` 之前调用 `WebviewController.setCustomUserAgent(data.userAgent)`
- **AND** 后续通过 `loadUrl()` 加载的页面 SHALL 使用开发者指定的 User-Agent

#### Scenario: 未提供 userAgent 时保持默认行为
- **WHEN** 开发者创建 Web 组件但未提供 `userAgent` 字段
- **THEN** 系统 SHALL 不调用 `setCustomUserAgent()`
- **AND** Web 组件 SHALL 使用系统默认的 User-Agent

#### Scenario: setCustomUserAgent 调用失败时不中断页面加载
- **WHEN** 调用 `WebviewController.setCustomUserAgent()` 抛出异常
- **THEN** 系统 SHALL 捕获异常并记录错误日志
- **AND** 系统 SHALL 继续执行页面加载流程，不中断应用

### Requirement: User-Agent 值直接使用开发者提供的字符串
OHOS WebView SHALL 直接使用开发者提供的 `userAgent` 字符串，不自动追加系统默认 User-Agent。

#### Scenario: 开发者提供完整自定义 User-Agent
- **WHEN** 开发者提供 `userAgent` 为 `"MyApp/1.0 Tauri/2.0"`
- **THEN** 系统 SHALL 将 User-Agent 设置为 `"MyApp/1.0 Tauri/2.0"`
- **AND** 不追加任何系统默认 User-Agent 信息

#### Scenario: 开发者需要追加默认 User-Agent
- **WHEN** 开发者希望在系统默认 User-Agent 基础上追加自定义信息
- **THEN** 开发者 SHALL 在 Rust 层通过 JavaScript 的 `navigator.userAgent` 获取当前默认值
- **AND** 开发者 SHALL 在 Rust 层拼接完整的 User-Agent 字符串后传递给 `WebviewBuilder::user_agent()`

### Requirement: WebBuilder 和 EmbeddedWebBuilder 均支持 User-Agent
OHOS 的 `WebBuilder` 和 `EmbeddedWebBuilder` SHALL 均支持自定义 User-Agent 设置。

#### Scenario: WebBuilder 支持 User-Agent
- **WHEN** 使用 `WebBuilder` 创建 Web 组件并提供 `userAgent` 字段
- **THEN** 系统 SHALL 在 `onControllerAttached` 回调中设置 User-Agent
- **AND** 行为与 Scenario "Web 组件有 src URL 时设置 User-Agent" 一致

#### Scenario: EmbeddedWebBuilder 支持 User-Agent
- **WHEN** 使用 `EmbeddedWebBuilder` 创建 Web 组件并提供 `userAgent` 字段
- **THEN** 系统 SHALL 在 `onControllerAttached` 回调中设置 User-Agent
- **AND** 行为与 Scenario "Web 组件有 src URL 时设置 User-Agent" 一致

### Requirement: 与 Windows/macOS 行为保持一致
OHOS WebView 的 User-Agent 设置行为 SHALL 与 Windows/macOS 保持一致。

#### Scenario: 跨平台 User-Agent 一致性
- **WHEN** 开发者在 Windows、macOS、OHOS 上使用相同的 `userAgent` 值创建 Web 组件
- **THEN** 三个平台 SHALL 使用相同的 User-Agent 字符串加载页面
- **AND** 页面通过 `navigator.userAgent` 获取的值 SHALL 一致


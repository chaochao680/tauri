## Why

OHOS WebView 支持自定义 User-Agent 能力，但当前 Tauri OHOS 适配中未实际使用该能力。虽然 `WebviewInitData` 接口已定义 `userAgent` 字段，且 wry 和 tauri-runtime-wry 已正确传递该值，但 ArkTS 层在创建 Web 组件时未调用 `setCustomUserAgent()` 方法，导致开发者设置的 User-Agent 被忽略。

开发者需要自定义 User-Agent 以实现：
- 标识应用身份（如 `MyApp/1.0 Tauri/2.0`）
- 服务端根据 User-Agent 返回适配的页面内容
- 数据统计和来源追踪
- 跨平台一致性（Windows/macOS 已支持）

## What Changes

- 在 `openharmony-ability` 的 ArkTS 层，Web 组件创建时调用 `WebviewController.setCustomUserAgent()` 方法
- 如果开发者提供了 `userAgent` 字段，在 `onControllerAttached` 回调中设置自定义 User-Agent
- 如果 Web 组件 `src` 为空（延迟加载场景），在 `loadUrl` 之前调用 `setCustomUserAgent()`
- 建议用法：先调用 `getUserAgent()` 获取默认值，再追加自定义信息

## Capabilities

### New Capabilities

- `ohos-webview-user-agent`: OHOS WebView 自定义 User-Agent 能力，支持在 Web 组件创建时设置自定义 User-Agent 字符串

### Modified Capabilities

（无）

## Impact

**代码变更**：
- `openharmony-ability/native_ability/src/main/ets/webview/DefaultWebview.ets`
  - `WebBuilder` 函数：在 `onControllerAttached` 回调中调用 `setCustomUserAgent()`
  - `EmbeddedWebBuilder` 函数：同上
- 可能修改 `type.ets`（如需调整 `WebviewInitData` 接口定义）

**API 兼容性**：
- `WebviewController.setCustomUserAgent()` 从 API 10+ 可用，满足 Tauri OHOS 最低 API 要求（API 12）
- 不影响现有 API 签名，仅补充实现

**平台行为**：
- 开发者通过 `WebviewBuilder::user_agent()` 或 `with_user_agent()` 设置的 User-Agent 将在 OHOS 上生效
- 与 Windows/macOS 行为保持一致

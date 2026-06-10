## Context

OHOS ArkWeb 提供 `WebviewController.setCustomUserAgent(userAgent: string)` 方法（API 10+）用于设置自定义 User-Agent。

**当前状态**：
- `WebviewInitData` 接口已定义 `userAgent?: string` 字段
- wry 通过 `WebViewBuilder::user_agent()` 传递 user_agent 到 `WebViewInitData`
- tauri-runtime-wry 调用 `with_user_agent()` 传递
- **问题**：ArkTS 层 `WebBuilder` 和 `EmbeddedWebBuilder` 未调用 `setCustomUserAgent()`，导致 User-Agent 被忽略

**OHOS API 约束**：
- `setCustomUserAgent()` 是 `WebviewController` 的方法，不是 Web 组件属性
- 必须在 Web 组件与 controller 绑定后调用（即 `onControllerAttached` 回调中）
- 如果 Web 组件 `src` 为空（延迟加载），需先调用 `setCustomUserAgent()`，再通过 `loadUrl` 加载页面
- 建议用法：先 `getUserAgent()` 获取默认值，再追加自定义信息

**OHOS 官方建议**：
- 当 Web 组件 `src` 设置了 URL 时，在 `onControllerAttached` 回调中设置 User-Agent
- 不建议在 `onLoadIntercept` 回调中设置，会概率性出现设置失败
- 当 Web 组件 `src` 为空时，先调用 `setCustomUserAgent()`，再通过 `loadUrl` 加载

## Goals / Non-Goals

**Goals:**
- 实现 OHOS WebView 自定义 User-Agent 能力，使开发者通过 `WebviewBuilder::user_agent()` 设置的 User-Agent 生效
- 支持两种加载场景：
  1. Web 组件 `src` 有 URL：在 `onControllerAttached` 回调中设置
  2. Web 组件 `src` 为空：在 `loadUrl` 之前设置
- 与 Windows/macOS 行为保持一致

**Non-Goals:**
- 不实现动态修改 User-Agent（页面加载后通过 JS 修改）
- 不提供 `getUserAgent()` 查询接口给 Tauri 开发者（可通过 JS 的 `navigator.userAgent` 获取）
- 不处理 User-Agent 的持久化或缓存

## Decisions

### 1. 在 `onControllerAttached` 回调中设置 User-Agent

**决策**：在 `WebBuilder` 和 `EmbeddedWebBuilder` 的 `onControllerAttached` 回调中，如果 `data.userAgent` 存在，调用 `ctrl.setCustomUserAgent(data.userAgent)`。

**理由**：
- OHOS 官方推荐在此回调中设置 User-Agent
- 此时 controller 已绑定，可以安全调用方法
- 适用于 `src` 有 URL 的场景

**代码示例**：
```typescript
.onControllerAttached(() => {
  const ctrl = data.controller;
  
  // Set custom User-Agent if provided
  if (data.userAgent) {
    try {
      ctrl.setCustomUserAgent(data.userAgent);
    } catch (error) {
      hilog.error(DOMAIN, 'DefaultWebview', 'setCustomUserAgent failed: %{public}s', JSON.stringify(error));
    }
  }
  
  // ... existing loadUrl/loadData logic
})
```

**替代方案**：
- 在 Web 组件外部调用 `setCustomUserAgent()`：不可行，必须等 controller 绑定
- 在 `onLoadIntercept` 中设置：OHOS 官方不推荐，概率性失败

### 2. 延迟加载场景的处理

**决策**：对于 `src` 为空的 Web 组件（使用 `loadUrl` 延迟加载），在调用 `loadUrl` 之前设置 User-Agent。

**理由**：
- OHOS 官方建议：当 `src` 为空时，先设置 User-Agent，再加载页面
- 如果在页面加载后设置，可能导致页面与实际 User-Agent 不符

**代码示例**：
```typescript
// 延迟加载场景
if (!data?.src && data.userAgent) {
  try {
    ctrl.setCustomUserAgent(data.userAgent);
  } catch (error) {
    hilog.error(DOMAIN, 'DefaultWebview', 'setCustomUserAgent failed: %{public}s', JSON.stringify(error));
  }
}
ctrl.loadUrl(url, headers);
```

### 3. 不追加默认 User-Agent

**决策**：直接使用开发者提供的 `userAgent` 字符串，不自动追加默认 User-Agent。

**理由**：
- OHOS 官方建议"先 `getUserAgent()` 获取默认值，再追加自定义信息"，但这是**建议**而非强制
- Tauri 开发者可能希望完全自定义 User-Agent（如 `MyApp/1.0`），不希望追加系统默认值
- 如果开发者需要追加，可以自己在 Rust 层获取默认值后追加

**替代方案**：
- 自动追加：调用 `ctrl.getUserAgent()` 获取默认值，拼接 `data.userAgent`
  - 优点：保留系统信息
  - 缺点：开发者无法完全自定义
- 提供两种模式：通过 `WebviewInitData` 新增字段控制是否追加
  - 优点：灵活
  - 缺点：增加复杂度，Tauri 其他平台无此选项

**最终选择**：直接使用开发者提供的值，不追加。与 Windows/macOS 行为一致。

## Risks / Trade-offs

**[Risk] `setCustomUserAgent()` 可能失败** → 使用 try-catch 捕获错误，记录日志但不中断页面加载

**[Risk] 延迟加载场景下 User-Agent 设置时机** → 在 `loadUrl` 之前设置，符合 OHOS 官方建议

**[Risk] 开发者期望追加默认 User-Agent** → 在文档中说明：如需追加，开发者应在 Rust 层自行获取默认值并拼接

**[Trade-off] 不提供 `getUserAgent()` 查询接口** → 开发者可通过 JS 的 `navigator.userAgent` 获取，无需额外 Rust API

**[Trade-off] 不支持动态修改 User-Agent** → 如需修改，开发者可通过 JS 修改 `navigator.userAgent`（但仅影响当前页面），或重新创建 Web 组件

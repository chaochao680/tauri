## 1. openharmony-ability ArkTS 层实现

- [x] 1.1 在 `native_ability/src/main/ets/webview/DefaultWebview.ets` 的 `WebBuilder` 函数中，在 `onControllerAttached` 回调中添加 User-Agent 设置逻辑：如果 `data.userAgent` 存在，调用 `ctrl.setCustomUserAgent(data.userAgent)`，使用 try-catch 捕获错误并记录日志
- [x] 1.2 在 `native_ability/src/main/ets/webview/DefaultWebview.ets` 的 `EmbeddedWebBuilder` 函数中，在 `onControllerAttached` 回调中添加相同的 User-Agent 设置逻辑
- [x] 1.3 在 `WebBuilder` 中处理延迟加载场景：如果 `data?.src` 为空且 `data.userAgent` 存在，在调用 `loadUrl()` 之前设置 User-Agent
- [x] 1.4 在 `EmbeddedWebBuilder` 中处理延迟加载场景：同上
- [x] 1.5 验证 `type.ets` 中 `WebviewInitData.userAgent` 字段定义是否正确（应为 `userAgent?: string`）

## 2. 测试验证

- [ ] 2.1 在 `examples/api/src/lib/tests/plugins.ts` 中添加 auto 测试：创建 Web 组件并设置自定义 User-Agent，通过 JavaScript 获取 `navigator.userAgent` 验证是否正确设置（当前为 manual 占位，auto 测试待实现）
- [ ] 2.2 添加 side-effect 测试：验证未设置 `userAgent` 时保持默认行为（当前为 manual 占位，auto 测试待实现）
- [x] 2.3 构建并部署到设备，验证 Web 组件加载的页面使用正确的 User-Agent

## 3. 文档更新

- [x] 3.1 更新 `openspec/changes/user-agent/proposal.md` 标记完成状态
- [x] 3.2 更新 `openspec/changes/user-agent/design.md` 标记完成状态
- [x] 3.3 更新 `openspec/changes/user-agent/specs/ohos-webview-user-agent/spec.md` 标记完成状态

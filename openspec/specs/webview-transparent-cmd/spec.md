## Requirements

### Requirement: OHOS mobile 平台 WebView 运行时背景色设置命令可用

`plugin:webview|set_webview_background_color` 命令 SHALL 在所有平台（包括 OHOS mobile）上注册并可用，使 `webviewWindow.setBackgroundColor()` 和 `webview.setBackgroundColor()` 的 TypeScript API 在 OHOS 设备上能正常调用。

#### Scenario: OHOS mobile 上设置 WebView 背景为透明
- **WHEN** 在 OHOS mobile 设备上调用 `webviewWindow.setBackgroundColor([0, 0, 0, 0])`
- **THEN** `plugin:window|set_background_color` SHALL 成功设置窗口壳层背景为透明
- **THEN** `plugin:webview|set_webview_background_color` SHALL 成功设置 WebView 组件背景为透明
- **THEN** HTML `body { background: transparent }` SHALL 能穿透到窗口壳层
- **测试分类**: `manual`（需人工确认透明穿透效果）

#### Scenario: OHOS mobile 上运行时切换 WebView 背景色
- **WHEN** 在 OHOS mobile 设备上调用 `webviewWindow.setBackgroundColor([255, 0, 0, 128])`（半透明红）
- **THEN** `plugin:webview|set_webview_background_color` SHALL 成功执行
- **THEN** WebView 组件背景 SHALL 更新为半透明红色
- **测试分类**: `side-effect`

#### Scenario: OHOS mobile 上重置 WebView 背景色
- **WHEN** 在 OHOS mobile 设备上调用 `webview.setBackgroundColor(null)`
- **THEN** `plugin:webview|set_webview_background_color` SHALL 成功执行
- **THEN** WebView 组件背景 SHALL 恢复为默认值
- **测试分类**: `side-effect`

#### Scenario: Desktop 平台行为不变
- **WHEN** 在 Windows/macOS/Linux 上调用 `webviewWindow.setBackgroundColor(...)`
- **THEN** 行为 SHALL 与修改前完全一致
- **THEN** 其他 `desktop_commands`（zoom/hide/show/print 等）SHALL 不受影响
- **测试分类**: `auto`

---

## API 映射表

| TypeScript API | Rust 命令 | 注册位置 | 修改前 | 修改后 |
|---------------|----------|---------|--------|--------|
| `webviewWindow.setBackgroundColor(color)` | `plugin:window\|set_background_color` | `window/plugin.rs:308` | ✅ 全平台 | ✅ 全平台（不变） |
| | `plugin:webview\|set_webview_background_color` | `webview/plugin.rs:246` | ❌ `#[cfg(desktop)]` | ✅ 全平台 |
| `webview.setBackgroundColor(color \| null)` | `plugin:webview\|set_webview_background_color` | 同上 | ❌ `#[cfg(desktop)]` | ✅ 全平台 |

## Why

Phase 1 在 wry OHOS 后端已完整实现了 `set_background_color`（RGBA → `0xAARRGGBB` 转换 + 调用 openharmony-ability NAPI），但 **tauri 层的命令注册遗漏了 OHOS mobile 平台**。

`webviewWindow.setBackgroundColor()` 的 TypeScript 实现（`webviewWindow.ts:222`）是链式调用：
```typescript
async setBackgroundColor(color: Color): Promise<void> {
  return invoke('plugin:window|set_background_color', { value: color })
    .then(() => {
      return invoke('plugin:webview|set_webview_background_color', { value: color })
    })
}
```

- 第一步 `plugin:window|set_background_color`：**全平台注册**（`window/plugin.rs:308`，无 `cfg(desktop)` 门控）→ ✅ OHOS 可用
- 第二步 `plugin:webview|set_webview_background_color`：**仅 desktop 注册**（`webview/plugin.rs:246`，在 `#[cfg(desktop)] mod desktop_commands` 内）→ ❌ OHOS mobile 不可用

在 OHOS mobile 设备上，第二步 invoke 报 `command not found` 错误，导致整个 `setBackgroundColor` 调用失败。窗口壳层已变透明，但 Webview 组件仍为白色不透明，HTML `body { background: transparent }` 无法穿透。

## What Changes

- **命令提取**：将 `set_webview_background_color` 从 `#[cfg(desktop)] mod desktop_commands` 提取到新的无平台限制的 `mod commands`
- **注册修改**：在 `generate_handler!` 宏中，将 `#[cfg(desktop)] desktop_commands::set_webview_background_color` 改为 `commands::set_webview_background_color`

## Capabilities

### New Capabilities
- `webview-transparent-cmd`: OHOS mobile 平台上 WebView 运行时背景色设置命令可用

### Modified Capabilities
（无现有 capability 的需求变更）

## Impact

- **tauri** (Rust)：`crates/tauri/src/webview/plugin.rs` — 新增 `commands` 模块（~15 行），修改 1 行注册代码
- **Desktop 平台**：行为完全不变，命令仍可用
- **OHOS mobile**：命令从不可用变为可用，修复卡片 2 的透明 WebView 功能
- **OHOS desktop**：行为不变（`cfg(desktop)=true` 时命令本来就可用）

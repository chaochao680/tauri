## Why

当前 Tauri OHOS 平台上窗口背景透明（`transparent: true`）和背景色（`background_color`）完全无效。tao 的 `set_background_color()` 是空操作，`Window::new()` 忽略 `window_attrs.transparent` 和 `window_attrs.background_color`。WindowConfig 接口无透明/背景色字段，WindowManager 不调用 `setWindowBackgroundColor`。这导致：
- 无法实现透明悬浮窗、毛玻璃效果等依赖窗口透明的 UI
- Phase 1 的 WebView 透明效果无法穿透到桌面（窗口不透明时 WebView 透明无意义）
- 与 Windows（DWM blur + WM_ERASEBKGND）/macOS（setOpaque + NSColor）行为不一致

## What Changes

- **tao OHOS Window 结构扩展**：`Window` 结构体添加 `transparent: bool` 字段；`Window::new()` 读取 `window_attrs.transparent` 和 `window_attrs.background_color` 并通过 NAPI 传递；`set_background_color()` 从 no-op 改为实际调用
- **NAPI 桥接层扩展**：`create_os_window` config 对象添加 `transparent` 和 `background_color` 字段；新增 `setWindowBackgroundColor(windowId, color)` NAPI 绑定
- **ArkTS WindowConfig 扩展**：`type.ets` 的 `WindowConfig` 添加 `transparent?: boolean` 和 `backgroundColor?: number` 字段
- **ArkTS WindowManager 窗口背景设置**：创建子窗口后调用 `win.setWindowBackgroundColor()` 设置背景色/透明
- **颜色格式转换**：Rust 端 `Option<RGBA>` → `0xAARRGGBB` u32，ArkTS 端 u32 → `#AARRGGBB` 字符串

## Capabilities

### New Capabilities
- `window-background`: OHOS 窗口背景色和透明控制，包括创建时 transparent/background_color 属性传递和运行时 set_background_color 动态更新

### Modified Capabilities
（无现有 capability 的需求变更）

## Impact

- **tao** (Rust)：`src/platform_impl/ohos/mod.rs` 的 Window 结构体、`new()`、`set_background_color()` 需修改
- **openharmony-ability** (Rust)：`crates/ability/src/window/mod.rs` 的 `create_os_window` 添加 transparent/backgroundColor 参数 + 新增 `set_window_background_color` NAPI 函数
- **openharmony-ability** (ArkTS)：`type.ets`（WindowConfig 接口）、`WindowManager.ets`（setWindowBackgroundColor 调用）需修改
- **API 兼容性**：`setWindowBackgroundColor` 为 API 9+（满足最低 API 12 要求）
- **向下兼容**：`transparent` 默认为 `false`，`background_color` 默认为 `None`，不改变现有行为

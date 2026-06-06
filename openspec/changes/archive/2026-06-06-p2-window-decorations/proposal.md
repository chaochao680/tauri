## Why

当前 Tauri OHOS 平台上 `decorations: false` 完全无效。tao 的 `set_decorations()` 是空操作（no-op），`is_decorated()` 始终返回 `true`。FloatPage.ets 的自定义标题栏（MenuBarComponent + 拖拽区 + 关闭按钮）无条件渲染，无法创建无边框窗口。这导致：
- 无法实现悬浮窗口、弹出面板、自定义标题栏等依赖 `decorations: false` 的 UI 效果
- 与 Windows/macOS 平台行为不一致（两个平台均已完整实现 decorations 控制）
- 应用无法跨平台使用统一的无边框窗口模式

## What Changes

- **tao OHOS Window 结构扩展**：`Window` 结构体添加 `decorations: AtomicBool` 字段；`Window::new()` 读取 `window_attrs.decorations` 并存储；`set_decorations()` 从 no-op 改为实际通过 NAPI 调用 ArkTS 更新窗口装饰状态；`is_decorated()` 读取 AtomicBool
- **NAPI 桥接层 decorations 传递**：`openharmony-ability/crates/ability/src/window/mod.rs` 的 `create_os_window` config 对象添加 `decorations` 字段；新增 `setWindowDecorations(windowId, bool)` NAPI 绑定用于运行时切换
- **ArkTS WindowConfig 接口扩展**：`type.ets` 的 `WindowConfig` 添加 `decorations?: boolean` 字段
- **ArkTS ArkHelper 处理 decorations**：`ArkHelper.ets` 的 `createOSWindow` 处理 decorations 属性，传递给 WindowManager；新增 `setWindowDecorations` handler
- **FloatPage 条件渲染**：`FloatPage.ets` 根据 decorations 状态条件渲染 MenuBarComponent、拖拽区和关闭按钮；decorations=false 时隐藏所有窗口装饰元素
- **状态栏隐藏（主窗口）**：当主窗口 decorations=false 时，通过 `setWindowSystemBarEnable([])` 隐藏系统状态栏（API 9+）
- **LocalStorage 状态传递**：WindowManager 将 decorations 状态通过 LocalStorage 传递给 FloatPage

## Capabilities

### New Capabilities
- `window-decorations`: OHOS 窗口装饰控制，包括创建时 decorations 属性传递和运行时 set_decorations 动态切换，覆盖 FloatPage 自定义标题栏和系统状态栏

### Modified Capabilities
（无现有 capability 的需求变更）

## Impact

- **tao** (Rust)：`src/platform_impl/ohos/mod.rs` 的 Window 结构体、`new()`、`set_decorations()`、`is_decorated()` 需修改
- **openharmony-ability** (Rust)：`crates/ability/src/window/mod.rs` 的 `create_os_window` 添加 decorations 参数 + 新增 `set_window_decorations` NAPI 函数
- **openharmony-ability** (ArkTS)：`type.ets`（WindowConfig 接口）、`ArkHelper.ets`（decorations 处理）、`WindowManager.ets`（decorations 传递到子窗口）、`FloatPage.ets`（条件渲染）需修改
- **API 兼容性**：不引入新 OHOS API 依赖。`setWindowSystemBarEnable` 为 API 9+（已满足最低 API 12 要求）
- **向下兼容**：`decorations` 默认为 `true`，不改变现有窗口行为

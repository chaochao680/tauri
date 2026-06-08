## Why

当前 Tauri OHOS 平台上 WebView 容器不支持透明背景。虽然 `ArkHelper.ets` 已有 `transparent` 属性处理逻辑，但**优先级逻辑反了**（`background_color` 优先于 `transparent`），且 `DefaultWebview.ets` 的 WebBuilder 中 `background_color` 的 `#RRGGBBAA` 格式与 OHOS 期望的 `#AARRGGBB` 不匹配，导致颜色渲染错误。此外，`wry` 的 `set_background_color()` 是空操作，运行时无法动态更新背景色。这些问题导致：
- 无法实现悬浮窗口、毛玻璃效果、叠加层等依赖透明背景的 UI 效果
- `decorations: false`（无标题栏）的窗口场景下，WebView 白色背景遮挡内容
- 与 Windows/macOS 平台行为不一致，影响跨平台应用体验

## What Changes

- **ArkHelper.ets 优先级修复**：修复 `transparent` 优先于 `background_color` 的逻辑（当前反了），透明处理统一收敛在 ArkHelper 层
- **WebView 颜色格式转换**：在 `DefaultWebview.ets` 的 `WebBuilder` 和 `EmbeddedWebBuilder` 中添加 `#RRGGBBAA` → `#AARRGGBB` 格式转换（含 `Color` 枚举类型守卫），WebBuilder 不处理 `transparent` 逻辑
- **父容器防御性透明**：`DefaultXComponent.ets` 的 `Stack`、`Row`、`Column` 三层容器显式设置透明背景
- **WebView set_background_color 动态更新**：`wry/src/ohos/mod.rs` 中 `set_background_color()` 从 no-op 改为实际调用。注意：`WebviewController.setBackgroundColor()` 不是 OHOS 官方 API，通过 openharmony-ability 自定义 NAPI 桥接（monkey-patch → applyStyle → WebBuilder 重渲染）实现
- **NAPI 桥接层类型修正**：`setBackgroundColor` 从 string 改为 number（`0xAARRGGBB`），涉及 Rust NAPI 绑定、ArkHelper 猴子补丁（两处）、`WebviewStyle` 类型扩展、`ProxyJsHelper` 签名等多处同步修改
- **渲染模式验证**（待定）：默认异步渲染模式下 surface 可能不透，需验证是否需要 `renderMode: RenderMode.SYNC_RENDER`

## Capabilities

### New Capabilities
- `webview-transparent-bg`: WebView 容器透明背景支持，包括创建时 transparent 属性和运行时 set_background_color 动态更新

### Modified Capabilities
（无现有 capability 的需求变更）

## Impact

- **openharmony-ability** (ArkTS)：`ArkHelper.ets`（优先级修复 + 猴子补丁签名）、`DefaultWebview.ets`（WebBuilder + EmbeddedWebBuilder 格式转换 + WebviewStyle 类型扩展）、`DefaultXComponent.ets`（Stack/Row/Column 防御性透明）、`Utils.ets`（转换函数 + JsHelper/ProxyJsHelper 签名）需修改
- **openharmony-ability** (Rust)：`helper/webview.rs` 的 `set_background_color` NAPI 参数类型从 `&str` 改为 `u32`，Function 泛型从 `String` 改为 `u32`
- **wry** (Rust)：`src/ohos/mod.rs` 的 `set_background_color()` 需从 no-op 改为实际调用，并进行 RGBA → `0xAARRGGBB` 格式转换
- **API 兼容性**：不引入新 API，仅补齐现有 `transparent` 和 `background_color` 字段在 OHOS 的实现
- **向下兼容**：`transparent` 默认为 `false`，不改变现有行为

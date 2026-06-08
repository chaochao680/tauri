## ADDED Requirements

### Requirement: WebView 创建时支持透明背景
当 `WebViewInitData.transparent` 为 `true` 时，ArkHelper.ets SHALL 将 `backgroundColor` 设置为 `Color.Transparent`，使 OHOS Web 组件渲染为透明背景，HTML 内容层的透明区域可以穿透到下层容器。

#### Scenario: transparent=true 创建 WebView
- **WHEN** 调用方创建 WebView 时设置 `transparent: true`
- **THEN** ArkHelper.ets SHALL 将 `style.backgroundColor` 设置为 `Color.Transparent`
- **THEN** Web 组件 SHALL 使用 `Color.Transparent` 作为 `backgroundColor`
- **测试分类**: `manual`（需人工确认透明效果是否可见）

#### Scenario: transparent=false 创建 WebView（默认行为）
- **WHEN** 调用方创建 WebView 时 `transparent` 为 `false` 或未设置
- **THEN** Web 组件 SHALL 使用默认不透明背景
- **测试分类**: `auto`（验证默认行为不变）

#### Scenario: transparent 优先于 background_color
- **WHEN** 调用方同时设置 `transparent: true` 和 `background_color: "#FF000080"`
- **THEN** ArkHelper.ets SHALL 忽略 `background_color`，使用 `Color.Transparent`
- **测试分类**: `side-effect`（验证 backgroundColor 最终值为 Color.Transparent 而非 hex 字符串）

### Requirement: WebView 创建时支持自定义背景色
当 `WebViewStyle.background_color` 指定且 `transparent` 不为 `true` 时，WebBuilder/EmbeddedWebBuilder SHALL 将颜色从 `#RRGGBBAA` 转换为 `#AARRGGBB` 格式后设置到 Web 组件。

#### Scenario: 指定 background_color 创建 WebView
- **WHEN** 调用方设置 `background_color: "#FF000080"`（`#RRGGBBAA` 格式，半透明红）且 `transparent` 不为 `true`
- **THEN** WebBuilder/EmbeddedWebBuilder SHALL 将 `#RRGGBBAA` 转换为 `#AARRGGBB` 格式（`#80FF0000`）并用作 `backgroundColor`
- **测试分类**: `side-effect`（验证渲染后颜色正确，需要视觉确认半透明效果）

#### Scenario: 未指定 background_color 且 transparent=false
- **WHEN** 调用方未设置 `background_color` 且 `transparent` 不为 `true`
- **THEN** Web 组件 SHALL 使用默认不透明背景
- **测试分类**: `auto`

### Requirement: 运行时动态更新 WebView 背景色
`set_background_color` 方法 SHALL 在运行时通过 `wry → openharmony-ability NAPI → ArkHelper applyStyle → WebBuilder 重渲染` 链路更新 Web 组件的背景色。参数为 `0xAARRGGBB` 格式的 number 类型。

**注意**：`WebviewController.setBackgroundColor()` 不是 OHOS 官方 API。本方案通过 openharmony-ability 的自定义 NAPI 桥接实现：Rust 端调用 ArkTS 端 monkey-patch 的 `setBackgroundColor` 方法，触发 `applyStyle` → `node.update()` → WebBuilder 重渲染。

#### Scenario: 运行时设置透明背景
- **WHEN** 调用方通过 `set_background_color` 传入透明色（`0x00000000`）
- **THEN** Web 组件背景 SHALL 变为透明
- **测试分类**: `side-effect`（需验证 applyStyle 触发重渲染后背景确实更新）

#### Scenario: 运行时设置自定义颜色
- **WHEN** 调用方通过 `set_background_color` 传入 `0xFFFF0000`（不透明红）
- **THEN** Web 组件背景 SHALL 变为对应颜色
- **测试分类**: `side-effect`

### Requirement: WebView 父容器在透明模式下不遮挡
当 WebView 设置为透明背景时，其所有父容器（Stack/Row/Column）SHALL 不阻止透明穿透。作为防御性编程，这些容器 SHALL 显式设置透明背景。

#### Scenario: 透明 WebView 的父容器
- **WHEN** WebView 创建时 `transparent: true`
- **THEN** 父 Stack、Row、Column 容器 SHALL 均显式使用透明背景（`Color.Transparent`）
- **测试分类**: `manual`（需人工确认透明穿透效果）

#### Scenario: 不透明 WebView 的父容器
- **WHEN** WebView 创建时 `transparent: false`
- **THEN** 父 Stack、Row、Column 容器 SHALL 使用默认背景（不改变现有行为）
- **测试分类**: `auto`

---

## API 映射表

| Tauri/wry 接口 | OHOS 实现方式 | 参数格式 | 说明 |
|---------------|--------------|---------|------|
| `WebViewAttributes.transparent: bool` | ArkHelper.ets 消费 → `init.style.backgroundColor = Color.Transparent` | — | NAPI `#[napi(object)]` 自动传为 JS boolean |
| `WebViewAttributes.background_color: Option<RGBA>` | wry 格式化为 `#RRGGBBAA` → ArkHelper 传入 `style.backgroundColor` → WebBuilder 转换为 `#AARRGGBB` | string `#RRGGBBAA` | 8 位 hex，alpha 在末尾 |
| `wry::InnerWebView::set_background_color(RGBA)` | Rust 转为 `0xAARRGGBB` u32 → NAPI → ArkTS monkey-patch `setBackgroundColor` → `applyStyle` → WebBuilder 重渲染 | number `0xAARRGGBB` | **非官方 API**，通过 NAPI 自定义桥接 |
| Web 组件 `.backgroundColor()` | ArkUI 通用属性 | `ResourceColor`（`string \| number \| Color`） | 支持 `#AARRGGBB`、`0xAARRGGBB`、`Color.Transparent` |
| `WebviewController.setBackgroundColor()` | **不存在于官方 API** | — | 代码库通过 NAPI 自定义方法实现 |
| `RenderMode.SYNC_RENDER` | Web 组件构造参数 | `RenderMode` 枚举 | 同步渲染模式，支持 alpha 通道混合。**透明度验证需测试此模式** |

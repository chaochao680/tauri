# window-background Specification

## Purpose
TBD - created by archiving change p3-window-background. Update Purpose after archive.
## Requirements
### Requirement: 窗口创建时支持 transparent 属性
当 `WindowAttributes.transparent` 为 `true` 时，tao OHOS Window SHALL 将窗口背景设置为完全透明，通过 OHOS `setWindowBackgroundColor('#00000000')` API 实现。

#### Scenario: transparent=true 创建 Float 窗口
- **WHEN** 调用方创建 Float 窗口时设置 `transparent: true`
- **THEN** tao `Window::new()` SHALL 将 transparent 状态存储
- **THEN** NAPI config 对象 SHALL 包含 `transparent: true`
- **THEN** WindowManager SHALL 在创建子窗口后调用 `setWindowBackgroundColor('#00000000')`
- **测试分类**: `manual`（需人工确认窗口背景透明效果）

#### Scenario: transparent=false 创建窗口（默认行为）
- **WHEN** 调用方创建窗口时 `transparent` 为 `false` 或未设置
- **THEN** 窗口 SHALL 使用默认不透明背景
- **测试分类**: `auto`（验证默认行为不变）

#### Scenario: transparent 优先于 background_color
- **WHEN** 调用方同时设置 `transparent: true` 和 `background_color: Some((255, 0, 0, 128))`
- **THEN** 窗口 SHALL 使用完全透明背景，忽略 background_color
- **测试分类**: `side-effect`（验证最终背景色为透明而非红色）

### Requirement: 窗口创建时支持 background_color 属性
当 `WindowAttributes.background_color` 指定且 `transparent` 不为 `true` 时，窗口 SHALL 使用指定颜色作为背景色。

#### Scenario: 指定 background_color 创建窗口
- **WHEN** 调用方设置 `background_color: Some((255, 0, 0, 128))`（半透明红）且 `transparent` 不为 `true`
- **THEN** Rust 端 SHALL 将 RGBA 转换为 `0xAARRGGBB` 格式（`0x80FF0000`）
- **THEN** NAPI config SHALL 包含 `backgroundColor: 0x80FF0000`
- **THEN** WindowManager SHALL 调用 `setWindowBackgroundColor('#80FF0000')`
- **测试分类**: `side-effect`（验证渲染后颜色正确）

#### Scenario: 未指定 background_color 且 transparent=false
- **WHEN** 调用方未设置 `background_color` 且 `transparent` 不为 `true`
- **THEN** 窗口 SHALL 使用默认背景（不调用 setWindowBackgroundColor）
- **测试分类**: `auto`

### Requirement: 运行时动态更新窗口背景色
`set_background_color(Option<RGBA>)` 方法 SHALL 在运行时通过 NAPI 调用 ArkTS 更新窗口背景色。

#### Scenario: 运行时设置透明背景
- **WHEN** 调用方通过 `set_background_color(None)` 且窗口 transparent=true
- **THEN** 窗口背景 SHALL 保持透明（fallback 到透明色 `0x00000000`）
- **测试分类**: `side-effect`

#### Scenario: 运行时设置自定义颜色
- **WHEN** 调用方通过 `set_background_color(Some((255, 0, 0, 255)))` 传入不透明红
- **THEN** 窗口背景 SHALL 变为红色（`#FFFF0000`）
- **测试分类**: `side-effect`

#### Scenario: 运行时设置 None 恢复默认
- **WHEN** 调用方通过 `set_background_color(None)` 且窗口 transparent=false
- **THEN** 窗口背景 SHALL 恢复为不透明白色（`#FFFFFFFF`）
- **测试分类**: `side-effect`

### Requirement: 颜色格式正确转换
Rust 端 SHALL 将 `Option<RGBA>` 正确转换为 `0xAARRGGBB` u32 格式，ArkTS 端 SHALL 将 u32 正确转换为 `#AARRGGBB` 字符串格式。

#### Scenario: RGBA 到 AARRGGBB 转换
- **WHEN** Rust 端收到 `RGBA(255, 128, 0, 200)`
- **THEN** SHALL 转换为 `0xC8FF8000`（AA=C8, RR=FF, GG=80, BB=00）
- **测试分类**: `auto`（可单元测试验证转换函数）

#### Scenario: ArkTS u32 到字符串转换
- **WHEN** ArkTS 端收到 `backgroundColor: 0xC8FF8000`
- **THEN** SHALL 转换为 `'#C8FF8000'` 字符串
- **测试分类**: `auto`

#### Scenario: 完全透明色转换
- **WHEN** transparent=true 时
- **THEN** Rust 端 SHALL 输出 `0x00000000`
- **THEN** ArkTS 端 SHALL 转换为 `'#00000000'`
- **测试分类**: `auto`

---


## ADDED Requirements

### Requirement: 窗口创建时支持 decorations 属性
当 `WindowAttributes.decorations` 为 `false` 时，tao OHOS Window SHALL 将 decorations 状态通过 NAPI 传递给 ArkTS 层，使 FloatPage 不渲染自定义标题栏（MenuBarComponent、拖拽区、关闭按钮）。

#### Scenario: decorations=false 创建 Float 窗口
- **WHEN** 调用方创建 Float 窗口时设置 `decorations: false`
- **THEN** tao `Window::new()` SHALL 将 `decorations=false` 存储在 `AtomicBool` 中
- **THEN** NAPI config 对象 SHALL 包含 `decorations: false` 字段
- **THEN** WindowManager SHALL 将 `decorations` 写入 LocalStorage
- **THEN** FloatPage SHALL 不渲染 MenuBarComponent、拖拽区和关闭按钮
- **测试分类**: `manual`（需人工确认无边框窗口效果）

#### Scenario: decorations=true 创建 Float 窗口（默认行为）
- **WHEN** 调用方创建 Float 窗口时 `decorations` 为 `true` 或未设置
- **THEN** FloatPage SHALL 渲染完整的自定义标题栏（MenuBarComponent + 拖拽区 + 关闭按钮）
- **测试分类**: `auto`（验证默认行为不变）

#### Scenario: decorations=false 创建主窗口（UIAbility）
- **WHEN** 调用方创建主窗口时设置 `decorations: false`
- **THEN** tao `Window::new()` SHALL 将 `decorations=false` 存储
- **THEN** ArkTS 侧 SHALL 调用 `setWindowSystemBarEnable([])` 隐藏系统状态栏
- **测试分类**: `manual`（需人工确认状态栏隐藏效果）

### Requirement: 运行时动态切换窗口 decorations
`set_decorations(bool)` 方法 SHALL 在运行时通过 NAPI 调用 ArkTS 更新窗口 decorations 状态，触发 FloatPage 条件渲染更新。

#### Scenario: 运行时隐藏 decorations
- **WHEN** 对已创建的 decorations=true 窗口调用 `set_decorations(false)`
- **THEN** tao `Window` SHALL 更新 `AtomicBool` 为 `false`
- **THEN** NAPI `setWindowDecorations(windowId, false)` SHALL 被调用
- **THEN** ArkTS LocalStorage 的 `decorations` SHALL 更新为 `false`
- **THEN** FloatPage SHALL 通过 `@LocalStorageProp` 响应式隐藏标题栏组件
- **测试分类**: `side-effect`（需验证 UI 动态更新）

#### Scenario: 运行时恢复 decorations
- **WHEN** 对 decorations=false 的窗口调用 `set_decorations(true)`
- **THEN** FloatPage SHALL 通过 `@LocalStorageProp` 响应式显示标题栏组件
- **测试分类**: `side-effect`

### Requirement: is_decorated 返回正确状态
`is_decorated()` 方法 SHALL 返回当前窗口的 decorations 状态，反映最近一次 `set_decorations` 或创建时的值。

#### Scenario: 查询 decorations=true 窗口
- **WHEN** 窗口创建时 `decorations=true` 且未调用过 `set_decorations(false)`
- **THEN** `is_decorated()` SHALL 返回 `true`
- **测试分类**: `auto`

#### Scenario: 查询 decorations=false 窗口
- **WHEN** 窗口创建时 `decorations=false`
- **THEN** `is_decorated()` SHALL 返回 `false`
- **测试分类**: `auto`

#### Scenario: 运行时切换后查询
- **WHEN** 窗口调用 `set_decorations(false)` 后查询 `is_decorated()`
- **THEN** SHALL 返回 `false`
- **测试分类**: `auto`

### Requirement: decorations=false 时保留 resize 能力
当 `decorations: false` 时，FloatPage 的 resize handle（右边缘、下边缘、右下角）SHALL 保持可见和可用，确保无边框窗口仍可调整大小。

#### Scenario: 无边框窗口 resize
- **WHEN** 窗口 `decorations=false`
- **THEN** FloatPage 的 resize handle SHALL 保持渲染和可用
- **测试分类**: `manual`（需人工验证拖拽调整大小）

### Requirement: 主窗口 decorations=false 时隐藏系统状态栏
当主窗口（UIAbility window）设置 `decorations: false` 时，ArkTS 层 SHALL 调用 `setWindowSystemBarEnable([])` 隐藏系统状态栏，使内容延伸到窗口顶部。

#### Scenario: 主窗口无边框模式
- **WHEN** 主窗口 `decorations=false` 且应用处于全屏/最大化模式
- **THEN** 系统状态栏 SHALL 被隐藏
- **测试分类**: `manual`（需人工确认状态栏隐藏和内容延伸效果）

#### Scenario: 主窗口非全屏模式下 decorations=false
- **WHEN** 主窗口 `decorations=false` 但应用处于悬浮窗/分屏模式
- **THEN** 系统状态栏 MAY 保持显示（API 限制，非全屏模式下不生效）
- **测试分类**: `manual`

---

## API 映射表

| Tauri/tao 接口 | OHOS 实现方式 | 参数 | 说明 |
|---------------|--------------|------|------|
| `WindowAttributes.decorations: bool` | NAPI config.decorations → LocalStorage → FloatPage 条件渲染 | `boolean` | 默认 `true` |
| `Window::set_decorations(bool)` | NAPI `setWindowDecorations(windowId, bool)` → LocalStorage 更新 → @LocalStorageProp 响应 | `(i64, boolean)` | 运行时切换 |
| `Window::is_decorated() -> bool` | `AtomicBool.load(Ordering::Acquire)` | — | 纯 Rust 侧读取 |
| 主窗口状态栏隐藏 | `windowClass.setWindowSystemBarEnable([])` | `Array<'status' \| 'navigation'>` | API 9+，需全屏模式 |
| FloatPage 标题栏 | ArkUI `@LocalStorageProp('decorations')` + `if` 条件渲染 | — | 响应式更新 |

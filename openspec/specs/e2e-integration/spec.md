# e2e-integration Specification

## Purpose
TBD - created by archiving change p4-gap-fix. Update Purpose after archive.
## Requirements
### Requirement: create_os_window 统一参数结构体
`create_os_window` SHALL 使用 `WindowCreateParams` 结构体传递所有参数，整合 decorations（Phase 2）、transparent 和 background_color（Phase 3）属性。

#### Scenario: 创建带完整属性的 Float 窗口
- **WHEN** 调用方使用 `WindowCreateParams { name, window_type, decorations: false, transparent: true, background_color: None }` 调用 `create_os_window`
- **THEN** NAPI config 对象 SHALL 包含所有字段：`name, type, windowId, width, height, x, y, decorations, transparent, backgroundColor`
- **THEN** ArkTS WindowManager SHALL 接收到完整配置并正确处理
- **测试分类**: `auto`

#### Scenario: 创建默认属性窗口
- **WHEN** 调用方使用默认参数 `WindowCreateParams { decorations: true, transparent: false, background_color: None }`
- **THEN** 窗口 SHALL 使用默认行为（有标题栏、不透明背景）
- **测试分类**: `auto`

### Requirement: 主窗口统一初始化
主窗口（window_id=0）的 decorations + transparent + background_color SHALL 通过统一的 `initMainWindow(config)` NAPI 调用处理，避免多个 Phase 分别处理导致时序冲突。

#### Scenario: 主窗口 decorations=false + transparent=true
- **WHEN** tao 创建主窗口时 `decorations=false` 且 `transparent=true`
- **THEN** NAPI `initMainWindow` SHALL 被调用
- **THEN** ArkTS SHALL 先隐藏系统状态栏，再设置窗口背景透明
- **测试分类**: `manual`

#### Scenario: 主窗口默认属性
- **WHEN** tao 创建主窗口时使用默认属性
- **THEN** `initMainWindow` 可以不调用或传入默认值
- **THEN** 主窗口行为不变
- **测试分类**: `auto`

### Requirement: decorations + transparent 组合场景
当 `decorations: false` 且 `transparent: true` 同时设置时，窗口 SHALL 同时隐藏标题栏并使背景透明，实现完整的无边框透明窗口效果。

#### Scenario: 无边框透明 Float 窗口
- **WHEN** 创建 Float 窗口 `decorations: false` 且 `transparent: true`
- **THEN** FloatPage SHALL 不渲染标题栏组件（Phase 2）
- **THEN** WindowManager SHALL 设置窗口背景透明（Phase 3）
- **THEN** 窗口 SHALL 呈现为无边框透明效果
- **测试分类**: `manual`

#### Scenario: 无边框 + 自定义背景色窗口
- **WHEN** 创建 Float 窗口 `decorations: false` 且 `background_color: (128, 0, 0, 200)`
- **THEN** FloatPage SHALL 不渲染标题栏组件
- **THEN** 窗口背景 SHALL 为半透明暗红色
- **测试分类**: `manual`

### Requirement: 多窗口状态隔离
多个窗口的 decorations、transparent、background_color 状态 SHALL 相互独立，修改一个窗口的属性不影响其他窗口。

#### Scenario: 两个窗口不同 decorations 状态
- **WHEN** 创建窗口 A（decorations=true）和窗口 B（decorations=false）
- **THEN** 窗口 A SHALL 显示标题栏，窗口 B SHALL 不显示
- **THEN** 切换窗口 A 的 decorations 不影响窗口 B
- **测试分类**: `side-effect`

#### Scenario: 两个窗口不同 transparent 状态
- **WHEN** 创建窗口 A（transparent=true）和窗口 B（transparent=false）
- **THEN** 窗口 A SHALL 背景透明，窗口 B SHALL 背景不透明
- **测试分类**: `manual`

### Requirement: 端到端属性传递完整性
从 Tauri config 到 ArkTS 的完整属性传递链路 SHALL 保持所有窗口属性（decorations、transparent、background_color）不丢失、不变更。

#### Scenario: Tauri → wry → tao → NAPI → ArkTS 全链路
- **WHEN** Tauri 配置 `decorations: false, transparent: true`
- **THEN** 属性 SHALL 完整传递到 ArkTS WindowManager
- **THEN** ArkTS SHALL 正确应用所有属性
- **测试分类**: `side-effect`

---


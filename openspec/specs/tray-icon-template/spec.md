# tray-icon-template Specification

## Purpose
TBD - created by archiving change tray-icon-as-template. Update Purpose after archive.
## Requirements
### Requirement: Template 模式下自动生成白色和黑色图标

当 `icon_as_template = true` 时，系统 SHALL 将用户提供的图标视为单色蒙版（alpha mask），自动生成白色版（RGB=255）和黑色版（RGB=0），分别填入 OHOS `StatusBarIcon.white` 和 `StatusBarIcon.black` 字段。alpha 通道 SHALL 完整保留。

#### Scenario: Builder 设置 template 模式后构建
- **WHEN** 调用 `TrayIconBuilder::new().icon(icon).icon_as_template(true).build(app)`
- **THEN** 状态栏图标在浅色壁纸下显示为黑色版，在深色壁纸下显示为白色版

#### Scenario: Builder 不设置 template 模式（默认行为）
- **WHEN** 调用 `TrayIconBuilder::new().icon(icon).build(app)` 且未调用 `icon_as_template()`
- **THEN** `StatusBarIcon.white` 和 `StatusBarIcon.black` 填入同一张原始图标（当前行为不变）

#### Scenario: 单色图标 template 模式
- **WHEN** 用户提供白色信封图标并设置 `icon_as_template(true)`
- **THEN** 白色版为纯白信封（alpha 保留），黑色版为纯黑信封（alpha 保留），系统根据壁纸自动切换

#### Scenario: 带透明度的图标 template 模式
- **WHEN** 用户提供带半透明抗锯齿边缘的图标并设置 `icon_as_template(true)`
- **THEN** 白色版和黑色版的半透明边缘 alpha 值保持不变，仅 RGB 被替换

### Requirement: set_icon_as_template 运行时切换

`TrayIcon::set_icon_as_template(bool)` 在 OHOS 上 SHALL 立即生效，通过 remove + rebuild + add 更新状态栏图标。

#### Scenario: 运行时启用 template 模式
- **WHEN** tray icon 已存在且当前 `icon_as_template = false`，调用 `set_icon_as_template(true)`
- **THEN** 状态栏图标更新为白色版/黑色版双图标，系统根据壁纸自动切换

#### Scenario: 运行时禁用 template 模式
- **WHEN** tray icon 已存在且当前 `icon_as_template = true`，调用 `set_icon_as_template(false)`
- **THEN** 状态栏图标的 white 和 black 字段恢复为同一张原始图标

### Requirement: set_icon 保持 template 设置

`TrayIcon::set_icon()` 更换图标后 SHALL 读取当前 `attrs.icon_is_template` 属性，若为 `true` 则自动生成新的白色版/黑色版图标。

#### Scenario: template 模式下更换图标
- **WHEN** 当前 `icon_as_template = true`，调用 `set_icon(new_icon)`
- **THEN** 新图标自动生成白色版和黑色版，template 效果不丢失

#### Scenario: 非 template 模式下更换图标
- **WHEN** 当前 `icon_as_template = false`，调用 `set_icon(new_icon)`
- **THEN** 新图标的 white 和 black 字段为同一张图（当前行为不变）

### Requirement: set_icon_with_as_template 组合设置

`TrayIcon::set_icon_with_as_template(icon, is_template)` 在 OHOS 上 SHALL 同时更新图标和 template 模式，新图标根据 `is_template` 值自动生成白色版/黑色版或保持原图。

#### Scenario: 组合设置启用 template 并更换图标
- **WHEN** 调用 `set_icon_with_as_template(new_icon, true)`
- **THEN** 图标更新为 `new_icon`，且自动生成白色版和黑色版，template 模式立即生效

#### Scenario: 组合设置禁用 template 并更换图标
- **WHEN** 调用 `set_icon_with_as_template(new_icon, false)`
- **THEN** 图标更新为 `new_icon`，white 和 black 字段为同一张原图，template 模式关闭

### Requirement: 前端 auto 测试覆盖

SHALL 在前端测试中添加 `iconAsTemplate` 相关的 auto 测试用例，验证 API 调用不抛异常。

#### Scenario: setIconAsTemplate 不抛异常
- **WHEN** 创建 tray icon 后调用 `tray.setIconAsTemplate(true)`
- **THEN** 调用成功，不抛出异常

#### Scenario: setIconAsTemplate false 不抛异常
- **WHEN** 调用 `tray.setIconAsTemplate(false)`
- **THEN** 调用成功，不抛出异常

### Requirement: 手动测试覆盖

SHALL 在前端手动测试 UI 中提供验证 template 模式视觉效果的手段。

#### Scenario: 手动验证 template 视觉效果
- **WHEN** 用户在 Tray UI 中勾选 "Icon as template" 并创建 tray icon
- **THEN** 更换系统壁纸（深色/浅色），状态栏图标颜色自动切换，始终可见

#### Scenario: 手动验证切换 template 模式
- **WHEN** tray icon 已存在，用户取消勾选 "Icon as template" 并调用 setIconAsTemplate(false)
- **THEN** 状态栏图标恢复为原始颜色，不随壁纸变化


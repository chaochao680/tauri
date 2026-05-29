# Phase 8: Menu & Tray OHOS 能力缺口审计与替代方案设计

> 职责：系统审计 `tauri::menu` 和 `tauri::tray` 模块在 OHOS 上的缺失能力，区分"可实现"、"有替代"、"不可实现"，设计修复方案与验证策略
> 代码位置：`tauri/crates/tauri/src/menu/`、`tauri/crates/tauri/src/tray/` (公开 API 层)，底层实现涉及 `muda/src/platform_impl/ohos/`、`tray-icon/src/platform_impl/ohos/`、`openharmony-ability/`
> 独立性：✓ 仅影响 OHOS 路径，不影响 Windows/macOS/Linux

---

## 一、审计范围

本审计的范围是 **Rust 侧 `tauri::menu` 和 `tauri::tray` 模块的公开类型**：

### 四、Rust 侧 tauri::menu 模块
- `Menu`
- `MenuEvent`
- `MenuItem`
- `PredefinedMenuItem`
- `Submenu`

### 五、Rust 侧 tauri::tray 模块
- `TrayIconBuilder`
- `TrayIconEvent`
- `MouseButton`
- `MouseButtonState`

这些类型是跨平台公开 API。底层 crate（muda、tray-icon、openharmony-ability）的改动是实现支撑，不在本审计的直接范围内，但必须为公开类型的正确行为做必要修改。

对照跨平台 API 逐项检查 OHOS 实现，以下是完整审计结果。在范围内的所有项目要么有实现方案，要么有明确原因说明不能实现。

---

## 二、可实现项（需要代码修复）

### 8.1 Accelerator 序列化缺失

**关联公开类型**：`MenuItem`、`Submenu`（通过底层 `MenuChild.to_menu_item_data()`）

**现状**：`muda/src/platform_impl/ohos/mod.rs:267` `to_menu_item_data()` 中 `accelerator: None` 硬编码，即使 `self.accelerator` 有值也不传给原生侧。

**影响**：菜单弹出时快捷键提示（如 "Ctrl+C"）永远不显示。

**ArkTS 能力**：`MenuPopup.ets:39` 已经用 `labelInfo: item.accelerator` 渲染快捷键文本。`MenuItemData.accelerator: Option<String>` 已定义。基础设施到位，只需 Rust 侧填入。

**方案**：
- `to_menu_item_data()` 中 `accelerator: None` → `accelerator: self.accelerator.map(|k| k.to_string())`
- `KeyAccelerator` 已有 `Display` impl（返回如 `"Ctrl+O"`），直接用 `to_string()`
- Submenu 的 `new_submenu()` 中 `accelerator` 也应初始化为传入值（当前硬编码 None）

**验证**：
- **手动**：现有 `menu.ts` 有 `MenuItem.setAccelerator` auto test——目前只验证不报错。添加手动测试：创建带 accelerator 的菜单项 → popup → 观察 ArkTS 菜单右侧是否显示快捷键文本
- **自动**：无法自动验证 UI 文本，但可通过 `menu.ts` 的 `setAccelerator` 确保不 crash

---

### 8.2 TrayIconEvent `_ => todo!()` catch-all

**关联公开类型**：`TrayIconEvent`

**现状**：`tauri/crates/tauri/src/tray/mod.rs:196` `From<tray_icon::TrayIconEvent>` 实现中有 `_ => todo!()` catch-all。如果 `tray-icon` crate 新增 variant，会 panic。

**影响**：潜在 crash 风险。

**方案**：
- `_ => todo!()` → 安全 fallback：`_ => TrayIconEvent::Click { id: TrayIconId::new("unknown"), position: PhysicalPosition::new(0.0, 0.0), rect: Rect::default(), button: MouseButton::Left, button_state: MouseButtonState::Up }`
- 加 `log::warn!("Unhandled TrayIconEvent variant, falling back to Click")`

**验证**：
- **自动**：`tray.ts` 现有 `TrayIcon.event_handler_register` test 验证回调注册不 crash
- **手动**：点击 tray 图标 → 观察控制台无 panic/warning（正常流程不触发 catch-all）

---

### 8.3 About PredefinedMenuItem — no-op → AlertDialog

**关联公开类型**：`PredefinedMenuItem`

**现状**：`predefined.ets:65` `case 'about': break;` — 空操作。

**ArkTS 能力**：`AlertDialog` 完全可用。`showAlertDialog(AlertDialogParamWithButtons)` 支持标题、消息、按钮。`CustomDialogController` 支持自定义布局。

**方案**：
- `predefined.ets` 的 `'about'` case → 调用 `showAlertDialog` 显示应用名称/版本/作者/版权等
- Rust 侧 `MenuChild::new_predefined(About(metadata))` 已存储 `AboutMetadata`
- `to_menu_item_data()` 需传递 `about_metadata` 到 ArkTS（当前丢弃）
- 扩展 `MenuItemData` 添加 `aboutMetadata?: { name?, version?, shortVersion?, authors?, comments?, copyright?, website?, icon?, license?, websiteLabel?, credits? }`
- `MenuPopup.ets` 点击 about predefined → 调 `MenuManager.handleItemClick` → `PredefinedActionExecutor.execute('about')` → `showAlertDialog`

**注意**：此修改涉及 menu 和 tray 两侧的 predefined executor。menu popup 中的 about 项点击走 `PredefinedActionExecutor`（ArkTS 侧），tray 的 status bar menu 中的 about 项点击走 Rust 侧 `execute_predefined_action` → `openharmony_ability::execute_predefined_action` TSFN → ArkTS `PredefinedActionExecutor`。两边都使用同一个 ArkTS `PredefinedActionExecutor`，所以只需修改一处 ArkTS 代码。

**验证**：
- **手动**：现有 `menu.ts` 有 `PredefinedMenuItem.about_exec` manual test（目前空壳）。点击 about 菜单项 → 观察 AlertDialog 弹出，显示应用信息
- **自动**：`PredefinedMenuItem.about` auto test 已验证构造不 crash

---

### 8.4 CloseWindow BUG — close = minimize（menu 和 tray 两侧）

**关联公开类型**：`PredefinedMenuItem`

**现状**：
- **Menu 侧**：`menu.ets PredefinedActionExecutor:54-56` `case 'close': case 'destroyWindow': await this.win?.minimize();`
- **Tray 侧**：`tray-icon/src/platform_impl/ohos/event.rs:97` `"minimize" | "hide" | "maximize" | "close" | "fullscreen"` 全部走 `openharmony_ability::execute_predefined_action` → 同一个 ArkTS `PredefinedActionExecutor` → `close` 也是 minimize
- **两层调用链**：tray Rust 侧 → TSFN → ArkTS `PredefinedActionExecutor`。所以 tray 和 menu 的 predefined 动作最终都由 ArkTS 侧执行。

**影响**：用户点击"关闭窗口"菜单项（无论是从 tray menu 还是 popup menu），窗口只是最小化而非关闭。

**方案**：
- 只需修改 ArkTS 侧一处：`predefined.ets` 的 `close` case
- `close` 应映射到 `this.context?.terminateSelf()`（即 Quit）
- 但 `destroyWindow()` 会销毁主窗口 → 触发 `onAbilityDestroy` → 进程终止（等同于 Quit）
- OHOS 只有一个主窗口，`close` = `destroyWindow` = 进程终止。这与 Windows/macOS 的 Close 行为一致（关闭主窗口 = 退出应用）
- Rust 侧 `execute_predefined_action` 已将 `"close"` 传给 ArkTS 侧，无需额外修改
- ⚠️ **需确认 UX 决策**：OHOS 只有一个主窗口，`close` = Quit 是 by design。如果应用需要"最小化而非退出"的行为，则保留 minimize 映射。默认推荐 Quit（与 Windows/macOS 行为一致）

**验证**：
- **手动**：`menu.ts` 有 `PredefinedMenuItem.closeWindow` manual test。点击 Close → 观察应用退出（而非最小化）
- tray 侧的 close 同样验证：tray menu 中有 CloseWindow 项 → 点击 → 应用退出

---

### 8.5 Fullscreen ≠ Maximize（menu 和 tray 两侧）

**关联公开类型**：`PredefinedMenuItem`

**现状**：
- **Menu 侧**：`menu.ets PredefinedActionExecutor:47-49` `case 'maximize': case 'fullscreen': await this.win?.maximize();`
- **Tray 侧**：Rust 侧 `"maximize" | "close" | "fullscreen"` 都走同一个 ArkTS executor，所以 tray 和 menu 都受影响
- fullscreen 和 maximize 行为相同，都调用 `win.maximize()`

**ArkTS 能力**：
- `window.setWindowLayoutFullScreen(true)` + `setWindowSystemBarEnable([])` = 沉浸式全屏（移动设备有效）
- `window.maximize(MaximizePresentation.ENTER_IMMERSIVE)` = 桌面全屏模式（2in1/desktop 设备）
- `window.recover()` = 恢复正常
- 移动设备：`setWindowLayoutFullScreen` 有效；桌面设备：只能用 `maximize`

**方案**：
- 分离 `fullscreen` 和 `maximize` 的行为（一处 ArkTS 修改覆盖 menu 和 tray）
- `maximize` → `this.win?.maximize()`
- `fullscreen` → 根据 `TAURI_OHOS_DEVICE_TYPE`：
  - desktop: `this.win?.maximize(window.MaximizePresentation.ENTER_IMMERSIVE)`
  - mobile: `this.win?.setWindowLayoutFullScreen(true)` + `setWindowSystemBarEnable([])`
- `recover` (退出全屏/最大化) → `this.win?.recover()` 或 `setWindowLayoutFullScreen(false)`
- Rust 侧 `execute_predefined_action` 已将 `"fullscreen"` 和 `"maximize"` 传给 ArkTS 侧，无需额外修改

**验证**：
- **手动**：`menu.ts` 有 `PredefinedMenuItem.fullscreen` 和 `PredefinedMenuItem.maximize` manual test。分别点击 → 验证行为不同（fullscreen 进入沉浸式，maximize 进入桌面全屏）
- tray menu 中同样验证 fullscreen 和 maximize 行为不同

---

## 三、有替代方案项（平台限制，需要验证近似合理性）

### 8.6 TrayIcon rect() — 从 None 到 AvoidArea 近似值

**关联公开类型**：`TrayIconBuilder`（通过 `TrayIcon.rect()`）

**现状**：`tauri/crates/tauri/src/tray/mod.rs:642-646` OHOS 分支 `rect()` 返回 `Ok(None)`。

**平台限制**：StatusBar API 不提供 tray icon 在状态栏内的精确位置和尺寸。

**替代方案合理性验证**：

| 平台 | 实现方式 | 返回值语义 | 精确度 |
|------|---------|-----------|--------|
| **Windows** | `Shell_NotifyIconGetRect()` | tray icon 的**精确** RECT（位置+尺寸） | 精确 |
| **macOS** | `NSStatusItem.button().window().frame()` → scale_factor 转换 | tray icon button 的**精确** Rect | 精确 |
| **OHOS（AvoidArea.topRect）** | `getWindowAvoidArea(TYPE_SYSTEM).topRect` | 整个状态栏区域 `{ left, top, width, height }` px | 近似，但语义偏差大 |

**关键差异**：Windows/macOS 返回的是 tray icon **自身**的精确 rect（例如 36x36 px），用于 popup 定位、相对位置计算。OHOS 的 `AvoidArea.topRect` 返回的是**整个状态栏区域**（例如 `{ left:0, top:0, width:1440, height:48 }`），不是 tray icon 本身。

**合理性判断**：
- ❌ 如果调用者用 `rect` 来定位 popup 菜单：返回整个状态栏宽度（1440px）会导致 popup 定位在错误位置
- ❌ 如果调用者用 `rect.size` 来判断 tray icon 尺寸：48px 高 × 1440px 宽是错误尺寸
- ✓ 如果调用者只需要"tray icon 大致在屏幕哪个区域"：AvoidArea 比完全返回 `None` 有信息量

**方案**：
- **不使用 AvoidArea.topRect 近似值**——语义偏差太大，可能误导调用者
- 保持 `rect()` 返回 `None`，与 Linux 平台一致（Linux 也是 `None`）
- 在 `tauri/crates/tauri/src/tray/mod.rs` `rect()` 方法文档中注明 OHOS 平台限制：
  ```
  // OHOS: Unsupported, always returns `None`.
  // StatusBar API does not provide tray icon position or dimensions.
  // AvoidArea.topRect returns the entire status bar area, not the tray icon itself,
  // so it cannot serve as a meaningful approximation.
  ```
- 在 `tray-icon/src/platform_impl/ohos/mod.rs` `rect()` 注明同样限制

**验证**：
- **自动**：现有 `tray.ts` 测试 `TrayIcon.rect()` 返回 `None`——不需要改动
- 与 Linux 行为对齐：Linux `rect()` 也是 `None`，这是跨平台已有模式

---

### 8.7 Menu Bar — 桌面模式持久菜单栏

**关联公开类型**：`Menu`

**现状**：OHOS 无原生菜单栏 API。`Menu::set_menu()` 在 OHOS 上存储菜单但不渲染为持久菜单栏。

**平台限制**：OHOS 没有 NSMenuBar / Win32 MenuBar 等系统级 API。桌面模式可以通过 ArkUI 组件模拟。

**替代方案**：
- **桌面/2in1 设备**：`Navigation` Split 模式 + `.toolbar(CustomBuilder)` + `ToolBarItem(placement: TOP_BAR_LEADING)` 在窗口顶部渲染持久菜单条
- **移动设备**：不渲染菜单栏（UX 不合适）
- 每个 `ToolBarItem` 对应 Menu 的顶级 Submenu
- 点击 `ToolBarItem` → popup 下拉菜单（使用现有 `MenuPopup` 组件）

**方案**（Phase 8 实现，不推迟到 Phase 9）：
- 在 `MainPage.ets` 中检测 `TAURI_OHOS_DEVICE_TYPE` → desktop 时在窗口顶部渲染 `Row` 菜单条
- Menu 的顶级 item 序列化 → 传入 `AppStorage` → `MainPage.ets` 渲染为 `ToolBarItem`
- 点击 `ToolBarItem` → 触发 `MenuPopup`（使用现有的 `popupFromJson` 机制）
- `Menu::set_menu()` 在 OHOS desktop 模式下需要额外将顶级菜单条数据通过 TSFN 传给 ArkTS
- mobile 模式下 `Menu::set_menu()` 行为不变（仅 popup）

**实现规模**：中等（约 60 行 ArkTS + 20 行 Rust）
- Rust: `Menu::set_menu()` OHOS 分支增加 menubar 数据序列化 + TSFN 调用
- ArkTS: `MainPage.ets` 新增顶部菜单条渲染逻辑（条件渲染，仅 desktop）
- ArkTS: 新增 `MenuBarManager` 或扩展现有 `MenuManager` 处理 menubar 数据

**验证**：
- **手动**：desktop 模式启动 → 观察窗口顶部菜单条 → 点击 File/Edit → 下拉菜单弹出
- **自动**：无法自动验证 UI 布局

---

## 四、不可实现项（平台限制，文档注明）

以下功能因 OHOS StatusBar API 限制**无法在 tray icon 上实现**。这些是系统级 API 限制，无法通过应用层代码绕过。OHOS 应用内组件有完整 API 支持（如 `onHover`、`TapGesture({ count: 2 })`、`MouseButton.Middle`），但 StatusBar tray icon 的回调只提供 `iconClickType` 和 `menuCode`，无坐标、无 hover、无双击、无中键、无 press 状态。

### 8.8 TrayIconEvent::DoubleClick

**关联公开类型**：`TrayIconEvent`

**跨平台定义**：`TrayIconEvent::DoubleClick { id, position, rect, button }`

**其他平台实现**：
- Windows: `WM_LBUTTONDBLCLK` / `WM_RBUTTONDBLCLK` / `WM_MBUTTONDBLCLK` → 系统级双击事件，精确的 rect + position
- macOS: `NSEvent.clickCount == 2` → 系统级双击检测

**OHOS 限制**：StatusBar click callback 只返回 `iconClickType: "leftClick"/"rightClick"`，无 doubleClick 类型。

**不能实现的原因**：StatusBar API 没有双击回调类型，无法在应用层合成（没有时间戳/间隔数据）。

**文档注明**：OHOS tray icon 不分发 DoubleClick 事件。`convert_icon_click()` 只生成 `Click` 事件。

---

### 8.9 TrayIconEvent::Enter / Move / Leave

**关联公开类型**：`TrayIconEvent`

**跨平台定义**：鼠标进入/移动/离开 tray icon 区域

**其他平台实现**：
- Windows: `WM_MOUSEMOVE` + `userdata.entered` flag → Enter/Leave 判定；每次 mouse move 生成 Move 事件，附带精确 `get_tray_rect` + cursor position
- macOS: `NSTrackingAreaOptions::MouseEnteredAndExited | NSTrackingAreaOptions::MouseMoved` → 系统级 hover tracking，精确的 icon_rect + cursor_position

**OHOS 限制**：StatusBar 无 hover/mouse tracking 回调。`onHover(isHover, HoverEvent)` 只能绑定到应用内组件。

**不能实现的原因**：StatusBar tray icon 不接收鼠标移动事件，没有坐标数据，无法在应用层合成。

**文档注明**：OHOS tray icon 不分发 Enter/Move/Leave 事件。

---

### 8.10 MouseButton::Middle

**关联公开类型**：`MouseButton`

**跨平台定义**：中键点击

**其他平台实现**：
- Windows: `WM_MBUTTONDOWN` / `WM_MBUTTONUP` / `WM_MBUTTONDBLCLK` → 完整支持
- macOS: `NSEvent.buttonNumber == 3` → 完整支持

**OHOS 限制**：StatusBar callback 无 `"middleClick"` 类型。OHOS 应用内 `MouseButton.Middle` 完整可用（通过 `onMouse`）。

**不能实现的原因**：StatusBar API 只有 `leftClick` 和 `rightClick`，无 `middleClick`。

**文档注明**：OHOS tray icon 只分发 Left（图标点击）和 Right（菜单点击）。

---

### 8.11 MouseButtonState::Down

**关联公开类型**：`MouseButtonState`

**跨平台定义**：按键按下状态

**其他平台实现**：
- Windows: `WM_LBUTTONDOWN` → `MouseButtonState::Down`; `WM_LBUTTONUP` → `MouseButtonState::Up` — 系统级区分 press/release
- macOS: `NSEvent.type == NSEventType::LeftMouseDown` → `Down`; `NSEventType::LeftMouseUp` → `Up` — 系统级区分

**OHOS 限制**：StatusBar callback 只在 click 完成后触发。OHOS 应用内 `MouseAction.Press` 完整可用。

**不能实现的原因**：StatusBar API 只通知 click 完成，不通知 press 开始。

**文档注明**：OHOS tray icon 所有事件 `button_state = Up`。

---

### 8.12 TrayIcon Click Position

**关联公开类型**：`TrayIconEvent::Click { position }`

**跨平台定义**：`ClickEvent` 提供 `x/y, windowX/windowY, displayX/displayY`

**其他平台实现**：
- Windows: `GetCursorPos()` → 精确的屏幕物理坐标
- macOS: `NSEvent.mouseLocation()` → 精确的屏幕逻辑坐标 + scale_factor 转物理坐标

**OHOS 限制**：StatusBar callback 不传递任何坐标数据。`StatusBarClickEvent` 只有 `click_type` 和 `menu_code`。

**不能实现的原因**：StatusBar API 不提供坐标。

**文档注明**：OHOS tray icon 所有 Click 事件 `position = (0, 0)`。

---

### 8.13 TrayIcon rect() — 不能用 AvoidArea 近似值

**关联公开类型**：`TrayIconBuilder`（通过 `TrayIcon.rect()`）

已在 8.6 中说明：`AvoidArea.topRect` 语义偏差太大（整个状态栏区域 vs tray icon 本身），与 Windows/macOS 返回的精确 tray icon rect 不可比。保持 `None` 是更合理的选择，与 Linux 行为对齐。

---

### 8.14 NativeIcon

**关联公开类型**：`TrayIconBuilder`（通过 `TrayIconBuilder.icon()` / `new_native_icon()`）

**跨平台定义**：macOS 系统图标（NSImage.Name）

**其他平台实现**：
- macOS: `NSImage.systemImageNamed()` → 完整支持
- Windows/Linux: 不支持，静默丢弃参数

**不能实现的原因**：macOS-only 特性。Windows/Linux 也不支持。`new_native_icon()` 静默丢弃参数是合理的跨平台行为。

**文档注明**：OHOS 不支持 NativeIcon，与 Windows/Linux 行为一致。

---

## 五、验证策略

### 自动测试能力分析

| # | 功能 | 能否自动测试 | 自动测试方式 | 能验证什么 | 不能验证什么 |
|---|------|------------|------------|-----------|------------|
| 8.1 | Accelerator 序列化 | ✓ Rust unit test + JS auto test | Rust: `to_menu_item_data_accelerator`; JS: `MenuItem.setAccelerator` (已有) | Rust 侧 JSON 中 `accelerator` 字段非 None; JS 侧 setAccelerator 不 crash | UI 上快捷键文本是否显示（需手动） |
| 8.2 | TrayIconEvent fallback | ✓ Rust unit test | `TrayIconEvent_from_no_panic` | 未知 variant → fallback Click，不 panic | 实际设备上的 tray 事件是否正常（需手动） |
| 8.3 | AboutMetadata 序列化 | ✓ Rust unit test + JS auto test | Rust: `about_metadata_in_menu_item_data`; JS: `PredefinedMenuItem.about` (已有) | Rust 侧 `MenuItemData.aboutMetadata` 字段存在且非空; JS 侧构造不 crash, `text()` 非空 | AlertDialog 是否弹出、内容是否正确（需手动） |
| 8.4 | CloseWindow = quit | ❌ | — | — | 进程终止 (terminateSelf)，自动测试会杀死自己 |
| 8.5 | Fullscreen vs maximize | ❌ | — | — | 窗口状态变化，需要人眼验证沉浸式 vs 最大化外观差异 |
| 8.6 | rect() = None | ✓ JS auto test | `TrayIcon.rect_returns_none` | OHOS 上 `rect()` 返回 null | — |
| 8.7 | Menu Bar 渲染 | ❌ | — | — | UI 布局，需要人眼验证顶部菜单条 + 下拉交互 |
| 8.8-8.11 | MouseButton/State/position | ✓ Rust unit test | `convert_icon_click` / `convert_menu_click` (已有) | button=Left/Right, state=Up, position=(0,0) | — |

### 已有自动测试覆盖

| 模块 | 测试文件 | 已覆盖的缺失项 |
|------|---------|---------------|
| menu | `menu.ts` | `MenuItem.setAccelerator` (auto)、`PredefinedMenuItem.about` (auto)、`PredefinedMenuItem.fullscreen` (auto, 仅验证构造)、`PredefinedMenuItem.minimize/maximize/closeWindow/hide/quit` (manual)、`Menu.popup/popup_at` (manual) |
| tray | `tray.ts` | `TrayIcon.new/setIcon/setMenu/setTooltip/setTitle/setVisible` (auto)、`TrayIcon.event_handler_register` (auto)、`TrayIcon.setMenu_replace` (auto) |
| tray-icon | `event.rs #[cfg(test)]` | `convert_icon_click` (button=Left, state=Up)、`convert_menu_click` (button=Right, state=Up) |

### 需新增的 Rust unit test

| 测试项 | 文件 | 验证内容 |
|--------|------|----------|
| `to_menu_item_data_accelerator` | `muda/src/platform_impl/ohos/mod.rs` | 创建 MenuChild 带 accelerator → `to_menu_item_data()` → 验证 `accelerator` 字段非 None，值为 `"Ctrl+O"` |
| `submenu_accelerator_preserved` | `muda/src/platform_impl/ohos/mod.rs` | 创建 Submenu → `set_key_accelerator("Ctrl+S")` → `to_menu_item_data()` → 验证 `accelerator` 字段非 None |
| `about_metadata_in_menu_item_data` | `muda/src/platform_impl/ohos/mod.rs` | 创建 `MenuChild::new_predefined(About(metadata))` 带 name/version → `to_menu_item_data()` → 验证 `aboutMetadata` 字段存在且包含 name/version |
| `TrayIconEvent_from_no_panic` | `tauri/crates/tauri/src/tray/mod.rs` | 构造未知的 tray_icon TrayIconEvent variant → `From` impl → 验证返回 Click fallback（不 panic），`log::warn` 被触发 |

### 需新增的 JS auto test

| 测试项 | 模块 | 验证内容 |
|--------|------|----------|
| `TrayIcon.rect_returns_none` | tray | 调用 `tray.rect()` → 验证返回值在 OHOS 上为 `null` |

### 需新增的手动测试

| 测试项 | 模块 | 验证内容 | 只能手动的原因 |
|--------|------|----------|--------------|
| `Menu.popup_with_accelerator` | menu | popup 菜单 → 观察 ArkTS 菜单项右侧显示快捷键文本 | UI 文本渲染 |
| `PredefinedMenuItem.about_dialog` | menu | popup 菜单含 About → 点击 → 观察 AlertDialog 弹出应用信息 | UI 弹窗行为 |
| `PredefinedMenuItem.closeWindow_exit` | menu | popup 菜单含 CloseWindow → 点击 → 观察应用退出（非最小化） | 进程终止 |
| `TrayMenu.PredefinedMenuItem.closeWindow_exit` | tray | tray menu 含 CloseWindow → 点击 → 观察应用退出 | 进程终止 |
| `PredefinedMenuItem.fullscreen_immersive` | menu | popup 菜单含 Fullscreen → 点击 → 观察窗口进入沉浸式全屏 | 窗口状态变化 |
| `TrayMenu.PredefinedMenuItem.fullscreen_immersive` | tray | tray menu 含 Fullscreen → 点击 → 验证行为与 menu 侧一致 | 窗口状态变化 |
| `PredefinedMenuItem.maximize_restore` | menu | popup 菜单含 Maximize → 点击 → 观察窗口最大化 | 窗口状态变化 |
| `TrayIcon.click_no_panic` | tray | 点击 tray 图标 → 验证控制台无 panic 或 unexpected warning | 需要实际设备交互 |
| `MenuBar.desktop_visible` | menu | desktop 模式启动 → 观察窗口顶部菜单条 → 点击菜单项 → popup 下拉 | UI 布局渲染 |

---

## 六、实施顺序与优先级

所有实现均在 Phase 8 完成，不推迟到 Phase 9。

| 优先级 | 任务 | 关联公开类型 | 预估工作量 | 风险 |
|--------|------|------------|-----------|------|
| P0 | 8.1 Accelerator 序列化 | MenuItem, Submenu | 1 行代码改动 | 低 — 基础设施已到位 |
| P0 | 8.2 TrayIconEvent todo!() fallback | TrayIconEvent | 5 行代码改动 | 低 — 防止 crash |
| P1 | 8.4 CloseWindow BUG 修复（menu+tray 两侧） | PredefinedMenuItem | 3 行 ArkTS 改动（一处修改覆盖两侧） | 中 — 需确认 close=quit 的 UX 决策 |
| P1 | 8.5 Fullscreen 分离（menu+tray 两侧） | PredefinedMenuItem | 15 行 ArkTS 改动（一处修改覆盖两侧） | 中 — 需设备类型分支 |
| P2 | 8.3 About AlertDialog | PredefinedMenuItem | 30 行改动（Rust + ArkTS） | 中 — 需扩展 MenuItemData + 实现 AlertDialog |
| P2 | 8.7 Menu Bar (desktop) | Menu | 60 行 ArkTS + 20 行 Rust | 中 — 需 Navigation toolbar 渲染逻辑 |
| — | 8.6 rect() 保持 None | TrayIconBuilder | 仅文档注释 | 无 — 与 Linux 行为对齐 |
| — | 8.8-8.12, 8.14 不可实现项 | TrayIconEvent, MouseButton, MouseButtonState | 仅文档注明 | 无 — 不改动代码 |

---

## 七、不可实现项的文档策略

在以下位置添加 OHOS 平台限制文档：

1. **`tauri/crates/tauri/src/tray/mod.rs`** — `TrayIconEvent` `From` impl 附近添加注释：
   ```
   // OHOS platform limitations (TrayIconEvent):
   // - DoubleClick, Enter, Move, Leave events are never dispatched
   // - All Click events have position (0,0) and rect Rect::default()
   // - Only MouseButton::Left (icon click) and Right (menu click) are dispatched
   // - All events have MouseButtonState::Up (no Down state)
   // - rect() always returns None (StatusBar API has no tray icon position;
   //   AvoidArea.topRect is the entire status bar area, not the tray icon itself)
   // - NativeIcon is not supported (same as Windows/Linux)
   ```

2. **`tray-icon/src/platform_impl/ohos/event.rs`** — 文件顶部添加：
   ```
   // OHOS StatusBar API limitations (cannot be fixed at application level):
   // - No double-click detection (StatusBar callback has no "doubleClick" type)
   // - No hover/enter/leave tracking (only "leftClick"/"rightClick" click_type)
   // - No click position coordinates (callback provides only click_type + menuCode)
   // - No middle button support (StatusBar has no "middleClick" type)
   // - No button press state (callback fires on completed click only)
   // Application-internal components have full OHOS gesture/mouse API support,
   // but StatusBar tray icon operates through a system-level extension with
   // limited callback data.
   ```

3. **`tray-icon/src/platform_impl/ohos/mod.rs`** — `rect()` 方法添加注释：
   ```
   // OHOS: rect() always returns None.
   // StatusBar API does not provide tray icon position or dimensions.
   // AvoidArea.topRect returns the entire status bar area (e.g. {0,0,1440,48}),
   // not the tray icon itself — using it as an approximation would mislead callers
   // who rely on rect for popup positioning or size calculations.
   // This is consistent with Linux, which also returns None.
   ```

4. **`tauri/crates/tauri/src/menu/mod.rs`** — Menu OHOS 注释（如果已有 popup 相关注释则补充）：
   ```
   // OHOS: Menu is displayed only as a transient popup via ContextMenu::popup().
   // Desktop mode optionally renders a persistent top menu bar via
   // Navigation.toolbar() + ToolBarItem (see 8.7).
   ```

---

## 八、Predefined Actions 审计：Menu vs Tray 一致性

OHOS 的 predefined actions 有两条调用链：

| 来源 | 调用链 | 最终执行 |
|------|-------|---------|
| **Menu popup** | MenuPopup.ets → `MenuManager.handleItemClick` → `PredefinedActionExecutor.execute(type)` | ArkTS 侧直接执行 |
| **Tray menu** | tray-icon Rust `execute_predefined_action` → TSFN → `openharmony_ability::execute_predefined_action` → ArkTS `PredefinedActionExecutor.execute(type)` | 同一个 ArkTS executor |

**关键发现**：两条链最终都调用同一个 ArkTS `PredefinedActionExecutor`。所以：

- ✅ `close = minimize` BUG：一处 ArkTS 修改覆盖 menu 和 tray 两侧
- ✅ `fullscreen = maximize` BUG：一处 ArkTS 修改覆盖 menu 和 tray 两侧
- ✅ `about = no-op`：一处 ArkTS 修改覆盖 menu 和 tray 两侧
- ⚠️ `quit` 行为不同：tray Rust 侧直接 `app.exit(0)`（绕过 ArkTS），menu ArkTS 侧 `exitFn(0)`（通过 callback）。但效果相同——都是进程退出

**例外**：tray Rust 侧 `"quit"` 不走 TSFN/ArkTS，直接在 Rust 侧调用 `app.exit(0)`。如果需要 quit 行为也统一到 ArkTS，需要额外修改 tray Rust 侧。但当前行为正确，无需改动。

---

## 九、方案审计

### 审计项 1：Spec 覆盖（按公开类型逐项检查）

| 公开类型 | OHOS 缺失能力 | 本设计是否覆盖 | 位置 | 能否实现/原因 |
|----------|-------------|--------------|------|-------------|
| **Menu** | 无持久菜单栏 | ✓ | 8.7 | Phase 8 实现（desktop Navigation.toolbar） |
| **Menu** | Accelerator 不显示（通过底层） | ✓ | 8.1 | 可实现 |
| **MenuEvent** | 无缺失 | — | — | 已正常工作 |
| **MenuItem** | Accelerator 不显示（通过底层） | ✓ | 8.1 | 可实现 |
| **PredefinedMenuItem** | about = no-op | ✓ | 8.3 | 可实现（AlertDialog） |
| **PredefinedMenuItem** | close = minimize BUG | ✓ | 8.4 | 可实现（menu+tray 两侧） |
| **PredefinedMenuItem** | fullscreen = maximize BUG | ✓ | 8.5 | 可实现（menu+tray 两侧） |
| **Submenu** | Accelerator 不显示（通过底层） | ✓ | 8.1 | 可实现 |
| **TrayIconBuilder** | rect() = None | ✓ | 8.6 | 保持 None（AvoidArea 近似值不合理，与 Linux 对齐） |
| **TrayIconBuilder** | NativeIcon 不支持 | ✓ | 8.14 | 不能实现（macOS-only，与 Windows/Linux 一致） |
| **TrayIconEvent** | `_ => todo!()` panic | ✓ | 8.2 | 可实现（安全 fallback） |
| **TrayIconEvent** | DoubleClick 不分发 | ✓ | 8.8 | 不能实现（StatusBar API 无 doubleClick 类型） |
| **TrayIconEvent** | Enter/Move/Leave 不分发 | ✓ | 8.9 | 不能实现（StatusBar API 无 hover 回调） |
| **TrayIconEvent** | Click position = (0,0) | ✓ | 8.12 | 不能实现（StatusBar API 无坐标） |
| **TrayIconEvent** | rect = Rect::default() | ✓ | 8.6+8.12 | 不能实现（与 position 一样无数据源） |
| **MouseButton** | Middle 不分发 | ✓ | 8.10 | 不能实现（StatusBar API 无 middleClick） |
| **MouseButtonState** | Down 不分发 | ✓ | 8.11 | 不能实现（StatusBar API 只通知 click 完成） |

**所有在范围内的项目已覆盖，不能实现的都有明确原因（StatusBar API 限制）。**

### 审计项 2：Placeholder 检查

无 TBD、TODO、"implement later"、"add validation"、"handle edge cases" 等占位符。所有方案有具体文件位置和改动描述。

### 审计项 3：类型一致性

- `MenuItemData.accelerator: Option<String>` — 已存在于 `menu_types.ets`
- `KeyAccelerator` → `to_string()` 产出如 `"Ctrl+O"` — 与 ArkTS `labelInfo` 期望 `ResourceStr` 一致
- `AvoidArea.topRect` → 不使用（语义偏差过大），保持 `None` 与 Linux 对齐
- `AlertDialogParamWithButtons` — ArkTS 类型已确认可用
- `AboutMetadata` Rust → ArkTS 序列化：需要新增 `aboutMetadata` 字段到 `MenuItemData`

### 审计项 4：是否遗漏

- ✅ 所有 5 个"可实现"项有代码方案
- ✅ Menu Bar（8.7）在 Phase 8 实现，不推迟
- ✅ rect() 近似值经与 Windows/macOS 比较验证——AvoidArea 不可用，保持 None
- ✅ Predefined actions 在 menu 和 tray 两侧都有审计——一处 ArkTS 修改覆盖两侧
- ✅ 5 个"不可实现"项有文档注明策略 + 与 Windows/macOS 实现对比
- ✅ 验证策略覆盖所有改动项（包括 tray menu 侧的 predefined 测试）
- ✅ 实施顺序与优先级已排列
- ✅ 未遗漏任何原审计表中的项

---

## 十、Phase 8 不做的事

本 Phase 是**设计阶段**，不做任何源码实现。以下明确排除：

1. ❌ 不修改 `muda/src/platform_impl/ohos/mod.rs` 的 `accelerator: None`
2. ❌ 不修改 `tauri/crates/tauri/src/tray/mod.rs` 的 `_ => todo!()`
3. ❌ 不修改 `predefined.ets` 的 About/Close/Fullscreen case
4. ❌ 不修改 `tauri/crates/tauri/src/tray/mod.rs` 的 `rect()` 返回值（保持 None）
5. ❌ 不修改 `MainPage.ets` 添加菜单条渲染
6. ❌ 不新增 Rust unit test
7. ❌ 不新增 autotest
8. ❌ 不新增 manual test

以上改动留待 Phase 8 实施阶段（Phase 8-progress）。
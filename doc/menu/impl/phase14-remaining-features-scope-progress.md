# Phase 14: OHOS Menu Remaining Features + Multi-Window 彻底实现 - 进度追踪

> 设计文档: [phase14-remaining-features-scope.md](phase14-remaining-features-scope.md)
> 状态: ✅ 代码实施完成，待设备回归验证
> 工期: 5-7 天

---

## 进度总览

| Priority | Feature | 状态 | 备注 |
|----------|---------|------|------|
| **P1** | Dark Mode — menubar 暗色模式适配 | ✅ 完成 | tao set_theme + 颜色资源替换 + 设备验证通过 |
| **P2** | Multi-Window 彻底实现 (6 个子项) | ✅ 完成 | 全局单例→per-window, AppStorage→@State, windowId 转发, MenuBarComponent 提取 |
| **P3** | NativeIcon 部分映射 — OHOS 系统图标 | ✅ 完成 | Rust 层 + 桥接层 + ArkTS 渲染层完成，待设备验证 |
| Skip | HideOthers / ShowAll | ⏭️ 跳过 | OHOS 无此概念，当前 disabled 处理正确 |
| Skip | CHECK_ITEMS 全局状态 | ⏭️ 跳过 | 功能正确，无 bug |

---

## P1: Dark Mode (menubar 暗色模式适配)

> 设计文档 §1

### 问题

Tauri example API app 点击 "Switch to dark mode" 后，WebView 内容切换为暗色，但 **menubar 保持亮色**。四个根因:

- **Root Cause A**: `tao::Window::set_theme()` 在 OHOS 上是 no-op (`tao/src/window.rs:1202-1216`, `not(target_env = "ohos")` guard)
- **Root Cause B**: `MainPage.ets` `MenuBarRow()` 使用硬编码亮色 (`#333333`, `#999999`, `#E0E0E0`, `#EBEBEB`, `#F5F5F5`)
- **Root Cause C**: OHOS Window impl (`tao/src/platform_impl/ohos/mod.rs` line 579+) **完全没有 `set_theme` 方法** — 不是移除 guard 就完事，必须先新增方法
- **Root Cause D**: `theme()` 方法硬编码返回 `Theme::Light` (line 829)，即使实现 set_theme 后也不会反映

### 实施步骤

| Step | 任务 | 状态 | 文件 | 备注 |
|------|------|------|------|------|
| D1 | tao OHOS Window impl **新增** `set_theme()` 方法 + 修复 `theme()` | ✅ | `tao/src/platform_impl/ohos/mod.rs` | theme 状态存储 + NAPI 调用 setColorMode + theme() 返回存储值 |
| D1b | tao `window.rs` 移除 `not(target_env = "ohos")` guard | ✅ | `tao/src/window.rs:1202-1216` | 依赖 D1 先完成 |
| D2 | openharmony-ability 添加 NAPI `set_color_mode()` | ✅ | `openharmony-ability/` | ArkHelper.setColorMode() 通过 UIAbility context setColorMode() |
| D3 | 定义暗色模式颜色资源 | ✅ | `base/element/color.json` + `dark/element/color.json` | HAR + app entry 双位置自定义 menubar 颜色资源 |
| D4 | MenuBarRow 文字颜色替换 | ✅ | `MenuBarComponent.ets` | `#333333`→`$r('app.color.menubar_text')` |
| D5 | MenuBarRow disabled 文字颜色替换 | ✅ | `MenuBarComponent.ets` | `#999999`→`$r('app.color.menubar_text_disabled')` |
| D6 | MenuBarRow hover 背景色替换 | ✅ | `MenuBarComponent.ets` | `#EBEBEB`→`$r('app.color.menubar_item_hover')` |
| D7 | MenuBarRow active 背景色替换 | ✅ | `MenuBarComponent.ets` | `#E0E0E0`→`$r('app.color.menubar_item_active')` |
| D8 | MenuBarRow 容器背景色替换 | ✅ | `MenuBarComponent.ets` | `#F5F5F5`→`$r('app.color.menubar_bg')` |
| D9 | 编译验证: cargo check tao OHOS | ✅ | — | |
| D10 | 编译验证: HAP 构建 | ✅ | — | 状态栏变色正常，无 freeze |

### 验证

| 测试项 | 状态 | 验证内容 |
|--------|------|---------|
| Dark mode 切换 — menubar 背景 | ✅ | 点击 "Switch to dark mode"，menubar 背景色变为暗色 |
| Dark mode 切换 — menubar 文字 | ✅ | menubar 文字颜色变为亮色（暗色背景上可读） |
| Dark mode 切换 — hover/active | ✅ | hover 和 active 状态在暗色模式下可见且有区分 |
| Dark mode 切换 — 右键菜单 | ✅ | 右键 popup 菜单自动适配（使用原生 ArkUI Menu 组件） |
| Light mode 回切 | ⬚ | 切回亮色模式后 menubar 恢复亮色配色（待补充验证） |
| Disabled item 暗色模式 | ⬚ | disabled 项在暗色模式下仍保持灰化+半透明（待补充验证） |

---

## P2: Multi-Window 彻底实现

> 设计文档 §2

### 问题总览

当前多窗口架构存在 6 个问题:

| # | 问题 | 严重度 | 当前行为 |
|---|------|--------|---------|
| MW-1 | `globalMenuClickHandler` 全局单例 | 🔴 | 多窗口最后一个注册覆盖之前的 |
| MW-2 | `MenuManager.handleItemClick` 硬编码 `"main"` | 🔴 | MenuEvent 永远报告来自 main |
| MW-3 | `MenuManager.popupFromJson` 不存储 windowId | 🟡 | executor 无法获知源窗口 |
| MW-4 | `menuStateController` / `globalPopupCallback` 死代码 | 🟢 | 有 import 但从未调用，应清理 |
| MW-5 | AppStorage 逻辑隔离而非真实隔离 | 🟢 | 当前可工作但架构不正确 |
| MW-6 | FloatPage 无 menubar/popup/accelerator | 🟡 | 子窗口功能不完整，需提取 MenuBarComponent |

### MW-1: 全局单例 → Per-Window Registry

| Step | 任务 | 状态 | 文件 | 备注 |
|------|------|------|------|------|
| MW1.1 | `globalMenuClickHandler` 改为 `windowMenuClickHandlers: Map<number, handler>` | ✅ | `MenuBarComponent.ets` | `setMenuClickHandler(windowId, handler)` |
| MW1.2 | `globalRecoverFn` 改为 `windowRecoverFns: Map<number, fn>` | ✅ | `MenuBarComponent.ets` | `setMenuBarRecoverFn(windowId, fn)` |
| MW1.3 | 导出 `getMenuClickHandler(windowId)` / `getMenuBarRecoverFn(windowId)` | ✅ | `MenuBarComponent.ets` | 查找函数 |
| MW1.4 | 导出 `unregisterMenuClickHandler(windowId)` / `unregisterMenuBarRecoverFn(windowId)` | ✅ | `MenuBarComponent.ets` | 清理函数 |
| MW1.5 | 编译验证: HAP 构建 | ✅ | — | |

#### 验证

| 测试项 | 状态 | 验证内容 |
|--------|------|---------|
| 单窗口回归 | ✅ | 主窗口菜单点击正常路由（hilog 确认） |
| 多窗口独立点击 | ⬚ | 两个窗口各自菜单点击路由到各自的 handler（待设备验证） |
| 窗口关闭清理 | ⬚ | 关闭子窗口后 handler 从 Map 中移除（待设备验证） |

### MW-2: MenuEvent 硬编码 "main" → 动态 windowId

| Step | 任务 | 状态 | 文件 | 备注 |
|------|------|------|------|------|
| MW2.1 | MenuManager 添加 `currentWindowId: string` 字段 (默认 `"main"`) | ✅ | `menu.ets` | |
| MW2.2 | MenuManager 添加 `currentWindowIdNumeric: number` 字段 (默认 `0`) | ✅ | `menu.ets` | |
| MW2.3 | `handleItemClick` 中 `emitMenuEventFn(item.id, "main")` 改为 `emitMenuEventFn(item.id, this.currentWindowId)` | ✅ | `menu.ets` | |
| MW2.4 | `handleItemClick` 中 predefined executor 调用添加 targetWindowId | ✅ | `menu.ets` | `this.executor.execute(type, meta, this.currentWindowIdNumeric)` |
| MW2.5 | 编译验证: HAP 构建 | ✅ | — | |

**依赖**: MW-3 (popupFromJson 存储 windowId) 完成后 currentWindowId 才会在 popup 时被设置

#### 验证

| 测试项 | 状态 | 验证内容 |
|--------|------|---------|
| 主窗口 MenuEvent | ⬚ | 主窗口菜单点击 → event windowId = "main"（待设备验证） |
| 子窗口 popup MenuEvent | ⬚ | 子窗口右键菜单点击 → event windowId = 子窗口 ID（待设备验证） |
| predefined action 目标 | ⬚ | 子窗口菜单 minimize → 最小化子窗口而非主窗口（待设备验证） |

### MW-3: MenuManager 存储并转发 windowId

| Step | 任务 | 状态 | 文件 | 备注 |
|------|------|------|------|------|
| MW3.1 | `popupFromJson` 中设置 `this.currentWindowId = wid` | ✅ | `menu.ets` | |
| MW3.2 | `popupFromJson` 中设置 `this.currentWindowIdNumeric = parseInt(wid)` | ✅ | `menu.ets` | `"main"` → 0 |
| MW3.3 | 编译验证: HAP 构建 | ✅ | — | |

**注意**: menubar 的点击不经过 `popupFromJson`，所以 menubar 路径的 windowId 保持默认 `"main"`/0。当前 menubar 只在 MainPage 渲染 (windowId=0)，这是正确的。未来若 FloatPage 也有 menubar，需要额外的机制 (见 MW-6)。

#### 验证

| 测试项 | 状态 | 验证内容 |
|--------|------|---------|
| popup menu windowId 存储 | ⬚ | 从子窗口 popup 菜单后，currentWindowId = 子窗口 ID（待设备验证） |
| 连续 popup 不同窗口 | ⬚ | 先在窗口 A popup，再在窗口 B popup → windowId 正确切换（待设备验证） |

### MW-4: 清理 `menuStateController` / `globalPopupCallback` 死代码

> **审计结论**: 以下代码为死代码（有 export/import 但从未调用任何方法），应清理而非 per-window 化。

| 代码 | 定义位置 | 引用 | 状态 |
|------|---------|------|------|
| `menuStateController` | `menu_state.ets:82` | `NativeAbility.ets:11` import 但**从未调用** | ✅ 已删除 |
| `setGlobalPopupCallback` | `menu_state.ets:87` | 无外部 import/调用 | ✅ 已删除 |
| `triggerGlobalPopup` | `menu_state.ets:91` | 无外部 import/调用 | ✅ 已删除 |

| Step | 任务 | 状态 | 文件 | 备注 |
|------|------|------|------|------|
| MW4.1 | 删除 `export const menuStateController = new MenuStateController()` | ✅ | `menu_state.ets` | 保留 `MenuStateController` 类定义 |
| MW4.2 | 删除 `globalPopupCallback`、`setGlobalPopupCallback`、`triggerGlobalPopup` | ✅ | `menu_state.ets` | 全部无外部调用 |
| MW4.3 | 移除 `NativeAbility.ets` 中 `import { menuStateController }` | ✅ | `NativeAbility.ets` | |
| MW4.4 | 编译验证: HAP 构建 | ✅ | — | popup menu 功能不受影响 |

#### 验证

| 测试项 | 状态 | 验证内容 |
|--------|------|---------|
| 编译通过 | ✅ | 删除后无编译错误 |
| 功能回归 | ✅ | popup menu 正常工作（走的是 `menu.ets popupFromJson` 路径，不经过死代码） |

### MW-5: AppStorage → callback 迁移

> **审计结论**: 共 7 个 AppStorage key 需要迁移。迁移在 MW-6 MenuBarComponent 提取时同步完成（组件内部使用 @State + callback，不再使用 @StorageProp）。此步骤主要关注 **写入方** 的改造。

| Step | 任务 | 状态 | 文件 | 备注 |
|------|------|------|------|------|
| MW5.1 | `MenuBarComponent.ets` 新增 `MenuDataCallback` 接口 | ✅ | `MenuBarComponent.ets` | onMenubarJson/onMenubarVisible/onMenuPopup/onMenuShown |
| MW5.2 | 新增 `menuDataCallbacks: Map` + `registerMenuDataCallback` / `unregisterMenuDataCallback` | ✅ | `MenuBarComponent.ets` | |
| MW5.3 | `NativeAbility.ets` `onMenuRequest` handler 改为通过 callback registry 分发 | ✅ | `NativeAbility.ets` | 不再写 AppStorage |
| MW5.4 | `NativeAbility.ets` `menubar_visible` 相关 AppStorage 写入改为 callback | ✅ | `NativeAbility.ets` | |
| MW5.5 | `menu.ets` fullscreen/recover 移除 AppStorage 直接写入 | ✅ | `menu.ets` | 改用 `getMenuDataCallback(windowKey)?.onMenubarVisible?.()` |
| MW5.6 | `ArkHelper.ets` 中 AppStorage 写入改为 callback | ✅ | `ArkHelper.ets` | 委托 `PredefinedActionExecutor` |
| MW5.7 | `popupFromJson` 中的 AppStorage 写入改为 callback | ✅ | `menu.ets` | `getMenuDataCallback(wid)?.onMenuPopup?.()` |
| MW5.8 | 清理不再使用的 AppStorage key 前缀 | ✅ | 全文件搜索 | 所有 `__openharmony_ability_menubar_*` / `__openharmony_ability_menu_*` 已清除 |
| MW5.9 | 编译验证: HAP 构建 | ✅ | — | 设备部署验证 136/146 pass (10 个非相关失败) |

**注意**: `@StorageProp → @State` 的读取方迁移在 MW-6 MenuBarComponent 提取时完成。MW-5 主要关注写入方（NativeAbility.ets、menu.ets、ArkHelper.ets）的改造。

**风险**: AppStorage 写入分散在多个文件中。需要全量搜索所有 `__openharmony_ability_` 前缀的 AppStorage 操作，确保无遗漏。

#### 验证

| 测试项 | 状态 | 验证内容 |
|--------|------|---------|
| 单窗口回归 | ✅ | 主窗口 menubar 渲染正常（hilog 确认） |
| menubar 更新 | ✅ | 动态修改菜单后 menubar 刷新 |
| menubar 显隐 | ⬚ | `set_visible` 后 menubar 正确显隐（待设备验证） |
| popup menu | ✅ | 右键菜单正常弹出 |
| 多窗口独立 | ⬚ | 两个窗口各自菜单数据不干扰（待设备验证） |
| AppStorage 无残留 | ✅ | 搜索确认无 `__openharmony_ability_menubar` 相关 AppStorage 操作 |

### MW-6: MenuBarComponent 完整提取 + FloatPage 支持

> **审计结论**: 采用 Approach C（完整提取），MenuBarComponent 包含所有 menu 功能：menubar 渲染、popup menu、RenderMenuItems、图标生命周期管理、accelerator matcher、handler 注册/清理。~280 行自包含组件。

| Step | 任务 | 状态 | 文件 | 备注 |
|------|------|------|------|------|
| MW6.1 | 新建 `MenuBarComponent.ets`，提取全部 menu 逻辑 | ✅ | `MenuBarComponent.ets` | ~280 行自包含组件 |
| MW6.2 | `@Prop windowId` + regular `acceleratorMatcher` property | ✅ | `MenuBarComponent.ets` | |
| MW6.3 | 7 个 `@StorageProp` (keyed to `::main`) → `@State` + callback | ✅ | `MenuBarComponent.ets` | 通过 `registerMenuDataCallback` 推送 |
| MW6.4 | `aboutToAppear` 初始化 menubar 数据 | ✅ | `MenuBarComponent.ets` | |
| MW6.5 | `aboutToDisappear` 释放 icon PixelMaps | ✅ | `MenuBarComponent.ets` | |
| MW6.6 | MainPage/FloatPage: `onKeyPreIme` 处理 accelerator + ESC 恢复 menubar | ✅ | `MainPage.ets` / `FloatPage.ets` | |
| MW6.7 | popup menu 内部 1×1 Column + `bindMenu` | ✅ | `MenuBarComponent.ets` | |
| MW6.8 | menubar 渲染条件 `menubarItems.length > 0 && menubarVisible` | ✅ | `MenuBarComponent.ets` | |
| MW6.9 | `MainPage.ets`: 移除全部 menu 逻辑 (~280 行)，改用 `MenuBarComponent({ windowId: 0 })` | ✅ | `MainPage.ets` | |
| MW6.10 | `FloatPage.ets`: 添加 `MenuBarComponent({ windowId: this.windowId })` | ✅ | `FloatPage.ets` | |
| MW6.11 | 编译验证: HAP 构建 | ✅ | — | |
| MW6.12 | 回归验证: 主窗口 menubar/popup/accelerator/icons | ⬚ | — | 待设备验证 |
| MW6.13 | 功能验证: 子窗口 menubar/popup/accelerator | ⬚ | — | 待设备验证 |

**注意**:
- MenuBarComponent 是**自包含**组件，不需要父组件管理 menu 状态
- `@Prop isDesktop` 由父组件从 `@StorageProp("__openharmony_ability_is_desktop__")` 传入
- popup menu 通过组件内部 1×1 Column + `bindMenu` 实现
- accelerator 通过 `onKeyPreIme` + 内部 `AcceleratorMatcher` 实例实现
- FloatPage 是否需要 menubar 取决于 tauri 是否为子窗口设置了菜单（menubarJson 为空时不渲染）

#### 验证

| 测试项 | 状态 | 验证内容 |
|--------|------|---------|
| MainPage 回归 | ⬚ | 提取 MenuBarComponent 后主窗口 menubar 正常（待设备验证） |
| FloatPage menubar 渲染 | ⬚ | 给子窗口设置菜单后，FloatPage 显示 menubar |
| FloatPage menubar 点击 | ⬚ | 子窗口 menubar 点击路由到子窗口 handler |
| FloatPage popup menu | ⬚ | 子窗口右键菜单在子窗口内弹出 |
| FloatPage accelerator | ⬚ | 子窗口内快捷键被正确拦截/处理 |
| FloatPage 无菜单时 | ⬚ | 未设置菜单的子窗口不显示 menubar |
| FloatPage 关闭清理 | ⬚ | 关闭子窗口后所有注册被清理 |
| Desktop/Mobile 条件 | ⬚ | `TAURI_OHOS_DEVICE_TYPE=mobile` 时 menubar 不渲染，popup 仍可用 |

---

## P3: NativeIcon 部分映射 (OHOS 系统图标)

> 设计文档 §3

### 问题

`set_native_icon()` 在 `IconMenuItem` 和 `Submenu` 上只有 `#[cfg(target_os = "macos")]` 分支 — OHOS 上是静默 no-op。`MenuChild::new_native_icon()` (`muda/src/platform_impl/ohos/mod.rs:246`) 接受 `_native_icon: Option<NativeIcon>` 但**丢弃它** (存储 `icon: None`)。

### OHOS 平台能力

- **Symbol icons**: `sys.symbol.*` 资源 (433 个矢量图标, API 12+)
- **Media icons**: `sys.media.*` 资源 (位图/矢量)

### 可映射变体 (~17/56)

Add, Home, Info, Folder, Network, GoLeft, GoRight, Refresh, LockLocked, LockUnlocked, Bluetooth, Computer, Share, TrashEmpty, TrashFull, User, StatusAvailable

### 不可映射变体 (~39/56)

ColorPanel, IChatTheater, FlowView, MobileMe, FollowLinkFreestanding 等 macOS 专属 UI 隐喻

### 实施步骤

| Step | 任务 | 状态 | 文件 | 备注 |
|------|------|------|------|------|
| N1 | `MenuChild` 添加 `native_icon: Option<String>` 字段 | ✅ | `muda/src/platform_impl/ohos/mod.rs` | |
| N2 | 创建 `native_icon_to_ohos(NativeIcon) -> Option<&'static str>` 映射函数 | ✅ | `muda/src/platform_impl/ohos/mod.rs` | 不可映射变体返回 `None` |
| N3 | `new_native_icon()` 存储映射值而非丢弃 | ✅ | `muda/src/platform_impl/ohos/mod.rs` | |
| N4 | 添加 `set_native_icon()` 方法到 OHOS `MenuChild` | ✅ | `muda/src/platform_impl/ohos/mod.rs` | |
| N5 | `MenuItemData` 添加 `native_icon: Option<String>` 字段 | ✅ | `openharmony-ability/crates/ability/src/menu/types.rs` | |
| N6 | `to_menu_item_data()` 序列化 `native_icon` | ✅ | `openharmony-ability/crates/ability/src/menu/types.rs` | |
| N7 | muda `IconMenuItem::set_native_icon()` 添加 OHOS 分支 | ✅ | `muda/src/items/icon.rs` | |
| N8 | muda `Submenu::set_native_icon()` 添加 OHOS 分支 | ✅ | `muda/src/items/submenu.rs` | |
| N9 | tauri `IconMenuItem::set_native_icon()` 添加 OHOS 分支 | ✅ | `tauri/crates/tauri/src/menu/icon.rs` | |
| N10 | tauri `Submenu::set_native_icon()` 添加 OHOS 分支 | ✅ | `tauri/crates/tauri/src/menu/submenu.rs` | |
| N11 | ArkTS `MenuItemData` 接口添加 `nativeIcon?: string` 字段 | ✅ | `menu_types.ets` | |
| N12 | ArkTS 渲染层处理 `nativeIcon` 字段 | ✅ | `MenuBarComponent.ets` RenderMenuItems | 使用 `symbolStartIcon` 或 `startIcon` |
| N13 | 编译验证: cargo check 全链 | ✅ | — | |
| N14 | 编译验证: HAP 构建 | ✅ | — | |

### 验证

| 测试项 | 状态 | 验证内容 |
|--------|------|---------|
| 映射函数单测 | ⬚ | 可映射变体返回正确的 OHOS 资源名（待设备验证） |
| 映射函数单测 — 不可映射 | ⬚ | 不可映射变体返回 `None`（待设备验证） |
| JSON 序列化 | ✅ | 含 native_icon 的菜单项 JSON 包含 `nativeIcon` 字段 |
| IconMenuItem set_native_icon | ⬚ | 调用后菜单项显示对应系统图标（待设备验证） |
| Submenu set_native_icon | ⬚ | 调用后子菜单项显示对应系统图标（待设备验证） |
| set_native_icon(None) | ⬚ | 调用后移除图标（待设备验证） |
| 不可映射变体 | ⬚ | 调用 set_native_icon 后不崩溃，静默 no-op（待设备验证） |

---

## 跳过的功能

| Feature | 原因 |
|---------|------|
| HideOthers / ShowAll | OHOS 无 "隐藏应用" 概念，当前 `enabled: false` 处理与 Windows/Linux 一致 |
| CHECK_ITEMS 全局状态 | 菜单 ID 由全局计数器生成，process-unique，跨窗口碰撞不可能 |

---

## 编译验证

| 目标 | 状态 |
|------|------|
| aarch64-unknown-linux-ohos (openharmony-ability) | ✅ |
| aarch64-unknown-linux-ohos (muda) | ✅ |
| aarch64-unknown-linux-ohos (tauri desktop) | ✅ |
| HAR 构建 | ✅ |
| HAP 签名+安装+启动 | ✅ |
| Windows 回归: cargo check -p tauri -p muda | ✅ |

---

## 手动测试

| 测试项 | 状态 | 验证内容 |
|--------|------|---------|
| Dark mode menubar | ✅ | API example app → Switch to dark mode → menubar 颜色适配 |
| NativeIcon 可映射 | ⬚ | 创建含 native icon 的菜单项，验证图标显示 |
| NativeIcon 不可映射 | ⬚ | 创建含不可映射 native icon 的菜单项，不崩溃 |
| 多窗口菜单点击路由 | ⬚ | 打开多个窗口，各自菜单点击路由正确 |
| 多窗口 MenuEvent 来源 | ⬚ | 不同窗口触发菜单事件，event 携带正确 windowId |
| 多窗口 popup 独立 | ⬚ | 两个窗口同时 popup 不干扰 |
| 子窗口 menubar | ⬚ | 给子窗口设置菜单后 menubar 渲染 |
| 子窗口 popup | ⬚ | 子窗口右键菜单在子窗口内弹出 |
| 子窗口 accelerator | ⬚ | 子窗口快捷键正常拦截 |
| 子窗口关闭清理 | ⬚ | 关闭子窗口后无残留 handler/controller |
| Phase 12 M1-M19 回归 | ⬚ | 使用 Phase 12 测试设计验证所有菜单操作 |
| Phase 13 回归 | ⬚ | 验证 predefined item (clipboard/tray) 不受影响 |

---

## 遇到的问题

| # | 问题 | 解决方案 |
|---|------|---------|
| 1 | clippy 警告: OHOS 分支 `set_accelerator` 返回 Result 未处理 | 在 `check.rs`、`icon.rs`、`normal.rs` 的 OHOS 分支添加 `let _ =` |
| 2 | clippy 警告: `window/mod.rs` 中 OHOS 早期返回导致尾部 `Ok(())` unreachable | 在 `hide_menu`/`show_menu`/`is_menu_visible` 尾部添加 `#[cfg(not(target_env = "ohos"))]` |
| 3 | clippy 警告: `tray-icon` 中 `MenuJsonItem`/`AboutMetadataJson` 字段标记 dead_code | 添加 `#[allow(dead_code)]`，字段仅用于 serde 反序列化 |
| 4 | hvigorfile.ts tauriPlugin 影响独立 HAP 构建 | 构建前禁用 `plugins:[tauriPlugin()]`，构建后恢复 |

---

## 关键实现决策记录

| # | 决策 | 原因 |
|---|------|------|
| 1 | Dark Mode 使用自定义颜色资源而非系统颜色 | 自定义 `$r('app.color.menubar_*')` 提供精确颜色控制，系统资源在不同设备/版本间可能不一致 |
| 2 | NativeIcon 映射 ~17/56 变体 | 混合使用 `sys.symbol.*` + `sys.media.ohos_ic_public_*`，其余 ~39 个是 macOS 专属 UI 隐喻 |
| 3 | 不可映射 NativeIcon 变体保持 no-op | 与 Windows/Linux 行为一致 |
| 4 | Per-window 使用 Map 而非 AppStorage | Map 提供真实隔离 + 窗口销毁自动清理，AppStorage 是全局单例 |
| 5 | MenuBarComponent 提取为自包含组件 (Approach C) | MainPage 和 FloatPage 共享全部 menu 逻辑（~280行），避免代码重复 |
| 6 | 回调注册模式 (registerMenuDataCallback) | 组件 aboutToAppear 注册，aboutToDisappear 清理，与 per-window handler 模式一致 |
| 7 | menuStateController 为死代码，清理而非 per-window 化 | grep 确认 import 后从未调用任何方法，popup 走完全独立的 menu.ets 路径 |
| 8 | Tray 只操作主窗口 | OHOS 单 tray 限制，Phase 13 已正确，无需修改 |
| 9 | fullscreen/recover 双重写入清理为单路径 | 移除 AppStorage 直接写入（路径 A），只保留 notifyMenubarVisibilityFn callback（路径 B） |
| 10 | TAURI_OHOS_DEVICE_TYPE 通过 @Prop 传入 MenuBarComponent | 设备级别标志，不需要 per-window 隔离，menubar 仅在 desktop 模式渲染 |
| 11 | MenuDataCallback 接口定义在 MenuBarComponent.ets 中 | 与 callback registry 共存于同一文件，减少跨文件依赖 |
| 12 | clippy 警告修复：`let _ =` 忽略 set_accelerator Result | OHOS 分支中 set_accelerator 返回 Result 但不需要传播，与 Windows/macOS 分支行为一致 |

---

## 代码量估算

| 层 | 行数估算 | 来源 |
|----|---------|------|
| Rust (tao platform_impl/ohos) | ~30 | D1: set_theme 方法 + theme 状态 + NAPI 调用 |
| Rust (tao window.rs) | ~5 | D1b: 移除 guard |
| Rust (openharmony-ability NAPI) | ~20 | D2: set_color_mode |
| Rust (muda ohos/mod.rs) | ~40 | N1-N4: native_icon 字段 + 映射函数 + set_native_icon |
| Rust (muda items/icon.rs + submenu.rs) | ~10 | N7-N8: OHOS 分支 |
| Rust (tauri menu/icon.rs + submenu.rs) | ~10 | N9-N10: OHOS 分支 |
| Rust (openharmony-ability types.rs) | ~10 | N5-N6: native_icon 字段 |
| ArkTS (MenuBarComponent.ets 新建) | ~280 | MW6: 从 MainPage 提取全部 menu 逻辑 |
| ArkTS (MainPage.ets 简化) | ~-250 | MW6: 移除 menu 逻辑，仅保留 Native Module + WebView |
| ArkTS (FloatPage.ets 增强) | ~20 | MW6: 添加 MenuBarComponent + isDesktop prop |
| ArkTS (menu.ets MenuManager) | ~15 | MW2+MW3: windowId 存储+转发 |
| ArkTS (menu.ets callback registry) | ~25 | MW5: menuDataCallbacks + MenuDataCallback 接口 |
| ArkTS (menu.ets fullscreen/recover) | ~-6 | MW5: 移除 AppStorage 双重写入 |
| ArkTS (menu_state.ets 清理) | ~-15 | MW4: 删除死代码 |
| ArkTS (NativeAbility.ets) | ~20 | MW5: onMenuRequest 改用 callback + 移除 menuStateController import |
| ArkTS (ArkHelper.ets) | ~-5 | MW5: AppStorage 写入改为 callback |
| ArkTS (MenuBarComponent 颜色) | ~20 | D4-D8: 硬编码颜色替换 |
| ArkTS (color.json × 2) | ~20 | D3: base + dark 颜色资源 |
| ArkTS (清理 AppStorage 残留) | ~-10 | MW5: 删除分散的 AppStorage 写入 |
| **合计** | **~240** (新增 ~495 + 清理 ~286) | |

# Phase 8: Menu & Tray OHOS 能力缺口审计与替代方案 — Progress

> 对应设计文档：[phase8-gap-analysis-design.md](./phase8-gap-analysis-design.md)
> 创建时间：2026-05-23
> 更新时间：2026-05-23

---

## 一、进度概览

| 任务 | 状态 | 说明 |
|------|------|------|
| 设计文档 | ✅ done | phase8-gap-analysis-design.md 已创建并审计 |
| Scope 确认 | ✅ done | 明确为 tauri::menu (Menu/MenuEvent/MenuItem/PredefinedMenuItem/Submenu) + tauri::tray (TrayIconBuilder/TrayIconEvent/MouseButton/MouseButtonState) |
| Predefined 两侧一致性审计 | ✅ done | 确认 menu 和 tray 两条调用链最终都走同一个 ArkTS PredefinedActionExecutor，一处修改覆盖两侧 |
| rect() 近似值合理性验证 | ✅ done | 与 Windows (Shell_NotifyIconGetRect 精确 tray icon RECT) + macOS (NSStatusItem.button.window.frame 精确 rect) 对比，AvoidArea.topRect 语义偏差太大，决定保持 None |
| 8.1 Accelerator 序列化 | ✅ done | `accelerator: None` → `self.accelerator.map(|k| k.to_string())` |
| 8.2 TrayIconEvent fallback | ✅ done | `_ => todo!()` → Click fallback + log::warn |
| 8.3 About AlertDialog | ✅ done | MenuItemData.aboutMetadata + PredefinedActionExecutor.setShowAboutFn + showAlertDialog |
| 8.4 CloseWindow BUG (menu+tray) | ✅ done | `close = minimize` → `close = quit (terminateSelf)`; tray side also fixed |
| 8.5 Fullscreen 分离 (menu+tray) | ✅ done | 分离 fullscreen (沉浸式) vs maximize (桌面全屏); display width > 800 检测 |
| 8.6 rect() 保持 None | ✅ done | 仅文档注释，代码不改 |
| 8.7 Menu Bar (desktop) | → Phase 10 | 设计文档 [phase10-menubar-design.md](../../menu/impl/phase10-menubar-design.md) |
| 8.8-8.14 不可实现项文档注明 | ✅ done | TrayIconEvent/MouseButton/MouseButtonState/NativeIcon/rect 注释 |
| Rust unit test | ✅ done | 4 个新增 (muda ohos tests) + TrayIconEvent fallback test |
| JS auto test | ✅ done | TrayIcon.rect_returns_none |
| 手动测试 | ⬜ pending | 9 个手动验证项 |
| OHOS 构建验证 | ⬜ pending | cargo clippy + HAP 构建 + 设备部署 |
| Windows 构建验证 | ✅ done | cargo check 通过 (tauri, muda, tray-icon, openharmony-ability) |

**整体进度**：`70%` — Steps 1-5, 7-10 已完成，Step 6 (Menu Bar) deferred

---

## 二、设计审计结果

### 2.1 Predefined Actions 一致性审计

**关键发现**：menu 和 tray 两条调用链最终都调用同一个 ArkTS `PredefinedActionExecutor`：

| 来源 | 调用链 | 最终执行 |
|------|-------|---------|
| Menu popup | MenuPopup.ets → `MenuManager.handleItemClick` → `PredefinedActionExecutor.execute(type)` | ArkTS 侧直接执行 |
| Tray menu | tray-icon Rust `execute_predefined_action` → TSFN → `openharmony_ability::execute_predefined_action` → ArkTS `PredefinedActionExecutor.execute(type)` | 同一个 ArkTS executor |

**影响**：
- `close = minimize` BUG 在 menu 和 tray **两侧都存在**
- `fullscreen = maximize` BUG 在 menu 和 tray **两侧都存在**
- 修复一处 ArkTS `PredefinedActionExecutor` 即覆盖两侧

**例外**：`quit` 在 tray Rust 侧直接 `app.exit(0)`（不走 TSFN/ArkTS），menu ArkTS 侧 `exitFn(0)`。效果相同，无需统一。

### 2.2 rect() 近似值验证

| 平台 | 实现方式 | 返回值语义 | 精确度 |
|------|---------|-----------|--------|
| **Windows** | `Shell_NotifyIconGetRect()` | tray icon **精确** RECT (位置+尺寸) | 精确 |
| **macOS** | `NSStatusItem.button().window().frame()` + scale_factor | tray icon button **精确** Rect | 精确 |
| **OHOS (AvoidArea.topRect)** | `getWindowAvoidArea(TYPE_SYSTEM).topRect` | 整个状态栏区域 `{left, top, width, height}` px | 近似，但语义偏差大 |

**结论**：AvoidArea.topRect 返回整个状态栏区域 (如 `{0,0,1440,48}`)，不是 tray icon 本身 (约 36×36 px)。与 Windows/macOS 返回的精确 tray icon rect 不可比。误导调用者用于 popup 定位或尺寸计算。**保持 None，与 Linux 行为对齐。**

### 2.3 Scope 确认

审计范围严格限定为 Rust 侧公开类型：

**tauri::menu**：Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu
**tauri::tray**：TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState

底层 crate (muda, tray-icon, openharmony-ability) 的改动是实现支撑。

所有在范围内的项目均已覆盖：5 项可实现 + 1 项有替代(保持 None) + 1 项 desktop Menu Bar + 7 项不可实现(文档注明)。

---

## 三、实现步骤

### Step 1: P0 — Accelerator 序列化 (8.1)

**文件**：`muda/src/platform_impl/ohos/mod.rs`
**改动**：
- `to_menu_item_data()` 第 267 行: `accelerator: None` → `accelerator: self.accelerator.map(|k| k.to_string())`
- Submenu `new_submenu()` 中 accelerator 初始化为传入值（当前硬编码 None）

**验证**：
- Rust unit test: `to_menu_item_data_accelerator`
- Rust unit test: `submenu_accelerator_preserved`
- JS auto test: `MenuItem.setAccelerator` (已有)
- 手动: popup 菜单观察快捷键文本

### Step 2: P0 — TrayIconEvent fallback (8.2)

**文件**：`tauri/crates/tauri/src/tray/mod.rs`
**改动**：
- 第 196 行 `_ => todo!()` → Click fallback + `log::warn!`

**验证**：
- Rust unit test: `TrayIconEvent_from_no_panic`

### Step 3: P1 — CloseWindow BUG 修复 (8.4)

**文件**：`openharmony-ability/native_ability/src/main/ets/helper/predefined.ets`
**改动**：
- `case 'close': case 'destroyWindow': await this.win?.minimize();` → `await this.context?.terminateSelf();`
- 一处修改覆盖 menu 和 tray 两侧

**验证**：
- 手动: popup menu 含 CloseWindow → 点击 → 应用退出
- 手动: tray menu 含 CloseWindow → 点击 → 应用退出
- ⚠️ 需确认 UX: close = quit (与 Windows/macOS Close 行为一致)

### Step 4: P1 — Fullscreen 分离 (8.5)

**文件**：`openharmony-ability/native_ability/src/main/ets/helper/predefined.ets`
**改动**：
- `case 'maximize': case 'fullscreen': await this.win?.maximize();` → 分离:
  - `maximize` → `this.win?.maximize()`
  - `fullscreen` → desktop: `this.win?.maximize(MaximizePresentation.ENTER_IMMERSIVE)`, mobile: `this.win?.setWindowLayoutFullScreen(true)` + `setWindowSystemBarEnable([])`
- 一处修改覆盖 menu 和 tray 两侧

**验证**：
- 手动: fullscreen 进入沉浸式 vs maximize 进入桌面全屏
- 手动: tray menu fullscreen/maximize 行为一致

### Step 5: P2 — About AlertDialog (8.3)

**文件**：
- `muda/src/platform_impl/ohos/mod.rs`: `to_menu_item_data()` 传递 aboutMetadata
- `openharmony-ability/native_ability/src/main/ets/helper/menu_types.ets`: MenuItemData 添加 aboutMetadata 字段
- `openharmony-ability/native_ability/src/main/ets/helper/menu.ets`: `PredefinedActionExecutor` about case → showAlertDialog

**改动**：
- Rust: MenuChild `to_menu_item_data()` 添加 `about_metadata` 字段序列化
- ArkTS: MenuItemData 添加 `aboutMetadata?: AboutMetadataData`
- ArkTS: about case 调用 `showAlertDialog` 显示应用信息

**验证**：
- Rust unit test: `about_metadata_in_menu_item_data`
- JS auto test: `PredefinedMenuItem.about` (已有)
- 手动: popup menu 含 About → 点击 → AlertDialog 弹出

### Step 6: P2 — Menu Bar desktop (8.7)

**文件**：
- `tauri/crates/tauri/src/menu/` (Rust): Menu set_menu OHOS 分支增加 menubar 数据序列化 + TSFN
- `openharmony-ability/native_ability/src/main/ets/components/MainPage.ets`: desktop 模式顶部菜单条渲染
- ArkTS: MenuBarManager 或扩展 MenuManager

**改动**：
- Rust: `Menu::set_menu()` OHOS desktop 分支序列化顶级菜单条数据 → TSFN 传给 ArkTS
- ArkTS: `MainPage.ets` 条件渲染顶部 Row 菜单条 (仅 desktop)
- ArkTS: 点击 ToolBarItem → popup 下拉 (使用现有 popupFromJson)

**验证**：
- 手动: desktop 模式 → 顶部菜单条 → 点击 → 下拉菜单

### Step 7: 文档注释 — rect/不可实现项 (8.6 + 8.8-8.14)

**文件**：
- `tauri/crates/tauri/src/tray/mod.rs`: TrayIconEvent From impl 注释 + rect() 注释
- `tray-icon/src/platform_impl/ohos/event.rs`: 文件顶部 StatusBar API 限制注释
- `tray-icon/src/platform_impl/ohos/mod.rs`: rect() 方法注释
- `tauri/crates/tauri/src/menu/mod.rs`: Menu OHOS 注释

**改动**：仅注释，不改代码

**验证**：
- JS auto test: `TrayIcon.rect_returns_none` (新增)
- Rust unit test: 已有的 `convert_icon_click` / `convert_menu_click` 覆盖 MouseButton/MouseButtonState/position

---

## 四、测试计划

### 4.1 新增 Rust unit test (4 个)

| 测试项 | 文件 | 验证内容 |
|--------|------|----------|
| `to_menu_item_data_accelerator` | `muda/src/platform_impl/ohos/mod.rs` | MenuChild 带 accelerator → `to_menu_item_data()` → `accelerator` 字段非 None，值为 `"Ctrl+O"` |
| `submenu_accelerator_preserved` | `muda/src/platform_impl/ohos/mod.rs` | Submenu → `set_key_accelerator("Ctrl+S")` → `to_menu_item_data()` → `accelerator` 非 None |
| `about_metadata_in_menu_item_data` | `muda/src/platform_impl/ohos/mod.rs` | MenuChild::new_predefined(About(metadata)) → `to_menu_item_data()` → `aboutMetadata` 字段存在且包含 name/version |
| `TrayIconEvent_from_no_panic` | `tauri/crates/tauri/src/tray/mod.rs` | 未知 variant → From impl → Click fallback（不 panic），log::warn 被触发 |

### 4.2 新增 JS auto test (1 个)

| 测试项 | 模块 | 验证内容 |
|--------|------|----------|
| `TrayIcon.rect_returns_none` | tray | 调用 `tray.rect()` → 验证返回 `null` |

### 4.3 手动测试 (9 个)

| 测试项 | 模块 | 验证内容 | 只能手动的原因 |
|--------|------|----------|--------------|
| `Menu.popup_with_accelerator` | menu | popup → 观察快捷键文本 | UI 文本渲染 |
| `PredefinedMenuItem.about_dialog` | menu | About → AlertDialog 弹出 | UI 弹窗 |
| `PredefinedMenuItem.closeWindow_exit` | menu | CloseWindow → 应用退出 | 进程终止 |
| `TrayMenu.closeWindow_exit` | tray | tray menu CloseWindow → 退出 | 进程终止 |
| `PredefinedMenuItem.fullscreen_immersive` | menu | Fullscreen → 沉浸式全屏 | 窗口状态 |
| `TrayMenu.fullscreen_immersive` | tray | tray menu Fullscreen → 沉浸式 | 窗口状态 |
| `PredefinedMenuItem.maximize_restore` | menu | Maximize → 恢复 | 窗口状态 |
| `TrayIcon.click_no_panic` | tray | 点击 tray → 无 panic | 需设备交互 |
| `MenuBar.desktop_visible` | menu | desktop → 顶部菜单条 + 下拉 | UI 布局 |

---

## 五、风险与待确认

| 项目 | 风险 | 待确认 |
|------|------|--------|
| 8.4 CloseWindow | close = quit 可能不符合某些应用预期 | ⚠️ 需确认 UX: close=quit 或 close=minimize |
| 8.5 Fullscreen | mobile/desktop 行为分支需设备类型检测 | TAURI_OHOS_DEVICE_TYPE 环境变量在 ArkTS 侧如何获取？ |
| 8.7 Menu Bar | Navigation.toolbar 渲染逻辑复杂度 | desktop 模式判断 + ArkUI 组件层级 |
| 8.3 About | MenuItemData 序列化扩展 | aboutMetadata 字段大小限制？icon 是否可传？ |

---

## 六、Log

### 2026-05-23

- 完成设计文档 phase8-gap-analysis-design.md
- 完成 Scope 确认：限定为 tauri::menu + tauri::tray 公开类型
- 完成 Predefined 两侧一致性审计：menu 和 tray 走同一个 ArkTS executor
- 完成 rect() 近似值合理性验证：与 Windows/macOS 对比，AvoidArea 不可用，保持 None
- 更新设计文档：rect() 从"返回 AvoidArea 近似值"改为"保持 None + 注释说明"
- 更新设计文档：8.7 Menu Bar 从"留作 Phase 9"改为"Phase 8 实现"
- 更新设计文档：8.4/8.5 标注"menu+tray 两侧"
- 更新验证策略：新增自动测试能力分析表 + 4 Rust unit test + 1 JS auto test
- 创建 progress 文档

- **实现代码改动**：
  - 8.1: muda ohos `to_menu_item_data()` — `accelerator: None` → `self.accelerator.map(|k| k.to_string())`
  - 8.2: tauri tray `From` impl — `_ => todo!()` → Click fallback + `log::warn!`
  - 8.4: menu.ets `PredefinedActionExecutor` — `close: minimize` → `close: this.exitFn(0)` (quit); DefaultXComponent.ets tray side `close: minimize` → `close: exit(0)`
  - 8.5: menu.ets — 分离 `fullscreen` vs `maximize`; DefaultXComponent.ets tray side 同样分离; display width > 800 检测
  - 8.3: openharmony-ability MenuItemData + AboutMetadataData; menu.ets PredefinedActionExecutor.setShowAboutFn; NativeAbility.ets UIContext.showAlertDialog; DefaultXComponent.ets tray side about = showAlertDialog; muda ohos to_menu_item_data() 传递 about_metadata
  - 8.6: tray/mod.rs rect() 注释; tray-icon event.rs 文件顶部注释; tray-icon mod.rs rect() 注释; menu/menu.rs 文件顶部注释
  - 8.8-8.14: tray/mod.rs TrayIconEvent From impl 注释
  - Rust unit tests: muda ohos 4 个; tauri tray TrayIconEvent_from_fallback_no_panic
  - JS auto test: tray.ts TrayIcon.rect_returns_none
  - Windows 构建验证: cargo check 通过 (tauri, muda, tray-icon, openharmony-ability)
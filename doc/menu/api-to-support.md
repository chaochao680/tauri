# Menu 模块 API 支持清单

> 基于 tauri/crates/tauri/src/menu 完整定义
> 更新时间：2026-05-14

---

## 外部依赖库

### Desktop 平台依赖

所有 Menu API 在 Windows/macOS/Linux 上依赖 **muda** crate：

| 平台 | 依赖库 | 用途 |
|------|--------|------|
| **Windows** | `windows-sys` (Win32 API) | HMENU, CreateMenu, TrackPopupMenu, WM_COMMAND |
| **macOS** | `objc2-app-kit` (NSMenu, NSMenuItem) | Cocoa 菜单 API |
| **Linux** | `gtk` (GTK3) + `libxdo` (xdotool) | GTK Menu + 剪贴板操作 |

### muda Cargo.toml 配置

```toml
# Windows
[target.'cfg(target_os = "windows")'.dependencies.windows-sys]
version = ">=0.60, <=0.61"
features = [
  "Win32_UI_WindowsAndMessaging",  # CreateMenu, TrackPopupMenu
  "Win32_UI_Controls",             # MENUITEMINFOW
  "Win32_UI_Input_KeyboardAndMouse", # SendInput (剪贴板)
]

# macOS
[target.'cfg(target_os = "macos")'.dependencies]
objc2-app-kit = { version = "0.3.0", features = [
  "NSMenu",      # 菜单容器
  "NSMenuItem",  # 菜单项
  "NSImage",     # 图标
  "NSApplication", # 全局菜单
] }

# Linux
[target.'cfg(target_os = "linux")'.dependencies]
gtk = { version = "0.18" }      # GTK Menu
libxdo = { version = "0.6.0" }  # xdotool (剪贴板模拟)
```

### OHOS 平台依赖

OHOS **不使用 muda**，使用原生 ArkUI Menu API：

| API | OHOS 原生依赖 | 封装位置 |
|------|--------------|---------|
| Menu | ArkUI `Menu()` 组件 | `openharmony-ability/menu.ets` |
| MenuItem | ArkUI `MenuItem()` 组件 | `openharmony-ability/menu.ets` |
| Submenu | ArkUI `MenuItem.builder` | `openharmony-ability/menu.ets` |
| PredefinedMenuItem | ArkUI + WebView + Window | `openharmony-ability/predefined.ets` + `window_ops.ets` |
| popup | ArkUI `bindContextMenu(isShown)` | `openharmony-ability/menu_state.ets` |

---

## 各平台底层 API 映射

### Windows (Win32 API)

| muda 类型 | Win32 API | 说明 |
|-----------|----------|------|
| `Menu` | `HMENU`, `CreateMenu()`, `CreatePopupMenu()` | 菜单句柄 |
| `MenuItem` | `AppendMenuW()`, `MENUITEMINFOW` | 菜单项 |
| `Submenu` | `MF_POPUP` flag | 子菜单 |
| `CheckMenuItem` | `MFS_CHECKED`, `MIIM_STATE` | 勾选状态 |
| `IconMenuItem` | `MIIM_BITMAP`, `HBITMAP` | 图标 |
| `PredefinedMenuItem.copy/cut/paste` | `SendInput()` + `VK_CONTROL` | 模拟键盘 |
| `PredefinedMenuItem.minimize/maximize` | `ShowWindow()` + `SW_MINIMIZE/SW_MAXIMIZE` | 窗口操作 |
| `PredefinedMenuItem.quit` | `PostQuitMessage()` | 退出 |
| `ContextMenu.popup()` | `TrackPopupMenu()` | 弹出菜单 |
| `MenuEvent` | `WM_COMMAND` | 菜单事件 |
| `Accelerator` | `CreateAcceleratorTableW()`, `HACCEL` | 快捷键 |

### macOS (Objective-C API)

| muda 类型 | Objective-C 类 | 说明 |
|-----------|----------------|------|
| `Menu` | `NSMenu` | 菜单容器 |
| `MenuItem` | `NSMenuItem` | 菜单项 |
| `Submenu` | `NSMenuItem` + `setSubmenu:` | 子菜单 |
| `CheckMenuItem` | `NSMenuItem.state` (NSControlStateValueOn/Off) | 勾选状态 |
| `IconMenuItem` | `NSMenuItem.image` (`NSImage`) | 图标 |
| `PredefinedMenuItem.copy/cut/paste` | `NSApplication.sendAction:` | 应用级命令 |
| `PredefinedMenuItem.minimize/maximize` | `NSWindow.minimize()/maximize()` | 窗口操作 |
| `PredefinedMenuItem.quit` | `NSApplication.terminate:` | 退出 |
| `PredefinedMenuItem.about` | `NSApplication.orderFrontStandardAboutPanelWithOptions:` | About 面板 |
| `PredefinedMenuItem.hide/show_all` | `NSApplication.hide:/unhideAllApplications:` | 应用隐藏 |
| `ContextMenu.popup()` | `NSMenu.popUpMenuPositioningItem:` | 弹出菜单 |
| `NativeIcon` | `NSImageName` (60+ 系统图标) | 原生图标 |
| `MenuEvent` | `NSMenuItem.action` → callback | 菜单事件 |
| `Accelerator` | `NSMenuItem.keyEquivalent` + `modifierMask` | 快捷键 |

### Linux (GTK API)

| muda 类型 | GTK 类型 | 说明 |
|-----------|----------|------|
| `Menu` | `Gtk.Menu`, `Gtk.MenuBar` | 菜单容器 |
| `MenuItem` | `Gtk.MenuItem` | 菜单项 |
| `Submenu` | `Gtk.MenuItem` + `set_submenu()` | 子菜单 |
| `CheckMenuItem` | `Gtk.CheckMenuItem` | 勾选菜单项 |
| `IconMenuItem` | `Gtk.MenuItem` + `Gtk.Image` | 图标菜单项 |
| `PredefinedMenuItem.copy/cut/paste/select_all` | `libxdo` (xdotool) | 模拟键盘 |
| `PredefinedMenuItem.about` | `Gtk.AboutDialog` | About 对话框 |
| `ContextMenu.popup()` | `Gtk.Menu.popup_at_pointer()` | 弹出菜单 |
| `MenuEvent` | `Gtk.MenuItem.activate` signal | 菜单事件 |
| `Accelerator` | `Gtk.AccelGroup` | 快捷键 |

### OHOS (ArkUI API)

| muda 类型 | OHOS API | 说明 |
|-----------|----------|------|
| `Menu` | `Menu()` | 菜单容器 (API 9+) |
| `MenuItem` | `MenuItem()` | 菜单项 (API 9+) |
| `Submenu` | `MenuItem.builder` | 子菜单 (API 9+) |
| `CheckMenuItem` | `MenuItem.selected` + `onChange` | 勾选状态 (API 9+) |
| `IconMenuItem` | `MenuItem.startIcon` / `symbolStartIcon` | 图标 (API 9+/12+) |
| `PredefinedMenuItem.copy/cut/paste` | `runJavaScript('execCommand("xxx")')` | WebView 剪贴板 |
| `PredefinedMenuItem.minimize/maximize` | `window.minimize()/maximize()` | 窗口操作 (API 12+) |
| `PredefinedMenuItem.quit` | `context.terminateSelf()` | 退出 (API 9+) |
| `ContextMenu.popup()` | `bindContextMenu(isShown)` | 状态变量弹出 (API 12+) |
| `MenuEvent` | `emit_menu_event()` NAPI | 菜单事件 |
| `Accelerator` | `MenuItem.labelInfo` | 快捷键显示 (非全局监听) |

### Tray 模块依赖 (tray-icon crate)

> Tray 模块使用独立的 `tray-icon` crate，同样不用于 OHOS

| 平台 | 依赖库 | 用途 |
|------|--------|------|
| **Windows** | `windows-sys` (Shell_NotifyIconW) | 系统托盘 API |
| **macOS** | `objc2-app-kit` (NSStatusItem) | 状态栏 API |
| **Linux** | `libappindicator` (AppIndicator) | AppIndicator 托盘 |

### tray-icon Cargo.toml 配置

```toml
# tray-icon 依赖 muda (用于托盘菜单)
[dependencies]
muda = { version = "0.19.1", default-features = false }

# Windows
[target."cfg(target_os = \"windows\")".dependencies.windows-sys]
features = [
  "Win32_UI_Shell",              # Shell_NotifyIconW, NOTIFYICONDATAW
  "Win32_UI_WindowsAndMessaging", # WM_USER 消息, TrackPopupMenu
]

# macOS
[target."cfg(target_os = \"macos\")'.dependencies]
objc2-app-kit = { version = "0.3.0", features = [
  "NSStatusBar",     # 状态栏
  "NSStatusItem",    # 托盘项
  "NSStatusBarButton", # 托盘按钮
  "NSMenu",          # 托盘菜单 (依赖 muda)
] }

# Linux
[target."cfg(target_os = \"linux\")'.dependencies]
libappindicator = { version = "0.9" }  # AppIndicator 托盘
```

### Tray 各平台底层 API 映射

| tray-icon 类型 | Windows | macOS | Linux |
|----------------|---------|-------|-------|
| `TrayIcon` | `Shell_NotifyIconW()` | `NSStatusItem` | `AppIndicator` |
| `set_icon()` | `NIM_MODIFY` + `NIF_ICON` | `NSStatusItem.button.image` | `set_icon_full()` |
| `set_menu()` | `TrackPopupMenu()` | `NSStatusItem.menu` | `AppIndicator.set_menu()` |
| `set_tooltip()` | `NIF_TIP` | `NSStatusItem.button.toolTip` | ❌ 不支持 |
| `set_title()` | ❌ 不支持 | `NSStatusItem.button.title` | `set_label()` |
| `TrayIconEvent.Click` | `WM_LBUTTONUP/WM_RBUTTONUP` | `NSStatusItem.button.action` | ❌ 仅菜单点击 |

---

## 模块总览

Tauri menu 模块由以下 6 个核心类型组成：

```
tauri::menu 模块
    │
    ├── Menu                    # 菜单容器（见 §一）
    │       └── 管理 MenuItem 列表
    │       └── 实现 ContextMenu trait (popup)
    │
    ├── MenuItem                # 普通菜单项（见 §二）
    │       └── 文本 + 启用状态 + 快捷键
    │
    ├── Submenu                 # 子菜单（见 §三）
    │       └── 嵌套 MenuItem 列表
    │       └ 实现 ContextMenu trait (popup)
    │
    ├── PredefinedMenuItem      # 预定义菜单项（见 §四）
    │       └── separator, copy, cut, paste, minimize, quit 等
    │
    ├── CheckMenuItem           # 勾选菜单项（见 §五）
    │       └ 文本 + 启用状态 + 勾选状态 + 快捷键
    │
    ├── IconMenuItem            # 图标菜单项（见 §六）
    │       └ 文本 + 启用状态 + 图标 + 快捷键
    │
    ├── MenuEvent               # 菜单事件（见 §七）
    │       └ 菜单项点击事件
    │
    └── MenuItemKind            # 菜单项枚举（见 §八）
    │       └ MenuItem / Submenu / Predefined / Check / Icon
    │
    └── AboutMetadata           # About 元数据（见 §九）
    │       └ 应用信息（名称、版本、版权等）
    │
    └── NativeIcon              # 原生图标枚举（见 §十）
            └── macOS only
```

### 类型职责

| 类型 | 职责 | OHOS 适配状态 |
|------|------|---------------|
| **Menu** | 菜单容器，管理菜单项列表 | ✓ 可适配（见 §一） |
| **MenuItem** | 普通菜单项（文本+快捷键） | ✓ 可适配（见 §二） |
| **Submenu** | 子菜单（嵌套菜单项） | ✓ 可适配（见 §三） |
| **PredefinedMenuItem** | 预定义菜单项（copy/quit等） | △ 部分适配（见 §四） |
| **CheckMenuItem** | 勾选菜单项 | △ 部分适配（见 §五） |
| **IconMenuItem** | 图标菜单项 | △ 部分适配（见 §六） |
| **MenuEvent** | 菜单事件 | ✓ 可适配（见 §七） |
| **MenuItemKind** | 菜单项枚举 | ✓ 可适配（见 §八） |
| **AboutMetadata** | About 元数据 | △ 需自定义（见 §九） |
| **NativeIcon** | 原生图标 | ✗ macOS only（见 §十） |

### 模块依赖关系

```
Menu / Submenu
    │
    │ append/prepend/insert/remove
    │
    ▼
MenuItemKind
    │
    ├──► MenuItem (文本)
    │
    ├──► Submenu (嵌套)
    │
    ├──► PredefinedMenuItem (预定义)
    │
    ├──► CheckMenuItem (勾选)
    │
    └────► IconMenuItem (图标)
    │
    │ 用户点击
    │
    ▼
MenuEvent
    │
    │ id: MenuId
    │
    └──► AppHandle.on_menu_event()
```

---

## 一、Menu

### 1.1 创建方法

| API | OHOS 支持 | 说明 |
|-----|-----------|------|
| `new(manager)` | ✓ | 创建菜单 |
| `with_id(manager, id)` | ✓ | 带指定 ID 创建 |
| `with_items(manager, items)` | ✓ | 带菜单项创建 |
| `with_id_and_items(manager, id, items)` | ✓ | 带 ID 和菜单项创建 |
| `default(app_handle)` | △ | 默认菜单（macOS 风格，OHOS 需简化） |

### 1.2 菜单项管理

| API | OHOS 支持 | 说明 |
|-----|-----------|------|
| `append(item)` | ✓ | 追加菜单项 |
| `append_items(items)` | ✓ | 追加多个菜单项 |
| `prepend(item)` | ✓ | 前置菜单项 |
| `prepend_items(items)` | ✓ | 前置多个菜单项 |
| `insert(item, position)` | ✓ | 插入菜单项 |
| `insert_items(items, position)` | ✓ | 插入多个菜单项 |
| `remove(item)` | ✓ | 移除菜单项 |
| `remove_at(position)` | ✓ | 移除指定位置菜单项 |
| `get(id)` | ✓ | 按 ID 获取菜单项 |
| `items()` | ✓ | 获取所有菜单项 |

### 1.3 ContextMenu 实现（popup）

| API | OHOS 支持 | 说明 |
|-----|-----------|------|
| `popup(window)` | ✓ | 在窗口光标位置弹出 |
| `popup_at(window, position)` | ✓ | 在指定位置弹出（bindContextMenu + offset/anchorPosition） |
| `hpopupmenu()` | ✗ | **Windows only**，获取 HMENU |

### 1.4 应用/窗口菜单

| API | OHOS 支持 | 说明 |
|-----|-----------|------|
| `set_as_app_menu()` | ✗ | **OHOS 不支持**：无全局菜单栏 |
| `set_as_window_menu(window)` | ✗ | **OHOS 不支持**：无窗口菜单栏 |

### 1.5 其他方法

| API | OHOS 支持 | 说明 |
|-----|-----------|------|
| `id()` | ✓ | 获取菜单 ID |
| `app_handle()` | ✓ | 获取 AppHandle |
| `inner()` | ✓（内部） | 获取 muda::Menu |

---

## 二、MenuItem

### 2.1 创建方法

| API | OHOS 支持 | 说明 |
|-----|-----------|------|
| `new(manager, text, enabled, accelerator)` | ✓ | 创建菜单项 |
| `with_id(manager, id, text, enabled, accelerator)` | ✓ | 带指定 ID 创建 |

### 2.2 属性方法

| API | OHOS 支持 | 说明 |
|-----|-----------|------|
| `id()` | ✓ | 获取菜单项 ID |
| `text()` | ✓ | 获取文本 |
| `set_text(text)` | ✓ | 设置文本 |
| `is_enabled()` | ✓ | 获取启用状态 |
| `set_enabled(enabled)` | ✓ | 设置启用状态 |
| `set_accelerator(accelerator)` | ✓ | 设置快捷键 |
| `app_handle()` | ✓ | 获取 AppHandle |

### 2.3 OHOS MenuItem 扩展属性

> OHOS MenuItem 支持额外属性（参考 menu_item.md）

| 属性 | OHOS 支持 | API 版本 | 说明 |
|------|----------|---------|------|
| `startIcon` | ✓ | API 9+ | 开始图标 |
| `endIcon` | ✓ | API 9+ | 结束图标 |
| `symbolStartIcon` | ✓ | API 12+ | Symbol 开始图标 |
| `symbolEndIcon` | ✓ | API 12+ | Symbol 结束图标 |
| `selected` | ✓ | API 9+ | 是否选中 |
| `selectIcon` | ✓ | API 9+ | 选中时显示的图标 |

### 2.4 OHOS MenuItem 扩展事件

| 事件 | OHOS 支持 | API 版本 | 说明 |
|------|----------|---------|------|
| `onClick` | ✓ | API 9+ | 点击事件 |
| `onChange` | ✓ | API 9+ | 选中状态变化事件（selected 菜单项） |

---

## 三、Submenu

### 3.1 创建方法

| API | OHOS 支持 | 说明 |
|-----|-----------|------|
| `new(manager, text, enabled)` | ✓ | 创建子菜单 |
| `new_with_icon(manager, text, enabled, icon)` | △ | 带图标创建（OHOS 支持 startIcon） |
| `new_with_native_icon(manager, text, enabled, native_icon)` | ✗ | **macOS only** |
| `with_id(manager, id, text, enabled)` | ✓ | 带指定 ID 创建 |
| `with_id_and_icon(manager, id, text, enabled, icon)` | △ | 带 ID 和图标创建 |
| `with_id_and_native_icon(manager, id, text, enabled, native_icon)` | ✗ | **macOS only** |
| `with_items(manager, text, enabled, items)` | ✓ | 带菜单项创建 |
| `with_id_and_items(manager, id, text, enabled, items)` | ✓ | 带 ID 和菜单项创建 |

### 3.2 菜单项管理

| API | OHOS 支持 | 说明 |
|-----|-----------|------|
| `append(item)` | ✓ | 追加菜单项 |
| `append_items(items)` | ✓ | 追加多个菜单项 |
| `prepend(item)` | ✓ | 前置菜单项 |
| `prepend_items(items)` | ✓ | 前置多个菜单项 |
| `insert(item, position)` | ✓ | 插入菜单项 |
| `insert_items(items, position)` | ✓ | 插入多个菜单项 |
| `remove(item)` | ✓ | 移除菜单项 |
| `remove_at(position)` | ✓ | 移除指定位置菜单项 |
| `get(id)` | ✓ | 按 ID 获取菜单项 |
| `items()` | ✓ | 获取所有菜单项 |

### 3.3 ContextMenu 实现（popup）

| API | OHOS 支持 | 说明 |
|-----|-----------|------|
| `popup(window)` | ✓ | 在窗口光标位置弹出 |
| `popup_at(window, position)` | ✓ | 在指定位置弹出 |

### 3.4 属性方法

| API | OHOS 支持 | 说明 |
|-----|-----------|------|
| `id()` | ✓ | 获取子菜单 ID |
| `text()` | ✓ | 获取文本 |
| `set_text(text)` | ✓ | 设置文本 |
| `is_enabled()` | ✓ | 获取启用状态 |
| `set_enabled(enabled)` | ✓ | 设置启用状态 |
| `set_icon(icon)` | △ | 设置图标（OHOS 用 startIcon） |
| `set_native_icon(native_icon)` | ✗ | **macOS only** |
| `app_handle()` | ✓ | 获取 AppHandle |

### 3.5 macOS 特有方法

| API | OHOS 支持 | 说明 |
|-----|-----------|------|
| `set_as_windows_menu_for_nsapp()` | ✗ | **macOS only** |
| `set_as_help_menu_for_nsapp()` | ✗ | **macOS only** |

### 3.6 OHOS 嵌套子菜单支持

> OHOS 支持多级嵌套子菜单（参考 menu.md 第 300-338 行）

```typescript
MenuItem({ 
  content: 'File', 
  builder: () => {
    Menu() {
      MenuItem({ content: 'Open', builder: () => this.NestedSubmenu() })
    }
  }
})
```

---

## 四、PredefinedMenuItem

### 4.1 创建方法

| API | OHOS 支持 | 实现方式 |
|-----|-----------|---------|
| `separator(manager)` | ✓ | MenuDivider |
| `copy(manager, text)` | ✓ | runJavaScript('execCommand("copy")') |
| `cut(manager, text)` | ✓ | runJavaScript('execCommand("cut")') |
| `paste(manager, text)` | ✓ | runJavaScript('execCommand("paste")') |
| `select_all(manager, text)` | ✓ | runJavaScript('execCommand("selectAll")') |
| `undo(manager, text)` | ✓ | runJavaScript('execCommand("undo")') |
| `redo(manager, text)` | ✓ | runJavaScript('execCommand("redo")') |
| `minimize(manager, text)` | ✓ | window.minimize() (API 12+) |
| `maximize(manager, text)` | ✓ | window.maximize() (API 12+) |
| `fullscreen(manager, text)` | ✓ | window.maximize(ENTER_IMMERSIVE) |
| `hide(manager, text)` | ✓ | minimize() 替代（无 hide API） |
| `hide_others(manager, text)` | ✗ | **OHOS 不支持**：跨应用限制 |
| `show_all(manager, text)` | ✗ | **OHOS 不支持**：跨应用限制 |
| `close_window(manager, text)` | ✓ | window.destroyWindow() (API 6+) |
| `quit(manager, text)` | ✓ | context.terminateSelf() (API 9+) |
| `about(manager, text, metadata)` | △ | 需自定义 AlertDialog |
| `services(manager, text)` | ✗ | **macOS only** |
| `bring_all_to_front(manager, text)` | ✗ | **macOS only** |

### 4.2 属性方法

| API | OHOS 支持 | 说明 |
|-----|-----------|------|
| `id()` | ✓ | 获取菜单项 ID |
| `text()` | ✓ | 获取文本 |
| `set_text(text)` | ✓ | 设置文本 |
| `app_handle()` | ✓ | 获取 AppHandle |

### 4.3 OHOS 窗口恢复 API 说明

> OHOS 有两个"恢复"API：

| Tauri API | OHOS API | 版本 | 说明 |
|-----------|----------|------|------|
| `restore()` | `window.recover()` | API 12+ | 从 maximize/fullscreen 恢复 |
| `restore()` | `window.restore()` | API 14+ | 从 minimize 恢复（仅 2in1 设备主窗口） |

---

## 五、CheckMenuItem

### 5.1 创建方法

| API | OHOS 支持 | 说明 |
|-----|-----------|------|
| `new(manager, text, enabled, checked, accelerator)` | ✓ | 创建勾选菜单项 |
| `with_id(manager, id, text, enabled, checked, accelerator)` | ✓ | 带指定 ID 创建 |

### 5.2 属性方法

| API | OHOS 支持 | 说明 |
|-----|-----------|------|
| `id()` | ✓ | 获取菜单项 ID |
| `text()` | ✓ | 获取文本 |
| `set_text(text)` | ✓ | 设置文本 |
| `is_enabled()` | ✓ | 获取启用状态 |
| `set_enabled(enabled)` | ✓ | 设置启用状态 |
| `is_checked()` | ✓ | 获取勾选状态（对应 OHOS selected） |
| `set_checked(checked)` | ✓ | 设置勾选状态（对应 OHOS selected） |
| `set_accelerator(accelerator)` | ✓ | 设置快捷键 |
| `app_handle()` | ✓ | 获取 AppHandle |

### 5.3 OHOS 实现

> OHOS 使用 MenuItem 的 `selected` + `selectIcon` + `onChange` 实现

```typescript
MenuItem({ 
  content: 'Option',
  selected: true,        // 对应 checked
  selectIcon: $r('sys.media.ohos_ic_public_ok'),  // 选中图标
})
.onChange((selected: boolean) => {
  // 对应 CheckMenuItem 状态变化
})
```

---

## 六、IconMenuItem

### 6.1 创建方法

| API | OHOS 支持 | 说明 |
|-----|-----------|------|
| `new(manager, text, enabled, icon, accelerator)` | ✓ | 创建图标菜单项 |
| `with_id(manager, id, text, enabled, icon, accelerator)` | ✓ | 带指定 ID 创建 |
| `with_native_icon(manager, text, enabled, native_icon, accelerator)` | ✗ | **macOS only** |
| `with_id_and_native_icon(manager, id, text, enabled, native_icon, accelerator)` | ✗ | **macOS only** |

### 6.2 属性方法

| API | OHOS 支持 | 说明 |
|-----|-----------|------|
| `id()` | ✓ | 获取菜单项 ID |
| `text()` | ✓ | 获取文本 |
| `set_text(text)` | ✓ | 设置文本 |
| `is_enabled()` | ✓ | 获取启用状态 |
| `set_enabled(enabled)` | ✓ | 设置启用状态 |
| `set_accelerator(accelerator)` | ✓ | 设置快捷键 |
| `set_icon(icon)` | ✓ | 设置图标（对应 OHOS startIcon） |
| `set_native_icon(native_icon)` | ✗ | **macOS only** |
| `app_handle()` | ✓ | 获取 AppHandle |

### 6.3 OHOS 实现

> OHOS 使用 MenuItem 的 `startIcon` / `symbolStartIcon` 实现

```typescript
MenuItem({ 
  content: 'Action',
  startIcon: $r('app.media.icon'),     // 对应 IconMenuItem icon
  symbolStartIcon: $r('sys.symbol.icon'), // API 12+
})
```

---

## 七、MenuEvent

### 7.1 事件结构

| 字段 | OHOS 支持 | 说明 |
|------|----------|------|
| `id` | ✓ | 菜单项 ID (MenuId) |

### 7.2 方法

| API | OHOS 支持 | 说明 |
|-----|-----------|------|
| `id()` | ✓ | 获取菜单项 ID |

### 7.3 OHOS 事件机制

```
ArkTS: MenuItem.onClick()
    ↓
emit_menu_event(menuId, itemId)
    ↓
Rust: MenuEventDispatcher.dispatch(MenuEvent { id })
    ↓
Tauri: AppHandle.on_menu_event(handler)
```

---

## 八、MenuItemKind

### 8.1 枚举值

| 值 | OHOS 支持 | 说明 |
|----|----------|------|
| `MenuItem(MenuItem<R>)` | ✓ | 普通菜单项 |
| `Submenu(Submenu<R>)` | ✓ | 子菜单 |
| `Predefined(PredefinedMenuItem<R>)` | ✓ | 预定义菜单项 |
| `Check(CheckMenuItem<R>)` | ✓ | 勾选菜单项 |
| `Icon(IconMenuItem<R>)` | ✓ | 图标菜单项 |

### 8.2 方法

| API | OHOS 支持 | 说明 |
|-----|-----------|------|
| `id()` | ✓ | 获取菜单项 ID |
| `as_menuitem()` | ✓ | 转换为 MenuItem |
| `as_menuitem_unchecked()` | ✓ | 强制转换（panic if not） |
| `as_submenu()` | ✓ | 转换为 Submenu |
| `as_submenu_unchecked()` | ✓ | 强制转换 |
| `as_predefined_menuitem()` | ✓ | 转换为 PredefinedMenuItem |
| `as_predefined_menuitem_unchecked()` | ✓ | 强制转换 |
| `as_check_menuitem()` | ✓ | 转换为 CheckMenuItem |
| `as_check_menuitem_unchecked()` | ✓ | 强制转换 |
| `as_icon_menuitem()` | ✓ | 转换为 IconMenuItem |
| `as_icon_menuitem_unchecked()` | ✓ | 强制转换 |

---

## 九、AboutMetadata

### 9.1 字段

| 字段 | OHOS 支持 | 说明 |
|------|----------|------|
| `name` | ✓ | 应用名称 |
| `version` | ✓ | 应用版本 |
| `short_version` | ✓ | 简短版本 |
| `authors` | ✓ | 作者列表 |
| `comments` | ✓ | 注释 |
| `copyright` | ✓ | 版权 |
| `license` | ✓ | 许可证 |
| `website` | ✓ | 网站 |
| `website_label` | ✓ | 网站标签 |
| `credits` | ✓ | 致谢 |
| `icon` | ✓ | 应用图标 |

### 9.2 OHOS 实现

> OHOS 无原生 About 对话框，需使用 AlertDialog 自定义实现

```typescript
AlertDialog.show({
  title: metadata.name,
  message: `Version: ${metadata.version}\n${metadata.copyright}`,
})
```

### 9.3 AboutMetadataBuilder

| API | OHOS 支持 | 说明 |
|-----|-----------|------|
| `new()` | ✓ | 创建 builder |
| `name(name)` | ✓ | 设置名称 |
| `version(version)` | ✓ | 设置版本 |
| `short_version(version)` | ✓ | 设置简短版本 |
| `authors(authors)` | ✓ | 设置作者 |
| `comments(comments)` | ✓ | 设置注释 |
| `copyright(copyright)` | ✓ | 设置版权 |
| `license(license)` | ✓ | 设置许可证 |
| `website(website)` | ✓ | 设置网站 |
| `website_label(label)` | ✓ | 设置网站标签 |
| `credits(credits)` | ✓ | 设置致谢 |
| `icon(icon)` | ✓ | 设置图标 |
| `build()` | ✓ | 构建 AboutMetadata |

---

## 十、NativeIcon

> **macOS only**，OHOS 不支持

### 10.1 枚举值（全部不支持）

- `Add`, `Advanced`, `Bluetooth`, `Bookmarks`, `Caution`, ...
- 共 60+ 个原生图标值，全部为 macOS only

---

## 十一、ContextMenu Trait

> Menu 和 Submenu 实现此 trait

| API | OHOS 支持 | 说明 |
|-----|-----------|------|
| `hpopupmenu()` | ✗ | **Windows only** |
| `popup(window)` | ✓ | 在窗口光标位置弹出 |
| `popup_at(window, position)` | ✓ | 在指定位置弹出 |

---

## 十二、IsMenuItem Trait

> 所有菜单项类型实现此 trait

| API | OHOS 支持 | 说明 |
|-----|-----------|------|
| `kind()` | ✓ | 返回 MenuItemKind |
| `id()` | ✓ | 返回 MenuId |

---

## 十三、平台专属 API（OHOS 无需实现）

| API | 平台 | 说明 |
|-----|------|------|
| `hpopupmenu()` | Windows | 获取 HMENU |
| `set_as_app_menu()` | macOS/Windows | 全局菜单栏 |
| `set_as_window_menu()` | macOS/Windows | 窗口菜单栏 |
| `set_as_windows_menu_for_nsapp()` | macOS | Window 菜单 |
| `set_as_help_menu_for_nsapp()` | macOS | Help 菜单 |
| `set_native_icon()` | macOS | 原生图标 |
| `new_with_native_icon()` | macOS | 带原生图标创建 |
| `hide_others()` | macOS | 隐藏其他窗口 |
| `show_all()` | macOS | 显示所有窗口 |
| `services()` | macOS | Services 菜单 |
| `bring_all_to_front()` | macOS | 全部置前 |
| `NativeIcon` | macOS | 原生图标枚举 |

---

## 十四、OHOS 特有限制

| 限制 | 影响 | 设计决策 |
|------|------|----------|
| 无全局菜单栏 | `set_as_app_menu()` 不支持 | 返回 "not supported on OHOS" |
| 无窗口菜单栏 | `set_as_window_menu()` 不支持 | 返回 "not supported on OHOS" |
| 无原生 hide API | `hide()` 用 minimize 替代 | 文档说明 |
| 跨应用限制 | `hide_others()`/`show_all()` 不支持 | 返回 "not supported on OHOS" |
| 无原生 About | `about()` 需自定义 AlertDialog | 文档说明 |
| 无原生图标 | `NativeIcon` 不支持 | macOS only |
| popup 需 bindContextMenu | API 12+ isShown 参数 | 文档说明 |
| 页面构建时序 | 必须等待页面构建完成 | 文档说明 |

---

## 十五、实现状态

> 更新时间：2026-05-15 | Phase 0-5 全部完成

| 优先级 | API | 状态 | 实现位置 |
|--------|-----|------|----------|
| **P0** | Menu.new() | ✅ 已完成 | `muda/ohos/mod.rs` |
| **P0** | MenuItem.new() | ✅ 已完成 | `muda/ohos/mod.rs` |
| **P0** | Submenu.new() | ✅ 已完成 | `muda/ohos/mod.rs` |
| **P0** | Menu.popup() | ✅ 已完成 | `tauri/menu/menu.rs` + `muda/ohos` |
| **P0** | MenuEvent | ✅ 已完成 | `mod.rs` + `event.rs` |
| **P1** | PredefinedMenuItem.separator() | ✅ 已完成 | `predefined.rs` |
| **P1** | PredefinedMenuItem.copy/cut/paste | ✅ 已完成 | `predefined.rs` + `menu.ets` |
| **P1** | PredefinedMenuItem.minimize/maximize | ✅ 已完成 | `predefined.rs` + `menu.ets` |
| **P1** | PredefinedMenuItem.quit() | ✅ 已完成 | `predefined.rs` + `menu.ets` |
| **P2** | CheckMenuItem | ✅ 已完成 | `muda/ohos/mod.rs` (映射到 selected + onChange) |
| **P2** | IconMenuItem | ✅ 已完成 | `muda/ohos/mod.rs` (映射到 startIcon) |
| **P2** | PredefinedMenuItem.about() | ✅ 已完成 | `predefined.rs` (factory 已实现，执行需 AlertDialog) |
| **N/A** | 平台专属 API | ✗ 不实现 | macOS/Windows only |

---

## 十六、外部库依赖总结

### Menu 模块依赖

| 平台 | crate | 底层库 | 版本 |
|------|-------|--------|------|
| **Windows** | `muda` | `windows-sys` (Win32) | 0.19 |
| **macOS** | `muda` | `objc2-app-kit` (Cocoa) | 0.19 |
| **Linux** | `muda` | `gtk` (GTK3) + `libxdo` | 0.19 |
| **OHOS** | `openharmony-ability` | ArkUI Menu API | ✅ 已实现 |

### Tray 模块依赖

| 平台 | crate | 底层库 | 版本 |
|------|-------|--------|------|
| **Windows** | `tray-icon` | `windows-sys` (Shell_NotifyIconW) | 0.24 |
| **macOS** | `tray-icon` | `objc2-app-kit` (NSStatusItem) | 0.24 |
| **Linux** | `tray-icon` | `libappindicator` (AppIndicator) | 0.24 |
| **OHOS** | `openharmony-ability` | StatusBar API | ✅ 已实现 |

### 依赖关系图

```
tauri::menu
    │
    │ cfg(any(desktop, target_env = "ohos"))
    │
    ├─► Windows/macOS/Linux: muda crate
    │       │
    │       ├─► Windows: windows-sys (Win32 API)
    │       ├─► macOS: objc2-app-kit (NSMenu, NSMenuItem)
    │       └──► Linux: gtk + libxdo
    │
    └──► OHOS: muda (path) → openharmony-ability (menu feature)
            │
            ├─► menu_types.ets: ArkTS 类型定义
            ├─► menu.ets: TauriMenuManager + PredefinedActionExecutor
            ├─► menu_state.ets: MenuStateController
            ├─► TauriMenu.ets: bindContextMenu(isShown) 组件
            └──► Rust: types.rs + event.rs + predefined.rs + popup.rs

tauri::tray
    │
    │ cfg(any(desktop, target_env = "ohos"))
    │
    ├─► Windows/macOS/Linux: tray-icon crate
    │       │
    │       ├─► Windows: windows-sys (Shell_NotifyIconW)
    │       ├─► macOS: objc2-app-kit (NSStatusItem)
    │       └──► Linux: libappindicator
    │
    └──► OHOS: openharmony-ability (已实现 statusBar 模块)
            │
            └─► statusBar.ets: StatusBar + QuickOperation
```

### OHOS 适配工作量

| 模块 | 需新增内容 | 状态 |
|------|----------|------|
| **menu** | menu_types.ets + menu.ets + predefined.ets + window_ops.ets + menu_state.ets + Rust FFI | ✅ 已完成 (Phase 1-5) |
| **tray** | statusBar.ets (已有) + Rust FFI | ✅ 已完成 |

---

## 十七、参考文档

- [muda crate](https://github.com/tauri-apps/muda) - Menu 底层实现
- [tray-icon crate](https://github.com/tauri-apps/tray-icon) - Tray 底层实现
- [OHOS Menu API](reference/menu.md) - ArkUI Menu 组件
- [OHOS MenuItem API](reference/menu_item.md) - ArkUI MenuItem 组件
- [OHOS bindContextMenu](reference/menu_control.md) - 菜单弹出控制
# Phase 0: muda OHOS 后端设计

> 版本：v4.0
> 更新时间：2026-05-14
> 目标：为 muda crate 实现 OHOS 平台支持
> 
> **当前状态**：架构修正，待重新实现

---

## 一、背景与决策

### 1.1 为什么在 muda 中实现

| 维度 | muda 实现 | tauri 直接实现 |
|------|-----------|----------------|
| **架构一致性** | ✓ 符合 muda 现有架构（windows/macos/gtk/ohos） | ✗ tauri 特有实现 |
| **复用性** | ✓ tauri menu module、tray-icon 都可使用 | ✗ 仅 tauri menu 可用 |
| **维护性** | ✓ 菜单逻辑集中一处 | ✗ 维护分散 |
| **事件统一** | ✓ 使用 muda::MenuEvent 现有机制 | ✗ 需额外映射 |
| **tray 一致性** | ✓ 与 tray 模块架构一致 | ✗ tray 用 muda，menu 用 tauri |

### 1.2 正确架构依赖链

```
tauri::menu
    │
    └── muda::platform_impl::ohos
            │
            └── openharmony-ability (menu feature)
                    │
                    ├── NAPI 层：供 ArkTS 调用
                    │   └── emit_menu_event(menu_id: String)
                    │
                    └── Rust 层：供 muda 调用
                        ├── menu_event_receiver() -> &'static Receiver<String>
                        └── popup_menu(menu_json: String, ...) -> Result<()> (stub)
```

**关键点**：
- openharmony-ability 封装鸿蒙系统接口（ArkUI Menu API）
- muda 调用 openharmony-ability 的 **Rust 层接口**
- ArkTS 调用 openharmony-ability 的 **NAPI 层接口**

### 1.3 事件流

```
用户点击菜单项
    │
    └── ArkUI Menu API onChange
            │
            └── ArkTS 调用 NAPI emit_menu_event(menu_id)
                    │
                    └── openharmony-ability channel 发送
                            │
                            └── muda 监听 channel
                                    │
                                    └── muda::MenuEvent::send()
                                            │
                                            └── 用户通过 MenuEvent::receiver() 接收
```

### 1.4 muda 使用范围

```
tauri
    │
    ├── tauri::tray
    │       │
    │       └── tray-icon crate
    │               │
    │               └── muda::ContextMenu (托盘右键菜单)
    │
    └── tauri::menu
            │
            └── muda::Menu (应用菜单)
                    │   OHOS: 用于 ContextMenu（popup 菜单）
                    │   OHOS: Window Menu Bar 不支持，stub 实现
```

### 1.3 与 tray 模块的关系

tray 模块的 Phase 0 已设计 muda OHOS 后端，menu 模块复用同一实现：

```
tray 模块 Phase 0 (phase0-muda-ohos-design.md)
    │
    └── muda::platform_impl::ohos
            │
            ├── Menu / MenuItem / Submenu
            ├── ContextMenu trait
            └── MenuEvent

menu 模块 Phase 1-5
    │
    └── 依赖 muda::platform_impl::ohos
            │
            ├── Menu.popup() → ContextMenu trait
            └── MenuEvent → muda::MenuEvent
```

---

## 二、muda 平台隔离方式分析

### 2.1 Cargo.toml 依赖隔离

muda 使用 `[target.'cfg(...)'.dependencies]` 按平台引入依赖，**不需要专门的 platform feature**：

```toml
# muda/Cargo.toml 现有结构

[target.'cfg(target_os = "windows")'.dependencies]
windows-sys = { version = "...", features = [...] }

[target.'cfg(target_os = "macos")'.dependencies]
objc2-app-kit = { version = "...", features = [...] }
png = "0.18"

[target.'cfg(any(target_os = "linux", ...))'.dependencies]
gtk = { version = "0.18", optional = true }
libxdo = { version = "0.6.0", optional = true }

# OHOS 新增
[target.'cfg(target_env = "ohos")'.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
base64 = "0.22"
png = "0.18"
openharmony-ability = { path = "../openharmony-ability/crates/ability", features = ["menu"] }
```

**关键点**：
- `target_env = "ohos"` 是独立编译目标
- muda 依赖 openharmony-ability（调用 Rust API）
- openharmony-ability 提供 menu feature（不需要额外依赖）

### 2.2 platform_impl/mod.rs 模块选择

```rust
// muda/src/platform_impl/mod.rs 现有结构

#[cfg(target_os = "windows")]
#[path = "windows/mod.rs"]
mod platform;

#[cfg(target_os = "macos")]
#[path = "macos/mod.rs"]
mod platform;

#[cfg(all(any(target_os = "linux", ...), feature = "gtk"))]
#[path = "gtk/mod.rs"]
mod platform;

// OHOS 新增
#[cfg(target_env = "ohos")]
#[path = "ohos/mod.rs"]
mod platform;

pub(crate) use self::platform::*;
```

### 2.3 各平台实现对比

| 平台 | 原生 Handle | Window Menu Bar | Context Menu | Event 机制 |
|------|-------------|-----------------|--------------|------------|
| Windows | `HMENU` | `SetMenu(hwnd)` + `WM_COMMAND` | `TrackPopupMenu` | `MenuEvent::send()` |
| macOS | `NSMenu` / `NSMenuItem` | `NSApp.setMainMenu` | `popUpMenuPositioningItem` | `MenuEvent::send()` |
| gtk | `GtkMenu` | stub（Linux 无传统菜单栏） | `gtk_menu_popup` | `MenuEvent::send()` |
| OHOS | 无原生 handle | **stub** | `bindContextMenu(isShown)` | `emit_menu_event()` NAPI |

### 2.3.1 跨平台 API 对齐要求

**tauri::menu 的 `popup_inner()` 调用方式因平台而异**：

```rust
// tauri/crates/tauri/src/menu/menu.rs (popup_inner 方法)

// Windows/macOS/GTK 需要 window/hwnd/view 参数来定位菜单
#[cfg(target_os = "macos")]
if let Ok(view) = window.ns_view() {
    unsafe { self_.inner().show_context_menu_for_nsview(view as _, position); }
}

#[cfg(all(any(target_os = "linux", ...), not(target_env = "ohos")))]
if let Ok(w) = window.gtk_window() {
    self_.inner().show_context_menu_for_gtk_window(w.as_ref(), position);
}

#[cfg(windows)]
if let Ok(hwnd) = window.hwnd() {
    unsafe { self_.inner().show_context_menu_for_hwnd(hwnd.0 as _, position); }
}

// OHOS 不需要 window 参数，直接调用 popup(x, y)
#[cfg(target_env = "ohos")]
{
    // 从 Position 提取 x, y 坐标
    let (x, y) = match position {
        Some(Position::Logical(p)) => (Some(p.x), Some(p.y)),
        Some(Position::Physical(p)) => (Some(p.x as f64), Some(p.y as f64)),
        None => (None, None),
    };
    self_.inner().popup(x, y)?;
}
```

**OHOS 与其他平台的关键差异**：
- Windows/macOS/GTK 的 `show_context_menu_for_*` 需要原生窗口句柄来定位菜单
- **OHOS 的 `popup()` 不需要窗口句柄**，只需要 x, y 坐标（通过 `bindContextMenu` 实现）
- 因此 OHOS **不需要**实现 `show_context_menu_for_ohos_window()` 方法

### 2.4 公共 API 导出

参考 `windows/mod.rs` 和 `macos/mod.rs`，OHOS 需导出：

```rust
// muda/src/platform_impl/ohos/mod.rs

mod accelerator;
mod icon;
mod util;

pub(crate) use icon::OhosIcon as PlatformIcon;  // 或 stub

pub struct Menu {
    id: MenuId,
    internal_id: u32,
    children: Vec<Rc<RefCell<MenuChild>>>,
    // OHOS: 不存储原生 handle
}

pub struct MenuChild {
    // 共享字段（与其他平台一致）
    item_type: MenuItemType,
    text: String,
    enabled: bool,
    id: MenuId,
    internal_id: u32,
    accelerator: Option<KeyAccelerator>,
    
    // predefined fields
    predefined_item_type: Option<PredefinedMenuItemType>,
    
    // check fields
    checked: bool,
    
    // icon fields
    icon: Option<Icon>,
    
    // submenu fields
    children: Option<Vec<Rc<RefCell<MenuChild>>>>,
}
```

---

## 三、OHOS 菜单能力分析

### 3.1 OHOS 支持（用于 ContextMenu）

| 功能 | OHOS API | 说明 |
|------|----------|------|
| Popup 菜单 | bindContextMenu(isShown) | ✓ 支持 API 12+ |
| MenuItem | MenuItem + startIcon/symbolStartIcon | ✓ 支持 |
| Submenu | MenuItem 嵌套 Menu | ✓ 支持（已验证 menu.md:300-338） |
| Separator | MenuDivider 组件 | ✓ 支持 |
| CheckMenuItem | MenuItem.selected + onChange | ✓ 支持 |
| IconMenuItem | MenuItem.startIcon/symbolStartIcon | ✓ 支持 |
| 菜单项点击事件 | onChange / onClick | ✓ 支持 |

### 3.2 OHOS 不支持（需 stub）

| 功能 | OHOS | 处理方式 | 参考 gtk 平台 |
|------|------|----------|---------------|
| Window Menu Bar | ✗ 无传统桌面菜单栏 | stub：API 存在但返回 Error | gtk 同样 stub |
| accelerator 功能 | ✗ 无快捷键系统 | 仅显示文本，忽略功能 | gtk 有快捷键但有限 |
| init_for_hwnd | ✗ 无 HWND | stub：返回 `NotSupportedOnPlatform` | gtk 有 init_for_window |
| init_for_nsapp | ✗ 无 NSApp | stub：返回 `NotSupportedOnPlatform` | - |

## 四、实际实现状态

### 4.1 设计 vs 实际差距

| 设计要求 | 实际状态 | 差距说明 |
|----------|----------|----------|
| `Menu::show_context_menu_for_ohos_window()` | ❌ 不存在 | **关键缺失**：tauri::menu 通过此方法调用各平台，必须实现 |
| `MenuChild::show_context_menu_for_ohos_window()` | ❌ 不存在 | Submenu 同理 |
| Window Menu Bar stub 方法 | ❌ 不存在 | 12个方法需与其他平台对齐 |
| `Menu::popup()` | ✅ 已有 | 内部调用 `popup_context_menu()` |
| `MenuChild::popup()` | ✅ 已有 | 内部调用 `popup_context_menu()` |
| `Menu::to_json()` | ✅ 已有 | 序列化菜单项为 JSON |
| `Menu::to_menu_items()` | ✅ 已有 | 转换为 MenuItemData 列表 |
| `MenuChild::to_menu_item_data()` | ✅ 已有 | 完整序列化所有字段 |
| 事件监听 | ✅ 已有 | `start_event_listener()` + channel |
| `PlatformIcon::from_rgba()` | ✅ 已有 | icon.rs |
| `PlatformIcon::to_base64()` | ❌ 缺失 | 设计文档要求但未实现 |

### 4.2 openharmony-ability menu 模块

**文件**: `openharmony-ability/crates/ability/src/menu/mod.rs` (75 行)

```rust
// Event channel: ArkTS → muda
static MENU_EVENT_CHANNEL: LazyLock<(Sender<String>, Receiver<String>)> = LazyLock::new(unbounded);

// Popup channel: muda → ArkTS
static POPUP_CHANNEL: LazyLock<(Sender<PopupRequest>, Receiver<PopupRequest>)> = LazyLock::new(unbounded);

/// Rust API: Get menu event receiver (for muda)
pub fn menu_event_receiver() -> &'static Receiver<String> { &MENU_EVENT_CHANNEL.1 }

/// Rust API: Get popup request receiver (for ArkTS)
pub fn popup_request_receiver() -> &'static Receiver<PopupRequest> { &POPUP_CHANNEL.1 }

/// NAPI API: Emit menu event from ArkTS
#[napi]
pub fn emit_menu_event(menu_id: String) { ... }

/// Rust API: Popup context menu (for muda)
pub fn popup_context_menu(json_data: String, x: Option<f64>, y: Option<f64>) -> Result<()> { ... }
```

### 4.3 muda/src/platform_impl/ohos/mod.rs

**文件**: `muda/src/platform_impl/ohos/mod.rs` (405 行)

**架构修正**: muda 使用 openharmony-ability 的类型，不再自己定义 ArkTsMenuItem

```rust
// 使用 openharmony-ability 的 MenuItemData (第 25 行)
use openharmony_ability::menu::MenuItemData;

// Menu 转换方法 (第 69-78 行)
pub fn to_menu_items(&self) -> Vec<MenuItemData> {
    self.children.iter().map(|c| c.borrow().to_menu_item_data()).collect()
}

pub fn to_json(&self) -> String {
    serde_json::to_string(&self.to_menu_items()).unwrap_or_default()
}

// Menu popup 方法 (第 80-86 行)
pub fn popup(&self, x: Option<f64>, y: Option<f64>) -> crate::Result<()> {
    init_menu_event_listener();
    let json = self.to_json();
    openharmony_ability::menu::popup_context_menu(json, x, y)?;
    Ok(())
}

// MenuChild 转换方法 (第 280-310 行)
pub fn to_menu_item_data(&self) -> MenuItemData { ... }

// 事件监听 (第 380-395 行)
fn start_event_listener() {
    let receiver = openharmony_ability::menu::menu_event_receiver();
    while let Ok(menu_id) = receiver.recv() {
        crate::MenuEvent::send(crate::MenuEvent { id: crate::MenuId::new(menu_id) });
    }
}
static EVENT_LISTENER_STARTED: AtomicBool = AtomicBool::new(false);

fn start_event_listener() {
    if EVENT_LISTENER_STARTED.swap(true, Ordering::AcqRel) {
        return;
    }
    std::thread::spawn(|| {
        let receiver = openharmony_ability::menu::menu_event_receiver();
        while let Ok(menu_id) = receiver.recv() {
            crate::MenuEvent::send(crate::MenuEvent {
                id: crate::MenuId::new(menu_id),
            });
        }
    });
}

pub fn init_menu_event_listener() {
    start_event_listener();
}
```

**已实现方法**：
- `Menu::new()`, `id()`, `add_menu_item()`, `remove()`, `items()`
- `Menu::build_arkts_menu()`, `Menu::to_json()` ✓
- `MenuChild::to_arkts_menu_item()` ✓
- `encode_rgba_to_png()` ✓
- `init_menu_event_listener()` ✓ (监听 openharmony-ability channel)

### 4.4 Cargo.toml 配置

**muda/Cargo.toml**:
```toml
[target.'cfg(target_env = "ohos")'.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
base64 = "0.22"
png = "0.18"
openharmony-ability = { path = "../openharmony-ability/crates/ability", features = ["menu"] }
```

**openharmony-ability/Cargo.toml**:
```toml
[features]
menu = []  # 无额外依赖，仅导出模块
```

### 4.5 待验证

| 内容 | 状态 | 说明 |
|------|------|------|
| OHOS 编译 | ⬜ pending | 需要 OHOS 编译工具链 |
| NAPI 注册 | ⬜ pending | openharmony-ability 中注册 emit_menu_event |

### 4.6 实现差异

| 维度 | 设计文档 | 实际实现 | 说明 |
|------|----------|----------|------|
| `checked` 字段 | `bool` | `Option<Rc<AtomicBool>>` | ✓ 优化：支持多引用同步 |
| `internal_id` | `u32` | 无 | 仅用 `id`，足够 |
| `is_syncing_checked_state` | 无 | `Option<Rc<AtomicBool>>` | ✓ 新增：防止递归 |

---

## 五、核心实现（待补充）

### 5.1 模块结构（实际 vs 设计）

**实际结构**（空实现）：
```
muda/src/platform_impl/ohos/
├── mod.rs          # 所有实现在此文件（287 行）
│   └── Menu + MenuChild
│   └── 所有构造方法
│   └── 所有 getter/setter
│   └── Submenu 方法
│   └── ❌ 缺少：build_arkts_menu, to_arkts_menu_item, stub 方法
│
└── icon.rs         # PlatformIcon（31 行）
    └── from_rgba()
    └── ❌ 缺少：to_base64()
```

**设计目标结构**（完成后）：
```
muda/src/platform_impl/ohos/
├── mod.rs          # 主要实现（合并）
│   └── Menu + MenuChild
│   └── ArkTsMenuItem + ArkTsMenuData（待添加）
│   └── build_arkts_menu() / to_arkts_menu_item()（待添加）
│   └── stub 方法（待添加）
│
├── icon.rs         # Icon 处理
│   └── PlatformIcon
│   └── to_base64()（待添加）
│
└── event.rs        # MenuEvent（待创建）
    └── emit_menu_event()
    └── menu_event_receiver()
```

### 5.2 Cargo.toml 依赖（待添加）

```rust
// muda/src/platform_impl/ohos/util.rs

#[derive(Debug, Clone, serde::Serialize)]
pub struct ArkTsMenuItem {
    pub id: String,
    pub text: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accelerator: Option<String>,
    pub item_type: String,  // "item" | "submenu" | "separator" | "check" | "icon"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,  // base64
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<ArkTsMenuItem>>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ArkTsMenuData {
    pub items: Vec<ArkTsMenuItem>,
}
```

---

## 六、待实现代码示例

以下代码示例为设计参考，实际实现已合并到 `mod.rs`。

### 6.1 ArkTsMenuItem 结构

```rust
// 需添加到 muda/src/platform_impl/ohos/mod.rs

use crate::{IsMenuItem, MenuId, MenuItemKind, MenuItemType, util::{AddOp, Counter}};
use std::{cell::RefCell, rc::Rc};

static COUNTER: Counter = Counter::new_with_start(1000);

#[derive(Debug)]
pub(crate) struct Menu {
    id: MenuId,
    internal_id: u32,
    children: Vec<Rc<RefCell<MenuChild>>>,
}

impl Menu {
    pub fn new(id: Option<MenuId>) -> Self {
        let internal_id = COUNTER.next();
        Self {
            id: id.unwrap_or_else(|| MenuId::new(internal_id.to_string())),
            internal_id,
            children: Vec::new(),
        }
    }
    
    pub fn id(&self) -> &MenuId {
        &self.id
    }
    
    pub fn add_menu_item(&mut self, item: &dyn IsMenuItem, op: AddOp) -> crate::Result<()> {
        let child = item.child();
        match op {
            AddOp::Append => self.children.push(child),
            AddOp::Insert(pos) => self.children.insert(pos, child),
        }
        Ok(())
    }
    
    pub fn remove(&mut self, item: &dyn IsMenuItem) -> crate::Result<()> {
        let id = item.id();
        let index = self.children
            .iter()
            .position(|c| c.borrow().id() == id)
            .ok_or(crate::Error::NotAChildOfThisMenu)?;
        self.children.remove(index);
        Ok(())
    }
    
    pub fn items(&self) -> Vec<MenuItemKind> {
        self.children.iter().map(|c| c.borrow().kind(c.clone())).collect()
    }
    
    pub fn build_arkts_menu(&self) -> ArkTsMenuData {
        ArkTsMenuData {
            items: self.children.iter().map(|c| c.borrow().to_arkts_menu_item()).collect(),
        }
    }
}

// Stub: Window Menu Bar 不支持
impl Menu {
    pub fn init_for_hwnd(&mut self, _hwnd: isize) -> crate::Result<()> {
        Err(crate::Error::NotSupportedOnPlatform)
    }
    
    pub fn init_for_nsapp(&self) -> crate::Result<()> {
        Err(crate::Error::NotSupportedOnPlatform)
    }
    
    pub fn remove_for_hwnd(&mut self, _hwnd: isize) -> crate::Result<()> {
        Err(crate::Error::NotSupportedOnPlatform)
    }
    
    pub fn hide_for_hwnd(&self, _hwnd: isize) -> crate::Result<()> {
        Err(crate::Error::NotSupportedOnPlatform)
    }
    
    pub fn show_for_hwnd(&self, _hwnd: isize) -> crate::Result<()> {
        Err(crate::Error::NotSupportedOnPlatform)
    }
}

// OHOS 特有：Context Menu
impl Menu {
    pub fn show_context_menu_for_component(&self) -> bool {
        // 调用 openharmony-ability 显示菜单
        // 返回 true 表示菜单已显示
        true
    }
}
```

### 6.2 MenuChild 实现（待添加）

```rust
// muda/src/platform_impl/ohos/menu_child.rs

use crate::{
    accelerator::KeyAccelerator,
    icon::{Icon, NativeIcon},
    items::PredefinedMenuItemType,
    MenuId, MenuItemKind, MenuItemType,
    util::Counter,
};
use std::{cell::RefCell, rc::Rc};

static COUNTER: Counter = Counter::new();

#[derive(Debug)]
pub(crate) struct MenuChild {
    item_type: MenuItemType,
    text: String,
    enabled: bool,
    id: MenuId,
    internal_id: u32,
    accelerator: Option<KeyAccelerator>,
    
    predefined_item_type: Option<PredefinedMenuItemType>,
    checked: bool,
    icon: Option<Icon>,
    
    children: Option<Vec<Rc<RefCell<MenuChild>>>>,
}

impl MenuChild {
    pub fn new(
        text: &str,
        enabled: bool,
        accelerator: Option<KeyAccelerator>,
        id: Option<MenuId>,
    ) -> Self {
        let internal_id = COUNTER.next();
        Self {
            item_type: MenuItemType::MenuItem,
            text: text.to_string(),
            enabled,
            id: id.unwrap_or_else(|| MenuId::new(internal_id.to_string())),
            internal_id,
            accelerator,
            predefined_item_type: None,
            checked: false,
            icon: None,
            children: None,
        }
    }
    
    pub fn new_submenu(text: &str, enabled: bool, id: Option<MenuId>) -> Self {
        let internal_id = COUNTER.next();
        Self {
            item_type: MenuItemType::Submenu,
            text: text.to_string(),
            enabled,
            id: id.unwrap_or_else(|| MenuId::new(internal_id.to_string())),
            internal_id,
            accelerator: None,
            predefined_item_type: None,
            checked: false,
            icon: None,
            children: Some(Vec::new()),
        }
    }
    
    pub fn new_predefined(item_type: PredefinedMenuItemType, text: Option<String>) -> Self {
        let internal_id = COUNTER.next();
        Self {
            item_type: MenuItemType::Predefined,
            text: text.unwrap_or_else(|| item_type.text().to_string()),
            enabled: true,
            id: MenuId::new(internal_id.to_string()),
            internal_id,
            accelerator: item_type.accelerator().map(Into::into),
            predefined_item_type: Some(item_type),
            checked: false,
            icon: None,
            children: None,
        }
    }
    
    pub fn new_check(
        text: &str,
        enabled: bool,
        checked: bool,
        accelerator: Option<KeyAccelerator>,
        id: Option<MenuId>,
    ) -> Self {
        let internal_id = COUNTER.next();
        Self {
            item_type: MenuItemType::Check,
            text: text.to_string(),
            enabled,
            id: id.unwrap_or_else(|| MenuId::new(internal_id.to_string())),
            internal_id,
            accelerator,
            predefined_item_type: None,
            checked,
            icon: None,
            children: None,
        }
    }
    
    pub fn new_icon(
        text: &str,
        enabled: bool,
        icon: Option<Icon>,
        accelerator: Option<KeyAccelerator>,
        id: Option<MenuId>,
    ) -> Self {
        let internal_id = COUNTER.next();
        Self {
            item_type: MenuItemType::Icon,
            text: text.to_string(),
            enabled,
            id: id.unwrap_or_else(|| MenuId::new(internal_id.to_string())),
            internal_id,
            accelerator,
            predefined_item_type: None,
            checked: false,
            icon,
            children: None,
        }
    }
    
    pub fn item_type(&self) -> MenuItemType {
        self.item_type
    }
    
    pub fn id(&self) -> &MenuId {
        &self.id
    }
    
    pub fn internal_id(&self) -> u32 {
        self.internal_id
    }
    
    pub fn text(&self) -> String {
        self.text.clone()
    }
    
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }
    
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    pub fn is_checked(&self) -> bool {
        self.checked
    }
    
    pub fn set_checked(&mut self, checked: bool) {
        self.checked = checked;
    }
    
    pub fn set_icon(&mut self, icon: Option<Icon>) {
        self.icon = icon;
    }
    
    pub fn set_key_accelerator(&mut self, accelerator: Option<KeyAccelerator>) -> crate::Result<()> {
        self.accelerator = accelerator;
        Ok(())
    }
}

impl MenuChild {
    pub fn add_menu_item(&mut self, item: &dyn crate::IsMenuItem, op: AddOp) -> crate::Result<()> {
        let child = item.child();
        let children = self.children.as_mut().unwrap();
        match op {
            AddOp::Append => children.push(child),
            AddOp::Insert(pos) => children.insert(pos, child),
        }
        Ok(())
    }
    
    pub fn remove(&mut self, item: &dyn crate::IsMenuItem) -> crate::Result<()> {
        let id = item.id();
        let children = self.children.as_mut().unwrap();
        let index = children
            .iter()
            .position(|c| c.borrow().id() == id)
            .ok_or(crate::Error::NotAChildOfThisMenu)?;
        children.remove(index);
        Ok(())
    }
    
    pub fn items(&self) -> Vec<MenuItemKind> {
        self.children.as_ref().unwrap()
            .iter()
            .map(|c| c.borrow().kind(c.clone()))
            .collect()
    }
    
    pub fn build_arkts_menu(&self) -> ArkTsMenuData {
        ArkTsMenuData {
            items: self.children.as_ref().unwrap()
                .iter()
                .map(|c| c.borrow().to_arkts_menu_item())
                .collect(),
        }
    }
}

impl MenuChild {
    pub fn to_arkts_menu_item(&self) -> ArkTsMenuItem {
        match self.item_type {
            MenuItemType::MenuItem => ArkTsMenuItem {
                id: self.id.as_ref().to_string(),
                text: self.text.clone(),
                enabled: self.enabled,
                accelerator: self.accelerator.as_ref().map(|a| a.to_string()),
                item_type: "item".to_string(),
                checked: None,
                icon: None,
                children: None,
            },
            MenuItemType::Submenu => ArkTsMenuItem {
                id: self.id.as_ref().to_string(),
                text: self.text.clone(),
                enabled: self.enabled,
                accelerator: None,
                item_type: "submenu".to_string(),
                checked: None,
                icon: None,
                children: Some(self.children.as_ref().unwrap()
                    .iter()
                    .map(|c| c.borrow().to_arkts_menu_item())
                    .collect()),
            },
            MenuItemType::Separator => ArkTsMenuItem {
                id: "".to_string(),
                text: "".to_string(),
                enabled: false,
                accelerator: None,
                item_type: "separator".to_string(),
                checked: None,
                icon: None,
                children: None,
            },
            MenuItemType::Check => ArkTsMenuItem {
                id: self.id.as_ref().to_string(),
                text: self.text.clone(),
                enabled: self.enabled,
                accelerator: self.accelerator.as_ref().map(|a| a.to_string()),
                item_type: "check".to_string(),
                checked: Some(self.checked),
                icon: None,
                children: None,
            },
            MenuItemType::Icon => ArkTsMenuItem {
                id: self.id.as_ref().to_string(),
                text: self.text.clone(),
                enabled: self.enabled,
                accelerator: self.accelerator.as_ref().map(|a| a.to_string()),
                item_type: "icon".to_string(),
                checked: None,
                icon: self.icon.as_ref().map(|i| i.to_base64()),
                children: None,
            },
            MenuItemType::Predefined => {
                let predefined_type = self.predefined_item_type.as_ref().unwrap();
                if predefined_type == &PredefinedMenuItemType::Separator {
                    ArkTsMenuItem {
                        id: "".to_string(),
                        text: "".to_string(),
                        enabled: false,
                        accelerator: None,
                        item_type: "separator".to_string(),
                        checked: None,
                        icon: None,
                        children: None,
                    }
                } else {
                    // 其他 Predefined 渲染为普通 MenuItem
                    ArkTsMenuItem {
                        id: self.id.as_ref().to_string(),
                        text: self.text.clone(),
                        enabled: self.enabled,
                        accelerator: self.accelerator.as_ref().map(|a| a.to_string()),
                        item_type: "predefined".to_string(),
                        checked: None,
                        icon: None,
                        children: None,
                    }
                }
            },
        }
    }
    
    fn kind(&self, c: Rc<RefCell<MenuChild>>) -> MenuItemKind {
        match self.item_type {
            MenuItemType::Submenu => MenuItemKind::Submenu(crate::Submenu {
                id: Rc::new(self.id.clone()),
                inner: c,
            }),
            MenuItemType::MenuItem => MenuItemKind::MenuItem(crate::MenuItem {
                id: Rc::new(self.id.clone()),
                inner: c,
            }),
            MenuItemType::Predefined => MenuItemKind::Predefined(crate::PredefinedMenuItem {
                id: Rc::new(self.id.clone()),
                inner: c,
            }),
            MenuItemType::Check => MenuItemKind::Check(crate::CheckMenuItem {
                id: Rc::new(self.id.clone()),
                inner: c,
            }),
            MenuItemType::Icon => MenuItemKind::Icon(crate::IconMenuItem {
                id: Rc::new(self.id.clone()),
                inner: c,
            }),
        }
    }
}
```

### 6.3 MenuEvent 处理（待添加）

```rust
// muda/src/platform_impl/ohos/event.rs

use crate::MenuEvent;
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::sync::LazyLock;

static MENU_CHANNEL: LazyLock<(Sender<MenuEvent>, Receiver<MenuEvent>)> = LazyLock::new(unbounded);

pub fn menu_event_receiver() -> &'static Receiver<MenuEvent> {
    &MENU_CHANNEL.1
}

pub fn emit_menu_event(menu_id: String) {
    let id = crate::MenuId::new(menu_id);
    MENU_CHANNEL.0.send(MenuEvent { id }).ok();
}
```

**NAPI 注册**：
```rust
// openharmony-ability/crates/ability/src/menu.rs

#[napi]
pub fn emit_menu_event(menu_id: String) {
    muda::platform_impl::ohos::emit_menu_event(menu_id);
}
```

### 6.4 Icon 处理（待添加 to_base64）

```rust
// muda/src/platform_impl/ohos/icon.rs

use crate::icon::Icon;

pub(crate) struct OhosIcon {
    rgba: Vec<u8>,
    width: u32,
    height: u32,
}

impl OhosIcon {
    pub fn from_icon(icon: &Icon) -> Self {
        Self {
            rgba: icon.rgba.clone(),
            width: icon.width,
            height: icon.height,
        }
    }
    
    pub fn to_base64(&self) -> String {
        // PNG 编码后 base64
        // 用于 ArkTS MenuItem.startIcon
        format!("data:image/png;base64,{}", encode_png_base64(&self.rgba, self.width, self.height))
    }
}

pub(crate) type PlatformIcon = OhosIcon;
```

---

## 七、Cargo.toml 配置（需修改）

### 7.1 muda/Cargo.toml（待添加 OHOS 依赖）

```toml
# muda/Cargo.toml

[target.'cfg(target_env = "ohos")'.dependencies]
openharmony-ability = { path = "../openharmony-ability/crates/ability" }
serde = { version = "1", features = ["derive"] }

[target.'cfg(target_env = "ohos")'.dev-dependencies]
# 测试依赖
```

**不需要添加 `ohos` feature**，因为：
1. OHOS 是独立编译目标 (`target_env = "ohos"`)
2. 依赖通过 cfg 条件编译自动引入
3. 与 Windows/macOS/Linux 平台隔离方式一致

### 7.2 tauri/Cargo.toml（关键修改）

**当前配置（错误）**：
```toml
# tauri/crates/tauri/Cargo.toml 第 88-95 行

# desktop
[target.'cfg(all(any(target_os = "linux", ..., target_os = "windows", target_os = "macos"), not(target_env = "ohos")))'.dependencies]
muda = { version = "0.17", default-features = false, features = [
  "serde",
  "gtk",
] }
tray-icon = { version = "0.22", ... }
```

**问题分析**：
- `not(target_env = "ohos")` 排除了 OHOS
- 导致 tauri::menu 在 OHOS 上无法编译（依赖 muda::* 类型）

**目标配置（修改）**：
```toml
# tauri/crates/tauri/Cargo.toml

# desktop (包括 OHOS)
[target.'cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "openbsd",
  target_os = "netbsd",
  target_os = "windows",
  target_os = "macos"
))'.dependencies]
muda = { version = "0.17", default-features = false, features = [
  "serde",
  "gtk",
] }
tray-icon = { version = "0.22", default-features = false, features = [
  "serde",
], optional = true }

# OHOS
[target.'cfg(target_env = "ohos")'.dependencies]
muda = { path = "../../muda" }
tray-icon = { path = "../../tray-icon", optional = true }
```

**关键点**：
1. 移除 `not(target_env = "ohos")` 条件，让桌面平台正常依赖 muda
2. 新增 OHOS 专用配置，使用 path 依赖本地 muda/tray-icon
3. OHOS 不需要 gtk feature，使用默认 features

### 7.3 tray-icon/Cargo.toml（待添加 OHOS 依赖）

```toml
# tray-icon/Cargo.toml

[target.'cfg(target_env = "ohos")'.dependencies]
muda = { path = "../muda" }
openharmony-ability = { path = "../openharmony-ability/crates/ability" }
```

---

## 八、tauri::menu 源码分析

### 8.1 为什么必须依赖 muda

从 `tauri/crates/tauri/src/menu/` 源码分析：

| 文件 | 行号 | muda 类型使用 |
|------|------|---------------|
| `menu.rs` | 15-16 | `muda::ContextMenu`, `muda::MenuId` |
| `normal.rs` | 37, 73 | `muda::MenuItem::new()`, `muda::MenuItem::with_id()` |
| `submenu.rs` | 14 | `muda::{ContextMenu, Icon as MudaIcon, MenuId}` |
| `predefined.rs` | 20, 39, 57 等 | `muda::PredefinedMenuItem::separator()`, `copy()`, `cut()` 等 |
| `check.rs` | 38, 75 | `muda::CheckMenuItem::new()`, `muda::CheckMenuItem::with_id()` |
| `icon.rs` | 41 | `muda::IconMenuItem::new()` |

**结论**：tauri::menu 在所有平台（包括 OHOS）都直接使用 muda 类型，无法绕过。

### 8.2 popup 平台分支

`tauri/crates/tauri/src/menu/menu.rs:49-83` 和 `submenu.rs:42-76` 有平台分支：

```rust
#[cfg(target_os = "macos")]
if let Ok(view) = window.ns_view() {
  unsafe {
    self_.inner().show_context_menu_for_nsview(view as _, position);
  }
}

#[cfg(all(any(target_os = "linux", ...), not(target_env = "ohos")))]
if let Ok(w) = window.gtk_window() {
  self_.inner().show_context_menu_for_gtk_window(w.as_ref(), position);
}

#[cfg(windows)]
if let Ok(hwnd) = window.hwnd() {
  unsafe {
    self_.inner().show_context_menu_for_hwnd(hwnd.0 as _, position);
  }
}
```

**需要新增 OHOS 分支**：
```rust
#[cfg(target_env = "ohos")]
if let Ok(component) = window.ohos_component() {
  self_.inner().show_context_menu_for_component(component, position);
}
```

---

## 九、与 tray 模块 Phase 0 的关系

### 9.1 共享设计

tray 模块的 `phase0-muda-ohos-design.md` 设计了 statusBar 托盘菜单：

```
tray Phase 0:
    └── statusBarManager.publish() → 托盘右键菜单
            └── StatusBarMenuItem (OHOS 特有)
```

menu 模块的 Phase 0 设计了通用 popup 菜单：

```
menu Phase 0:
    └── bindContextMenu(isShown) → 任意位置 popup 菜单
            └── MenuItem (ArkUI 通用组件)
```

### 9.2 区别

| 维度 | tray Phase 0 | menu Phase 0 |
|------|--------------|--------------|
| 菜单位置 | 系统托盘 | 任意位置 |
| OHOS API | statusBarManager | bindContextMenu |
| 菜单类型 | StatusBarMenuItem | MenuItem |
| 适用场景 | 托盘右键菜单 | 应用内 popup 菜单 |

### 9.3 合并策略

**两个模块共享 `muda::platform_impl::ohos` 基础实现**：

```
muda/src/platform_impl/ohos/
├── mod.rs          # 公共部分
├── menu.rs         # Menu 通用实现
├── menu_child.rs   # MenuItem 通用实现
├── event.rs        # MenuEvent 通用处理
├── icon.rs         # Icon 通用处理
├── util.rs         # Counter, AddOp, ArkTsMenuItem
├── accelerator.rs  # stub
└── predefined.rs   # PredefinedMenuItem 专用执行逻辑
```

---

## 十、验证方式

| 验证项 | 方式 | 说明 |
|--------|------|------|
| Menu::new() 返回值 | **auto** | `assert!(menu.id().len() > 0)` |
| MenuItem::text() 返回值 | **auto** | `assert_eq!(item.text(), "Test")` |
| Submenu::items() 返回值 | **auto** | `assert!(submenu.items().len() > 0)` |
| CheckMenuItem::checked() | **auto** | `assert!(check.is_checked())` |
| Menu::build_arkts_menu() | **auto** | 验证输出 JSON 结构 |
| stub 方法返回 Error | **auto** | `assert!(menu.init_for_hwnd(0).is_err())` |
| popup 菜单显示 | **manual** | 人工确认菜单可见 |
| 菜单项点击事件 | **manual** | 验证 MenuEvent.id 正确 |

---

## 十一、依赖关系

**上游依赖**：
- openharmony-ability crate (NAPI bridge)
- serde (ArkTsMenuItem 序列化)
- crossbeam-channel (已有)

**下游使用**：
- tray-icon（托盘右键菜单）
- tauri::menu（应用 popup 菜单）

---

## 十二、风险项

| 集险 | 描述 | 应对 |
|------|------|------|
| accelerator stub | OHOS 不支持快捷键 | 仅显示文本，忽略功能 |
| Window Menu Bar stub | OHOS 无传统菜单栏 | 返回 `NotSupportedOnPlatform` |
| predefined 执行 | 需在 Rust 或 ArkTS 实现 | Phase 3 专门处理 |
| menuId 映射 | MenuId ↔ ArkTS string 转换 | 直接使用字符串 |

---

## 十三、完成后通知

本 Phase 完成后，通知：
1. **tray 模块 Phase 1**: 可使用 muda OHOS 后端
2. **menu 模块 Phase 1**: 可开始基础架构实现

---

## 十四、参考文件

| 文件 | 说明 |
|------|------|
| `muda/src/platform_impl/mod.rs` | 平台模块选择 |
| `muda/src/platform_impl/windows/mod.rs` | Windows 实现（参考 HMENU） |
| `muda/src/platform_impl/macos/mod.rs` | macOS 实现（参考 NSMenu） |
| `muda/src/platform_impl/gtk/mod.rs` | gtk 实现（参考 stub 模式） |
| `muda/Cargo.toml` | 依赖配置 |
| `tauri/tray/doc/impl/phase0-muda-ohos-design.md` | tray 模块 Phase 0 设计 |
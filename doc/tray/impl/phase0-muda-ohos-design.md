# Phase 0: muda OHOS 后端设计

> 版本：v1.1
> 更新时间：2026-05-14
> 目标：为 muda crate 实现 OHOS 平台支持

---

## 一、架构依赖

### 1.1 依赖关系图

```
┌─────────────────────────────────────────────────────────────────────┐
│                         ArkTS / OHOS 系统                           │
│                                                                     │
│  ┌─────────────────────┐     ┌──────────────────────────────────┐  │
│  │  helper.statusBar   │     │  helper.emit_menu_event(menuId)  │  │
│  │  Manager API        │     │  (NAPI 调用点)                   │  │
│  └─────────────────────┘     └──────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
                    │                           │
                    │ NAPI                      │ NAPI
                    ▼                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    openharmony-ability                              │
│                                                                     │
│  ┌─────────────────────┐     ┌──────────────────────────────────┐  │
│  │  statusbar 模块      │     │  menu 模块 (feature="menu")      │  │
│  │  - add_to_status_bar│     │  - emit_menu_event() #[napi]     │  │
│  │  - update_icon/menu │     │  - menu_event_receiver()         │  │
│  │  - 事件监听         │     │    (Rust API)                    │  │
│  └─────────────────────┘     └──────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
                    │                           │
                    │                           │
                    ▼                           ▼
┌────────────────────────────────────┐  ┌─────────────────────────────┐
│           tray-icon                │  │           muda              │
│                                    │  │                             │
│  依赖 statusbar 模块               │  │ 依赖 menu 模块              │
│  - 调用 statusBarManager API       │  │ - menu_event_receiver()     │
│  - 图标创建/更新/删除              │  │ - init_menu_event_listener()│
│                                    │  │                             │
└────────────────────────────────────┘  └─────────────────────────────┘
```

### 1.2 与其他平台对比

| 平台 | muda 依赖 | tray-icon 依赖 | 系统接口库 |
|------|-----------|----------------|-----------|
| **Windows** | `windows-sys` | `windows-sys` | Win32 API |
| **macOS** | `objc2-app-kit` | `objc2-app-kit` | Objective-C API |
| **Linux** | `gtk` | `libappindicator` + `muda` | GTK |
| **OHOS** | `openharmony-ability` (menu feature) | `openharmony-ability` (statusbar) | NAPI 封装 |

### 1.3 muda 使用 openharmony-ability

```toml
# muda/Cargo.toml Line 81-86
[target.'cfg(target_env = "ohos")'.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
base64 = "0.22"
png = "0.18"
openharmony-ability = { path = "../openharmony-ability/crates/ability", features = ["menu"] }
```

```rust
// muda/src/platform_impl/ohos/mod.rs Line 397-405
fn start_event_listener() {
    std::thread::spawn(|| {
        let receiver = openharmony_ability::menu::menu_event_receiver();
        while let Ok(menu_id) = receiver.recv() {
            crate::MenuEvent::send(crate::MenuEvent { id: crate::MenuId::new(menu_id) });
        }
    });
}

pub fn init_menu_event_listener() {
    start_event_listener();
}
```

---

## 二、背景与决策

### 1.1 为什么在 muda 中实现

| 维度 | muda 实现 | tray-icon 转换 |
|------|-----------|----------------|
| **架构一致性** | ✓ 符合 muda 现有架构（windows/macos/gtk/ohos） | ✗ tray-icon 特有实现 |
| **复用性** | ✓ tauri menu module、tray-icon 都可使用 | ✗ 仅 tray-icon 可用 |
| **维护性** | ✓ 菜单逻辑集中一处 | ✗ 维护分散 |
| **事件统一** | ✓ 使用 muda::MenuEvent 现有机制 | ✗ 需额外映射 |

### 1.2 muda 使用范围

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
            └── muda::Menu (Window Menu Bar)
            │   OHOS 不支持，但需 stub 避免编译错误
```

---

## 二、OHOS 菜单能力分析

### 2.1 OHOS 支持

| 功能 | OHOS API | 说明 |
|------|----------|------|
| 托盘右键菜单 | statusBarManager.publish() | ✓ 支持 |
| MenuItem | StatusBarMenuItem | ✓ 支持 |
| Submenu | StatusBarMenuItem.subMenu | ✓ 支持 |
| Separator | 空标题 + disabled | ✓ 模拟 |
| 菜单项点击事件 | rightMenuClick | ✓ 支持 |

### 2.2 OHOS 不支持（需 stub）

| 功能 | OHOS | 处理方式 |
|------|------|----------|
| Window Menu Bar | ✗ 无传统桌面菜单栏 | stub：API 存在但无实际效果 |
| CheckMenuItem | ✗ 无 checkbox 菜单项 | stub：渲染为普通文本 |
| IconMenuItem | ✗ 菜单项不支持图标 | stub：忽略图标 |
| accelerator | ✗ 无快捷键系统 | stub：忽略快捷键 |
| PredefinedMenuItem（除 Separator） | ✗ 无 Copy/Cut/Paste | stub：渲染为普通文本或隐藏 |

---

## 三、架构设计

### 3.1 模块结构

```
muda/src/platform_impl/ohos/
├── mod.rs          # OHOS 平台入口
│   └── pub struct Menu
│   └── pub struct MenuChild
│
├── menu.rs         # Menu 实现
│   └── Menu::new()
│   └── Menu::items()
│   └── Menu::add_menu_item()
│
├── menu_child.rs   # MenuChild 实现
│   └── MenuChild::new()
│   └── MenuChild::new_submenu()
│   └── MenuChild::text()
│   └── MenuChild::items()
│
├── context.rs      # ContextMenu trait 实现
│   └── Menu::ohos_context_menu()
│   └── Submenu::ohos_context_menu()
│
└── event.rs        # MenuEvent OHOS 处理
│   └── register_menu_click_handler()
│   └── menu_event_receiver()
```

### 3.2 公共 trait 实现

```rust
// muda/src/platform_impl/ohos/mod.rs

pub struct Menu {
    id: MenuId,
    children: Vec<Rc<RefCell<MenuChild>>>,
    ohos_menu: Option<StatusBarMenu>,  // OHOS 原生菜单
}

pub struct MenuChild {
    item_type: MenuItemType,
    text: String,
    enabled: bool,
    id: MenuId,
    
    // submenu fields
    children: Option<Vec<Rc<RefCell<MenuChild>>>>,
}

// ContextMenu trait 实现
impl ContextMenu for Menu {
    fn as_menu(&self) -> Option<&Menu> { Some(self) }
    
    // OHOS 专属方法
    fn ohos_context_menu(&self) -> StatusBarMenu {
        self.build_ohos_menu()
    }
}

impl ContextMenu for Submenu {
    fn as_submenu(&self) -> Option<&Submenu> { Some(self) }
    
    fn ohos_context_menu(&self) -> StatusBarMenu {
        self.inner.borrow().build_ohos_menu()
    }
}
```

---

## 四、核心实现

### 4.1 Menu 实现

```rust
// muda/src/platform_impl/ohos/menu.rs

impl Menu {
    pub fn new(id: Option<MenuId>) -> Self {
        Self {
            id: id.unwrap_or_else(|| MenuId::new_unique()),
            children: Vec::new(),
            ohos_menu: None,
        }
    }
    
    pub fn id(&self) -> &MenuId {
        &self.id
    }
    
    pub fn add_menu_item(&mut self, item: &dyn IsMenuItem, op: AddOp) -> Result<()> {
        let child = item.child();
        match op {
            AddOp::Append => self.children.push(child),
            AddOp::Insert(pos) => self.children.insert(pos, child),
        }
        Ok(())
    }
    
    pub fn items(&self) -> Vec<MenuItemKind> {
        self.children.iter().map(|c| {
            c.borrow().kind(c.clone())
        }).collect()
    }
    
    pub fn build_ohos_menu(&self) -> StatusBarMenu {
        StatusBarMenu {
            items: self.children.iter().map(|c| {
                c.borrow().to_ohos_menu_item()
            }).collect(),
        }
    }
}
```

### 4.2 MenuChild 实现

```rust
// muda/src/platform_impl/ohos/menu_child.rs

impl MenuChild {
    pub fn new(text: &str, enabled: bool, accelerator: Option<KeyAccelerator>, id: Option<MenuId>) -> Self {
        Self {
            item_type: MenuItemType::MenuItem,
            text: text.to_string(),
            enabled,
            id: id.unwrap_or_else(|| MenuId::new_unique()),
            children: None,
        }
    }
    
    pub fn new_submenu(text: &str, enabled: bool, id: Option<MenuId>) -> Self {
        Self {
            item_type: MenuItemType::Submenu,
            text: text.to_string(),
            enabled,
            id: id.unwrap_or_else(|| MenuId::new_unique()),
            children: Some(Vec::new()),
        }
    }
    
    pub fn text(&self) -> String {
        self.text.clone()
    }
    
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    pub fn items(&self) -> Vec<MenuItemKind> {
        if let Some(children) = &self.children {
            children.iter().map(|c| c.borrow().kind(c.clone())).collect()
        } else {
            Vec::new()
        }
    }
    
    pub fn to_ohos_menu_item(&self) -> StatusBarMenuItem {
        match self.item_type {
            MenuItemType::MenuItem => StatusBarMenuItem {
                menuCode: self.id.as_ref(),
                title: self.text.clone(),
                isEnabled: self.enabled,
                notifyOnly: true,  // 强制设置，触发事件
                subMenu: None,
            },
            MenuItemType::Submenu => StatusBarMenuItem {
                menuCode: self.id.as_ref(),
                title: self.text.clone(),
                isEnabled: self.enabled,
                notifyOnly: false,
                subMenu: Some(self.build_submenu()),
            },
            MenuItemType::Predefined => {
                // Separator: 空标题 + disabled
                StatusBarMenuItem {
                    menuCode: "",
                    title: "",
                    isEnabled: false,
                    notifyOnly: false,
                    subMenu: None,
                }
            },
            MenuItemType::Check => {
                // stub: 渲染为普通文本
                StatusBarMenuItem {
                    menuCode: self.id.as_ref(),
                    title: format!("{} {}", if self.checked { "✓" } else { " " }, self.text),
                    isEnabled: self.enabled,
                    notifyOnly: true,
                    subMenu: None,
                }
            },
            MenuItemType::Icon => {
                // stub: 忽略图标
                StatusBarMenuItem {
                    menuCode: self.id.as_ref(),
                    title: self.text.clone(),
                    isEnabled: self.enabled,
                    notifyOnly: true,
                    subMenu: None,
                }
            },
        }
    }
    
    fn build_submenu(&self) -> StatusBarMenu {
        if let Some(children) = &self.children {
            StatusBarMenu {
                items: children.iter().map(|c| c.borrow().to_ohos_menu_item()).collect(),
            }
        } else {
            StatusBarMenu { items: Vec::new() }
        }
    }
}
```

### 4.3 ContextMenu 实现

```rust
// muda/src/platform_impl/ohos/context.rs

impl ContextMenu for Menu {
    fn as_menu(&self) -> Option<&Menu> {
        Some(self)
    }
}

impl ContextMenu for Submenu {
    fn as_submenu(&self) -> Option<&Submenu> {
        Some(self)
    }
}

// OHOS 专属方法（通过 extension trait）
pub trait OhosContextMenu: ContextMenu {
    fn ohos_context_menu(&self) -> StatusBarMenu;
}

impl OhosContextMenu for Menu {
    fn ohos_context_menu(&self) -> StatusBarMenu {
        self.build_ohos_menu()
    }
}

impl OhosContextMenu for Submenu {
    fn ohos_context_menu(&self) -> StatusBarMenu {
        self.inner.borrow().build_ohos_menu()
    }
}
```

### 4.4 MenuEvent 处理

```rust
// muda/src/platform_impl/ohos/event.rs

use crossbeam_channel::{unbounded, Receiver};

static MENU_CHANNEL: Lazy<(Sender<MenuEvent>, Receiver<MenuEvent>)> = Lazy::new(unbounded);

pub fn register_menu_click_handler(status_bar_manager: &StatusBarManager) {
    status_bar_manager.on('rightMenuClick', |event| {
        let menu_code = event.menuCode;
        let menu_id = MenuId::from(menu_code);
        MenuEvent::send(MenuEvent { id: menu_id });
    });
}

pub fn menu_event_receiver() -> &'static Receiver<MenuEvent> {
    &MENU_CHANNEL.1
}
```

---

## 五、Cargo.toml 配置

```toml
# muda/Cargo.toml

[target.'cfg(target_env = "ohos")'.dependencies]
openharmony-ability = { path = "../openharmony-ability/crates/ability" }

# 添加 ohos feature
[features]
ohos = []
```

```toml
# muda/src/platform_impl/mod.rs

#[cfg(target_env = "ohos")]
#[path = "ohos/mod.rs"]
mod platform;
```

---

## 六、验证方式

| 验证项 | 方式 | 说明 |
|--------|------|------|
| Menu::new() 返回值 | **auto** | 验证 menu.id 存在且类型正确 |
| MenuItem::text() 返回值 | **auto** | 验证文本正确 |
| Submenu::items() 返回值 | **auto** | 验证子菜单项数量 |
| 托盘菜单显示 | **manual** | 无查询 API，需人工确认右键菜单可见 |
| 菜单项点击事件 | **manual** | 需用户点击，验证 MenuEvent.id 正确 |

---

## 七、依赖关系

**上游依赖**：
- OHOS statusBarManager API（通过 openharmony-ability::statusbar）

**下游使用**：
- tray-icon（托盘右键菜单）
- tauri::menu（Window Menu Bar stub）

---

## 八、风险项

| 风险 | 描述 | 应对 |
|------|------|------|
| CheckMenuItem stub | OHOS 不支持 checkbox | 渲染为带 ✓ 符号的文本 |
| IconMenuItem stub | OHOS 不支持菜单项图标 | 忽略图标 |
| accelerator stub | OHOS 不支持快捷键 | 忽略快捷键 |
| menuCode 映射 | MenuId ↔ menuCode 转换 | 直接使用字符串 |

---

## 九、完成后通知

本 Phase 完成后，通知 Phase 1 和 Phase 2 可以开始依赖 muda OHOS 后端。
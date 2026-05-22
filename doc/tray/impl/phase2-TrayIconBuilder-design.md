# API 设计：TrayIconBuilder

> 职责：构建托盘图标，提供配置方法
> 代码位置：`tray-icon/src/platform_impl/ohos/mod.rs`
> 独立性：✓ 可独立实现、独立验证（依赖 phase1 ohos-statusbar 模块)

## 参考资料

- [../reference/status_bar_api.md](../reference/status_bar_api.md) - OHOS statusBarManager API 完整文档
- [../reference/tray-reference.md](../reference/tray-reference.md) - OHOS 托盘方案完整示例（包含后台保活、事件处理等）

---

## 一、接口定义

### 1.1 TrayIconBuilder 公开接口（由 tray-icon lib.rs 定义）

```rust
// tray-icon/src/lib.rs 已定义，无需修改

pub struct TrayIconBuilder {
    id: TrayIconId,
    attrs: TrayIconAttributes,
}

impl TrayIconBuilder {
    pub fn new() -> Self;
    pub fn with_id<I: Into<TrayIconId>>(id: I) -> Self;
    pub fn with_icon(icon: Icon) -> Self;
    pub fn with_menu(menu: Box<dyn menu::ContextMenu>) -> Self;
    pub fn with_tooltip<S: AsRef<str>>(s: S) -> Self;
    pub fn with_title<S: AsRef<str>>(title: S) -> Self;
    pub fn with_icon_as_template(is_template: bool) -> Self;
    pub fn with_menu_on_left_click(enable: bool) -> Self;
    pub fn id(&self) -> &TrayIconId;
    pub fn build(self) -> Result<TrayIcon>;
}
```

### 1.2 TrayIconAttributes 结构

```rust
// tray-icon/src/lib.rs 已定义

pub struct TrayIconAttributes {
    pub tooltip: Option<String>,
    pub menu: Option<Box<dyn menu::ContextMenu>>,
    pub icon: Option<Icon>,
    pub temp_dir_path: Option<PathBuf>,
    pub icon_is_template: bool,
    pub menu_on_left_click: bool,
    pub menu_on_right_click: bool,
    pub title: Option<String>,
}
```

---

## 二、OHOS 实现设计

### 2.1 build() 实现流程

```
TrayIconBuilder::build()
    │
    ├── 1. 获取 OpenHarmonyApp context
    │       └── 从全局 OHOS_APP 获取
    │
    ├── 2. 转换 Icon → StatusBarIcon
    │       ├── rgba → white PixelMap (背景 255,255,255)
    │       ├── rgba → black PixelMap (背景 0,0,0)
    │       └── 尺寸缩放至 24x24
    │
    ├── 3. 构建 QuickOperation
    │       ├── title ← attrs.title
    │       ├── ability_name ← "" (空字符串触发事件模式)
    │       ├── height ← 200
    │       └── module_name ← None
    │
    ├── 4. 转换 Menu → StatusBarMenuItem[]
    │       │
    │       ├── 使用 muda::OhosContextMenu::ohos_context_menu()
    │       │   （Phase 0 已实现）
    │       │
    │       └── muda 内部处理：
    │               ├── menuCode ← item.id()
    │               ├── notifyOnly ← true
    │               ├── abilityName ← ""
    │               ├── Separator: 空标题 + disabled
    │               ├── CheckMenuItem: stub（带 ✓ 文本）
    │               └── IconMenuItem: stub（忽略图标）
    │
    ├── 5. 构建 StatusBarItem
    │       ├── icons
    │       ├── quick_operation
    │       ├── status_bar_group_menu
    │       └── hover_tips ← attrs.tooltip (版本兼容)
    │
    ├── 6. 调用 add_to_status_bar(context, item)
    │       └── OHOS statusBarManager.addToStatusBar()
    │
    ├── 7. 注册两个事件监听
    │       ├── register_icon_click_handler() → 左键点击图标
    │       └── register_menu_click_handler() → 右键菜单项点击
    │
    ├── 8. 启动事件转发线程
    │       ├── 监听 icon_click_receiver()
    │       ├── 监听 menu_click_receiver()
    │       ├── 转换为 TrayIconEvent
    │       └── TrayIconEvent::send()
    │
    └── 9. 返回 TrayIcon 实例
```

### 2.2 TrayIcon 结构

```rust
// platform_impl::ohos::TrayIcon
// 注意：实际使用 RefCell 包裹 attrs 和 is_visible，
// 因为 set_menu 等方法需要内部可变性（&mut self 但 attrs.menu 是 Box<dyn ContextMenu>）

pub struct TrayIcon {
    attrs: RefCell<TrayIconAttributes>,
    is_visible: RefCell<bool>,
}
```

### 2.3 TrayIcon 方法实现

| 方法 | OHOS 实现 | 版本要求 |
|------|-----------|----------|
| `set_icon(icon)` | `update_status_bar_icon()` | 5.0.0(12) |
| `set_menu(menu)` | `update_status_bar_menu()` | 5.0.0(12) |
| `set_tooltip(tips)` | `update_hover_tips()` | **△ 部分：6.0.2(22)**，hoverTips 非标准 tooltip |
| `set_title(title)` | **需重建** StatusBarItem | 5.0.0(12)，无 updateQuickOperationTitle API |
| `set_visible(true)` | `add_to_status_bar()` | 5.0.0(12) |
| `set_visible(false)` | `remove_from_status_bar()` | 5.0.0(12) |
| `rect()` | 返回 `None` | 不支持 |

---

## 三、核心代码实现

### 3.1 TrayIcon::new

> **注意**：以下为实际实现的精简版本。实际代码使用 `RefCell` 包裹 `attrs` 和 `is_visible`。

```rust
// tray-icon/src/platform_impl/ohos/mod.rs

use crate::{TrayIconId, TrayIconAttributes, TrayIconEvent};
use once_cell::sync::OnceCell;
use std::cell::RefCell;
use std::io;

static OHOS_APP: OnceCell<openharmony_ability::OpenHarmonyApp> = OnceCell::new();

pub fn set_ohos_app(app: openharmony_ability::OpenHarmonyApp) {
    OHOS_APP.set(app).expect("OHOS_APP already set");
}

fn get_ohos_app() -> &'static openharmony_ability::OpenHarmonyApp {
    OHOS_APP.get().expect("OHOS_APP not initialized")
}

pub struct TrayIcon {
    attrs: RefCell<TrayIconAttributes>,
    is_visible: RefCell<bool>,
}

impl TrayIcon {
    pub fn new(id: TrayIconId, attrs: TrayIconAttributes) -> crate::Result<Self> {
        let app = get_ohos_app();
        
        // 1. 转换图标
        let icon = attrs.icon.as_ref().ok_or_else(|| {
            crate::Error::OsError(io::Error::new(io::ErrorKind::InvalidData, "No icon provided"))
        })?;
        let status_bar_icon = icon::icon_to_status_bar_icon(&icon.inner)?;
        
        // 2. 构建 QuickOperation
        let quick_operation = openharmony_ability::statusbar::QuickOperation {
            ability_name: String::new(),
            title: attrs.title.clone().unwrap_or_else(|| "Tauri App".to_string()),
            height: 200,
            module_name: None,
            loading_status: None,
        };
        
        // 3. 转换菜单（通过 JSON 序列化/反序列化）
        let menus = menu_to_status_bar_items(&attrs.menu);
        
        // 4. 构建 StatusBarItem
        let item = openharmony_ability::statusbar::StatusBarItem {
            icons: status_bar_icon,
            quick_operation,
            status_bar_group_menu: menus,
            hover_tips: attrs.tooltip.clone(),
        };
        
        // 5. 添加到状态栏
        openharmony_ability::statusbar::add_to_status_bar(app, &item)
            .map_err(|e| crate::Error::OhosError(e.to_string()))?;
        
        // 6. 注册事件监听
        openharmony_ability::statusbar::register_icon_click_handler()
            .map_err(|e| crate::Error::OhosError(e.to_string()))?;
        openharmony_ability::statusbar::register_menu_click_handler()
            .map_err(|e| crate::Error::OhosError(e.to_string()))?;
        
        // 7. 注册 tray ID 并启动事件转发
        event::register_tray_id(id);
        event::start_event_forward_thread();
        
        Ok(Self {
            attrs: RefCell::new(attrs),
            is_visible: RefCell::new(true),
        })
    }
```
    
    pub fn set_icon(&mut self, icon: Option<crate::Icon>) -> crate::Result<()> {
        let app = get_ohos_app();
        if let Some(i) = &icon {
            let status_bar_icon = icon::icon_to_status_bar_icon(&i.inner)?;
            openharmony_ability::statusbar::update_status_bar_icon(app, &status_bar_icon)
                .map_err(|e| crate::Error::OhosError(e.to_string()))?;
        }
        self.attrs.borrow_mut().icon = icon;
        Ok(())
    }
    
    pub fn set_menu(&mut self, menu: Option<Box<dyn crate::menu::ContextMenu>>) {
        let app = get_ohos_app();
        let menus = menu_to_status_bar_items(&menu);
        if let Some(m) = menus {
            openharmony_ability::statusbar::update_status_bar_menu(app, &m)
                .map_err(|e| crate::Error::OhosError(e.to_string()))
                .ok();
        }
        self.attrs.borrow_mut().menu = menu;
    }
    
    pub fn set_tooltip<S: AsRef<str>>(&mut self, tooltip: Option<S>) -> crate::Result<()> {
        let app = get_ohos_app();
        let tips = tooltip.map(|s| s.as_ref().to_string());
        if let Some(t) = &tips {
            if !t.is_empty() && t.len() <= 128 {
                openharmony_ability::statusbar::update_hover_tips(app, t)
                    .map_err(|e| crate::Error::OhosError(e.to_string()))
                    .ok();
            }
        }
        self.attrs.borrow_mut().tooltip = tips;
        Ok(())
    }
    
    pub fn set_title<S: AsRef<str>>(&mut self, _title: Option<S>) {}
    
    pub fn set_visible(&mut self, visible: bool) -> crate::Result<()> {
        let app = get_ohos_app();
        
        if visible && !*self.is_visible.borrow() {
            let item = build_item_from_attrs(&self.attrs.borrow())?;
            openharmony_ability::statusbar::add_to_status_bar(app, &item)
                .map_err(|e| crate::Error::OhosError(e.to_string()))?;
            // 重新注册事件监听
            openharmony_ability::statusbar::register_icon_click_handler()
                .map_err(|e| crate::Error::OhosError(e.to_string()))?;
            openharmony_ability::statusbar::register_menu_click_handler()
                .map_err(|e| crate::Error::OhosError(e.to_string()))?;
            *self.is_visible.borrow_mut() = true;
        } else if !visible && *self.is_visible.borrow() {
            openharmony_ability::statusbar::remove_from_status_bar(app)
                .map_err(|e| crate::Error::OhosError(e.to_string()))
                .ok();
            openharmony_ability::statusbar::unregister_icon_click_handler()
                .map_err(|e| crate::Error::OhosError(e.to_string()))
                .ok();
            openharmony_ability::statusbar::unregister_menu_click_handler()
                .map_err(|e| crate::Error::OhosError(e.to_string()))
                .ok();
            *self.is_visible.borrow_mut() = false;
        }
        Ok(())
    }
    
    pub fn set_temp_dir_path<P: AsRef<std::path::Path>>(&mut self, _path: Option<P>) {}
    
    pub fn rect(&self) -> Option<crate::Rect> {
        None
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        if *self.is_visible.borrow() {
            let app = get_ohos_app();
            openharmony_ability::statusbar::remove_from_status_bar(app)
                .map_err(|e| crate::Error::OhosError(e.to_string()))
                .ok();
            openharmony_ability::statusbar::unregister_icon_click_handler()
                .map_err(|e| crate::Error::OhosError(e.to_string()))
                .ok();
            openharmony_ability::statusbar::unregister_menu_click_handler()
                .map_err(|e| crate::Error::OhosError(e.to_string()))
                .ok();
        }
    }
}
```

### 3.2 图标转换

> **注意**：实际 `icon_to_status_bar_icon` 接受 `&PlatformIcon` 而非 `&Option<Icon>`，
> 调用方负责解包 `icon.inner`。StatusBarIcon 使用 `RefCell<Option<Object>>` 包裹 PixelMap。

```rust
// tray-icon/src/platform_impl/ohos/icon.rs

use std::cell::RefCell;

#[derive(Debug, Clone)]
pub struct PlatformIcon {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl PlatformIcon {
    pub fn from_rgba(rgba: Vec<u8>, width: u32, height: u32) -> Result<Self, crate::icon::BadIcon> {
        Ok(Self { rgba, width, height })
    }

    pub fn write_to_png(&self, _path: impl AsRef<std::path::Path>) -> crate::Result<()> {
        Err(crate::Error::Unsupported)
    }
}

pub fn icon_to_status_bar_icon(icon: &PlatformIcon) -> crate::Result<openharmony_ability::statusbar::StatusBarIcon> {
    let rgba = &icon.rgba;
    let (width, height) = (icon.width, icon.height);
    let target_size = 24;
    
    let scaled_rgba = if width != target_size || height != target_size {
        scale_rgba(rgba, width, height, target_size, target_size)
    } else {
        rgba.clone()
    };
    
    let white_rgba = blend_rgba_with_background(&scaled_rgba, [255, 255, 255]);
    let black_rgba = blend_rgba_with_background(&scaled_rgba, [0, 0, 0]);
    
    let white_pixelmap = openharmony_ability::statusbar::create_pixelmap_from_rgba(&white_rgba, target_size, target_size)
        .map_err(|e| crate::Error::OhosError(e.to_string()))?;
    let black_pixelmap = openharmony_ability::statusbar::create_pixelmap_from_rgba(&black_rgba, target_size, target_size)
        .map_err(|e| crate::Error::OhosError(e.to_string()))?;
    
    Ok(openharmony_ability::statusbar::StatusBarIcon {
        white: RefCell::new(Some(white_pixelmap)),
        black: RefCell::new(Some(black_pixelmap)),
    })
}

fn scale_rgba(rgba: &[u8], src_w: u32, src_h: u32, dst_w: u32, dst_h: u32) -> Vec<u8> {
    // 最近邻缩放
    let mut result = Vec::with_capacity(dst_w as usize * dst_h as usize * 4);
    for dst_y in 0..dst_h {
        for dst_x in 0..dst_w {
            let src_x = (dst_x as f32 * src_w as f32 / dst_w as f32) as u32;
            let src_y = (dst_y as f32 * src_h as f32 / dst_h as f32) as u32;
            let src_idx = (src_y * src_w + src_x) as usize * 4;
            result.push(rgba[src_idx]);
            result.push(rgba[src_idx + 1]);
            result.push(rgba[src_idx + 2]);
            result.push(rgba[src_idx + 3]);
        }
    }
    result
}

fn blend_rgba_with_background(rgba: &[u8], bg: [u8; 3]) -> Vec<u8> {
    rgba.chunks(4)
        .flat_map(|pixel| {
            let a = pixel[3] as f32 / 255.0;
            let r = (pixel[0] as f32 * a + bg[0] as f32 * (1.0 - a)) as u8;
            let g = (pixel[1] as f32 * a + bg[1] as f32 * (1.0 - a)) as u8;
            let b = (pixel[2] as f32 * a + bg[2] as f32 * (1.0 - a)) as u8;
            vec![r, g, b, 255]  // 注意：alpha 固定为 255
        })
        .collect()
}
```

### 3.3 菜单转换（JSON 序列化/反序列化方式）

> **注意**：实际实现不使用 `OhosContextMenu` trait 直接获取 `StatusBarMenu`，
> 而是通过 `m.ohos_context_menu()` 获取 JSON 字符串，再反序列化为 `MenuJsonItem` 结构，
> 最后转换为 OHOS 菜单项。这种方式解耦了 tray-icon 和 muda 的内部类型。

```rust
fn menu_to_status_bar_items(
    menu: &Option<Box<dyn crate::menu::ContextMenu>>
) -> Option<Vec<Vec<openharmony_ability::statusbar::StatusBarMenuItem>>> {
    menu.as_ref().map(|m| {
        let json = m.ohos_context_menu();
        let menu_data: MenuJsonData = serde_json::from_str(&json).unwrap_or_default();
        vec![menu_json_to_status_bar_items(menu_data.items)]
    })
}

#[derive(serde::Deserialize, Default)]
struct MenuJsonData {
    items: Vec<MenuJsonItem>,
}

#[derive(serde::Deserialize)]
struct MenuJsonItem {
    id: String,
    text: String,
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    item_type: String,
    children: Option<Vec<MenuJsonItem>>,
}

fn menu_json_to_status_bar_items(
    items: Vec<MenuJsonItem>
) -> openharmony_ability::statusbar::StatusBarMenuItem {
    let sub_items: Vec<openharmony_ability::statusbar::StatusBarSubMenuItem> = items
        .into_iter()
        .flat_map(|item| {
            if item.item_type == "separator" {
                // 分隔线：使用特殊字符模拟
                vec![openharmony_ability::statusbar::StatusBarSubMenuItem {
                    sub_title: "──────────".to_string(),
                    menu_code: None,
                    menu_action: openharmony_ability::statusbar::StatusBarMenuAction::default(),
                    options: None,
                }]
            } else if item.item_type == "submenu" {
                // Submenu 扁平化：标题作为分组标识 + 子项缩进
                let mut result = Vec::new();
                result.push(openharmony_ability::statusbar::StatusBarSubMenuItem {
                    sub_title: format!("【{}】", item.text),
                    menu_code: None,
                    menu_action: openharmony_ability::statusbar::StatusBarMenuAction::default(),
                    options: None,
                });
                for child in item.children.unwrap_or_default() {
                    result.push(openharmony_ability::statusbar::StatusBarSubMenuItem {
                        sub_title: format!("  {}", child.text),
                        menu_code: Some(child.id),
                        menu_action: openharmony_ability::statusbar::StatusBarMenuAction {
                            ability_name: String::new(),
                            module_name: None,
                            menu_code: None,
                            notify_only: Some(true),
                        },
                        options: None,
                    });
                }
                result
            } else {
                // 普通菜单项
                vec![openharmony_ability::statusbar::StatusBarSubMenuItem {
                    sub_title: item.text,
                    menu_code: Some(item.id),
                    menu_action: openharmony_ability::statusbar::StatusBarMenuAction {
                        ability_name: String::new(),
                        module_name: None,
                        menu_code: None,
                        notify_only: Some(true),
                    },
                    options: None,
                }]
            }
        })
        .collect();

    openharmony_ability::statusbar::StatusBarMenuItem {
        title: String::new(),
        menu_code: None,
        sub_menu: Some(sub_items),
        menu_action: None,
        options: None,
    }
}
```

### 3.3.1 build_item_from_attrs 辅助函数

> `set_visible(true)` 时需要从 attrs 重建完整的 StatusBarItem，
> 因此提取了 `build_item_from_attrs` 辅助函数。

```rust
fn build_item_from_attrs(
    attrs: &TrayIconAttributes
) -> crate::Result<openharmony_ability::statusbar::StatusBarItem> {
    let icon = attrs.icon.as_ref().ok_or_else(|| {
        crate::Error::OsError(io::Error::new(io::ErrorKind::InvalidData, "No icon provided"))
    })?;
    let status_bar_icon = icon::icon_to_status_bar_icon(&icon.inner)?;
    let quick_operation = openharmony_ability::statusbar::QuickOperation {
        ability_name: String::new(),
        title: attrs.title.clone().unwrap_or_else(|| "Tauri App".to_string()),
        height: 200,
        module_name: None,
        loading_status: None,
    };
    let menus = menu_to_status_bar_items(&attrs.menu);
    Ok(openharmony_ability::statusbar::StatusBarItem {
        icons: status_bar_icon,
        quick_operation,
        status_bar_group_menu: menus,
        hover_tips: attrs.tooltip.clone(),
    })
}
```

### 3.4 事件转发线程

> **注意**：实际函数名为 `convert_icon_click` 和 `convert_menu_click`（无 `_event` 后缀）。
> `start_event_forward_thread` 使用 `AtomicBool::swap` 确保只启动一次。

```rust
use crate::{dpi::PhysicalPosition, MouseButton, MouseButtonState, Rect, TrayIconEvent, TrayIconId};
use crossbeam_channel::select;
use once_cell::sync::OnceCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

static EVENT_THREAD_STARTED: AtomicBool = AtomicBool::new(false);
static TRAY_ID: OnceCell<TrayIconId> = OnceCell::new();

pub fn register_tray_id(id: TrayIconId) {
    TRAY_ID.set(id).ok();
}

pub fn get_current_tray_id() -> TrayIconId {
    TRAY_ID.get().cloned()
        .unwrap_or_else(|| TrayIconId::new("main"))
}

pub fn start_event_forward_thread() {
    if EVENT_THREAD_STARTED.swap(true, Ordering::Relaxed) {
        return;
    }
    thread::spawn(move || {
        let icon_receiver = openharmony_ability::statusbar::icon_click_receiver();
        let menu_receiver = openharmony_ability::statusbar::menu_click_receiver();
        loop {
            select! {
                recv(icon_receiver) -> event => {
                    if let Ok(status_bar_event) = event {
                        let tray_event = convert_icon_click(status_bar_event);
                        TrayIconEvent::send(tray_event);
                    }
                },
                recv(menu_receiver) -> event => {
                    if let Ok(status_bar_event) = event {
                        let tray_event = convert_menu_click(status_bar_event);
                        TrayIconEvent::send(tray_event);
                    }
                },
            }
        }
    });
}

fn convert_icon_click(_event: openharmony_ability::statusbar::StatusBarClickEvent) -> TrayIconEvent {
    TrayIconEvent::Click {
        id: get_current_tray_id(),
        position: PhysicalPosition::new(0.0, 0.0),
        rect: Rect::default(),
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
    }
}

fn convert_menu_click(_event: openharmony_ability::statusbar::StatusBarClickEvent) -> TrayIconEvent {
    TrayIconEvent::Click {
        id: get_current_tray_id(),
        position: PhysicalPosition::new(0.0, 0.0),
        rect: Rect::default(),
        button: MouseButton::Right,
        button_state: MouseButtonState::Up,
    }
}
```

---

## 四、架构说明

### 4.1 层级关系

```
tray-icon (纯 Rust crate，不直接调用 napi)
    │
    ├── 依赖 muda → 获取菜单 JSON (ohos_context_menu)
    │
    └── 依赖 openharmony-ability → 调用 OHOS API
            │
            ├── statusbar 模块 → 状态栏图标 API
            │   ├── add_to_status_bar()
            │   ├── update_status_bar_icon()
            │   ├── update_status_bar_menu()
            │   ├── create_pixelmap() ← **新增：从 RGBA 创建 PixelMap**
            │   └── 事件监听 (icon_click_receiver/menu_click_receiver)
            │
            └── menu 模块 → 菜单事件监听
                └── menu_event_receiver()
```

### 4.2 关键设计原则

**tray-icon 不能直接调用 napi_ohos**，必须通过 openharmony-ability 封装 Rust 接口：

| 操作 | tray-icon 调用 | openharmony-ability 提供 |
|------|----------------|---------------------------|
| 创建 PixelMap | `create_pixelmap_from_rgba(rgba, w, h)` | napi 调用 `image.createPixelMap` |
| 添加图标 | `add_to_status_bar(app, item)` | napi 调用 `statusBarManager.addToStatusBar` |
| 更新图标 | `update_status_bar_icon(app, icon)` | napi 调用 `statusBarManager.updateStatusBarIcon` |
| 监听事件 | `icon_click_receiver()` | napi 注册 `on('statusBarIconClick')` |

**原因**：
- napi_ohos 只能在 openharmony-ability crate 中使用（已配置 napi_derive_ohos）
- tray-icon 是纯 Rust crate，无法直接调用 NAPI

---

## 五、Cargo.toml 配置

```toml
# tray-icon/Cargo.toml 新增

[target."cfg(target_env = \"ohos\")".dependencies]
openharmony-ability = { path = "../openharmony-ability/crates/ability", features = ["menu"] }
muda = { path = "../muda" }
png = "0.18"
```

**注意**：
- muda crate 依赖（Phase 0）必须先完成 OHOS 后端实现
- openharmony-ability 必须启用 `menu` feature 来接收菜单事件

---

## 六、openharmony-ability 新增接口

需要在 `openharmony-ability/crates/ability/src/statusbar/` 添加 PixelMap 创建接口：

### 6.1 manager.rs 新增

```rust
/// 从 RGBA 数据创建 PixelMap
pub fn create_pixelmap_from_rgba(rgba: &[u8], width: u32, height: u32) -> Result<Object<'static>> {
    // TODO: 需要在主线程调用 OHOS image.createPixelMap API
    Err(Error::from_reason("create_pixelmap_from_rgba not implemented yet"))
}
```

### 6.2 types.rs 实际 StatusBarIcon 定义

> **注意**：实际使用 `RefCell<Option<Object<'static>>>` 而非直接的 `Object`，
> 允许延迟初始化和内部可变性。

```rust
pub struct StatusBarIcon {
    pub white: RefCell<Option<napi_ohos::bindgen_prelude::Object<'static>>>,
    pub black: RefCell<Option<napi_ohos::bindgen_prelude::Object<'static>>>,
}
```

---

## 七、验证方案

### 7.1 Rust 单元测试（ohos-rust-ut）

适用于纯逻辑验证（图标转换、菜单转换等）。

**验证范围**：图标处理和菜单转换逻辑

```rust
// tray-icon/src/platform_impl/ohos/icon.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgba_blend_with_white_background() {
        let rgba = vec![100, 100, 100, 128];  // 半透明灰色
        let blended = blend_rgba_with_background(&rgba, [255, 255, 255]);
        // 验证 alpha blending 计算正确
        assert!(blended[0] > 100 && blended[0] < 255);
    }

    #[test]
    fn rgba_blend_with_black_background() {
        let rgba = vec![100, 100, 100, 128];
        let blended = blend_rgba_with_background(&rgba, [0, 0, 0]);
        // 验证与黑色背景混合
        assert!(blended[0] < 100);
    }

    #[test]
    fn icon_scaled_to_24x24() {
        let rgba = vec![255u8; 48 * 48 * 4];  // 48x48
        let scaled = scale_rgba(&rgba, 48, 48, 24, 24);
        assert_eq!(scaled.len(), 24 * 24 * 4);
    }
}
```

**运行命令**：
```bash
bash D:/workspace/tauri/tauri/.claude/skills/ohos-rust-ut/scripts/run-ut.sh tray_icon::ohos::icon
```

### 7.2 端到端测试（frontend-api-testing）

**测试位置**：`examples/api/src/lib/tests/plugins.ts`

```typescript
// TrayIcon API 返回值验证（auto）
{
  name: '@tauri-apps/plugin-tray.TrayIcon.new',
  category: 'auto',
  async fn() {
    const { TrayIcon } = await import('@tauri-apps/plugin-tray');
    const icon = await createTestIcon();
    const tray = await TrayIcon.new({
      icon,
      tooltip: 'Auto Test',
      menu: createTestMenu()
    });
    
    // 验证返回值
    assert(tray !== undefined, 'tray object returned');
    assert(tray.id !== undefined, 'tray.id exists');
    assert(typeof tray.id === 'string', 'tray.id is string');
    
    // 验证方法存在
    assert(typeof tray.setIcon === 'function', 'setIcon method exists');
    assert(typeof tray.setToolTip === 'function', 'setToolTip method exists');
    assert(typeof tray.setVisible === 'function', 'setVisible method exists');
    assert(typeof tray.destroy === 'function', 'destroy method exists');
    
    // 清理
    tray.destroy();
  },
},

// TrayIconBuilder 返回值验证（auto）
{
  name: '@tauri-apps/plugin-tray.TrayIconBuilder',
  category: 'auto',
  async fn() {
    const { TrayIconBuilder } = await import('@tauri-apps/plugin-tray');
    const builder = new TrayIconBuilder()
      .icon(await createTestIcon())
      .tooltip('Builder Test')
      .id('test-tray-id');
    
    const tray = await builder.build();
    assert(tray !== undefined, 'builder returns tray');
    assert(tray.id === 'test-tray-id', 'id matches builder setting');
    
    tray.destroy();
  },
},
```

### 7.3 手动测试清单（视觉效果和事件）

**必须用 manual 测试**的原因：
- OHOS statusBarManager **无查询 API**（无法程序验证图标是否存在）
- 用户交互（点击）**无法程序模拟**

| 测试项 | 操作 | 预期结果 | 测试类型 |
|--------|------|----------|----------|
| T1 | 状态栏显示图标 | 可见 | **manual** |
| T2 | 左键点击图标 | 触发 Click 事件 | **manual** |
| T3 | 右键弹出菜单 | 菜单可见 | **manual** |
| T4 | 右键点击菜单项 | 触发 Click { Right } | **manual** |
| T5 | set_icon() 后 | 图标变化 | **manual** |
| T6 | set_tooltip() hover | 显示提示 | **manual** |
| T7 | set_visible(false) | 图标消失 | **manual** |
| T8 | set_visible(true) | 图标重现 | **manual** |

### 7.4 验证流程

```
Phase 2 验证流程
    │
    ├── 1. Rust UT 验证图标转换逻辑
    │       └── run-ut.sh tray_icon::ohos::icon
    │
    ├── 2. 端到端测试验证 TrayIcon API
    │       └── build-ohos.sh → sign-and-install.sh
    │       └── 查看 test-report.json
    │
    └── 3. 手动测试验证用户交互
            └── 点击图标 → 查看控制台输出
            └── 拉取 console-log.txt 分析事件
```

### 7.5 端到端测试配置

需要在 `examples/api` 中配置 tray plugin：

**Cargo.toml**：
```toml
[target.'cfg(target_env = "ohos")'.dependencies]
tauri-plugin-tray = { path = "../../../plugins-workspace/plugins/tray" }
```

**capabilities/run-app.json**：
```json
"tray:default"
```

参考 `frontend-api-testing` skill 的 [接入新 plugin](../../../../../.claude/skills/frontend-api-testing/SKILL.md#接入新-plugin) 章节。

---

## 八、风险项

| 风险 | 描述 | 应对 |
|------|------|------|
| muda menu 依赖 | tray-icon 依赖 muda crate | **已在 muda 实现 OHOS 后端**（Phase 0 完成） |
| Icon 内部结构 | 通过 `icon.inner` 访问 `PlatformIcon` | 已确认 `inner` 字段可访问 |
| Context 初始化 | tauri 需在正确时机调用 `set_ohos_app` | 文档说明 |
| hoverTips 版本 | 6.0.2(22) 起支持 | 低版本静默失败（`.ok()` 忽略错误） |
| notifyOnly 配置 | 必须为 true 才触发事件 | 菜单转换中强制设置 `notify_only: Some(true)` |
| JSON 反序列化 | muda 的 `ohos_context_menu()` 返回 JSON 格式 | 使用 `serde_json::from_str` + `unwrap_or_default` 容错 |
| RefCell 借用 | `set_visible` 中分离 `borrow` 和 `borrow_mut` 避免运行时 panic | 已正确实现 |
| 事件重复注册 | `set_visible(true)` 时重新注册事件监听 | 需确认 OHOS 是否允许重复注册 |

---

## 九、依赖关系

**上游依赖**：
- `muda::platform_impl::ohos` (Phase 0，需先完成)
- `openharmony-ability::statusbar` (Phase 1)

**下游使用**：
- `tauri::tray` 通过 tray-icon crate 使用

---

## 十、完成后通知

本 API 实现完成后，通知 TrayIconEvent 模块验证事件转发是否正常工作。
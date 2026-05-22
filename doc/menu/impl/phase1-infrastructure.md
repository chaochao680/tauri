# Phase 1: 基础架构

## 目标
建立 OHOS Menu 适配的基础设施，包括 ArkTS 类型定义和 openharmony-ability FFI 接口。

## 依赖
- Phase 0: muda OHOS 后端完成

## 架构说明

> **重要**：Phase 0 完成后，tauri::menu 将依赖 muda，muda 依赖 openharmony-ability。
>
> ```
> tauri::menu → muda::platform_impl::ohos → openharmony-ability → ArkUI Menu API
> ```
>
> 本 Phase 的代码主要在 openharmony-ability 中，供 muda 调用。

### 架构设计调整

**设计文档原计划**：使用统一的 `MenuManager` struct 管理所有菜单。

**实际实现**：采用分散的 NAPI 类型架构，更符合 napi-ohos 最佳实践：

```
openharmony-ability/menu/
├── mod.rs          # 模块入口、channel、NAPI emit_menu_event()
├── types.rs        # MenuItemData + Menu/MenuItem/Submenu NAPI 类型
├── event.rs        # MenuEvent + MenuEventDispatcher
├── state.rs        # MenuStateController (替代原 MenuManager)
├── popup.rs        # MenuPopup
└── predefined.rs   # PredefinedMenuItem + PredefinedType
```

**功能等价说明**：
- 原 `MenuManager::create_menu()` → `Menu::new()` + `MenuStateController::create_menu()`
- 原 `MenuManager::popup()` → `popup_context_menu()` 函数
- 原 `MenuManager::destroy()` → `MenuStateController::destroy_menu()`

---

## 工作内容

### 1.1 ArkTS 类型定义

**文件**: `openharmony-ability/native_ability/src/main/ets/helper/menu_types.ets` (60 行)

```typescript
/**
 * Tauri 菜单模块类型定义
 */

export interface TauriMenuItemData {
  id: string;
  type: 'item' | 'separator' | 'submenu' | 'predefined' | 'check' | 'icon';
  text?: string;
  enabled?: boolean;
  accelerator?: string;
  predefinedType?: PredefinedType;
  checked?: boolean;
  icon?: string;
  submenuItems?: TauriMenuItemData[];
}

/**
 * 预定义菜单项类型
 * 注意：OHOS 有两个"恢复"API：
 * - recover: 从 maximize/fullscreen 恢复（API 12+）
 * - restore: 从 minimize 恢复（API 14+，仅 2in1 设备）
 */
export type PredefinedType =
  | 'copy' | 'cut' | 'paste' | 'selectAll'
  | 'undo' | 'redo'
  | 'minimize' | 'maximize' | 'recover' | 'restore' | 'destroyWindow'
  | 'quit' | 'about' | 'hide' | 'hideOthers' | 'showAll';

export interface MenuCreateOptions {
  id?: string;
  items?: TauriMenuItemData[];
}

export interface MenuItemCreateOptions {
  id?: string;
  text: string;
  enabled?: boolean;
  accelerator?: string;
  startIcon?: ResourceStr;
  endIcon?: ResourceStr;
  symbolStartIcon?: SymbolGlyphOptions;
  symbolEndIcon?: SymbolGlyphOptions;
  selected?: boolean;
  selectIcon?: ResourceStr;
}

export interface SubmenuCreateOptions {
  id?: string;
  text: string;
  items?: TauriMenuItemData[];
}

export interface ContextMenuHandle {
  popup: (x?: number, y?: number) => void;
  close: () => void;
}

export interface MenuEventCallback {
  onMenuClick: (menuId: string, itemId: string) => void;
}
```

### 1.2 ArkHelper 扩展

**文件**: `openharmony-ability/native_ability/src/main/ets/ability/type.ets`

```typescript
// ArkHelper 接口扩展
export interface ArkHelper {
  // 现有方法...
  createWebview?: (webTag: string) => web_webview.WebviewController;
  
  // 新增菜单方法
  createMenu?: (options: MenuCreateOptions) => Promise<string>;
  createMenuItem?: (options: MenuItemCreateOptions) => Promise<string>;
  createSubmenu?: (options: SubmenuCreateOptions) => Promise<string>;
  appendMenuItem?: (menuId: string, itemId: string) => Promise<void>;
  popupMenu?: (menuId: string, x?: number, y?: number) => Promise<void>;
  destroyMenu?: (menuId: string) => Promise<void>;
}
```

### 1.3 Rust FFI 接口框架

**文件结构**:
```
openharmony-ability/crates/ability/src/menu/
├── mod.rs          (99 行)   # 模块入口、channel、emit_menu_event()
├── types.rs        (258 行)  # MenuItemData + Menu/MenuItem/Submenu
├── event.rs        (112 行)  # MenuEvent + MenuEventDispatcher
├── state.rs        (75 行)   # MenuStateController
├── popup.rs        (68 行)   # MenuPopup
└── predefined.rs   (288 行)  # PredefinedMenuItem + PredefinedType
```

#### 1.3.1 MenuItemData (types.rs)

```rust
use napi_ohos::bindgen_prelude::*;
use napi_derive_ohos::napi;
use serde::{Deserialize, Serialize};

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuItemData {
    pub id: String,
    #[napi(js_name = "type")]
    #[serde(rename = "type")]
    pub item_type: String,
    pub text: Option<String>,
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accelerator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "predefinedType")]
    pub predefined_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[napi(js_name = "submenuItems")]
    #[serde(rename = "submenuItems")]
    pub submenu_items: Option<Vec<MenuItemData>>,
}
```

#### 1.3.2 Menu NAPI 类型 (types.rs)

```rust
#[napi]
pub struct Menu {
    id: String,
    items: Vec<MenuItemData>,
}

#[napi]
impl Menu {
    #[napi(constructor)]
    pub fn new(id: Option<String>) -> Self { ... }

    #[napi]
    pub fn id(&self) -> String { ... }

    #[napi]
    pub fn append(&mut self, item: MenuItemData) -> Result<()> { ... }

    #[napi]
    pub fn items(&self) -> Vec<MenuItemData> { ... }

    pub fn to_data(&self) -> MenuItemData { ... }
}

#[napi]
pub struct MenuItem {
    id: String,
    text: String,
    enabled: bool,
    accelerator: Option<String>,
}

#[napi]
impl MenuItem {
    #[napi(constructor)]
    pub fn new(id: Option<String>, text: String, enabled: Option<bool>, accelerator: Option<String>) -> Self { ... }

    #[napi]
    pub fn id(&self) -> String { ... }

    #[napi]
    pub fn text(&self) -> String { ... }

    #[napi]
    pub fn set_text(&mut self, text: String) { ... }

    #[napi]
    pub fn enabled(&self) -> bool { ... }

    #[napi]
    pub fn set_enabled(&mut self, enabled: bool) { ... }

    pub fn to_data(&self) -> MenuItemData { ... }
}

#[napi]
pub struct Submenu {
    id: String,
    text: String,
    items: Vec<MenuItemData>,
}

#[napi]
impl Submenu {
    #[napi(constructor)]
    pub fn new(id: Option<String>, text: String) -> Self { ... }

    #[napi]
    pub fn id(&self) -> String { ... }

    #[napi]
    pub fn text(&self) -> String { ... }

    #[napi]
    pub fn set_text(&mut self, text: String) { ... }

    #[napi]
    pub fn append(&mut self, item: MenuItemData) -> Result<()> { ... }

    #[napi]
    pub fn items(&self) -> Vec<MenuItemData> { ... }

    pub fn to_data(&self) -> MenuItemData { ... }
}
```

#### 1.3.3 事件通道 (mod.rs)

```rust
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::sync::LazyLock;

/// Popup request data
#[derive(Debug, Clone)]
pub struct PopupRequest {
    pub json_data: String,
    pub x: Option<f64>,
    pub y: Option<f64>,
}

// Event channel: ArkTS → muda
static MENU_EVENT_CHANNEL: LazyLock<(Sender<String>, Receiver<String>)> =
    LazyLock::new(unbounded);

// Popup channel: muda → ArkTS
static POPUP_CHANNEL: LazyLock<(Sender<PopupRequest>, Receiver<PopupRequest>)> =
    LazyLock::new(unbounded);

/// Rust API: Get menu event receiver (for muda)
pub fn menu_event_receiver() -> &'static Receiver<String> {
    &MENU_EVENT_CHANNEL.1
}

/// Rust API: Get popup request receiver (for ArkTS NAPI)
pub fn popup_request_receiver() -> &'static Receiver<PopupRequest> {
    &POPUP_CHANNEL.1
}

/// NAPI API: Emit menu event from ArkTS
#[napi]
pub fn emit_menu_event(menu_id: String) {
    MENU_EVENT_CHANNEL.0.send(menu_id.clone()).ok();
    dispatch_menu_event(&MenuEvent::new(menu_id));
}

/// Rust API: Popup context menu (for muda)
pub fn popup_context_menu(json_data: String, x: Option<f64>, y: Option<f64>) -> Result<()> {
    POPUP_CHANNEL.0.send(PopupRequest { json_data, x, y }).ok();
    Ok(())
}
```

#### 1.3.4 MenuStateController (state.rs)

替代原设计中的 `MenuManager`，提供菜单状态管理：

```rust
#[napi]
pub struct MenuStateController {
    menus: Arc<RwLock<HashMap<String, Vec<MenuItemData>>>>,
}

#[napi]
impl MenuStateController {
    #[napi(constructor)]
    pub fn new() -> Self { ... }

    #[napi]
    pub fn create_menu(&self, id: String, items: Vec<MenuItemData>) -> Result<()> { ... }

    #[napi]
    pub fn append_item(&self, menu_id: String, item: MenuItemData) -> Result<()> { ... }

    #[napi]
    pub fn get_menu_items(&self, menu_id: String) -> Result<Vec<MenuItemData>> { ... }

    #[napi]
    pub fn destroy_menu(&self, menu_id: String) -> Result<()> { ... }
}
```

#### 1.3.5 MenuPopup (popup.rs)

```rust
#[napi]
pub struct MenuPopup {
    menus: Arc<RwLock<HashMap<String, Vec<MenuItemData>>>>,
}

#[napi]
impl MenuPopup {
    #[napi(constructor)]
    pub fn new() -> Self { ... }

    #[napi]
    pub fn set_menu_items(&self, menu_id: String, items: Vec<MenuItemData>) -> Result<()> { ... }

    #[napi]
    pub fn show(&self, menu_id: String, x: f64, y: f64) -> Result<()> { ... }

    #[napi]
    pub fn hide(&self) -> Result<()> { ... }
}
```

### 1.4 Cargo.toml 配置

**文件**: `openharmony-ability/crates/ability/Cargo.toml`

```toml
[features]
menu = []

[dependencies]
# NAPI bindings
napi-ohos = { workspace = true, features = ["napi8"] }
napi-derive-ohos = { workspace = true }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Utilities
uuid = { version = "1", features = ["v4"] }
crossbeam-channel = "0.5"
tokio = { version = "1", features = ["sync"] }
```

---

## 验证方法

> **验证策略**：本阶段均为内部接口，使用 Rust UT 验证

### 1.5 Rust UT（内部接口）

**验证范围**：

| 接口 | 类型 | 测试方法 | 状态 |
|------|------|---------|------|
| `MenuItemData` struct | 内部 | Rust UT | ✅ 已验证 |
| `Menu::new()` | 内部 | Rust UT | ✅ 已验证 |
| `MenuItem::new()` | 内部 | Rust UT | ✅ 已验证 |
| `Submenu::new()` | 内部 | Rust UT | ✅ 已验证 |
| `MenuStateController::create_menu()` | 内部 | Rust UT | ✅ 已验证 |
| `MenuStateController::destroy_menu()` | 内部 | Rust UT | ✅ 已验证 |
| `MenuPopup::new()` | 内部 | Rust UT | ✅ 已验证 |
| `MenuEvent` 创建和分发 | 内部 | Rust UT | ✅ 已验证 |
| `MENU_EVENT_CHANNEL` | 内部 | Rust UT | ✅ 已验证 |
| `POPUP_CHANNEL` | 内部 | Rust UT | ✅ 已验证 |

**测试文件分布**：
- `types.rs`: 4 个测试 (menu_item_data_creation, submenu_nested_items, menu_creation, menu_item_creation)
- `event.rs`: 3 个测试 (menu_event_creation, menu_event_dispatcher, multiple_listeners)
- `state.rs`: 2 个测试 (menu_state_controller_creation, menu_state_create_and_destroy)
- `popup.rs`: 1 个测试 (menu_popup_creation)
- `mod.rs`: 2 个测试 (menu_event_channel, popup_channel)

### 1.6 编译验证

```bash
# 验证 openharmony-ability 编译通过 (OHOS target)
cd openharmony-ability/crates/ability
cargo build --target aarch64-unknown-linux-ohos --features menu

# 验证 ArkTS 类型定义
cd native_ability
ohpm install
hvigorw assembleHap
```

### 1.7 相关 SKILL 文档

- [ohos-rust-ut](../../.claude/skills/ohos-rust-ut/SKILL.md) - Rust UT 执行指南

---

## 输出物

| 文件 | 行数 | 说明 |
|------|------|------|
| `helper/menu_types.ets` | 60 | ArkTS 类型定义 |
| `helper/index.ets` | 修改 | 导出 menu_types |
| `ability/type.ets` | 修改 | ArkHelper 接口扩展 |
| `menu/mod.rs` | 99 | 模块入口、channel、NAPI emit_menu_event |
| `menu/types.rs` | 258 | MenuItemData + Menu/MenuItem/Submenu |
| `menu/event.rs` | 112 | MenuEvent + MenuEventDispatcher |
| `menu/state.rs` | 75 | MenuStateController |
| `menu/popup.rs` | 68 | MenuPopup |
| `menu/predefined.rs` | 288 | PredefinedMenuItem + PredefinedType |

---

## 参考文档

- [Menu 组件](../reference/menu.md) - Menu/MenuItem/MenuItemGroup API
- [MenuItem 组件](../reference/menu_item.md) - MenuItem 属性和事件
- [bindContextMenu](../reference/menu_control.md) - 菜单绑定和弹出控制

---

## 实现状态

### 已完成 (2026-05-15)

**Phase 1 完成度: 100%**

| 任务 | 状态 | 说明 |
|------|------|------|
| ArkTS 类型定义 | ✅ 完成 | menu_types.ets 包含所有接口 |
| ArkHelper 扩展 | ✅ 完成 | 6 个菜单方法已添加 |
| Rust NAPI 类型 | ✅ 完成 | Menu/MenuItem/Submenu/MenuItemData |
| 事件通道 | ✅ 完成 | MENU_EVENT_CHANNEL + POPUP_CHANNEL |
| MenuStateController | ✅ 完成 | 替代原 MenuManager |
| MenuPopup | ✅ 完成 | popup 状态管理 |
| Cargo.toml 配置 | ✅ 完成 | napi-ohos + serde + uuid |
| 单元测试 | ✅ 完成 | 12 个测试全部通过 |

### 架构差异说明

| 设计文档 | 实际实现 | 影响 |
|----------|----------|------|
| 统一 `MenuManager` struct | 分散的 `Menu`/`MenuItem`/`Submenu` + `MenuStateController` | 无影响 - 功能等价 |
| `MenuManager::popup()` | `popup_context_menu()` 函数 | 无影响 - muda 使用相同 API |
| `on_menu_click()` 回调 | `emit_menu_event()` NAPI + channel | 无影响 - 事件机制等价 |
| `napi` crate | `napi-ohos` crate | OHOS 专用 NAPI 绑定 |

**对 muda 的使用无影响**：muda 只使用 `menu_event_receiver()` 和 `popup_context_menu()` 这两个 Rust API，不涉及 NAPI 类型的具体组织方式。

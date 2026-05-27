# Phase 11: OHOS Menu Bar 能力拉齐设计

> 职责：将 OHOS desktop menubar 的功能拉齐到 Windows/GTK/macOS 水平
> 来源：Phase 10 审计发现 11 项能力缺口
> 前置：Phase 10 menubar 基础渲染 + Bug 修复已完成
> 独立性：✓ 仅影响 OHOS desktop 路径，不影响 Windows/macOS/Linux/mobile

---

## 一、能力缺口总表

### 1.1 核心 API 缺失

| # | 缺口 | 其他平台 | OHOS 现状 | 优先级 |
|---|------|---------|----------|--------|
| A1 | hide_menu / show_menu / is_menu_visible | Win: `SetMenu(NULL)/SetMenu(HMENU)/GetMenu`; GTK: `hide()/show_all()/get_visible()` — per-window | tauri Window 方法 no-op; 当前 `set_menu_json` 无 window_id，多窗时所有窗口共享同一 AppStorage key | **P0** |
| A2 | 菜单项修改后自动刷新 menubar | Win/GTK/mac: OS 自动更新原生 widget — per-window native update | 需手动 `refresh_menubar()`; 当前推 JSON 到全局 AppStorage key，多窗时各窗口菜单互相覆盖 | **P0** |

### 1.2 渲染/视觉能力缺失

| # | 缺口 | 其他平台 | OHOS 现状 | 优先级 |
|---|------|---------|----------|--------|
| B1 | Hover 高亮（bar-level） | Win: `ODS_HOTLIGHT`; GTK/mac: 原生 hover | 只在 activeDropdown 时 backgroundColor 切换，无 hover 效果 | **P1** |
| B2 | Bar-level 图标 | Win/GTK/mac: submenu icon 可显示在菜单条上 | bar-level 只有 Text | **P1** |
| B3 | 暗色模式适配 | Win: `MenuTheme{Dark,Light,Auto}`; GTK/mac: 系统主题 | 硬编码颜色 | **P1** |
| B4 | About 对话框 | Win: `MessageBox`; GTK: `AboutDialog`; mac: `NSAboutPanel` | ✅ 已实现 (`showAlertDialog`) | — 无缺口 |
| B5 | 系统自适应样式 | 系统决定 fontSize/height/padding | 硬编码 fontSize(14)/height(40)/padding(12,10) | **P2** |
| B6 | Disabled bar-level 视觉区分 | 系统灰化样式 | fontColor 不变 | **P2** |

### 1.3 交互能力缺失

| # | 缺口 | 其他平台 | OHOS 现状 | 优先级 |
|---|------|---------|----------|--------|
| C1 | Accelerator 快捷键执行 | Win: `HACCEL+TranslateAccelerator`; GTK: `AccelGroup`; mac: `keyEquivalent` | accelerator 只是数据字符串 | **P1** |
| C2 | Predefined 原生动作执行 | Win: `SendInput/ShowWindow/PostQuitMessage`; GTK: `libxdo`; mac: ObjC selectors | 全部只 fire MenuEvent，部分无原生行为 | **P1** |
| C3 | Fullscreen 沉浸式菜单条隐藏 | Win/GTK: hide_menu; mac: 原生 auto-hide | ArkTS 侧已有实现，但 Rust API 不连接此机制 | **P1 → A1 实现后自动完成** |

### 1.4 已排除项（所有平台都不支持）

| 项目 | 说明 |
|------|------|
| 菜单条拖拽排序 | Win32 HMENU / GTK MenuBar / NSMenu 都不支持原生拖拽 |
| 菜单条右键菜单 | 所有平台菜单条都没有右键菜单 |
| Bar-level accelerator 显示 | 所有平台 bar-level 项都是 submenu text，无自身快捷键 |
| 原生系统图标（除 macOS） | Windows/GTK 也是 no-op |

---

## 二、OHOS 原生能力判定

| # | 缺口 | OHOS 原生能力 | 方案类型 |
|---|------|-------------|---------|
| A1 | hide/show/is_visible | ❌ 无原生 menubar widget，AppStorage + TSFN 可控制渲染 — 需 per-window AppStorage key + window_id 路由 | 替代方案（per-window） |
| A2 | 自动刷新 | ❌ 无原生自动更新，需 TSFN 推 JSON — 需 per-window MenuRequest.window_id | 替代方案（per-window） |
| B1 | Hover | ✅ ArkUI `onHover(isHover)` 回调 | 原生支持 |
| B2 | Bar-level icon | ✅ ArkUI `Image` + `Row` 布局 | 原生支持 |
| B3 | 暗色模式 | ✅ `Configuration.colorMode` + `onConfigurationUpdate` 管道已存在 | 原生支持（管道需连接） |
| B5+B6 | 主题/样式 | ✅ 通过 colorMode 推导颜色 | 替代方案（从 colorMode 推导） |
| C1 | Accelerator | ✅ ArkUI `onKeyPreIme`（API 12+，第一派发获焦链）+ `KeyEvent.keyText/ctrlKey/shiftKey` — 每个 MainPage 有自己的 Column → per-window 天然隔离 | 原生支持（需第一派发模式） |
| C2 | Predefined 动作 | ✅ `window.Window.hide/show/minimize/maximize/recover/destroyWindow`; `pasteboard` API; `showAlertDialog` | 原生支持（部分需补齐） |
| C3 | Fullscreen | ✅ 已有实现（AppStorage `menubar_visible`） | A1 实现后自动完成 |

---

## 三、A1: hide_menu / show_menu / is_menu_visible（Per-window)

### 3.1 设计方案

**核心思路**：所有菜单请求（JSON/visibility/popup）通过 `window_id` 字段路由到对应窗口的 AppStorage key，实现 per-window 独立 menubar。与 Windows/GTK per-window 菜单模型一致。

> **设计原则**：`hide_menu()` / `show_menu()` / `is_menu_visible()` 在 tauri 中是 `Window` 方法（per-window）。OHOS 实现必须保持 per-window 语义，不能退化为全局行为。当前 OHOS 单窗口场景下 per-window = 全局只有一个 key，但数据管道从一开始就是多窗兼容的。

#### openharmony-ability 侧

**文件**：`crates/ability/src/menu/mod.rs`

**`MenuRequest` / `MenuRequestData` 扩展** — 增加 `window_id` 字段：

```rust
pub struct MenuRequest {
    pub json_data: String,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub visible: Option<bool>,
    pub window_id: String,           // 新增：来自 Window::label()，永不为空
}

#[napi(object)]
pub struct MenuRequestData {
    pub json_data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_id: Option<String>,    // 新增：NAPI 侧 Optional，None→不序列化
}
```

**`window_id` 设计说明**：
- Rust 侧 `MenuRequest.window_id` 是 `String`（必填），每个请求都携带来源窗口 label
- NAPI 侧 `MenuRequestData.window_id` 是 `Option<String>`（可选序列化），与 `x/y/visible` 保持 `skip_serializing_if` 一致
- `window_id` 来自 `Window::label()`（tauri 窗口唯一标识，如 `"main"`、`"secondary"`）

**请求类型区分**（扩展后）：

| 请求类型 | json_data | x | y | window_id | visible |
|----------|-----------|---|---|-----------|---------|
| menubar JSON | 有内容 | None | None | Some(label) | None |
| popup | 有内容 | Some | Some | Some(label) | None |
| visibility | `""` | None | None | Some(label) | Some(bool) |

**per-window visibility 状态** — `HashMap` 替代全局 AtomicBool：

```rust
use std::collections::HashMap;
use std::sync::RwLock;

static MENUBAR_VISIBLE: RwLock<HashMap<String, bool>> = RwLock::new(HashMap::new());

#[napi]
pub fn notify_menubar_visibility(window_id: String, visible: bool) {
    let mut map = MENUBAR_VISIBLE.write().unwrap();
    map.insert(window_id, visible);
}

pub fn is_menubar_visible(window_id: &str) -> bool {
    let map = MENUBAR_VISIBLE.read().unwrap();
    map.get(window_id).copied().unwrap_or(true)  // 默认 true
}

pub fn set_menubar_visible(window_id: String, visible: bool) -> Result<()> {
    {
        let mut map = MENUBAR_VISIBLE.write().unwrap();
        map.insert(window_id.clone(), visible);
    }
    MENU_CHANNEL.0.send(MenuRequest {
        json_data: "".to_string(),
        x: None,
        y: None,
        visible: Some(visible),
        window_id,
    }).ok();
    Ok(())
}
```

**`set_menu_json` / `popup_context_menu` 增加 window_id 参数**：

```rust
pub fn set_menu_json(json_data: String, window_id: String) -> Result<()> {
    MENU_CHANNEL.0.send(MenuRequest { json_data, x: None, y: None, visible: None, window_id }).ok();
    Ok(())
}

pub fn popup_context_menu(json_data: String, x: Option<f64>, y: Option<f64>, window_id: String) -> Result<()> {
    MENU_CHANNEL.0.send(MenuRequest { json_data, x, y, visible: None, window_id }).ok();
    Ok(())
}
```

**forwarder 更新** — `MenuRequestData` 构造增加 `window_id`：

```rust
// start_menu_forwarder 内部：
let data = MenuRequestData {
    json_data: req.json_data,
    x: req.x,
    y: req.y,
    visible: req.visible,
    window_id: Some(req.window_id),    // 新增
};
```

**`lib.rs` 导出更新**：

```rust
#[cfg(feature = "menu")]
pub use menu::{
    on_menu_request, on_popup_request,
    start_menu_forwarder, start_popup_forwarder,
    MenuRequestData, PopupRequestData,
    popup_context_menu, set_menu_json,
    menu_event_receiver, popup_request_receiver, menu_request_receiver,
    set_menubar_visible, is_menubar_visible, notify_menubar_visibility,  // 新增
};
```

#### muda 侧

**不新增 `visible` 管理**。muda 是数据层，visibility 是 UI 层。与 Phase 10 `refresh_menubar` 不加 `#[cfg(desktop)]` 的设计一致。

**`refresh_menubar` / `popup` 增加 `window_id` 透传**：

```rust
// muda/src/platform_impl/ohos/mod.rs
pub fn refresh_menubar(&self, window_id: &str) -> crate::Result<()> {
    init_menu_event_listener();
    let json = self.to_json();
    openharmony_ability::menu::set_menu_json(json, window_id.to_string())
        .map_err(|e| crate::Error::CustomError(e.to_string()))?;
    Ok(())
}

pub fn popup(&self, x: f64, y: f64, window_id: &str) -> crate::Result<()> {
    init_menu_event_listener();
    let json = self.to_json();
    openharmony_ability::menu::popup_context_menu(json, Some(x), Some(y), window_id.to_string())
        .map_err(|e| crate::Error::CustomError(e.to_string()))?;
    Ok(())
}
```

#### tauri 侧

**文件**：`crates/tauri/src/window/mod.rs`

所有 OHOS 分支通过 `self.label()` 获取 window_id，传入 openharmony-ability API：

```rust
pub fn hide_menu(&self) -> crate::Result<()> {
    #[cfg(target_env = "ohos")]
    if let Some(_) = &*self.menu_lock() {
        openharmony_ability::menu::set_menubar_visible(false, self.label().to_string()).ok();
    }

    #[cfg(not(target_env = "ohos"))]
    // ... 现有 run_on_main_thread 闭包 ...
    Ok(())
}

pub fn show_menu(&self) -> crate::Result<()> {
    #[cfg(target_env = "ohos")]
    if let Some(window_menu) = &*self.menu_lock() {
        window_menu.menu.inner().refresh_menubar(self.label()).ok();
        openharmony_ability::menu::set_menubar_visible(true, self.label().to_string()).ok();
    }

    #[cfg(not(target_env = "ohos"))]
    // ... 现有代码 ...
    Ok(())
}

pub fn is_menu_visible(&self) -> crate::Result<bool> {
    #[cfg(target_env = "ohos")]
    return Ok(openharmony_ability::menu::is_menubar_visible(self.label()));

    #[cfg(not(target_env = "ohos"))]
    // ... 现有 run_on_main_thread + mpsc 代码 ...
    Ok(false)
}
```

**`set_menu` / `remove_menu` 也传入 window_id**：

```rust
// Window::set_menu OHOS block:
#[cfg(target_env = "ohos")]
{
    menu.inner().refresh_menubar(self.label()).ok();
}

// Window::remove_menu OHOS block:
#[cfg(target_env = "ohos")]
if let Some(_menu) = &prev_menu {
    openharmony_ability::menu::set_menu_json("[]".to_string(), self.label().to_string()).ok();
}
```

#### ArkTS 侧

**NativeAbility.ets** — callback 按 window_id 路由到 per-window AppStorage key：

```typescript
primaryModule.onMenuRequest((data: { jsonData: string; x?: number; y?: number; visible?: boolean; windowId?: string }) => {
    const windowId = data.windowId ?? "main";  // 默认主窗口

    // 1. Visibility 请求
    if (data.visible !== undefined && data.visible !== null) {
        AppStorage.setOrCreate("__openharmony_ability_menubar_visible__::" + windowId, data.visible);
        primaryModule.notifyMenubarVisibility(windowId, data.visible);
        return;
    }

    // 2. Popup 请求 — 传 windowId 给 popupFromJson
if (data.x !== undefined && data.y !== undefined) {
        menuManager.popupFromJson(data.jsonData, data.x ?? 0, data.y ?? 0, windowId);
        return;
    }

    // 3. Menubar JSON 请求
    AppStorage.setOrCreate("__openharmony_ability_menubar_json__::" + windowId, data.jsonData);
});
```

**AppStorage key 命名规则**：`__openharmony_ability_menubar_json__::${windowId}`

| key | 示例 | 说明 |
|-----|------|------|
| menubar JSON | `__openharmony_ability_menubar_json__::main` | 主窗口菜单数据 |
| menubar visible | `__openharmony_ability_menubar_visible__::main` | 主窗口菜单可见性 |
| menubar JSON | `__openharmony_ability_menubar_json__::secondary` | 第二窗口菜单数据 |
| popup shown | `__openharmony_ability_menu_shown__::main` | 主窗口 popup 显示状态 |
| popup JSON | `__openharmony_ability_menu_json__::main` | 主窗口 popup 菜单数据 |
| popup x | `__openharmony_ability_menu_x__::main` | 主窗口 popup 横坐标 |
| popup y | `__openharmony_ability_menu_y__::main` | 主窗口 popup 纵坐标 |

> **所有菜单 AppStorage key 统一加 `::${windowId}` 后缀** — menubar 2 个 + popup 4 个，共 6 个 key。子窗口共享同一 AppStorage，不带后缀会导致两窗口互相覆盖。设备级 key（如 `is_desktop__`、`color_mode__`）不加后缀。

> **AppStorage 动态 key**：`AppStorage.setOrCreate()` 支持运行时动态字符串 key，ArkTS callback 用 `windowId` 拼接 key 是合法的。`@StorageProp` 的 key 必须是编译时常量 — 主窗口 `::main` 是硬编码常量，可工作。子窗口的 `@StorageProp` 绑定方式见 §18。

**MainPage.ets** — 主窗口绑定改为 `::main` 后缀（menubar + popup）：

```typescript
// Menubar
@Watch("onMenubarJsonChange") @StorageProp("__openharmony_ability_menubar_json__::main") menubarJson: string = "[]";
@StorageProp("__openharmony_ability_menubar_visible__::main") menubarVisible: boolean = true;

// Popup — Phase 10 用 @StorageLink，改为 @StorageProp::main 后缀（单向同步足够）
@Watch("onMenuJsonChange") @StorageProp("__openharmony_ability_menu_json__::main") menuJson: string = "[]";
@StorageProp("__openharmony_ability_menu_shown__::main") menuShown: boolean = false;
@StorageProp("__openharmony_ability_menu_x__::main") menuX: number = 0;
@StorageProp("__openharmony_ability_menu_y__::main") menuY: number = 0;
```

**menu.ets PredefinedActionExecutor** — fullscreen/recover 同步 Rust 状态（当前主窗口）：

```typescript
case 'fullscreen':
    primaryModule.notifyMenubarVisibility("main", false);
    AppStorage.setOrCreate("__openharmony_ability_menubar_visible__::main", false);
    // ... maximize immersive ...
    break;
case 'recover':
    primaryModule.notifyMenubarVisibility("main", true);
    AppStorage.setOrCreate("__openharmony_ability_menubar_visible__::main", true);
    // ... win.recover() ...
    break;
```

> **fullscreen/recover 多窗扩展**：当前 PredefinedActionExecutor 只服务于主窗口。多窗场景下每个窗口有自己的 PredefinedActionExecutor 和 window 实例，fullscreen/recover 自然操作当前窗口。NAPI `notifyMenubarVisibility` 的 `window_id` 参数此时传的是 executor 所属的窗口 label。

### 3.2 关键设计决策

| 决策 | 原因 |
|------|------|
| per-window HashMap 替代全局 AtomicBool | `hide_menu()` / `show_menu()` / `is_menu_visible()` 是 Window 方法（per-window 语义）。全局 AtomicBool 在多窗口场景下 hide(window_A) 会导致 window_B 菜单也消失 |
| `MenuRequest.window_id` 是 `String`（必填） | 每个请求都来自特定窗口。window_id 不应为 Optional — 退化到全局 key 会破坏 per-window 路由 |
| `MenuRequestData.window_id` 是 `Option<String>`（NAPI） | NAPI 序列化风格一致性：与 `x/y/visible` 一样用 `skip_serializing_if`。但实际 Rust 侧构造时 **永远传入 Some** |
| `window_id` 来自 `Window::label()` | tauri 窗口唯一标识。主窗口默认 `"main"`，与 tauri 跨平台约定一致 |
| AppStorage key 格式 `::${windowId}` | 双冒号分隔符，不会与现有单窗 key 名冲突。便于动态拼接（callback 端）和硬编码（主窗口 @StorageProp） |
| show_menu 先推 JSON 再设 visible | FIFO channel 保证顺序：JSON 更新先到达 → ArkTS 更新 menubarItems → 然后设 menubarVisible=true → 渲染。避免重新显示时闪烁旧数据 |
| hide_menu 只设 visible=false 不清 `"[]"` | hide = "临时隐藏"，remove = "永久移除"。与 `remove_menu()` 语义区分 |
| visibility 请求的 json_data 为空字符串 `""` | `json_data: String` 是必填字段，visibility-only 请求无有意义的 JSON。ArkTS callback 先检查 visible 分支并 return early，不处理空 jsonData |
| ArkTS callback 检查 `visible !== undefined && visible !== null` | NAPI 序列化 Option<bool)：Some(true)→true, Some(false)→false, None→不序列化(undefined)。ArkTS 需检查两者 |
| `is_menubar_visible` 默认返回 true | HashMap 中未记录的窗口（尚未 set_menu 的窗口）视为 visible — 与 Win/GTK 行为一致 |
| muda `refresh_menubar` / `popup` 接受 `window_id` 参数 | muda 不存储窗口信息，需要调用者传入。与 Phase 10 设计一致（muda 是数据层，窗口信息由 tauri 层提供） |

### 3.3 边缘场景

| 场景 | 行为 |
|------|------|
| hide_menu(window_A) 后 is_menu_visible(window_A) | 返回 false（HashMap 已更新） |
| hide_menu(window_A) 后 is_menu_visible(window_B) | 返回 true（独立状态，不受影响） |
| fullscreen ArkTS 设 visible=false 后 Rust is_menu_visible("main") | 返回 false（ArkTS 调了 notifyMenubarVisibility("main", false)） |
| hide_menu 后修改菜单项（set_text 等） | 内存修改 + auto_refresh 推 JSON(window_id) → ArkTS 路由到正确 AppStorage key → @Watch 触发 → menubarItems 更新（但 menubarVisible=false 不渲染） |
| hide_menu 后 show_menu | show_menu 推最新 JSON(window_id) + 设 visible=true(window_id) → 对应窗口 menubar 渲染最新状态 |
| hide_menu 后 remove_menu | remove_menu 推 `"[]"`(window_id) → 对应窗口 menubarItems=[] → show_menu 后 menubarItems.length===0 不渲染 |
| hide_menu 后 set_menu(new_menu) | set_menu 推新 JSON(window_id) → menubarItems 更新 → show_menu 再推一次（AppStorage dedup 不触发 @Watch） |
| 两个窗口同时 set_menu | Window_A 推 JSON("main") → AppStorage("menubar_json::main")；Window_B 推 JSON("secondary") → AppStorage("menubar_json::secondary")。互不干扰 |

---

## 四、A2: 菜单项修改后自动刷新 menubar

### 4.1 设计方案

**核心思路**：tauri 层辅助函数 `auto_refresh_menubar()`，在每个 OHOS setter 分支末尾调用。只刷新**当前菜单所属的窗口**，不是所有窗口。

#### 辅助函数

**文件**：`tauri/crates/tauri/src/menu/mod.rs`

```rust
#[cfg(all(target_env = "ohos", desktop))]
fn auto_refresh_menubar<R: Runtime>(menu_item: &MenuItemType<R>) {
    // 刷新 app-level menu（macOS 模式，OHOS 也支持）
    if let Some(menu) = menu_item.app_handle.menu() {
        menu.refresh_menubar("main").ok();  // app-level menu 绑定主窗口
    }
    // 刷新当前菜单项所属的窗口菜单（per-window，Windows/GTK 模式）
    // 每个 menu item 有 window 引用，通过 menu_lock 找到所属窗口
    for window in menu_item.app_handle.manager().windows().values() {
        if let Some(wm) = window.menu_lock().as_ref() {
            wm.menu.refresh_menubar(window.label()).ok();
        }
    }
}
```

> **设计说明**：`refresh_menubar` 现在接受 `window_id` 参数，每个窗口推 JSON 到自己的 AppStorage key。auto_refresh 对每个窗口独立调用 `refresh_menubar(window_label())`，per-window 天然隔离。

#### 每个 setter 追加调用

在 6 个文件共 31 个方法的 OHOS 分支末尾追加 `auto_refresh_menubar(&self.0.app_handle)`：

| 文件 | 方法 | 改动模式 |
|------|------|---------|
| `menu/menu.rs` | append, prepend, insert, remove, remove_at | 末尾追加 |
| `menu/menu.rs` | append_items, prepend_items, insert_items | 改为循环修改后一次性刷新（见 §4.2） |
| `menu/submenu.rs` | append, prepend, insert, remove, remove_at, set_text, set_enabled, set_icon | 末尾追加 |
| `menu/submenu.rs` | append_items, prepend_items, insert_items | 改为循环修改后一次性刷新 |
| `menu/normal.rs` | set_text, set_enabled, set_accelerator, set_key_accelerator | 末尾追加 |
| `menu/check.rs` | set_text, set_enabled, set_accelerator, set_key_accelerator, set_checked | 末尾追加 |
| `menu/icon.rs` | set_text, set_enabled, set_accelerator, set_key_accelerator, set_icon | 末尾追加 |
| `menu/predefined.rs` | set_text | 末尾追加 |

#### setter 改动示例

```rust
// Submenu::set_text OHOS 分支（修改后）
#[cfg(target_env = "ohos")]
{
    (*self.0).as_ref().set_text(text);
    auto_refresh_menubar(&self.0.app_handle);
    Ok(())
}
```

### 4.2 批量操作优化

`append_items` / `insert_items` / `prepend_items` 的 OHOS 分支改为先循环修改内存，最后一次性刷新：

```rust
// Menu::append_items OHOS 分支（修改后）
#[cfg(target_env = "ohos")]
{
    for item in items {
        (*self.0).as_ref().append(item.inner().inner_muda())?;
    }
    auto_refresh_menubar(&self.0.app_handle);
    Ok(())
}
```

原实现逐项调用 `self.append(item)?`，每次 append 的 OHOS 分支将追加 `auto_refresh_menubar`。改为直接循环 `(*self.0).as_ref().append(...)` + 末尾一次性刷新，避免 N 次冗余 TSFN 推送。

### 4.3 性能考量

| 场景 | TSFN 推送次数 | 说明 |
|------|-------------|------|
| 单次 set_text | 1 | 可接受 |
| 循环 10 次 set_text | 10 | 建议开发者用 append_items 等批量 API |
| 批量 append_items(5项) | 1 | 优化后只推一次 |
| Window::set_menu + auto_refresh | 2（冗余） | set_menu 已调 refresh_menubar，setter 再调一次。AppStorage dedup 不触发 @Watch，安全但浪费 |

### 4.4 关键设计决策

| 决策 | 原因 |
|------|------|
| 在 tauri 层而非 muda 层 | MenuChild 无 root Menu 反向引用，muda 层无法调用 refresh_menubar。tauri 每个菜单项有 app_handle |
| 用 `manager().windows()` 而非 `available_windows()` | `available_windows()` 不存在，`Manager::windows()` 需要 unstable feature，`manager().windows()` 是 crate-internal 可用 |
| AppStorage dedup 防止冗余渲染 | `setOrCreate` 相同值不触发 @Watch，冗余推送安全 |

---

## 五、B1: Hover 高亮

### 5.1 设计方案

**MainPage.ets MenuBarRow** — 每个 bar-level Text 新增 `onHover(isHover)` + `hoveredItemId` 状态：

```typescript
@State private hoveredItemId: string = "";

// MenuBarRow 内 ForEach 的 Text：
.onHover((isHover: boolean) => {
    this.hoveredItemId = isHover ? item.id : "";
})
.backgroundColor(
    this.activeDropdownId === item.id ? this.getActiveBg() :
    this.hoveredItemId === item.id ? this.getHoverBg() :
    this.getMenubarBg()
)
```

三态颜色（与 B3 暗色模式联动）：

| 状态 | Light | Dark |
|------|-------|------|
| Normal | `#F5F5F5` | `#2D2D2D` |
| Hover | `#EBEBEB` | `#3B3B3B` |
| Active (dropdown open) | `#E0E0E0` | `#404040` |

---

## 六、B2: Bar-level 图标

### 6.1 设计方案

**MenuBarRow** — 每个 bar-level 项从纯 Text 改为 `Row { Image + Text }` 条件渲染：

```typescript
ForEach(this.menubarItems, (item: MenuItemData) => {
    Row() {
        if (item.icon && this.iconPixelMaps.has(item.id)) {
            Image(this.iconPixelMaps.get(item.id))
                .width(16).height(16)
                .margin({ right: 4 })
        }
        Text(item.text ?? '')
    }
    .fontSize(14)
    .fontColor(this.getTextColor())
    .padding({ left: 12, right: 12, top: 10, bottom: 10 })
    .enabled(item.enabled ?? true)
    .opacity((item.enabled ?? true) ? 1.0 : 0.5)    // B6 disabled 视觉区分
    .backgroundColor(...)
    .onClick(...)
    .bindMenu(...)
    .onHover(...)
}, (item: MenuItemData) => item.id)
```

**关键点**：
- bar-level icon 用 16x16（与 GTK 一致，dropdown 用 24x24）
- `menubarIconIds` 已在 `onMenubarJsonChange` 中收集，`prepareIcons` 已处理 PixelMap
- 无 icon 的项只显示 Text，不影响现有布局
- disabled 项的 fontColor 变灰 + opacity 0.5（B6 合并实现）

---

## 七、B3 + B5 + B6: 暗色模式 + 样式适配

### 7.1 现状分析

OHOS 已有完整 dark mode 检测管道：
- ArkTS: `onConfigurationUpdate(newConfig)` → Rust `ColorMode { Dark=0, Light=1, NoSet=-1 }`
- Rust: `Event::ConfigChanged(Configuration { color_mode })` 已到达 tao event loop
- **缺口**：tao `ConfigChanged` handler 只发 `ScaleFactorChanged`，不发 `ThemeChanged`；`theme()` 硬编码 `Light`

### 7.2 tao 修复（前置步骤）

**文件**：`tao/src/platform_impl/ohos/mod.rs`

**ConfigChanged handler** — 同时发 `ScaleFactorChanged` 和 `ThemeChanged`：

```rust
// MainEvent::ConfigChanged handler 中新增：
let theme = match configuration.color_mode {
    ColorMode::Dark => Theme::Dark,
    _ => Theme::Light,  // Light 或 NoSet 默认 Light
};
// 在现有 ScaleFactorChanged 发送之后追加：
window_event_handler(WindowEvent::ThemeChanged(theme));
```

**theme() 方法** — 读 config.color_mode：

```rust
pub fn theme(&self) -> Theme {
    match self.app.config().color_mode {
        ColorMode::Dark => Theme::Dark,
        _ => Theme::Light,
    }
}
```

**新增 import**：`use openharmony_ability::ColorMode;`

### 7.3 ArkTS 侧

**NativeAbility.ets** — 初始 colorMode 设置：

```typescript
// setupMenuPopup 中追加：
AppStorage.setOrCreate("__openharmony_ability_color_mode__", this.context.config.colorMode ?? 1);
```

**NativeAbility.ets** — onConfigurationUpdate 追加 colorMode：

```typescript
onConfigurationUpdate(newConfig: Configuration): void {
    AppStorage.setOrCreate("__openharmony_ability_color_mode__", newConfig.colorMode ?? 1);
    this.forEachLifecycle((lifecycle) =>
        lifecycle.environmentCallback.onConfigurationUpdated(newConfig),
    );
}
```

### 7.4 MainPage.ets 动态颜色

新增 `@StorageProp` + 颜色辅助函数：

```typescript
@StorageProp("__openharmony_ability_color_mode__") colorMode: number = 1;

private isDark(): boolean {
    return this.colorMode === 0;
}

private getTextColor(): string {
    return this.isDark() ? '#E0E0E0' : '#333333';
}

private getMenubarBg(): string {
    return this.isDark() ? '#2D2D2D' : '#F5F5F5';
}

private getHoverBg(): string {
    return this.isDark() ? '#3B3B3B' : '#EBEBEB';
}

private getActiveBg(): string {
    return this.isDark() ? '#404040' : '#E0E0E0';
}

private getDisabledTextColor(): string {
    return this.isDark() ? '#666666' : '#999999';
}
```

MenuBarRow 中所有硬编码颜色替换为动态函数调用。

### 7.5 B6 Disabled 视觉区分

合并到 B2 bar-level icon 改动中：

```typescript
.fontColor((item.enabled ?? true) ? this.getTextColor() : this.getDisabledTextColor())
.opacity((item.enabled ?? true) ? 1.0 : 0.5)
```

### 7.6 B5 系统自适应样式

fontSize(14) / height(40) / padding(12,10) 保持固定值 — OHOS 桌面没有系统级 menubar 样式 API，只能通过 colorMode 推导颜色。尺寸参数保持固定是合理的。

---

## 八、B4: About 对话框

**无需改动**。已有完整实现：
- `PredefinedActionExecutor.execute('about')` → `showAboutFn(aboutMetadata)`
- `NativeAbility.ets` → `uiContext.showAlertDialog({ title, message })`
- `AboutMetadataData` Rust → NAPI → ArkTS

只需验证 menubar 下拉中的 About 触发路径与 popup 中一致。

---

## 九、C1: Accelerator 快捷键执行

### 9.1 设计方案

**核心思路**：ArkTS 层拦截键盘事件，匹配 accelerator，命中则 fire MenuEvent。每个 MainPage 实例有自己的 AcceleratorMatcher — per-window 天然隔离。

#### 拦截机制

使用 **`onKeyPreIme`**（第一派发，获焦链传播）而非 `onKeyEvent`（第三派发，bubble 模式）。

- `onKeyEvent`：事件从焦点子组件向上冒泡。WebView 先处理 Ctrl+C，如果消费了事件，父组件的 `onKeyEvent` 永远收不到
- `onKeyPreIme`：第一派发沿获焦链叶→根传播。子组件(WebView)没有 onKeyPreIme(隐式 false) → 事件传播到父 Column 的 onKeyPreIme → 匹配则消费(return true)，不匹配则继续第二/第三派发

**MainPage.ets** — 在包含 MenuBarRow + DefaultXComponent 的 Column 上注册：

```typescript
.onKeyPreIme((event: KeyEvent) => {
    if (event.type === KeyType.DOWN && this.acceleratorMatcher.matches(event)) {
        this.acceleratorMatcher.fireMatchedItem();
        return true;   // 消费事件，阻止 WebView 接收
    }
    return false;      // 传递给 WebView
})
```

> **Per-window 天然隔离**：每个窗口有自己的 MainPage 实例 → 自己的 Column → 自己的 `onKeyPreIme` + `AcceleratorMatcher`。多窗场景下各窗口的快捷键互不干扰。

#### AcceleratorMatcher

新增类，在 `onMenubarJsonChange` 中构建 accelerator 映射表：

```typescript
class AcceleratorMatcher {
    private accelerators: Map<string, string> = new Map();  // keyCombo → menuId

    buildFromItems(items: MenuItemData[]): void {
        this.accelerators.clear();
        const queue: MenuItemData[] = [...items];
        while (queue.length > 0) {
            const item = queue.shift()!;
            if (item.accelerator) {
                const keyCombo = this.normalizeAccelerator(item.accelerator);
                this.accelerators.set(keyCombo, item.id);
            }
            if (item.submenuItems) {
                queue.push(...item.submenuItems);
            }
        }
    }

    private normalizeAccelerator(accel: string): string {
        // "Ctrl+O" → "ctrl+O"
        // "Ctrl+Shift+S" → "ctrl+shift+S"
        return accel.toLowerCase();
    }

    matches(event: KeyEvent): boolean {
        const keyText = this.getKeyText(event);
        if (!keyText) return false;
        let combo = '';
        if (event.ctrlKey) combo += 'ctrl+';
        if (event.shiftKey) combo += 'shift+';
        if (event.altKey) combo += 'alt+';
        combo += keyText.toLowerCase();
        return this.accelerators.has(combo);
    }

    fireMatchedItem(): void {
        // 查找匹配的 menuId，调用 globalMenuClickHandler
    }

    private getKeyText(event: KeyEvent): string | null {
        // ArkUI KeyEvent.keyText 提供键名（"A", "0", "Enter", "Esc" 等）
        // 需将 keyText 映射为 Rust 侧 Display impl 的格式
        // 特殊映射："Escape"→"Esc", "Delete"→"Del", "Insert"→"Ins",
        //            "PageUp"→"PgUp", "PageDown"→"PgDn"
        // 字母/数字直接用 keyText
        return event.keyText;
    }
}
```

#### accelerator 解析

Rust OHOS `KeyAccelerator::fmt` 输出格式：
- Modifiers: `Ctrl+`, `Shift+`, `Alt+`, `Super+`
- Key: 大写字母 (`O`), 特殊名 (`Esc`, `Del`, `Ins`, `PgUp`, `PgDn`, `Tab`, `Space`, `Left/Right/Up/Down`, `Enter`)
- OHOS 上 `CmdOrCtrl` 解为 `Ctrl`（`CMD_OR_CTRL = Modifiers::CONTROL`）

ArkTS normalize 时转小写比对：`"Ctrl+O"` → `"ctrl+o"`，KeyEvent 组合拼接为 `"ctrl+o"`，直接匹配。

#### 特殊键映射

| Rust Display | ArkUI keyText | 说明 |
|-------------|-------------|------|
| `Esc` | `Escape` | 需映射 |
| `Del` | `Delete` | 需映射 |
| `Ins` | `Insert` | 需映射 |
| `PgUp` | `PageUp` | 需映射 |
| `PgDn` | `PageDown` | 需映射 |
| `Space` | `Space` | 直接匹配 |
| `Enter` | `Enter` | 直接匹配（Rust Debug fallback 输出 "Enter"） |
| A-Z | `A`-`Z` | 直接匹配 |
| F1-F12 | `F1`-`F12` | 直接匹配 |

### 9.2 Ctrl+C/V/X 与 WebView 冲突

拦截 Ctrl+C 后：
- **Predefined "Copy" 项**：MenuEvent → PredefinedActionExecutor → `document.execCommand("copy")` → 与 WebView 原生 copy 效果相同
- **自定义 Ctrl+C 项**：MenuEvent → 开发者 handler → 开发者有意覆盖原生行为，正确

**结论**：拦截 Ctrl+C/V/X/A/Z 并消费事件是安全的 — predefined handler 复制了原生行为。

### 9.3 PredefinedActionExecutor.controller null Bug 修复

**审计发现**：`NativeAbility.ets` 创建 `PredefinedActionExecutor` 后只调 `setWindow()`，未调 `setController()`。所有 `document.execCommand` 调用静默失败。

**修复方案**：需要在 NativeAbility 中获取 WebView 的 `WebviewController` 并传给 executor。WebviewController 在 `DefaultXComponent.ets` 中创建。桥接方式：

1. DefaultXComponent 暴露 `getWebviewController()` 方法
2. NativeAbility 通过 `this.primaryModule` 或直接引用获取 controller
3. 调用 `executor.setController(controller)`

**具体桥接设计**：在 NativeAbility 的 `setupMenuPopup` 或后续回调中，当 DefaultXComponent 的 WebviewController 就绪后，通过 AppStorage 或 global callback 传递给 PredefinedActionExecutor。

### 9.4 关键设计决策

| 决策 | 原因 |
|------|------|
| ArkTS 层拦截而非 Rust 层 | accelerator 数据在 ArkTS（menubar JSON），映射表自然在 ArkTS 构建 |
| `onKeyPreIme` 而非 `onKeyEvent` | 第一派发获焦链传播，子先触发(隐式 false)后父可拦截；第三派发(bubble)WebView 先消费 |
| normalizeAccelerator 转小写匹配 | Rust 输出 `"Ctrl+O"`（大写 O），KeyEvent.keyText 也是 `"O"`（大写），统一小写比对 |
| CmdOrCtrl 不需 ArkTS 处理 | Rust OHOS 已解为 `"Ctrl"` |

---

## 十、C2: Predefined 原生动作执行

### 10.1 已有实现（无需改动）

| 动作 | 当前实现 | 状态 |
|------|---------|------|
| minimize | `win.minimize()` | ✅ |
| maximize | `win.maximize()` | ✅ |
| fullscreen | `maximize(ENTER_IMMERSIVE)` | ✅ |
| recover | `win.recover()` | ✅ |
| close / quit | `context.terminateSelf()` | ✅ |
| about | `showAlertDialog()` | ✅ |
| copy/cut/paste/selectAll/undo/redo | WebView `document.execCommand()` | ⚠️ 需 controller fix |

### 10.2 需要补齐

| 动作 | 当前 | 修正 |
|------|------|------|
| hide | `win.minimize()` | 改为 `win.hide()`（OHOS 有 `window.Window.hide()` API） |

**文件**：`menu.ets` PredefinedActionExecutor `'hide'` case：

```typescript
case 'hide':
    await this.win?.hide();  // 替代 win.minimize()
    break;
```

### 10.3 PredefinedActionExecutor.controller 修复

见 §9.3。修复后 copy/cut/paste/selectAll/undo/redo 的 WebView `document.execCommand` 才能真正执行。

---

## 十一、C3: Fullscreen 沉浸式菜单条隐藏

**A1 实现后自动完成**。已有实现：
- ArkTS: `PredefinedActionExecutor.execute('fullscreen')` → AppStorage + maximize
- 新增 Rust API 路径：`Window::hide_menu()` → `set_menubar_visible(false, window_label)` → ArkTS callback → AppStorage

**唯一需要追加**：fullscreen/recover handler 调 `primaryModule.notifyMenubarVisibility("main", visible)` 同步 Rust per-window HashMap（已在 §3.1 中覆盖）。

---

## 十二、改动清单

### 12.1 Rust 侧

| 文件 | 改动 | 行数估算 |
|------|------|---------|
| `openharmony-ability/menu/mod.rs` | MenuRequest 新增 window_id(String); MenuRequestData 新增 window_id(Option<String>+skip_serializing_if); MENUBAR_VISIBLE RwLock<HashMap>; set_menubar_visible/is_menubar_visible/notify_menubar_visibility 接 window_id; set_menu_json/popup_context_menu 接 window_id; forwarder 更新; UT | ~100 |
| `openharmony-ability/menu/mod.rs` UT | test_menu_channel_visibility; test_menu_request_data_visible_serde; test_menu_request_data_window_id_serde; test_menubar_visible_per_window | ~30 |
| `openharmony-ability/menu/event.rs` | MenuEvent 新增 window_id: Option<String>; emit_menu_event 新增 window_id 参数; GLOBAL_DISPATCHER dispatch 按 window_id 过滤 | ~15 |
| `openharmony-ability/lib.rs` | 新增 set_menubar_visible/is_menubar_visible/notify_menubar_visibility 导出; emit_menu_event 签名变更; set_menu_json/popup_context_menu 签名变更 | ~5 |
| `tao/src/platform_impl/ohos/mod.rs` | ConfigChanged handler emit ThemeChanged; theme() 读 colorMode; 新增 ColorMode import | ~15 |
| `muda/src/platform_impl/ohos/mod.rs` | refresh_menubar/popup 新增 window_id 参数; set_menu_json/popup_context_menu 调用传入 window_id | ~8 |
| `muda/src/menu.rs` | Menu::refresh_menubar 新增 window_id 参数 #[cfg(target_env = "ohos")] | ~5 |
| `tauri/crates/tauri/src/window/mod.rs` | hide_menu/show_menu/is_menu_visible OHOS blocks 传 self.label(); set_menu/remove_menu 传 self.label() | ~25 |
| `tauri/crates/tauri/src/menu/mod.rs` | auto_refresh_menubar 辅助函数（per-window） | ~12 |
| `tauri/crates/tauri/src/menu/menu.rs` | 6 个方法追加 auto_refresh; refresh_menubar 接 window_id 参数 | ~45 |
| `tauri/crates/tauri/src/menu/submenu.rs` | 11 个方法追加 auto_refresh | ~22 |
| `tauri/crates/tauri/src/menu/normal.rs` | 4 个方法追加 auto_refresh | ~8 |
| `tauri/crates/tauri/src/menu/check.rs` | 5 个方法追加 auto_refresh | ~10 |
| `tauri/crates/tauri/src/menu/icon.rs` | 5 个方法追加 auto_refresh | ~10 |
| `tauri/crates/tauri/src/menu/predefined.rs` | 1 个方法追加 auto_refresh | ~2 |
| **Rust 合计** | | **~285** |

### 12.2 ArkTS 侧

| 文件 | 改动 | 行数估算 |
|------|------|---------|
| `NativeAbility.ets` | callback 按 windowId 路由 AppStorage key; notifyMenubarVisibility(windowId, visible); popupFromJson 传 windowId; setupMenuPopup 新增 colorMode; onConfigurationUpdate 新增 colorMode; WebviewController 桥接给 executor | ~40 |
| `MainPage.ets` | @StorageProp colorMode; isDark() + 颜色辅助函数; MenuBarRow onHover + hoveredItemId; bar-level Row{Image+Text}; backgroundColor/fontColor/opacity 动态; onKeyPreIme + AcceleratorMatcher; @StorageProp key 改为 ::main 后缀（menubar 2个 + popup 4个 @StorageLink→@StorageProp::main）; emit_menu_event 传 windowId | ~115 |
| `menu.ets` | hide→win.hide(); fullscreen/recover 调 notifyMenubarVisibility("main", visible); popupFromJson 接 windowId 参数 + popup AppStorage key 加 ::${windowId} 后缀; emitMenuEventFn 传 windowId | ~20 |
| **ArkTS 合计** | | **~175** |

**总计**：~460 行

---

## 十三、实施顺序

| Step | 优先级 | 内容 | 依赖 |
|------|--------|------|------|
| 前置 | P0 | tao theme 修复（ConfigChanged → ThemeChanged + theme() 读 colorMode） | 无 |
| Step 1 | P0 | A1 + C2(hide→win.hide) + C3 + controller fix + popup/menubar AppStorage key 重命名 + popupFromJson windowId | 前置完成 |
| Step 2 | P0 | A2 (auto-refresh) | Step 1 完成 |
| Step 3 | P1 | B1+B2+B3+B5+B6 (hover/icon/dark mode/style) | 前置完成 + Step 1 完成 |
| Step 4 | P1 | C1 (accelerator) | Step 2 + Step 3 完成（需要 menubar JSON 包含 accelerator 数据 + colorMode 用于样式） |

---

## 十四、验证策略

### 14.1 Rust UT

| 测试项 | 文件 | 验证内容 |
|--------|------|---------|
| `test_menu_channel_visibility_hide` | menu/mod.rs | 发送 MenuRequest(visible=Some(false), window_id=Some("main")) → recv → visible 有值 |
| `test_menu_channel_visibility_show` | menu/mod.rs | 发送 MenuRequest(visible=Some(true), window_id=Some("main")) → recv → visible 有值 |
| `test_menu_request_data_visible_serde` | menu/mod.rs | skip_serializing_if — visible=Some(true) 时序列化包含，None 时不含 |
| `test_menubar_visible_default_true` | menu/mod.rs | MENUBAR_VISIBLE 初始值为 true |
| `test_menu_request_data_window_id_serde` | menu/mod.rs | skip_serializing_if — window_id=Some("main") 时序列化包含，None 时不含 |
| `test_menubar_visible_per_window` | menu/mod.rs | hide("A")→false, is_visible("A")=false, is_visible("B")=true |

### 14.2 HAP Autotest

| 测试项 | 模块 | 验证内容 |
|--------|------|---------|
| `Menu.hide_menu_no_crash_desktop` | menu | hide_menu 不崩溃 |
| `Menu.show_menu_no_crash_desktop` | menu | show_menu 不崩溃 |
| `Menu.is_menu_visible_after_hide` | menu | hide 后 is_menu_visible 返回 false |
| `AppStorage.color_mode_dark` | menu | dark mode 下颜色正确 |
| `Menu.auto_refresh_after_set_text` | menu | set_text 后 menubar 自动更新 |
| `Menu.popup_per_window_key` | menu | popup AppStorage key 含 ::main 后缀 |

### 14.3 手动测试

| 测试项 | 验证内容 |
|--------|---------|
| MenuBar.hover_highlight | 鼠标悬浮 bar-level 项，背景色变化 |
| MenuBar.bar_level_icon | bar-level 项显示 icon |
| MenuBar.dark_mode_colors | 系统切换 dark mode，menubar 颜色自动适配 |
| MenuBar.disabled_item_visual | disabled 项灰化 + 半透明 |
| MenuBar.hide_show_cycle | hide_menu → menubar 消失 → show_menu → menubar 恢复 |
| MenuBar.auto_refresh_text | set_text 后 menubar 自动更新文字 |
| MenuBar.auto_refresh_checked | set_checked 后 menubar 下拉自动更新 checked |
| MenuBar.accelerator_ctrl_o | Ctrl+O 触发菜单项 click |
| MenuBar.accelerator_ctrl_c | Ctrl+C 触发 predefined copy |
| MenuBar.fullscreen_hide | fullscreen → menubar 隐藏 → recover → 恢复 |
| MenuBar.predefined_hide | hide predefined action → 窗口隐藏 |
| MenuBar.popup_still_works | popup 右键菜单仍正常（回归验证 AppStorage key 重命名） |
| MenuBar.popup_per_window | popup 数据路由到 ::main AppStorage key（回归验证） |

---

## 十五、风险与待确认

| 项目 | 风险 | 缓解方案 |
|------|------|---------|
| `onKeyPreIme` 获焦链传播到祖先 | 中 | 需真机验证 Column（非 focusable）的 onKeyPreIme 在子组件获焦时是否触发。官方文档确认按键事件沿获焦链从叶到根传播，但 onKeyPreIme 是否同规则需验证。若不传播，回退方案：(1) Column 设 focusable(true) + 焦点管理；(2) keyboardShortcut 在 bar-level Row 上；(3) Web 组件 onInterceptKeyEvent |
| `KeyEvent.keyText` 字段可用性 | ✅ 已确认 | ArkUI KeyEvent.keyText 从 API version 7 起可用，提供按键名称字符串（"A", "Enter", "Escape" 等） |
| WebView controller 桥接 | 中 | 需具体实现方案：DefaultXComponent 暴露 getWebviewController() → NativeAbility 在 Web 初始化后调用 executor.setController() |
| `window.Window.hide()` 主窗口可用性 | 中 | AI 回答矛盾：一说 hide() 通常只用于子窗口，一说可用于主窗口。minimize() 确认可用于主窗口。建议：先测试 hide() 在主窗口是否生效，若不生效回退 minimize()；或改用 moveToBackground()（API 9+，确认可用于主窗口） |
| AppStorage 多窗口遗留 | 低 | Phase 11 已完成 per-window key 路由（menubar+popup 6 个 key）。遗留项见 §18.8：MenuEvent window_id、CHECK_ITEMS per-window key、子窗口 @StorageProp 动态绑定等，待多窗真正实现时完成 |
| tao theme 修复范围 | 低 | 修改 tao ConfigChanged handler 是 Phase 11 前置，但不影响其他平台 |
| auto_refresh 性能 | 低 | 单次修改 1 次 TSFN 推送（~1ms），批量方法优化后 1 次。冗余推送 AppStorage dedup |
| accelerator 特殊键映射 | 低 | 有限映射表，可枚举覆盖 |

---

## 十六、不做的事

1. ❌ 菜单条拖拽排序 — 所有平台都不支持
2. ❌ 菜单条右键菜单 — 所有平台都不支持
3. ❌ Bar-level accelerator 文本显示 — 所有平台 bar-level 都不显示
4. ❌ 原生系统图标 — Windows/GTK 也是 no-op
5. ❌ Mnemonic (Alt+F) — macOS 也不支持
6. ❌ muda 层 visibility 管理 — visibility 是 UI 属性，放在 openharmony-ability per-window HashMap
7. ❌ native pasteboard 替代 WebView document.execCommand — pasteboard 只提供数据读写，不提供"粘贴到焦点"功能；WebView execCommand 更直接有效
8. ❌ 子窗口 @StorageProp 动态绑定 — ArkUI @StorageProp key 必须是编译时常量，子窗口的动态 key 需要不同机制（见 §18）

---

## 十七、审计结果（对照 openharmony-docs + arkts-helper MCP）

> 审计日期：2026-05-24
> 审计方法：对照 OHOS 官方文档（find_docs + read_doc）和华为 AI 问答助手（ask_ai / ask_ai_batch）逐项验证方案中引用的 API

### 17.1 🔴 严重问题（必须在实施前修正）

#### P0-1: `onKeyEventIntercept` 不存在 → 改用 `onKeyPreIme`

**原方案 §9.1**: 在 Column 上注册 `.onKeyEventIntercept((event: KeyEvent) => boolean)` 拦截键盘事件。

**审计发现**: ArkUI 声明式范式中**不存在 `onKeyEventIntercept` API**。华为 AI 问答确认此 API 不存在，可能混淆了 Web 组件的 `onInterceptKeyEvent`。

**替代方案**: 改用 **`onKeyPreIme`**（API 12+），理由：

| 对比项 | onKeyEventIntercept (不存在) | onKeyPreIme (正确替代) |
|--------|------------------------------|----------------------|
| 触发时机 | (假想) intercept 模式：父先拦截 | 第一派发：获焦链叶→根，叶先触发 |
| 拦截能力 | (假想) return true 阻止子组件 | return true 阻止**所有后续处理**：keyboardShortcut、输入法、onKeyEventDispatch、onKeyEvent |
| 父组件触发 | (假想) 父先触发 | 获焦链传播：子先→父后。子无 onKeyPreIme(隐式 false) → 父可拦截 |
| 焦点要求 | (假想) 无 | Column 不需 focusable(true)。子组件获焦时，Column 在获焦链中自动收到事件 |
| API 版本 | — | API 12+ |

**代码改动**：§9.1 中的 `.onKeyEventIntercept(...)` 改为 `.onKeyPreIme(...)`，回调签名兼容 `(event: KeyEvent) => boolean`。

**验证要点**：需真机确认 onKeyPreIme 在获焦链中传播到祖先 Column（官方文档说按键事件沿获焦链传播，但 onKeyPreIme 是否同此规则需实测）。若不传播，回退方案见 §15 风险表。

**按键事件数据流**（官方文档确认）：

```
第一派发: onKeyPreIme → keyboardShortcut (获焦链叶→根)
第二派发: 输入法处理
第三派发: onKeyEventDispatch → onKeyEvent (获焦链叶→根)

Web 组件特殊流程:
  第一派发 onKeyPreIme 返回 false → Web 不匹配快捷键
  第三派发 Web 内部处理 → 未消费事件 ReDispatch 回 ArkUI → 再匹配快捷键
```

**对 C1 accelerator 的影响**：

- 场景 A（accelerator 匹配）：Column 的 onKeyPreIme return true → 事件被消费 → WebView 不处理 → MenuEvent 触发 ✓
- 场景 B（无 accelerator）：Column 的 onKeyPreIme return false → 事件继续 → 第二派发(输入法) → 第三派发(WebView 处理) ✓
- 场景 C（Ctrl+C predefined Copy）：accelerator 匹配 → return true → WebView 不处理 → MenuEvent → execCommand("copy") ✓

#### P0-2: `window.Window.hide()` 主窗口可用性不确定

**原方案 §10.2**: `'hide'` case 从 `win.minimize()` 改为 `await this.win?.hide()`。

**审计发现**: 华为 AI 两次回答矛盾：
- 第一次：hide() "通常可用于子窗口或自定义浮窗"，不建议主窗口
- 第二次：hide() "适用于主窗口"

**确认事实**:
- `hide(): Promise<void>` — API 7+ (或 9+，文档不一致)
- `minimize(): Promise<void>` — API 9+，确认可用于主窗口
- `moveToBackground(): Promise<void>` — API 9+，确认可用于主窗口

**建议修正**:
- 优先测试 `hide()` 在主窗口（`windowStage.getMainWindowSync()` 获取的）是否生效
- 若不生效，回退 `minimize()`（当前实现已验证可工作）
- 或改用 `moveToBackground()` 作为"隐藏"语义的替代（将应用移至后台，不在任务栏显示快照）
- 在代码中加注释标记此不确定性，便于后续真机验证后调整

#### P0-3: WebviewController 桥接设计需具体化

**原方案 §9.3**: "通过 AppStorage 或 global callback 传递给 PredefinedActionExecutor"

**审计发现**: 方案过于笼统，未给出具体实现路径。代码审计确认：
- `PredefinedActionExecutor` 有 `setController(web_webview.WebviewController)` 方法
- `NativeAbility.ets` 只调了 `executor.setWindow(mainWindow)`，**从未调 `setController()`**
- `DefaultXComponent.ets` 创建 WebviewController 并绑定 Web 组件

**建议具体方案**:
1. DefaultXComponent 新增 `getWebviewController()` 公共方法，返回 WebviewController
2. NativeAbility 在 Web 组件初始化完成后（如 Web 的 `onPageEnd` 回调或 DefaultXComponent 的 `aboutToAppear`），调用 `executor.setController(xComponent.getWebviewController())`
3. 时序：必须在 Web 组件就绪后调用，否则 controller 仍为 null

### 17.2 ✅ 已确认正确（无需修改）

| 项目 | 验证来源 | 结论 |
|------|---------|------|
| AppStorage.setOrCreate 同值不触发 @Watch | 官方文档 + AI 问答 | ✅ 确认。setOrCreate 值与存储值严格相等时不触发同步，@Watch 不回调 |
| Configuration.colorMode 值 | AI 问答 | ✅ Dark=0, Light=1, NoSet=-1。方案中 colorMode===0 判断暗色正确 |
| NoSet 默认 Light | AI 问答 | ✅ NoSet(-1) 时系统默认 Light，方案中 `_ => Theme::Light` 正确 |
| KeyEvent.keyText 可用性 | 官方文档 | ✅ 从 API 7 起可用，提供按键名称字符串 |
| @StorageProp 单向同步 | 官方文档 | ✅ 方案用 @StorageProp(非 @StorageLink)接收 colorMode，正确（colorMode 由 AppStorage 推送） |
| @StorageProp number 类型 | 代码审计 | ✅ 已有 `@StorageProp("__openharmony_ability_menubar_visible__") menubarVisible: boolean = true`，模式一致 |
| onHover 通用事件 | 官方文档 | ✅ onHover(isHover: boolean) 是通用事件，可用于 Row/Column 等所有组件 |
| bindMenu showInSubWindow | 代码审计 | ✅ 已有 `.bindMenu(this.menuShown, this.MenuContent, { showInSubWindow: true, onWillDisappear: ... })` |
| PredefinedActionExecutor.controller null bug | 代码审计 | ✅ NativeAbility.ets 只调 setWindow() 未调 setController()，所有 execCommand 静默失败 |
| MENUBAR_VISIBLE AtomicBool 全局管理 | 代码审计 | ✅ OHOS 单窗口，全局 AtomicBool 等效 per-window。openharmony-ability 已有 MENU_CHANNEL 模式 |
| MenuRequestData visible: Option<bool> + skip_serializing_if | NAPI 规范 | ✅ #[napi(object)] optional field + serde skip_serializing_if 是标准模式 |
| auto_refresh 用 manager().windows() | 代码审计 | ✅ crate-internal 方法，不需要 unstable feature。OHOS supports_multiple_windows()==false 实际 0-1 窗口 |

### 17.3 ⚠️ 需真机验证的项

| 项目 | 验证内容 | 风险等级 |
|------|---------|---------|
| onKeyPreIme 获焦链传播 | Column(非 focusable) 在子组件获焦时，其 onKeyPreIme 是否触发 | 中 |
| window.Window.hide() 主窗口 | getMainWindowSync() 获取的主窗口调用 hide() 是否生效 | 中 |
| KeyEvent.keyText 特殊键值 | OHOS 桌面设备物理键盘的 keyText 值（Escape/Delete/Insert/PageUp/PageDown）是否与映射表一致 | 低 |
| onKeyPreIme + WebView 交互 | Column onKeyPreIme return true 是否真正阻止 Web 组件内部键盘处理 | 低（官方文档确认 return true 阻止所有后续派发） |

### 17.4 方案修正清单

| 章节 | 原内容 | 修正 |
|------|--------|------|
| §9.1 拦截机制 | `onKeyEventIntercept` (intercept 模式) | → `onKeyPreIme` (第一派发，获焦链传播)。回调签名不变。Column 不需 focusable(true) |
| §9.1 代码 | `.onKeyEventIntercept((event: KeyEvent) => ...)` | → `.onKeyPreIme((event: KeyEvent) => ...)` |
| §9.4 决策表 | "onKeyEventIntercept 而非 onKeyEvent" | → "onKeyPreIme 而非 onKeyEvent" |
| §9.4 决策表 | "intercept 模式父组件先拦截" | → "第一派发获焦链传播，子先触发(隐式 false)后父可拦截" |
| §10.2 hide 修正 | `await this.win?.hide()` | → `await this.win?.hide()` 但加注释标记主窗口不确定性 + minimize() 回退 |
| §9.3 controller 桥接 | "通过 AppStorage 或 global callback" | → DefaultXComponent.getWebviewController() + NativeAbility 调 executor.setController() 在 Web 初始化后 |
| §15 风险表 | "`onKeyEventIntercept` API 可用性" | → "`onKeyPreIme` 获焦链传播到祖先" + 3 个回退方案 |
| §15 风险表 | "`KeyEvent.keyText` 字段可用性" 中风险 | → ✅ 已确认，降为低风险（真机验证特殊键值） |
| §2 原生能力 C1 | "ArkUI `onKeyPreIme` (第一派发)" | → "ArkUI `onKeyPreIme` (第一派发)" |

---

## 十八、多窗口适配设计

> 设计目标：确保 menubar 数据管道（Rust → NAPI → ArkTS → AppStorage → 渲染）从单窗口扩展到多窗口时正确路由到每个窗口的独立 menubar
> 当前阶段：Rust/NAPI 层多窗兼容，ArkTS 渲染层先适配主窗口，子窗口 UI 待多窗功能真正实现时演进

### 18.1 OHOS 多窗口机制

OHOS 桌面多窗有两种方式：

| 方式 | 机制 | AppStorage | 适用场景 |
|------|------|-----------|---------|
| 子窗口 | 同一 UIAbility 内 `window.createWindow()` | 共享（同一进程同一 AppStorage） | Tauri `WebviewWindow::new()` 预计走此路径 |
| 多 UIAbility | 启动新 UIAbility 实例 | 独立（每个实例有自己的 AppStorage） | 独立窗口文档， |

**重点**： Tauri 多窗大概率走子窗口模式。子窗口共享同一 AppStorage，所以单 AppStorage key 模式下，两个窗口的 MainPage 组件绑定同一 key 会会导致互相覆盖。

### 18.2 数据管道改造

**核心改动**: `MenuRequest / MenuRequestData` 增加 `window_id: Option<String>` 字段，所有路由按窗口 ID 区分。

#### 请求类型区分表（修正后）

| 请求类型 | json_data | x | y | window_id | visible |
|----------|-----------|---|---|-----------|---------|
| menubar JSON | 有内容 | None | None | Some(label) | None |
| popup | 有内容 | Some | Some | Some(label) | None |
| visibility | `""` | None | None | Some(label) | Some(bool) |
| app-level menu | 有内容 | None | None | None | None |

**注意**: `window_id` 为 `None` 表示 app-level menu（macOS 模式）。OHOS 审面为 per-window 菜，`window_id` 始终由 tauri `Window::label()` 填充。

#### Rust 侧改动

**`MenuRequest`**:
```rust
pub struct MenuRequest {
    pub json_data: String,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub visible: Option<bool>,
    pub window_id: Option<String>,      // 新增
}
```

**`MenuRequestData` (NAPI)**:
```rust
#[napi(object)]
pub struct MenuRequestData {
    pub json_data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_id: Option<String>,       // 新增
}
```

**`MENUBAR_VISIBLE`**: 全局 AtomicBool → per-window `RwLock<HashMap<String, bool>>`

```rust
static MENUBAR_VISIBLE: LazyLock<RwLock<HashMap<String, bool>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub fn set_menubar_visible(visible: bool, window_id: String) -> Result<()> {
    MENUBAR_VISIBLE.write().insert(window_id, visible);
    MENU_CHANNEL.0.send(MenuRequest {
        json_data: "".to_string(),
        x: None, y: None,
        visible: Some(visible),
        window_id: Some(window_id),
    }).ok();
    Ok(())
}

pub fn is_menubar_visible(window_id: &str) -> bool {
    MENUBAR_VISIBLE.read().get(&window_id).copied().unwrap_or(true)
}

pub fn notify_menubar_visibility(visible: bool, window_id: String) {
    MENUBAR_VISIBLE.write().insert(window_id, visible);
}
```

**默认值**: HashMap 中未记录的窗口 ID 返回 `true`（初始可见），与 Windows/GTK 行为一致。

**`set_menu_json / popup_context_menu`**: 增加 `window_id` 参数

```rust
pub fn set_menu_json(json_data: String, window_id: Option<String>) -> Result<()> {
    MENU_CHANNEL.0.send(MenuRequest {
        json_data, x: None, y: None,
        visible: None, window_id,
    }).ok();
    Ok(())
}

pub fn popup_context_menu(json_data: String, x: Option<f64>, y: Option<f64>, window_id: Option<String>) -> Result<()> {
    MENU_CHANNEL.0.send(MenuRequest {
        json_data, x, y,
 visible: None, window_id,
    }).ok();
    Ok(())
}
```

**muda `refresh_menubar`**: 增加 `window_id` 参数

```rust
pub fn refresh_menubar(&self, window_id: Option<String>) -> crate::Result<()> {
    init_menu_event_listener();
    let json = self.to_json();
    openharmony_ability::menu::set_menu_json(json, window_id)
        .map_err(|e| crate::Error::CustomError(e.to_string()))?;
    Ok(())
}
```

#### tauri 侧改动

**`Window::hide_menu/show_menu/is_menu_visible`**: 传 `self.label()`

```rust
// hide_menu OHOS block:
#[cfg(target_env = "ohos")]
if let Some(_) = &*self.menu_lock() {
    openharmony_ability::menu::set_menubar_visible(false, self.label()).ok();
}

// show_menu OHOS block:
#[cfg(target_env = "ohos")]
if let Some(window_menu) = &*self.menu_lock() {
    window_menu.menu.inner().refresh_menubar(Some(self.label())).ok();
    openharmony_ability::menu::set_menubar_visible(true, self.label()).ok();
}

// is_menu_visible OHOS block:
#[cfg(target_env = "ohos")]
return Ok(openharmony_ability::menu::is_menubar_visible(&self.label()));
```

**`Window::set_menu/remove_menu`**: 传 `self.label()`

```rust
// set_menu OHOS block:
#[cfg(target_env = "ohos")]
{
    menu.inner().refresh_menubar(Some(self.label())).ok();
}

// remove_menu OHOS block:
#[cfg(target_env = "ohos")]
if let Some(_menu) = &prev_menu {
    openharmony_ability::menu::set_menu_json("[]".to_string(), Some(self.label())).ok();
}
```

**`auto_refresh_menubar`**: per-window，刷新

```rust
#[cfg(all(target_env = "ohos", desktop))]
fn auto_refresh_menubar<R: Runtime>(menu_item: &MenuItemType<R>) {
    // 刷新 app-level menu（macOS 模式）
    if let Some(menu) = menu_item.app_handle.menu() {
        menu.refresh_menubar(None).ok();  // window_id=None → app-level
    }
    // 刷新当前菜单项所属的窗口
    for window in menu_item.app_handle.manager().windows().values() {
        if let Some(wm) = window.menu_lock().as_ref() {
            wm.menu.refresh_menubar(Some(window.label())).ok();
        }
    }
}
```

**`Menu::refresh_menubar`**: 接受 `window_id` 参数

```rust
#[cfg(all(target_env = "ohos", desktop))]
pub fn refresh_menubar(&self, window_id: Option<String>) -> crate::Result<()> {
    (*self.0).as_ref().refresh_menubar(window_id).map_err(Into::into)
}
```

#### ArkTS 侧改动

**NativeAbility.ets callback**: 按 `windowId` 路由到 per-window AppStorage key

```typescript
primaryModule.onMenuRequest((data: { jsonData: string; x?: number; y?: number; visible?: boolean; windowId?: string }) => {
    const windowId = data.windowId ?? "main";  // 默认 "main"

    if (data.visible !== undefined && data.visible !== null) {
        // Visibility 请求
        AppStorage.setOrCreate("__openharmony_ability_menubar_visible__::" + windowId, data.visible);
        primaryModule.notifyMenubarVisibility(data.visible, windowId);
        return;
    }
    if (data.x !== undefined && data.y !== undefined) {
        // Popup 请求 — 传 windowId 给 popupFromJson（内部写 per-window popup key）
        menuManager.popupFromJson(data.jsonData, data.x ?? 0, data.y ?? 0, windowId);
    } else {
        // Menubar JSON 请求
        AppStorage.setOrCreate("__openharmony_ability_menubar_json__::" + windowId, data.jsonData);
    }
});
```

**AppStorage key 格式**: `__openharmony_ability_menubar_json__::${windowId}` / `__openharmony_ability_menubar_visible__::${windowId}`

`AppStorage.setOrCreate` 支持动态字符串 key，运行时拼接合法。但 `@StorageProp` 要求编译时常量 key。

**MainPage.ets**: 主窗口硬编码绑定，子窗口动态获取

```typescript
// 主窗口（label 永远是 "main"）— 编译时常量 key
@Watch("onMenubarJsonChange") @StorageProp("__openharmony_ability_menubar_json__::main") menubarJson: string = "[]";
@StorageProp("__openharmony_ability_menubar_visible__::main") menubarVisible: boolean = true;

// 子窗口（待多窗实现后）— 不用 @StorageProp，用 @State + 手动读取
// @State menubarJson: string = "";
// aboutToAppear() { this.menubarJson = AppStorage.get("__openharmony_ability_menubar_json__::" + this.windowLabel) ?? "[]"; }
// 需要额外机制监听更新（如 AppStorage @Watch 或 EventBus）— 待设计
```

**PredefinedActionExecutor fullscreen/recover**: 传窗口 ID

```typescript
case 'fullscreen':
    primaryModule.notifyMenubarVisibility(false, "main");  // 主窗口
    AppStorage.setOrCreate("__openharmony_ability_menubar_visible__::main", false);
    break;
case 'recover':
    primaryModule.notifyMenubarVisibility(true, "main");
    AppStorage.setOrCreate("__openharmony_ability_menubar_visible__::main", true);
    break;
```

**注意**: fullscreen/recover 的窗口 ID 需要从实际窗口获取，而非硬编码 "main"。子窗口场景下，PredefinedActionExecutor 也需知道自己的窗口 label。

### 18.3 设计原则

| 原则 | 说明 |
|------|------|
| Rust/NAPI 层多窗兼容 | `window_id` 字段从现在起引入，所有路由按窗口 ID 区分 |
| ArkTS 主窗口先适配 | 主窗口用编译时常量 `@StorageProp` key，可工作 |
| 子窗口 UI 后续演进 | 子窗口动态 AppStorage key 不能用 @StorageProp，需 @State + 手动监听，待多窗功能实现时设计 |
| per-window visibility | `HashMap<String, bool>` 替代全局 AtomicBool，每个窗口独立可见性 |
| 无窗口 ID 的向后兼容 | `window_id: None` → ArkTS callback 用 `"main"` 默认值，Phase 10 单窗代码无需修改 |
| popup 可能不需要 window_id | popup 在当前获焦窗口弹出，获焦窗口就是目标窗口。但子窗口 popup 仍需路由到正确窗口 → 带 window_id |

### 18.4 Phase 10 兼容性

**Phase 10 已实现代码需要更新 AppStorage key 名**。所有菜单相关 key 加 `::windowId` 后缀：

| Phase 10 代码 | 当前行为 | 多窗修正后 | 兼容性 |
|---------------|---------|------------|--------|
| `set_menu_json(json)` | 无 window_id | → `set_menu_json(json, None)` | ✅ None → ArkTS 默认 "main" |
| `popup_context_menu(json, x, y)` | 无 window_id | → `popup_context_menu(json, x, y, None)` | ✅ None → ArkTS 默认 "main" |
| AppStorage `__openharmony_ability_menubar_json__` | 单 key | → `::main` 后缀 | ❌ key 名变了，需要更新 |
| AppStorage `__openharmony_ability_menubar_visible__` | 单 key | → `::main` 后缀 | ❌ key 名变了，需要更新 |
| AppStorage `__openharmony_ability_menu_shown__` | 单 key | → `::main` 后缀 | ❌ key 名变了，需要更新 |
| AppStorage `__openharmony_ability_menu_json__` | 单 key | → `::main` 后缀 | ❌ key 名变了，需要更新 |
| AppStorage `__openharmony_ability_menu_x__` | 单 key | → `::main` 后缀 | ❌ key 名变了，需要更新 |
| AppStorage `__openharmony_ability_menu_y__` | 单 key | → `::main` 后缀 | ❌ key 名变了，需要更新 |
| `@StorageLink("__openharmony_ability_menu_shown__")` | 单 key | → `@StorageProp("...::main")` | ❌ key 名变了 + @StorageLink→@StorageProp |
| `@StorageLink("__openharmony_ability_menu_json__")` | 单 key | → `@StorageProp("...::main")` | ❌ key 名变了 + @StorageLink→@StorageProp |
| `@StorageLink("__openharmony_ability_menu_x__")` | 单 key | → `@StorageProp("...::main")` | ❌ key 名变了 + @StorageLink→@StorageProp |
| `@StorageLink("__openharmony_ability_menu_y__")` | 单 key | → `@StorageProp("...::main")` | ❌ key 名变了 + @StorageLink→@StorageProp |
| `@StorageProp("__openharmony_ability_menubar_json__")` | 单 key | → `::main` 后缀 | ❌ key 名变了，需要更新 |
| `@StorageProp("__openharmony_ability_menubar_visible__")` | 单 key | → `::main` 后缀 | ❌ key 名变了，需要更新 |
| `MenuManager.popupFromJson(json, x, y)` | 无 windowId | → `popupFromJson(json, x, y, windowId)` | ❌ 签名变了 |
| `menu.ets popup AppStorage key` | 无后缀 | → `::${windowId}` 后缀 | ❌ key 名变了 |

**需要更新**: 共 16 处 key 名/签名变更。这是**重命名 + @StorageLink→@StorageProp** 而非架构变更——Phase 10 ArkTS 代码逻辑不变，只改 key 名和装饰器。

> **@StorageLink → @StorageProp**: Phase 10 用 `@StorageLink` 做 popup 双向同步，但 AppStorage 文档不建议用 `@StorageLink` 做消息传递（触发所有绑定组件的无意义重新渲染）。popup 状态只从 AppStorage 推送到组件，不需要双向同步回 AppStorage。改为 `@StorageProp` 单向同步与 menubar 方案一致。

**向后兼容**: Rust `set_menu_json(json, None)` 与旧签名 `set_menu_json(json)` 等价（None → ArkTS 默认 "main"）。可保留旧签名作为 wrapper 过渡期。

### 18.5 风险

| 项目 | 风险 | 缓解方案 |
|------|------|---------|
| AppStorage key 重命名（16处） | 中 | Phase 10 已实现的 ArkTS 代码需要更新 key 名，但逻辑不变。改动是重命名 + @StorageLink→@StorageProp |
| 子窗口 @StorageProp 动态 key | 高 | ArkUI 限制：@StorageProp key 必须编译时常量。子窗口需 @State + 手动监听机制（待设计） |
| popup @StorageLink→@StorageProp | 低 | popup 状态只需单向同步（AppStorage→组件），不需要双向。与 menubar 方案一致 |
| popup window_id 路由 | 低 | popup 路径当前受 window_id 影响：popupFromJson 写 per-window AppStorage key，确保多窗不互相覆盖 |
| MenuEvent window_id 传播 | 低 | emit_menu_event 从 ArkTS 传 menu_id + window_id，muda event listener 过滤。单窗场景下 window_id="main"，不影响行为 |
| 全局 menu（app-level） | 低 | `window_id: None` 表示 app-level menu，OHOS 桌面为 per-window 模式，app-level menu 不常用 |
| CHECK_ITEMS 全局 HashMap | 低 | 当前 OHOS 单窗口，menu_id 不冲突。多窗口时同一 menu_id 的 check 状态需按 window_id 区分（待多窗实现时修正） |
| globalMenuClickHandler 全局单实例 | 低 | 单窗只有一套菜单，click handler 不冲突。多窗时需 per-window handler 或 handler 内按 windowId 分发（待多窗实现时修正） |

### 18.6 Popup 多窗适配

**核心改动**: `MenuManager.popupFromJson` 增加 `windowId` 参数，popup 4 个 AppStorage key 加 `::${windowId}` 后缀。

**menu.ets MenuManager.popupFromJson** — 增加 `windowId` 参数：

```typescript
popupFromJson(jsonData: string, x?: number, y?: number, windowId?: string): void {
    const wid = windowId ?? "main";
    AppStorage.setOrCreate("__openharmony_ability_menu_shown__::" + wid, false);
    AppStorage.setOrCreate("__openharmony_ability_menu_json__::" + wid, jsonData);
    AppStorage.setOrCreate("__openharmony_ability_menu_x__::" + wid, x ?? 0);
    AppStorage.setOrCreate("__openharmony_ability_menu_y__::" + wid, y ?? 0);
    setTimeout(() => {
        AppStorage.setOrCreate("__openharmony_ability_menu_shown__::" + wid, true);
    }, 0);
}
```

**MainPage.ets popup 状态绑定** — 从 `@StorageLink` 改为 `@StorageProp::main`：

```typescript
// Phase 10 原代码:
@StorageLink("__openharmony_ability_menu_shown__") menuShown: boolean = false;
@Watch("onMenuJsonChange") @StorageLink("__openharmony_ability_menu_json__") menuJson: string = "[]";
@StorageLink("__openharmony_ability_menu_x__") menuX: number = 0;
@StorageLink("__openharmony_ability_menu_y__") menuY: number = 0;

// Phase 11 改为:
@Watch("onMenuJsonChange") @StorageProp("__openharmony_ability_menu_json__::main") menuJson: string = "[]";
@StorageProp("__openharmony_ability_menu_shown__::main") menuShown: boolean = false;
@StorageProp("__openharmony_ability_menu_x__::main") menuX: number = 0;
@StorageProp("__openharmony_ability_menu_y__::main") menuY: number = 0;
```

> **@StorageLink → @StorageProp**: popup 状态只从 AppStorage 推送到组件（单向），不需要双向同步回 AppStorage。`menuShown` 由 `popupFromJson` 通过 AppStorage 控制（先设 false 再 setTimeout 设 true），组件不需要反向修改 AppStorage。改为 `@StorageProp` 与 menubar 方案一致，避免不必要的双向同步触发。

**PredefinedActionExecutor fullscreen/recover popup key** — 同样加 `::main` 后缀：

```typescript
case 'fullscreen':
    AppStorage.setOrCreate("__openharmony_ability_menu_shown__::main", false);  // 关闭 popup
    // ... (menubar visible 已在 §3.1 中覆盖)
    break;
```

### 18.7 MenuEvent 多窗适配（可延后）

**当前 MenuEvent 传播链路**:

```
ArkTS: emit_menu_event(menuId) → NAPI → MENU_EVENT_CHANNEL → muda event listener → MenuEvent::send(MenuEvent { id })
```

**需要改动（可延后至真正多窗实现时）**:

| 改动 | 文件 | 说明 | 优先级 |
|------|------|------|--------|
| MenuEvent 新增 `window_id: Option<String>` | `event.rs` | 事件携带来源窗口 ID | 中 — 单窗可工作 |
| `emit_menu_event(menuId, windowId)` 新增参数 | `menu/mod.rs` | NAPI 函数签名变更 | 中 |
| `emitMenuEventFn(item.id, windowId)` 传 windowId | `menu.ets` | ArkTS 点击回调传来源窗口 | 中 |
| `MENU_EVENT_CHANNEL` 接 `MenuEvent { id, window_id }` | `menu/mod.rs` | channel 数据变更 | 中 |
| `start_event_listener` 按 window_id 过滤 | `muda ohos/mod.rs` | 多窗时只处理属于自己窗口的事件 | 中 — 单窗无需过滤 |
| `CHECK_ITEMS` 按 `(window_id, menu_id)` 存储 | `muda ohos/mod.rs` | 防止多窗同 menu_id 冲突 | 低 |
| `globalMenuClickHandler` → per-window 分发 | `MainPage.ets` | click handler 内按 windowId 路由 | 低 |

**延后原因**: 当前 OHOS 单窗口（`supports_multiple_windows() == false`），MenuEvent 不携带 window_id 不会造成问题。只有当真正实现多窗口时，上述改动才需要完成。

**Phase 11 建议**: 只改 `emit_menu_event` 签名（增加 `window_id: Option<String>`，默认 None），ArkTS 侧暂时不传 windowId（传 None/"main"）。这样 Rust 端多窗兼容，ArkTS 端单窗行为不变。真正多窗时再在 ArkTS 传真实 windowId。

### 18.8 多窗遗留事项清单

> Phase 11 完成后，以下项目需要在未来真正实现 OHOS 多窗口功能时完成。
> 当前 OHOS `supports_multiple_windows() == false`，单窗场景下这些遗留项不影响功能正确性。

#### 18.8.1 必须完成（多窗功能上线前）

| # | 遗留项 | 当前状态 | 需要的改动 | 文件 | 说明 |
|---|--------|---------|------------|------|------|
| 1 | `MenuEvent` 无 `window_id` | `MenuEvent { id: String }` | 新增 `window_id: Option<String>` 字段 | `menu/event.rs` | 多窗时事件需携带来源窗口 ID，否则无法区分哪个窗口的菜单被点击 |
| 2 | `emit_menu_event(menuId)` 无 window_id | NAPI 函数只接 menuId | → `emit_menu_event(menuId, windowId: Option<String>)` | `menu/mod.rs` | 签名变更；Rust 端 Phase 11 预留 Option<String> |
| 3 | ArkTS `emitMenuEventFn` 不传 windowId | 只传 `item.id` | → 传 `item.id + windowId` | `menu.ets`, `MainPage.ets` | click 回调从 PredefinedActionExecutor/MenuManager 获取当前窗口 label |
| 4 | `CHECK_ITEMS` 全局 HashMap | `HashMap<String, Arc<AtomicBool>>` | → `HashMap<(String, String), Arc<AtomicBool>>`，key 为 `(window_id, menu_id)` | `muda/ohos/mod.rs` | 多窗同 menu_id 的 check 项状态会冲突 |
| 5 | `globalMenuClickHandler` 全局单实例 | `let handler: ((item) => void) \| null` | → per-window handler Map 或 handler 内按 windowId 分发 | `MainPage.ets` | 两个窗口的 click handler 会互相覆盖 |
| 6 | 子窗口 `@StorageProp` 动态 key 绑定 | 主窗口硬编码 `::main` 可工作 | → 子窗口用 `@State` + `AppStorage.get("::" + windowLabel)` + 变更监听 | `子窗口 MainPage.ets` | @StorageProp key 必须编译时常量，子窗口需不同机制（§18.3） |
| 7 | `PredefinedActionExecutor` 单 win 引用 | `private win: window.Window \| null` | → 按 windowId 存储 per-window win Map，或 execute() 接 windowId 参数 | `menu.ets` | 当前只服务于主窗口，子窗口有自己的 PredefinedActionExecutor 但无 win 引用 |
| 8 | `supports_multiple_windows()` 返回 false | 硬编码 false | → 返回 true | `tao/ohos/mod.rs` 或 `tauri runtime` | 返回 false 会阻止 JS 层创建多窗。OHOS 桌面实际支持 `window.createWindow()` |
| 9 | `MenuPopup` / `MenuStateController` NAPI class 按 menu_id 存储 | `HashMap<String, Vec<MenuItemData>>` | → key 为 `(window_id, menu_id)` | `menu/popup.rs`, `menu/state.rs` | 同 menu_id 跨窗口冲突。当前未被 muda 路径使用，但直接 NAPI 调用会冲突 |
| 10 | `menuStateController` ArkTS 全局单实例 | `export const menuStateController = new MenuStateController()` | → per-window 实例或 Map | `menu_state.ets` | 全局实例所有窗口共享状态 |

#### 18.8.2 建议完成（多窗功能上线时优化）

| # | 遗留项 | 说明 |
|---|--------|------|
| 11 | `globalPopupCallback` 全局单实例 | `MainPage.ets` 中只有一个 popup callback。多窗时需 per-window |
| 12 | `MenuState` interface 无 windowId | `menu_state.ets` 中 MenuState 缺少 windowId 字段 |
| 13 | `MenuManager` 单实例 | `menu.ets` 中 NativeAbility 创建一个 MenuManager。多窗时每个窗口需自己的 MenuManager 或 MenuManager 支持 per-window 路由 |
| 14 | fullscreen/recover 硬编码 "main" | `menu.ets` PredefinedActionExecutor 中 `notifyMenubarVisibility(false, "main")`。多窗时需从 PredefinedActionExecutor 所属窗口获取真实 windowLabel |
| 15 | `EVENT_LISTENER_STARTED` 全局 AtomicBool | `muda/ohos/mod.rs` 中单全局标志。一旦启动无法重置。多窗时如果需要 per-window event listener 初始化，需改为 per-window 管理 |

#### 18.8.3 子窗口 UI 设计要点（待设计）

当真正实现多窗口时，子窗口的页面设计需要解决以下问题：

| 问题 | 说明 |
|------|------|
| `@StorageProp` 动态 key | ArkUI 限制：key 必须编译时常量。子窗口不能 `@StorageProp("::" + this.windowLabel)` |
| 可能的方案 | (1) `@State` + `AppStorage.get()` + `AppStorage.on('change', key, callback)` 监听；(2) EventBus 跨组件通信；(3) 子窗口页面独立，不复用 MainPage，有自己的硬编码 key |
| 子窗口页面加载 | `window.createWindow()` + `setUIContent('pages/SubWindowPage')` — 子窗口加载独立页面文件 |
| 子窗口菜单数据来源 | Rust 端 `refresh_menubar(Some("secondary"))` → AppStorage `menubar_json::secondary` → 子窗口页面通过 @State 读取 |
| 子窗口 popup | `popupFromJson(json, x, y, "secondary")` → AppStorage popup key `::secondary` → 子窗口页面绑定 |
| 子窗口 PredefinedActionExecutor | 子窗口有自己的 executor 实例，需 `setWindow(subWindow)` + `setController(subWebviewController)` |
| 子窗口 onKeyPreIme | 子窗口有自己的 Column → 自己的 AcceleratorMatcher → per-window 天然隔离 |
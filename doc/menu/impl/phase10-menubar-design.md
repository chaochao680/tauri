# Phase 10: OHOS Desktop Menu Bar 设计

> 职责：在 OHOS desktop 模式下实现持久菜单栏（Menu Bar），复用 muda Menu 数据模型和现有 popup 渲染机制
> 代码位置：`tauri/crates/tauri/src/menu/menu.rs`（Rust 公开 API）、`muda/src/platform_impl/ohos/`（底层）、`openharmony-ability/`（TSFN/NAPI 桥接）、`native_ability/`（ArkTS UI 渲染）
> 独立性：✓ 仅影响 OHOS desktop 路径，不影响 Windows/macOS/Linux/mobile；mobile 模式不渲染菜单栏
> 来源：Phase 8 gap analysis 8.7 deferred → Phase 10 独立实现

---

## 一、现状分析

### 1.1 跨平台 Menu Bar 行为

| 平台 | 菜单栏位置 | 实现方式 | API 入口 |
|------|-----------|---------|---------|
| macOS | 系统 NSMenuBar（全局） | `NSMenu` + `NSApp.mainMenu` | `AppHandle::set_menu()` |
| Windows | 窗口顶部 Win32 MenuBar | `HMENU` + `SetMenu(hwnd)` | `Window::set_menu()` |
| Linux/GTK | 窗口顶部 GTK MenuBar | `GtkMenuBar` + `gtk_box_pack_start` | `Window::set_menu()` |
| OHOS | **不存在** | 无原生 API | `Window::set_menu()` 存内存但不渲染 |

### 1.2 OHOS 代码现状

**Rust 侧**：
- `AppHandle::set_menu()` 是 `#[cfg(desktop)]` — OHOS 上当 `TAURI_OHOS_DEVICE_TYPE=desktop` 时存在
- `Window::set_menu()` 在 OHOS 分支中只存储 Menu 到 `menu_lock()`，**不渲染任何内容**（`run_on_main_thread` closure 中无 `#[cfg(target_env = "ohos")]` block）
- Menu 数据通过 `Menu::popup()` → `openharmony_ability::menu::popup_context_menu()` → TSFN → ArkTS `bindMenu` **以瞬态弹窗显示**

**openharmony-ability menu 模块隔离机制**：

menu 模块通过两层隔离：

1. **Cargo feature gate**: `#[cfg(feature = "menu")]` 在 `lib.rs` 中控制整个 `menu` module 的编译和导出。只有当下游 crate（如 muda）在依赖中启用 `features = ["menu"]` 时才编译。
2. **NAPI vs Rust-only**: `#[napi]` 标注的类型/函数导出到 ArkTS（如 `MenuItemData`、`MenuPopup`、`MenuStateController`）；非 `#[napi]` 的纯 Rust API 给 muda 使用（如 `popup_context_menu()`、`menu_event_receiver()`）。

现有 Rust-only API（muda 使用）：
- `popup_context_menu(json_data, x, y)` → 发送 `PopupRequest` 到 `POPUP_CHANNEL`
- `menu_event_receiver()` → 返回 `MENU_EVENT_CHANNEL` 的 receiver
- `add_menu_event_listener()` / `dispatch_menu_event()` → 菜单事件监听/分发
- `on_popup_request(callback)` → `#[napi]` TSFN 注册
- `start_popup_forwarder()` → 启动后台线程转发 TSFN 数据
- `emit_menu_event(menu_id)` → `#[napi]` ArkTS 侧发送菜单事件

现有 `#[napi]` 类型（ArkTS 直接使用）：
- `MenuItemData` — `#[napi(object)]` 数据结构
- `AboutMetadataData` — `#[napi(object)]`
- `MenuPopup` — `#[napi]` class（菜单项存储+show/hide），目前未被 muda 使用
- `MenuStateController` — `#[napi]` class（菜单状态 CRUD），目前未被 muda 使用
- `Menu / MenuItem / Submenu` — `#[napi]` class，目前未被 muda 使用

**关键观察**：`MenuPopup`、`MenuStateController`、`Menu`、`MenuItem`、`Submenu` 这些 NAPI class 目前 ArkTS 没用到。真正被使用的 popup 流程是 TSFN + AppStorage 驱动的。

**ArkTS 侧**：
- `MainPage.ets` 使用 `Stack` 布局（不是 Navigation），顶部无菜单条
- `bindMenu` 已实现完整 popup 菜单渲染（含 separator 分组、check 项、icon 项、submenu、predefined 项）
- `MenuManager.popupFromJson()` 通过 AppStorage 驱动 popup 显示/隐藏
- `globalMenuClickHandler` 处理所有菜单项点击 → `MenuManager.handleItemClick()` → predefined action 或 emit event

**数据格式**：
- `MenuItemData`（JSON）已有完整结构：`id, type, text, enabled, accelerator, predefinedType, checked, icon, submenuItems, aboutMetadata`
- 顶级 submenu 项的 `submenuItems` 包含完整下拉内容
- 现有 popup 流程序列化整棵 Menu 树（顶级→所有子项→递归嵌套）

### 1.3 关键差距

| 差距 | 说明 |
|------|------|
| 无 Menu Bar 渲染 | desktop 模式下窗口顶部无持久菜单条 |
| `set_menu()` 无可见效果 | Menu 仅存内存，不触发任何 UI 更新 |
| 无 TSFN 桥接 menubar 数据 | 没有将菜单栏数据传递给 ArkTS 的通道 |
| ArkTS 无 menubar UI 组件 | MainPage.ets 不包含菜单条渲染逻辑 |

---

## 二、ArkUI 能力调研

### 2.1 Navigation.title(CustomBuilder)

**调研结论**：Navigation.title(CustomBuilder) 方案可行但改动大（需重构页面布局），不作为首选。

### 2.2 Navigation.menus(CustomBuilder)

**调研结论**：menus 渲染在标题栏右侧，不适合完整菜单条。

### 2.3 Stack + Row 条件渲染方案（首选）

直接在 `MainPage.ets` 的 `Stack` 顶部条件渲染一个 `Row` 菜单条。

**优势**：
- 不改变现有 Stack 布局结构
- mobile 模式自然不渲染（条件 `isDesktop && menubarItems.length > 0`）

**这是最终选择的方案。**

### 2.4 bindMenu 复用

ArkUI `bindMenu` 支持两种签名（经华为官方文档验证）：

1. `bindMenu(content: Array<MenuElement> | CustomBuilder, options?: MenuOptions)` — 响应组件点击自动弹出
2. `bindMenu(isShow: boolean, content: Array<MenuElement> | CustomBuilder, options?: MenuOptions)` — 通过 boolean 控制显隐

**三参数签名 `bindMenu(boolean, CustomBuilder, options)` 已确认可用**，且 `options` 支持 `showInSubWindow: true`、`onWillDisappear` 回调。

菜单条下拉可复用现有 `RenderMenuItems` builder（90% 复用度）。

### 2.5 设备类型检测

`TAURI_OHOS_DEVICE_TYPE` 环境变量在 ArkTS 侧不可用。需要通过 NAPI 函数将编译时信息传递给 ArkTS。

---

## 三、设计方案

### 3.0 审计修正总结

基于 napi-ohos skill、ArkUI 官方文档、现有代码审计，原设计有以下修正：

| # | 原设计问题 | 修正 |
|---|-----------|------|
| 1 | `MenubarRequest` 和 `PopupRequest` 分别有独立 channel + forwarder 线程 | **合并**为 `MenuRequest`，共享一个 `MENU_CHANNEL` + 一个 `forwarder` 线程，避免创建多余 OS thread |
| 2 | `MenubarRequestData` 和 `PopupRequestData` 分别是 `#[napi(object)]` | **合并**为 `MenuRequestData { json_data, x?, y? }`，popup 时 x/y 有值，menubar 时 x/y 为 None/undefined |
| 3 | `on_popup_request` + `on_menubar_request` 分别注册 TSFN callback | **合并**为 `on_menu_request`，一个 TSFN callback 处理两种请求 |
| 4 | `start_popup_forwarder` + `start_menubar_forwarder` 分别创建线程 | **合并**为 `start_menu_forwarder`，一个线程转发所有 menu TSFN 数据 |
| 5 | `popup_context_menu` + `set_menubar_json` 分别调用 | **合并**为 `set_menu_json`（menubar 路径 x=None）和 `popup_context_menu`（popup 路径 x/y 有值），两者都发到同一个 `MENU_CHANNEL` |
| 6 | `is_desktop_device()` 放在 `menu/mod.rs` | **移到 `app.rs`**，设备类型检测不是 menu 特有的功能 |
| 7 | ~~openharmony-ability 需要 Cargo feature `desktop`~~ | **❌ 不需要 Cargo feature `desktop`**。与 popup 方案一致：openharmony-ability/muda 不新增 `desktop` feature，不新增 build.rs。tauri build.rs 已定义 `cfg(desktop)`（基于 `TAURI_OHOS_DEVICE_TYPE`），所有 `#[cfg(desktop)]` 判断在 tauri crate 内直接可用。openharmony-ability/muda 中需要 desktop 判断的代码使用 `#[cfg(all(target_env = "ohos", desktop))]`（tauri build.rs 的 cfg 输出通过 rustc-cfg 传播到所有依赖 crate） |
| 8 | `lib.rs` 导出 `on_popup_request, start_popup_forwarder, PopupRequestData` | 更新为 `on_menu_request, start_menu_forwarder, MenuRequestData, is_desktop_device` |
| 9 | ArkTS `@Watch` + `@StorageLink` menubarJson | AppStorage 文档警告不建议用 `@StorageLink` 双向同步做消息传递。改为 `@StorageProp`（单向同步）。menubarJson 不需要双向同步回 AppStorage |
| 10 | ~~TSFN `.build()` 替代 `.build_callback()`~~ | **❌ 继续用 `.build_callback()`**。现有 `on_popup_request` 用 `.build_callback()`（menu/mod.rs:91），经源码审计确认这是正确用法。`#[napi(object)]` 类型不能直接 `.build()` — napi-ohos 中 `#[napi(object)]` 生成的是 struct 级转换，不实现 `JsValuesTupleIntoVec`。合并后的 `on_menu_request` 同样用 `.build_callback()` |
| 11 | ~~openharmony-ability/muda 需要 Cargo feature `desktop` 替代 `cfg(desktop)`~~ | **❌ 不需要**。tauri build.rs 输出的 `cfg(desktop)` 通过 `cargo:rustc-cfg=desktop` 传播到所有依赖 crate（包括 openharmony-ability 和 muda）。所以 `#[cfg(desktop)]` 和 `cfg!(desktop)` 在 openharmony-ability/muda 中**直接可用**，不需要 Cargo feature 间接传播 |
| 12 | ArkTS `@StorageLink("__openharmony_ability_is_desktop__")` | 改为 `@StorageProp`（只读，不需要双向同步） |

**3.0 审计修正核心变更**：原方案试图通过 Cargo feature `desktop` 在依赖链中传播 desktop 状态，但审计发现：
1. tauri build.rs 输出的 `cfg(desktop)` 已经通过 `cargo:rustc-cfg` 自动传播到所有依赖 crate
2. 不需要在 Cargo.toml 中做 `cfg(all(target_env = "ohos", desktop))` 的依赖分支（Cargo.toml 的 cfg 条件在 Cargo 解析阶段评估，此时 build.rs 还没运行，`cfg(desktop)` 可能不生效）
3. popup 方案不依赖任何 Cargo feature desktop 传播就能正常工作

简化后的方案与 popup 完全一致：代码层用 `#[cfg(desktop)]` 控制桌面逻辑，Cargo.toml 不变。

### 3.1 整体架构

```
Rust 侧                              ArkTS 侧

AppHandle::set_menu()  ─────►  Window::set_menu() #[cfg(desktop)] OHOS block
                                │  (直接调用, bypass run_on_main_thread)
                                │
                                ▼
                      Menu::refresh_menubar()
                                │  (init_menu_event_listener + to_json + set_menu_json)
                                │
                                ▼
                      set_menu_json(json) → MENU_CHANNEL (x=None, y=None)
                               │
                               ▼
                     menu_forwarder thread → MENU_CALLBACK TSFN
                               │
                               ▼
                     ArkTS on_menu_request callback
                               │
                               ├─ if data.x != undefined → AppStorage(popup keys)
                               └─ if data.x == undefined → AppStorage(menubar keys)
                               │
                               ▼
                     AppStorage.setOrCreate("__openharmony_ability_menubar_json__")
                               │
                               ▼
                     MainPage.ets @StorageProp + @Watch → onMenubarJsonChange → menubarItems
                               │
                               ▼
                     build() → if isDesktop && menubarItems.length > 0 → MenuBarRow
                               │
                               ▼
                     Text("File") ── onClick ── set activeDropdownId + shown=true
                               │
                               ▼
                     bindMenu(true/false, MenubarDropdownContent, { showInSubWindow })
                               │
                               ▼ (click item)
                     globalMenuClickHandler → MenuManager.handleItemClick
                               │
                               ▼
                     emit_menu_event(menuId) → Rust MENU_EVENT_CHANNEL
```

**核心思路**：合并 popup 和 menubar 的 TSFN 桥接为单一通道，根据 `x/y` 是否有值区分 popup 和 menubar 请求。与 popup 方案完全一致的模式（直接调用、bypass `run_on_main_thread`、AppStorage 驱动渲染）。

### 3.2 Rust 侧改动

#### 3.2.1 ~~openharmony-ability Cargo feature `desktop`~~（不需要）

**审计结论**：不需要在 openharmony-ability/muda 新增 Cargo feature `desktop`。

原因：
1. tauri build.rs 通过 `cargo:rustc-cfg=desktop` 输出的 cfg 标志**自动传播到所有依赖 crate**（包括 openharmony-ability 和 muda）
2. `#[cfg(desktop)]` 和 `cfg!(desktop)` 在 openharmony-ability/muda 中**直接可用**
3. popup 方案不依赖 Cargo feature desktop 传播就能正常工作
4. Cargo.toml 的 `[target.'cfg(...)']` 条件在 Cargo 解析阶段评估（build.rs 还没运行），`cfg(desktop)` 条件可能不生效

**openharmony-ability/crates/ability/Cargo.toml 不改动**。

#### 3.2.2 openharmony-ability 合并 popup/menubar TSFN 桥接

**文件**：`openharmony-ability/crates/ability/src/menu/mod.rs`

将 `PopupRequest`、`PopupRequestData`、`POPUP_CHANNEL`、`POPUP_CALLBACK` 合并为 `MenuRequest`、`MenuRequestData`、`MENU_CHANNEL`、`MENU_CALLBACK`：

```rust
/// Menu request data (unified popup + menubar)
#[derive(Debug, Clone)]
pub struct MenuRequest {
    pub json_data: String,
    pub x: Option<f64>,
    pub y: Option<f64>,
}

/// Menu request data for NAPI
#[derive(Debug, Clone, serde::Serialize)]
#[napi(object)]
pub struct MenuRequestData {
    pub json_data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,
}

// Channel: Rust → ArkTS (unified for popup + menubar)
static MENU_CHANNEL: LazyLock<(Sender<MenuRequest>, Receiver<MenuRequest>)> =
    LazyLock::new(unbounded);

// TSFN callback: Rust → ArkTS (CalleeHandled=false → JS callback: (data) => void)
type MenuTsfn = ThreadsafeFunction<MenuRequestData, Unknown<'static>, FnArgs<(MenuRequestData,)>, Status, false>;
static MENU_CALLBACK: Mutex<Option<MenuTsfn>> = Mutex::new(None);

/// NAPI: ArkTS registers unified menu callback
#[napi(ts_args_type = "callback: (data: MenuRequestData) => void")]
pub fn on_menu_request(callback: Function<'static>) -> Result<()> {
    let tsfn: MenuTsfn = callback
        .build_threadsafe_function::<MenuRequestData>()
        .callee_handled::<false>()
        .build_callback(|ctx: ThreadsafeCallContext<MenuRequestData>| {
            Ok(FnArgs { data: (ctx.value,) })
        })?;
    let mut guard = MENU_CALLBACK.lock().map_err(|_| Error::from_reason("lock poisoned"))?;
    *guard = Some(tsfn);
    Ok(())
}

/// Rust API: Popup context menu (for muda) — x/y coordinates present
pub fn popup_context_menu(json_data: String, x: Option<f64>, y: Option<f64>) -> Result<()> {
    MENU_CHANNEL.0.send(MenuRequest { json_data, x, y }).ok();
    Ok(())
}

/// Rust API: Set menu bar JSON (for menubar) — x/y absent
pub fn set_menu_json(json_data: String) -> Result<()> {
    MENU_CHANNEL.0.send(MenuRequest { json_data, x: None, y: None }).ok();
    Ok(())
}

/// Background thread: forward menu data to ArkTS (unified)
pub fn start_menu_forwarder() {
    std::thread::spawn(|| {
        let receiver = &MENU_CHANNEL.1;
        while let Ok(req) = receiver.recv() {
            let guard = MENU_CALLBACK.lock().ok();
            if let Some(tsfn) = guard.as_ref().and_then(|g| g.as_ref()) {
                let data = MenuRequestData {
                    json_data: req.json_data,
                    x: req.x,
                    y: req.y,
                };
                tsfn.call(data, ThreadsafeFunctionCallMode::NonBlocking);
            }
        }
    });
}
```

**审计修正**：继续用 `.build_callback()`（与现有 popup 一致），不切换到 `.build()`。原因见 §3.0 修正 #10。

**保留旧 API 向后兼容**（过渡期）：

```rust
// Backward compat: keep old names as wrappers
pub fn on_popup_request(callback: Function<'static>) -> Result<()> {
    on_menu_request(callback)
}

pub fn start_popup_forwarder() {
    start_menu_forwarder()
}

pub fn popup_request_receiver() -> &'static Receiver<MenuRequest> {
    &MENU_CHANNEL.1
}

pub fn menu_request_receiver() -> &'static Receiver<MenuRequest> {
    &MENU_CHANNEL.1
}
```

#### 3.2.3 openharmony-ability 新增：`is_desktop_device` NAPI 函数

**文件**：`openharmony-ability/crates/ability/src/app.rs`（不是 menu/mod.rs）

设备类型检测不是 menu 特有的功能，放在 `app.rs` 更合理：

```rust
#[napi]
#[cfg(target_env = "ohos")]
pub fn is_desktop_device() -> bool {
    cfg!(desktop)
}
```

**实现修正**：
- 加了 `#[cfg(target_env = "ohos")]` gate — ohrs standalone 编译时此函数不存在（避免 mobile 编译 warning）
- 放在 `app.rs` impl block **之后**（不是内部），避免提前关闭 impl block
- `#[napi]` 仍然存在，但只在 OHOS 编译时生效

#### 3.2.4 lib.rs 更新导出

**文件**：`openharmony-ability/crates/ability/src/lib.rs`

```rust
#[cfg(feature = "menu")]
pub use menu::{
    on_menu_request, on_popup_request,
    start_menu_forwarder, start_popup_forwarder,
    MenuRequestData,
    popup_context_menu, set_menu_json,
    menu_event_receiver, popup_request_receiver, menu_request_receiver,
};

#[cfg(feature = "menu")]
pub use app::is_desktop_device;
```

#### 3.2.5 muda 侧改动

**文件**：`muda/Cargo.toml`（不改动）

不需要新增 Cargo feature `desktop` — `cfg(desktop)` 通过 tauri build.rs 自动传播。

**文件**：`muda/src/platform_impl/ohos/mod.rs`

新增 `Menu::refresh_menubar()` 方法：

```rust
pub fn refresh_menubar(&self) -> crate::Result<()> {
    init_menu_event_listener();
    let json = self.to_json();
    openharmony_ability::menu::set_menu_json(json)
        .map_err(|e| crate::Error::CustomError(e.to_string()))?;
    Ok(())
}
```

**注意**：不加 `#[cfg(desktop)]`。muda 编译独立于 tauri（通过 ohrs），不依赖 cfg(desktop) 传播。作为 inherent method，mobile 编译时此函数也存在但调用不会触发 menubar 渲染（ArkTS 条件渲染 gate）。与 `popup()` 方法保持一致（同样无 cfg gate）。

#### 3.2.6 tauri 侧改动

**文件**：`tauri/crates/tauri/Cargo.toml`（不改动）

不需要新增 `[target.'cfg(all(target_env = "ohos", desktop))']` section。现有 `[target.'cfg(target_env = "ohos")'.dependencies]` section 保持不变。desktop/mobile 区分在 Rust 代码层通过 `#[cfg(desktop)]` 控制。

**文件**：`tauri/crates/tauri/src/window/mod.rs`

`Window::set_menu()` 在 `#[cfg(desktop)]` impl block 内（line 1197）。新增 OHOS 分支，与 popup 方案一致——直接调用，bypass `run_on_main_thread`：

```rust
// 在 run_on_main_thread 闭包之前添加 OHOS 分支（注意：#[cfg(target_env = "ohos")] 无 desktop）
#[cfg(target_env = "ohos")]
{
    menu.inner().refresh_menubar().ok();
}

// 非 OHOS: 原有 run_on_main_thread 闭包保持不变（不含 OHOS 分支）
// 注意：不加 #[cfg(not(target_env = "ohos"))] wrapper — 闭包内已有各平台 cfg
self.run_on_main_thread(move || {
    // ... windows/linux/mac branches ...
})?;
```

**注意**：
- OHOS 分支放在 `run_on_main_thread` 闭包**之外**。OHOS bypass `run_on_main_thread`（与所有其他 OHOS 方法一致，避免 Chrome_IOThread deadlock）
- 使用 `#[cfg(target_env = "ohos")]` 而非 `#[cfg(all(target_env = "ohos", desktop))]` — 因为此代码在 `#[cfg(desktop)]` impl block 内，desktop 已隐含
- 不在原有 `run_on_main_thread` 闭包上加 `#[cfg(not(target_env = "ohos"))]` wrapper — 闭包内已有各平台 `#[cfg]` 隔离 dead code

**Bug 3 修正**：原设计写 `menu.inner().to_json()` + `set_menu_json(json)`，实际改为 `menu.inner().refresh_menubar()`。原因：如果首次菜单操作是 `set_menu`（而非 `popup`），`init_menu_event_listener()` 和 `collect_check_items()` 从未被调用，导致 check 项 toggle 不触发事件、MenuEvent 不 fire 到 tauri。`refresh_menubar()` 内部依次调用 `init_menu_event_listener()` + `to_json()` + `set_menu_json()`，确保所有必要初始化在首次操作时完成。`remove_menu` 不需要此处理（只是清空 UI，无需事件监听）。

`Window::remove_menu()` 新增 OHOS 分支清空菜单栏：

```rust
#[cfg(target_env = "ohos")]
if let Some(_menu) = &prev_menu {
    openharmony_ability::menu::set_menu_json("[]".to_string()).ok();
}
```

**注意**：`"[]".to_string()` — `set_menu_json` 参数是 `String` 不是 `&str`，必须 `.to_string()`。

**文件**：`tauri/crates/tauri/src/menu/menu.rs`

新增 `Menu::refresh_menubar()` 公开 API：

```rust
#[cfg(all(target_env = "ohos", desktop))]
pub fn refresh_menubar(&self) -> crate::Result<()> {
    (*self.0).as_ref().refresh_menubar().map_err(Into::into)
}
```

**注意**：
- OHOS 直接调用 muda 的 `refresh_menubar()`，不使用 `run_item_main_thread!`。与 popup 方案一致
- `.map_err(Into::into)` — `tauri::Error` 有 `Menu(#[from] muda::Error)` variant（在 `#[cfg(desktop)]` 下），`muda::Error` 自动转换
- 此方法在 impl block 内，`#[cfg(all(target_env = "ohos", desktop))]` 是完整 gate（impl block 无 desktop 隐含）

### 3.3 ArkTS 侧改动

#### 3.3.1 NativeAbility.ets — 注册统一 menu TSFN 回调

**文件**：`native_ability/src/main/ets/ability/NativeAbility.ets`

在 `setupMenuPopup` 方法中改为使用合并后的 `on_menu_request`：

```typescript
// 方法名保留 setupMenuPopup（未重命名），但内容改为统一 onMenuRequest
private setupMenuPopup(windowStage: window.WindowStage): void {
    const primaryModule: ESObject = this.nativeModules[0] as ESObject;
    if (!primaryModule || typeof primaryModule.onMenuRequest !== 'function') {
        return;
    }

    // Write desktop flag first (before UI renders)
    AppStorage.setOrCreate("__openharmony_ability_is_desktop__", primaryModule.isDesktopDevice());

    // Register unified menu TSFN callback
    primaryModule.onMenuRequest((data: { jsonData: string; x?: number; y?: number }) => {
        if (data.x !== undefined && data.y !== undefined) {
            // Popup request — set popup AppStorage keys
            menuManager.popupFromJson(data.jsonData, data.x ?? 0, data.y ?? 0);
        } else {
            // Menubar request — set menubar AppStorage key
            AppStorage.setOrCreate("__openharmony_ability_menubar_json__", data.jsonData);
        }
    });
}
```

**实现修正**：
- `isDesktopDevice()` 通过 `primaryModule.isDesktopDevice()` 调用（NAPI 导出），不是直接 import
- `onMenuRequest` 通过 `primaryModule.onMenuRequest` 注册（NAPI 导出）
- Popup 路径使用 `menuManager.popupFromJson()` 而非直接设 AppStorage，popup 位置计算由 MenuManager 处理
- Menubar 路径直接设 `AppStorage("__openharmony_ability_menubar_json__")`
- 方法名保留 `setupMenuPopup` 未重命名为 `setupMenu`

**审计修正**：原设计将 popup 和 menubar 的 TSFN callback 分开注册，改为统一注册。ArkTS callback 根据 `data.x` 是否有值区分 popup 和 menubar 请求。

**AppStorage 跨线程安全**：华为官方文档确认 `AppStorage.setOrCreate` 只能在 UI 主线程调用。TSFN callback 在 JS 主线程执行，而 ArkUI 的 JS 主线程就是 UI 主线程。所以从 TSFN callback 调用 `AppStorage.setOrCreate` 是安全的。

#### 3.3.2 MainPage.ets — 条件渲染菜单条

**文件**：`native_ability/src/main/ets/components/MainPage.ets`

新增状态变量（审计修正：`@StorageProp` 替代 `@StorageLink`，单向同步）：

```typescript
@StorageProp("__openharmony_ability_is_desktop__") isDesktop: boolean = false;
@Watch("onMenubarJsonChange") @StorageProp("__openharmony_ability_menubar_json__") menubarJson: string = "[]";
@StorageProp("__openharmony_ability_menubar_visible__") menubarVisible: boolean = true;
@State private menubarItems: MenuItemData[] = [];
@State private activeDropdownId: string = "";
@State private activeDropdownShown: boolean = false;
```

**审计修正说明**：
- `isDesktop` 用 `@StorageProp` — 只读，不需要双向同步回 AppStorage
- `menubarJson` 用 `@StorageProp` — 单向从 AppStorage 到组件。AppStorage 文档警告不建议用 `@StorageLink` 双向同步做消息传递（会触发所有绑定组件的无意义重新渲染）
- `menubarVisible` 用 `@StorageProp` — 只读
- `activeDropdownId`/`activeDropdownShown` 用 `@State` — 组件内部状态

新增 handler：

```typescript
onMenuJsonChange(): void {
    try {
        const items = JSON.parse(this.menuJson) as MenuItemData[];
        this.popupIconIds.clear();
        this.collectIconIds(items, this.popupIconIds);
        this.prepareIcons(items);
        this.cleanupStaleIcons();
        this.menuItems = items;
    } catch (e) {
        this.popupIconIds.clear();
        this.cleanupStaleIcons();
        this.menuItems = [];
    }
}

onMenubarJsonChange(): void {
    try {
        const items = JSON.parse(this.menubarJson) as MenuItemData[];
        this.menubarIconIds.clear();
        this.collectIconIds(items, this.menubarIconIds);
        this.prepareIcons(items);
        this.cleanupStaleIcons();
        this.menubarItems = items.filter(i => i.type === 'submenu' && i.text);
    } catch (e) {
        this.menubarIconIds.clear();
        this.cleanupStaleIcons();
        this.menubarItems = [];
    }
}
```

**Bug 1 修正**：原设计 `onMenubarJsonChange` 只做 `JSON.parse` + `filter submenu`，缺少 `prepareIcons()` + `collectIconIds()` + `cleanupStaleIcons()`。`RenderMenuItems` 对 icon 项的渲染条件是 `this.iconPixelMaps.has(item.id)`，如果不调用 `prepareIcons`，icon 项降级为纯文本 MenuItem。

**Bug 2 修正**：原设计 `onMenuJsonChange` (popup) 使用 `clear()` + `release()` 清空整个 `iconPixelMaps`，会误删 menubar 下拉菜单正在使用的 PixelMap。改为增量清理机制：维护 `popupIconIds` 和 `menubarIconIds` 两套 ID 集合，`cleanupStaleIcons()` 只释放不在 `popupIconIds ∪ menubarIconIds` 中的 PixelMap，确保 popup/menubar 互相保护对方的图标数据。

**新增状态变量**：

```typescript
private popupIconIds: Set<string> = new Set();
private menubarIconIds: Set<string> = new Set();
```

**新增辅助方法**（均使用循环+BFS队列遍历菜单树，不使用递归）：

```typescript
private prepareIcons(items: MenuItemData[]): void {
    const queue: MenuItemData[] = [...items];
    while (queue.length > 0) {
        const item = queue.shift()!;
        if (item.icon) {
            try {
                const decoded = buffer.from(item.icon, 'base64');
                const arrayBuf = decoded.buffer as ArrayBuffer;
                const imageSource = image.createImageSource(arrayBuf);
                const pixelMap = imageSource.createPixelMapSync();
                imageSource.release();
                this.iconPixelMaps.set(item.id, pixelMap);
            } catch (e) {
                console.error(`[Menu] prepareIcons failed for ${item.id}: ${e}`);
            }
        }
        if (item.submenuItems) {
            queue.push(...item.submenuItems);
        }
    }
}

private cleanupStaleIcons(): void {
    const liveIds = new Set<string>();
    for (const id of this.popupIconIds) { liveIds.add(id); }
    for (const id of this.menubarIconIds) { liveIds.add(id); }
    for (const [id, pm] of this.iconPixelMaps) {
        if (!liveIds.has(id)) {
            pm.release();
            this.iconPixelMaps.delete(id);
        }
    }
}

private collectIconIds(items: MenuItemData[], ids: Set<string>): void {
    const queue: MenuItemData[] = [...items];
    while (queue.length > 0) {
        const item = queue.shift()!;
        if (item.icon) {
            ids.add(item.id);
        }
        if (item.submenuItems) {
            queue.push(...item.submenuItems);
        }
    }
}
```

**注意**：`JSON.parse` 反序列化后的 JSON 对象用的是 serde 序列化名 `"type"`（因为 `MenuItemData` 有 `#[serde(rename = "type")]`），所以 `i.type` 是正确的。而 ArkTS NAPI 导出用的是 `#[napi(js_name = "type")]`（即 `item.type`），两者一致。

```typescript
@Builder
MenuBarRow() {
    Row() {
        ForEach(this.menubarItems, (item: MenuItemData) => {
            Text(item.text ?? '')
                .fontSize(14)
                .fontColor('#333333')
                .padding({ left: 12, right: 12, top: 10, bottom: 10 })
                .enabled(item.enabled ?? true)
                .backgroundColor(this.activeDropdownId === item.id ? '#E0E0E0' : '#F5F5F5')
                .onClick(() => {
                    this.activeDropdownId = item.id;
                    this.activeDropdownShown = true;
                })
                .bindMenu(
                    this.activeDropdownShown && this.activeDropdownId === item.id,
                    this.MenubarDropdownContent(item),
                    {
                        showInSubWindow: true,
                        onWillDisappear: () => {
                            this.activeDropdownShown = false;
                            this.activeDropdownId = "";
                        }
                    }
                )
        }, (item: MenuItemData) => item.id)
    }
    .width("100%")
    .height(40)
    .backgroundColor('#F5F5F5')
    .alignItems(VerticalAlign.Center)
}

@Builder
MenubarDropdownContent(item: MenuItemData) {
    Menu() {
        this.RenderMenuItems(item.submenuItems ?? [])
    }
}
```

修改 `build()` — 在 Stack 内 Column 顶部条件渲染菜单条：

```typescript
build() {
    Stack() {
        Column() {
            if (this.isDesktop && this.menubarItems.length > 0 && this.menubarVisible) {
                this.MenuBarRow()
            }

            Row() {
                Column() {
                    if (this.primaryModuleName) {
                        DefaultXComponent({ moduleName: this.primaryModuleName });
                    }
                }.width("100%")
            }
            .layoutWeight(1)
            .height("100%")

        }.width("100%").height("100%")

        Column()
            .width(1).height(1)
            .position({ x: 0, y: 0 })
            .bindMenu(this.menuShown, this.MenuContent, {
                anchorPosition: { x: this.menuX, y: this.menuY },
                showInSubWindow: true,
                onWillDisappear: () => { this.menuShown = false; }
            })
    }
}
```

### 3.4 菜单栏更新场景

显式 API `Menu::refresh_menubar()` — 开发者修改菜单项后手动调用。

原因：
1. 菜单栏是持久 UI，不像 popup 是一次性显示
2. 避免每次 `set_text` 都触发跨线程 TSFN 调用
3. 与 Windows/macOS 行为一致

### 3.5 fullscreen 沉浸式模式下的菜单条

在 `PredefinedActionExecutor.execute('fullscreen')` ArkTS 侧，设置 AppStorage：

```typescript
case 'fullscreen':
    AppStorage.setOrCreate("__openharmony_ability_menubar_visible__", false);
    break;
case 'recover':
    AppStorage.setOrCreate("__openharmony_ability_menubar_visible__", true);
    break;
```

---

## 四、数据流详解

### 4.1 初始化流程

```
1. Rust: TAURI_OHOS_DEVICE_TYPE=desktop → tauri build.rs: cfg(desktop) enabled
2. Rust: cfg(desktop) 通过 cargo:rustc-cfg=desktop 传播到所有依赖 crate
3. ArkTS: NativeAbility.onWindowStageCreate → load native module
4. ArkTS: call is_desktop_device() → true (cfg!(desktop)) → AppStorage("__openharmony_ability_is_desktop__", true)
5. ArkTS: call on_menu_request(callback) + start_menu_forwarder()
6. Rust: AppHandle::set_menu(menu) → Window::set_menu(menu)
7. Rust: #[cfg(target_env = "ohos")] block → menu.inner().refresh_menubar() → init_menu_event_listener + to_json + set_menu_json → MENU_CHANNEL (x=None, y=None)
8. Rust: menu_forwarder thread → MENU_CALLBACK TSFN → ArkTS callback
9. ArkTS: callback → data.x undefined → menubar path → AppStorage("__openharmony_ability_menubar_json__", json)
10. ArkTS: MainPage @StorageProp + @Watch → onMenubarJsonChange → menubarItems updated
11. ArkTS: build() → if isDesktop && menubarItems.length > 0 → render MenuBarRow
```

### 4.2 Popup 流程（与 menubar 共享同一 TSFN）

```
1. Rust: Menu::popup() → popup_context_menu(json, x, y) → MENU_CHANNEL (x=Some, y=Some)
2. Rust: menu_forwarder thread → MENU_CALLBACK TSFN → ArkTS callback
3. ArkTS: callback → data.x defined → popup path → AppStorage(popup keys)
4. ArkTS: MainPage @StorageProp → menuShown=true, menuX, menuY → bindMenu popup shown
```

### 4.3 点击菜单项下拉流程

```
1. ArkTS: User clicks Text("File") → onClick handler
2. ArkTS: activeDropdownId = item.id, activeDropdownShown = true
3. ArkTS: Text.backgroundColor switches to '#E0E0E0' (active highlight)
4. ArkTS: bindMenu(true, MenubarDropdownContent, { showInSubWindow }) renders dropdown
5. ArkTS: User clicks dropdown item → globalMenuClickHandler(item)
6. ArkTS: MenuManager.handleItemClick → predefined action or emitMenuEventFn(item.id)
7. ArkTS: onWillDisappear → activeDropdownShown = false, activeDropdownId = ""
8. ArkTS: emit_menu_event(menuId) → NAPI → Rust MENU_EVENT_CHANNEL
9. Rust: muda event listener → MenuEvent::send(MenuEvent { id })
10. Rust: tauri → on_menu_event callback
```

### 4.4 移除菜单栏流程

```
1. Rust: Window::remove_menu() → #[cfg(target_env = "ohos")] → set_menu_json("[]".to_string())
2. ArkTS: callback → AppStorage("__openharmony_ability_menubar_json__", "[]")
3. ArkTS: onMenubarJsonChange → menubarItems = [] (empty)
4. ArkTS: build() → menubarItems.length === 0 → no MenuBarRow rendered
```

### 4.5 refresh_menubar 流程

```
1. Rust: MenuItem.set_text("New Label") → only modifies in-memory data
2. Rust: menu.refresh_menubar() → init_menu_event_listener + to_json + set_menu_json(new_json)
3. ArkTS: callback → AppStorage update → onMenubarJsonChange → re-render MenuBarRow
4. ArkTS: Text("New Label") displayed
```

### 4.6 popup 与 menubar 共享机制对比

| 机制 | Popup | Menubar | 共享 |
|------|-------|---------|------|
| TSFN callback | `on_menu_request` | `on_menu_request` | ✅ 同一个 |
| forwarder 线程 | `start_menu_forwarder` | `start_menu_forwarder` | ✅ 同一个 |
| Channel | `MENU_CHANNEL` | `MENU_CHANNEL` | ✅ 同一个 |
| Rust→ArkTS 数据 | `MenuRequestData { json, x=Some, y=Some }` | `MenuRequestData { json, x=None, y=None }` | ✅ 同一个类型 |
| ArkTS callback 分支 | `data.x != undefined` → popup keys | `data.x == undefined` → menubar keys | ✅ 同一个回调 |
| AppStorage keys | 4个 popup keys | 1个 menubar key | ❌ 不同 key |
| ArkTS 渲染 | bindMenu on 1x1 anchor | 条件渲染 MenuBarRow | ❌ 不同位置 |
| 点击处理 | globalMenuClickHandler | globalMenuClickHandler | ✅ 相同 |
| 事件回调 | emit_menu_event | emit_menu_event | ✅ 相同 |
| Rust bypass run_on_main_thread | ✅ 直接调用 | ✅ 直接调用 | ✅ 相同模式 |

---

## 五、改动清单

### 5.1 Rust 侧

| 文件 | 改动 | 行数估算 |
|------|------|---------|
| `openharmony-ability/crates/ability/src/menu/mod.rs` | 合并 PopupRequest→MenuRequest, PopupRequestData→MenuRequestData+type alias, POPUP_CHANNEL→MENU_CHANNEL, POPUP_CALLBACK→MENU_CALLBACK; 新增 on_menu_request(), set_menu_json(), start_menu_forwarder(), 保留向后兼容 wrapper; 4 个 Rust UT | ~225 行 (含原有+新增) |
| `openharmony-ability/crates/ability/src/app.rs` | 新增 `is_desktop_device()` #[napi] #[cfg(target_env = "ohos")] cfg!(desktop) — 放在 impl block 之后 | ~4 行 |
| `openharmony-ability/crates/ability/src/lib.rs` | 更新导出：on_menu_request, start_menu_forwarder, MenuRequestData, PopupRequestData, set_menu_json, is_desktop_device 等 | ~7 行 |
| `muda/src/platform_impl/ohos/mod.rs` | 新增 `Menu::refresh_menubar()` (无 #[cfg(desktop)], inherent method, init_menu_event_listener + to_json + set_menu_json) | ~7 行 |
| `muda/src/menu.rs` | 新增 `Menu::to_json()` #[cfg(target_env = "ohos")] + `Menu::refresh_menubar()` #[cfg(target_env = "ohos")] | ~5 行 |
| `tauri/crates/tauri/src/menu/menu.rs` | 新增 `Menu::refresh_menubar()` #[cfg(all(target_env = "ohos", desktop))] with .map_err(Into::into) | ~4 行 |
| `tauri/crates/tauri/src/window/mod.rs` | Window::set_menu() #[cfg(target_env = "ohos")] block (4行), Window::remove_menu() #[cfg(target_env = "ohos")] block (3行) | ~7 行 |

**Rust 合计**：~97 行

**与原方案差异**：
- muda `refresh_menubar()` 无 `#[cfg(desktop)]` — inherent method, ohrs 编译不需要 cfg(desktop)
- `is_desktop_device()` 加了 `#[cfg(target_env = "ohos")]` gate — ohrs standalone 编译时不产生此函数
- `PopupRequestData` 保留为 type alias (`pub type PopupRequestData = MenuRequestData`)
- `Window::set_menu/remove_menu` 用 `#[cfg(target_env = "ohos")]` 而非 `#[cfg(all(target_env = "ohos", desktop))]` — 在 desktop impl block 内，desktop 已隐含

### 5.2 ArkTS 侧

| 文件 | 改动 | 行数估算 |
|------|------|---------|
| `NativeAbility.ets` | setupMenuPopup 保留名称，内容改为统一 onMenuRequest + isDesktopDevice AppStorage | ~30 行修改 |
| `MainPage.ets` | 新增 @StorageProp/@State (6个), onMenubarJsonChange, MenuBarRow builder, MenubarDropdownContent builder, build() 修改 | ~40 行新增 |
| `menu.ets (PredefinedActionExecutor)` | fullscreen/recover 设置 menubar_visible AppStorage | ~2 行新增 |

**ArkTS 合计**：~79 行

**总计**：~176 行

---

## 六、验证策略

### 6.1 Rust UT 清单

| 测试项 | 文件 | 状态 | 验证内容 |
|--------|------|------|----------|
| `test_menu_channel_popup` | `menu/mod.rs` | ✅ | 发送 MenuRequest(x=Some,y=Some) → recv → x/y 有值 |
| `test_menu_channel_menubar` | `menu/mod.rs` | ✅ | 发送 MenuRequest(x=None,y=None) → recv → x/y 为 None |
| `test_menu_request_data_serde` | `menu/mod.rs` | ✅ | skip_serializing_if 验证 — x/y 有值时序列化包含，为 None 时不含 |
| `test_is_desktop_device` | `app.rs` | ⬚ | 延后 — 需 #[cfg(all(test, target_env = "ohos"))] 环境 |
| `test_menubar_json_submenu_format` | `muda/mod.rs` | ⬚ | 延后 — muda 环境限制 |
| `test_menubar_empty_menu_json` | `muda/mod.rs` | ⬚ | 延后 — muda 环境限制 |

### 6.2 HAP Autotest 清单

| 测试项 | 模块 | 状态 | 验证内容 |
|--------|------|------|----------|
| `Menu.set_menu_no_crash_desktop` | menu | ⬚ | 延后 |
| `Menu.remove_menu_no_crash_desktop` | menu | ⬚ | 延后 |
| `Menu.refresh_menubar_no_crash` | menu | ⬚ | 延后 |
| `AppStorage.is_desktop_flag` | menu | ⬚ | 延后 |

### 6.3 手动测试清单

| 测试项 | 状态 | 验证内容 |
|--------|------|---------|
| `MenuBar.desktop_visible` | ⬚ | desktop 启动 → 窗口顶部显示菜单条 |
| `MenuBar.dropdown_click` | ⬚ | 点击 "File" → 下拉弹出 → 点击 "Open" → emit menu event |
| `MenuBar.submenu_nested` | ⬚ | 嵌套 submenu 正常 |
| `MenuBar.predefined_actions` | ⬚ | Close/Fullscreen/About 行为正确 |
| `MenuBar.remove_hides` | ⬚ | remove_menu → 菜单条消失 |
| `MenuBar.refresh_update` | ⬚ | 修改 text → refresh_menubar → 文本更新 |
| `MenuBar.mobile_no_render` | ⬚ | mobile 模式无菜单条 |
| `MenuBar.check_icon_items` | ⬚ | check/icon 项渲染正确 |
| `MenuBar.fullscreen_hides` | ⬚ | fullscreen → 菜单条隐藏 → recover → 菜单条恢复 |

### 6.4 回归验证（已完成）

| 测试项 | 结果 | 备注 |
|--------|------|------|
| 137 项 auto 测试 | 134 ✅ / 3 ❌ | 失败项是 plugin http/autostart/clipboard 未注册 |
| Popup 菜单正常 | ✅ | TSFN 合未破坏 popup |
| Tray 正常 | ✅ | 21 项 tray 测试全通过 |
| Menu 正常 | ✅ | 66 项 menu 测试全通过 |

### 6.5 审计发现的 Bug 及修复

| Bug | 描述 | 修复 | 文件 |
|-----|------|------|------|
| Bug 1 | `onMenubarJsonChange` 缺少 `prepareIcons()` → menubar 下拉中 icon 项降级为纯文本 | 调用 `prepareIcons(items)` + `collectIconIds` + `cleanupStaleIcons` | `MainPage.ets` |
| Bug 2 | `onMenuJsonChange` (popup) `clear()+release()` 释放所有 PixelMap → 误删 menubar 的图标 | 增量清理：维护 `popupIconIds` + `menubarIconIds`，`cleanupStaleIcons()` 只释放不在 union 中的 PixelMap | `MainPage.ets` |
| Bug 3 | `Window::set_menu()` OHOS block 直接 `to_json()+set_menu_json()` → 首次菜单操作时 `init_menu_event_listener` 和 `collect_check_items` 未调用，check toggle 和 MenuEvent 不工作 | 改为调用 `menu.inner().refresh_menubar()`（包含 init + to_json + set_menu_json） | `window/mod.rs`, `muda/ohos/mod.rs` |

---

## 七、风险与待确认

| 项目 | 风险 | 缓解方案 |
|------|------|---------|
| bindMenu 绑定到 Text | 中 | ArkUI 官方文档确认 `bindMenu(boolean, CustomBuilder, options)` 三参数签名可用。需设备验证下拉位置 |
| cfg(desktop) 跨 crate 传播 | 低 | tauri build.rs 通过 `cargo:rustc-cfg=desktop` 输出，该标志自动传播到所有依赖 crate。已验证 popup 方案中 `#[cfg(target_env = "ohos")]` 在 muda/openharmony-ability 中可用，`#[cfg(desktop)]` 同理 |
| AppStorage 值相同不通知 | 低 | `set_menu_json` 每次发送完整 JSON（含所有项），如果内容不变则 AppStorage 不通知 `@StorageProp`。这在 `refresh_menubar` 时是正确行为（无变化就不重渲染） |
| popup/menubar 合并向后兼容 | 低 | 保留旧 API wrapper（on_popup_request/start_popup_forwarder/popup_request_receiver），过渡期不破坏现有代码 |
| hover 样式 | 中 | Phase 10 用 backgroundColor 动态切换模拟 active，hover 留后续 |
| 折叠屏 | 中 | `is_desktop_device()` 编译时确定（`cfg!(desktop)`），运行时不变。不在 Phase 10 处理 |
| .build() vs .build_callback() | 低 | 继续用 `.build_callback()`（与现有 popup 一致），不切换到 `.build()`。如需后续优化可在验证 `.build()` 可行后切换 |

---

## 八、实施顺序

| 步骤 | 优先级 | 内容 |
|------|--------|------|
| Step 1 | P0 | openharmony-ability: MenuRequest 合并 + on_menu_request + set_menu_json + start_menu_forwarder + is_desktop_device (app.rs, cfg!(desktop)) |
| Step 2 | P0 | muda: Menu::refresh_menubar() #[cfg(desktop)] |
| Step 3 | P0 | tauri: Window::set_menu() OHOS desktop block + Window::remove_menu() OHOS desktop block + Menu::refresh_menubar() |
| Step 4 | P0 | ArkTS: NativeAbility.ets setupMenu 合并 + is_desktop_device AppStorage |
| Step 5 | P0 | ArkTS: MainPage.ets MenuBarRow + dropdown + build() 修改 |
| Step 6 | P1 | Rust UT: 6 个新增测试 |
| Step 7 | P1 | HAP autotest: 4 个新增测试 |
| Step 8 | P2 | 手动测试: 9 项 |
| Step 9 | P2 | fullscreen 沉浸式菜单条隐藏 |
| Step 10 | P3 | hover 高亮样式 |

---

## 九、不做的事

1. ❌ 不改为 Navigation 布局
2. ❌ 不实现菜单条拖拽排序
3. ❌ 不实现菜单条右键菜单
4. ❌ 不实现菜单条 accelerators 显示
5. ❌ 不实现折叠屏动态切换
6. ❌ 不实现 hover 高亮
7. ❌ 不分别创建 popup/menubar forwarder 线程 — 合并为单一线程
8. ❌ 不用 `@StorageLink` 做 menubarJson 双向同步 — 改为 `@StorageProp` 单向
9. ❌ 不切换到 `.build()` 构建 TSFN — 继续用 `.build_callback()`（与 popup 一致）
10. ❌ 不给 openharmony-ability/muda 新增 Cargo feature `desktop` — cfg(desktop) 通过 tauri build.rs 自动传播
11. ❌ 不给 openharmony-ability/muda 新增 build.rs — tauri build.rs 的 cfg 输出自动传播到依赖 crate
12. ❌ 不在 tauri Cargo.toml 新增 `[target.'cfg(all(target_env = "ohos", desktop))']` section — Cargo.toml 解析阶段 build.rs 还没运行，cfg(desktop) 条件可能不生效。desktop/mobile 区分在 Rust 代码层 `#[cfg(desktop)]` 控制
13. ❌ 不在 openharmony-ability/muda 使用 `cfg!(feature = "desktop")` / `#[cfg(feature = "desktop")]` — 直接使用 `cfg!(desktop)` / `#[cfg(desktop)]`（tauri build.rs 自动传播）

---

## 十、审计依据

| 来源 | 验证内容 |
|------|---------|
| 现有代码 (menu/mod.rs) | POPUP_CHANNEL/POPUP_CALLBACK/PopupTsfn 模式；forwarder 线程结构；`.build_callback()` 是正确用法（非 `.build()`） |
| 现有代码 (lib.rs) | `#[cfg(feature = "menu")]` 隔离；导出 on_popup_request/start_popup_forwarder/PopupRequestData |
| 玀有代码 (MainPage.ets) | Stack 布局；bindMenu popup anchor；RenderMenuItems builder；@StorageLink 用于 popup 状态 |
| 现有代码 (NativeAbility.ets) | setupMenuPopup 方法；primaryModule.onPopupRequest 注册 |
| 现有代码 (menu.ets) | MenuManager.popupFromJson + PredefinedActionExecutor；AppStorage 驱动 popup |
| 现有代码 (window/mod.rs) | `#[cfg(desktop)]` gate（line 1197）；set_menu/remove_menu 无 OHOS 分支；run_on_main_thread 闭包无 OHOS block |
| 现有代码 (menu/menu.rs) | OHOS 方法 bypass run_item_main_thread；直接调用 muda |
| 现有代码 (tauri/build.rs) | `TAURI_OHOS_DEVICE_TYPE` → `alias("desktop", !mobile)` → `cargo:rustc-cfg=desktop` 传播到所有依赖 |
| Cargo.toml (openharmony-ability) | `[features]` section 有 default/drag_and_drop/webview/menu；不需要新增 desktop |
| Cargo.toml (muda) | OHOS 依赖 `openharmony-ability = { features = ["menu"] }`；不需要新增 desktop feature |
| Cargo.toml (tauri) | `[target.'cfg(target_env = "ohos")'.dependencies]` 包含 muda/openharmony-ability with menu feature；不需要新增 desktop 条件分支 |
| ArkUI 官方文档 (bindMenu) | `bindMenu(boolean, CustomBuilder, options)` 三参数签名可用；`showInSubWindow`、`onWillDisappear` 选项可用 |
| ArkUI 官方文档 (AppStorage) | `@StorageProp` 单向同步；`@StorageLink` 双向同步；`setOrCreate` 值相同时不通知；不建议用双向同步做消息传递 |
| popup 方案对比 | popup 不依赖 Cargo feature desktop 传播；所有 OHOS 方法 bypass run_on_main_thread；menubar 应保持相同模式 |
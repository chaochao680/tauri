# Menu 模块 OHOS 适配问题修复记录

> 本文档记录 menu 模块 OHOS 适配过程中发现和修复的所有问题，按发现顺序排列。

---

## Fix 1: Autotest 在 ohos desktop 模式下无法运行

**现象**：
- `TAURI_OHOS_DEVICE_TYPE=desktop` 编译 api-demo 后，Autotest 等待超时
- 设备端未生成 `test-report.json`
- 应用进程存活，无 panic、无 crash，但无任何 tauri/webview 日志输出
- hilog 显示 ArkTS 侧 `LoadContentByName` 正常调用，ACE 框架初始化完成，但 webview 从未创建

**根因**：`#[cfg_attr(mobile, tauri::mobile_entry_point)]` 在 ohos desktop 模式下不生效

`build.rs` 中当 `TAURI_OHOS_DEVICE_TYPE=desktop` 时：
- `mobile = false`，`desktop = true`
- `#[cfg_attr(mobile, tauri::mobile_entry_point)]` 条件不满足
- `tauri::mobile_entry_point` 宏不被应用
- NAPI 入口点（`init`、`render`、`registerCustomProtocol`）未注册
- ArkTS 侧 `Loadable.load("api_lib")` 加载 so 后找不到 `init`/`render` 函数
- `MainPage.aboutToAppear()` 静默失败，`primaryModuleName` 未设置
- `DefaultXComponent` 永远不被创建 → webview 不加载 → 测试不执行

**修复**：
```rust
// 修复前
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() { ... }

// 修复后
#[cfg_attr(any(mobile, target_env = "ohos"), tauri::mobile_entry_point)]
pub fn run() { ... }
```

**文件**：`examples/api/src-tauri/src/lib.rs`

**状态**：✅ 已解决。修复后 autotest 恢复正常：36 passed, 5 failed, 0 skipped。

---

## Fix 2: Menu 测试大量超时导致应用卡死

**现象**：
- Autotest 跑完非 menu 测试后，进入 menu 测试阶段，大量用例超时（timeout 2.5s）
- 测试报告只写到第43条（`Menu.with_id`），后续 menu 测试没有写入报告
- 应用界面 freeze，无法交互
- 最终结果：38 passed, 69 failed（大部分是 menu 相关）

**根因**：`run_item_main_thread!` 和 `run_main_thread!` 宏在 OHOS 上导致死锁

宏通过 `run_on_main_thread(task)` + `rx.recv()` 阻塞等待任务完成。OHOS 上：
1. IPC 回调在 ArkWeb JS 引擎工作线程派发（非主线程）
2. `run_on_main_thread` → `send_user_message` → `proxy.send_event()` 发送到 tao 事件循环 mpsc channel
3. `rx.recv()` 阻塞 IPC 回调线程，等待 task 执行完毕
4. `OpenHarmonyWaker::wake()` 未能正确触发事件循环唤醒
5. 事件循环未消费 `MainEvent::UserEvent` → task 永远不执行 → 永久阻塞

muda OHOS 实现是**纯数据结构**（`Vec::new()`, `Vec::push()`, 字段读写），本身就不需要主线程调度。

**修复**：采用方法级别绕过（方案 B），在 tauri 的 menu 类型方法内部加 `#[cfg(target_env = "ohos")]` 分支，直接调用 muda 方法：

```rust
pub fn new<M: Manager<R>>(manager: &M) -> crate::Result<Self> {
    let handle = manager.app_handle();
    #[cfg(target_env = "ohos")]
    let menu = muda::Menu::new();  // 直接执行
    #[cfg(not(target_env = "ohos"))]
    let menu = run_main_thread!(handle, || muda::Menu::new())?;
    Ok(Self(Arc::new(menu)))
}
```

**修改文件**：
- `crates/tauri/src/menu/menu.rs`
- `crates/tauri/src/menu/submenu.rs`
- `crates/tauri/src/menu/normal.rs`
- `crates/tauri/src/menu/check.rs`
- `crates/tauri/src/menu/icon.rs`
- `crates/tauri/src/menu/predefined.rs`

**状态**：✅ 已解决。Menu 42 项测试全部通过，零超时、零死锁。Tray 和 Global Shortcut 未受影响。

---

## Fix 3: setupMenuPopup 调用时序错误

**现象**：
```
[Menu] setupMenuPopup: entered
[Menu] setupMenuPopup: getUIContext failed: Error: This window state is abnormal.
[Menu] setupMenuPopup: uiContext is null
```
`POPUP_CALLBACK` 永远是 None，popup 不弹出。

**根因**：`NativeAbility.onWindowStageCreate` 中，`setupMenuPopup(windowStage)` 在 `loadContentByName` **之前**调用。`windowStage.getMainWindowSync().getUIContext()` 依赖 UIContent 已初始化，而 UIContent 是在 `loadContentByName` 完成后才创建的。因此 `getUIContext()` 抛异常，`setupMenuPopup` early return，`onPopupRequest` 从未被调用。

**修复**：将 `setupMenuPopup` 移到 `loadContentByName` 之后：
```typescript
// 修复前：
setupMenuPopup(windowStage);  // UIContent 未初始化 → getUIContext() 失败
await windowStage.loadContentByName(Entry.RouteName);

// 修复后：
await windowStage.loadContentByName(Entry.RouteName);  // 先加载内容
setupMenuPopup(windowStage);  // UIContent 已就绪 → getUIContext() 成功
```

**文件**：`openharmony-ability/native_ability/src/main/ets/ability/NativeAbility.ets`

**状态**：✅ 已解决。

---

## Fix 4: ThreadsafeFunction downcast 类型不匹配

**现象**：
```
[Menu] on_popup_request called from ArkTS
[Menu] on_popup_request: callback registered successfully
...
[Menu] forwarder received request, json_len=77
[Menu] forwarder: downcast_ref failed
```

**根因**：`POPUP_CALLBACK` 使用 `Box<dyn Any + Send + Sync>` 存储 ThreadsafeFunction，取出时通过 `downcast_ref` 还原类型。但 napi-ohos 的 `ThreadsafeFunction` 有 7 个泛型参数，存入和取出时的类型不匹配：

- 存入时第3参数（`CallJsBackArgs`）为 `Vec<PopupRequestData>`
- 取出时默认展开为 `PopupRequestData`（第3参数默认=T）
- 类型不匹配 → `downcast_ref` 返回 None

**修复**：去掉 `Box<dyn Any>` + downcast 模式，改为直接存储具体类型：
```rust
type PopupTsfn = ThreadsafeFunction<PopupRequestData, Unknown<'static>, FnArgs<(PopupRequestData,)>, Status, false>;
static POPUP_CALLBACK: Mutex<Option<PopupTsfn>> = Mutex::new(None);

let tsfn: PopupTsfn = callback
    .build_threadsafe_function::<PopupRequestData>()
    .callee_handled::<false>()
    .build_callback(|ctx| Ok(FnArgs { data: (ctx.value,) }))?;
*POPUP_CALLBACK.lock()? = Some(tsfn);
```

同时使用 `callee_handled::<false>()` 使 JS 回调签名为 `(data) => void`，与 ArkTS 侧一致。

**文件**：`openharmony-ability/crates/ability/src/menu/mod.rs`

**状态**：✅ 已解决。

---

## Fix 5: Menu.popup() 不生效 — bindMenu 重构 + 全类型 Item 支持

**现象**：
- Rust 侧日志显示完整流程执行，但 `POPUP_CALLBACK is None` 永远是 None
- ArkTS 侧 `setupMenuPopup` 的 `console.error` 日志完全没有出现
- 早期方案中 `openMenu + ComponentContent`（模块级 @Builder，无 this）导致 crash

**根因（两层）**：

1. **两个 .so 的静态变量隔离**：`libapi_lib.so` 和 `libnative_ability.so` 各自链接了 `openharmony-ability` crate，拥有独立的 `POPUP_CALLBACK` 静态变量副本。ArkTS 调用 `libnative_ability.so` 的 `onPopupRequest` 设置的是该 so 的副本，而 `start_popup_forwarder()` 检查的是 `libapi_lib.so` 的副本。

2. **ComponentContent 方案 crash**：模块级 `@Builder` 无 `this` 上下文，无法递归渲染子菜单，且 ArkUI 框架对动态 ComponentContent 的支持存在限制。

**修复**：将 popup menu 从 `openMenu + ComponentContent` 重构为 `bindMenu(isShow)` + `@Component @Builder` 方法：

1. **AppStorage 桥接**：TauriMenuManager（非 UI 类）通过 `AppStorage.setOrCreate` 驱动 `@StorageLink` 响应式更新
2. **递归子菜单**：`MenuItem({ builder: (): void => this.SubmenuContent(items) })` — @Component 内 this 有效
3. **全类型支持**：normal/check/icon/predefined/submenu/separator 在 ForEach 中按 type 分支渲染
4. **Separator**：空 `MenuItemGroup() {}` 利用 group 边界自动产生分隔线
5. **位置控制**：`anchorPosition: {x, y}` 相对于 Column(0,0)，等效窗口绝对坐标

**修改文件**：
| 文件 | 修改内容 |
|------|---------|
| `openharmony-ability/.../components/MainPage.ets` | bindMenu + @Builder MenuContent/SubmenuContent 递归渲染全类型 |
| `openharmony-ability/.../helper/menu.ets` | TauriMenuManager 改用 AppStorage；删除 openMenu/ComponentContent |
| `openharmony-ability/.../ability/NativeAbility.ets` | setupMenuPopup 中 executor.setWindow() |
| `muda/src/platform_impl/ohos/mod.rs` | accelerator=None + 删除死代码 |

**状态**：✅ 已解决。128 tests: 120 passed, 8 failed。Menu 66 项全部通过，Tray 21 项全部通过，零回归。

---

## Fix 6: openMenu + ComponentContent 渲染子菜单 crash

**现象**：
- 点击 Popup 按钮后，ArkTS 侧 crash：`TypeError: Cannot read property observeComponentCreation2 of undefined`
- 子菜单无法渲染，应用崩溃

**根因**：`openMenu + ComponentContent` 方案使用**模块级 `@Builder`**，但模块级 `@Builder` 没有 `this` 上下文。当 `MenuItem.builder` 需要调用另一个 `@Builder`（渲染子菜单）时，ArkUI 框架无法注入渲染上下文，直接 crash。

**修复**：改用 `bindMenu(isShow, builder, options)` + `@Component @Builder` 方法：

| 对比 | openMenu + ComponentContent | bindMenu(isShow) |
|------|---------------------------|------------------|
| @Builder 上下文 | 模块级，无 this | @Component 内，this 有效 |
| 子菜单支持 | ❌ crash | ✅ `builder: this.SubmenuContent(items)` 递归 |
| 触发语义 | 系统级右键菜单 | 程序控制显隐 |
| 位置控制 | 有限 | `anchorPosition: {x, y}` 精确定位 |
| 子窗口 | 不可控 | `showInSubWindow: true` 可配置 |
| API 版本 | — | API 11+ |

**关键设计**：
- `@Component` 内的 `@Builder` 方法有 `this`，支持递归：`builder: (): void => this.SubmenuContent(items)`
- `AppStorage` 桥接 Rust 和 ArkTS：`popupFromJson()` 设置数据 → `@StorageLink` 响应 → `bindMenu` 弹出
- `anchorPosition: {x, y}` 相对于绑定组件左上角，等效窗口绝对坐标
- `onWillDisappear` 重置 `menuShown = false`，确保下次可再触发

**修改文件**：
- `openharmony-ability/.../components/MainPage.ets` — bindMenu + @Builder MenuContent/SubmenuContent
- `openharmony-ability/.../helper/menu.ets` — TauriMenuManager 改用 AppStorage，删除 openMenu/ComponentContent

**状态**：✅ 已解决。子菜单 hover 展开正常，递归嵌套正常，全类型 item 渲染正确。

---

## Fix 7: is_menu_visible=true after Remove Menu; Restore doesn't re-show

### 根因

Two sub-problems:

**Sub-A: `is_menu_visible` returns true after Remove Menu**

`remove_menu()` in `window/mod.rs` on OHOS only calls `set_menu_json("[]", window_id)` — it does NOT call `set_menubar_visible(false, window_id)`.

Meanwhile, `is_menubar_visible(window_id)` returns `map.get(window_id).copied().unwrap_or(true)` — the default for missing keys is `true`.

So after Remove:
- `menubarVisible` in AppStorage stays true (because no `visible=false` was pushed)
- `menubarItems` becomes empty (because "[]" was pushed)
- Rendering condition `isDesktop && menubarItems.length > 0 && menubarVisible` → items=0 → bar disappears
- But `is_menu_visible()` → default=true → reports visible

**Sub-B: Restore Default Menu doesn't re-show bar**

Previously broken because `PredefinedMenuItem.new({ item: 'About', text: '...' })` triggered a serde deserialization error. After fix, Restore should work.

However, there's still a **semantic bug**: `remove_menu()` should make `is_menu_visible` return false, and `set_menu()` should make it return true.

### 修复

**File: `tauri/crates/tauri/src/window/mod.rs`**

In `remove_menu()` OHOS branch, add `set_menubar_visible(false)`:
```rust
#[cfg(target_env = "ohos")]
if let Some(_menu) = &prev_menu {
  openharmony_ability::menu::set_menubar_visible(false, self.label().to_string()).ok();
  openharmony_ability::menu::set_menu_json("[]".to_string(), self.label().to_string()).ok();
}
```

In `set_menu()` OHOS branch, add `set_menubar_visible(true)`:
```rust
#[cfg(target_env = "ohos")]
{
  menu.inner().refresh_menubar(self.label()).ok();
  openharmony_ability::menu::set_menubar_visible(true, self.label().to_string()).ok();
}
```

**状态**：✅ Deployed to device, verified working.

---

## Fix 8: Auto Refresh Text still shows "Original" after setText("Updated!")

### 根因

`MenubarDropdownContent` builder 中直接使用 ForEach 闭包捕获的 `item.submenuItems`，但 `item` 来自 ForEach 的旧状态。当 `onMenubarJsonChange` 更新 `menubarItems` 后，ForEach 闭包中的 `item` 仍是旧引用，`item.submenuItems` 为旧数据。

### 修复

`MenubarDropdownContent` 改为使用 `this.getActiveDropdownItem()?.submenuItems`，从最新 `menubarItems` 状态中查找当前下拉项。

同时，`onMenubarJsonChange` 中如果下拉菜单已打开，先关闭再重新打开，强制 re-render：
```typescript
if (this.activeDropdownShown && this.activeDropdownId) {
  this.activeDropdownShown = false;
  setTimeout(() => { this.activeDropdownShown = true; }, 0);
}
```

**状态**：✅ 已部署验证，Auto Refresh Text 正常工作。

---

## Fix 9: Dark Mode colors not adapting

### 状态: ⚠️ 暂不支持

**已尝试的方案 (均无效)**:
1. `this.context?.config?.colorMode` — OHOS desktop 返回 -1 (NOT_SET)
2. `this.context?.resourceManager?.getConfigurationSync().colorMode` — 静默失败
3. `onConfigurationUpdate(newConfig)` — desktop 不触发
4. `onForeground()` 中重新获取 — 同样无效

**结论**: OHOS desktop 设备目前不传播系统 colorMode 变化。

**决策**: 暂不支持 dark theme，menubar 颜色固定为浅色模式 (hardcoded light colors)。已移除所有 colorMode 相关代码:
- NativeAbility.ets: 移除 colorMode AppStorage 初始化和 onConfigurationUpdate colorMode 更新
- MainPage.ets: 移除 `@StorageProp colorMode`、`isDark()` 及所有颜色 helper 方法，menubar 颜色改为硬编码 (`#333333`/`#999999`/`#F5F5F5`/`#EBEBEB`/`#E0E0E0`)

---

## Fix 10: Accelerator Ctrl+O keyboard shortcut doesn't fire

### 根因

已实现: MainPage.ets 使用 Column 级 `onKeyPreIme` + `AcceleratorMatcher` 匹配快捷键。在 OHOS desktop 设备上验证确认可以正常工作。

**状态**：✅ 已验证工作正常。

---

## Fix 11: Fullscreen menubar disappears, no auto-recover on exit

### 根因

`execute('fullscreen')` 设置 `menubar_visible = false` + `ENTER_IMMERSIVE` 全屏。退出全屏时需要恢复 menubar。

### 修复

**Fix 11A: windowRectChange RECOVER 检测** (核心，已验证工作)
```typescript
win.on("windowRectChange", (options: window.RectChangeOptions) => {
  if (!AppStorage.get<boolean>("__openharmony_ability_menubar_visible__::main") &&
      options.reason === window.RectChangeReason.RECOVER) {
    AppStorage.setOrCreate("__openharmony_ability_menubar_visible__::main", true);
  }
  this.forEachLifecycle(...);
});
```

**Fix 11B: fullscreen 单向进入 + recover 退出** (menu.ets)
```typescript
case 'fullscreen':
  AppStorage.setOrCreate("menubar_visible::main", false);
  AppStorage.setOrCreate("menu_shown::main", false);
  notifyMenubarVisibilityFn?.("main", false);
  await win?.maximize(ENTER_IMMERSIVE);
  break;
case 'recover':
  AppStorage.setOrCreate("menubar_visible::main", true);
  notifyMenubarVisibilityFn?.("main", true);
  await win?.recover();
  break;
```

**Fix 11C: Escape 键退出全屏** (MainPage.ets)
```typescript
if (event.keyCode === KeyCode.KEYCODE_ESCAPE && !this.menubarVisible) {
  AppStorage.setOrCreate("menubar_visible::main", true);
  globalRecoverFn?.();
  return true;
}
```

**已移除的冗余逻辑** (精简):
- avoidAreaChange 中的 menubar 恢复检测 (冗余，RECOVER 已覆盖)
- display fallback 尺寸检测 (硬编码 3120/2080，不可靠)
- `isFullscreen` 字段 (无实际用途，也是 Fix 17 的根因)
- `executor.setRecoverFn` (冗余，recover 动作直接用 win.recover())
- 所有 hilog 调试日志
- colorMode 相关逻辑 (dark theme 暂不支持)

**状态**：✅ 已部署验证，全屏退出后 menubar 自动恢复。

---

## Fix 12: Menubar 启动时不显示 (TSFN 时序问题)

**现象**: 应用启动时 menubar 不显示，必须手动点击 "restore menu bar" 才出现。

**根因**: TSFN 时序竞争。`set_menu()` 在 Rust `.setup()` 阶段调用，同时发送两条消息到 `MENU_CHANNEL`:
1. `refresh_menubar("main")` → `set_menu_json(json, "main")` → menubar JSON 数据
2. `set_menubar_visible(true, "main")` → visibility=true

但是 `onMenuRequest` TSFN callback (`MENU_CALLBACK`) 是在 ArkTS 的 `onWindowStageCreate` → `setupMenuPopup` → `primaryModule.onMenuRequest(callback)` 中才注册。

转发线程 (`start_menu_forwarder`) 在 menu plugin `init()` 时就已启动 (早于 `.setup()`)，收到消息后发现 `MENU_CALLBACK` 为 None，两条消息都被丢弃。

`@StorageProp("__openharmony_ability_menubar_json__::main")` 保持初始值 `"[]"`，`menubarItems` 为空数组，渲染条件 `isDesktop && menubarItems.length > 0 && menubarVisible` 的 `items=0` 分支不满足 → menubar 不渲染。

**涉及 commit**: `dd6d3fe` (添加 `window_id::main` suffix 分发) — 之前使用 `__openharmony_ability_menubar_json__` 无 suffix 时，`NativeAbility.onWindowStageCreate` 中通过 `AppStorage.setOrCreate` 直接设置 menubar JSON，不依赖 TSFN。改为 TSFN 分发后，初始 menubar JSON 必须通过 TSFN 传递，但 TSFN 在 `.setup()` 时尚未注册。

**修复方案**: 在 `setupMenuPopup` 注册 TSFN callback 后，触发一次初始 menubar refresh。或改为 `loadContentByName` 在 `setupMenuPopup` 之后执行。

**文件**:
- `openharmony-ability/.../NativeAbility.ets` (时序: loadContentByName 在 setupMenuPopup 之前)
- `tauri/.../window/mod.rs` (set_menu 调用时机)
- `openharmony-ability/.../menu/mod.rs` (MENU_CALLBACK 注册时序)

**状态**：🔲 待修复

---

## Fix 13: Ctrl+V 失效 (AcceleratorMatcher 消费 keyboard event)

**现象**: 点击 "restore menu bar" 后 Ctrl+V 失效。Webview 内无法粘贴。

**根因**: 两个叠加问题:

**问题 A: AcceleratorMatcher 消费 Ctrl+V 阻止 webview 接收**

`onKeyPreIme` + `AcceleratorMatcher` 匹配到 Ctrl+V → Paste MenuItemData，返回 `true` 消费 key event，阻止 webview 收到 Ctrl+V。路由到 `PredefinedActionExecutor.execute('paste')` → `controller?.runJavaScript('document.execCommand("paste")")`。

`document.execCommand("paste")` 在 OHOS webview 上不会触发真正的系统剪贴板粘贴（浏览器安全限制要求 user activation），paste 操作无效。结果: Ctrl+V 被拦截但 paste 不生效。

只在 menubar JSON 设置后出现问题，因为 `acceleratorMatcher.buildFromItems(items)` 只在 `onMenubarJsonChange` 触发时注册加速键。

**问题 B: PredefinedActionExecutor.controller 未设置 (TSFN 时序)**

`setPrimaryWebviewControllerCallback` 在 `setupMenuPopup` (line 171-173) 中注册，但 `loadContentByName` (line 246) 在 `setupMenuPopup` (line 251) **之前**运行。当 DefaultXComponent 的 `aboutToAppear` → `addWebview` 触发时，`primaryWebviewControllerCallback` 还是 null。

回调被设置一次后立即置为 null (`primaryWebviewControllerCallback = null`, line 409)。由于 webview 在 `setupMenuPopup` 之前就创建了，回调从未触发，`PredefinedActionExecutor.controller` 永远是 null。

**涉及 commit**: `dd6d3fe` (添加 AcceleratorMatcher 和 setPrimaryWebviewControllerCallback)

**修复方案**:
- 问题 A: AcceleratorMatcher 不应拦截 clipboard 操作 (paste/copy/cut/selectAll/undo/redo) 的快捷键，让 webview 自己处理
- 问题 B: 将 `loadContentByName` 移到 `setupMenuPopup` 之后，或改为在 `setupMenuPopup` 中直接获取 webviewController

**文件**:
- `openharmony-ability/.../MainPage.ets` (onKeyPreIme + AcceleratorMatcher)
- `openharmony-ability/.../NativeAbility.ets` (loadContentByName 在 setupMenuPopup 之前)
- `openharmony-ability/.../DefaultXComponent.ets` (primaryWebviewControllerCallback 时序)
- `openharmony-ability/.../helper/accelerator_matcher.ets` (clipboard 快捷键匹配)

**状态**：🔲 待修复

---

## Fix 14: set_menu() OHOS 分支未存储 menu 引用 → auto_refresh_menubar 物理不工作

**现象**: 所有动态更新操作（setText、setEnabled、setAccelerator、insert、remove）在 OHOS 上不生效。menubar 显示初始内容，但后续修改不被推送。

**根因**: `window/mod.rs` 中 `set_menu()` 的 OHOS 分支只调用 `refresh_menubar()` + `set_menubar_visible(true)`，但**没有调用 `self.menu_lock().store(menu.clone())`** 存储菜单引用。

而 `auto_refresh_menubar` 的刷新流程为：
1. Rust 侧修改 menu 数据结构（setText 等操作修改本地 muda Menu 对象）
2. 调用 `auto_refresh_menubar(window_id)` → `window.menu_lock().as_ref()` 获取 Menu 引用
3. Menu 引用 → `menu.inner().refresh_menubar(window_id)` → ArkTS 更新 menubar JSON

由于 `menu_lock()` 为 None（从未 store），步骤 2 返回 None，`auto_refresh_menubar` 无法执行，所有动态更新被静默丢弃。

Fix 8 (auto refresh text) 实际上就是 Fix 14 的表现，之前误判为 "ForEach stale data" 问题。真正根因是 `menu_lock()` 为空导致 `auto_refresh_menubar` 无法执行。

**修复方案**:
```rust
// window/mod.rs — set_menu() OHOS 分支
#[cfg(target_env = "ohos")]
{
  menu.inner().refresh_menubar(self.label()).ok();
  self.menu_lock().store(menu.clone());  // ← 新增: 存储 menu 引用
  openharmony_ability::menu::set_menubar_visible(true, self.label().to_string()).ok();
}

// window/mod.rs — remove_menu() OHOS 分支
#[cfg(target_env = "ohos")]
if let Some(_menu) = &prev_menu {
  openharmony_ability::menu::set_menubar_visible(false, self.label().to_string()).ok();
  openharmony_ability::menu::set_menu_json("[]".to_string(), self.label().to_string()).ok();
  self.menu_lock().take();  // ← 新增: 清除 menu 引用
}
```

**状态**：🔲 待修复（最高优先级）

---

## Fix 15: onKeyPreIme 焦点链限制 — Row/Column 无法在 WebView 持有焦点时拦截按键

**现象**: 当 XComponent/WebView 拥有键盘焦点时，Row 和 Column 上的 `onKeyPreIme` 事件处理器永远不会触发。只有当焦点在 menubar 区域时才工作。

**根因**: OHOS key event dispatch 机制：
1. `onKeyPreIme` 只在**当前焦点节点**上触发（叶子节点优先）
2. 当 WebView/XComponent 持有焦点时，key event 直接路由到该节点
3. Row 和 Column 即使设置了 `HitTestMode.Transparent`，也无法获得 `onKeyPreIme` 事件
4. ArkUI 的 `onKeyPreIme` 不像 Android 那样在 View 层级中向上传播

**修复方案**: 使用 `UIContext.onKeyPreIme`（API 12+）替代 Row/Column 级 `onKeyPreIme`：
```typescript
// NativeAbility.ets — setupMenuPopup 中
uiContext.onKeyPreIme((event: KeyEvent) => {
  if (acceleratorMatcher?.match(event)) {
    return true;  // 消费匹配的快捷键
  }
  if (event.keyCode === KeyCode.KEYCODE_ESCAPE && !menubarVisible) {
    // Escape 退出全屏
    return true;
  }
  return false;  // 不消费，让 webview 处理
});
```

`UIContext.onKeyPreIme` 是**窗口级拦截**，不受焦点链限制，在任何组件持有焦点时都能拦截。

**注意**: 需验证 OHOS desktop SDK 是否支持 `UIContext.onKeyPreIme`（API 12+）。

**状态**：🔲 待修复

---

## Fix 16: maximize(ENTER_IMMERSIVE) 触发 windowStatusChange MAXIMIZE → listener 误判为恢复信号

**现象**: 点击 fullscreen 菜单项进入沉浸式全屏后，menubar 立刻重新出现（1-2帧内），窗口并未真正进入全屏状态。

**根因**: `maximize(window.MaximizeType.ENTER_IMMERSIVE)` 触发 `windowStatusChange` 事件，状态类型为 `windowStatusType.MAXIMIZE`（值为 2），而非 `windowStatusType.FULL_SCREEN`（值为 1）。

`windowStatusType` 枚举值：
| 值 | 枚举 | 含义 |
|----|------|------|
| 0 | UNDEFINED | 未定义 |
| 1 | FULL_SCREEN | 全屏 |
| 2 | MAXIMIZE | 最大化 |
| 3 | MINIMIZE | 最小化 |
| 4 | FLOATING | 浮动 |

如果 `windowStatusChange` listener 使用 `windowStatusType !== FLOATING` 作为恢复条件（即 MAXIMIZE 也被视为恢复），则在 `ENTER_IMMERSIVE` 触发后：
1. `execute('fullscreen')` → `menubar_visible=false` + `maximize(ENTER_IMMERSIVE)`
2. `maximize(ENTER_IMMERSIVE)` → `windowStatusChange(MAXIMIZE)` 触发
3. listener 判断 `MAXIMIZE !== FLOATING` → 误判为恢复 → `menubar_visible=true`
4. menubar 立刻恢复，全屏状态被破坏

**修复方案**:
- 方案 A: 删除 `windowStatusChange` listener，只依赖 `windowRectChange` RECOVER 检测（已验证可靠）
- 方案 B: 在 `windowStatusChange` listener 中区分 `MAXIMIZE` 来自 `ENTER_IMMERSIVE` 还是正常恢复（需要额外状态标记）
- 方案 C: fullscreen 操作改为 `setFullScreen(true)` 而非 `maximize(ENTER_IMMERSIVE)`，触发 `FULL_SCREEN` 状态而非 `MAXIMIZE`

**状态**：🔲 待修复

---

## Fix 17: isFullscreen toggle 状态被 Fix 16 破坏 → 点击全屏在全屏中退出

**现象**: 如果使用了 `isFullscreen` toggle 设计（点击 fullscreen 在全屏状态时退出全屏），Fix 16 的 `windowStatusChange` listener 在进入全屏后立刻恢复 `isFullscreen=false`，导致第二次点击 fullscreen 被误判为"进入全屏"而非"退出全屏"。

**根因**: Fix 16 导致 `isFullscreen` 状态被错误重置。`isFullscreen` toggle 设计本身就存在竞态条件：
1. `execute('fullscreen')` → `isFullscreen=true` + `maximize(ENTER_IMMERSIVE)`
2. `windowStatusChange(MAXIMIZE)` → listener 重置 `isFullscreen=false`
3. 用户再次点击 fullscreen → `isFullscreen` 为 false → 误判为"进入全屏"

**修复方案**: fullscreen 操作应为**单向进入**，退出全屏通过：
- Escape 键（已有 Fix 11C）
- `execute('recover')` 菜单项
- 系统手势/窗口恢复

不使用 toggle 设计。同时删除 `isFullscreen` 状态字段（Fix 11 精简时已移除，确认无回归）。

**状态**：🔲 待修复（与 Fix 16 同修）

---

## Fix 18: colorMode -1→1 normalization 可能不生效

**现象**: Fix 9 添加了 colorMode normalization（-1→1，将 NOT_SET 默认为浅色模式），但 dark theme 在设备上仍然不工作。

**可能根因（需设备日志验证）**:
1. **HAR 未包含修复**: 如果 HAR 包未重新构建/安装，设备上运行的仍是旧版本代码
2. **onConfigurationUpdate 不触发**: OHOS desktop 在切换系统暗色模式时不触发此回调
3. **@StorageProp 不触发 re-render**: 即使 AppStorage 中 colorMode 值改变，`@StorageProp` 绑定的组件可能不重新渲染
4. **colorMode 检测本身无效**: OHOS desktop 不通过 Configuration/ResourceManager 传播 colorMode

**修复方案**: 在 NativeAbility.ets 的 colorMode normalization 路径添加 console.log：
```typescript
// 初始化
console.log(`[Menu] colorMode init: raw=${rawMode}, normalized=${normalizedMode}`);

// onConfigurationUpdate
console.log(`[Menu] onConfigurationUpdate: newColorMode=${newConfig.colorMode}`);
```

重建 HAR → ohpm → cargo → HAP → sign → install，验证设备日志确认 normalization 是否执行。

**优先级**: 低 — dark theme 暂不支持是已确认的结论 (Fix 9)，此验证仅为排除 normalization 代码本身的 bug。

**状态**：🔲 待验证

---

## Summary

| Fix | 问题 | 严重程度 | 状态 |
|-----|------|---------|------|
| 1 | NAPI 入口点未注册 (desktop 模式) | 致命 | ✅ 已解决 |
| 2 | run_main_thread! 宏死锁 | 致命 | ✅ 已解决 |
| 3 | setupMenuPopup 时序错误 | 高 | ✅ 已解决 |
| 4 | ThreadsafeFunction downcast 类型不匹配 | 高 | ✅ 已解决 |
| 5 | bindMenu 重构 + 全类型支持 | 高 | ✅ 已解决 |
| 6 | openMenu → bindMenu 重构决策 | 高 | ✅ 已解决 |
| 7 | remove_menu visibility 缺失 | 高 | ✅ 已解决 |
| 8 | Auto Refresh Text stale data | 中 | ✅ 已解决 (实际是 Fix 14 的表现) |
| 9 | Dark Mode 不适配 | 中 | ⚠️ 暂不支持 |
| 10 | Accelerator 快捷键 | 中 | ✅ 已验证工作 |
| 11 | Fullscreen menubar 恢复 | 高 | ✅ 已解决 |
| 12 | Menubar 启动不显示 (TSFN 时序) | 高 | 🔲 待修复 |
| 13 | Ctrl+V 失效 | 高 | 🔲 待修复 |
| 14 | menu_lock() 未 store → auto_refresh 不工作 | 最高 | 🔲 待修复 |
| 15 | onKeyPreIme 焦点链限制 | 高 | 🔲 待修复 |
| 16 | windowStatusChange MAXIMIZE 误判 | 高 | 🔲 待修复 |
| 17 | isFullscreen toggle 竞态 | 高 | 🔲 待修复 (与 Fix 16 同修) |
| 18 | colorMode normalization 验证 | 低 | 🔲 待验证 |

---

## Additional Findings

1. **OHOS desktop colorMode 不传播**: `Configuration.colorMode` 返回 -1, `resourceManager.getConfigurationSync().colorMode` 静默失败, `onConfigurationUpdate` 不触发。Dark theme 暂不支持。

2. **windowRectChange RECOVER 是可靠的退出全屏检测**: hilog 确认退出全屏时 `reason=RECOVER` 正确触发，一行条件判断即可恢复 menubar，无需复杂 fallback。

3. **maximize(ENTER_IMMERSIVE) 触发 MAXIMIZE 而非 FULL_SCREEN**: 这是 Fix 16 的核心问题。`windowStatusType.MAXIMIZE` 和正常恢复的信号相同，listener 无法区分。

4. **onKeyPreIme 只在焦点节点触发**: OHOS 的 `onKeyPreIme` 不像 Android 那样在 View 层级中向上传播。需要 `UIContext.onKeyPreIme`（API 12+）实现窗口级拦截。

5. **menu_lock() 为 None 导致 auto_refresh_menubar 不工作**: 这是 Fix 14 的核心。`set_menu()` OHOS 分支跳过了 `menu_lock().store()`，导致所有后续动态更新被静默丢弃。Fix 8 的 "ForEach stale data" 实际是此问题的表现。

6. **TSFN 必须使用 `callee_handled::<false>()` + `FnArgs<>` wrapper**: napi-ohos ThreadsafeFunction 的 CallJsArgs 类型必须与存入取出一致，否则 downcast_ref 失败。

7. **Function::call() 在 render() NAPI context 中静默失败**: ArkWeb IPC 回调在 JS 引擎工作线程派发，render() context 下 JS 调用不生效。

8. **NAPI napi-derive auto converts snake_case → camelCase**: `emit_menu_event` → `emitMenuEvent`，函数命名必须遵循此规则。

9. **两个 .so 的静态变量隔离**: `libapi_lib.so` 和 `libnative_ability.so` 各有独立的 POPUP_CALLBACK 副本，ArkTS 设置的是 native_ability 的副本，forwarder 检查的是 api_lib 的副本。

---

## Files Modified (汇总)

| File | Fixes |
|------|-------|
| `examples/api/src-tauri/src/lib.rs` | Fix 1: mobile_entry_point |
| `crates/tauri/src/menu/*.rs` (6 files) | Fix 2: bypass run_main_thread! |
| `openharmony-ability/.../NativeAbility.ets` | Fix 3: 时序修正; Fix 9: colorMode norm; Fix 11: 精简; Fix 15: UIContext.onKeyPreIme (待) |
| `openharmony-ability/.../menu/mod.rs` | Fix 4: TSFN type fix |
| `openharmony-ability/.../components/MainPage.ets` | Fix 5/6: bindMenu; Fix 8: getActiveDropdownItem; Fix 11C: Escape; Fix 10: AcceleratorMatcher; Fix 15: UIContext.onKeyPreIme (待) |
| `openharmony-ability/.../helper/menu.ets` | Fix 5: AppStorage 桥接; Fix 11B: fullscreen/recover; Fix 16: 删除 windowStatusChange (待) |
| `openharmony-ability/.../DefaultXComponent.ets` | Fix 10: acceleratorKeyHandler |
| `tauri/.../window/mod.rs` | Fix 7: set_menubar_visible; Fix 14: menu_lock().store/take (待) |
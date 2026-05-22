# Menu 模块 OHOS 适配问题修复记录

> 本文档记录 menu 模块 OHOS 适配过程中发现和修复的所有问题，按修复顺序排列。

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

## 总结

| Fix | 问题类型 | 严重程度 | 状态 |
|-----|---------|---------|------|
| 1 | NAPI 入口点未注册 | 致命 | ✅ 已解决 |
| 2 | run_main_thread! 宏死锁 | 致命 | ✅ 已解决 |
| 3 | setupMenuPopup 时序错误 | 高 | ✅ 已解决 |
| 4 | ThreadsafeFunction downcast 类型不匹配 | 高 | ✅ 已解决 |
| 5 | bindMenu 重构 + 全类型支持 | 高 | ✅ 已解决 |
| 6 | openMenu → bindMenu 重构决策 | 高 | ✅ 已解决 |

**编译验证**：`cargo check --target aarch64-unknown-linux-ohos` 通过

**测试验证**：128 tests: 120 passed, 8 failed（预期：插件未加载 + Channel 吞吐）

## Context

当前 Tauri OHOS 适配中，窗口装饰（decorations）控制完全缺失：

**已有管道（部分存在）：**
```
Tauri config → tauri-runtime-wry → tao WindowAttributes.decorations → platform_impl/ohos/mod.rs → ❌ 丢弃
```

**断点位置：**
1. `tao/src/platform_impl/ohos/mod.rs:779`：`set_decorations()` 是空操作 `pub fn set_decorations(&self, _decorations: bool) {}`
2. `tao/src/platform_impl/ohos/mod.rs:783`：`is_decorated()` 始终返回 `true`
3. `Window` 结构体无 `decorations` 字段，`Window::new()` 接收 `window_attrs.decorations` 但未读取
4. `PlatformSpecificWindowBuilderAttributes` 仅有 `label` 和 `window_kind`，无 decorations
5. `openharmony-ability/crates/ability/src/window/mod.rs`：`create_os_window` 的 config 对象无 `decorations` 字段
6. `type.ets` 的 `WindowConfig` 接口无 decorations 字段
7. `FloatPage.ets`：MenuBarComponent + 拖拽区 + 关闭按钮无条件渲染，无 decorations 条件判断

**OHOS 窗口装饰原理：**
- OHOS 子窗口（FloatPage 使用 `windowStage.createSubWindow`）的"标题栏"实际是自定义 ArkUI 组件，不是 OS 原生标题栏
- `setWindowDecorHeight(height)` (API 11+) 取值范围 [37, 112]，不能设为 0，不适用于隐藏
- `setWindowSystemBarEnable([])` (API 9+) 可隐藏系统状态栏，需全屏/最大化模式
- `setSpecificSystemBarEnabled` (API 11+) 子窗口调用后不生效

**其他平台实现参考：**
- **Windows**：通过 `WindowFlags::MARKER_DECORATIONS` bitflag 控制 Win32 window style（`WS_CAPTION | WS_SYSMENU | WS_SIZEBOX`），支持运行时切换
- **macOS**：通过 `AtomicBool` 存储 decorations 状态，切换 `NSWindowStyleMask`（`Borderless` vs `Titled | Closable | Miniaturizable | Resizable`），支持运行时切换

## Goals / Non-Goals

**Goals:**
- 窗口创建时 `decorations: false` 生效：FloatPage 不渲染自定义标题栏（MenuBarComponent + 拖拽区 + 关闭按钮）
- 运行时 `set_decorations(bool)` 生效：动态切换标题栏显隐
- `is_decorated()` 返回正确的当前状态
- 主窗口 decorations=false 时隐藏系统状态栏（通过 `setWindowSystemBarEnable([])`）
- 与 Windows/macOS 行为意图一致：decorations=false = 无边框/无标题栏

**Non-Goals:**
- 不涉及窗口背景透明 — 属于 Phase 3
- 不涉及 WebView 透明背景 — 属于 Phase 1
- 不涉及窗口 resize handle 的隐藏（decorations=false 时 resize handle 保持，与 Windows/macOS 行为一致）
- 不涉及系统导航栏（底部导航条）隐藏 — 仅控制状态栏和自定义标题栏
- 不涉及 `setWindowDecorHeight` — 该 API 最小值为 37vp，无法隐藏标题栏

## Decisions

### Decision 1: decorations 状态存储在 tao Window 结构体 + LocalStorage 双重存储

**选择**：
- **Rust 侧**：`Window` 结构体添加 `decorations: AtomicBool`，`set_decorations()` 更新 AtomicBool 并通过 NAPI 通知 ArkTS
- **ArkTS 侧**：WindowManager 将 `decorations` 状态存入 `LocalStorage`，FloatPage 通过 `@LocalStorageProp('decorations')` 读取

**理由**：
- 与 macOS 实现模式一致（`AtomicBool` 存储 + 运行时切换）
- `LocalStorage` 是 FloatPage 已有的 per-window 状态传递机制（当前用于传递 `windowId`），复用该模式
- 使用 `@LocalStorageProp` 实现响应式更新：decorations 变化时 FloatPage 自动重渲染

**LocalStorage 双层机制说明**：
- FloatPage 的 `@Entry` 装饰器使用 `LocalStorage.getShared()`（全局单例）作为**默认回退**
- 真正的 per-window 隔离来自 `WindowManager.createSubWindow` 中的 `loadContentByName(routeName, storage)` —— 第二个参数 `storage` 是**新创建的 LocalStorage 实例**，覆盖了 @Entry 的默认值
- 因此 `@LocalStorageProp('decorations')` 读取的是 per-window 的 LocalStorage，不是全局的
- 主窗口（windowId=0）不使用 FloatPage（主窗口使用 MainPage），所以不存在 LocalStorage 冲突

**替代方案**：通过 NAPI 回调直接操作 FloatPage 组件属性 → 需要额外的跨线程通信机制，比 LocalStorage 复杂

### Decision 2: FloatPage 通过条件渲染控制标题栏显隐

**选择**：FloatPage.ets 的 `build()` 方法中，对三个装饰元素添加 `@LocalStorageProp('decorations')` 条件判断：
1. `MenuBarComponent`：已有 `if (this.isDesktop)` 守卫，添加 `&& this.decorations` 条件
2. 拖拽区 + 关闭按钮 Row：已有 `if (this.windowClass)` 守卫，添加 `&& this.decorations` 条件
3. resize handle：保持不变（decorations=false 时仍保留 resize 能力，与 Windows/macOS 一致）

**理由**：
- 复用 ArkUI 声明式条件渲染，无需手动 DOM 操作
- `@LocalStorageProp` 是响应式的，状态变化自动触发 UI 更新
- resize handle 保留是因为 Tauri 窗口即使无边框也通常需要可调整大小（Windows/macOS 均如此）

### Decision 3: 创建时 decorations 通过 WindowConfig 传递，运行时通过独立 NAPI 函数

**选择**：
- **创建时**：`openharmony-ability/crates/ability/src/window/mod.rs` 的 config 对象添加 `decorations` 字段（默认 `true`），传递给 `WindowManager.createSubWindow`
- **运行时**：新增 `setWindowDecorations(windowId: number, decorations: boolean)` NAPI 函数，tao 的 `set_decorations()` 调用此函数

**理由**：
- 创建时传递 config 对象是 `createOSWindow` 已有的模式，添加字段成本最低
- 运行时切换需要独立的 NAPI 函数，因为 tao 的 `set_decorations` 可能在窗口创建后的任意时刻调用
- 与 Windows 实现模式一致（创建时 `WindowFlags::MARKER_DECORATIONS` + 运行时 `set_decorations` 更新 flags）

### Decision 4: 主窗口 decorations=false 时通过 setWindowSystemBarEnable 隐藏状态栏

**选择**：当主窗口（UIAbility window）设置 `decorations: false` 时，在 ArkTS 侧调用 `windowClass.setWindowSystemBarEnable([])` 隐藏系统状态栏。

**理由**：
- 主窗口是 UIAbility 容器窗口，不走 `createOSWindow` 路径，需要通过 ArkTS 直接操作 window 实例
- `setWindowSystemBarEnable` 是 API 9+，满足最低 API 12 要求
- `setSpecificSystemBarEnabled` (API 11+) 子窗口调用后不生效，不适合统一使用
- 隐藏状态栏让主窗口内容可以延伸到顶部，实现无边框效果

**注意事项**：
- `setWindowSystemBarEnable` 在非全屏/最大化模式下不生效。Tauri OHOS 应用通常是全屏模式，应可正常工作
- 如果应用处于悬浮窗/分屏模式，状态栏隐藏不生效 → 作为已知限制记录

### Decision 5: tao 的 set_decorations 通过 TSFN 异步调用 NAPI

**选择**：`tao/src/platform_impl/ohos/mod.rs` 的 `set_decorations` 方法通过 `openharmony_ability` 的新 NAPI 函数 `set_window_decorations(window_id, decorations)` 调用 ArkTS。

**线程安全考虑**：
- `set_decorations` 可能从任意线程调用
- `create_os_window` 使用 `get_main_thread_env()` 获取主线程 NAPI 环境，`set_window_decorations` 应使用相同模式
- 如果 wry 调用 `set_decorations` 的线程与 `create_os_window` 相同（Chrome_IOThread），则 `get_main_thread_env()` 可正常工作
- **注意**：与 Phase 1 的 `set_background_color` 使用相同的线程模型

**替代方案**：使用 TSFN 将调用调度到 ArkTS 主线程 → 增加复杂度，且与现有 `create_os_window` 模式不一致

### Decision 6: WindowManager 存储 LocalStorage 引用以支持运行时更新

**选择**：`WindowManager.ets` 的 `windows` Map 需要同时存储 `window.Window` 对象和对应的 `LocalStorage` 引用，以支持运行时 `setDecorations` 通过 `storage.setOrCreate('decorations', newValue)` 更新。

**理由**：
- 当前 `windows` Map 仅存储 `window.Window` 对象，没有保留 `LocalStorage` 引用
- `@LocalStorageProp` 是单向同步：从 LocalStorage → 组件。要触发更新，必须通过 `LocalStorage.setOrCreate()` 修改存储值
- `createSubWindow` 中已有 `let storage = new LocalStorage()` 和 `storage.setOrCreate('windowId', windowId)` 的代码，只需将 `storage` 引用保存到 Map 中

**实施方式**：扩展 `windows` Map 的值类型：
```typescript
interface WindowEntry {
  window: window.Window;
  storage: LocalStorage;
}
// 替换原来的 Map<number, window.Window>
private windows: Map<number, WindowEntry> = new Map();
```

### Decision 7: 主窗口 decorations 通过独立 NAPI 调用传递

**选择**：主窗口（UIAbility window，window_id=0）不走 `createOSWindow` 路径。主窗口的 decorations=false 通过 tao 的 `set_decorations(false)` → NAPI `setWindowDecorations(0, false)` 传递，由 ArkTS 的 `setWindowDecorations` handler 处理。

**获取主窗口实例**：`WindowManager` 当前**没有** `mainWindow` 属性。获取主窗口有两种方式：
- **方式 A（推荐）**：在 `hideSystemBar()` 中调用 `this.windowStage.getMainWindowSync()` 获取主窗口实例，然后调用 `setWindowSystemBarEnable([])`
- **方式 B**：在 NativeAbility 的 `onWindowStageCreate` 中将主窗口引用传给 WindowManager 存储

选择方式 A，因为 `windowStage` 已在 WindowManager 中存在（`createSubWindow` 已使用），无需额外存储。

**理由**：
- 主窗口在 `Window::new()` 中设置 `window_id = Some(0)`，不调用 `create_os_window`
- `set_decorations` 方法已有 `if let Some(window_id) = self.window_id` 判断，window_id=0 也会进入 NAPI 调用路径
- ArkTS 侧的 `setWindowDecorations` handler 收到 `windowId=0` 时，知道是主窗口，调用 `WindowManager.hideSystemBar()` 处理

**时序考虑**：
- `setWindowSystemBarEnable` 需要在窗口处于全屏/最大化模式时才生效
- tao 创建主窗口后，应用通常已进入全屏模式，`set_decorations(false)` 调用时机通常在窗口创建之后
- 如果 `setWindowSystemBarEnable` 调用时窗口尚未全屏，操作不生效但不报错（静默跳过，符合降级策略）

### Decision 8: ArkHelper 接口与实现同步更新

**选择**：所有新增的 NAPI 函数（`setWindowDecorations`）必须同时更新两个位置：
1. `type.ets` 中的 `ArkHelper` 接口声明（添加 `setWindowDecorations: (windowId: number, decorations: boolean) => void`）
2. `ArkHelper.ets` 中 `createArkHelper()` 返回的对象字面量（添加实现）

**理由**：Rust NAPI 层通过 `get_named_property::<Function>("setWindowDecorations")` 查找函数。如果 `type.ets` 接口中未声明，TypeScript 不会强制实现存在，运行时 `get_named_property` 会找不到函数导致崩溃。

### Decision 9: ArkHelper.createOSWindow 调用链转发

**选择**：`ArkHelper.ets` 中 `createOSWindow` handler 当前用**位置参数**调用 `WindowManager.createSubWindow`：
```typescript
return await wm.createSubWindow(
    config.name, config.type, config.windowId,
    config.width, config.height, config.x, config.y
);
```
添加 `decorations` 后，必须在此调用中追加 `config.decorations` 参数。

**理由**：`WindowConfig` 接口添加了 `decorations` 字段，但 ArkHelper 不自动转发——需要显式添加参数。否则 `decorations` 值在 NAPI config 中存在但从未传递给 `createSubWindow`。

### Decision 10: 双份文件同步

**选择**：`native_ability/src/main/ets/` 和 `package/src/main/ets/` 两个目录存在相同文件的副本（`type.ets`、`ArkHelper.ets` 等）。所有 ArkTS 修改必须**同时应用到两个位置**。

**理由**：`package/` 是 HAR 打包的源目录，`native_ability/` 是开发目录。只修改一个会导致打包版本与开发版本不一致。

## cfg 隔离策略

本 Phase 所有修改均在 OHOS 特有代码路径中：

| 文件 | 隔离方式 | 说明 |
|------|---------|------|
| `tao/src/platform_impl/ohos/mod.rs` | 整个文件在 `ohos` 模块下 | 仅 OHOS 编译时包含 |
| `openharmony-ability/*` | 整个 crate 是 OHOS 专用 | 仅用于 OHOS 平台 |
| `FloatPage.ets` | ArkTS 文件，仅 OHOS | 不参与其他平台编译 |
| `WindowManager.ets` | ArkTS 文件，仅 OHOS | 不参与其他平台编译 |
| `ArkHelper.ets` | ArkTS 文件，仅 OHOS | 不参与其他平台编译 |
| `type.ets` | ArkTS 文件，仅 OHOS | 不参与其他平台编译 |

**结论**：无需添加额外的 `cfg` gate，所有修改天然隔离。

## Risks / Trade-offs

- **[setWindowSystemBarEnable 全屏依赖]** 主窗口 `setWindowSystemBarEnable([])` 仅在窗口处于全屏/最大化模式时生效。如果应用处于悬浮窗或分屏模式，状态栏隐藏不生效 → **降级行为**：记录为已知限制，decorations=false 在非全屏模式下仅隐藏自定义标题栏，状态栏保持显示
- **[setWindowDecorHeight 不可用于隐藏]** API 11+ 的 `setWindowDecorHeight` 取值范围 [37,112]，不能设为 0 → 本方案不依赖此 API，通过 FloatPage 条件渲染实现标题栏隐藏
- **[运行时 decorations 切换的竞态]** `set_decorations` 通过 NAPI 调用 ArkTS 更新 LocalStorage，触发 FloatPage 重渲染。如果快速连续调用 `set_decorations(true)` + `set_decorations(false)`，可能出现 UI 闪烁 → **缓解**：与 Windows/macOS 一致，不做防抖处理
- **[主窗口 decorations 状态传递]** 主窗口不走 `createOSWindow` 路径（window_id=0，复用 UIAbility 容器）。主窗口的 decorations=false 需要在 EntryAbility 或 WindowManager 初始化时处理 → **实施方案**：在 WindowManager 的初始化阶段检查主窗口 decorations 状态并调用 `setWindowSystemBarEnable`
- **[FloatPage LocalStorage 初始化]** WindowManager.createSubWindow 创建 LocalStorage 时需要将 decorations 状态写入。当前仅写入 `windowId`，需扩展写入 `decorations` → 简单添加 `storage.setOrCreate('decorations', config.decorations ?? true)` 即可
- **[API 版本兼容]** `setWindowSystemBarEnable` 是 API 9+，`setSpecificSystemBarEnabled` 是 API 11+。本方案使用 API 9+ 的 `setWindowSystemBarEnable`，满足 tauri api demo 默认 API 12 要求，无需版本守卫

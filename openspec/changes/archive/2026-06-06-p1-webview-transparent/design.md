## Context

当前 Tauri OHOS 适配中，WebView 透明背景的传递链路**大部分已打通**，但仍有关键断点：

**已修复的断点（源码中已完成）：**
1. ✅ `ArkHelper.ets:230`：`transparent` 优先级已修复为 `if (data?.transparent)`
2. ✅ `setBackgroundColor` 参数类型已从 `string` 改为 `number`（4 处全部更新）
3. ✅ `wry/src/ohos/mod.rs:244`：`set_background_color()` 已实现 RGBA → `0xAARRGGBB` 转换
4. ✅ `openharmony-ability` NAPI 层已改为 `Function<'_, u32, ()>`
5. ✅ `DefaultXComponent.ets`：父容器 Stack/Row/Column 已显式设置 `Color.Transparent`
6. ✅ `convertRRGGBBAAtoAARRGGBB` 函数已在 `Utils.ets:18` 实现并导入

**仍存在的断点（需实施）：**
1. ❌ `DefaultWebview.ets`：`convertRRGGBBAAtoAARRGGBB` 已导入但**未在 WebBuilder/EmbeddedWebBuilder 中调用**（line 56, 149 仍使用原始值）
2. ❌ renderMode 验证：默认异步渲染模式下 surface 是否透明未验证
3. ❌ set_background_color 线程安全：Task 5.4 子任务未确认

**重要发现 — 字符串转换路径是死代码**：Rust 端 `wry/src/ohos/mod.rs:66-68` 已在传入 NAPI 前将 RGBA 转为 `u32`，因此 `init.style.backgroundColor` 永远是 `number | Color | undefined`，**永远不会是 `#RRGGBBAA` 字符串**。`convertRRGGBBAAtoAARRGGBB` 的字符串分支（整个函数的核心逻辑）是死代码。但该函数仍应调用作为防御性编程（处理 `number` 和 `Color` 的透传），防止未来调用路径变化。

**遗留清理工作：**
- `webview.rs` 中有 6 处 `log::error!` 用于正常路径追踪，应降级为 `log::debug!`
- `wry/src/ohos/mod.rs` 使用 `eprintln!` 而非 `log` crate，应替换
- `demo_native/src/lib.rs:119` 的 `set_background_color` 仍传 `String`，与 ArkTS 端 `number` 类型不匹配

**OHOS Web 组件透明原理：**
- `.backgroundColor(Color.Transparent)` 设置 Web 容器自身背景透明
- 网页 CSS `body { background-color: transparent; }` 设置内容层透明
- 父容器（Stack/Row/Column/XComponent）也需要透明才能透出下层内容
- **渲染模式影响**：默认异步渲染（ASYNC_RENDER）使用独立 surface，可能不透；同步渲染（`RenderMode.SYNC_RENDER`）使用 canvas，支持 alpha 通道。当前代码库未设置 renderMode，需在设备上验证默认模式是否足够（见 Decision 5）

## Goals / Non-Goals

**Goals:**
- WebView 创建时 `transparent: true` 生效：Web 组件背景透明
- WebView 运行时 `set_background_color()` 生效：动态更新背景色（含透明）
- `background_color` 指定 RGBA 时正确应用（`#RRGGBBAA` → `#AARRGGBB` 格式转换）
- `transparent: true` 优先级高于 `background_color`（修复 ArkHelper.ets 现有优先级错误）
- 父容器在 WebView 透明时也不遮挡

**Non-Goals:**
- 不涉及窗口级别的透明（`tao` Window transparent）— 属于 Phase 3
- 不涉及无标题栏（decorations）— 属于 Phase 2
- 不修改 Web 页面自身 CSS（由应用前端控制）
- 不涉及 WebView 的 opacity 属性（全局透明度）

## Decisions

### Decision 1: transparent 优先于 background_color（修复 ArkHelper.ets）

**选择**：当 `transparent: true` 时，忽略 `background_color`，直接设置 `Color.Transparent`。**透明逻辑统一收敛在 ArkHelper.ets 层处理**，不传递给 WebBuilder/EmbeddedWebBuilder。

**理由**：
- 与 Windows/macOS 行为一致。wry 的 `WebViewAttributes` 文档明确说明 `transparent: true` 优先
- `transparent` 是 NAPI 层传入的属性，在 ArkHelper（NAPI 与 ArkUI 的桥接层）消费后无需向下传递
- `WebviewInitData`（DefaultWebview.ets）**不需要**添加 `transparent` 字段，保持接口精简

**修复位置**：`ArkHelper.ets:230` — 当前逻辑：
```typescript
// ❌ 当前：background_color 优先于 transparent（逻辑反了）
if (data?.transparent && !init.style.backgroundColor) {
  init.style.backgroundColor = Color.Transparent;
}
```
修复后：
```typescript
// ✅ 修复：transparent 优先于 background_color
if (data?.transparent) {
  init.style.backgroundColor = Color.Transparent;
}
```

**替代方案**：transparent 和 background_color 可叠加使用 → 增加复杂度且与其他平台不一致。

### Decision 2: 在 WebBuilder 和 EmbeddedWebBuilder 中进行颜色格式转换

**选择**：在 `DefaultWebview.ets` 的 `@Builder WebBuilder` 和 `@Builder EmbeddedWebBuilder` 两个构建函数中，对 `data.style?.backgroundColor` 进行 `#RRGGBBAA` → `#AARRGGBB` 格式转换。

**职责划分**：
- **ArkHelper.ets**：消费 `transparent`，设置 `Color.Transparent` 或保留 `background_color`（Decision 1）
- **WebBuilder/EmbeddedWebBuilder**：只做颜色格式转换，**不处理 transparent 逻辑**

**理由**：
- 两个 Builder 是 Web 组件的两个构造位置（主 WebView 和嵌入式 WebView），格式转换逻辑必须一致
- `transparent` 已在 ArkHelper 层处理，WebBuilder 收到的 `backgroundColor` 要么是 `Color.Transparent`（枚举）要么是 `#RRGGBBAA`（字符串），只需做类型安全的格式转换

**颜色格式转换**：
- wry 传入格式：`#RRGGBBAA`（如 `#FF000080` 表示半透明红）
- OHOS `.backgroundColor()` 接受格式：`#AARRGGBB` 字符串（如 `#80FF0000` 表示半透明红）
- **转换函数需要类型守卫**：`backgroundColor` 可能是 `string`（`#RRGGBBAA`）或 `Color` 枚举（`Color.Transparent`），函数需处理两种输入
  ```typescript
  export function convertRRGGBBAAtoAARRGGBB(color: string | Color): string | Color {
    if (typeof color !== 'string') return color; // Color 枚举直接透传
    if (!color.startsWith('#') || color.length !== 9) return color; // 非标准格式透传
    const rrggbb = color.substring(1, 7);
    const aa = color.substring(7, 9);
    return `#${aa}${rrggbb}`;
  }
  ```

**创建时 vs 运行时的职责划分**：
- **创建时**（WebBuilder/EmbeddedWebBuilder）：通过 `.backgroundColor(convertRRGGBBAAtoAARRGGBB(data.style?.backgroundColor))` 声明式属性设置
- **运行时**（set_background_color）：通过 `controller.setBackgroundColor()` 命令式 API 设置，参数为 `0xAARRGGBB` number
- `onControllerAttached` 中**不需要**重复调用 `setBackgroundColor()`，创建时的 `.backgroundColor()` 已生效

**替代方案**：在 `onControllerAttached` 回调中用 `controller.runJavaScript()` 注入透明 CSS → 时序不可靠，且仅影响内容层，不影响容器层。

### Decision 3: set_background_color 通过自定义 NAPI 桥接实现（非官方 API）

**选择**：`wry/src/ohos/mod.rs` 的 `set_background_color()` 调用 `webview.set_background_color()` → openharmony-ability Rust NAPI → ArkTS monkey-patch `setBackgroundColor` → `applyStyle` → WebBuilder 重渲染。

**重要说明**：`WebviewController.setBackgroundColor()` **不是 OHOS 官方 API**（经 arkts-helper 验证，官方 WebviewController 类中不存在此方法）。本方案通过 openharmony-ability 的自定义 NAPI 桥接实现：
- ArkHelper.ets 在 WebView 创建后，将 `setBackgroundColor` 作为 monkey-patch 注入到 controller 对象上
- 该 monkey-patch 调用 `applyStyle({ backgroundColor: color })` → 合并到 `init.style` → 触发 `node.update()` → WebBuilder 重渲染
- 这与 `setVisible`、`dispose` 等方法使用相同的 monkey-patch 模式

**理由**：
- `Utils.ets` 已有 `setBackgroundColor` 扩展声明（`JsHelper` 接口）
- `openharmony-ability` 的 `Webview::set_background_color()` 方法已实现（`helper/webview.rs:241`）
- 只需打通 wry → openharmony-ability 的调用即可

**参数类型修正（string → number）**：
- 当前 ArkTS 声明：`setBackgroundColor: (color: string) => void`
- OHOS 官方 Web `.backgroundColor()` 属性接受 `ResourceColor`（`string | number | Color`），number 格式为 `0xAARRGGBB`
- **解决方案**：修改为 `(color: number) => void`，wry 端传入 `0xAARRGGBB` 格式的 number
- monkey-patch 中 `applyStyle({ backgroundColor: number })` 同样有效，因为 Web `.backgroundColor()` 接受 number

**需要同步修改的 4 个位置**：
1. `openharmony-ability/crates/ability/src/helper/webview.rs`：`set_background_color` 方法签名和 NAPI Function 泛型（`String` → `u32`）
2. `openharmony-ability/native_ability/src/main/ets/webview/Utils.ets`：`JsHelper` 接口和 `ProxyJsHelper` 类的 `setBackgroundColor` 签名
3. `openharmony-ability/native_ability/src/main/ets/webview/DefaultWebview.ets`：`WebviewController` 扩展声明
4. **`openharmony-ability/native_ability/src/main/ets/ability/ArkHelper.ets:271,345`**：两处猴子补丁覆盖的 `setBackgroundColor`（当前为 `(color: string) => applyStyle()`，需改为 `(color: number) => applyStyle()`）。同时 `WebviewStyle.backgroundColor` 类型需扩展为 `string | Color | number`

**线程安全**：
- `wry` 的 `set_background_color` 可能从任意线程调用
- `openharmony-ability` 的 `set_background_color` 方法内部使用 `get_main_thread_env()` 获取主线程 NAPI 环境，然后直接调用 JS 函数 — 这保证了 NAPI 调用在主线程执行
- **注意**：当前实现**不使用 TSFN**，而是直接在主线程调用 JS 函数。这意味着如果从非主线程调用，`get_main_thread_env()` 可能返回 `None`（因为 `MAIN_THREAD_ENV` 存储在 `thread_local!` 中）。需要确认 wry 调用 `set_background_color` 的线程是否一定是主线程
- 如果不是主线程，需要通过 TSFN 将调用调度到主线程（类似其他跨线程 NAPI 操作的模式）

### Decision 4: 父容器透明通过全层级容器 backgroundColor 控制（防御性设置）

**选择**：`DefaultXComponent.ets` 的 `build()` 方法中，在 `Stack`、`Row`、`Column` 三个容器上都显式添加 `.backgroundColor(Color.Transparent)`。

**理由**：
- `build()` 的容器层级为 `Stack → Row → Column → NodeContainer(Web)`，任何一层不透明都会遮挡透明穿透
- 当前代码中这三层容器**均未设置 `backgroundColor`**，ArkUI 容器默认背景为透明，所以这可能已经可以工作
- 但显式设置作为**防御性编程**是必要的：防止 ArkUI 默认行为变化、框架升级或第三方样式注入导致容器变为不透明
- 对不透明 WebView 无副作用（子组件不透明时自然遮盖）

### Decision 5: 渲染模式 — 需验证 SYNC_RENDER 对透明度的影响

**发现**（来自 arkts-helper 官方文档验证）：

OHOS Web 组件有两种渲染模式：
- **ASYNC_RENDER**（异步，默认）：Web 组件作为独立 surface 节点送显。surface 本身可能默认不透明
- **SYNC_RENDER**（同步）：Web 组件作为 canvas 节点与系统组件一起送显，canvas 支持 alpha 通道，更易于实现背景透明

**当前状态**：代码库中**未设置 `renderMode`**，使用默认的异步渲染模式。

**影响**：
- 如果异步渲染模式下 `.backgroundColor(Color.Transparent)` 无法让 surface 透明，则需要切换到 `renderMode: RenderMode.SYNC_RENDER`
- 但同步渲染模式有额外性能消耗，且不支持动态切换

**实施策略**：
1. 先在异步渲染模式下测试透明效果
2. 如果不透明，添加 `renderMode: RenderMode.SYNC_RENDER` 到 WebBuilder/EmbeddedWebBuilder
3. 在 design.md Risks 中标注此风险

**验证任务**：见 tasks.md Task 5.3

## cfg 隔离策略

本 Phase 所有修改均在 OHOS 特有代码路径中，不涉及跨平台代码：

| 文件 | 隔离方式 | 说明 |
|------|---------|------|
| `wry/src/ohos/mod.rs` | 整个文件在 `ohos` 模块下 | `mod.rs` 位于 `src/ohos/` 目录，仅 OHOS 编译时包含 |
| `openharmony-ability/*` | 整个 crate 是 OHOS 专用 | openharmony-ability 仅用于 OHOS 平台 |
| `DefaultWebview.ets` | ArkTS 文件，仅 OHOS | 不参与其他平台编译 |
| `DefaultXComponent.ets` | ArkTS 文件，仅 OHOS | 不参与其他平台编译 |
| `ArkHelper.ets` | ArkTS 文件，仅 OHOS | 不参与其他平台编译 |
| `Utils.ets` | ArkTS 文件，仅 OHOS | 不参与其他平台编译 |

**结论**：无需添加额外的 `cfg` gate，所有修改天然隔离。其他平台（Windows/macOS/Linux）的编译和行为不受影响。

## Risks / Trade-offs

- **[Phase 3 依赖 — 窗口级透明]** Phase 1 仅实现 WebView 容器级别的透明。如果窗口本身不透明（`tao` Window transparent），WebView 透明后仍然无法穿透到桌面或其他应用窗口。**Phase 1 的透明效果仅在窗口本身已透明的场景下可见**（如叠加层、毛玻璃容器等）。完整桌面穿透效果需等待 Phase 3 窗口级透明支持。
- **[渲染模式与 surface 透明度 — 需验证]** OHOS Web 组件默认使用异步渲染模式（ASYNC_RENDER），该模式下 Web 作为独立 surface 节点送显，**surface 可能默认不透明**。如果 `.backgroundColor(Color.Transparent)` 无法让 surface 透明，需切换到 `renderMode: RenderMode.SYNC_RENDER`（同步渲染使用 canvas，支持 alpha 通道）。当前代码库未设置 renderMode。**实施前必须在设备上验证**（Task 5.3）。
- **[WebviewController.setBackgroundColor 非官方 API]** 运行时 `set_background_color` 依赖 openharmony-ability 自定义的 NAPI monkey-patch，不是 OHOS 官方 API。如果未来 OHOS 提供官方 API，可能需要迁移。但当前 monkey-patch 模式已被 `setVisible`、`dispose` 等方法验证可行。
- **[set_background_color 线程安全]** `wry` 的 `set_background_color` 可能从非主线程调用，而 `openharmony-ability` 的 NAPI 调用依赖 `get_main_thread_env()` 获取主线程环境（`thread_local!`）。**如果从非主线程调用，`get_main_thread_env()` 返回 `None` 导致调用失败**。需要在实施时确认 wry 调用 `set_background_color` 的线程上下文，如果不是主线程则需通过 TSFN 调度。
- **[Web 组件透明限制]** OHOS Web 组件的透明行为可能受系统版本影响 → 使用 API 12+ 的标准属性，覆盖目标设备
- **[父容器透明副作用]** Stack/Row/Column 显式设置透明背景可能影响不透明 WebView 场景 → 实际无影响：不透明 WebView 自然遮盖，且当前代码已默认透明，显式设置仅为防御性编程
- **[颜色格式转换开销]** ArkTS 端需要 `#RRGGBBAA` → `#AARRGGBB` 转换 → 字符串操作开销极小（<1ms），可忽略
- **[number 精度限制]** `0xAARRGGBB` 作为 JavaScript number 可能超出安全整数范围（`2^53`）→ 实际最大值为 `0xFFFFFFFF`（约 42 亿），远小于 `2^53`（约 9007 万亿），安全

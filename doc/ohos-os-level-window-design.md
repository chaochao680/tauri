# OpenHarmony 操作系统级窗口创建设计

## 1. 概述
本文档概述了在 Tauri 中为 OpenHarmony (OHOS) 实现真正的多窗口支持的设计方案。目前，Tauri 在 OHOS 上是通过在单个系统窗口内添加 WebView 来模拟多窗口的。本设计旨在通过调用 `@ohos.window` API 来创建真正的操作系统级窗口。

## 2. 问题陈述
**当前行为（`tao/src/platform_impl/ohos/mod.rs` 中的第 4 步）：**
当调用 `Window::new` 时，`tao` 仅仅克隆了 `OpenHarmonyApp` 句柄。它**并没有**创建新的系统窗口。所有的“窗口”实际上都渲染在主 `UIAbility` 的窗口层级中。

**目标行为：**
对 `Window::new` 的调用应当触发创建一个独立的操作系统窗口（例如 `TYPE_NORMAL` 或 `TYPE_FLOAT`），该窗口可以被操作系统窗口管理器独立移动、调整大小和管理。

## 3. 参考实现
基于 `demo3signature/entry/src/main/ets/pages/Index.ets` 中的示例代码，OHOS 创建窗口的标准方式如下：

```typescript
import { window } from '@kit.ArkUI';
import { common } from '@kit.AbilityKit';

// 1. 获取上下文
let context = getContext(this) as common.UIAbilityContext;

// 2. 定义配置
let subWindowConfig: window.Configuration = {
    name: "MySubWindow",
    windowType: window.WindowType.TYPE_FLOAT, // 或者 TYPE_NORMAL
    ctx: context
};

// 3. 创建窗口
let subWindow = await window.createWindow(subWindowConfig);

// 4. 设置内容并显示
await subWindow.setUIContent('pages/MyPage');
await subWindow.showWindow();
await subWindow.resize(500, 800);
```

## 4. 提议的架构

### 4.1 高层数据流
1.  **Tauri 应用 (Rust)**：调用 `WebviewWindowBuilder::build()`。
2.  **tao (Rust)**：调用 `Window::new()`。
    *   *变更点*：不再返回虚拟句柄，而是调用新的 NAPI 函数 `create_os_window`。
3.  **openharmony-ability (Rust/NAPI)**：将调用桥接到 ArkTS 侧。
4.  **@ohos-rs/ability (ArkTS)**：
    *   接收配置（名称、类型、尺寸）。
    *   调用 `window.createWindow()`。
    *   设置一个通用的内容页面（例如 `TauriWindowPage`），用于承载 WebView。
    *   返回一个 `WindowHandle` (ID) 给 Rust。
5.  **tao (Rust)**：存储 `WindowHandle` 并将其用于后续操作（调整大小、获取焦点等）。

### 4.2 组件变更

#### A. ArkTS 侧 (`@ohos-rs/ability`)
我们需要在 ArkTS 中引入一个 `WindowManager` 来跟踪创建的窗口。

*   **新文件**：`src/main/ets/window/WindowManager.ets`
*   **关键方法**：
    *   `createWindow(config: WindowConfig): Promise<number>`：调用 `window.createWindow`，设置内容，返回 ID。
    *   `getWindow(id: number): window.Window`：获取窗口实例。
    *   `closeWindow(id: number)`：销毁窗口。

**ArkTS 实现草图：**
```typescript
import { window } from '@kit.ArkUI';

export class WindowManager {
  private windows: Map<number, window.Window> = new Map();
  private context: common.UIAbilityContext;

  constructor(context: common.UIAbilityContext) {
    this.context = context;
  }

  async createWindow(name: string, type: number): Promise<number> {
    const config: window.Configuration = {
      name: name,
      windowType: type,
      ctx: this.context
    };
    
    const win = await window.createWindow(config);
    // 存储引用
    const id = this.generateId();
    this.windows.set(id, win);
    
    // 加载通用的 Tauri 页面，该页面将接受来自 Rust 的 WebView
    await win.setUIContent('pages/TauriWebViewPage'); 
    await win.showWindow();
    
    return id;
  }
}
```

#### B. Rust 侧 (`openharmony-ability` crate)
通过 NAPI 将 ArkTS 管理器暴露给 Rust。

*   **新函数**：`pub fn create_os_window(name: String, window_type: i32) -> Result<i64>`
*   **位置**：`crates/ability/src/window/mod.rs`（新模块）

#### C. Tao 集成 (`tao` crate)
更新 `tao/src/platform_impl/ohos/mod.rs`。

*   **`Window::new`**：
    *   调用 `openharmony_ability::create_os_window(...)`。
    *   存储返回的 ID。
    *   实现 `raw_window_handle` 以返回与此新窗口类型兼容的句柄（或者如果 WebView 可以附加到这个新窗口表面，则适配现有逻辑）。

## 5. 实施步骤

1.  **阶段 1：ArkTS 基础设施**
    *   在 `@ohos-rs/ability` 中创建 `WindowManager`。
    *   创建一个通用的 `TauriWebViewPage`（ArkTS），它可以接收来自 Rust 的 WebView 控制器（类似于目前 `DefaultXComponent` 的工作方式）。

2.  **阶段 2：NAPI 桥接**
    *   将 `create_os_window` 添加到 NAPI 辅助对象中。
    *   确保窗口方法（resize, move）被映射为 NAPI 调用。

3.  **阶段 3：Tao 集成**
    *   修改 `Window::new` 以使用该桥接。
    *   确保 `EventLoop` 能够接收来自多个窗口的事件（目前它假设只有一个主窗口）。

## 6. 挑战与考量
*   **事件循环 (Event Loop)**：当前 `tao` 中的 `EventLoop` 紧密绑定在主 Ability 生命周期上。我们需要确保来自新 OS 窗口的事件（触摸、调整大小等）能够转发给 Rust 事件循环。
*   **WebView 挂载**：我们需要弄清楚如何将 Wry 创建的 `Web` 组件附加到*新* OS 窗口的 UIContent 上。`setUIContent` 加载一个页面；该页面需要一个插槽（NodeContainer）供 Rust 注入 WebView。
*   **生命周期管理**：处理子窗口的 `onWindowStageDestroy` 以防止内存泄漏。

## 7. 结论
通过从“主窗口注入 WebView"转向"OS 窗口创建 + WebView 注入”，Tauri 在 OHOS 上将支持真正的多窗口工作流，从而与桌面平台（Windows/macOS/Linux）的行为保持一致。

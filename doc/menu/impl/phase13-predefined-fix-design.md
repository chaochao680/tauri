# Phase 13: Predefined Item 功能缺口修复与 Menu/Tray 统一

## Status: Implemented

## Context

Phase 3 实现了 PredefinedMenuItem 的基础框架，但实际验证发现以下问题：

1. **Copy/Paste 不生效**: api demo 默认 menubar 的 Edit 菜单中 SelectAll 可以正常工作，但 Copy 和 Paste 不生效。根本原因是 `document.execCommand` 在 ArkWeb 中已废弃。
2. **两条独立的执行路径**: Menu 和 Tray 各自维护一套 predefined action 实现，代码重复且行为不一致。
3. **多窗口语义缺失**: Menu 已支持 windowId（popup menu 可来自任意窗口），但 predefined action 的窗口操作始终使用主窗口。Tray 在 OHOS 上唯一，其窗口操作的目标窗口未明确定义。
4. **Clipboard plugin 在 OHOS 不可用**: `plugins-workspace/plugins/clipboard-manager/src/lib.rs` 通过 `#[cfg(any(desktop, target_env = "ohos"))]` 将 OHOS 路由到 `desktop.rs`，但 `desktop.rs` 使用 `arboard::Clipboard::new()` — **arboard 不支持 OHOS**，初始化时静默失败。因此 predefined menu item 必须自行实现剪贴板操作，不能依赖 clipboard plugin。
5. **Tray minimize 窗口闪烁**: OHOS 桌面模式下，StatusBar 右键菜单关闭时系统自动恢复主窗口焦点 (WINDOW_ACTIVE)，与 `win.minimize()` 竞态，导致窗口"闪缩还原"。

本文档系统审计所有 predefined item 的实现状态，设计修复方案，并统一 menu/tray 两条路径。

### 决策记录

| 决策项 | 选择 | 理由 |
|--------|------|------|
| CloseWindow 行为 | 保持 minimize | 不改为 terminateSelf，与当前行为一致 |
| Tray/Menu 架构 | 统一（单一 executor） | 消除代码重复，保证行为一致 |
| 剪贴板权限 | ~~添加 `READ_PASTEBOARD`~~ 暂不添加 | `system_basic` 级受限权限，需 AGC Profile 授权，侧载安装失败。改为 try/catch 容错 |
| 多窗口语义 | Option C (Tray→主窗口, Menu→源窗口) | 与 macOS 行为一致，优于 Windows |
| 统一方案 | Plan 2 (executor + 可选 targetWindowId) | 最小改动，向下兼容 |
| Tray minimize 闪烁 | 延迟 minimize，事件驱动 | 注册 WINDOW_ACTIVE 监听，系统焦点恢复后再 minimize，避免竞态 |

---

## 一、跨平台行为对照：Windows / macOS / OHOS

### 1.1 各平台 predefined action 目标窗口语义

```
┌──────────────────────────────────────────────────────────────────┐
│                  目标窗口选择策略                                  │
├──────────┬──────────────────┬──────────────────┬─────────────────┤
│ 操作     │ Windows          │ macOS            │ OHOS (设计)     │
├──────────┼──────────────────┼──────────────────┼─────────────────┤
│ Menu     │ hwnd = menu 所在 │ ObjC selector    │ windowId =      │
│ minimize │ 窗口 (通过       │ 沿 responder     │ 菜单所在窗口    │
│          │ subclass proc    │ chain → key      │ (由 onMenu-     │
│          │ 传入)            │ window           │ Request 携带)   │
├──────────┼──────────────────┼──────────────────┼─────────────────┤
│ Menu     │ SendInput 模拟   │ copy:/paste:     │ pasteboard +    │
│ copy/    │ Ctrl+C/V →      │ selector →       │ getSelection/   │
│ paste    │ 发给 focused     │ key window       │ insertText      │
│          │ window           │                  │ (当前窗口       │
│          │                  │                  │  WebView)       │
├──────────┼──────────────────┼──────────────────┼─────────────────┤
│ Tray     │ hwnd = tray 的   │ selector →       │ windowId = 0    │
│ minimize │ 隐藏窗口 →      │ key window       │ (主窗口)        │
│          │ 实际 no-op       │ (有效操作)       │ (有效操作)      │
├──────────┼──────────────────┼──────────────────┼─────────────────┤
│ Tray     │ 同上 → no-op    │ selector →       │ pasteboard +    │
│ copy/    │                  │ key window       │ 主窗口 WebView  │
│ paste    │                  │                  │                 │
└──────────┴──────────────────┴──────────────────┴─────────────────┘
```

### 1.2 分析结论

- **Windows Tray minimize**: hwnd 指向 tray 的隐藏消息窗口，`ShowWindow(hwnd, SW_MINIMIZE)` 实际无效果 — 属于平台设计缺陷
- **macOS Tray minimize**: ObjC selector `performMiniaturize:` 沿 responder chain 到达 key window，行为正确
- **OHOS (Option C) Tray minimize**: 明确操作主窗口 (windowId=0)，行为清晰，**优于 Windows**
- **OHOS (Option C) Menu minimize**: 操作菜单源窗口 (windowId 由 Rust 侧传入)，**与 macOS 行为一致**

### 1.3 OHOS 特有约束

- **Tray 唯一性**: 华为官方确认 OHOS StatusBar API 每个应用只允许一个 tray icon（`statusBarManager.addToStatusBar` 只接受一个 context）
- **Menu 多窗口**: Menu 已通过 `onMenuRequest` 的 `windowId` 字段支持多窗口（popup menu 可来自任意窗口）
- **Menubar 仅主窗口**: 当前架构中 menubar 只在 MainPage (windowId=0) 渲染，子窗口 (FloatPage) 没有 menubar；但 popup menu（右键菜单）可来自任意窗口

---

## 二、关键审计发现：WebView 原生键盘快捷键

### 2.1 ArkWeb WebView 原生处理剪贴板快捷键

华为官方确认：**ArkWeb Web 组件默认支持 Ctrl+X/C/V 键盘快捷键自动处理剪贴板操作**，无需开发者额外编写代码。当设备连接物理键盘时，这些快捷键直接与系统剪贴板交互，功能完整（支持文本、图片、富文本）。

这意味着：
- **Ctrl+C** → WebView 原生复制 ✅ 已可用
- **Ctrl+V** → WebView 原生粘贴 ✅ 已可用
- **Ctrl+X** → WebView 原生剪切 ✅ 已可用
- **Ctrl+A** → WebView 原生全选 ✅ 已可用
- **Ctrl+Z/Y** → WebView 原生 undo/redo ✅ 已可用

### 2.2 与 Phase 11 Accelerator 拦截的冲突

Phase 11 设计使用 `onKeyPreIme`（第一派发，获焦链叶→根）拦截所有 accelerator 快捷键。如果在 Column 上拦截 Ctrl+C 并返回 `true`：

```
用户按 Ctrl+C
  → Column.onKeyPreIme 匹配 accelerator → return true (消费)
  → WebView 永远不收到 Ctrl+C
  → WebView 的原生剪贴板处理被阻断
  → 改为执行我们的 pasteboard 方案（功能降级：只支持纯文本）
```

**结论**：Phase 11 的 `AcceleratorMatcher` **不应拦截剪贴板类快捷键** (copy/cut/paste/selectAll/undo/redo)，应让 WebView 原生处理。

### 2.3 Menu 点击 vs 键盘快捷键 — 两条独立路径

```
路径 A: 键盘快捷键 (Ctrl+C/V/X/A/Z/Y)
  → 当前: WebView 原生处理 ✅ (不需要我们的代码)
  → Phase 11 后: 仍然让 WebView 处理 (accelerator matcher 跳过这些键)

路径 B: 菜单点击 (Edit → Copy)
  → 当前: document.execCommand("copy") ❌ (已废弃)
  → 修复后: @ohos.pasteboard + runJavaScript ✅
```

**路径 B 是本次修复的重点**。路径 A 已经可用，且不应被破坏。

---

## 三、架构问题：两条独立的执行路径

### 3.1 当前状态

```
Menu 路径:
  Rust onMenuRequest(windowId)
    → ArkTS MenuManager.popupFromJson(json, x, y, windowId)
      → UI 渲染菜单 → 用户点击
        → MenuManager.handleItemClick(item)
          → PredefinedActionExecutor.execute(type)   ← menu.ets
            → this.win (固定主窗口) / this.controller (固定主 WebView)

Tray 路径:
  Rust TrayIcon click event
    → event.rs: execute_predefined_action(type)
      → TSFN → ArkHelper.executePredefinedAction(actionType)  ← ArkHelper.ets
        → 独立 switch/case (与 menu 侧代码重复)
          → context.windowStage.getMainWindowSync() (硬编码主窗口)
```

### 3.2 目标状态

```
统一路径:
  Menu 路径 ──┐
              ├→ PredefinedActionExecutor.execute(type, about?, targetWindowId?)
  Tray 路径 ──┘     ↓
                    WindowManager.getWindow(targetWindowId ?? 0)
                    WindowManager.getController(targetWindowId ?? 0)
```

- 单一 `PredefinedActionExecutor` 实例，通过全局 getter 共享
- Tray 通过 `ArkHelper.ets` 委托给同一 executor
- 可选 `targetWindowId` 参数：Menu 传源窗口 ID，Tray 传 0（主窗口）

---

## 四、Predefined Item 状态审计（修复前）

| # | Item | Menu 侧 | Tray 侧 | 键盘(Ctrl+) | 修复动作 |
|---|------|---------|---------|------------|---------|
| 1 | **copy** | ❌ `execCommand` 不可靠 | ❌ 不支持 | ✅ WebView 原生 | 修复 menu 点击路径 |
| 2 | **cut** | ❌ `execCommand` 不可靠 | ❌ 不支持 | ✅ WebView 原生 | 修复 menu 点击路径 |
| 3 | **paste** | ❌ `execCommand` 不可用 | ❌ 不支持 | ✅ WebView 原生 | 修复 menu 点击路径 |
| 4 | **selectAll** | ✅ `execCommand` 可工作 | ❌ 不支持 | ✅ WebView 原生 | 保持 execCommand |
| 5 | **undo** | ⚠️ `execCommand` 未验证 | ❌ 不支持 | ✅ WebView 原生 | 保持 execCommand |
| 6 | **redo** | ⚠️ `execCommand` 未验证 | ❌ 不支持 | ✅ WebView 原生 | 保持 execCommand |
| 7 | separator | ✅ | ✅ | — | OK |
| 8 | minimize | ✅ 主窗口 | ✅ 主窗口 | — | 统一后支持 targetWindowId |
| 9 | maximize | ✅ 主窗口 | ✅ 主窗口 | — | 统一后支持 targetWindowId |
| 10 | fullscreen | ✅ 主窗口 | ✅ 主窗口 | — | 统一后支持 targetWindowId |
| 11 | close | ⚠️=minimize | ⚠️=minimize | — | **保持 minimize（用户确认）** |
| 12 | hide | =minimize | =minimize | — | OK |
| 13 | recover | ✅ 主窗口 | ❌ 不支持 | — | 统一后自动修复 |
| 14 | quit | ✅ exit(0) | ✅ exit(0) | — | OK |
| 15 | about | ✅ AlertDialog | ❌ Rust 不转发 | — | 补齐 tray 转发 |
| 16 | hideOthers | no-op | 不支持 | — | OK (OHOS 不支持) |
| 17 | showAll | no-op | 不支持 | — | OK (OHOS 不支持) |
| 18 | services | macOS only | 不支持 | — | OK |
| 19 | bringAllToFront | macOS only | 不支持 | — | OK |
| 20 | none | no-op | 不支持 | — | OK |

**需修复**: copy/cut/paste (改用 pasteboard)、recover (tray 补齐)、about (tray 补齐)、unify tray path

---

## 五、实施步骤

### Step 1: 修复剪贴板操作（PredefinedActionExecutor）

**文件**: `openharmony-ability/native_ability/src/main/ets/helper/menu.ets`

将剪贴板操作从 `document.execCommand` 改为 `@ohos.pasteboard` 原生 API。

**注意**: 这些修复只影响**菜单点击路径**（路径 B）。键盘快捷键仍由 WebView 原生处理。

```typescript
import { pasteboard } from '@kit.BasicServicesKit';

case 'copy': {
    const raw = await this.controller?.runJavaScript('window.getSelection().toString()');
    if (raw) {
        const text = JSON.parse(raw);
        if (text) {
            const pasteData = pasteboard.createData(pasteboard.MIMETYPE_TEXT_PLAIN, text);
            const systemPasteboard = pasteboard.getSystemPasteboard();
            await systemPasteboard.setData(pasteData);
        }
    }
    break;
}

case 'cut': {
    const raw = await this.controller?.runJavaScript('window.getSelection().toString()');
    if (raw) {
        const text = JSON.parse(raw);
        if (text) {
            const pasteData = pasteboard.createData(pasteboard.MIMETYPE_TEXT_PLAIN, text);
            const systemPasteboard = pasteboard.getSystemPasteboard();
            await systemPasteboard.setData(pasteData);
            await this.controller?.runJavaScript('document.execCommand("delete")');
        }
    }
    break;
}

case 'paste': {
    const systemPasteboard = pasteboard.getSystemPasteboard();
    const pasteData = await systemPasteboard.getData();
    const text = pasteData.getPrimaryText();
    if (text) {
        const escaped = text.replace(/\\/g, '\\\\').replace(/'/g, "\\'").replace(/\n/g, '\\n');
        await this.controller?.runJavaScript(
            `document.execCommand("insertText", false, '${escaped}')`
        );
    }
    break;
}
```

**权限**: 在 api demo 的 `module.json5` 中添加 `ohos.permission.READ_PASTEBOARD`。

### Step 2: 扩展 PredefinedActionExecutor 支持 targetWindowId

**文件**: `openharmony-ability/native_ability/src/main/ets/helper/menu.ets`

当前 `execute(type, aboutMetadata)` 使用固定的 `this.win` 和 `this.controller`（初始化时 set 的主窗口/主 WebView）。扩展为可选动态查找：

```typescript
import { WindowManager } from '../window/WindowManager';

async execute(
    type: PredefinedType,
    aboutMetadata?: AboutMetadataData,
    targetWindowId?: number
): Promise<void> {
    // 动态解析目标窗口和控制器
    let win: window.Window | null = this.win;
    let controller: web_webview.WebviewController | null = this.controller;

    if (targetWindowId !== undefined && targetWindowId !== 0) {
        try {
            const wm = WindowManager.getInstance();
            win = wm.getWindow(targetWindowId) ?? this.win;
            const rustCtrl = wm.getController(targetWindowId);
            // RustWebviewNodeController 包含实际的 WebviewController
            // 此处需要根据实际 API 获取
        } catch (_) {
            // fallback to main window
        }
    }

    switch (type) {
        case 'copy': { /* 使用 controller（剪贴板操作始终使用主 WebView） */ break; }
        case 'minimize': await win?.minimize(); break;
        case 'maximize': await win?.maximize(); break;
        // ... 窗口操作使用 win，剪贴板操作使用 controller
    }
}
```

**设计要点**：
- 窗口操作 (minimize/maximize/fullscreen/recover/close/hide) 使用 `targetWindowId` 解析的窗口
- 剪贴板操作 (copy/cut/paste/selectAll/undo/redo) 始终使用 `this.controller`（主 WebView），因为剪贴板操作需要有焦点的 WebView
- `targetWindowId` 为 `undefined` 或 `0` 时使用已有的 `this.win`，零额外开销
- **当前 menubar 只在主窗口 (MainPage) 渲染**，所以 menu 路径的 targetWindowId 暂时总是 0；未来若子窗口也有 menubar，只需传入正确的 windowId 即可

**当前限制**: `MenuManager.handleItemClick` 调用 `this.executor.execute(item.predefinedType, item.aboutMetadata)` 时**未传 windowId**。`popupFromJson(json, x, y, windowId)` 接收了 windowId 但未存储到 MenuManager 实例。本次修复阶段 targetWindowId 暂不传递（默认走主窗口），完整的多窗口 windowId 转发见 [Phase 14](phase14-remaining-features-scope.md) (MW-2, MW-3)。

### Step 3: 统一 Tray 执行路径

#### 3a. 暴露 PredefinedActionExecutor 实例

**文件**: `openharmony-ability/native_ability/src/main/ets/helper/menu.ets`

```typescript
let globalExecutor: PredefinedActionExecutor | null = null;

export function setPredefinedActionExecutor(executor: PredefinedActionExecutor): void {
    globalExecutor = executor;
}

export function getPredefinedActionExecutor(): PredefinedActionExecutor | null {
    return globalExecutor;
}
```

#### 3b. NativeAbility.ets 注册全局实例

**文件**: `openharmony-ability/native_ability/src/main/ets/ability/NativeAbility.ets`

```typescript
// setupMenuPopup 中，executor 创建后立即注册：
const executor = new PredefinedActionExecutor((code: number) => {
    this.context?.terminateSelf();
});
executor.setWindow(mainWindow);
// ... existing setup ...
setPredefinedActionExecutor(executor);  // ← 新增
```

#### 3c. ArkHelper.ets 委托给 PredefinedActionExecutor

**文件**: `openharmony-ability/native_ability/src/main/ets/ability/ArkHelper.ets`

替换现有的独立 switch/case 实现：

```typescript
executePredefinedAction: (actionType: string) => {
    const executor = getPredefinedActionExecutor();
    if (executor) {
        // Tray 不传 targetWindowId → 默认使用主窗口 (this.win)
        executor.execute(actionType as PredefinedType);
    } else {
        hilog.warn(DOMAIN, 'ArkHelper',
            'PredefinedActionExecutor not initialized, action: %{public}s', actionType);
    }
}
```

**替换范围**: 删除 ArkHelper.ets 中 `executePredefinedAction` 的整个 switch/case 块（约 60 行），替换为 5 行委托代码。原有的 quit/minimize/maximize/fullscreen 逻辑全部由 `PredefinedActionExecutor.execute` 统一处理。

#### 3d. Tray Rust 侧补齐 action 转发

**文件**: `tray-icon/src/platform_impl/ohos/event.rs`

```rust
fn execute_predefined_action(predefined_type: &str) {
    match predefined_type {
        "quit" => {
            let app = super::get_ohos_app();
            app.exit(0);
        }
        "minimize" | "hide" | "maximize" | "close" | "fullscreen"
        | "copy" | "cut" | "paste" | "selectAll" | "undo" | "redo"
        | "recover" | "about" => {
            openharmony_ability::statusbar::execute_predefined_action(predefined_type).ok();
        }
        _ => {
            log::debug!("[TrayIcon] unsupported predefined action: {}", predefined_type);
        }
    }
}
```

**新增转发**: `copy`, `cut`, `paste`, `selectAll`, `undo`, `redo`, `recover`（原来只有 minimize/hide/maximize/close/fullscreen/about）

### Step 4: 补齐 ArkTS 类型定义

**文件**: `openharmony-ability/native_ability/src/main/ets/helper/menu_types.ets`

```typescript
export type PredefinedType =
  | 'separator'
  | 'copy' | 'cut' | 'paste' | 'selectAll'
  | 'undo' | 'redo'
  | 'minimize' | 'maximize' | 'fullscreen' | 'recover' | 'restore' | 'destroyWindow' | 'close'
  | 'quit' | 'about' | 'hide' | 'hideOthers' | 'showAll'
  | 'services' | 'bringAllToFront' | 'none';
```

**新增**: `services`, `bringAllToFront`, `none`（对齐 Rust 侧 `PredefinedMenuItemType` 枚举的全部变体）

### Step 5: 补齐 AcceleratorMatcher 剪贴板排除列表

> **现状**: Phase 11 已实现 `AcceleratorMatcher` (`accelerator_matcher.ets`) 和 `onKeyPreIme` 拦截。其中已有 `CLIPBOARD_ACCELERATORS` 排除了 `ctrl+c`, `ctrl+x`, `ctrl+v`, `ctrl+a`，但**缺少 `ctrl+z` (undo) 和 `ctrl+y` (redo)**。

**文件**: `openharmony-ability/native_ability/src/main/ets/helper/accelerator_matcher.ets`

当前代码 (`accelerator_matcher.ets:38-43`):
```typescript
const CLIPBOARD_ACCELERATORS: Set<string> = new Set([
  'ctrl+c',   // copy
  'ctrl+x',   // cut
  'ctrl+v',   // paste
  'ctrl+a',   // selectAll
]);
```

需要补齐:
```typescript
const CLIPBOARD_ACCELERATORS: Set<string> = new Set([
  'ctrl+c',   // copy
  'ctrl+x',   // cut
  'ctrl+v',   // paste
  'ctrl+a',   // selectAll
  'ctrl+z',   // undo  ← 新增
  'ctrl+y',   // redo  ← 新增
]);
```

**影响**: 缺少 `ctrl+z`/`ctrl+y` 意味着如果菜单项有 Undo/Redo accelerator，`onKeyPreIme` 会拦截并走 `execCommand("undo")` 而非 WebView 原生 undo/redo。功能上可能仍可工作，但绕过了 WebView 的原生处理（支持更丰富的 undo stack）。

**设计原则**：
- **剪贴板类** (Ctrl+C/V/X/A/Z/Y) → 不拦截 → WebView 原生处理（功能完整，支持富文本）
- **窗口类** (Ctrl+M minimize 等) → 拦截 → PredefinedActionExecutor 处理
- **自定义快捷键** → 拦截 → MenuEvent 发给开发者

### Step 6: 修复 Tray minimize 窗口闪烁 (minimizeWithRestoreGuard)

> **问题**: OHOS 桌面模式下，StatusBar 右键菜单关闭时系统自动恢复主窗口焦点 (WINDOW_ACTIVE 事件)，与 `win.minimize()` 竞态。导致窗口"闪缩还原"。maximize/fullscreen/copy 等不受影响（不改变前台状态）。

**文件**: `openharmony-ability/native_ability/src/main/ets/helper/menu.ets`

```typescript
private async minimizeWithRestoreGuard(win: window.Window | null): Promise<void> {
    if (!win) return;
    let consumed = false;
    const doMinimize = () => {
      if (!consumed) {
        consumed = true;
        try { win.off('windowEvent', onWindowEvent); } catch (_) {}
        win.minimize();
      }
    };
    const onWindowEvent = (eventType: window.WindowEventType) => {
      if (!consumed && eventType === window.WindowEventType.WINDOW_ACTIVE) {
        doMinimize();
      }
    };
    try { win.on('windowEvent', onWindowEvent); } catch (_) { win.minimize(); return; }
    setTimeout(doMinimize, 50);  // fallback if WINDOW_ACTIVE never fires
}
```

**关键设计点**：
- **不先 minimize 再 guard**：避免两次 minimize 造成的卡顿感
- **延迟到系统焦点恢复后再 minimize**：WINDOW_ACTIVE 触发时安全执行，只调用一次
- **50ms 兜底**：menu 路径不会触发焦点恢复，用 50ms setTimeout 直接 minimize（用户无感知）
- **一次性 guard**：`consumed` 标志确保只 minimize 一次，不会误触发后续正常交互
- **适用范围**：minimize / hide / close 三个操作，不影响 maximize / fullscreen

---

## 六、WebView 键盘快捷键与 Menu 点击行为对照表

| 操作 | 键盘快捷键 | 谁处理 | Menu 点击 | 谁处理 | 是否一致 |
|------|-----------|--------|-----------|--------|---------|
| Copy | Ctrl+C | WebView 原生 | pasteboard + getSelection | 基本一致（纯文本） |
| Cut | Ctrl+X | WebView 原生 | pasteboard + getSelection + delete | 基本一致 |
| Paste | Ctrl+V | WebView 原生 | pasteboard.getData + insertText | 基本一致（纯文本） |
| SelectAll | Ctrl+A | WebView 原生 | execCommand("selectAll") | 一致 |
| Undo | Ctrl+Z | WebView 原生 | execCommand("undo") | 一致 |
| Redo | Ctrl+Y | WebView 原生 | execCommand("redo") | 一致 |
| Minimize | Ctrl+M | Phase 11 拦截 | win.minimize() | 一致 |
| Fullscreen | — | Phase 11 拦截 | immersive maximize | 一致 |

**注意**: Menu 点击的 copy/paste 只处理纯文本，而 WebView 键盘快捷键支持富文本/图片。这是菜单点击路径的限制（Windows/macOS 的菜单 copy 也是纯文本操作），可以接受。

---

## 七、关键文件清单

| 文件 | 修改说明 |
|------|---------|
| `openharmony-ability/.../helper/menu.ets` | 重写剪贴板操作；添加全局 executor getter/setter；execute 添加 targetWindowId 参数；添加 minimizeWithRestoreGuard |
| `openharmony-ability/.../helper/menu_types.ets` | PredefinedType 补齐 services/bringAllToFront/none |
| `openharmony-ability/.../helper/accelerator_matcher.ets` | CLIPBOARD_ACCELERATORS 补齐 ctrl+z/ctrl+y |
| `openharmony-ability/.../ability/ArkHelper.ets` | executePredefinedAction 委托给 PredefinedActionExecutor（删除 60 行 switch/case） |
| `openharmony-ability/.../ability/NativeAbility.ets` | 调用 setPredefinedActionExecutor 存储实例 |
| `tray-icon/src/platform_impl/ohos/event.rs` | execute_predefined_action 补齐 clipboard/recover 转发 |
| ~~api demo `module.json5`~~ | ~~添加 `ohos.permission.READ_PASTEBOARD` 权限~~ — 已移除，见风险表 |

---

## 八、验证策略

### 手动测试（设备部署后）

| 测试 | 操作 | 预期结果 |
|------|------|---------|
| 键盘 Ctrl+C | WebView 选中文字 → Ctrl+C → 外部粘贴 | 文字出现（WebView 原生处理） |
| 键盘 Ctrl+V | 外部复制 → demo 输入框 → Ctrl+V | 文字插入（WebView 原生处理） |
| Menu Copy | 选中文字 → Edit → Copy → 外部粘贴 | 文字出现在剪贴板 |
| Menu Paste | 外部复制 → 输入框 → Edit → Paste | 文字插入输入框 |
| Menu Cut | 选中文字 → Edit → Cut → 外部粘贴 | 文字消失 + 出现在剪贴板 |
| Tray Copy | tray 菜单 → Copy | 与 Menu Copy 行为一致 |
| Tray About | tray 菜单 → About | AlertDialog 弹出 |
| Tray Recover | tray 菜单 → Recover | 窗口恢复正常 |
| Tray Minimize | tray 菜单 → Minimize | 主窗口最小化 |
| Tray Fullscreen | tray 菜单 → Fullscreen | 主窗口全屏（沉浸式） |

### 验证代码改动正确性

1. `ArkHelper.ets` 的 `executePredefinedAction` 不再有 switch/case，只有委托调用
2. `menu.ets` 的 `PredefinedActionExecutor.execute` 处理所有 20 种 predefined type
3. `event.rs` 的 match 包含所有 OHOS 支持的 predefined action
4. `menu_types.ets` 的 `PredefinedType` 包含所有 Rust 侧枚举变体的字符串映射

---

## 九、风险与注意事项

| 风险 | 影响 | 缓解 |
|------|------|------|
| `READ_PASTEBOARD` 是 `system_basic` 级受限权限 | 侧载安装失败 (error 9568289)。需要 AGC Profile 中包含此权限的 `allowed-acls` | 当前已从 `module.json5` 移除。paste 菜单点击静默 no-op。启用需：① AGC 申请 ACL 权限 ② 重新下载 Profile ③ 加回 module.json5 |
| `execCommand("insertText")` 也可能被 ArkWeb 废弃 | paste 菜单点击可能不工作 | 备选：`runJavaScript` 直接操作 DOM `activeElement.value` |
| `execCommand("delete")` 不可靠 | cut 的删除部分可能不工作 | 备选：`Range.deleteContents()` |
| Tray clipboard 需要 WebView controller 已初始化 | controller 未就绪时静默失败 | 添加 null check + warn 日志 |
| Menu 点击 copy 只支持纯文本 | 无法复制图片/富文本 | 可接受 — Windows/macOS 菜单 copy 也是纯文本 |
| Phase 11 AcceleratorMatcher 已排除 ctrl+c/x/v/a 但缺少 ctrl+z/y | WebView 原生 undo/redo 被绕过，改走 execCommand | Step 5 补齐 ctrl+z/y 到 CLIPBOARD_ACCELERATORS |
| `PredefinedActionExecutor` 全局单例生命周期 | Ability 重建时 executor 可能过期 | `setPredefinedActionExecutor` 在 `onCreate` 调用，跟随 Ability 生命周期 |
| 子窗口 WebView controller 获取方式 | WindowManager.getController 返回 RustWebviewNodeController，需要额外步骤获取 WebviewController | 当前 targetWindowId 主要为 0（主窗口），未来扩展时需确认 API |
| OHOS 桌面模式 StatusBar 菜单关闭触发焦点恢复 | Tray minimize 窗口"闪缩还原" | Step 6: `minimizeWithRestoreGuard` 延迟 minimize 到系统焦点恢复后，只调用一次 |

# Phase 14: OHOS Menu Remaining Features + Multi-Window 彻底实现

## Status: ✅ 代码实施完成，待设备回归验证

## Context

Phase 0-13 completed the core menu implementation and predefined item fixes for OHOS: Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu, clipboard pasteboard integration, Menu/Tray unification — all public API branches are in place. This document identifies the **remaining gaps**, with **multi-window support** as the primary focus.

Architecture chain: **tauri::menu** → **muda::platform_impl::ohos** → **openharmony-ability** (NAPI/Rust bridge) → **ArkTS** → **ArkUI**

---

## Architecture

### Repo Dependency Chain

```
┌──────────────┐    ┌───────────────────┐    ┌────────────────────────┐    ┌─────────┐    ┌───────┐
│  tauri       │───▶│  muda             │───▶│  openharmony-ability   │───▶│  ArkTS  │───▶│ ArkUI │
│  (上层 API)  │    │  (平台抽象层)     │    │  (NAPI / Rust 桥接)    │    │ (UI逻辑)│    │(渲染) │
└──────────────┘    └───────────────────┘    └────────────────────────┘    └─────────┘    └───────┘
  crates/tauri/       src/platform_impl/       crates/ability/src/menu/     native_ability/
  src/menu/           ohos/mod.rs              + crates/ability/src/lib.rs  src/main/ets/
```

### Data Flow

**菜单推送 (Rust → ArkTS)**:
```
tauri set_menu/set_text/set_checked/...
  → muda::Menu::refresh_menubar(window_id)
    → openharmony_ability::menu::set_menu_json(json, window_id)
      → MENU_CHANNEL.send(MenuRequest)
        → TSFN callback (Rust thread → ArkTS)
          → ArkTS onMenuRequest handler
            → menuDataCallbacks.get(windowId)?.onMenubarJson(json)
              → MenuBarComponent @State menubarJson → ArkUI 渲染
```

**用户操作 (ArkTS → Rust)**:
```
ArkUI MenuItem onClick
  → ArkTS MenuBarComponent onMenuItemClick(item, windowId)
    → NAPI emit_menu_event(menu_id, window_id)
      → MENU_EVENT_CHANNEL.send(menu_id)
        → muda start_event_listener receives
          → CHECK_ITEMS toggle (if check item)
          → MenuEvent::send(MenuEvent { id })
            → tauri menu plugin receives event
```

**Dark Mode 切换 (Phase 14 P1 新增)**:
```
tauri setTheme('dark')
  → tao::Window::set_theme(Some(Theme::Dark))
    → openharmony_ability NAPI set_color_mode("dark")
      → ArkTS UIAbility context setColorMode()
        → $r('sys.color.*') 系统资源自动切换
```

**NativeIcon 推送 (Phase 14 P3 新增)**:
```
tauri IconMenuItem::set_native_icon(NativeIcon::Home)
  → muda IconMenuItem::set_native_icon
    → MenuChild::set_native_icon → native_icon_to_ohos(Home) → Some("sys.symbol.ohos_home")
      → refresh_menubar → JSON 包含 "nativeIcon": "sys.symbol.ohos_home"
        → ArkTS RenderMenuItems → symbolStartIcon / startIcon
```

### Affected Repos Per Feature

| Repo / Module | P1 Dark Mode | P2 Multi-Window | P3 NativeIcon |
|---|---|---|---|
| **tao** (`src/window.rs`, `src/platform_impl/ohos/`) | ✅ 添加 OHOS `set_theme` 分支 | — | — |
| **muda** (`src/platform_impl/ohos/mod.rs`, `src/items/icon.rs`, `src/items/submenu.rs`) | — | — | ✅ 存储/映射/设置 native_icon |
| **tauri** (`crates/tauri/src/menu/icon.rs`, `crates/tauri/src/menu/submenu.rs`) | — | — | ✅ 添加 OHOS 分支到 `set_native_icon` |
| **openharmony-ability Rust** (`crates/ability/src/menu/types.rs`, `crates/ability/src/lib.rs`) | ✅ NAPI `set_color_mode` | — | ✅ MenuItemData 加 `native_icon` 字段 |
| **openharmony-ability ArkTS** (`native_ability/src/main/ets/`) | ✅ 替换硬编码颜色为系统资源 | ✅ 全局单例→per-window Map, AppStorage→@State, MenuManager windowId, MenuBarComponent 提取, 死代码清理 | ✅ RenderMenuItems 处理 `nativeIcon` |
| **资源文件** (`base/element/color.json`, `dark/element/color.json`) | ✅ 自定义暗色模式颜色 | — | — |
| **ArkTS 组件** (`MenuBarComponent.ets`, `MainPage.ets`, `FloatPage.ets`) | — | ✅ MenuBarComponent 提取（~280行自包含组件），MainPage/FloatPage 复用 | — |

### Not Affected

| Repo | Reason |
|------|--------|
| `tauri/crates/tauri-runtime` | 不直接涉及菜单渲染 |
| `tauri/examples` | 仅用于验证，不修改源码 |
| `muda/src/icon.rs` | `NativeIcon` enum 定义不变 |
| `muda/src/lib.rs` | `MenuEvent` 结构不变 (window_id 不实施) |
| `tray-icon` | Tray 只操作主窗口（OHOS 单 tray 限制），Phase 13 已正确 |

### 跨平台 Menubar 模型对照

| 平台 | Menubar 模型 | 子窗口行为 |
|------|-------------|-----------|
| **macOS** | 全局唯一，跟随 key window 的 responder chain | 共享同一 menubar |
| **Windows** | per-window `SetMenu(hwnd, hMenu)` | 每窗口独立 menubar |
| **OHOS** | per-window，每窗口独立 ArkUI 组件树（同 Windows） | 每窗口独立 menubar |

OHOS 的 Rust 侧已支持 per-window 推送（`refresh_menubar(window_id)` → `set_menu_json(json, window_id)`），问题在 ArkTS 侧：MainPage 只读 `::main` 的 AppStorage key，FloatPage 完全没有 menubar。

### TAURI_OHOS_DEVICE_TYPE 与 Menubar 渲染条件

OHOS 可以是 mobile 或 desktop 设备，由构建时环境变量 `TAURI_OHOS_DEVICE_TYPE` 控制：
- `TAURI_OHOS_DEVICE_TYPE=desktop` → `cfg(desktop)` 为 true → `is_desktop_device()` NAPI 返回 true
- `TAURI_OHOS_DEVICE_TYPE=mobile` (或未设置) → `cfg(mobile)` 为 true → `is_desktop_device()` 返回 false

当前 MainPage.ets 通过 `@StorageProp("__openharmony_ability_is_desktop__") isDesktop` 获取此标志，menubar 仅在 `isDesktop === true` 时渲染（line 328）。MenuBarComponent 提取后通过 `@Prop isDesktop` 传入，保留相同条件。

**影响范围**: 仅影响 menubar 可见性。Popup menu（右键菜单）和 accelerator 不受此限制——mobile 模式下仍支持 popup menu 和快捷键。

---

## Conclusion: What To Implement vs What Cannot

### ✅ To Implement (OHOS platform has the capability)

| Priority | Feature | OHOS Platform Basis |
|----------|---------|---------------------|
| **P1** | **Dark Mode** — Menubar currently hardcoded light colors, `set_theme` is no-op on OHOS | `Configuration.colorMode` (API 9+), `$r('sys.color.*')` system resources auto-switch light/dark |
| **P2** | **Multi-Window 彻底实现** — 全局单例、AppStorage 逻辑隔离、windowId 不转发、MenuEvent 硬编码 "main" | OHOS `LocalStorage` per-window 隔离, `WindowManager` 已有 per-window Map |
| **P3** | **NativeIcon partial mapping** — `set_native_icon()` currently silent no-op, discards the icon | `sys.symbol.*` / `sys.media.*` system icon resources; `symbolStartIcon` / `startIcon` on MenuItem |

### ❌ Cannot Implement (OHOS platform lacks the capability)

| Feature | Why |
|---------|-----|
| **HideOthers** predefined item | OHOS has no `NSApplication.hideOtherApplications()` equivalent. Apps don't have a "hidden" state. The window manager supports per-window minimize/show/hide but not cross-app "hide all others". Current handling (`enabled: false`) matches Windows/Linux behavior — correct. |
| **ShowAll** predefined item | Same reason — no cross-app "show all hidden apps" concept exists on OHOS. Current handling (`enabled: false`) is correct. |
| **NativeIcon complete mapping** (~39 variants) | `NativeIcon` enum has 56 macOS NSImage semantic names (`ColorPanel`, `IChatTheater`, `FlowView`, `MobileMe`, `FollowLinkFreestanding`, etc.). These are macOS-specific UI metaphors with no OHOS equivalent. Only ~17 variants can be mapped; the rest must remain no-op. |

### ⏭️ Not Needed (current implementation works)

| Feature | Why No Action |
|---------|--------------|
| **CHECK_ITEMS global state** | Menu IDs are generated by a global counter, making them process-unique. The event listener toggles by ID, so cross-window collisions are impossible. Works correctly. |

---

## 1. Dark Mode (CAN implement, requires Rust + ArkTS changes)

### Problem
Tauri example API app has a "Switch to dark mode" button that calls `setTheme('dark')`. The WebView content (HTML/CSS) correctly switches to dark mode, but **the menubar stays in light mode**.

### Root Cause A: `tao::Window::set_theme()` is a no-op on OHOS
`tao/src/window.rs:1202-1216` has an explicit `not(target_env = "ohos")` guard:
```rust
pub fn set_theme(&self, #[allow(unused)] theme: Option<Theme>) {
    #[cfg(all(
      any(windows, target_os = "linux", ..., target_os = "macos"),
      not(target_env = "ohos")  // ← OHOS excluded
    ))]
    self.window.set_theme(theme)
}
```
### Root Cause C: OHOS Window impl 缺少 `set_theme` 方法

`tao/src/platform_impl/ohos/mod.rs` 的 `Window` impl（line 579+）**完全没有 `set_theme` 方法**。即使移除 `window.rs` 的 `not(target_env = "ohos")` guard，`self.window.set_theme(theme)` 也会编译失败。需要先在 OHOS Window impl 中**新增**该方法。

### Root Cause D: `theme()` 硬编码返回 `Theme::Light`

`tao/src/platform_impl/ohos/mod.rs:829`:
```rust
pub fn theme(&self) -> Theme {
    Theme::Light  // ← 永远返回 Light，set_theme 后也不变
}
```

### Root Cause B: Menubar uses hardcoded light-mode colors
`MainPage.ets` `MenuBarRow()` (lines 227-273) uses hardcoded hex colors:
```typescript
.fontColor((item.enabled ?? true) ? '#333333' : '#999999')  // line 239
.backgroundColor(
  this.activeDropdownId === item.id ? '#E0E0E0' :           // line 245
  this.hoveredItemId === item.id ? '#EBEBEB' :              // line 246
  '#F5F5F5'                                                  // line 247
)
.backgroundColor('#F5F5F5')  // container row, line 271
```

**Note**: The right-click context menu (`MenuPopup.ets`) uses native ArkUI `Menu()`/`MenuItem()` components without hardcoded colors — it should auto-adapt to dark mode.

### OHOS Platform Capability
OHOS fully supports dark mode on desktop:
1. **`Configuration.colorMode`** (API 9+) — Can be set to force app color mode via `setColorMode()`
2. **ArkUI system resources** — `$r('sys.color.ohos_id_color_text_primary')` etc. auto-switch between light/dark
3. **Resource directories** — `base/element/color.json` + `dark/element/color.json` with same names auto-switch
4. **`@ohos.uiAppearance.getDarkMode()`** (API 20+) — Direct query for system dark mode setting

### Implementation Approach

**Step 1 (Rust — tao OHOS Window impl)**: 在 `platform_impl/ohos/mod.rs` 新增 `set_theme` 方法 + 修复 `theme()`:
- 添加 theme 状态存储（`AtomicU8` 或 `Mutex<Option<Theme>>`）
- 新增 `pub fn set_theme(&self, theme: Option<Theme>)` — 通过 NAPI 调用 ArkTS `setColorMode()`
- 修改 `pub fn theme()` — 返回存储的 theme 而非硬编码 `Theme::Light`
- 移除 `window.rs:1213` 的 `not(target_env = "ohos")` guard

> **注意**: 不是"移除 guard 就完事"，必须先新增 OHOS impl 方法，否则编译失败。

**Step 2 (ArkTS — MenuBarComponent.ets)**: Replace hardcoded colors with custom resources:
```typescript
// Before:
.fontColor((item.enabled ?? true) ? '#333333' : '#999999')
.backgroundColor('#F5F5F5')

// After (custom resources — base/element/color.json + dark/element/color.json):
.fontColor((item.enabled ?? true)
  ? $r('app.color.menubar_text')
  : $r('app.color.menubar_text_disabled'))
.backgroundColor($r('app.color.menubar_bg'))
```

Custom resources defined in `base/element/color.json` + `dark/element/color.json` (HAR + app entry dual deployment):
```json
// base: { "name": "menubar_bg", "value": "#F5F5F5" }
// dark:  { "name": "menubar_bg", "value": "#1A1A1A" }
```

**Step 3 (ArkTS — hover/active states)**: Replace `#E0E0E0` and `#EBEBEB` with system hover/press color resources or custom dark-adaptive resources.

### Files to Modify
- `tao/src/platform_impl/ohos/mod.rs` — 新增 `set_theme()` 方法 + 修复 `theme()` 硬编码 ✅
- `tao/src/window.rs:1202-1216` — 移除 `not(target_env = "ohos")` guard（依赖上一步先完成） ✅
- `openharmony-ability/` — 新增 NAPI `set_color_mode()` 桥接 ArkTS `setColorMode()` ✅
- `MenuBarComponent.ets` — 替换硬编码颜色为自定义资源 ✅
- `base/element/color.json` + `dark/element/color.json` — 自定义 menubar 颜色资源（HAR + app entry 双位置） ✅

---

## 2. Multi-Window 彻底实现 (CAN implement, comprehensive)

### 2.1 当前多窗口架构分析

```
┌───────────────────────────────────────────────────────────────────────────┐
│                        当前架构 (Phase 13 之后)                            │
├───────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  MainPage (windowId=0)                   FloatPage (windowId=N>0)        │
│  ┌─────────────────────────┐             ┌──────────────────────────┐    │
│  │ MenuBarRow (menubar)    │             │ DefaultXComponent        │    │
│  │ @StorageProp("...::main")│            │  (WebView only)          │    │
│  │                         │             │                          │    │
│  │ DefaultXComponent       │             │ ❌ 没有 menubar           │    │
│  │  (WebView)              │             │ ❌ 没有 popup menu        │    │
│  │                         │             │ ❌ 没有 accelerator       │    │
│  │ onKeyPreIme             │             │                          │    │
│  │  (accelerator_matcher)  │             │                          │    │
│  └─────────────────────────┘             └──────────────────────────┘    │
│           │                                         │                     │
│           ▼                                         ▼                     │
│  ┌─────────────────────────────────────────────────────────────┐         │
│  │                    GLOBAL SINGLETONS                         │         │
│  │                                                              │         │
│  │  globalMenuClickHandler ─────── overwritten by last window   │         │
│  │  globalRecoverFn ────────────── overwritten by last window   │         │
│  │  menuStateController ────────── single instance, shared      │         │
│  │  globalPopupCallback ────────── single callback              │         │
│  │  PredefinedActionExecutor ───── single instance (Phase 13)   │         │
│  │  AppStorage ─────────────────── global, logical isolation    │         │
│  └─────────────────────────────────────────────────────────────┘         │
└───────────────────────────────────────────────────────────────────────────┘
```

### 2.2 识别的六个多窗口问题

| # | 问题 | 严重度 | 位置 | 影响 |
|---|------|--------|------|------|
| MW-1 | `globalMenuClickHandler` 全局单例 | 🔴 高 | `MainPage.ets:14` | 多窗口菜单点击路由到最后一个注册的窗口 |
| MW-2 | `MenuManager.handleItemClick` 硬编码 `"main"` | 🔴 高 | `menu.ets:133` | MenuEvent 永远报告来自 main 窗口 |
| MW-3 | `MenuManager.popupFromJson` 不存储 windowId | 🟡 中 | `menu.ets:114` | executor 无法获知菜单源窗口，targetWindowId 永远为 0 |
| MW-4 | `menuStateController` / `globalPopupCallback` 死代码 | 🟢 低 | `menu_state.ets:82-95` | `menuStateController` 被 import 但从未调用方法；`setGlobalPopupCallback` / `triggerGlobalPopup` 无外部调用。应清理而非 per-window 化 |
| MW-5 | AppStorage 逻辑隔离而非真实隔离 | 🟢 低 | `MainPage.ets:44` | 当前 window label 唯一所以可工作，但架构不正确 |
| MW-6 | FloatPage 无 menubar/popup/accelerator | 🟡 中 | `FloatPage.ets` | 子窗口无菜单栏、无右键菜单、无快捷键 |

### 2.3 目标架构

```
┌───────────────────────────────────────────────────────────────────────────┐
│                        目标架构 (Phase 14 之后)                            │
├───────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  MainPage (windowId=0)                   FloatPage (windowId=N>0)        │
│  ┌─────────────────────────┐             ┌──────────────────────────┐    │
│  │ MenuBarComponent        │             │ MenuBarComponent         │    │
│  │  ├─ menubar 渲染        │             │  ├─ menubar 渲染         │    │
│  │  ├─ popup menu          │             │  ├─ popup menu           │    │
│  │  ├─ RenderMenuItems     │             │  ├─ RenderMenuItems      │    │
│  │  ├─ 图标管理            │             │  ├─ 图标管理             │    │
│  │  ├─ accelerator matcher │             │  ├─ accelerator matcher  │    │
│  │  └─ handler 注册/清理   │             │  └─ handler 注册/清理    │    │
│  │                         │             │                          │    │
│  │ DefaultXComponent       │             │ DefaultXComponent        │    │
│  │  (WebView)              │             │  (WebView)               │    │
│  └──────────┬──────────────┘             └──────────┬───────────────┘    │
│             │                                       │                     │
│             ▼                                       ▼                     │
│  ┌─────────────────────────────────────────────────────────────┐         │
│  │                 PER-WINDOW REGISTRY                          │         │
│  │                                                              │         │
│  │  windowMenuClickHandlers: Map<windowId, handler>             │         │
│  │  windowRecoverFns: Map<windowId, fn>                         │         │
│  │  menuDataCallbacks: Map<string, callback>                    │         │
│  │  PredefinedActionExecutor ── shared, targetWindowId dynamic  │         │
│  │  @State (per-component) ── no AppStorage for menu data       │         │
│  │                                                              │         │
│  │  (死代码已清理: menuStateController, globalPopupCallback)    │         │
│  └─────────────────────────────────────────────────────────────┘         │
└───────────────────────────────────────────────────────────────────────────┘
```

### 2.4 实施步骤

#### MW-1: 全局单例 → Per-Window Registry

**文件**: `MainPage.ets`

**当前代码** (`MainPage.ets:14-22`):
```typescript
let globalMenuClickHandler: ((item: MenuItemData) => void) | null = null;
let globalRecoverFn: (() => void) | null = null;

export function setMenuClickHandler(handler: (item: MenuItemData) => void): void {
  globalMenuClickHandler = handler;
}
export function setRecoverFn(fn: () => void): void {
  globalRecoverFn = fn;
}
```

**目标代码**:
```typescript
// Per-window handler registry
const windowMenuClickHandlers = new Map<number, (item: MenuItemData) => void>();
const windowRecoverFns = new Map<number, () => void>();

export function setMenuClickHandler(windowId: number, handler: (item: MenuItemData) => void): void {
  windowMenuClickHandlers.set(windowId, handler);
}
export function setRecoverFn(windowId: number, fn: () => void): void {
  windowRecoverFns.set(windowId, fn);
}

// Internal helpers for UI event dispatch
function getMenuClickHandler(windowId: number): ((item: MenuItemData) => void) | undefined {
  return windowMenuClickHandlers.get(windowId);
}
function getRecoverFn(windowId: number): (() => void) | undefined {
  return windowRecoverFns.get(windowId);
}
```

**影响范围**: 所有调用 `globalMenuClickHandler(item)` 的位置 (`MainPage.ets:158,175,183,348`) 需要改为 `getMenuClickHandler(this.windowId)?.(item)` 或使用组件自身的 windowId。MainPage 使用 `windowId = 0`，FloatPage 使用 `this.windowId`。

**额外注意**: `menu.ets` 中 `MenuManager` 构造函数也调用了 `setMenuClickHandler`:
```typescript
// menu.ets:209 — 当前代码
constructor(executor, emitMenuEventFn) {
    setMenuClickHandler((item) => {   // ← 无 windowId 参数
        this.handleItemClick(item);
    });
}
```
MW-1 修改 `setMenuClickHandler` 签名后，此处需改为 `setMenuClickHandler(0, handler)` 或在 MenuManager 构造时传入 windowId。但 MW-6 的 MenuBarComponent 提取后，MenuManager 构造函数中的 `setMenuClickHandler` 调用将被移除（改为 MenuBarComponent.aboutToAppear 中注册），所以此处不需要改。

**清理**: 窗口销毁时需要从 Map 中移除对应 handler，防止内存泄漏。FloatPage 的 `aboutToDisappear` 中调用:
```typescript
windowMenuClickHandlers.delete(this.windowId);
windowRecoverFns.delete(this.windowId);
```

#### MW-2: MenuEvent 硬编码 "main" → 动态 windowId

**文件**: `menu.ets` (MenuManager 类)

**当前代码** (`menu.ets:129-135`):
```typescript
handleItemClick(item: MenuItemData): void {
    if (item.type === 'predefined' && item.predefinedType) {
      this.executor.execute(item.predefinedType, item.aboutMetadata);
    } else {
      this.emitMenuEventFn(item.id, "main");  // ← 硬编码
    }
}
```

**目标代码**:
```typescript
handleItemClick(item: MenuItemData): void {
    const windowId = this.currentWindowId ?? "main";
    if (item.type === 'predefined' && item.predefinedType) {
      const targetWinId = this.currentWindowIdNumeric ?? 0;
      this.executor.execute(item.predefinedType, item.aboutMetadata, targetWinId);
    } else {
      this.emitMenuEventFn(item.id, windowId);
    }
}
```

**依赖**: `currentWindowId` 和 `currentWindowIdNumeric` 由 MW-3 在 `popupFromJson` 中存储。

#### MW-3: MenuManager 存储并转发 windowId

**文件**: `menu.ets` (MenuManager 类)

**当前代码** (`menu.ets:114-127`):
```typescript
popupFromJson(jsonData: string, x?: number, y?: number, windowId?: string): void {
    try {
      const wid = windowId ?? "main";
      AppStorage.setOrCreate("__openharmony_ability_menu_shown__::" + wid, false);
      // ... 使用 wid 但不存储到实例
    }
}
```

**目标代码**:
```typescript
private currentWindowId: string = "main";
private currentWindowIdNumeric: number = 0;

popupFromJson(jsonData: string, x?: number, y?: number, windowId?: string): void {
    try {
      const wid = windowId ?? "main";
      this.currentWindowId = wid;
      this.currentWindowIdNumeric = wid === "main" ? 0 : parseInt(wid, 10) || 0;
      AppStorage.setOrCreate("__openharmony_ability_menu_shown__::" + wid, false);
      // ... rest unchanged
    }
}
```

**说明**: `popupFromJson` 是 popup menu (右键菜单) 的入口。存储 windowId 后，`handleItemClick` 可以正确传递给 executor 和 emitMenuEventFn。

**注意**: menubar 的菜单点击不经过 `popupFromJson`，而是直接调用 `globalMenuClickHandler`。menubar 的 windowId 需要在 MenuManager 构造时设置或由调用方传入。当前 menubar 只在 MainPage (windowId=0) 渲染，所以默认为 "main"/0 是正确的。

> **MW-6 后变化**: MenuBarComponent 的 aboutToAppear 中注册 per-window handler，menubar 点击通过 MenuBarComponent 内部的 windowId 参数分发，不再依赖 MenuManager 存储 windowId。

#### MW-4: 清理 `menuStateController` / `globalPopupCallback` 死代码

**文件**: `menu_state.ets`, `NativeAbility.ets`

**问题**: 审计确认以下代码为死代码（有 export/import 但从未调用任何方法）：

| 代码 | 定义位置 | 引用 | 状态 |
|------|---------|------|------|
| `menuStateController` | `menu_state.ets:82` | `NativeAbility.ets:11` import 但**从未调用** `.showMenu()`/`.hideMenu()`/`.subscribe()` | 死代码 |
| `setGlobalPopupCallback` | `menu_state.ets:87` | 无外部 import/调用 | 死代码 |
| `triggerGlobalPopup` | `menu_state.ets:91` | 无外部 import/调用 | 死代码 |

当前 popup menu 走的是完全独立的路径：`menu.ets popupFromJson` → AppStorage → `MainPage.ets @StorageProp` → `bindMenu()`，不经过 `menuStateController`。

**处理方式**: 删除这些死代码，保留 `MenuStateController` 类和 `MenuState` 接口（未来可能需要），但移除全局实例和 popup callback 相关函数。

```typescript
// 保留: MenuStateController class, MenuState interface
// 删除: export const menuStateController = new MenuStateController();
// 删除: globalPopupCallback, setGlobalPopupCallback, triggerGlobalPopup

// NativeAbility.ets: 移除 import { menuStateController }
```

**注意**: 如果未来需要 per-window popup 状态管理，应在 MenuBarComponent 内部管理（@State），而非恢复全局单例模式。

#### MW-5: AppStorage → @State 迁移

**文件**: `MainPage.ets`, `MenuBarComponent.ets`, `NativeAbility.ets`, `menu.ets`, `ArkHelper.ets`

**问题**: `@StorageProp` 需要编译时常量 key，无法动态绑定不同 windowId。AppStorage 是全局单例。

**需要迁移的完整 AppStorage key 清单**（审计后补充）：

| Key | 读取方 | 写入方 | 迁移目标 |
|-----|--------|--------|---------|
| `__openharmony_ability_menubar_json__::main` | MainPage.ets:44 `@StorageProp` | NativeAbility.ets:204 `onMenuRequest` | callback → MenuBarComponent `@State menubarJson` |
| `__openharmony_ability_menubar_visible__::main` | MainPage.ets:45 `@StorageProp` | NativeAbility.ets:190, menu.ets:167,183, ArkHelper.ets:526, MainPage.ets:354 | callback → MenuBarComponent `@State menubarVisible` |
| `__openharmony_ability_menu_shown__::main` | MainPage.ets:34 `@StorageProp` | menu.ets:168,217,222, MainPage.ets:369 | callback → MenuBarComponent `@State menuShown` |
| `__openharmony_ability_menu_json__::main` | MainPage.ets:37 `@Watch @StorageProp` | menu.ets:218 `popupFromJson` | callback → MenuBarComponent `@State menuJson` |
| `__openharmony_ability_menu_x__::main` | MainPage.ets:35 `@StorageProp` | menu.ets:219 `popupFromJson` | callback → MenuBarComponent `@State menuX` |
| `__openharmony_ability_menu_y__::main` | MainPage.ets:36 `@StorageProp` | menu.ets:220 `popupFromJson` | callback → MenuBarComponent `@State menuY` |
| `__openharmony_ability_is_desktop__` | MainPage.ets:43 `@StorageProp` | NativeAbility.ets:131 `setupMenuPopup` | `@Prop` 传入 MenuBarComponent（无需 per-window） |

**双重写入问题**: `fullscreen`/`recover` 中 menubar 可见性通过两条路径更新：
```
menu.ets fullscreen:
  → AppStorage.setOrCreate("::main", false)       ← 直接写（路径 A）
  → notifyMenubarVisibilityFn("main", false)
    → primaryModule.notifyMenubarVisibility(...)   ← Rust NAPI
      → onMenuRequest({ visible: false })
        → AppStorage.setOrCreate("::main", false)  ← 回调又写一次（路径 B）
```
迁移后应**只保留路径 B**（callback），移除路径 A 的 AppStorage 直接写入。

**目标**: 所有 `@StorageProp` 改为 `@State`，数据通过 callback 直接推送。

**回调注册模式**（与 MW-1 一致）:
```typescript
// menu.ets 中新增
const menuDataCallbacks = new Map<string, MenuDataCallback>();

interface MenuDataCallback {
  onMenubarJson?: (json: string) => void;
  onMenubarVisible?: (visible: boolean) => void;
  onMenuPopup?: (json: string, x: number, y: number) => void;
  onMenuShown?: (shown: boolean) => void;
}

export function registerMenuDataCallback(windowId: string, callback: MenuDataCallback): void {
  menuDataCallbacks.set(windowId, callback);
}

// MenuBarComponent.aboutToAppear 中
registerMenuDataCallback(String(this.windowId), {
  onMenubarJson: (json) => { this.menubarJson = json; },
  onMenubarVisible: (visible) => { this.menubarVisible = visible; },
  onMenuPopup: (json, x, y) => {
    this.menuJson = json; this.menuX = x; this.menuY = y;
  },
  onMenuShown: (shown) => { this.menuShown = shown; },
});
```

**NativeAbility.ets 修改**: `onMenuRequest` handler 改用 callback registry 而非 AppStorage:
```typescript
primaryModule.onMenuRequest((data) => {
    const windowId = data.windowId ?? "main";
    const callback = menuDataCallbacks.get(windowId);
    if (data.visible !== undefined) {
        callback?.onMenubarVisible?.(data.visible);
        primaryModule.notifyMenubarVisibility(windowId, data.visible);
        return;
    }
    if (data.x !== undefined && data.y !== undefined) {
        callback?.onMenuPopup?.(data.jsonData, data.x, data.y);
        // popup 显示通过 onMenuPopup 设置 json+x+y，再由 MenuBarComponent 内部 bindMenu
        callback?.onMenuShown?.(true);
        return;
    }
    callback?.onMenubarJson?.(data.jsonData);
});
```

**menu.ets fullscreen/recover 修改**: 移除 AppStorage 直接写入，只通过 callback 更新:
```typescript
case 'fullscreen':
    // 移除: AppStorage.setOrCreate("__openharmony_ability_menubar_visible__::main", false);
    // 移除: AppStorage.setOrCreate("__openharmony_ability_menu_shown__::main", false);
    this.notifyMenubarVisibilityFn?.("main", false);  // ← 保留，通过 Rust 回调更新
    ...
case 'recover':
    // 移除: AppStorage.setOrCreate("__openharmony_ability_menubar_visible__::main", true);
    this.notifyMenubarVisibilityFn?.("main", true);
    ...
```

**popup menu 迁移**: `popupFromJson` 中的 AppStorage 写入改为 callback:
```typescript
popupFromJson(jsonData: string, x?: number, y?: number, windowId?: string): void {
    const wid = windowId ?? "main";
    const callback = menuDataCallbacks.get(wid);
    callback?.onMenuPopup?.(jsonData, x ?? 0, y ?? 0);
    setTimeout(() => { callback?.onMenuShown?.(true); }, 0);
}
```

#### MW-6: MenuBarComponent 完整提取 + FloatPage 支持

**文件**: 新建 `MenuBarComponent.ets`，修改 `MainPage.ets` 和 `FloatPage.ets`

**当前**: MainPage.ets 包含所有 menu 相关逻辑（menubar 渲染、popup menu、accelerator、图标生命周期管理），共 ~280 行。FloatPage 完全没有 menu 支持。

**目标**: 提取一个**自包含的 `MenuBarComponent`**，包含以下全部功能：

| 功能 | 当前所在 (MainPage.ets) | 提取到 MenuBarComponent |
|------|------------------------|------------------------|
| Menubar 渲染 (MenuBarRow) | lines 226-273 | ✅ `@Builder MenuBarRow()` |
| Popup menu 渲染 (MenuContent) | lines 220-224 | ✅ `@Builder MenuContent()` |
| RenderMenuItems (递归子菜单) | lines 141-191 | ✅ `@Builder RenderMenuItems()` |
| SubmenuContent | lines 212-217 | ✅ `@Builder SubmenuContent()` |
| MenubarDropdownContent | lines 275-280 | ✅ `@Builder MenubarDropdownContent()` |
| 图标生命周期管理 (prepareIcons/cleanupStaleIcons/collectIconIds) | lines 93-138 | ✅ 内部方法 |
| menuItems/menubarItems 解析 | lines 52-91 | ✅ `@Watch` 回调 |
| Accelerator matcher | lines 50, 345-352 | ✅ `onKeyPreIme` 处理 |
| handler 注册/清理 | MainPage.aboutToAppear/Disappear | ✅ `aboutToAppear`/`aboutToDisappear` |
| Popup bindMenu | lines 361-371 | ✅ 内部 1×1 Column |
| Menubar 可见性控制 | line 328 | ✅ `@State menubarVisible` |
| ESC 恢复 menubar | lines 353-357 | ✅ `onKeyPreIme` |

**MenuBarComponent 接口设计**:

```typescript
// 新建 MenuBarComponent.ets
@Component
export struct MenuBarComponent {
  // 窗口标识（MainPage=0, FloatPage=N>0）
  @Prop windowId: number = 0;
  // 是否为桌面设备（来自 TAURI_OHOS_DEVICE_TYPE，由父组件传入）
  @Prop isDesktop: boolean = false;

  // 以下为 @State，通过 callback 从 Rust/MenuManager 推送更新
  @State menubarJson: string = "[]";
  @State menubarVisible: boolean = true;
  @State menuShown: boolean = false;
  @State menuJson: string = "[]";
  @State menuX: number = 0;
  @State menuY: number = 0;

  // 内部 UI 状态
  @State private menubarItems: MenuItemData[] = [];
  @State private menuItems: MenuItemData[] = [];
  @State private activeDropdownId: string = "";
  @State private activeDropdownShown: boolean = false;
  @State private hoveredItemId: string = "";
  private iconPixelMaps: Map<string, image.PixelMap> = new Map();
  private acceleratorMatcher: AcceleratorMatcher = new AcceleratorMatcher();

  aboutToAppear(): void {
    // 注册 per-window 数据回调
    registerMenuDataCallback(String(this.windowId), {
      onMenubarJson: (json) => { this.menubarJson = json; },
      onMenubarVisible: (visible) => { this.menubarVisible = visible; },
      onMenuPopup: (json, x, y) => {
        this.menuJson = json; this.menuX = x; this.menuY = y;
      },
      onMenuShown: (shown) => { this.menuShown = shown; },
    });
    // 注册 per-window 点击处理
    setMenuClickHandler(this.windowId, (item) => {
      this.handleMenuItemClick(item);
    });
  }

  aboutToDisappear(): void {
    // 清理 per-window 注册
    unregisterMenuDataCallback(String(this.windowId));
    unregisterMenuClickHandler(this.windowId);
    this.releaseAllIcons();
  }

  build() {
    Stack() {
      Column() {
        // Menubar: 仅在桌面模式 + 有菜单项 + 可见时渲染
        if (this.isDesktop && this.menubarItems.length > 0 && this.menubarVisible) {
          this.MenuBarRow()
        }
        // WebView 插槽（由父组件通过 @BuilderParam 提供）
      }
      .onKeyPreIme((event: KeyEvent) => {
        // Accelerator 匹配
        if (event.type === KeyType.Down && this.acceleratorMatcher.matches(event)) {
          const matchedItem = this.acceleratorMatcher.getMatchedItem();
          if (matchedItem) { this.handleMenuItemClick(matchedItem); }
          return true;
        }
        // ESC 恢复 menubar（全屏后按 ESC 恢复）
        if (event.type === KeyType.Down && event.keyCode === KeyCode.KEYCODE_ESCAPE
            && !this.menubarVisible) {
          this.menubarVisible = true;
          notifyMenubarVisibility(String(this.windowId), true);
          return true;
        }
        return false;
      })

      // Popup menu anchor (1×1 invisible)
      Column()
        .width(1).height(1).position({ x: 0, y: 0 })
        .bindMenu(this.menuShown, this.MenuContent, {
          anchorPosition: { x: this.menuX, y: this.menuY },
          showInSubWindow: true,
          onWillDisappear: () => { this.menuShown = false; }
        })
    }
  }

  private handleMenuItemClick(item: MenuItemData): void {
    // 委托给 MenuManager（predefined → executor, 其他 → emitMenuEvent）
    // 使用 this.windowId 标识来源窗口
    menuManager.handleItemClick(item, this.windowId);
  }
}
```

**TAURI_OHOS_DEVICE_TYPE 上下文**:

OHOS 可以是 mobile 或 desktop 设备，由 `TAURI_OHOS_DEVICE_TYPE` 环境变量控制（构建时通过 `cfg(desktop)` / `cfg(mobile)` 判断）。`is_desktop_device()` NAPI 暴露给 ArkTS。当前 MainPage.ets:328 的条件 `if (this.isDesktop && ...)` 确保 **menubar 只在桌面模式渲染**。MenuBarComponent 通过 `@Prop isDesktop` 接收此标志，保留相同行为：

- `TAURI_OHOS_DEVICE_TYPE=desktop` → `isDesktop=true` → menubar 渲染
- `TAURI_OHOS_DEVICE_TYPE=mobile` (或未设置) → `isDesktop=false` → menubar 不渲染

此标志是全局的（设备级别），不需要 per-window 隔离。

**MainPage.ets 提取后简化为**:

```typescript
@Entry({ routeName: RouteName })
@Component
struct MainPage {
  @StorageProp("moduleName") moduleNames: string | string[] = "";
  @StorageProp("loadMode") loadMode: "async" | "sync" = "async";
  @State private nativeModule: ESObject | null = null;
  @State private resolvedModuleName: string = "";
  @StorageProp("__openharmony_ability_is_desktop__") isDesktop: boolean = false;

  async aboutToAppear(): Promise<void> {
    // 加载 Native Module（主窗口专属）
    // ... 保持不变
  }

  build() {
    Column() {
      MenuBarComponent({ windowId: 0, isDesktop: this.isDesktop }) {
        // WebView 内容
        if (this.resolvedModuleName !== "") {
          DefaultXComponent({ moduleName: this.resolvedModuleName, windowId: 0 })
        } else {
          Text("Loading Main Window...")
        }
      }
    }
  }
}
```

**FloatPage.ets 修改**:

```typescript
@Entry({ routeName: RouteName, storage: LocalStorage.getShared() })
@Component
struct FloatPage {
  @LocalStorageProp('windowId') windowId: number = 0;
  @StorageProp("__openharmony_ability_is_desktop__") isDesktop: boolean = false;
  // ... windowClass, offsetX, offsetY 保持不变

  build() {
    Stack({ alignContent: Alignment.TopEnd }) {
      Column() {
        MenuBarComponent({ windowId: this.windowId, isDesktop: this.isDesktop }) {
          DefaultXComponent({ windowId: this.windowId })
            .width('100%').height('100%');
        }
      }
      // 拖拽条 + 关闭按钮（保持不变）
    }
  }
}
```

**注意**: FloatPage 是否需要 menubar 取决于 tauri 是否为子窗口设置了菜单。如果 `set_menu()` 只给主窗口设置了菜单，FloatPage 的 `menubarJson` 为空，menubar 不渲染。如果开发者给子窗口也设置了菜单 (`window.set_menu()`)，则对应的 JSON 通过 callback 推送到对应 windowId 的 MenuBarComponent。

### 2.5 实施优先级

| 子项 | 优先级 | 理由 | 依赖 |
|------|--------|------|------|
| MW-1: 全局单例 → Map | 🔴 P1 | 不修复则多窗口菜单点击路由错误 | — |
| MW-2: MenuEvent "main" | 🔴 P1 | 不修复则开发者无法区分事件来源 | MW-3 |
| MW-3: MenuManager windowId 转发 | 🔴 P1 | 不修复则 executor 无法操作源窗口 | — |
| MW-4: 死代码清理 | 🟢 P3 | 不影响功能，但清理后可减少维护负担 | — |
| MW-5: AppStorage → callback | 🟡 P2 | 当前可工作但架构不正确，与 MW-6 MenuBarComponent 同步实施 | MW-1 |
| MW-6: MenuBarComponent 提取 + FloatPage | 🔴 P1 | 核心重构：提取自包含组件，MainPage/FloatPage 统一复用 | MW-1, MW-5 |

### 2.6 跨平台对照

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                  多窗口菜单行为 — 各平台对照                                   │
├──────────────┬───────────────────┬───────────────────┬───────────────────────┤
│ 能力         │ Windows           │ macOS             │ OHOS (Phase 14 后)    │
├──────────────┼───────────────────┼───────────────────┼───────────────────────┤
│ 每窗口独立   │ ✅ 每窗口独立     │ ❌ 全局 menubar   │ ✅ Per-window Map     │
│ menubar      │  HWND + subclass  │  跟随 key window  │  + @State             │
├──────────────┼───────────────────┼───────────────────┼───────────────────────┤
│ 菜单事件     │ ✅ hwnd 在        │ ✅ NSWindow 在    │ ✅ windowId 在        │
│ 来源识别     │  message 中       │  event 中         │  MenuBarComponent 中  │
├──────────────┼───────────────────┼───────────────────┼───────────────────────┤
│ Popup menu   │ ✅ TrackPopupMenu │ ✅ NSMenu popup   │ ✅ per-window         │
│ 目标窗口     │  (hwnd owner)     │  (位置指定)       │  callback + bindMenu  │
├──────────────┼───────────────────┼───────────────────┼───────────────────────┤
│ Predefined   │ ✅ 操作 menu      │ ✅ selector →     │ ✅ targetWindowId     │
│ action 目标  │  owner 窗口       │  key window       │  → WindowManager     │
├──────────────┼───────────────────┼───────────────────┼───────────────────────┤
│ 窗口销毁     │ ✅ subclass 自动  │ ✅ NSWindow       │ ✅ Map.delete         │
│ 清理         │  清理             │  dealloc          │  (aboutToDisappear)   │
├──────────────┼───────────────────┼───────────────────┼───────────────────────┤
│ Mobile 模式  │ N/A (仅桌面)     │ N/A (仅桌面)     │ ✅ TAURI_OHOS_DEVICE  │
│              │                   │                   │  _TYPE 控制 menubar   │
│              │                   │                   │  可见性               │
└──────────────┴───────────────────┴───────────────────┴───────────────────────┘
```

### Files to Modify

| 文件 | 修改说明 | 子项 |
|------|---------|------|
| (新文件) `MenuBarComponent.ets` | 从 MainPage 提取的自包含菜单组件：menubar 渲染 + popup menu + RenderMenuItems + 图标生命周期 + accelerator + handler 注册/清理。~280 行 | MW-6 |
| `MainPage.ets` | 移除全部 menu 逻辑（~280 行），仅保留 Native Module 加载 + WebView + `MenuBarComponent({ windowId: 0 })` | MW-1, MW-5, MW-6 |
| `FloatPage.ets` | 添加 `MenuBarComponent({ windowId: this.windowId })`，支持子窗口 menubar/popup/accelerator | MW-6 |
| `menu.ets` | MenuManager `handleItemClick` 接受 windowId 参数; 新增 `registerMenuDataCallback`/`unregisterMenuDataCallback`; `popupFromJson` 改用 callback 替代 AppStorage | MW-2, MW-3, MW-5 |
| `menu_state.ets` | 删除死代码：`menuStateController` 全局实例、`globalPopupCallback`、`setGlobalPopupCallback`、`triggerGlobalPopup` | MW-4 |
| `NativeAbility.ets` | `onMenuRequest` handler 改用 `menuDataCallbacks` registry; 移除 `menuStateController` import; `setupMenuPopup` 适配 callback 架构 | MW-5 |

---

## 3. NativeIcon Support (PARTIALLY implementable)

### Problem
`set_native_icon()` on both `IconMenuItem` and `Submenu` only has `#[cfg(target_os = "macos")]` branches — silent no-op on OHOS. `MenuChild::new_native_icon()` in `muda/src/platform_impl/ohos/mod.rs:246` accepts `_native_icon: Option<NativeIcon>` but **discards it** (stores `icon: None`).

### OHOS Platform Capability
OHOS provides two system icon mechanisms:
- **Symbol icons**: `sys.symbol.*` resources via `SymbolGlyphModifier` (API 12+), used as `symbolStartIcon` on MenuItem
- **Media icons**: `sys.media.*` resources via `ResourceStr`, used as `startIcon` on MenuItem

Both can display system icons on menu items.

### Why Only Partial
`NativeIcon` enum (`muda/src/icon.rs:175`) has 56 variants that are **macOS NSImage semantic names**. These are macOS-specific UI metaphors — OHOS has no equivalent for most. Only a subset can be reasonably mapped:

| NativeIcon variant | OHOS mapping | Notes |
|---|---|---|
| `Add` | `sys.media.ohos_ic_public_add` | Good match |
| `Home` | `sys.symbol.ohos_home` | Good match |
| `Info` | `sys.symbol.ohos_info` | Good match |
| `Folder` | `sys.symbol.ohos_folder` | Good match |
| `Network` | `sys.media.ohos_ic_public_network` | Good match |
| `GoLeft` | `sys.symbol.ohos_arrow_left` | Good match |
| `GoRight` | `sys.symbol.ohos_arrow_right` | Good match |
| `Refresh` | `sys.symbol.ohos_refresh` | Good match |
| `LockLocked` | `sys.media.ohos_ic_public_lock` | Good match |
| `LockUnlocked` | `sys.media.ohos_ic_public_unlock` | Good match |
| `Bluetooth` | `sys.symbol.ohos_bluetooth` | Good match |
| `Computer` | `sys.symbol.ohos_computer` | Good match |
| `Share` | `sys.symbol.ohos_share` | Good match |
| `TrashEmpty` | `sys.symbol.ohos_trash` | Good match |
| `TrashFull` | `sys.symbol.ohos_trash` | Good match |
| `User` | `sys.symbol.ohos_account` (unconfirmed) | Approximate, needs verification |
| `StatusAvailable` | `sys.symbol.ohos_check_circle` | Approximate |
| `ColorPanel` | **None** | macOS-only concept |
| `IChatTheater` | **None** | macOS-only concept |
| `FlowView` | **None** | macOS-only concept |
| `MobileMe` | **None** | macOS-only concept |
| `FollowLinkFreestanding` | **None** | macOS-only concept |
| ... (~34 more unmappable) | **None** | |

### Implementation Approach

**Step 1**: Add `native_icon: Option<String>` field to `MenuChild` in `muda/src/platform_impl/ohos/mod.rs`

**Step 2**: Create `native_icon_to_ohos(NativeIcon) -> Option<&'static str>` mapping function. Unmappable variants return `None`.

**Step 3**: In `new_native_icon()`, store the mapped value instead of discarding. Add `set_native_icon()` method on OHOS `MenuChild`.

**Step 4**: Add `native_icon: Option<String>` field to `MenuItemData` in `openharmony-ability/src/menu/types.rs`, serialize in `to_menu_item_data()`.

**Step 5**: Add OHOS branch to `set_native_icon()` in:
- `muda/src/items/icon.rs:196`
- `muda/src/items/submenu.rs:254`
- `tauri/crates/tauri/src/menu/icon.rs:322`
- `tauri/crates/tauri/src/menu/submenu.rs:660`

**Step 6**: In ArkTS, add `nativeIcon?: string` to `MenuItemData` interface in `menu_types.ets`, then use it in MenuBarComponent's `RenderMenuItems` to set `symbolStartIcon` or `startIcon` on `MenuItem`.

**What CANNOT be done**: Complete 1:1 mapping of all 56 macOS `NativeIcon` variants. Unmappable variants (~39) remain no-op (same as Windows/Linux behavior).

### Files to Modify
- `muda/src/platform_impl/ohos/mod.rs` — MenuChild struct, `new_native_icon`, add `set_native_icon`
- `muda/src/items/icon.rs` — Add OHOS branch to `set_native_icon`
- `muda/src/items/submenu.rs` — Add OHOS branch to `set_native_icon`
- `tauri/crates/tauri/src/menu/icon.rs:322` — Add OHOS branch
- `tauri/crates/tauri/src/menu/submenu.rs:660` — Add OHOS branch
- `openharmony-ability/crates/ability/src/menu/types.rs` — Add `native_icon` field to MenuItemData
- `openharmony-ability/native_ability/src/main/ets/helper/menu_types.ets` — Add `nativeIcon?: string` to `MenuItemData` interface
- `MenuBarComponent.ets` — `RenderMenuItems` 处理 `nativeIcon` 字段（symbolStartIcon / startIcon）

---

## Implementation Priority Order

| Priority | Feature | Status |
|----------|---------|--------|
| **P1** | Dark Mode (#1) | ✅ 代码完成 + 设备验证通过 |
| **P2** | Multi-Window 彻底实现 (#2) | ✅ 代码完成，待多窗口设备回归验证 |
| **P3** | NativeIcon partial mapping (#3) | ✅ 代码完成，待设备验证 |
| Skip | HideOthers/ShowAll | ⏭️ Platform limitation, correctly handled |
| Skip | CHECK_ITEMS | ⏭️ Functionally correct |

## Verification Plan

1. **Dark Mode**: Use tauri example API app (`examples/api/src-tauri`), click "Switch to dark mode", verify menubar background and text colors change. Verify right-click context menu also adapts.
2. **Multi-Window**:
   - Open two windows with different menus, verify click handlers route to correct window
   - Trigger menu events from different windows, verify event source windowId is correct
   - Popup menu from sub-window, verify it appears in the correct window
   - Close sub-window, verify handler cleanup (no memory leak)
   - Set menu on sub-window, verify menubar renders in FloatPage
3. **NativeIcon**: Unit test mapping function. Create test menu with native icons, verify JSON serialization includes mapped OHOS resource names. Verify unmappable variants produce `None`.
4. **Manual test**: Use Phase 12 test design (M1-M19 buttons) to verify all menu operations in multi-window scenarios.

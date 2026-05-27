# tauri::menu 模块 OHOS 适配设计方案

更新时间: 2026-05-21

> **实现文档**：详见 [README.md](README.md) 及 [impl/](impl/) 目录下的 9 个阶段设计文档。
> **参考文档**：详见 [reference/](reference/) 目录下的 OHOS Menu API 文档。
> **Popup 方案演进**：Phase 4 (bindContextMenu) → Phase 6 (openMenu) → Phase 9 (bindMenu 最终方案)。详见 [phase9-popup-bindcontextmenu-design.md](impl/phase9-popup-bindcontextmenu-design.md)。

## 一、需求概述

本轮需要支持 `tauri::menu` 模块的以下接口：

- Menu
- MenuEvent
- MenuItem
- PredefinedMenuItem
- Submenu

## 二、现状分析

### 2.1 桌面端实现架构

| 层级 | 实现 | 说明 |
|------|------|------|
| JS API | `@tauri-apps/api/menu` | 前端调用接口 |
| Rust API | `tauri/src/menu/*.rs` | 菜单逻辑实现 |
| 底层库 | `muda` crate | Windows/macOS/Linux 系统菜单 |

关键发现：
- `menu` 模块使用 `#[cfg(desktop)]` 条件编译，仅在桌面平台启用
- `muda` crate 在 Cargo.toml 中通过 `not(target_env = "ohos")` 排除 OHOS
- OHOS 平台当前没有任何菜单相关代码

### 2.2 openharmony-ability 现状

| 功能 | OHOS API | 当前封装状态 |
|------|----------|------------|
| 退出应用 | `process.exit()` | ✅ 已封装 `app.exit(code)` |
| 显示键盘 | `IME.show()` | ✅ 已封装 |
| 隐藏键盘 | `IME.hide()` | ✅ 已封装 |
| 窗口事件 | `window.on(...)` | ✅ 已封装 |
| **窗口操作** | `window.minimize/maximize/close` | ❌ 未封装 |
| **弹出菜单** | `UIContext.getPopupController()` | ❌ 未封装 |
| **上下文菜单** | `Web.bindContextMenu()` | ❌ 未封装 |

### 2.3 鸿蒙 ArkUI Menu API

根据官方文档（`doc/menu/*.md`），鸿蒙提供以下菜单组件：

| API | 功能 | 可用性 |
|-----|------|--------|
| `Menu()` | 菜单容器组件 | ✅ API 9+ |
| `MenuItem()` | 菜单项（支持图标、文本、快捷键、子菜单） | ✅ API 9+ |
| `MenuItemGroup()` | 菜单分组 | ✅ API 9+ |
| `bindMenu()` | 绑定菜单到组件（点击触发） | ✅ API 9+ |
| `bindContextMenu(content, responseType)` | 绑定上下文菜单（长按/右键触发） | ✅ API 9+ |
| `bindContextMenu(isShown, ...)` | 程序化控制菜单显隐 | ✅ API 12+ |
| `ContextMenu.close()` | 关闭上下文菜单 | ⚠️ API 8+（**API 18 废弃**） |
| `UIContext.getContextMenuController().close()` | 推荐的关闭方法 | ✅ API 18+ |
| **全局菜单栏** | macOS/Windows 风格的窗口顶部菜单 | ❌ **不存在** |

**关键发现**：
- 鸿蒙没有传统桌面端的"窗口菜单栏"概念，菜单需要通过组件绑定方式触发
- **但 `bindContextMenu(isShown: boolean, ...)` API 12+ 支持程序化控制菜单显隐**：
  - `isShown = true` → 弹出菜单
  - `isShown = false` → 关闭菜单
  - 可通过状态变量在任意时刻触发，无需用户手势

## 三、设计决策

### 3.1 API 支持策略

| Tauri API | OHOS 支持 | 实现方式 |
|-----------|----------|---------|
| `Menu.new()` | ✅ 支持 | 创建 ArkUI Menu 数据结构 |
| `MenuItem.new()` | ✅ 支持 | 创建 MenuItem 数据 |
| `Submenu.new()` | ✅ 支持 | MenuItem.builder 参数 |
| `Menu.popup()` | ✅ 支持 | `bindContextMenu(isShown)` + 状态变量控制，offset 参数定位 |
| `Menu.set_as_app_menu()` | ❌ 不支持 | 返回 `"not supported on OHOS"` |
| `Menu.set_as_window_menu()` | ❌ 不支持 | 返回 `"not supported on OHOS"` |
| MenuEvent | ✅ 支持 | 通过回调机制 |

### 3.2 PredefinedMenuItem 实现

| Tauri API | OHOS 实现 | 封装位置 | 备注 |
|-----------|----------|---------|------|
| `separator()` | `MenuDivider` | menu.ets | ✅ |
| `copy()` | `runJavaScript('document.execCommand("copy")')` | menu.ets | WebView API |
| `cut()` | `runJavaScript('document.execCommand("cut")')` | menu.ets | WebView API |
| `paste()` | `runJavaScript('document.execCommand("paste")')` | menu.ets | WebView API |
| `selectAll()` | `runJavaScript('document.execCommand("selectAll")')` | menu.ets | WebView API |
| `undo()` | `runJavaScript('document.execCommand("undo")')` | menu.ets | ⚠️ JS fallback |
| `redo()` | `runJavaScript('document.execCommand("redo")')` | menu.ets | ⚠️ JS fallback |
| `quit()` | `context.terminateSelf()` | os.ets | ✅ UIAbility API |
| `minimize()` | `window.minimize()` | window.ets | ✅ API 12+ |
| `maximize()` | `window.maximize()` | window.ets | ✅ API 12+ |
| `restore()` | `window.recover()` | window.ets | ✅ 使用 recover (非 restore) |
| `closeWindow()` | `window.destroyWindow()` | window.ets | ✅ 用 destroyWindow (非 close) |
| `fullscreen()` | `window.maximize(ENTER_IMMERSIVE)` | window.ets | ✅ 无 setFullScreen |
| `hide()` | ⚠️ 用 `minimize()` 替代 | window.ets | 主窗口无 hide，用 minimize；子窗口 minimize 即隐藏 |
| `hideOthers()` | ❌ 不支持 | - | 跨应用操作，OHOS 不允许 |
| `showAll()` | ❌ 不支持 | - | 跨应用操作，OHOS 不允许 |
| `about()` | ⚠️ 需自定义 | dialog.ets | 用 AlertDialog 自定义实现 |
| `services()` | ❌ 不支持 | - | macOS only |
| `bringAllToFront()` | ❌ 不支持 | - | macOS only |

## 四、实现方案

### 4.1 模块依赖关系

```
┌─────────────────────────────────────────────────────────────┐
│                JS API (@tauri-apps/api/menu)                 │
│                         保持不变                              │
├─────────────────────────────────────────────────────────────┤
│                 tauri/src/menu/plugin.rs                     │
│              添加 OHOS 条件编译的 command 实现                 │
├─────────────────────────────────────────────────────────────┤
│                 tauri/src/menu/mod.rs                        │
│           修改 cfg 条件：#[cfg(any(desktop, ohos))]          │
├─────────────────────────────────────────────────────────────┤
│          OHOS 特有实现 (menu/ohos.rs) [新增]                 │
│         - OhosMenu / OhosMenuItem / OhosSubmenu             │
│         - 不依赖 muda，使用 openharmony-ability              │
├─────────────────────────────────────────────────────────────┤
│              openharmony-ability [新增 Menu 模块]            │
│         - menu.ets: ArkUI Menu/PopupMenu 绑定               │
│         - window.ets: 窗口操作封装                           │
│         - menu/mod.rs: Rust 封装                             │
├─────────────────────────────────────────────────────────────┤
│                    ArkUI Native API                          │
│        Menu/MenuItem/bindContextMenu/WebviewController       │
└─────────────────────────────────────────────────────────────┘
```

### 4.2 需要修改的文件清单

#### openharmony-ability（新增）

| 文件 | 说明 |
|------|------|
| `native_ability/src/main/ets/helper/menu.ets` | ArkUI Menu 组件封装 |
| `native_ability/src/main/ets/helper/window.ets` | 窗口操作封装 |
| `crates/ability/src/menu/mod.rs` | Menu Rust API 封装 |

#### tauri（修改 + 新增）

| 文件 | 修改内容 |
|------|---------|
| `src/lib.rs` | 修改 `#[cfg(desktop)]` → `#[cfg(any(desktop, target_env = "ohos"))]` |
| `src/menu/menu.rs` | `popup_inner` 添加 OHOS 实现 |
| `src/menu/submenu.rs` | `popup_inner` 添加 OHOS 实现 |
| `src/menu/predefined.rs` | OHOS 行为实现 |
| `src/menu/plugin.rs` | 添加 OHOS command 支持 |
| **新增** `src/menu/ohos.rs` | OHOS 特有实现，调用 openharmony-ability |
| `Cargo.toml` | 添加 OHOS 菜单依赖配置 |

### 4.2.1 Menu.popup() 实现方案

OHOS 通过 `bindContextMenu(isShown: boolean, content, options)` 实现程序化弹出菜单：

**API 签名**（API 12+）：
```typescript
bindContextMenu(isShown: boolean, content: CustomBuilder, options?: ContextMenuOptions): T
```

**核心参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| `isShown` | boolean | 状态变量控制显隐：`true` 弹出，`false` 关闭 |
| `content` | CustomBuilder | 菜单内容构建器 |
| `options.offset` | Position | 菜单偏移量 `{ x, y }` |
| `options.placement` | Placement | 预设位置（Top/Bottom/Left/Right 等） |
| `options.anchorPosition` | Position | 相对绑定组件左上角的精确位置（API 20+） |

**实现策略**：
1. 创建一个**透明占位组件**绑定菜单（覆盖整个窗口或指定区域）
2. 通过状态变量 `isShown` 控制弹出时机
3. 使用 `offset` 或 `anchorPosition` 控制弹出位置
4. 调用 `Menu.popup(x, y)` 时，设置 `offset = { x, y }` 并触发 `isShown = true`

**示例代码**：
```typescript
@Component
struct TauriMenuHost {
  @State isMenuShown: boolean = false;
  @State menuOffset: Position = { x: 0, y: 0 };
  private menuItems: TauriMenuItemData[] = [];

  @Builder
  MenuContent() {
    Menu() {
      ForEach(this.menuItems, (item: TauriMenuItemData) => {
        if (item.type === 'separator') {
          MenuDivider()
        } else if (item.type === 'item') {
          MenuItem({ content: item.text! })
            .onClick(() => this.onMenuItemClick(item.id))
        }
      })
    }
  }

  popup(x: number, y: number) {
    this.menuOffset = { x, y };
    this.isMenuShown = true;  // 程序化弹出
  }

  build() {
    Stack() {
      // 透明占位层，用于绑定菜单
      Column()
        .width('100%')
        .height('100%')
        .backgroundColor(Color.Transparent)
        .bindContextMenu(this.isMenuShown, this.MenuContent, {
          offset: this.menuOffset,
          placement: Placement.BottomLeft,
          aboutToDisappear: () => { this.isMenuShown = false; }
        })
    }
  }
}
```

**注意事项**：
- 菜单弹出位置相对于**绑定组件**，而非屏幕绝对坐标
- `offset` 参数受 `placement` 影响，需根据场景调整
- 菜单关闭时会触发 `aboutToDisappear`，需在此回调中重置 `isShown`
- API 20+ 的 `anchorPosition` 可实现更精确的定位

### 4.3 ArkTS 实现设计

#### menu.ets

```typescript
import { UIContext } from "@ohos.arkui.UIContext";
import web_webview from "@ohos.web.webview";
import window from "@ohos.window";

// 菜单数据结构
export interface TauriMenuItemData {
  id: string;
  type: 'item' | 'submenu' | 'separator' | 'predefined';
  text?: string;
  enabled?: boolean;
  accelerator?: string;
  startIcon?: ResourceStr;
  predefinedType?: string;
  submenuItems?: TauriMenuItemData[];
}

// 预定义菜单项执行器
export class PredefinedActionExecutor {
  private webviewController: web_webview.WebviewController | null = null;
  private window: window.Window | null = null;
  private exitFn: (code: number) => void;

  constructor(exitFn: (code: number) => void) {
    this.exitFn = exitFn;
  }

  setWebviewController(controller: web_webview.WebviewController) {
    this.webviewController = controller;
  }

  setWindow(win: window.Window) {
    this.window = win;
  }

  async execute(type: string): Promise<void> {
    switch (type) {
      case 'copy':
        await this.webviewController?.runJavaScript('document.execCommand("copy")');
        break;
      case 'cut':
        await this.webviewController?.runJavaScript('document.execCommand("cut")');
        break;
      case 'paste':
        await this.webviewController?.runJavaScript('document.execCommand("paste")');
        break;
      case 'selectAll':
        await this.webviewController?.runJavaScript('document.execCommand("selectAll")');
        break;
      case 'undo':
        await this.webviewController?.runJavaScript('document.execCommand("undo")');
        break;
      case 'redo':
        await this.webviewController?.runJavaScript('document.execCommand("redo")');
        break;
      case 'quit':
        this.exitFn(0);
        break;
      case 'minimize':
        await this.window?.minimize();
        break;
      case 'maximize':
        await this.window?.maximize();
        break;
      case 'close':
        await this.window?.destroyWindow();
        break;
      case 'fullscreen':
        await this.window?.maximize(window.MaximizePresentation.ENTER_IMMERSIVE);
        break;
    }
  }
}

// 菜单管理器（使用状态变量控制弹出）
export class TauriMenuManager {
  private executor: PredefinedActionExecutor;
  private onMenuItemClick: ((id: string) => void) | null = null;
  private menus: Map<string, TauriMenuItemData[]> = new Map();
  private isShown: boolean = false;
  private currentMenuId: string = '';
  private menuOffset: Position = { x: 0, y: 0 };

  constructor(executor: PredefinedActionExecutor) {
    this.executor = executor;
  }

  createMenu(id: string, items: TauriMenuItemData[]): void {
    this.menus.set(id, items);
  }

  setOnMenuItemClick(callback: (id: string) => void): void {
    this.onMenuItemClick = callback;
  }

  // 弹出菜单（程序化控制）
  popup(menuId: string, x?: number, y?: number): void {
    this.currentMenuId = menuId;
    this.menuOffset = { x: x ?? 0, y: y ?? 0 };
    this.isShown = true;  // 触发菜单弹出
  }

  // 关闭菜单
  close(): void {
    this.isShown = false;
  }

  // 获取当前状态（供组件绑定使用）
  getIsShown(): boolean {
    return this.isShown;
  }

  getOffset(): Position {
    return this.menuOffset;
  }

  getMenuItems(): TauriMenuItemData[] {
    return this.menus.get(this.currentMenuId) ?? [];
  }

  // 菜单关闭回调
  onMenuDisappear(): void {
    this.isShown = false;
  }

  // 执行菜单项点击
  handleItemClick(item: TauriMenuItemData): void {
    if (item.type === 'predefined') {
      this.executor.execute(item.predefinedType!);
    } else if (item.type === 'item') {
      this.onMenuItemClick?.(item.id);
    }
    this.close();
  }
}

// 独立的菜单构建函数（@Builder 只能用于函数或组件内方法）
@Builder
function TauriMenuBuilder(manager: TauriMenuManager) {
  const items = manager.getMenuItems();
  Menu() {
    ForEach(items, (item: TauriMenuItemData) => {
      if (item.type === 'separator') {
        MenuDivider()
      } else if (item.type === 'item' || item.type === 'predefined') {
        MenuItem({ 
          content: item.text ?? '', 
          labelInfo: item.accelerator,
          // 图标属性（参考 menu_item.md 第 43-76 行）
          startIcon: item.startIcon,
          endIcon: item.endIcon,
          symbolStartIcon: item.symbolStartIcon,  // API 12+
          symbolEndIcon: item.symbolEndIcon,      // API 12+
          // 选中状态属性（参考 menu_item.md 第 86-113 行）
          selected: item.selected,
          selectIcon: item.selectIcon,
        })
          .enabled(item.enabled ?? true)
          .onClick(() => manager.handleItemClick(item))
          // onChange 事件（参考 menu_item.md 第 228-249 行）
          .onChange((selected: boolean) => {
            if (item.selected !== undefined) {
              manager.handleItemChange?.(item.id, selected);
            }
          })
      } else if (item.type === 'submenu') {
        MenuItem({ 
          content: item.text ?? '', 
          builder: () => TauriSubmenuBuilder(manager, item.submenuItems!) 
        })
      }
    })
  }
}

@Builder
function TauriSubmenuBuilder(manager: TauriMenuManager, items: TauriMenuItemData[]) {
  Menu() {
    ForEach(items, (item: TauriMenuItemData) => {
      if (item.type === 'separator') {
        MenuDivider()
      } else if (item.type === 'item' || item.type === 'predefined') {
        MenuItem({ 
          content: item.text ?? '',
          startIcon: item.startIcon,
          endIcon: item.endIcon,
          selected: item.selected,
        })
          .onClick(() => manager.handleItemClick(item))
          .onChange((selected: boolean) => {
            if (item.selected !== undefined) {
              manager.handleItemChange?.(item.id, selected);
            }
          })
      }
    })
  }
}
```

**MenuItem 属性说明**（参考 menu_item.md）：

| 属性 | 类型 | API 版本 | 说明 |
|------|------|---------|------|
| `content` | string | API 9+ | 菜单项文本 |
| `labelInfo` | string | API 9+ | 结束标签（快捷键显示位置） |
| `startIcon` | ResourceStr | API 9+ | 开始图标 |
| `endIcon` | ResourceStr | API 9+ | 结束图标 |
| `symbolStartIcon` | SymbolGlyphOptions | API 12+ | Symbol 开始图标 |
| `symbolEndIcon` | SymbolGlyphOptions | API 12+ | Symbol 结束图标 |
| `selected` | boolean | API 9+ | 是否选中 |
| `selectIcon` | ResourceStr | API 9+ | 选中时显示的图标 |
| `enabled` | boolean | API 9+ | 是否启用 |

**MenuItem 事件说明**：

| 事件 | 参数 | API 版本 | 说明 |
|------|------|---------|------|
| `onClick` | () => void | API 9+ | 点击事件 |
| `onChange` | (selected: boolean) => void | API 9+ | 选中状态变化事件 |

**Menu 容器属性说明**（参考 menu.md）：

| 属性 | 类型 | API 版本 | 说明 |
|------|------|---------|------|
| `subMenuExpandingMode` | SubMenuExpandingMode | API 12+ | 子菜单展开样式 |

**SubMenuExpandingMode 枚举值**（参考 menu.md 第 195-215 行）：

| 值 | 说明 |
|------|------|
| `SIDE_EXPAND` | 子菜单在右侧展开（默认） |
| `EMBEDDED_EXPAND` | 子菜单嵌入当前菜单展开 |
| `STACK_EXPAND` | 子菜单以堆叠方式展开 |

```typescript
// 使用示例
Menu() {
  MenuItem({ content: 'File', builder: () => this.FileSubmenu() })
}
.subMenuExpandingMode(SubMenuExpandingMode.SIDE_EXPAND)
```

#### window.ets

```typescript
import window from "@ohos.window";

export interface WindowHelper {
  minimize: () => Promise<void>;
  maximize: () => Promise<void>;
  recover: () => Promise<void>;  // OHOS 用 recover 而非 restore
  destroyWindow: () => Promise<void>;  // OHOS 用 destroyWindow 而非 close
  setLayoutFullScreen: (fullScreen: boolean) => Promise<void>;
}

export function createWindowHelper(win: window.Window): WindowHelper {
  return {
    minimize: async () => await win.minimize(),
    maximize: async () => await win.maximize(),
    recover: async () => await win.recover(),  // 从 maximize/fullscreen 恢复
    destroyWindow: async () => await win.destroyWindow(),
    setLayoutFullScreen: async (fs) => await win.setWindowLayoutFullScreen(fs),
  };
}
```

### 4.4 Rust 实现设计

#### openharmony-ability/crates/ability/src/menu/mod.rs

```rust
pub struct MenuBuilder {
    id: Option<String>,
    items: Vec<MenuItemKind>,
}

pub struct Menu {
    id: String,
    items: Vec<MenuItemKind>,
}

pub enum MenuItemKind {
    Item(MenuItem),
    Separator,
    Submenu(Submenu),
    Predefined(PredefinedMenuItem),
}

pub struct MenuItem {
    id: String,
    text: String,
    enabled: bool,
    accelerator: Option<String>,
}

pub enum PredefinedMenuItem {
    Copy, Cut, Paste, SelectAll, Undo, Redo,
    Quit, Minimize, Maximize, CloseWindow, Fullscreen, Separator,
}

impl Menu {
    pub fn new() -> Result<Self> { /* 调用 ArkTS */ }
    pub fn popup(&self, x: Option<f64>, y: Option<f64>) -> Result<()> { /* 调用 ArkTS */ }
}
```

#### tauri/src/menu/ohos.rs

```rust
#[cfg(target_env = "ohos")]
pub(crate) mod ohos_impl {
    use crate::{AppHandle, Runtime, Window};
    use crate::menu::MenuId;

    pub(crate) struct OhosMenuInner<R: Runtime> {
        id: MenuId,
        menu: openharmony_ability::menu::Menu,
        app_handle: AppHandle<R>,
    }

    pub fn create_menu<R: Runtime>(app: &AppHandle<R>) -> crate::Result<Menu<R>> {
        // ...
    }

    pub fn popup_menu<R: Runtime>(
        menu: &Menu<R>,
        window: &Window<R>,
        position: Option<Position>
    ) -> crate::Result<()> {
        // 调用 openharmony_ability::menu::Menu::popup()
    }
}
```

#### tauri/src/menu/menu.rs 修改

```rust
fn popup_inner<T: Runtime, P: Into<crate::Position>>(
    &self,
    window: crate::Window<T>,
    position: Option<P>,
) -> crate::Result<()> {
    // ... 原有 desktop 实现 ...

    // OHOS - 新增
    #[cfg(target_env = "ohos")]
    {
        use crate::menu::ohos_impl::popup_menu;
        popup_menu(&self_, &window, position)?;
    }
}
```

## 五、测试方案

> **详细测试方案**：详见 [impl/phase5-integration-testing.md](impl/phase5-integration-testing.md)

本项目采用两种测试方式：
1. **Rust UT**：使用 `ohos-rust-ut` skill，测试 `#[cfg(target_env = "ohos")]` 门控代码
2. **前端 API 测试**：使用 `frontend-api-testing` + `ohos-build` skill，端到端测试 API 功能

### 5.1 Rust 单元测试

用于测试 OHOS 特有的 Rust 代码逻辑（无法在 desktop 平台编译）。

**文件位置**：`tauri/crates/tauri/src/menu/ohos.rs`

```rust
#[cfg(all(test, target_env = "ohos"))]
mod tests {
    use super::*;

    #[test]
    fn test_menu_id_generation() {
        let menu = OhosMenu::new(None);
        assert!(menu.id().starts_with("menu_"));
    }

    #[test]
    fn test_menu_item_data_serialization() {
        let data = MenuItemData {
            id: "test".to_string(),
            item_type: "item".to_string(),
            text: Some("Test".to_string()),
            enabled: Some(true),
            accelerator: Some("Ctrl+T".to_string()),
            predefined_type: None,
            submenu_items: None,
        };
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("\"type\":\"item\""));
    }
}
```

**运行命令**（使用 ohos-rust-ut skill）：
```bash
bash D:/workspace/tauri/tauri/.claude/skills/ohos-rust-ut/scripts/run-ut.sh menu::ohos
```

**注意事项**：
- 不能使用 `mock_app()`、`mock_builder()`（desktop-only）
- 只能测试纯逻辑函数
- 测试二进制推送到设备运行

### 5.2 前端 API 测试

用于端到端测试 Tauri Menu API 功能。

**文件位置**：`examples/api/src/lib/tests/core.ts`（添加到现有文件）

```typescript
// === Menu 模块测试 ===
import { Menu, MenuItem, Submenu, PredefinedMenuItem } from '@tauri-apps/api/menu';

// 自动测试（category: 'auto'）
{
  name: '@tauri-apps/api/menu.Menu.new',
  category: 'auto',
  async fn() {
    const menu = await Menu.new();
    assert(menu !== undefined, 'Menu.new returned undefined');
  },
},
{
  name: '@tauri-apps/api/menu.MenuItem.new',
  category: 'auto',
  async fn() {
    const item = await MenuItem.new({ text: 'Test' });
    const text = await item.text();
    assert(text === 'Test', `text mismatch: ${text}`);
  },
},
{
  name: '@tauri-apps/api/menu.Menu.append',
  category: 'auto',
  async fn() {
    const menu = await Menu.new();
    const item = await MenuItem.new({ text: 'Item' });
    await menu.append(item);
    const items = await menu.items();
    assert(items.length === 1, `items length: ${items.length}`);
  },
},
{
  name: '@tauri-apps/api/menu.PredefinedMenuItem.copy',
  category: 'auto',
  async fn() {
    const copy = await PredefinedMenuItem.copy();
    assert(copy !== undefined, 'PredefinedMenuItem.copy returned undefined');
  },
},

// 副作用测试（category: 'side-effect'）- 程序可验证的副作用
{
  name: '@tauri-apps/api/menu.PredefinedMenuItem.minimize',
  category: 'side-effect',
  async fn() {
    const minimize = await PredefinedMenuItem.minimize();
    assert(minimize !== undefined);
    // 窗口最小化可通过窗口状态验证
  },
},
```

### 5.3 手动测试（TestRunner.svelte）

用于需要人工确认的场景（菜单弹出、UI 显示）。

**文件位置**：`examples/api/src/views/TestRunner.svelte`

```typescript
// 手动测试 handler（使用 wrapManual 包装）
async function manualMenuPopup() {
  await wrapManual('menu.popup', async () => {
    const menu = await Menu.new({
      items: [
        await PredefinedMenuItem.copy({ text: '复制' }),
        await PredefinedMenuItem.paste({ text: '粘贴' }),
        await PredefinedMenuItem.separator(),
        await MenuItem.new({ text: '自定义项' }),
      ]
    });
    await menu.popup();
    manualResult = 'Menu.popup() triggered - check if menu appears at cursor position';
    onMessage(manualResult);
  });
}

async function manualPredefinedCopy() {
  await wrapManual('menu.copy', async () => {
    // 先在 WebView 中选中文字
    const copy = await PredefinedMenuItem.copy();
    const menu = await Menu.new({ items: [copy] });
    await menu.popup();
    manualResult = 'Click "Copy" then verify clipboard has selected text';
    onMessage(manualResult);
  });
}
```

```svelte
<!-- Manual Tests 区域添加按钮 -->
<button class="btn" onclick={manualMenuPopup}>Menu.popup (should show menu)</button>
<button class="btn" onclick={manualPredefinedCopy}>Copy (should copy selected text)</button>
```

### 5.4 测试执行流程

**一键测试**（推荐）：
```bash
bash D:/workspace/tauri/tauri/.claude/skills/ohos-build/scripts/run-tests.sh
```

**分步流程**：
```bash
# 1. 禁用 hvigorfile.ts 中的 tauriPlugin（见 ohos-build skill）
# 2. 构建（启用 AUTOTEST）
export VITE_AUTOTEST=true
bash D:/workspace/tauri/tauri/.claude/skills/ohos-build/scripts/build-ohos.sh

# 3. 签名安装
bash D:/workspace/tauri/tauri/.claude/skills/ohos-build/scripts/sign-and-install.sh

# 4. 等待测试完成（约 10-15 秒）

# 5. 拉取测试报告（使用 cmd.exe 避免 Git Bash 路径转义）
cmd.exe /c "hdc file recv /data/app/el2/100/base/com.tauri.api/cache/test-report.json D:\workspace\tauri\tauri\examples\api\test-report.json"

# 6. 拉取手动测试日志
cmd.exe /c "hdc file recv /data/app/el2/100/base/com.tauri.api/cache/console-log.txt D:\workspace\tauri\tauri\examples\api\console-log.txt"

# 7. 恢复 hvigorfile.ts
```

### 5.5 测试报告格式

**自动测试报告** (`test-report.json`)：
```json
{
  "timestamp": "2026-05-14T05:20:47.253Z",
  "total": 15,
  "passed": 12,
  "failed": 2,
  "skipped": 1,
  "results": [
    { "name": "@tauri-apps/api/menu.Menu.new", "category": "auto", "status": "pass", "duration": 10 },
    { "name": "@tauri-apps/api/menu.Menu.popup", "category": "manual", "status": "skip" },
    { "name": "@tauri-apps/api/menu.PredefinedMenuItem.copy", "category": "auto", "status": "fail", "error": "..." }
  ]
}
```

**手动测试日志** (`console-log.txt`)：
```
[menu.popup] Menu.popup() triggered - check if menu appears at cursor position
[menu.copy] Click "Copy" then verify clipboard has selected text
```

### 5.6 Rust UT 执行流程

```bash
# 运行 menu 模块的 OHOS 单元测试
bash D:/workspace/tauri/tauri/.claude/skills/ohos-rust-ut/scripts/run-ut.sh menu::ohos

# 或过滤特定测试
bash .../run-ut.sh test_menu_id_generation

# 输出示例：
# running 5 tests
# test menu::ohos::tests::test_menu_id_generation ... ok
# test menu::ohos::tests::test_menu_item_data_serialization ... ok
# test result: ok. 5 passed; 0 failed; 0 ignored
```

### 5.7 测试类别说明

| 类别 | 适用场景 | 自动执行 | 示例 |
|------|----------|----------|------|
| `auto` | 纯函数调用，有返回值可断言 | ✓ | `Menu.new()`, `MenuItem.text()` |
| `side-effect` | 有副作用但可程序验证 | ✓ | `PredefinedMenuItem.minimize()` |
| `manual` | 需人工确认（UI、交互） | ✗（跳过） | `Menu.popup()` |

### 5.8 相关 SKILL 文档

- [ohos-rust-ut](../.claude/skills/ohos-rust-ut/SKILL.md) - Rust UT 执行
- [frontend-api-testing](../.claude/skills/frontend-api-testing/SKILL.md) - 前端测试编写
- [ohos-build](../.claude/skills/ohos-build/SKILL.md) - OHOS 构建和安装

## 六、工作量估算

| 模块 | 工作内容 | 估算 |
|------|---------|------|
| openharmony-ability ETS | menu.ets + window.ets | 3 天 |
| openharmony-ability Rust | menu/mod.rs | 2 天 |
| tauri menu ohos.rs | OHOS 特有实现 | 3 天 |
| tauri menu.rs/submenu.rs | popup 实现 + cfg 调整 | 1 天 |
| tauri predefined.rs | OHOS 行为映射 | 2 天 |
| Rust UT 编写 | ohos.rs #[cfg(test)] | 0.5 天 |
| 前端测试用例 | core.ts + TestRunner.svelte | 1 天 |
| 调试与修复 | 集成测试 | 2-3 天 |
| **总计** | | **12-14 天** |

## 七、审计结果（2026-05-14）

### 7.1 严重问题

#### 问题 1：WebviewController.executeCopy 等方法不存在

**原设计假设**：
- 使用 `webviewController.executeCopy()`、`executeCut()`、`executePaste()`、`executeSelectAll()`、`executeUndo()`、`executeRedo()` 执行预定义菜单项操作

**实际情况**：
- **WebviewController 没有这些方法**
- 这些操作通过 `WebContextMenuResult` 对象完成，仅在 `onContextMenuShow` 回调中可用
- `WebContextMenuResult` 提供：`copy()`、`cut()`、`paste()`、`selectAll()`、`copyImage()`、`closeContextMenu()`
- **undo/redo 不存在**：没有对应的 OHOS API

**正确实现方式**：
```typescript
// 在 onContextMenuShow 回调中使用 WebContextMenuResult
Web({ src: '...', controller: this.controller })
  .onContextMenuShow((event) => {
    if (event) {
      this.result = event.result;  // WebContextMenuResult
    }
    return true;
  })

// 菜单项点击时调用
MenuItem({ content: '复制' })
  .onClick(() => {
    this.result?.copy();       // 正确方式
    this.result?.closeContextMenu();
  })
```

#### 问题 2：popup 程序化弹出菜单的实现方式（已解决）

**原设计假设**：
- 使用 `UIContext.getPopupController()` 程序化弹出菜单

**实际情况（已修正）**：
- **`bindContextMenu(isShown: boolean, ...)` API 12+ 支持程序化控制菜单显隐**
- 通过状态变量 `isShown` 控制：
  - `isShown = true` → 弹出菜单
  - `isShown = false` → 关闭菜单
- 可在任意时刻通过代码触发，无需用户手势

**正确实现方式**：
```typescript
@State isMenuShown: boolean = false;
@State menuOffset: Position = { x: 0, y: 0 };

// 透明占位组件绑定菜单
Column()
  .bindContextMenu(this.isMenuShown, this.MenuContent, {
    offset: this.menuOffset,
    aboutToDisappear: () => { this.isMenuShown = false; }
  })

// 程序化弹出（等效于 Tauri Menu.popup()）
popup(x: number, y: number) {
  this.menuOffset = { x, y };
  this.isMenuShown = true;  // 代码触发弹出
}
```

**限制与注意事项**：
- 菜单位置相对于**绑定组件**，需通过 `offset` 或 `anchorPosition`（API 20+）控制
- 需创建占位组件用于绑定菜单
- 关闭时需在 `aboutToDisappear` 回调中重置 `isShown`

#### 问题 3：@Builder 装饰器使用错误

**原设计错误**：
```typescript
class TauriMenuManager {
  @Builder  // 错误！不能在类方法上使用
  buildMenuContent(menuId: string): void { ... }
}
```

**正确用法**：
- `@Builder` 只能用于：
  1. 组件 (`@Component`) 内的方法
  2. 独立的函数（不能是类方法）

**修正代码**：
```typescript
// 正确：独立函数
@Builder
function MenuBuilder(items: TauriMenuItemData[]) {
  Menu() {
    ForEach(items, (item) => { ... })
  }
}

// 正确：组件内方法
@Component
struct WebComponent {
  @Builder
  MyMenuBuilder() { ... }  // 组件内可以
}
```

### 7.2 中等问题

#### 问题 4：缺少 ArkHelper 模式集成说明

**原设计问题**：
- 设计创建独立的 `TauriMenuManager` 类，但未说明如何与现有 `ArkHelper` 集成
- 现有 `ArkHelper` 定义在 `type.ets`，包括 `createWebview` 方法

**应遵循的模式**：
```typescript
// type.ets 中扩展 ArkHelper
export interface ArkHelper {
  exit: (code: number) => void;
  createWebview: (data: WebViewInitData) => Object;
  createEmbeddedWebview: (data: WebViewInitData) => Object;
  requestPermission: (permission: string | string[]) => Promise<number | number[]>;
  getWindowAvoidArea: (type: number) => WindowAvoidAreaInfo | undefined;
  // 新增菜单相关方法
  createContextMenu?: (options: ContextMenuOptions) => ContextMenuHandle;
  getWebViewController?: (webTag: string) => web_webview.WebviewController;
  getWindow?: () => window.Window;
}
```

#### 问题 5：WebContextMenuResult 生命周期未说明

**问题**：
- `WebContextMenuResult` 只在 `onContextMenuShow` 回调中有效
- 菜单关闭后，`result` 对象失效，后续调用会出错

**需要处理**：
- 存储 `result` 到组件状态变量
- 菜单关闭时调用 `closeContextMenu()` 并清空状态

#### 问题 6：窗口操作 API 验证结果（已验证 ✅）

**已验证的窗口 API**：

| API | 状态 | OHOS 接口 | 备注 |
|-----|------|----------|------|
| `minimize()` | ✅ 存在 | `window.minimize()` | API 12+ |
| `maximize()` | ✅ 存在 | `window.maximize()` | API 12+ |
| `restore()` | ✅ 存在 | `window.recover()` | 用 recover（非 restore） |
| `close()` | ✅ 存在 | `window.destroyWindow()` | 用 destroyWindow（非 close） |
| `fullscreen` | ✅ 存在 | `window.maximize(ENTER_IMMERSIVE)` | 无 setFullScreen |
| `quit` | ✅ 存在 | `context.terminateSelf()` | UIAbility API |

**结论**：所有窗口操作 API 都已验证可用，只需注意命名差异。

### 7.3 需要修正的 PredefinedMenuItem 实现

| Tauri API | 原设计 | 正确实现 |
|-----------|--------|---------|
| `copy()` | `webviewController.executeCopy()` ❌ | `WebContextMenuResult.copy()` 或 `runJavaScript('document.execCommand("copy")')` |
| `cut()` | `webviewController.executeCut()` ❌ | `WebContextMenuResult.cut()` 或 JS |
| `paste()` | `webviewController.executePaste()` ❌ | `WebContextMenuResult.paste()` 或 JS |
| `selectAll()` | `webviewController.executeSelectAll()` ❌ | `WebContextMenuResult.selectAll()` 或 JS |
| `undo()` | `webviewController.executeUndo()` ❌ | **不存在** - 用 JS 或标记不支持 |
| `redo()` | `webviewController.executeRedo()` ❌ | **不存在** - 用 JS 或标记不支持 |

### 7.4 最终实现策略

#### 策略调整（已确定）

| API | 实现策略 | 状态 |
|-----|---------|------|
| `Menu.popup()` | 使用 `bindContextMenu(isShown)` + 状态变量控制 | ✅ 可实现 |
| `copy/cut/paste/selectAll` | 通过 `runJavaScript` 执行 `document.execCommand()` | ✅ 可实现 |
| `undo/redo` | 通过 JS `document.execCommand("undo/redo")` 或标记不支持 | ⚠️ JS fallback |
| `minimize/maximize` | 使用 `window.minimize()` / `window.maximize()` | ✅ 已验证 |
| `restore` | 使用 `window.recover()` | ✅ 已验证 |
| `close` | 使用 `window.destroyWindow()` | ✅ 已验证 |
| `fullscreen` | 使用 `window.maximize(ENTER_IMMERSIVE)` | ✅ 已验证 |
| `quit` | 使用 `context.terminateSelf()` | ✅ 已验证 |

#### 修正后的 ArkTS 实现

```typescript
// helper/menu.ets - 修正版
import web_webview from "@ohos.web.webview";
import window from "@ohos.window";

export interface ContextMenuOptions {
  items: TauriMenuItemData[];
  onMenuItemClick?: (id: string) => void;
}

export interface ContextMenuHandle {
  show: (x: number, y: number) => void;  // 可能不支持
  close: () => void;
}

// 预定义菜单项执行器（修正版）
export class PredefinedActionExecutor {
  private controller: web_webview.WebviewController | null = null;
  private win: window.Window | null = null;
  private exitFn: (code: number) => void;

  constructor(exitFn: (code: number) => void) {
    this.exitFn = exitFn;
  }

  setController(controller: web_webview.WebviewController) {
    this.controller = controller;
  }

  setWindow(win: window.Window) {
    this.win = win;
  }

  // 使用 runJavaScript 执行（修正版）
  async execute(type: string): Promise<void> {
    if (!this.controller) return;

    switch (type) {
      case 'copy':
        await this.controller.runJavaScript('document.execCommand("copy")');
        break;
      case 'cut':
        await this.controller.runJavaScript('document.execCommand("cut")');
        break;
      case 'paste':
        await this.controller.runJavaScript('document.execCommand("paste")');
        break;
      case 'selectAll':
        await this.controller.runJavaScript('document.execCommand("selectAll")');
        break;
      case 'undo':
        // OHOS 不支持，使用 JS 方式
        await this.controller.runJavaScript('document.execCommand("undo")');
        break;
      case 'redo':
        await this.controller.runJavaScript('document.execCommand("redo")');
        break;
      case 'quit':
        this.exitFn(0);
        break;
      case 'minimize':
        if (this.win) await this.win.minimize();
        break;
      case 'maximize':
        if (this.win) await this.win.maximize();
        break;
      case 'close':
        // 注意：OHOS 使用 destroyWindow 而非 close
        if (this.win) await this.win.destroyWindow();
        break;
      case 'recover':
        // 注意：OHOS 用 recover 从 maximize/fullscreen 恢复
        if (this.win) await this.win.recover();
        break;
      case 'fullscreen':
        // 注意：OHOS 无 setFullScreen，使用 maximize(ENTER_IMMERSIVE)
        if (this.win) {
          await this.win.maximize(window.MaximizePresentation.ENTER_IMMERSIVE);
        }
        break;
    }
  }
}
```

### 7.5 最终结论

**设计方案已确定可行**：

1. **popup() 功能**：✅ 已确定可通过 `bindContextMenu(isShown)` + 状态变量控制实现
   - 详见 Phase 4 设计文档和 4.2.1 章节

2. **预定义菜单项**：✅ 已确定实现方式
   - copy/cut/paste/selectAll：使用 `runJavaScript` + `document.execCommand()`
   - undo/redo：使用 JS `document.execCommand("undo/redo")`（已验证可行）
   - 窗口操作：使用正确的 OHOS API（见 Phase 3）

3. **MenuEvent**：✅ 已设计完成
   - 详见 Phase 2 的 MenuEvent 实现章节

4. **实现阶段拆分**：已拆分为 5 个阶段
   - Phase 1: 基础架构（2天）
   - Phase 2: 核心菜单类型 + MenuEvent（3天）
   - Phase 3: PredefinedMenuItem（2天）
   - Phase 4: Popup 集成（3天）
   - Phase 5: 集成与测试（2-3天）
   - 总工期：12-14 天

### 7.6 窗口 API 验证结果（2026-05-14）

#### 已验证的窗口操作 API

| API | 状态 | OHOS 接口签名 | 说明 |
|-----|------|--------------|------|
| **minimize()** | ✅ 存在 | `minimize(callback)` / `minimize(): Promise<void>` | 主窗口最小化到 Dock，子窗口/悬浮窗隐藏 |
| **maximize()** | ✅ 存在 | `maximize(presentation?, acrossDisplay?): Promise<void>` | 主窗口最大化，子窗口需设置 `maximizeSupported` |
| **restore()** | ✅ 存在 | `restore(): Promise<void>` | 从最小化状态恢复（仅主窗口，仅 2in1 设备） |
| **recover()** | ✅ 存在 | `recover(): Promise<void>` | 从最大化/全屏/分屏恢复到浮动窗口模式 |
| **close/destroy** | ✅ 存在 | `destroyWindow(callback)` / `destroyWindow(): Promise<void>` | **注意：OHOS 使用 `destroyWindow` 而非 `close`** |
| **全屏设置** | ✅ 存在 | `setWindowLayoutFullScreen(boolean): Promise<void>` | 设置沉浸式布局 |
| **退出应用** | ✅ 存在 | `context.terminateSelf()` | UIAbility 退出 |

#### API 详细信息

**minimize()** (API 12+):
```typescript
// 主窗口：最小化到 Dock，用 restore() 恢复
// 子窗口/悬浮窗：隐藏，用 showWindow() 恢复
windowClass.minimize(): Promise<void>
// 错误码：801（不支持）、1300002（窗口状态异常）、1300003（服务异常）
```

**maximize()** (API 12+):
```typescript
// 主窗口可直接调用，子窗口需先设置 maximizeSupported=true
windowClass.maximize(
  presentation?: MaximizePresentation,  // 默认 ENTER_IMMERSIVE
  acrossDisplay?: boolean               // 折叠屏瀑布流控制
): Promise<void>
```

**restore()** (API 14+):
```typescript
// 仅 2in1 设备，仅主窗口，仅最小化状态恢复
windowClass.restore(): Promise<void>
```

**recover()** (API 12+):
```typescript
// 从最大化/全屏/分屏恢复到浮动窗口模式
// 自由窗口设备上可用，其他返回 801
windowClass.recover(): Promise<void>
```

**destroyWindow()** (API 6+):
```typescript
// 销毁窗口（OHOS 没有 close 方法）
// 支持：系统窗口、子窗口、全局悬浮窗、模态窗
windowClass.destroyWindow(): Promise<void>
```

**setWindowLayoutFullScreen()** (API 12+):
```typescript
// 设置沉浸式布局（不是传统全屏）
// true: 布局不避让状态栏
// false: 布局避让状态栏
windowClass.setWindowLayoutFullScreen(isLayoutFullScreen: boolean): Promise<void>
```

**terminateSelf()**:
```typescript
// UIAbility 退出方法
this.context.terminateSelf(): Promise<void>
```

#### PredefinedMenuItem 实现修正

| Tauri API | OHOS 实现 | 可用性 |
|-----------|----------|--------|
| `minimize()` | `window.minimize()` | ✅ 可用 |
| `maximize()` | `window.maximize()` | ✅ 可用 |
| `restore()` | `window.recover()` (非 restore) | ✅ 可用（使用 recover） |
| `close()` | `window.destroyWindow()` | ✅ 可用 |
| `fullscreen()` | `window.maximize(ENTER_IMMERSIVE)` | ✅ 可用 |
| `quit()` | `context.terminateSelf()` | ✅ 可用 |
| `hide()` | `window.minimize()` | ⚠️ **替代方案**：主窗口 minimize 到 Dock，子窗口 minimize 即隐藏 |
| `hideOthers()` | ❌ 不支持 | ❌ 跨应用操作，返回错误 |
| `showAll()` | ❌ 不支持 | ❌ 跨应用操作，返回错误 |
| `about()` | 自定义 AlertDialog | ⚠️ 需自行实现 |

#### document.execCommand 验证

在 OHOS WebView 中 `document.execCommand` 可用：
- 参考：`arkweb-adapter/references/12-processing-web-content-part6-clipboard.md` 第 668 行
- 示例：`document.execCommand('copy')` 用于复制操作

**支持的 execCommand 参数**：
- `'copy'` - 复制选中内容
- `'cut'` - 剪切选中内容
- `'paste'` - 粘贴（需要用户授权）
- `'selectAll'` - 全选
- `'undo'` / `'redo'` - 需要 JS 环境支持（OHOS 无原生 API）

### 7.7 API 版本要求汇总

| 功能 | 最低 API 版本 | 来源 |
|------|-------------|------|
| `bindContextMenu(isShown)` | API 12 | menu_control.md |
| `anchorPosition` 精确定位 | API 20 | menu_control.md |
| `Menu/MenuItem` 组件 | API 9 | menu.md |
| `minimize/maximize/recover` | API 12 | window API |
| `restore` (从 minimize) | API 14（仅 2in1） | window API |
| `destroyWindow` | API 6 | window API |
| `terminateSelf` | API 9 | UIAbility |

---

## 八、风险与注意事项

### 已解决的问题

1. **popup 实现**：✅ 已确定使用 `bindContextMenu(isShown)` + 状态变量控制
2. **剪贴板操作**：✅ 已确定使用 `runJavaScript` + `document.execCommand`
3. **窗口操作 API**：✅ 已验证，注意命名差异（destroyWindow/recover）
4. **MenuEvent**：✅ 已设计回调机制（Phase 2）

### 需要注意的限制

1. **页面构建时序**：菜单必须等待页面全部构建完成后才能展示（menu_control.md:57）
2. **位置精度**：API 12-19 使用 `offset`，API 20+ 可用 `anchorPosition` 精确定位
3. **restore vs recover**：OHOS 有两个恢复 API：
   - `restore()` - 仅用于从 minimize 恢复（仅 2in1，仅主窗口）
   - `recover()` - 从 maximize/fullscreen/split 恢复（更通用）
4. **hideOthers/showAll**：❌ 不支持（跨应用操作，OHOS 不允许）
5. **about()**：⚠️ 需自定义 AlertDialog 实现
6. **多 WebView 支持**：预定义菜单项需要正确的 WebView controller 引用
7. **嵌套 submenu**：✅ 已验证支持（menu.md 示例 300-338 行）

## 九、参考文档索引

### OHOS ArkUI Menu API
- [Menu 组件](reference/menu.md) - Menu 容器、SubMenuExpandingMode、示例代码
- [MenuItem 组件](reference/menu_item.md) - MenuItem 属性、onClick/onChange 事件
- [MenuItemGroup](reference/menu_item_group.md) - 菜单项分组
- [bindContextMenu](reference/menu_control.md) - 菜单弹出控制（核心 API）
- [ContextMenu.close](reference/context_menu.md) - 关闭菜单方法
- [promptAction](reference/prompt_action.md) - ActionMenu 备选方案

### 实现设计文档
- [README.md](README.md) - 阶段索引和概览
- [impl/phase1-infrastructure.md](impl/phase1-infrastructure.md) - 基础架构
- [impl/phase2-core-types.md](impl/phase2-core-types.md) - 核心类型 + MenuEvent
- [impl/phase3-predefined.md](impl/phase3-predefined.md) - PredefinedMenuItem
- [impl/phase4-popup-integration.md](impl/phase4-popup-integration.md) - Popup 实现
- [impl/phase5-integration-testing.md](impl/phase5-integration-testing.md) - 测试方案
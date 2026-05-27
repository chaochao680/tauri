# Phase 3: PredefinedMenuItem 实现

## 目标
在 muda::platform_impl::ohos 和 openharmony-ability 中实现所有 PredefinedMenuItem 功能，包括剪贴板操作、窗口操作和应用退出。

## 依赖
- Phase 0: muda OHOS 后端完成
- Phase 2: 核心菜单类型验证完成

## 架构说明

> **重要**：PredefinedMenuItem 的实现分层：
>
> ```
> tauri::menu::PredefinedMenuItem
>     ↓
> muda::platform_impl::ohos::MenuChild (存储 predefined_item_type)
>     ↓
> openharmony-ability::PredefinedActionExecutor (执行具体操作)
>     ↓
> OHOS API (runJavaScript, Window API, context.terminateSelf)
> ```
>
> - muda 层：存储 PredefinedMenuItemType，构建 ArkTsMenuItem
> - openharmony-ability 层：执行剪贴板、窗口操作

## 工作内容

### 3.1 剪贴板操作实现

**剪贴板操作使用 `runJavaScript` + `document.execCommand`**

**文件**: `openharmony-ability/native_ability/src/main/ets/helper/predefined.ets`

```typescript
import web_webview from '@ohos.web.webview';

/**
 * 剪贴板操作执行器
 * 通过 WebviewController.runJavaScript 执行
 */
export class ClipboardExecutor {
  private controller: web_webview.WebviewController | null = null;

  setController(controller: web_webview.WebviewController): void {
    this.controller = controller;
  }

  async copy(): Promise<void> {
    await this.controller?.runJavaScript('document.execCommand("copy")');
  }

  async cut(): Promise<void> {
    await this.controller?.runJavaScript('document.execCommand("cut")');
  }

  async paste(): Promise<void> {
    // paste 需要 API 权限或用户交互触发
    await this.controller?.runJavaScript('document.execCommand("paste")');
  }

  async selectAll(): Promise<void> {
    await this.controller?.runJavaScript('document.execCommand("selectAll")');
  }

  async undo(): Promise<void> {
    await this.controller?.runJavaScript('document.execCommand("undo")');
  }

  async redo(): Promise<void> {
    await this.controller?.runJavaScript('document.execCommand("redo")');
  }
}
```

### 3.2 窗口操作实现

**窗口操作使用 OHOS Window API**

> **API 版本要求**（来自 OHOS Window 文档）：
> - `minimize()` - API 12+
> - `maximize()` - API 12+
> - `recover()` - API 12+
> - `restore()` - API 14+（仅 2in1 设备）
> - `destroyWindow()` - API 6+

**文件**: `openharmony-ability/native_ability/src/main/ets/helper/window_ops.ets`

```typescript
import window from '@ohos.window';
import common from '@ohos.app.ability.common';

export interface WindowOps {
  minimize(): Promise<void>;
  maximize(): Promise<void>;
  recover(): Promise<void>;
  destroyWindow(): Promise<void>;
  setFullScreen(fullScreen: boolean): Promise<void>;
  hide(): Promise<void>;
}

/**
 * 窗口操作实现
 * 注意 OHOS API 命名差异：
 * - close → destroyWindow
 * - restore → recover (从 maximize/fullscreen)
 * - restore → restore (从 minimize，仅 2in1)
 * - fullscreen → maximize(ENTER_IMMERSIVE)
 */
export class WindowOpsExecutor implements WindowOps {
  private win: window.Window | null = null;
  private context: common.UIAbilityContext | null = null;

  setWindow(win: window.Window): void {
    this.win = win;
  }

  setContext(context: common.UIAbilityContext): void {
    this.context = context;
  }

  async minimize(): Promise<void> {
    if (!this.win) throw new Error('Window not set');
    await this.win.minimize();
  }

  async maximize(): Promise<void> {
    if (!this.win) throw new Error('Window not set');
    await this.win.maximize();
  }

  async recover(): Promise<void> {
    if (!this.win) throw new Error('Window not set');
    // recover() 从 maximize/fullscreen/split 恢复
    await this.win.recover();
  }

  async destroyWindow(): Promise<void> {
    if (!this.win) throw new Error('Window not set');
    await this.win.destroyWindow();
  }

  async setFullScreen(fullScreen: boolean): Promise<void> {
    if (!this.win) throw new Error('Window not set');
    if (fullScreen) {
      // 使用 maximize(ENTER_IMMERSIVE) 实现全屏
      await this.win.maximize(window.MaximizePresentation.ENTER_IMMERSIVE);
    } else {
      // 从全屏恢复
      await this.win.recover();
    }
  }

  async hide(): Promise<void> {
    // OHOS 主窗口 hide = minimize
    // 子窗口 minimize = hide
    await this.minimize();
  }
}
```

### 3.3 应用退出实现

```typescript
export class AppExecutor {
  private context: common.UIAbilityContext | null = null;

  setContext(context: common.UIAbilityContext): void {
    this.context = context;
  }

  async quit(code: number = 0): Promise<void> {
    if (!this.context) throw new Error('Context not set');
    await this.context.terminateSelf();
  }
}
```

### 3.4 PredefinedMenuItem 工厂

**文件**: `openharmony-ability/crates/ability/src/menu/predefined.rs`

```rust
use crate::menu::types::MenuItemData;
use napi::bindgen_prelude::*;

/**
 * 预定义菜单项类型
 * 注意：OHOS 有两个"恢复"API：
 * - Recover: 从 maximize/fullscreen 恢复（API 12+）
 * - Restore: 从 minimize 恢复（API 14+，仅 2in1 设备主窗口）
 */
#[derive(Debug, Clone, Copy)]
pub enum PredefinedType {
    Copy,
    Cut,
    Paste,
    SelectAll,
    Undo,
    Redo,
    Minimize,
    Maximize,
    Recover,     // 从 maximize/fullscreen 恢复（API 12+）
    Restore,     // 从 minimize 恢复（API 14+，仅 2in1）
    DestroyWindow,
    Quit,
    Hide,
    HideOthers,  // Not supported on OHOS
    ShowAll,     // Not supported on OHOS
    About,       // Requires custom implementation
    Separator,
}

impl PredefinedType {
    pub fn to_string(&self) -> &'static str {
        match self {
            Self::Copy => "copy",
            Self::Cut => "cut",
            Self::Paste => "paste",
            Self::SelectAll => "selectAll",
            Self::Undo => "undo",
            Self::Redo => "redo",
            Self::Minimize => "minimize",
            Self::Maximize => "maximize",
            Self::Recover => "recover",
            Self::Restore => "restore",
            Self::DestroyWindow => "destroyWindow",
            Self::Quit => "quit",
            Self::Hide => "hide",
            Self::HideOthers => "hideOthers",
            Self::ShowAll => "showAll",
            Self::About => "about",
            Self::Separator => "separator",
        }
    }

    pub fn display_text(&self) -> &'static str {
        match self {
            Self::Copy => "Copy",
            Self::Cut => "Cut",
            Self::Paste => "Paste",
            Self::SelectAll => "Select All",
            Self::Undo => "Undo",
            Self::Redo => "Redo",
            Self::Minimize => "Minimize",
            Self::Maximize => "Maximize",
            Self::Recover => "Restore (from Maximize)",
            Self::Restore => "Restore (from Minimize)",
            Self::DestroyWindow => "Close Window",
            Self::Quit => "Quit",
            Self::Hide => "Hide",
            Self::HideOthers => "Hide Others",
            Self::ShowAll => "Show All",
            Self::About => "About",
            Self::Separator => "",
        }
    }

    pub fn is_supported_on_ohos(&self) -> bool {
        match self {
            Self::HideOthers | Self::ShowAll => false,
            _ => true,
        }
    }

    pub fn accelerator(&self) -> Option<&'static str> {
        match self {
            Self::Copy => Some("Ctrl+C"),
            Self::Cut => Some("Ctrl+X"),
            Self::Paste => Some("Ctrl+V"),
            Self::SelectAll => Some("Ctrl+A"),
            Self::Undo => Some("Ctrl+Z"),
            Self::Redo => Some("Ctrl+Y"),
            Self::Minimize => Some("Ctrl+M"),
            _ => None,
        }
    }
}

#[napi]
pub struct PredefinedMenuItem {
    predefined_type: PredefinedType,
    id: String,
    text: Option<String>,
    accelerator: Option<String>,
}

#[napi]
impl PredefinedMenuItem {
    #[napi(factory)]
    pub fn copy(id: Option<String>, text: Option<String>) -> Self {
        Self {
            predefined_type: PredefinedType::Copy,
            id: id.unwrap_or_else(|| "predefined_copy".to_string()),
            text,
            accelerator: Some("Ctrl+C".to_string()),
        }
    }

    #[napi(factory)]
    pub fn cut(id: Option<String>, text: Option<String>) -> Self {
        Self {
            predefined_type: PredefinedType::Cut,
            id: id.unwrap_or_else(|| "predefined_cut".to_string()),
            text,
            accelerator: Some("Ctrl+X".to_string()),
        }
    }

    #[napi(factory)]
    pub fn paste(id: Option<String>, text: Option<String>) -> Self {
        Self {
            predefined_type: PredefinedType::Paste,
            id: id.unwrap_or_else(|| "predefined_paste".to_string()),
            text,
            accelerator: Some("Ctrl+V".to_string()),
        }
    }

    #[napi(factory)]
    pub fn select_all(id: Option<String>, text: Option<String>) -> Self {
        Self {
            predefined_type: PredefinedType::SelectAll,
            id: id.unwrap_or_else(|| "predefined_selectAll".to_string()),
            text,
            accelerator: Some("Ctrl+A".to_string()),
        }
    }

    #[napi(factory)]
    pub fn undo(id: Option<String>, text: Option<String>) -> Self {
        Self {
            predefined_type: PredefinedType::Undo,
            id: id.unwrap_or_else(|| "predefined_undo".to_string()),
            text,
            accelerator: Some("Ctrl+Z".to_string()),
        }
    }

    #[napi(factory)]
    pub fn redo(id: Option<String>, text: Option<String>) -> Self {
        Self {
            predefined_type: PredefinedType::Redo,
            id: id.unwrap_or_else(|| "predefined_redo".to_string()),
            text,
            accelerator: Some("Ctrl+Y".to_string()),
        }
    }

    #[napi(factory)]
    pub fn minimize(id: Option<String>, text: Option<String>) -> Self {
        Self {
            predefined_type: PredefinedType::Minimize,
            id: id.unwrap_or_else(|| "predefined_minimize".to_string()),
            text,
            accelerator: Some("Ctrl+M".to_string()),
        }
    }

    #[napi(factory)]
    pub fn maximize(id: Option<String>, text: Option<String>) -> Self {
        Self {
            predefined_type: PredefinedType::Maximize,
            id: id.unwrap_or_else(|| "predefined_maximize".to_string()),
            text,
            accelerator: None,
        }
    }

    #[napi(factory)]
    pub fn close_window(id: Option<String>, text: Option<String>) -> Self {
        Self {
            predefined_type: PredefinedType::DestroyWindow,
            id: id.unwrap_or_else(|| "predefined_close".to_string()),
            text,
            accelerator: Some("Ctrl+W".to_string()),
        }
    }

    #[napi(factory)]
    pub fn quit(id: Option<String>, text: Option<String>) -> Self {
        Self {
            predefined_type: PredefinedType::Quit,
            id: id.unwrap_or_else(|| "predefined_quit".to_string()),
            text,
            accelerator: Some("Ctrl+Q".to_string()),
        }
    }

    #[napi(factory)]
    pub fn separator(id: Option<String>) -> Self {
        Self {
            predefined_type: PredefinedType::Separator,
            id: id.unwrap_or_else(|| format!("separator_{}", uuid::Uuid::new_v4())),
            text: None,
            accelerator: None,
        }
    }

    fn to_data(&self) -> MenuItemData {
        MenuItemData {
            id: self.id.clone(),
            item_type: "predefined".to_string(),
            text: self.text.clone().or_else(|| {
                Some(self.predefined_type.display_text().to_string())
            }),
            enabled: Some(self.predefined_type.is_supported_on_ohos()),
            accelerator: self.accelerator.clone(),
            predefined_type: Some(self.predefined_type.to_string().to_string()),
            submenu_items: None,
        }
    }
}
```

### 3.5 PredefinedMenuItem 执行集成

**文件**: `openharmony-ability/crates/ability/src/menu/executor.rs`

```rust
use napi::bindgen_prelude::*;
use std::sync::Arc;
use tokio::sync::RwLock;

#[napi]
pub struct PredefinedExecutor {
    clipboard_fn: Arc<RwLock<Option<Function<String, ()>>>>,
    window_fn: Arc<RwLock<Option<Function<String, ()>>>>,
    app_fn: Arc<RwLock<Option<Function<String, ()>>>>,
}

#[napi]
impl PredefinedExecutor {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            clipboard_fn: Arc::new(RwLock::new(None)),
            window_fn: Arc::new(RwLock::new(None)),
            app_fn: Arc::new(RwLock::new(None)),
        }
    }

    #[napi]
    pub async fn set_clipboard_handler(&mut self, handler: Function<String, ()>) -> Result<()> {
        let mut guard = self.clipboard_fn.write().await;
        *guard = Some(handler);
        Ok(())
    }

    #[napi]
    pub async fn set_window_handler(&mut self, handler: Function<String, ()>) -> Result<()> {
        let mut guard = self.window_fn.write().await;
        *guard = Some(handler);
        Ok(())
    }

    #[napi]
    pub async fn set_app_handler(&mut self, handler: Function<String, ()>) -> Result<()> {
        let mut guard = self.app_fn.write().await;
        *guard = Some(handler);
        Ok(())
    }

    pub async fn execute(&self, predefined_type: &str) -> Result<()> {
        let category = match predefined_type {
            "copy" | "cut" | "paste" | "selectAll" | "undo" | "redo" => "clipboard",
            "minimize" | "maximize" | "recover" | "destroyWindow" | "hide" => "window",
            "quit" => "app",
            _ => return Ok(()),
        };

        let handler = match category {
            "clipboard" => self.clipboard_fn.read().await.clone(),
            "window" => self.window_fn.read().await.clone(),
            "app" => self.app_fn.read().await.clone(),
            _ => None,
        };

        if let Some(fn_handler) = handler {
            fn_handler.call(predefined_type.to_string()).await?;
        }

        Ok(())
    }
}
```

## 验证方法

> **验证策略**：
> - 内部接口 → Rust UT
> - 外部接口 → api-demo（autotest 或 manual test）

### 3.5 Rust UT（内部接口）

**验证范围**：
| 接口 | 类型 | 测试方法 |
|------|------|---------|
| `PredefinedMenuItem::copy()` | 内部 | Rust UT |
| `PredefinedMenuItem::cut()` | 内部 | Rust UT |
| `PredefinedMenuItem::separator()` | 内部 | Rust UT |
| `PredefinedType::is_supported_on_ohos()` | 内部 | Rust UT |
| `PredefinedType::display_text()` | 内部 | Rust UT |
| `PredefinedType::accelerator()` | 内部 | Rust UT |

**测试文件**: `menu/predefined.rs`

```rust
#[cfg(all(test, target_env = "ohos"))]
mod tests {
    use super::*;

    #[test]
    fn test_predefined_type_serialization() {
        let predefined = PredefinedType::Copy;
        let json = serde_json::to_string(&predefined).unwrap();
        assert_eq!(json, "\"copy\"");
    }

    #[test]
    fn test_predefined_copy_factory() {
        let item = PredefinedMenuItem::copy(None, None);
        assert_eq!(item.predefined_type, PredefinedType::Copy);
        assert_eq!(item.accelerator, Some("Ctrl+C".to_string()));
        assert!(item.predefined_type.is_supported_on_ohos());
    }

    #[test]
    fn test_predefined_minimize_factory() {
        let item = PredefinedMenuItem::minimize(None, None);
        assert_eq!(item.predefined_type, PredefinedType::Minimize);
        assert!(item.predefined_type.is_supported_on_ohos());
    }

    #[test]
    fn test_predefined_separator_factory() {
        let item = PredefinedMenuItem::separator(None);
        assert_eq!(item.predefined_type, PredefinedType::Separator);
    }

    #[test]
    fn test_unsupported_items() {
        assert!(!PredefinedType::HideOthers.is_supported_on_ohos());
        assert!(!PredefinedType::ShowAll.is_supported_on_ohos());
    }

    #[test]
    fn test_display_text() {
        assert_eq!(PredefinedType::Copy.display_text(), "Copy");
        assert_eq!(PredefinedType::Minimize.display_text(), "Minimize");
        assert_eq!(PredefinedType::Recover.display_text(), "Restore");
    }

    #[test]
    fn test_accelerator_mapping() {
        assert_eq!(PredefinedType::Copy.accelerator(), Some("Ctrl+C"));
        assert_eq!(PredefinedType::Cut.accelerator(), Some("Ctrl+X"));
        assert_eq!(PredefinedType::Undo.accelerator(), Some("Ctrl+Z"));
        assert_eq!(PredefinedType::Minimize.accelerator(), Some("Ctrl+M"));
        assert_eq!(PredefinedType::Maximize.accelerator(), None);
    }
}
```

**执行命令**:
```bash
# 运行 predefined 模块 Rust UT
bash .claude/skills/ohos-rust-ut/scripts/run-ut.sh menu::predefined

# 输出预期
running 7 tests
test menu::predefined::tests::test_predefined_type_serialization ... ok
test menu::predefined::tests::test_predefined_copy_factory ... ok
test menu::predefined::tests::test_predefined_minimize_factory ... ok
test menu::predefined::tests::test_predefined_separator_factory ... ok
test menu::predefined::tests::test_unsupported_items ... ok
test menu::predefined::tests::test_display_text ... ok
test menu::predefined::tests::test_accelerator_mapping ... ok
test result: ok. 7 passed; 0 failed
```

### 3.6 api-demo 自动测试（外部接口）

**验证范围**：
| 接口 | 类型 | 测试方法 | category |
|------|------|---------|----------|
| `PredefinedMenuItem.copy()` | 外部 | api-demo autotest | auto |
| `PredefinedMenuItem.cut()` | 外部 | api-demo autotest | auto |
| `PredefinedMenuItem.paste()` | 外部 | api-demo autotest | auto |
| `PredefinedMenuItem.separator()` | 外部 | api-demo autotest | auto |

**测试文件**: `examples/api/src/lib/tests/core.ts`

```typescript
// === PredefinedMenuItem 测试 ===

import { PredefinedMenuItem } from '@tauri-apps/api/menu';

{
  name: '@tauri-apps/api/menu.PredefinedMenuItem.separator',
  category: 'auto',
  async fn() {
    const sep = await PredefinedMenuItem.separator();
    assert(sep !== undefined, 'separator returned undefined');
  },
},
{
  name: '@tauri-apps/api/menu.PredefinedMenuItem.copy',
  category: 'auto',
  async fn() {
    const copy = await PredefinedMenuItem.copy();
    assert(copy !== undefined, 'copy returned undefined');
  },
},
{
  name: '@tauri-apps/api/menu.PredefinedMenuItem.cut',
  category: 'auto',
  async fn() {
    const cut = await PredefinedMenuItem.cut();
    assert(cut !== undefined);
  },
},
{
  name: '@tauri-apps/api/menu.PredefinedMenuItem.paste',
  category: 'auto',
  async fn() {
    const paste = await PredefinedMenuItem.paste();
    assert(paste !== undefined);
  },
},
{
  name: '@tauri-apps/api/menu.PredefinedMenuItem.selectAll',
  category: 'auto',
  async fn() {
    const selectAll = await PredefinedMenuItem.selectAll();
    assert(selectAll !== undefined);
  },
},
{
  name: '@tauri-apps/api/menu.PredefinedMenuItem.undo',
  category: 'auto',
  async fn() {
    const undo = await PredefinedMenuItem.undo();
    assert(undo !== undefined);
  },
},
{
  name: '@tauri-apps/api/menu.PredefinedMenuItem.redo',
  category: 'auto',
  async fn() {
    const redo = await PredefinedMenuItem.redo();
    assert(redo !== undefined);
  },
},
```

**执行命令**:
```bash
bash .claude/skills/ohos-build/scripts/run-tests.sh
```

### 3.7 api-demo 手动测试（外部接口 - 需 WebView/Window）

**验证范围**：
| 接口 | 类型 | 测试方法 | 原因 |
|------|------|---------|------|
| ClipboardExecutor (copy/cut/paste) | 外部 | api-demo manual | 需 WebView + 选中文字 |
| WindowOpsExecutor (minimize) | 外部 | api-demo manual | 窗口状态变化需观察 |
| WindowOpsExecutor (maximize) | 外部 | api-demo manual | 窗口状态变化需观察 |
| WindowOpsExecutor (destroyWindow) | 外部 | api-demo manual | 窗口关闭需观察 |
| AppExecutor (quit) | 外部 | api-demo manual | 应用退出需观察 |

**测试文件**: `examples/api/src/views/TestRunner.svelte`

```typescript
// 手动测试：剪贴板操作
import { Menu, PredefinedMenuItem } from '@tauri-apps/api/menu';

async function manualClipboardCopy() {
  await wrapManual('menu.clipboard.copy', async () => {
    // 前置条件：在 WebView 中选中文字
    const copy = await PredefinedMenuItem.copy({ text: '复制' });
    const menu = await Menu.new({ items: [copy] });
    await menu.popup();
    onMessage('Select text in WebView, click "Copy", verify clipboard');
  });
}

async function manualClipboardPaste() {
  await wrapManual('menu.clipboard.paste', async () => {
    const paste = await PredefinedMenuItem.paste({ text: '粘贴' });
    const menu = await Menu.new({ items: [paste] });
    await menu.popup();
    onMessage('Click "Paste", verify text inserted in WebView');
  });
}

// 手动测试：窗口操作
async function manualWindowMinimize() {
  await wrapManual('menu.window.minimize', async () => {
    const minimize = await PredefinedMenuItem.minimize({ text: '最小化' });
    const menu = await Menu.new({ items: [minimize] });
    await menu.popup();
    onMessage('Click "Minimize", window should minimize');
  });
}

async function manualWindowMaximize() {
  await wrapManual('menu.window.maximize', async () => {
    const maximize = await PredefinedMenuItem.maximize({ text: '最大化' });
    const menu = await Menu.new({ items: [maximize] });
    await menu.popup();
    onMessage('Click "Maximize", window should maximize');
  });
}

async function manualWindowClose() {
  await wrapManual('menu.window.close', async () => {
    const close = await PredefinedMenuItem.closeWindow({ text: '关闭窗口' });
    const menu = await Menu.new({ items: [close] });
    await menu.popup();
    onMessage('Click "Close", window should close (app will exit)');
  });
}

async function manualAppQuit() {
  await wrapManual('menu.app.quit', async () => {
    const quit = await PredefinedMenuItem.quit({ text: '退出应用' });
    const menu = await Menu.new({ items: [quit] });
    await menu.popup();
    onMessage('Click "Quit", app should terminate');
  });
}
```

```svelte
<!-- Manual Tests 按钮 -->
<div class="manual-tests">
  <h3>Clipboard Tests</h3>
  <button class="btn" onclick={manualClipboardCopy}>
    Copy (select text first)
  </button>
  <button class="btn" onclick={manualClipboardPaste}>
    Paste
  </button>
  
  <h3>Window Tests</h3>
  <button class="btn" onclick={manualWindowMinimize}>
    Minimize
  </button>
  <button class="btn" onclick={manualWindowMaximize}>
    Maximize
  </button>
  <button class="btn" onclick={manualWindowClose}>
    Close Window
  </button>
  
  <h3>App Tests</h3>
  <button class="btn" onclick={manualAppQuit}>
    Quit App
  </button>
</div>
```

### 3.8 相关 SKILL 文档

- [ohos-rust-ut](../../.claude/skills/ohos-rust-ut/SKILL.md) - Rust UT 执行
- [frontend-api-testing](../../.claude/skills/frontend-api-testing/SKILL.md) - 前端测试编写
- [ohos-build](../../.claude/skills/ohos-build/SKILL.md) - OHOS 构建
- [arkweb-adapter](../../.claude/skills/arkweb-adapter/SKILL.md) - WebView runJavaScript 参考

## 工期
- 2 天

## 输出物
- `predefined.ets` - 剪贴板执行器
- `window_ops.ets` - 窗口操作执行器
- `menu/predefined.rs` - Rust 预定义菜单项
- `menu/executor.rs` - Rust 执行器
- 单元测试和集成测试通过

## 参考文档
- Window API 版本要求详见 `../ohos-menu-api-design.md` 第 920-980 行
- 剪贴板操作使用 `document.execCommand()` 验证于 `arkweb-adapter` 技能文档

## API 版本说明

| OHOS API | 版本要求 | 对应 Tauri API | 说明 |
|----------|---------|---------------|------|
| `minimize()` | API 12+ | `PredefinedMenuItem.minimize()` | 窗口最小化 |
| `maximize()` | API 12+ | `PredefinedMenuItem.maximize()` | 窗口最大化 |
| `recover()` | API 12+ | `PredefinedMenuItem.restore()` | 从 maximize/fullscreen 恢复 |
| `restore()` | API 14+ | `PredefinedMenuItem.restore()` | 从 minimize 恢复，仅 2in1 设备主窗口 |
| `destroyWindow()` | API 6+ | `PredefinedMenuItem.closeWindow()` | 关闭窗口 |
| `terminateSelf()` | API 9+ | `PredefinedMenuItem.quit()` | 退出应用 |

**重要说明**：
- Tauri 的 `restore()` API 在 OHOS 上需要区分两种情况：
  1. 从 maximize 恢复 → 使用 `window.recover()` (API 12+)
  2. 从 minimize 恢复 → 使用 `window.restore()` (API 14+, 仅 2in1)
- 非 2in1 设备上从 minimize 恢复无法实现，建议返回错误提示

---

## 实现状态

### 已完成 (2026-05-15)

**Phase 3 完成度: 100%**

| 任务 | 状态 | 说明 |
|------|------|------|
| ArkTS ClipboardExecutor | ✅ 完成 | copy/cut/paste/selectAll/undo/redo |
| ArkTS WindowOpsExecutor | ✅ 完成 | minimize/maximize/recover/destroyWindow/setFullScreen/hide |
| ArkTS AppExecutor | ✅ 完成 | quit (terminateSelf) |
| Rust PredefinedType | ✅ 完成 | 17 种类型 + PartialEq derive |
| Rust PredefinedMenuItem | ✅ 完成 | 17 个 factory 方法 |
| Rust UT | ✅ 完成 | 12 个测试全部通过 |
| 编译验证 | ✅ 完成 | openharmony-ability 编译通过 |

### 文件清单

| 文件 | 行数 | 说明 |
|------|------|------|
| `openharmony-ability/crates/ability/src/menu/predefined.rs` | 390 | PredefinedType + PredefinedMenuItem + 17 factory + 12 tests |
| `openharmony-ability/native_ability/src/main/ets/helper/predefined.ets` | 85 | ClipboardExecutor + WindowOpsExecutor + AppExecutor |
| `openharmony-ability/native_ability/src/main/ets/helper/menu.ets` | 219 | PredefinedActionExecutor (集成到 TauriMenuManager) |

### 预定义菜单项工厂方法 (17 个)

| 方法 | 状态 | PredefinedType | 说明 |
|------|------|----------------|------|
| `copy()` | ✅ | Copy | Ctrl+C |
| `cut()` | ✅ | Cut | Ctrl+X |
| `paste()` | ✅ | Paste | Ctrl+V |
| `select_all()` | ✅ | SelectAll | Ctrl+A |
| `undo()` | ✅ | Undo | Ctrl+Z |
| `redo()` | ✅ | Redo | Ctrl+Y |
| `minimize()` | ✅ | Minimize | Ctrl+M |
| `maximize()` | ✅ | Maximize | - |
| `close_window()` | ✅ | DestroyWindow | Ctrl+W |
| `quit()` | ✅ | Quit | Ctrl+Q |
| `separator()` | ✅ | Separator | - |
| `recover()` | ✅ | Recover | 从 maximize/fullscreen 恢复 |
| `restore()` | ✅ | Restore | 从 minimize 恢复 (API 14+, 2in1) |
| `hide()` | ✅ | Hide | 隐藏 (minimize) |
| `hide_others()` | ✅ | HideOthers | OHOS 不支持 (disabled) |
| `show_all()` | ✅ | ShowAll | OHOS 不支持 (disabled) |
| `about()` | ✅ | About | 需要 AlertDialog |

### 单元测试 (12 个)

| 测试 | 状态 | 验证内容 |
|------|------|----------|
| `test_predefined_copy_factory` | ✅ | Copy type + accelerator + supported |
| `test_predefined_minimize_factory` | ✅ | Minimize type + supported |
| `test_predefined_separator_factory` | ✅ | Separator type |
| `test_unsupported_items` | ✅ | HideOthers/ShowAll not supported |
| `test_display_text` | ✅ | Copy/Minimize/Recover display text |
| `test_accelerator_mapping` | ✅ | Ctrl+C/X/Z/M + None |
| `test_recover_factory` | ✅ | Recover type + id prefix |
| `test_restore_factory` | ✅ | Restore type + id prefix |
| `test_hide_factory` | ✅ | Hide type + id prefix |
| `test_hide_others_factory` | ✅ | HideOthers type + not supported |
| `test_show_all_factory` | ✅ | ShowAll type + not supported |
| `test_about_factory` | ✅ | About type + id prefix |

### 架构差异说明

| 设计文档 | 实际实现 | 影响 |
|----------|----------|------|
| `executor.rs` 独立文件 | 执行逻辑在 `PredefinedActionExecutor` (ArkTS) | 无影响 - ArkTS 侧直接执行 |
| `PredefinedExecutor` (Rust handler) | 不需要 - ArkTS 直接调用 OHOS API | 简化架构，减少 FFI 往返 |
| 10 个 factory 方法 | 17 个 factory 方法 | 更完整 - 覆盖所有 PredefinedType |
| 6 个测试 | 12 个测试 | 更完整 - 覆盖所有新增 factory |

### 编译状态

| 组件 | 命令 | 状态 |
|------|------|------|
| openharmony-ability | `cargo check --target aarch64-unknown-linux-ohos --features menu` | ✅ 通过 (1 warning: unused `accelerator()`) |
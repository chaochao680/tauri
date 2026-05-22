# Phase 2: 核心菜单类型

## 目标
在 muda::platform_impl::ohos 中实现 Menu、MenuItem、Submenu、MenuEvent 的完整创建和配置功能。

## 依赖
- Phase 0: muda OHOS 后端完成（已在 Phase 0 实现）
- Phase 1: openharmony-ability ArkTS 类型定义完成

## 架构说明

> **重要**：核心菜单类型实现已在 Phase 0 的 muda::platform_impl::ohos 中完成。
>
> 本 Phase 的工作：
> 1. 验证 muda::platform_impl::ohos 的核心类型实现
> 2. 在 openharmony-ability 中实现 ArkTS 菜单管理器（供 muda 调用）
> 3. 处理 PredefinedMenuItem 执行逻辑（部分在此 Phase，部分在 Phase 3）

## 工作内容

### 2.1 menu.ets 实现

**文件**: `openharmony-ability/native_ability/src/main/ets/helper/menu.ets`

```typescript
import { TauriMenuItemData, PredefinedType } from './menu_types';
import web_webview from '@ohos.web.webview';
import window from '@ohos.window';

/**
 * 预定义菜单项执行器
 */
export class PredefinedActionExecutor {
  private controller: web_webview.WebviewController | null = null;
  private win: window.Window | null = null;
  private exitFn: (code: number) => void;

  constructor(exitFn: (code: number) => void) {
    this.exitFn = exitFn;
  }

  setController(controller: web_webview.WebviewController): void {
    this.controller = controller;
  }

  setWindow(win: window.Window): void {
    this.win = win;
  }

  async execute(type: PredefinedType): Promise<void> {
    switch (type) {
      case 'copy':
        await this.controller?.runJavaScript('document.execCommand("copy")');
        break;
      case 'cut':
        await this.controller?.runJavaScript('document.execCommand("cut")');
        break;
      case 'paste':
        await this.controller?.runJavaScript('document.execCommand("paste")');
        break;
      case 'selectAll':
        await this.controller?.runJavaScript('document.execCommand("selectAll")');
        break;
      case 'undo':
        await this.controller?.runJavaScript('document.execCommand("undo")');
        break;
      case 'redo':
        await this.controller?.runJavaScript('document.execCommand("redo")');
        break;
      case 'minimize':
        await this.win?.minimize();
        break;
      case 'maximize':
        await this.win?.maximize();
        break;
      case 'recover':
        await this.win?.recover();
        break;
      case 'destroyWindow':
        await this.win?.destroyWindow();
        break;
      case 'quit':
        this.exitFn(0);
        break;
      case 'hide':
        await this.win?.minimize();  // OHOS 主窗口 minimize = 隐藏
        break;
      case 'hideOthers':
      case 'showAll':
        console.warn(`${type} not supported on OHOS`);
        break;
      case 'about':
        console.warn('about() requires custom AlertDialog implementation');
        break;
    }
  }
}

/**
 * 菜单管理器
 * 使用状态变量控制菜单弹出
 */
export class TauriMenuManager {
  private executor: PredefinedActionExecutor;
  private onMenuClick: ((menuId: string, itemId: string) => void) | null = null;
  private menus: Map<string, TauriMenuItemData[]> = new Map();
  
  // 状态变量（供 UI 绑定）
  private isShownState: boolean = false;
  private currentMenuIdState: string = '';
  private offsetState: { x: number; y: number } = { x: 0, y: 0 };

  constructor(executor: PredefinedActionExecutor) {
    this.executor = executor;
  }

  // === 菜单创建 ===
  
  createMenu(id: string, items: TauriMenuItemData[]): void {
    this.menus.set(id, items);
  }

  // === 菜单创建（含图标属性支持） ===
  
  createMenuItem(options: {
    id: string;
    text: string;
    enabled?: boolean;
    accelerator?: string;
    // 图标属性（参考 menu_item.md 第 43-76 行）
    startIcon?: ResourceStr;
    endIcon?: ResourceStr;
    symbolStartIcon?: SymbolGlyphOptions;  // API 12+
    symbolEndIcon?: SymbolGlyphOptions;    // API 12+
    // 选中状态属性（参考 menu_item.md 第 86-113 行）
    selected?: boolean;
    selectIcon?: ResourceStr;
  }): TauriMenuItemData {
    return {
      id: options.id,
      type: 'item',
      text: options.text,
      enabled: options.enabled ?? true,
      accelerator: options.accelerator,
      startIcon: options.startIcon,
      endIcon: options.endIcon,
      symbolStartIcon: options.symbolStartIcon,
      symbolEndIcon: options.symbolEndIcon,
      selected: options.selected,
      selectIcon: options.selectIcon,
    };
  }

  createSeparator(): TauriMenuItemData {
    return { id: `sep_${Date.now()}`, type: 'separator' };
  }

  createPredefined(type: PredefinedType): TauriMenuItemData {
    const textMap: Record<PredefinedType, string> = {
      copy: 'Copy',
      cut: 'Cut',
      paste: 'Paste',
      selectAll: 'Select All',
      undo: 'Undo',
      redo: 'Redo',
      minimize: 'Minimize',
      maximize: 'Maximize',
      recover: 'Restore (from Maximize)',
      restore: 'Restore (from Minimize)',
      destroyWindow: 'Close',
      quit: 'Quit',
      hide: 'Hide',
      hideOthers: 'Hide Others',
      showAll: 'Show All',
      about: 'About',
    };
    return {
      id: `predefined_${type}`,
      type: 'predefined',
      text: textMap[type],
      predefinedType: type,
    };
  }

  createSubmenu(id: string, text: string, items: TauriMenuItemData[]): TauriMenuItemData {
    return {
      id,
      type: 'submenu',
      text,
      submenuItems: items,
    };
  }

  // === 菜单操作 ===

  append(menuId: string, item: TauriMenuItemData): void {
    const items = this.menus.get(menuId);
    if (items) {
      items.push(item);
    }
  }

  remove(menuId: string, itemId: string): void {
    const items = this.menus.get(menuId);
    if (items) {
      const index = items.findIndex(i => i.id === itemId);
      if (index >= 0) {
        items.splice(index, 1);
      }
    }
  }

  // === Popup 控制 ===

  popup(menuId: string, x?: number, y?: number): void {
    this.currentMenuIdState = menuId;
    this.offsetState = { x: x ?? 0, y: y ?? 0 };
    this.isShownState = true;
  }

  close(): void {
    this.isShownState = false;
  }

  // === 状态访问器（供 UI 绑定） ===

  get isShown(): boolean {
    return this.isShownState;
  }

  get currentMenuId(): string {
    return this.currentMenuIdState;
  }

  get offset(): { x: number; y: number } {
    return this.offsetState;
  }

  getMenuItems(menuId: string): TauriMenuItemData[] {
    return this.menus.get(menuId) ?? [];
  }

  // === 回调设置 ===

  setOnMenuClick(callback: (menuId: string, itemId: string) => void): void {
    this.onMenuClick = callback;
  }

  // === 事件处理 ===

  /**
   * 处理菜单项点击
   * 注意：submenu 类型不应关闭菜单，而是展开子菜单（由 ArkUI 自动处理）
   */
  handleItemClick(item: TauriMenuItemData): void {
    if (item.type === 'predefined' && item.predefinedType) {
      this.executor.execute(item.predefinedType);
      this.close();  // 预定义项执行后关闭菜单
    } else if (item.type === 'item') {
      this.onMenuClick?.(this.currentMenuIdState, item.id);
      this.close();  // 普通项点击后关闭菜单
    } else if (item.type === 'separator') {
      // 分隔线无操作
    } else if (item.type === 'submenu') {
      // submenu 点击由 ArkUI 自动展开子菜单，不关闭主菜单
      // 参考 menu.md 第 300-338 行：MenuItem 的 builder 参数处理嵌套展开
    }
  }
}
```

### 2.2 Rust 菜单类型实现

**文件**: `openharmony-ability/crates/ability/src/menu/types.rs`

```rust
use napi::bindgen_prelude::*;
use serde::{Deserialize, Serialize};

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuItemData {
    pub id: String,
    #[serde(rename = "type")]
    pub item_type: String,
    pub text: Option<String>,
    pub enabled: Option<bool>,
    pub accelerator: Option<String>,
    pub predefined_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submenu_items: Option<Vec<MenuItemData>>,
}

#[napi]
pub struct Menu {
    id: String,
    manager: Reference<MenuManager>,
}

#[napi]
impl Menu {
    #[napi(constructor)]
    pub fn new(manager: Reference<MenuManager>, id: String) -> Result<Self> {
        Ok(Self { id, manager })
    }

    #[napi]
    pub async fn append(&self, item: Reference<MenuItem>) -> Result<()> {
        let data = item.to_data();
        self.manager.append_menu_item(self.id.clone(), data).await
    }

    #[napi]
    pub async fn items(&self) -> Result<Vec<MenuItemData>> {
        self.manager.get_menu_items(self.id.clone()).await
    }

    #[napi]
    pub async fn popup(&self, x: Option<f64>, y: Option<f64>) -> Result<()> {
        self.manager.popup_menu(self.id.clone(), x, y).await
    }
}

#[napi]
pub struct MenuItem {
    id: String,
    text: String,
    enabled: bool,
    accelerator: Option<String>,
}

#[napi]
impl MenuItem {
    #[napi(constructor)]
    pub fn new(id: Option<String>, text: String, enabled: Option<bool>, accelerator: Option<String>) -> Self {
        Self {
            id: id.unwrap_or_else(|| format!("item_{}", uuid::Uuid::new_v4())),
            text,
            enabled: enabled.unwrap_or(true),
            accelerator,
        }
    }

    fn to_data(&self) -> MenuItemData {
        MenuItemData {
            id: self.id.clone(),
            item_type: "item".to_string(),
            text: Some(self.text.clone()),
            enabled: Some(self.enabled),
            accelerator: self.accelerator.clone(),
            predefined_type: None,
            submenu_items: None,
        }
    }
}

#[napi]
pub struct Submenu {
    id: String,
    text: String,
    items: Vec<MenuItemData>,
}

#[napi]
impl Submenu {
    #[napi(constructor)]
    pub fn new(id: Option<String>, text: String) -> Self {
        Self {
            id: id.unwrap_or_else(|| format!("submenu_{}", uuid::Uuid::new_v4())),
            text,
            items: vec![],
        }
    }

    #[napi]
    pub async fn append(&mut self, item: MenuItemData) -> Result<()> {
        self.items.push(item);
        Ok(())
    }
}
```

### 2.3 Rust MenuManager 实现

**文件**: `openharmony-ability/crates/ability/src/menu/manager.rs`

```rust
use super::types::MenuItemData;
use napi::bindgen_prelude::*;
use std::collections::HashMap;

#[napi]
pub struct MenuManager {
    menus: HashMap<String, Vec<MenuItemData>>,
    callbacks: HashMap<String, Vec<Function<(), Void>>>,
}

#[napi]
impl MenuManager {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            menus: HashMap::new(),
            callbacks: HashMap::new(),
        }
    }

    #[napi]
    pub async fn create_menu(&mut self, id: String, items: Vec<MenuItemData>) -> Result<()> {
        self.menus.insert(id.clone(), items);
        self.callbacks.insert(id, vec![]);
        Ok(())
    }

    #[napi]
    pub async fn append_menu_item(&mut self, menu_id: String, item: MenuItemData) -> Result<()> {
        if let Some(items) = self.menus.get_mut(&menu_id) {
            items.push(item);
        }
        Ok(())
    }

    #[napi]
    pub async fn get_menu_items(&self, menu_id: String) -> Result<Vec<MenuItemData>> {
        Ok(self.menus.get(&menu_id).cloned().unwrap_or_default())
    }

    #[napi]
    pub async fn popup_menu(&self, menu_id: String, x: Option<f64>, y: Option<f64>) -> Result<()> {
        // 调用 ArkTS popup
        // 通过 NAPI 调用 TauriMenuManager.popup()
        Ok(())
    }

    #[napi]
    pub async fn destroy_menu(&mut self, menu_id: String) -> Result<()> {
        self.menus.remove(&menu_id);
        self.callbacks.remove(&menu_id);
        Ok(())
    }

    #[napi]
    pub async fn on_menu_click(&mut self, menu_id: String, callback: Function<(), Void>) -> Result<()> {
        if let Some(callbacks) = self.callbacks.get_mut(&menu_id) {
            callbacks.push(callback);
        }
        Ok(())
    }
}
```

### 2.4 MenuEvent 实现

**MenuEvent 是菜单项点击事件的载体，用于将 ArkTS 的点击事件传递到 Rust 的 AppHandle event loop。**

**文件**: `openharmony-ability/crates/ability/src/menu/event.rs`

```rust
use serde::{Deserialize, Serialize};

/// 菜单事件，当菜单项被激活时触发
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuEvent {
    /// 触发事件的菜单项 ID
    pub id: String,
}

impl MenuEvent {
    pub fn new(id: String) -> Self {
        Self { id }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

/// 事件监听器类型
pub type MenuEventListener = Box<dyn Fn(&MenuEvent) + Send + Sync>;

/// 事件分发器
pub struct MenuEventDispatcher {
    listeners: Vec<MenuEventListener>,
}

impl MenuEventDispatcher {
    pub fn new() -> Self {
        Self { listeners: vec![] }
    }

    pub fn add_listener<F: Fn(&MenuEvent) + Send + Sync + 'static>(&mut self, listener: F) {
        self.listeners.push(Box::new(listener));
    }

    pub fn dispatch(&self, event: &MenuEvent) {
        for listener in &self.listeners {
            listener(event);
        }
    }
}

impl Default for MenuEventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}
```

**文件**: `openharmony-ability/crates/ability/src/menu/mod.rs`

```rust
mod types;
mod manager;
mod event;  // 新增

pub use event::{MenuEvent, MenuEventDispatcher};
pub use types::{MenuItemData, Menu, MenuItem, Submenu};
pub use manager::MenuManager;

use napi::bindgen_prelude::*;

#[napi]
pub fn emit_menu_event(menu_id: String, item_id: String) -> Result<()> {
    // 调用全局事件分发器
    let event = MenuEvent::new(item_id);
    
    // 通过 NAPI 传递到 Tauri AppHandle
    // 此函数由 ArkTS 在 handleItemClick 时调用
    Ok(())
}
```

**ArkTS 事件触发（更新 menu.ets）**:

```typescript
// TauriMenuManager 中添加事件发送
export class TauriMenuManager {
  private eventDispatcher: (menuId: string, itemId: string) => void;

  setEventDispatcher(dispatcher: (menuId: string, itemId: string) => void): void {
    this.eventDispatcher = dispatcher;
  }

  handleItemClick(item: TauriMenuItemData): void {
    // 1. 执行操作
    if (item.type === 'predefined' && item.predefinedType) {
      this.executor.execute(item.predefinedType);
    }
    
    // 2. 发送事件到 Rust
    this.eventDispatcher?.(this.currentMenuIdState, item.id);
    
    // 3. 关闭菜单
    this.close();
  }
}
```

**Tauri 集成（Phase 5 详细实现）**:

```rust
// tauri/crates/tauri/src/menu/ohos.rs
impl<R: Runtime> AppHandle<R> {
    pub fn on_menu_event<F: Fn(&AppHandle<R>, MenuEvent) + Send + Sync + 'static>(
        &self,
        handler: F,
    ) {
        self.manager.menu.on_menu_event(handler);
    }
}

// 事件接收
fn on_ohos_menu_event(menu_id: String, item_id: String) {
    let event = MenuEvent { id: item_id };
    // 分发到所有监听器
    for listener in &app.manager.menu.global_event_listeners {
        listener(&app, event);
    }
}
```

## 验证方法

> **验证策略**：
> - 内部接口 → Rust UT
> - 外部接口 → api-demo（autotest 或 manual test）

### 2.5 Rust UT（内部接口）

**验证范围**：
| 接口 | 类型 | 测试方法 |
|------|------|---------|
| `Menu::new()` | 内部 | Rust UT |
| `MenuItem::new()` | 内部 | Rust UT |
| `Submenu::new()` | 内部 | Rust UT |
| `MenuManager::create_menu()` | 内部 | Rust UT |
| `MenuManager::append_menu_item()` | 内部 | Rust UT |
| `MenuManager::get_menu_items()` | 内部 | Rust UT |
| `MenuEvent::new()` | 内部 | Rust UT |
| `MenuEventDispatcher::add_listener()` | 内部 | Rust UT |
| `MenuEventDispatcher::dispatch()` | 内部 | Rust UT |

**测试文件**: `menu/types.rs`, `menu/manager.rs`, `menu/event.rs`

```rust
#[cfg(all(test, target_env = "ohos"))]
mod tests {
    use super::*;

    #[test]
    fn test_menu_item_data_serialization() {
        let data = MenuItemData {
            id: "item1".to_string(),
            item_type: "item".to_string(),
            text: Some("File".to_string()),
            enabled: Some(true),
            accelerator: Some("Ctrl+F".to_string()),
            predefined_type: None,
            submenu_items: None,
        };
        
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("\"id\":\"item1\""));
        assert!(json.contains("\"type\":\"item\""));
        assert!(json.contains("\"text\":\"File\""));
    }

    #[test]
    fn test_submenu_nested_items() {
        let submenu_data = MenuItemData {
            id: "submenu_1".to_string(),
            item_type: "submenu".to_string(),
            text: Some("File".to_string()),
            enabled: Some(true),
            accelerator: None,
            predefined_type: None,
            submenu_items: Some(vec![
                MenuItemData {
                    id: "item_1".to_string(),
                    item_type: "item".to_string(),
                    text: Some("Open".to_string()),
                    enabled: Some(true),
                    accelerator: None,
                    predefined_type: None,
                    submenu_items: None,
                },
            ]),
        };
        
        assert!(submenu_data.submenu_items.is_some());
        assert_eq!(submenu_data.submenu_items.unwrap().len(), 1);
    }

    #[test]
    fn test_menu_event_creation() {
        let event = MenuEvent::new("item123".to_string());
        assert_eq!(event.id(), "item123");
    }

    #[test]
    fn test_menu_event_dispatcher() {
        let mut dispatcher = MenuEventDispatcher::new();
        let mut received_events: Vec<String> = vec![];
        
        dispatcher.add_listener(|event| {
            received_events.push(event.id.clone());
        });
        
        let event = MenuEvent::new("test_item".to_string());
        dispatcher.dispatch(&event);
        
        assert_eq!(received_events.len(), 1);
        assert_eq!(received_events[0], "test_item");
    }

    #[test]
    fn test_multiple_listeners() {
        let mut dispatcher = MenuEventDispatcher::new();
        let mut count1 = 0;
        let mut count2 = 0;
        
        dispatcher.add_listener(|_| { count1 += 1; });
        dispatcher.add_listener(|_| { count2 += 1; });
        
        let event = MenuEvent::new("item".to_string());
        dispatcher.dispatch(&event);
        
        assert_eq!(count1, 1);
        assert_eq!(count2, 1);
    }
}
```

**执行命令**:
```bash
# 运行 menu 模块 Rust UT
bash .claude/skills/ohos-rust-ut/scripts/run-ut.sh menu::types menu::manager menu::event

# 输出预期
running 5 tests
test menu::types::tests::test_menu_item_data_serialization ... ok
test menu::types::tests::test_submenu_nested_items ... ok
test menu::event::tests::test_menu_event_creation ... ok
test menu::event::tests::test_menu_event_dispatcher ... ok
test menu::event::tests::test_multiple_listeners ... ok
test result: ok. 5 passed; 0 failed
```

### 2.6 api-demo 自动测试（外部接口）

**验证范围**：
| 接口 | 类型 | 测试方法 | category |
|------|------|---------|----------|
| `Menu.new()` | 外部 | api-demo autotest | auto |
| `MenuItem.new()` | 外部 | api-demo autotest | auto |
| `Submenu.new()` | 外部 | api-demo autotest | auto |
| `Menu.append()` | 外部 | api-demo autotest | auto |
| `Menu.items()` | 外部 | api-demo autotest | auto |

**测试文件**: `examples/api/src/lib/tests/core.ts`

```typescript
// === Menu 模块测试 ===

import { Menu, MenuItem, Submenu } from '@tauri-apps/api/menu';

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
    const item = await MenuItem.new({ text: 'Test Item' });
    const text = await item.text();
    assert(text === 'Test Item', `text mismatch: ${text}`);
  },
},
{
  name: '@tauri-apps/api/menu.Submenu.new',
  category: 'auto',
  async fn() {
    const submenu = await Submenu.new({ text: 'Submenu' });
    assert(submenu !== undefined, 'Submenu.new returned undefined');
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
    assert(items.length === 1, `items.length should be 1, got ${items.length}`);
  },
},
{
  name: '@tauri-apps/api/menu.Menu with submenu',
  category: 'auto',
  async fn() {
    const menu = await Menu.new();
    const submenu = await Submenu.new({ text: 'File' });
    const item = await MenuItem.new({ text: 'Open' });
    await submenu.append(item);
    await menu.append(submenu);
    
    const items = await menu.items();
    assert(items.length === 1, `items.length should be 1`);
  },
},
```

**执行命令**:
```bash
# 一键运行 api-demo 测试
bash .claude/skills/ohos-build/scripts/run-tests.sh

# 或分步执行
export VITE_AUTOTEST=true
bash .claude/skills/ohos-build/scripts/build-ohos.sh
bash .claude/skills/ohos-build/scripts/sign-and-install.sh
cmd.exe /c "hdc file recv /data/app/el2/100/base/com.tauri.api/cache/test-report.json D:\workspace\tauri\tauri\examples\api\test-report.json"
```

### 2.7 api-demo 手动测试（外部接口 - UI 相关）

**验证范围**：
| 接口 | 类型 | 测试方法 | 原因 |
|------|------|---------|------|
| `Menu.popup()` | 外部 | api-demo manual | UI 显示需人工确认 |
| `handleItemClick()` | 外部 | api-demo manual | 交互需人工触发 |

**测试文件**: `examples/api/src/views/TestRunner.svelte`

```typescript
// 手动测试：Menu.popup() 功能
import { Menu, MenuItem } from '@tauri-apps/api/menu';

async function manualMenuPopup() {
  await wrapManual('menu.popup', async () => {
    const menu = await Menu.new({
      items: [
        await MenuItem.new({ text: 'Custom Action' }),
      ]
    });
    await menu.popup();
    onMessage('Menu.popup() - should show menu at cursor position');
  });
}

async function manualMenuItemClick() {
  await wrapManual('menu.itemClick', async () => {
    const menu = await Menu.new({
      items: [
        await MenuItem.new({ text: 'Click Me' }),
      ]
    });
    await menu.popup();
    onMessage('Click "Click Me" - verify event fired in console');
  });
}
```

```svelte
<!-- Manual Tests 按钮 -->
<div class="manual-tests">
  <h3>Menu Manual Tests</h3>
  <button class="btn" onclick={manualMenuPopup}>
    Menu.popup (should show menu)
  </button>
  <button class="btn" onclick={manualMenuItemClick}>
    Menu.itemClick (verify event)
  </button>
</div>
```

### 2.8 相关 SKILL 文档

- [ohos-rust-ut](../../.claude/skills/ohos-rust-ut/SKILL.md) - Rust UT 执行
- [frontend-api-testing](../../.claude/skills/frontend-api-testing/SKILL.md) - 前端测试编写
- [ohos-build](../../.claude/skills/ohos-build/SKILL.md) - OHOS 构建

## 工期
- 3 天（含 MenuEvent 0.5 天）

## 输出物
- `menu.ets` - 完整菜单管理实现（含事件分发）
- `menu/types.rs` - Rust 菜单类型
- `menu/manager.rs` - Rust 菜单管理器
- `menu/event.rs` - MenuEvent 和事件分发器
- 单元测试和集成测试通过（含 MenuEvent 测试）

## 参考文档
- [Menu 组件](../reference/menu.md) - Menu 容器和子菜单展开样式
- [MenuItem 组件](../reference/menu_item.md) - MenuItem 属性和 onClick/onChange 事件
- [MenuItemGroup](../reference/menu_item_group.md) - 菜单项分组（Tauri 无对应 API，可选实现）

---

## 实现状态

### 已完成 (2026-05-15)

**Phase 2 完成度: 100%**

| 任务 | 状态 | 说明 |
|------|------|------|
| Rust 核心类型 | ✅ 完成 | Menu/MenuItem/Submenu/MenuItemData |
| Rust 事件系统 | ✅ 完成 | MenuEvent + MenuEventDispatcher + channel |
| Rust 状态管理 | ✅ 完成 | MenuStateController |
| Rust Popup | ✅ 完成 | MenuPopup |
| Rust 预定义项 | ✅ 完成 | PredefinedMenuItem + 17 个 factory 方法 |
| ArkTS 类型定义 | ✅ 完成 | menu_types.ets (60 行) |
| ArkTS 菜单管理器 | ✅ 完成 | menu.ets (219 行) |
| ArkTS 状态管理 | ✅ 完成 | menu_state.ets (64 行) |
| ArkTS UI 渲染 | ✅ 完成 | TauriMenu.ets (78 行) |
| muda OHOS 后端 | ✅ 完成 | mod.rs (455 行) |
| muda 编译修复 | ✅ 完成 | gtk cfg guards + error variants + KeyAccelerator |
| 编译验证 | ✅ 完成 | openharmony-ability + muda 均通过 |

### 文件清单

| 文件 | 行数 | 说明 |
|------|------|------|
| `openharmony-ability/crates/ability/src/menu/mod.rs` | 99 | 模块入口、channel、emit_menu_event |
| `openharmony-ability/crates/ability/src/menu/types.rs` | 258 | MenuItemData + Menu/MenuItem/Submenu |
| `openharmony-ability/crates/ability/src/menu/event.rs` | 112 | MenuEvent + MenuEventDispatcher |
| `openharmony-ability/crates/ability/src/menu/state.rs` | 75 | MenuStateController |
| `openharmony-ability/crates/ability/src/menu/popup.rs` | 68 | MenuPopup |
| `openharmony-ability/crates/ability/src/menu/predefined.rs` | 348 | PredefinedMenuItem + 17 factory 方法 |
| `openharmony-ability/native_ability/src/main/ets/helper/menu_types.ets` | 60 | ArkTS 类型定义 |
| `openharmony-ability/native_ability/src/main/ets/helper/menu.ets` | 219 | PredefinedActionExecutor + TauriMenuManager |
| `openharmony-ability/native_ability/src/main/ets/helper/menu_state.ets` | 64 | MenuStateController |
| `openharmony-ability/native_ability/src/main/ets/components/TauriMenu.ets` | 78 | TauriMenuPopup UI 组件 (含 check/icon) |
| `muda/src/platform_impl/ohos/mod.rs` | 455 | muda OHOS 后端实现 |
| `muda/src/error.rs` | 57 | 新增 CustomError + NotSupportedOnPlatform |
| `muda/src/menu.rs` | 修改 | 8 个 gtk 方法添加 `not(target_env = "ohos")` |
| `muda/src/items/submenu.rs` | 修改 | 2 个 gtk 方法添加 `not(target_env = "ohos")` |

### 本次完成的修复 (2026-05-15)

#### ArkTS 修复
| 文件 | 修复内容 |
|------|----------|
| `TauriMenu.ets` | 添加 `check` 类型渲染: `MenuItem({ type: Check, checked, onChange })` |
| `TauriMenu.ets` | 添加 `icon` 类型渲染: `MenuItem({ content, labelInfo })` |
| `menu.ets` | `handleItemClick()` 添加 `check`/`icon`/`separator`/`submenu` 分支处理 |
| `menu.ets` | `createMenuItem()` 添加 `checked` 和 `icon` 属性支持 |

#### Rust 修复
| 文件 | 修复内容 |
|------|----------|
| `predefined.rs` | 添加 6 个缺失的 factory: `recover`, `restore`, `hide`, `hide_others`, `show_all`, `about` |
| `predefined.rs` | 为 `PredefinedType` 添加 `PartialEq` derive (修复测试编译) |
| `menu.rs` | 为 8 个 gtk 方法添加 `not(target_env = "ohos")` cfg guard |
| `submenu.rs` | 为 2 个 gtk 方法添加 `not(target_env = "ohos")` cfg guard |
| `error.rs` | 添加 `CustomError(String)` 和 `NotSupportedOnPlatform` 错误变体 |
| `ohos/mod.rs` | 修复 `PredefinedMenuItemType::Close` → `CloseWindow` |
| `ohos/mod.rs` | 添加缺失的 `item_type()` 方法 |
| `ohos/mod.rs` | 实现 `KeyAccelerator` → 字符串转换 (Ctrl+/Alt+/Shift+/Super+) |
| `ohos/mod.rs` | 删除重复的 `to_menu_item_data()` 方法 |
| `ohos/mod.rs` | 添加 `Services`/`BringAllToFront`/`None` 预定义类型映射 |

### 架构修正

1. **muda 不定义自己的类型**：使用 `openharmony_ability::menu::MenuItemData` 进行序列化
2. **分散的 NAPI 类型**：不使用统一的 `MenuManager`，而是 `Menu`/`MenuItem`/`Submenu` + `MenuStateController`
3. **事件通道机制**：`MENU_EVENT_CHANNEL` (ArkTS → muda) + `POPUP_CHANNEL` (muda → ArkTS)
4. **Popup 不需要窗口句柄**：OHOS 的 `bindContextMenu` 不需要像 Windows `TrackPopupMenu` 那样传入 `hwnd`
5. **gtk cfg 修复**：所有 gtk 相关方法必须添加 `not(target_env = "ohos")` 避免 OHOS 编译时引入 gtk crate

### 预定义菜单项工厂方法 (17 个)

| 方法 | 状态 | 说明 |
|------|------|------|
| `copy()` | ✅ | Ctrl+C |
| `cut()` | ✅ | Ctrl+X |
| `paste()` | ✅ | Ctrl+V |
| `select_all()` | ✅ | Ctrl+A |
| `undo()` | ✅ | Ctrl+Z |
| `redo()` | ✅ | Ctrl+Y |
| `minimize()` | ✅ | Ctrl+M |
| `maximize()` | ✅ | - |
| `close_window()` | ✅ | Ctrl+W |
| `quit()` | ✅ | Ctrl+Q |
| `separator()` | ✅ | - |
| `recover()` | ✅ | 从最大化恢复 |
| `restore()` | ✅ | 从最小化恢复 (API 14+) |
| `hide()` | ✅ | 隐藏 (minimize) |
| `hide_others()` | ✅ | OHOS 不支持 |
| `show_all()` | ✅ | OHOS 不支持 |
| `about()` | ✅ | 需要 AlertDialog |

### ArkTS UI 渲染支持

| 菜单项类型 | TauriMenu.ets 渲染 | 状态 |
|-----------|-------------------|------|
| `separator` | `MenuDivider()` | ✅ |
| `item` | `MenuItem({ content, labelInfo })` | ✅ |
| `predefined` | `MenuItem({ content, labelInfo })` | ✅ |
| `check` | `MenuItem({ type: Check, checked, onChange })` | ✅ |
| `icon` | `MenuItem({ content, labelInfo })` | ✅ |
| `submenu` | `MenuItem({ content, builder })` | ✅ |

### 编译状态

| 组件 | 命令 | 状态 |
|------|------|------|
| openharmony-ability | `cargo check --target aarch64-unknown-linux-ohos --features menu` | ✅ 通过 (1 warning) |
| muda | `cargo check --target aarch64-unknown-linux-ohos` | ✅ 通过 (11 warnings) |
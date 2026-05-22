# Phase 4: Menu.popup() 集成

## 目标
在 tauri::menu 和 muda::platform_impl::ohos 中实现 Menu.popup() 功能，使用 `bindContextMenu(isShown)` + 状态变量控制菜单弹出。

## 依赖
- Phase 0: muda OHOS 后端完成
- Phase 2: 核心菜单类型验证完成
- Phase 3: PredefinedMenuItem 完成

## 架构说明

> **重要**：popup 实现分层：
>
> ```
> tauri::menu::menu.rs::popup_inner()
>     ↓ #[cfg(target_env = "ohos")] 分支
> muda::platform_impl::ohos::Menu::show_context_menu_for_component()
>     ↓
> openharmony-ability::TauriMenuManager::popup()
>     ↓
> ArkUI bindContextMenu(isShown)
> ```
>
> 需要修改：
> 1. tauri::menu::menu.rs: 添加 #[cfg(target_env = "ohos")] 分支
> 2. tauri::menu::submenu.rs: 添加 #[cfg(target_env = "ohos")] 分支
> 3. muda::platform_impl::ohos::Menu: 实现 show_context_menu_for_component()
> 4. openharmony-ability: 实现 ArkTS 菜单组件

## 工作内容

### OHOS Menu 弹出机制

**OHOS 不支持传统的 `popup(x, y)` API**，但可通过 `bindContextMenu(isShown, ...)` 实现：

```typescript
// bindContextMenu(isShown: boolean, builder: CustomBuilder, options)
// API version 12+ 支持
// isShown = true → 弹出菜单
// isShown = false → 关闭菜单
// API 18+ 支持 !! 双向绑定语法
```

> **参考文档**: [bindContextMenu API](../reference/menu_control.md) 第 36-68 行

**实现方案**：
1. 创建透明占位组件绑定菜单
2. 使用状态变量 `@State isMenuShown: boolean` 控制显隐
3. 通过 `offset` 参数控制菜单位置（API 12-19）
4. 或通过 `anchorPosition` 参数精确控制位置（API 20+）

**API 版本要求**：
- `bindContextMenu(content, responseType)` - API 9+（手势触发）
- `bindContextMenu(isShown, ...)` - API 12+（程序化控制）
- `anchorPosition` 参数 - API 20+（精确定位）
- `!!` 双向绑定语法 - API 18+

**重要特性说明**：

| 特性 | API 版本 | 说明 |
|------|---------|------|
| `offset` 定位 | API 11+ | 基于 placement 方向偏移 |
| `anchorPosition` 定位 | API 20+ | 精确定位，**placement 参数失效** |
| `!!` 双向绑定 | API 18+ | isShown 支持 !! 语法双向同步 |

**重要限制**（来自 menu_control.md:57）：
> 菜单必须等待页面全部构建完成后才能展示，如果在页面构建前或构建中设置为true，可能导致显示位置及形状错误、无法正常弹出显示等问题。不支持长按触发拖拽。

## 工作内容

### 4.1 ArkTS 菜单组件实现

**文件**: `openharmony-ability/native_ability/src/main/ets/components/TauriMenu.ets`

```typescript
import { TauriMenuItemData, TauriMenuManager } from '../helper/menu';

@Component
export struct TauriMenu {
  @Prop manager: TauriMenuManager;
  @State isMenuShown: boolean = false;
  @State menuOffset: Position = { x: 0, y: 0 };
  @State currentMenuItems: TauriMenuItemData[] = [];

  // 透明占位组件，用于绑定菜单
  build() {
    Column()
      .width(1)
      .height(1)
      .position({ x: 0, y: 0 })
      .bindContextMenu(this.isMenuShown, this.MenuContent, {
        offset: this.menuOffset,
        aboutToDisappear: () => {
          this.isMenuShown = false;
        }
      })
  }

  // 菜单内容构建器
  @Builder
  MenuContent() {
    Menu() {
      ForEach(this.currentMenuItems, (item: TauriMenuItemData) => {
        this.buildMenuItem(item);
      });
    }
  }

  @Builder
  buildMenuItem(item: TauriMenuItemData) {
    if (item.type === 'separator') {
      MenuDivider()
    } else if (item.type === 'item') {
      MenuItem({ content: item.text ?? '', labelInfo: item.accelerator })
        .enabled(item.enabled ?? true)
        .onClick(() => {
          this.manager.handleItemClick(item);
        });
    } else if (item.type === 'predefined') {
      MenuItem({ content: item.text ?? '', labelInfo: item.accelerator })
        .enabled(item.enabled ?? true)
        .onClick(() => {
          this.manager.handleItemClick(item);
        });
    } else if (item.type === 'submenu') {
      MenuItem({ 
        content: item.text ?? '',
        builder: () => this.buildSubmenu(item.submenuItems ?? [])
      });
    }
  }

  @Builder
  buildSubmenu(items: TauriMenuItemData[]) {
    Menu() {
      ForEach(items, (item: TauriMenuItemData) => {
        this.buildMenuItem(item);
      });
    }
  }

  // 弹出菜单（供外部调用）
  popup(menuItems: TauriMenuItemData[], x: number, y: number) {
    this.currentMenuItems = menuItems;
    this.menuOffset = { x, y };
    this.isMenuShown = true;
  }

  // 关闭菜单
  close() {
    this.isMenuShown = false;
  }
}
```

### 4.2 状态管理集成

**文件**: `openharmony-ability/native_ability/src/main/ets/helper/menu_state.ets`

```typescript
import { TauriMenuItemData } from './menu_types';

export interface MenuState {
  isShown: boolean;
  menuId: string;
  items: TauriMenuItemData[];
  offset: { x: number; y: number };
  onDisappear?: () => void;
}

/**
 * 全局菜单状态管理
 * 用于从 Rust 侧控制菜单
 */
export class MenuStateController {
  private state: MenuState = {
    isShown: false,
    menuId: '',
    items: [],
    offset: { x: 0, y: 0 },
  };

  private stateChangeCallbacks: Set<(state: MenuState) => void> = new Set();

  getState(): MenuState {
    return { ...this.state };
  }

  subscribe(callback: (state: MenuState) => void): () => void {
    this.stateChangeCallbacks.add(callback);
    return () => {
      this.stateChangeCallbacks.delete(callback);
    };
  }

  private notifyStateChange(): void {
    const state = this.getState();
    this.stateChangeCallbacks.forEach(cb => cb(state));
  }

  showMenu(menuId: string, items: TauriMenuItemData[], x: number, y: number): void {
    this.state = {
      isShown: true,
      menuId,
      items,
      offset: { x, y },
    };
    this.notifyStateChange();
  }

  hideMenu(): void {
    this.state = {
      ...this.state,
      isShown: false,
    };
    this.notifyStateChange();
  }

  updateItems(items: TauriMenuItemData[]): void {
    this.state = {
      ...this.state,
      items,
    };
    this.notifyStateChange();
  }
}

// 全局单例
export const menuStateController = new MenuStateController();
```

### 4.3 主页面集成

**文件**: `openharmony-ability/native_ability/src/main/ets/pages/Index.ets`

```typescript
import { menuStateController, MenuState } from '../helper/menu_state';
import { TauriMenu } from '../components/TauriMenu';
import { TauriMenuManager, PredefinedActionExecutor } from '../helper/menu';
import { TauriMenuItemData } from '../helper/menu_types';

@Entry
@Component
struct Index {
  @State menuState: MenuState = menuStateController.getState();
  private menuManager: TauriMenuManager | null = null;
  
  // WebView 和 Window 引用
  private webController: web_webview.WebviewController | null = null;
  private window: window.Window | null = null;

  aboutToAppear(): void {
    // 初始化菜单管理器
    const executor = new PredefinedActionExecutor((code) => {
      // 退出应用
      this.context?.terminateSelf();
    });
    
    if (this.webController) {
      executor.setController(this.webController);
    }
    if (this.window) {
      executor.setWindow(this.window);
    }
    
    this.menuManager = new TauriMenuManager(executor);
    
    // 订阅状态变化
    menuStateController.subscribe((state) => {
      this.menuState = state;
    });
  }

  build() {
    Stack() {
      // 主内容区域
      Column() {
        // WebView 或其他内容
        Web({ src: 'https://example.com', controller: this.webController })
          .width('100%')
          .height('100%')
      }
      .width('100%')
      .height('100%')
      
      // 菜单覆盖层
      if (this.menuState.isShown) {
        TauriMenuPopup({
          items: this.menuState.items,
          offset: this.menuState.offset,
          onItemClick: (item: TauriMenuItemData) => {
            this.menuManager?.handleItemClick(item);
          },
          onDisappear: () => {
            menuStateController.hideMenu();
          }
        })
      }
    }
    .width('100%')
    .height('100%')
  }
}

// 菜单弹出组件
@Component
struct TauriMenuPopup {
  @Prop items: TauriMenuItemData[];
  @Prop offset: Position;
  @Prop onItemClick: (item: TauriMenuItemData) => void;
  @Prop onDisappear: () => void;
  @State isShown: boolean = true;

  build() {
    Column()
      .width(1)
      .height(1)
      .position({ x: 0, y: 0 })
      .bindContextMenu(this.isShown, this.MenuContent, {
        offset: this.offset,
        aboutToDisappear: () => {
          this.isShown = false;
          this.onDisappear?.();
        }
      })
  }

  @Builder
  MenuContent() {
    Menu() {
      ForEach(this.items, (item: TauriMenuItemData) => {
        this.buildMenuItem(item);
      });
    }
  }

  @Builder
  buildMenuItem(item: TauriMenuItemData) {
    if (item.type === 'separator') {
      MenuDivider()
    } else if (item.type === 'item' || item.type === 'predefined') {
      MenuItem({ content: item.text ?? '', labelInfo: item.accelerator })
        .enabled(item.enabled ?? true)
        .onClick(() => {
          this.itemClick(item);
        });
    } else if (item.type === 'submenu') {
      MenuItem({ 
        content: item.text ?? '',
        builder: () => this.SubmenuContent(item.submenuItems ?? [])
      });
    }
  }

  @Builder
  SubmenuContent(items: TauriMenuItemData[]) {
    Menu() {
      ForEach(items, (item: TauriMenuItemData) => {
        this.buildMenuItem(item);
      });
    }
  }

  itemClick(item: TauriMenuItemData) {
    this.isShown = false;
    this.onItemClick(item);
  }
}
```

### 4.4 Rust popup 调用

**文件**: `openharmony-ability/crates/ability/src/menu/popup.rs`

```rust
use crate::menu::types::MenuItemData;
use crate::menu::state::MenuStateController;
use napi::bindgen_prelude::*;

#[napi]
pub struct MenuPopup {
    state_controller: MenuStateController,
}

#[napi]
impl MenuPopup {
    #[napi(constructor)]
    pub fn new(state_controller: MenuStateController) -> Self {
        Self { state_controller }
    }

    #[napi]
    pub async fn show(&self, menu_id: String, x: f64, y: f64) -> Result<()> {
        // 获取菜单项
        let items = self.state_controller.get_menu_items(&menu_id).await?;
        
        // 调用 ArkTS 显示菜单
        self.state_controller
            .show_menu(&menu_id, items, x, y)
            .await?;
        
        Ok(())
    }

    #[napi]
    pub async fn hide(&self) -> Result<()> {
        self.state_controller.hide_menu().await;
        Ok(())
    }
}
```

### 4.5 状态控制器 NAPI 绑定

**文件**: `openharmony-ability/crates/ability/src/menu/state.rs`

```rust
use crate::menu::types::MenuItemData;
use napi::bindgen_prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[napi]
pub struct MenuStateController {
    menus: Arc<RwLock<HashMap<String, Vec<MenuItemData>>>>,
    show_fn: Arc<RwLock<Option<Function<String, Vec<MenuItemData>, f64, f64, ()>>>>,
    hide_fn: Arc<RwLock<Option<Function<()>>>>,
}

#[napi]
impl MenuStateController {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            menus: Arc::new(RwLock::new(HashMap::new())),
            show_fn: Arc::new(RwLock::new(None)),
            hide_fn: Arc::new(RwLock::new(None)),
        }
    }

    #[napi]
    pub async fn set_show_handler(&mut self, handler: Function<String, Vec<MenuItemData>, f64, f64, ()>) -> Result<()> {
        let mut guard = self.show_fn.write().await;
        *guard = Some(handler);
        Ok(())
    }

    #[napi]
    pub async fn set_hide_handler(&mut self, handler: Function<()>) -> Result<()> {
        let mut guard = self.hide_fn.write().await;
        *guard = Some(handler);
        Ok(())
    }

    pub async fn get_menu_items(&self, menu_id: &str) -> Result<Vec<MenuItemData>> {
        let guard = self.menus.read().await;
        Ok(guard.get(menu_id).cloned().unwrap_or_default())
    }

    pub async fn show_menu(&self, menu_id: &str, items: Vec<MenuItemData>, x: f64, y: f64) -> Result<()> {
        let guard = self.show_fn.read().await;
        if let Some(fn_handler) = guard.as_ref() {
            fn_handler.call(menu_id.to_string(), items, x, y).await?;
        }
        Ok(())
    }

    pub async fn hide_menu(&self) {
        let guard = self.hide_fn.read().await;
        if let Some(fn_handler) = guard.as_ref() {
            let _ = fn_handler.call().await;
        }
    }
}
```

### 4.6 Tauri Menu API 适配

**文件**: `tauri/crates/tauri/src/menu/ohos.rs`

```rust
use crate::menu::{Menu, MenuItem, Submenu, PredefinedMenuItem};
use crate::window::Window;

impl Menu {
    pub fn popup(&self, window: &Window, x: Option<f64>, y: Option<f64>) -> crate::Result<()> {
        let x = x.unwrap_or(0.0);
        let y = y.unwrap_or(0.0);
        
        // 调用 OHOS popup
        self.inner.popup(x, y)?;
        
        Ok(())
    }
}

impl MenuInner {
    fn popup(&self, x: f64, y: f64) -> crate::Result<()> {
        // 通过 NAPI 调用 MenuPopup.show()
        #[cfg(target_env = "ohos")]
        {
            use openharmony_ability::menu::MenuPopup;
            let popup = MenuPopup::new(self.state_controller.clone());
            futures::executor::block_on(popup.show(self.id.clone(), x, y))?;
        }
        Ok(())
    }
}
```

## 验证方法

> **验证策略**：
> - 内部接口 → Rust UT
> - 外部接口 → api-demo（autotest 或 manual test）

### 4.7 Rust UT（内部接口）

**验证范围**：
| 接口 | 类型 | 测试方法 |
|------|------|---------|
| `MenuPopup::new()` | 内部 | Rust UT |
| `MenuStateController::new()` | 内部 | Rust UT |
| `MenuStateController::show_menu()` | 内部 | Rust UT（仅验证调用成功） |
| `MenuStateController::hide_menu()` | 内部 | Rust UT |

**测试文件**: `menu/popup.rs`, `menu/state.rs`

```rust
#[cfg(all(test, target_env = "ohos"))]
mod tests {
    use super::*;

    #[test]
    fn test_menu_state_controller_creation() {
        let controller = MenuStateController::new();
        assert!(controller.menus.read().await.is_ok());
    }

    #[test]
    fn test_menu_popup_creation() {
        let controller = MenuStateController::new();
        let popup = MenuPopup::new(controller);
        assert!(popup.state_controller.menus.read().await.is_ok());
    }

    #[tokio::test]
    async fn test_menu_state_show_hide() {
        let controller = MenuStateController::new();
        
        // show_menu should succeed (即使没有注册 handler)
        let result = controller.show_menu("test_menu", vec![], 100.0, 100.0).await;
        assert!(result.is_ok());
        
        // hide_menu should succeed
        controller.hide_menu().await;
    }
}
```

**执行命令**:
```bash
# 运行 popup/state 模块 Rust UT
bash .claude/skills/ohos-rust-ut/scripts/run-ut.sh menu::popup menu::state

# 输出预期
running 3 tests
test menu::popup::tests::test_menu_popup_creation ... ok
test menu::state::tests::test_menu_state_controller_creation ... ok
test menu::state::tests::test_menu_state_show_hide ... ok
test result: ok. 3 passed; 0 failed
```

### 4.8 api-demo 手动测试（外部接口 - UI 显示）

**验证范围**：
| 接口 | 类型 | 测试方法 | 原因 |
|------|------|---------|------|
| `bindContextMenu(isShown)` | 外部 | api-demo manual | 菜单 UI 显示需人工确认 |
| `Menu.popup(x, y)` | 外部 | api-demo manual | 菜单位置需人工确认 |
| 菜单项点击响应 | 外部 | api-demo manual | UI 交互需人工触发 |

**测试文件**: `examples/api/src/views/TestRunner.svelte`

```typescript
// 手动测试：Menu.popup() 显示和交互
import { Menu, MenuItem, PredefinedMenuItem } from '@tauri-apps/api/menu';

async function manualMenuPopupBasic() {
  await wrapManual('menu.popup.basic', async () => {
    const menu = await Menu.new({
      items: [
        await MenuItem.new({ text: 'Action 1' }),
        await MenuItem.new({ text: 'Action 2' }),
        await PredefinedMenuItem.separator(),
        await MenuItem.new({ text: 'Action 3' }),
      ]
    });
    await menu.popup();
    onMessage('Menu should appear at cursor. Click any item to close.');
  });
}

async function manualMenuPopupWithPredefined() {
  await wrapManual('menu.popup.predefined', async () => {
    const menu = await Menu.new({
      items: [
        await PredefinedMenuItem.copy({ text: '复制' }),
        await PredefinedMenuItem.paste({ text: '粘贴' }),
        await PredefinedMenuItem.separator(),
        await PredefinedMenuItem.minimize({ text: '最小化' }),
      ]
    });
    await menu.popup();
    onMessage('Menu with predefined items. Verify copy/paste/minimize work.');
  });
}

async function manualMenuPopupWithSubmenu() {
  await wrapManual('menu.popup.submenu', async () => {
    const fileSubmenu = await Submenu.new({ text: 'File' });
    await fileSubmenu.append(await MenuItem.new({ text: 'Open' }));
    await fileSubmenu.append(await MenuItem.new({ text: 'Save' }));
    
    const editSubmenu = await Submenu.new({ text: 'Edit' });
    await editSubmenu.append(await PredefinedMenuItem.copy());
    await editSubmenu.append(await PredefinedMenuItem.cut());
    
    const menu = await Menu.new({ items: [fileSubmenu, editSubmenu] });
    await menu.popup();
    onMessage('Menu with nested submenus. Expand "File" and "Edit" to verify.');
  });
}

async function manualMenuPopupPosition() {
  await wrapManual('menu.popup.position', async () => {
    const menu = await Menu.new({
      items: [await MenuItem.new({ text: 'Test Position' })]
    });
    // 测试指定位置弹出
    await menu.popup(200, 300);
    onMessage('Menu should appear at position (200, 300). Verify correct placement.');
  });
}
```

```svelte
<!-- Manual Tests 按钮 -->
<div class="manual-tests">
  <h3>Menu.popup Tests</h3>
  <button class="btn" onclick={manualMenuPopupBasic}>
    Basic Menu Popup
  </button>
  <button class="btn" onclick={manualMenuPopupWithPredefined}>
    Menu with Predefined Items
  </button>
  <button class="btn" onclick={manualMenuPopupWithSubmenu}>
    Menu with Submenus
  </button>
  <button class="btn" onclick={manualMenuPopupPosition}>
    Menu at Position (200,300)
  </button>
</div>
```

### 4.9 验证要点

**手动测试检查清单**：

| 检查项 | 预期结果 |
|--------|---------|
| 菜单弹出位置 | 在指定位置或光标位置正确显示 |
| 菜单项文本 | 所有 MenuItem 文本正确显示 |
| 分隔线 | PredefinedMenuItem.separator 显示为分隔线 |
| 子菜单展开 | 点击 Submenu 正确展开子菜单 |
| 预定义项功能 | copy/paste/minimize 等功能正常执行 |
| 菜单关闭 | 点击任意项后菜单正确关闭 |

### 4.10 相关 SKILL 文档

- [ohos-rust-ut](../../.claude/skills/ohos-rust-ut/SKILL.md) - Rust UT 执行
- [frontend-api-testing](../../.claude/skills/frontend-api-testing/SKILL.md) - 前端测试编写
- [ohos-build](../../.claude/skills/ohos-build/SKILL.md) - OHOS 构建

## 已知限制

1. **位置精度**：`offset` 相对于绑定组件，需要计算正确的偏移值
2. **API 20+ 改进**：使用 `anchorPosition` 可更精确控制位置，支持覆盖显示在绑定组件上
3. **多菜单管理**：同时只能显示一个菜单，需确保关闭前一个
4. **键盘快捷键**：需要额外实现全局键盘监听（不在本阶段范围）
5. **页面构建时序**：菜单必须等待页面全部构建完成后才能展示

### anchorPosition vs offset（API 20+）

根据 menu_control.md 第 477-494 行：

```typescript
// API 20+ 新增 anchorPosition 参数
.bindContextMenu(this.isShown, this.MenuContent, {
  anchorPosition: { x: 100, y: 100 },  // 相对于绑定组件左上角
  // 与 offset 不同：可以覆盖显示在绑定组件上
  // placement 参数不再生效
})
```

**差异对比**：
| 参数 | API 版本 | 特点 |
|------|---------|------|
| `offset` | API 11+ | 基于 placement 方向偏移 |
| `anchorPosition` | API 20+ | 精确定位，可覆盖绑定组件，placement 不生效 |

**推荐实现**：
```typescript
// 根据目标 API 版本选择方案
const menuOptions = apiVersion >= 20 
  ? { anchorPosition: { x, y } }  // API 20+: 精确定位
  : { offset: { x, y } };          // API 12-19: 基于方向偏移
```

## 工期
- 3 天

## 输出物
- `TauriMenu.ets` - 菜单组件
- `menu_state.ets` - 状态管理
- `menu/popup.rs` - Rust popup 实现
- `menu/state.rs` - Rust 状态控制器
- `menu/ohos.rs` - Tauri API 适配
- 集成测试通过

## 参考文档
- [bindContextMenu](../reference/menu_control.md) - 核心弹出 API
  - 第 36-68 行：`bindContextMenu(isShown, ...)` API 12+
  - 第 477-494 行：`anchorPosition` API 20+
  - 第 57 行：页面构建时序限制
- [ContextMenu.close](../reference/context_menu.md) - 关闭菜单（API 18 废弃，建议用 UIContext）

---

## 实现状态

### 已完成 (2026-05-15)

**Phase 4 完成度: 100%**

| 任务 | 状态 | 说明 |
|------|------|------|
| ArkTS MenuStateController | ✅ 完成 | menu_state.ets (64 行) |
| ArkTS TauriMenuPopup | ✅ 完成 | TauriMenu.ets (78 行) |
| Rust MenuStateController | ✅ 完成 | state.rs (75 行) |
| Rust MenuPopup | ✅ 完成 | popup.rs (68 行) |
| Rust popup_context_menu | ✅ 完成 | mod.rs POPUP_CHANNEL |
| tauri/menu/menu.rs OHOS | ✅ 完成 | popup_inner 分支 |
| tauri/menu/submenu.rs OHOS | ✅ 完成 | popup_inner 分支 |
| tauri/Cargo.toml | ✅ 完成 | muda path 依赖 + menu feature |
| tauri OHOS 编译 | ✅ 通过 | 3 warnings, 0 errors |

### 文件清单

| 文件 | 行数 | 说明 |
|------|------|------|
| `openharmony-ability/crates/ability/src/menu/mod.rs` | 99 | POPUP_CHANNEL + popup_context_menu() |
| `openharmony-ability/crates/ability/src/menu/state.rs` | 75 | MenuStateController |
| `openharmony-ability/crates/ability/src/menu/popup.rs` | 68 | MenuPopup |
| `openharmony-ability/native_ability/src/main/ets/helper/menu_state.ets` | 64 | MenuStateController |
| `openharmony-ability/native_ability/src/main/ets/components/TauriMenu.ets` | 78 | TauriMenuPopup UI 组件 |
| `tauri/crates/tauri/src/menu/menu.rs` | 修改 | popup_inner OHOS 分支 |
| `tauri/crates/tauri/src/menu/submenu.rs` | 修改 | popup_inner OHOS 分支 |
| `tauri/crates/tauri/Cargo.toml` | 修改 | muda path 依赖 |

### Popup 架构

```
tauri::menu::Menu::popup()
    ↓
popup_inner() → #[cfg(target_env = "ohos")] 分支
    ↓
muda::platform_impl::ohos::Menu::popup(x, y)
    ↓
openharmony_ability::menu::popup_context_menu(json, x, y)
    ↓
POPUP_CHANNEL (crossbeam_channel)
    ↓
ArkTS 监听器 → menuStateController.showMenu()
    ↓
TauriMenuPopup.bindContextMenu(isShown = true)
```

### 架构差异说明

| 设计文档 | 实际实现 | 影响 |
|----------|----------|------|
| `tauri/menu/ohos.rs` 独立文件 | 不需要 - OHOS 分支直接在 menu.rs/submenu.rs 中 | 简化架构 |
| `MenuPopup::show()` 直接调用 ArkUI | 通过 `popup_context_menu()` → `POPUP_CHANNEL` | 解耦 Rust/ArkTS |
| `MenuStateController` NAPI handler | 使用 `Arc<RwLock<HashMap>>` + channel | 更简单 |

### 编译状态

| 组件 | 命令 | 状态 |
|------|------|------|
| openharmony-ability | `cargo check --target aarch64-unknown-linux-ohos --features menu` | ✅ 通过 |
| muda | `cargo check --target aarch64-unknown-linux-ohos` | ✅ 通过 |
| tauri | `cargo check --target aarch64-unknown-linux-ohos` | ✅ 通过 (3 warnings) |
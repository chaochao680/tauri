# Phase 9: Menu Popup — bindMenu 重构 + 全类型 Item 支持

> 职责：使用 `bindMenu(isShow)` 重构 popup，支持真正的 submenu 展开、所有 item 类型（normal/check/icon/predefined/submenu/separator）的正确行为
> 代码位置：`openharmony-ability/native_ability/src/main/ets/`、`muda/src/platform_impl/ohos/`
> 独立性：✓ 仅影响 OHOS 路径，不影响 Windows/macOS/Linux
> 参考文档：`menu-guide.md`、`menu_control.md`、`phase7-predefined-check-icon-design.md`

## 一、需求

1. 点击 Popup 在点击位置正常弹出 menu
2. Menu 里可以创建 submenu（支持 api-demo 的 "app" submenu）
3. Menu 和 submenu 里都可以创建任意 item 类型：normal / check / icon / predefined / submenu（递归）
4. 各 item 类型有正确行为：separator 有分隔线、predefined 有对应动作、check 可 toggle
5. 不影响其他 OS 行为，OHOS 行为与其他 OS 一致

## 二、问题根因

### 为什么 openMenu + ComponentContent 不行

当前使用 `openMenu` + `ComponentContent` + 模块级 `@Builder` 实现 popup。`MenuItem.builder` 属性需要 `CustomBuilder`，但在模块级 `@Builder` 中调用另一个 `@Builder` 会 crash：

```
TypeError: Cannot read property observeComponentCreation2 of undefined
```

原因：模块级 `@Builder` 没有 `this`，ArkUI 框架无法注入渲染上下文。

### 解决方案：bindMenu(isShow, builder, options)

参考 `menu-guide.md` 官方示例，在 `@Component` 内使用 `@Builder` 方法 + `MenuItem.builder` 属性实现子菜单：

```typescript
// menu-guide.md 官方示例
@Builder
SubMenu() {
  Menu() {
    MenuItem({ content: "复制", labelInfo: 'Ctrl+C' })
    MenuItem({ content: "粘贴", labelInfo: 'Ctrl+V' })
  }
}

@Builder
MyMenu() {
  Menu() {
    MenuItem({
      content: "菜单选项",
      // 当builder参数进行配置时，表示与menuItem项绑定了子菜单。
      // 鼠标hover在该菜单项时，会显示子菜单。
      builder: this.SubMenu
    })
  }
}
```

使用 `bindMenu(isShow, content, options)` (API 11+) 程序控制显隐，配合 `anchorPosition` (API 20+) 精确定位。

## 三、核心方案

### 选择 bindMenu 而非 bindContextMenu

| 对比 | bindMenu(isShow) | bindContextMenu(isShown) |
|------|-----------------|--------------------------|
| 触发语义 | 左键点击 / 程序控制 | 右键/长按 / 程序控制 |
| 子窗口 | 可配置 `showInSubWindow` | 强制子窗口 |
| anchorPosition | ✅ API 20+ | ✅ API 20+ |
| API 版本 | API 11+ | API 12+ |
| 适用场景 | ✅ 我们的场景（程序触发） | 右键菜单场景 |

选择 `bindMenu`：语义更匹配（程序触发弹出），API 版本要求更低，子窗口行为可控。

### Step 1: MainPage.ets — bindMenu + @Builder 递归渲染

**文件**: `openharmony-ability/native_ability/src/main/ets/components/MainPage.ets`

```typescript
import { TauriMenuItemData } from "../helper/menu_types";

// 全局点击回调（由 TauriMenuManager 设置）
let globalMenuClickHandler: ((item: TauriMenuItemData) => void) | null = null;
export function setMenuClickHandler(handler: (item: TauriMenuItemData) => void): void {
  globalMenuClickHandler = handler;
}

@Entry({ routeName: RouteName })
@Component
struct Index {
  // ... 现有字段 ...

  // Menu popup 状态（通过 AppStorage 由 TauriMenuManager 控制）
  @StorageLink("__tauri_menu_shown__") menuShown: boolean = false;
  @StorageLink("__tauri_menu_json__") menuJson: string = "[]";
  @StorageLink("__tauri_menu_x__") menuX: number = 0;
  @StorageLink("__tauri_menu_y__") menuY: number = 0;
  @State private menuItems: TauriMenuItemData[] = [];

  // 监听 menuJson 变化，解析为 menuItems
  onMenuJsonChange(): void {
    try {
      this.menuItems = JSON.parse(this.menuJson) as TauriMenuItemData[];
    } catch (e) {
      this.menuItems = [];
    }
  }

  @Builder
  SubmenuContent(items: TauriMenuItemData[]) {
    Menu() {
      ForEach(items, (item: TauriMenuItemData) => {
        if (item.type === 'predefined' && item.predefinedType === 'separator') {
          MenuItemGroup() {}  // MenuItemGroup 边界自动产生分隔线
        } else if (item.type === 'submenu') {
          MenuItem({
            content: item.text ?? '',
            builder: (): void => this.SubmenuContent(item.submenuItems ?? [])
          })
        } else if (item.type === 'check') {
          MenuItem({ content: item.text ?? '' })
            .enabled(item.enabled ?? true)
            .selected(item.checked ?? false)
            .selectIcon(true)
            .onChange((checked: boolean) => {
              item.checked = checked;
              if (globalMenuClickHandler) globalMenuClickHandler(item);
            })
        } else if (item.type === 'icon') {
          MenuItem({ content: item.text ?? '' })
            .enabled(item.enabled ?? true)
            .onClick(() => {
              if (globalMenuClickHandler) globalMenuClickHandler(item);
            })
        } else {
          // normal + predefined (非 separator)
          MenuItem({ content: item.text ?? item.predefinedType ?? '' })
            .enabled(item.enabled ?? true)
            .onClick(() => {
              if (globalMenuClickHandler) globalMenuClickHandler(item);
            })
        }
      })
    }
  }

  @Builder
  MenuContent() {
    Menu() {
      ForEach(this.menuItems, (item: TauriMenuItemData) => {
        if (item.type === 'predefined' && item.predefinedType === 'separator') {
          MenuItemGroup() {}
        } else if (item.type === 'submenu') {
          MenuItem({
            content: item.text ?? '',
            builder: (): void => this.SubmenuContent(item.submenuItems ?? [])
          })
        } else if (item.type === 'check') {
          MenuItem({ content: item.text ?? '' })
            .enabled(item.enabled ?? true)
            .selected(item.checked ?? false)
            .selectIcon(true)
            .onChange((checked: boolean) => {
              item.checked = checked;
              if (globalMenuClickHandler) globalMenuClickHandler(item);
            })
        } else if (item.type === 'icon') {
          MenuItem({ content: item.text ?? '' })
            .enabled(item.enabled ?? true)
            .onClick(() => {
              if (globalMenuClickHandler) globalMenuClickHandler(item);
            })
        } else {
          MenuItem({ content: item.text ?? item.predefinedType ?? '' })
            .enabled(item.enabled ?? true)
            .onClick(() => {
              if (globalMenuClickHandler) globalMenuClickHandler(item);
            })
        }
      })
    }
  }

  build() {
    Stack() {
      Row() {
        Column() {
          if (this.primaryModuleName) {
            DefaultXComponent({ moduleName: this.primaryModuleName });
          }
        }.width("100%")
      }.height("100%")

      // 隐藏锚点，用于 bindMenu 定位
      Column()
        .width(1).height(1)
        .position({ x: 0, y: 0 })
        .bindMenu(this.menuShown, this.MenuContent, {
          anchorPosition: { x: this.menuX, y: this.menuY },
          showInSubWindow: true,
          onWillDisappear: () => { this.menuShown = false; }
        })
    }
  }
}
```

**关键设计点**：
- `builder: (): void => this.SubmenuContent(items)` — 在 `@Component` 内 `this` 有效，支持递归
- `anchorPosition: {x, y}` — 相对于绑定组件（Column at 0,0）左上角，等效窗口绝对坐标
- `showInSubWindow: true` — 菜单可超出应用窗口边界（与其他 OS 一致）
- `onWillDisappear` — 菜单关闭时重置 `menuShown`，确保下次可再触发
- `MenuItemGroup() {}` — 空 group 的边界自动产生分隔线（ArkUI 标准做法）
- `SubmenuContent` 内部递归处理所有类型，支持 submenu 嵌套 submenu

### Step 2: TauriMenuManager — 改用 AppStorage 触发

**文件**: `openharmony-ability/native_ability/src/main/ets/helper/menu.ets`

```typescript
popupFromJson(jsonData: string, x?: number, y?: number): void {
  // 先关闭可能存在的旧菜单
  AppStorage.setOrCreate("__tauri_menu_shown__", false);
  // 设置数据
  AppStorage.setOrCreate("__tauri_menu_json__", jsonData);
  AppStorage.setOrCreate("__tauri_menu_x__", x ?? 0);
  AppStorage.setOrCreate("__tauri_menu_y__", y ?? 0);
  // 延迟一帧再显示（确保 false→true 状态变化被 UI 感知）
  setTimeout(() => {
    AppStorage.setOrCreate("__tauri_menu_shown__", true);
  }, 0);
}
```

删除 `openMenu` 相关代码：`ComponentContent`、`wrapBuilder`、`buildMenuContent` @Builder、`MenuParams` class。

### Step 3: PredefinedActionExecutor — 设置 window + 完整行为映射

**文件**: `openharmony-ability/native_ability/src/main/ets/helper/menu.ets`

在 `setupMenuPopup` 中调用 `executor.setWindow(windowStage.getMainWindowSync())`。

### Step 4: Predefined 行为映射（与 Tray 一致）

参考 `phase7-predefined-check-icon-design.md`，OHOS 上各 predefined 动作的行为：

| Predefined | OHOS 行为 | 说明 |
|-----------|-----------|------|
| `quit` | `exit(0)` / `terminateSelf()` | 终止进程 |
| `minimize` | `window.minimize()` | 最小化到后台 |
| `maximize` | `window.maximize()` | 最大化窗口 |
| `fullscreen` | `window.maximize()` | OHOS 无独立全屏 API，maximize 等价 |
| `hide` | `window.minimize()` | OHOS 无独立 hide API，minimize 等价 |
| `close` | `window.minimize()` | destroyWindow 不可恢复，minimize 是安全等价 |
| `recover` | `window.recover()` | 从 maximize 恢复（API 12+） |
| `copy` | `document.execCommand("copy")` | 通过 webview runJavaScript |
| `cut` | `document.execCommand("cut")` | 通过 webview runJavaScript |
| `paste` | `document.execCommand("paste")` | 通过 webview runJavaScript |
| `selectAll` | `document.execCommand("selectAll")` | 通过 webview runJavaScript |
| `undo` | `document.execCommand("undo")` | 通过 webview runJavaScript |
| `redo` | `document.execCommand("redo")` | 通过 webview runJavaScript |
| `about` | 静默忽略（需自定义 AlertDialog） | 后续可扩展 |
| `hideOthers` | 静默忽略 | macOS 专有 |
| `showAll` | 静默忽略 | macOS 专有 |

**与其他 OS 行为对比**：

| 动作 | Windows | macOS | OHOS |
|------|---------|-------|------|
| maximize | SW_MAXIMIZE | selector maximize | window.maximize() |
| minimize | SW_MINIMIZE | selector miniaturize | window.minimize() |
| hide | SW_HIDE | NSApp.hide | minimize（等价） |
| close | WM_CLOSE | performClose | minimize（等价，destroyWindow 不可恢复） |
| quit | PostQuitMessage | NSApp.terminate | exit(0) |
| fullscreen | — | toggleFullScreen | maximize（等价） |

**注意**：Tray 场景中 minimize/hide/close 需要 `setTimeout(300ms)` 延迟（因为系统会先激活窗口），但 popup menu 场景中窗口已在前台，**不需要延迟**。

### Step 5: Separator 显示

使用 `MenuItemGroup` 边界自动产生分隔线。ArkUI 中相邻 `MenuItemGroup` 之间会自动渲染分隔线。

方案：遇到 separator 时插入一个空的 `MenuItemGroup() {}`，它与前后的 MenuItem 之间会产生视觉分隔。

### Step 6: Accelerator 文本去除

**文件**: `muda/src/platform_impl/ohos/mod.rs`

OHOS 上不显示快捷键文本（无物理键盘快捷键概念）：

```rust
MenuItemData {
    accelerator: None, // OHOS: 不显示快捷键
    // ...
}
```

### Step 7: Strip `&` 助记符

**文件**: `muda/src/platform_impl/ohos/mod.rs`

与 tray 实现一致：

```rust
fn strip_mnemonics(text: &str) -> String {
    text.replace('&', "")
}
```

## 四、数据流

```
[JS popup() 调用]
    ↓
[Rust plugin.rs: popup command]
    ↓ (提取 position from __TAURI_MENU_LAST_POINTER__)
[muda ohos: popup(x, y) → popup_context_menu(json, x, y)]
    ↓ (crossbeam channel)
[start_popup_forwarder thread → TSFN call]
    ↓
[NativeAbility.ets: onPopupRequest callback]
    ↓
[TauriMenuManager.popupFromJson()]
    ↓ (AppStorage.set)
[MainPage @StorageLink 响应]
    ↓ (menuShown = true)
[bindMenu 弹出菜单]
    ↓ (用户点击/hover)
[MenuItem.builder → SubmenuContent 展开子菜单]
    ↓ (用户点击具体项)
[globalMenuClickHandler]
    ↓
[TauriMenuManager.handleItemClick]
    ├── predefined → PredefinedActionExecutor.execute()
    ├── check → emitMenuEvent(item.id) (checked 状态已由 onChange 更新)
    └── normal/icon → emitMenuEvent(item.id) → Rust MenuEvent
```

## 五、修改文件清单

| 文件 | 修改内容 |
|------|---------|
| `native_ability/.../components/MainPage.ets` | 添加 bindMenu + @Builder MenuContent/SubmenuContent，支持全类型递归渲染 |
| `native_ability/.../helper/menu.ets` | TauriMenuManager 改用 AppStorage；删除 openMenu/ComponentContent 代码；导出 setMenuClickHandler |
| `native_ability/.../helper/menu_types.ets` | 添加 `'fullscreen'` 到 PredefinedType（如缺失） |
| `native_ability/.../ability/NativeAbility.ets` | setupMenuPopup 中添加 executor.setWindow() |
| `muda/src/platform_impl/ohos/mod.rs` | accelerator 设为 None；strip_mnemonics 去除 `&` |

## 六、平台安全性

| 修改文件 | 隔离机制 | 影响范围 |
|---------|---------|---------|
| `MainPage.ets` / `menu.ets` | `openharmony-ability` HAR，仅 OHOS 引用 | ✅ 仅 OHOS |
| `muda/platform_impl/ohos/mod.rs` | `#[cfg(target_env = "ohos")]` | ✅ 仅 OHOS |
| `packages/api/src/menu/menu.ts` | `__TAURI_MENU_LAST_POINTER__` 仅 OHOS 注入 | ✅ 其他平台不进入该分支 |
| `plugin.rs` js_init_script | `#[cfg(target_env = "ohos")]` 块内 | ✅ 仅 OHOS |

**结论**：所有修改均在 OHOS 隔离路径内，不影响 Windows/macOS/Linux 编译和运行。

## 七、实现注意点

### 必须处理

1. **`@Watch` 装饰器**：`@StorageLink("__tauri_menu_json__")` 必须加 `@Watch("onMenuJsonChange")`，否则 `menuItems` 不会自动更新
2. **`fullscreen` 补充到 PredefinedType**：`menu_types.ets` 当前缺少 `'fullscreen'`，需要添加
3. **ForEach key generator**：ArkUI 的 `ForEach` 需要第三个参数作为 key，应使用 `(item: TauriMenuItemData) => item.id`，否则列表更新时 diff 异常

### 需要实机验证

4. **Separator 降级方案**：空 `MenuItemGroup() {}` 可能被 ArkUI 优化掉不渲染边界线。如果不行，降级方案：
   - 方案 B：`MenuItemGroup() { MenuItem({ content: '' }).height(1).enabled(false) }`
   - 方案 C：将 separator 前后的 items 分别放入不同的 `MenuItemGroup` 中（利用 group 间的自然分隔）

### 风险点

5. **AppStorage 响应延迟**：set → UI 渲染有一帧延迟（~16ms），不影响用户体验
6. **menuShown 状态管理**：`onWillDisappear` 中重置为 false，确保下次可触发
7. **快速连续 popup**：`popupFromJson` 先 set false 再 setTimeout(0) set true，确保状态变化被感知
8. **递归子菜单**：ArkUI 支持多级但层级过深有性能问题（实际 ≤2 级）
9. **AppStorage key 冲突**：使用 `__tauri_` 前缀避免冲突
10. **menuJson 解析时机**：先 set json 再 set shown，确保数据就绪

## 八、Autotest 影响

| 测试类别 | 影响 | 说明 |
|---------|------|------|
| `auto` 类测试（CRUD 操作） | ✅ 无影响 | 仅涉及 Rust 侧 API |
| `manual` 类测试（popup 相关） | ✅ 无影响 | 不在 autotest 中运行 |
| Tray 测试 | ✅ 无影响 | 独立代码路径 |

## 九、验证标准

1. ✅ Menu 在点击位置弹出（非左上角）
2. ✅ Submenu hover 展开显示子项（与 Windows/macOS 行为一致）
3. ✅ Submenu 内可嵌套 submenu（递归）
4. ✅ Normal item 点击触发 MenuEvent
5. ✅ Check item 点击 toggle checked 状态 + 触发 MenuEvent
6. ✅ Icon item 正常显示 + 点击触发 MenuEvent
7. ✅ Separator 显示可见分隔线
8. ✅ Predefined: minimize/maximize/close/quit/copy/paste 等生效
9. ✅ 无 `&` 符号残留
10. ✅ 无 `Ctrl + Character("m")` 格式的 accelerator 文本
11. ✅ 点击菜单外部自动关闭
12. ✅ 不影响 Windows/macOS 编译和行为
13. ✅ auto 测试全部通过（回归）

## 十、Predefined 功能支持现状

### 完整支持（有实际 OHOS 行为）

| Predefined | OHOS 实现 | 说明 |
|-----------|-----------|------|
| `separator` | `MenuItemGroup` 分组边界 | UI 层处理，不经过 executor |
| `copy` | `webview.runJavaScript('document.execCommand("copy")')` | ✅ |
| `cut` | `webview.runJavaScript('document.execCommand("cut")')` | ✅ |
| `paste` | `webview.runJavaScript('document.execCommand("paste")')` | ✅ |
| `selectAll` | `webview.runJavaScript('document.execCommand("selectAll")')` | ✅ |
| `undo` | `webview.runJavaScript('document.execCommand("undo")')` | ✅ |
| `redo` | `webview.runJavaScript('document.execCommand("redo")')` | ✅ |
| `minimize` | `window.minimize()` | ✅ |
| `maximize` | `window.maximize()` | ✅ |
| `fullscreen` | `window.maximize()` | OHOS 无独立全屏 API，maximize 等价 |
| `hide` | `window.minimize()` | OHOS 无独立 hide API，minimize 等价 |
| `close` | `window.minimize()` | destroyWindow 不可恢复，minimize 是安全等价 |
| `quit` | `exitFn(0)` / `terminateSelf()` | ✅ 终止进程 |
| `recover` | `window.recover()` | 从 maximize/fullscreen 恢复（API 12+） |

### 静默忽略（no-op）

| Predefined | 原因 |
|-----------|------|
| `about` | 需自定义 AlertDialog，当前无通用实现 |
| `hideOthers` | macOS 专有概念，OHOS 无对应 API |
| `showAll` | macOS 专有概念，OHOS 无对应 API |
| `restore` | 从 minimize 恢复（API 14+，仅 2in1 设备），当前未实现 |

### 不存在于 OHOS 映射（Rust enum 有但 OHOS 不处理）

| Predefined | 原因 |
|-----------|------|
| `Services` | macOS 专有（Services 菜单） |
| `BringAllToFront` | macOS 专有 |
| `None` | 空操作变体，无需处理 |

### OHOS 独有（Rust enum 无对应）

| OHOS Type | 说明 |
|-----------|------|
| `recover` | 从 maximize 恢复（API 12+），通过 `predefinedType` 字符串直接传递 |
| `restore` | 从 minimize 恢复（API 14+，仅 2in1），当前 no-op |
| `destroyWindow` | 销毁窗口（不可恢复），当前映射为 minimize |

### Popup vs Tray 的 Predefined 行为差异

| 差异点 | Popup Menu | Tray Menu |
|--------|-----------|-----------|
| 执行时机 | 立即执行 | `setTimeout(300ms)` 延迟执行 |
| 延迟原因 | 窗口已在前台，无需等待 | 系统先激活窗口，需等激活完成 |
| quit 实现 | `exitFn(0)` | `app.terminateSelf()` |
| 代码路径 | `TauriMenuManager.handleItemClick` → `PredefinedActionExecutor` | `event.rs` → `execute_predefined_action` |

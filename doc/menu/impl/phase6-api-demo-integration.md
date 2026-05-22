# Phase 6: api-demo 集成与端到端验证

## 目标
完成 Tauri Menu API 在 api-demo 应用中的端到端集成，使 `@tauri-apps/api/menu` 前端 API 可在 OHOS 设备上正常工作，包括菜单创建、popup 显示、点击事件回传、预定义项执行等完整链路。

> **⚠️ 注意**：本文档描述的 `openMenu` 方案已在 **Phase 9** 中被 `bindMenu` 方案完全替代。Phase 6 的 `openMenu` 实现因模块级 `@Builder` 无 `this` 上下文导致子菜单渲染 crash（`TypeError: Cannot read property observeComponentCreation2 of undefined`），Phase 9 重构为 `bindMenu(isShow)` + `@Component @Builder` 方案。详见 [phase9-popup-bindcontextmenu-design.md](./phase9-popup-bindcontextmenu-design.md) 和 [DEBUG.md Fix 6](../DEBUG.md#fix-6-openmenu--componentcontent-渲染子菜单-crash)。

## 依赖
- Phase 0-5: 所有模块完成并编译通过
- api-demo 应用可在 OHOS 设备上运行
- `@tauri-apps/api/menu` 前端包已就绪
- 目标设备 API 版本 >= 18（用于 `promptAction.openMenu`）

## 现状分析

### ✅ 已完成 (Phase 0-5)
| 层级 | 组件 | 状态 |
|------|------|------|
| Rust | muda OHOS 后端 | ✅ 完成 |
| Rust | openharmony-ability menu 模块 | ✅ 完成 |
| Rust | tauri::menu OHOS 分支 | ✅ 完成 |
| ArkTS | menu_types.ets 类型定义 | ✅ 完成 |
| ArkTS | menu.ets 菜单管理器 | ✅ 完成 |
| ArkTS | menu_state.ets 状态管理 | ✅ 完成 |
| ArkTS | TauriMenu.ets 组件 | ✅ 完成 |
| 集成 | popup forwarding (channel → TSFN) | ✅ 完成 |
| 编译 | OHOS 目标编译通过 | ✅ 完成 |

### ❌ 待完成 (Phase 6)
| 问题 | 文件 | 优先级 | 状态 |
|------|------|--------|------|
| Popup 机制需重构为 `openMenu` | `menu.ets` / `NativeAbility.ets` | P0 | ✅ 已完成 |
| `menu_plugin.rs` 排除 OHOS | `examples/api/src-tauri/src/menu_plugin.rs` | P0 | ✅ 已完成 |
| `menu_plugin::init()` 调用排除 OHOS | `examples/api/src-tauri/src/lib.rs` | P0 | ✅ 已完成 |
| `PopupMenu<R>` 状态未管理 | `examples/api/src-tauri/src/lib.rs` | P0 | ✅ 已完成 |
| `toggle` 命令 OHOS 不兼容 | `examples/api/src-tauri/src/menu_plugin.rs` | P1 | ✅ 已完成 |
| ThreadsafeFunction API 兼容性 | `openharmony-ability/crates/ability/src/menu/mod.rs` | P0 | ✅ 已完成 |
| tray 调用隔离 (本次只验证 menu) | `examples/api/src-tauri/src/lib.rs` | P1 | ✅ 已完成 |
| OHOS 构建验证 | - | P0 | ✅ 已完成 (HAP 生成成功) |
| 窗口菜单未设置 | `lib.rs` setup | P1 | ⏸️ 暂缓 |
| 菜单点击事件回传链路 | NAPI → Rust → JS | P1 | ✅ 已完成 |

## 架构说明

### Popup 方案选择：`promptAction.openMenu` (API 18+) — **已被 Phase 9 bindMenu 替代**

> 以下为 Phase 6 的历史方案记录，已不再使用。Phase 9 改用 `bindMenu(isShow)` + `@Component @Builder` 方案。

基于 OpenHarmony 官方文档，Phase 6 采用 `promptAction.openMenu` 方案：

| 特性 | `bindContextMenu` | `openMenu` (推荐) |
|------|------------------|-------------------|
| 需要 UI 组件 | ✅ 必须绑定到 UI 树 | ❌ 不需要，纯 API 调用 |
| 位置控制 | offset / anchorPosition | anchorPosition 精确控制 |
| 关闭回调 | aboutToDisappear | Promise.then() |
| 多窗口安全 | ❌ 状态可能冲突 | ✅ 每次独立调用 |
| 代码复杂度 | 高 (占位组件+状态管理) | 低 (直接调用) |
| API 版本 | 12+ | 18+ |

### 完整调用链路

```
前端 JS: Menu.new() / menu.popup()
    ↓
@tauri-apps/api/menu (JS SDK)
    ↓
invoke('plugin:menu|popup') 
    ↓
Rust: tauri::menu::Menu::popup()
    ↓
popup_inner() → #[cfg(target_env = "ohos")]
    ↓
muda::platform_impl::ohos::Menu::popup(x, y)
    ↓
openharmony_ability::menu::popup_context_menu(json, x, y)
    ↓
POPUP_CHANNEL → background thread → NAPI TSFN
    ↓
ArkTS: on_popup_request callback
    ↓
TauriMenuManager.popupFromJson()
    ↓
1. 获取 Target (getFrameNodeByUniqueId)
2. 创建 ComponentContent (nestingBuilderSupported: true)
3. promptAction.openMenu(contentNode, target, options)
    ↓
ArkUI 渲染菜单 (独立窗口/覆盖层)
    ↓
用户点击菜单项
    ↓
Builder.onClick → params.manager.handleItemClick(item)
                → promptAction.closeMenu(params.contentNode)
    ↓
PredefinedActionExecutor.execute() 或 emit event to Rust
    ↓
Promise.then() → 系统自动回收资源
```

### 需要打通的关键断点

```
断点 1: 前端 API → Rust
  @tauri-apps/api/menu 调用的是 invoke('plugin:menu|...')
  但 OHOS 上 menu_plugin 未注册
  解决方案: 修改 menu_plugin.rs cfg 包含 OHOS

断点 2: Rust → ArkTS popup
  popup_context_menu() 写入 POPUP_CHANNEL
  解决方案: 确保 start_popup_forwarder() 在 plugin init 时调用

断点 3: ArkTS callback → UI 渲染 (已解决)
  使用 promptAction.openMenu() 直接弹出，无需 UI 组件
  解决方案: 重构 TauriMenuManager.popupFromJson() 使用 openMenu

断点 4: UI 点击 → Rust 事件回传
  用户点击菜单项后，事件需要回传到 Rust/JS
  解决方案: handleItemClick() 执行操作后调用 closeMenu()
```

## 工作内容

### 6.1 重构 Popup 机制使用 `openMenu` ✅ 已完成

**文件**: `openharmony-ability/native_ability/src/main/ets/helper/menu.ets`

**说明**: 本节描述的是对 `menu.ets` 的**完全替换**。原有的 `bindContextMenu` + `menuStateController` 方案已被 `promptAction.openMenu` 方案替代。`PredefinedActionExecutor` 类保持不变，`TauriMenuManager` 类已重构。

### 6.2 修复 NativeAbility.ets 中 TauriMenuManager 构造 ✅ 已完成

**文件**: `openharmony-ability/native_ability/src/main/ets/ability/NativeAbility.ets`

### 6.3 注册 OHOS Menu Plugin ✅ 已完成

**文件**: `examples/api/src-tauri/src/menu_plugin.rs`

### 6.4 管理 PopupMenu 状态 on OHOS ✅ 已完成

**文件**: `examples/api/src-tauri/src/lib.rs`

需要修改两个地方：

**1. menu_plugin 注册 (包含 OHOS)**: ✅ 已完成
**2. PopupMenu 状态管理 (包含 OHOS)**: ✅ 已完成
**3. tray 调用隔离**: ✅ 已完成 (注释掉 `tray::create_tray(handle)?;`)

### 6.5 修复 ThreadsafeFunction API 兼容性 ✅ 已完成

**文件**: `openharmony-ability/crates/ability/src/menu/mod.rs`

napi-ohos 1.1.6 的 API 与旧版本不同，需要使用 `build_threadsafe_function` + `build_callback` 方式创建 ThreadsafeFunction。
```rust
// 修改前
#[cfg(all(desktop, not(test), not(target_env = "ohos")))]
app.manage(PopupMenu(
  tauri::menu::MenuBuilder::new(app)
    .check("check", "Tauri is awesome!")
    .text("text", "Do something")
    .copy()
    .build()?,
));

// 修改后
#[cfg(any(
  all(desktop, not(test)),
  target_env = "ohos"
))]
app.manage(PopupMenu(
  tauri::menu::MenuBuilder::new(app)
    .check("check", "Tauri is awesome!")
    .text("text", "Do something")
    .copy()
    .build()?,
));
```

### 6.5 连接 @tauri-apps/api/menu 到 OHOS

确认 `menu_plugin` 在 OHOS 上注册后，JS API 应可正常工作。

### 6.6 完善菜单点击事件回传

通过 `handleItemClick()` 执行操作，菜单关闭由 `openMenu` 的 Promise 处理。

### 6.7 设置窗口菜单（可选）

**建议**: 暂不实现窗口菜单，优先完成 popup 功能。

## 降级方案 (API < 18)

如果目标设备 API 版本 < 18，回退到 `bindContextMenu` + 状态管理方案：

```typescript
// menu_state.ets 扩展
export class MenuStateController {
  private currentManager: TauriMenuManager | null = null;
  
  setManager(manager: TauriMenuManager): void {
    this.currentManager = manager;
  }
  
  handleItemClick(item: TauriMenuItemData): void {
    this.currentManager?.handleItemClick(item);
  }
}

// DefaultXComponent.ets
@State menuState: MenuState = menuStateController.getState();

aboutToAppear() {
  menuStateController.subscribe(state => this.menuState = state);
  menuStateController.setManager(this.menuManager);
}

build() {
  Stack() {
    // WebView...
    if (this.menuState.isShown) {
      TauriMenuPopup({
        items: this.menuState.items,
        onItemClick: (item) => menuStateController.handleItemClick(item)
      })
    }
  }
}
```

## 测试方案

### 6.8 自动测试用例

**文件**: `examples/api/src/lib/tests/core.ts`

```typescript
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
```

### 6.9 手动测试用例

**文件**: `examples/api/src/views/TestRunner.svelte`

```typescript
async function manualMenuPopupBasic() {
  await wrapManual('menu.popup.basic', async () => {
    const menu = await Menu.new({
      items: [
        await MenuItem.new({ text: 'Action 1' }),
        await MenuItem.new({ text: 'Action 2' }),
        await PredefinedMenuItem.separator(),
      ]
    });
    await menu.popup();
    onMessage('Menu should appear. Click any item to close.');
  });
}
```

### 6.10 Welcome 页面 Context Menu 测试

**文件**: `examples/api/src/views/Welcome.svelte`

修复 menu_plugin 后，该按钮应该可以正常工作。

### 6.11 Menu.svelte 页面测试

**文件**: `examples/api/src/views/Menu.svelte`

验证 `Menu.new()`, `Submenu.new()`, `m.popup()` 等 API。

## 验证清单

### 编译验证
- [ ] `menu.ets` 重构后编译通过
- [ ] `NativeAbility.ets` 修改后编译通过
- [ ] `menu_plugin.rs` 包含 OHOS 后编译通过
- [ ] `lib.rs` PopupMenu 状态管理修改后编译通过
- [ ] 完整 OHOS 构建通过

### 功能验证
- [ ] Welcome 页面 "Context menu" 按钮可弹出菜单
- [ ] 菜单项文本正确显示
- [ ] 分隔线正确显示
- [ ] 点击菜单项可关闭菜单
- [ ] 预定义项（copy, minimize 等）可执行
- [ ] Menu.svelte 页面 "Popup" 按钮可工作
- [ ] 子菜单可展开
- [ ] 菜单点击事件可回传到 JS

### 测试验证
- [ ] 自动测试用例通过
- [ ] 手动测试用例执行
- [ ] 测试报告生成

## 已知限制

1. **窗口菜单栏**: OHOS 可能不支持传统窗口菜单栏，暂不实现
2. **键盘快捷键**: 需要额外实现全局键盘监听（不在本阶段范围）
3. **多菜单管理**: 同时只能显示一个菜单
4. **位置精度**: `offset` 定位可能不够精确，API 20+ 可使用 `anchorPosition`
5. **页面构建时序**: 菜单必须等待页面全部构建完成后才能展示

## 风险评估与缓解

### 风险 1: Target 获取失败 (P0)

**问题**: `openMenu` 要求传入有效的 `TargetInfo`，若 target 无效则无法弹出。

**官方文档方案**:
```typescript
let frameNode: FrameNode | null = context.getFrameNodeByUniqueId(uniqueId);
let frameNodeTarget = frameNode?.getFirstChild();
frameNodeTarget = frameNodeTarget?.getChild(0);
let targetId = frameNodeTarget?.getUniqueId();
```

**缓解措施**:
1. 在 `NativeAbility.ets` 的 `onCreate` 中获取 `uniqueId` 并传入 Manager
2. 使用官方推荐的链式调用获取 Target ID
3. 如果获取失败 (undefined)，fallback 到 `0`
4. 首次 popup 时打印日志验证 target 是否有效

**验证步骤**:
```typescript
// 在 popupFromJson 中添加
console.info(`[Menu] Target ID: ${target.id}`);
```

### 风险 2: ComponentContent 内存泄漏 (P0)

**问题**: 快速连续 popup 可能导致旧实例未释放。

**官方文档方案**: 每次 onClick 创建新实例，系统自动回收。

**缓解措施**:
```typescript
// 每次 popupFromJson 创建新实例
const contentNode = new ComponentContent(...);
this.uiContext.getPromptAction().openMenu(contentNode, target, options)
  .then(() => {
    // 系统自动回收，无需手动清理
  });
```

### 风险 3: Builder 函数作用域 (P1)

**问题**: `@Builder` 函数必须是模块级函数，不能是类方法。

**缓解措施**: 将 `buildMenuContent` 和 `buildSubmenuContent` 定义为 `menu.ets` 的模块级函数，通过 `MenuParams` 传递 manager 引用。

### 风险 4: 子菜单嵌套支持 (P1)

**问题**: 子菜单使用 `MenuItem.builder` 参数，可能需要 `nestingBuilderSupported: true`。

**缓解措施**: 创建 `ComponentContent` 时始终传入 `{ nestingBuilderSupported: true }`，文档说明此选项用于包含 Popup/Chip 的场景，子菜单应该也适用。

### 风险 5: closeMenu 引用丢失 (P1)

**问题**: 如果 `currentContentNode` 被意外置 null，无法关闭菜单。

**缓解措施**: 在 `MenuParams` 中保存 contentNode 引用，Builder 中可直接调用 `promptAction.closeMenu(params.contentNode)`。

### 风险 6: MenuParams 循环依赖 (P0)

**问题**: `MenuParams` 需要引用 `contentNode`，但 `contentNode` 创建时需要 `MenuParams`。

**缓解措施**: 使用延迟引用方案：
```typescript
class MenuParams {
  contentNode: ComponentContent<Object> | null = null;  // 先为 null
  // ...
}

const params = new MenuParams(items, this);
const contentNode = new ComponentContent(..., params, ...);
params.contentNode = contentNode;  // 创建后赋值
```

## 工期
- 2-3 天 (代码修改完成，构建成功，待设备测试)

## 输出物
- 重构后的 `menu.ets` (使用 openMenu，完全替换) ✅
- 修改后的 `NativeAbility.ets` ✅
- 修改后的 `menu_plugin.rs` ✅
- 修改后的 `lib.rs` ✅
- 修复后的 `openharmony-ability/crates/ability/src/menu/mod.rs` ✅
- 测试用例（自动 + 手动）🟡 待开始
- 测试报告 🟡 待开始

## 参考文档
- [Phase 5: 集成与测试](./phase5-integration-testing.md)
- [Phase 4: Popup 集成](./phase4-popup-integration.md)
- [openMenu API](../reference/arkts-popup-and-menu-components-uicontext-menu.md)
- [Menu 组件](../reference/menu.md)
- [OpenHarmony 官方文档: 不依赖UI组件的全局菜单](../../../openharmony-docs/zh-cn/application-dev/ui/arkts-popup-and-menu-components-uicontext-menu.md)

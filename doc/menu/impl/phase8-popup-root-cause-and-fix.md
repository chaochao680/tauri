# Phase 8: Menu.popup() OHOS 修复方案

> 状态: 设计完成，待实施
> 日期: 2026-05-20
> 审计: 已通过外部审计（phase8-popup-review.md），所有问题已纳入
> 工期: 1 天

---

## TL;DR

OHOS 上 `Menu.popup()` 完全无效果。根因是 `openMenu` API 的 `TargetInfo` 使用了无效的 arbitrary number 作为组件 ID。修复方案：在 MainPage 根组件设置固定 string `.id()`，用 `anchorPosition` 实现绝对定位。同时修复 5 个关联问题使 popup 功能完整可用。

---

## 修改文件清单

| # | 文件 | 改动摘要 |
|---|------|---------|
| 1 | `openharmony-ability/.../MainPage.ets` | 根 Row 加 `.id("__tauri_internal_menu_popup_anchor__")` |
| 2 | `openharmony-ability/.../menu.ets` | target 改 string id + separator 判断 + handleItemClick 调 emit_menu_event + executor 加 `'close'` |
| 3 | `openharmony-ability/.../menu_types.ets` | PredefinedType 联合类型补 `'close'` |
| 4 | `openharmony-ability/.../NativeAbility.ets` | 删除 uniqueId 生成和传递 |
| 5 | `muda/src/platform_impl/ohos/mod.rs` | `to_menu_item_data()` 中 `self.text.replace("&", "")` |
| 6 | `tauri/crates/tauri/src/menu/plugin.rs` | OHOS cfg: js_init_script + popup handler 坐标获取（含超时） |

不修改: Windows/macOS 路径、`@tauri-apps/api`、`popup_context_menu()` 接口签名。

---

## 实施步骤

### Step 1: MainPage.ets — 添加锚点

```typescript
// openharmony-ability/native_ability/src/main/ets/components/MainPage.ets
build() {
  Row() {
    Column() {
      if (this.primaryModuleName) {
        DefaultXComponent({ moduleName: this.primaryModuleName });
      }
    }.width("100%")
  }
  .height("100%")
  .id("__tauri_internal_menu_popup_anchor__")  // 新增
}
```

### Step 2: NativeAbility.ets — 删除 uniqueId

```typescript
// openharmony-ability/native_ability/src/main/ets/ability/NativeAbility.ets
private setupMenuPopup(): void {
  // ...
  const executor = new PredefinedActionExecutor((code: number) => {
    this.context?.terminateSelf();
  });
  // 删除: const uniqueId = ...
  const menuManager = new TauriMenuManager(uiContext, executor);  // 删除 uniqueId 参数
  // ...
}
```

### Step 3: menu.ets — 核心修复

#### 3.1 构造函数删除 uniqueId

```typescript
export class TauriMenuManager {
  private executor: PredefinedActionExecutor;
  readonly uiContext: UIContext;
  private menus: Map<string, TauriMenuItemData[]> = new Map();

  constructor(uiContext: UIContext, executor: PredefinedActionExecutor) {
    this.uiContext = uiContext;
    this.executor = executor;
  }
}
```

#### 3.2 popupFromJson 改用 string id + anchorPosition

```typescript
popupFromJson(jsonData: string, x?: number, y?: number): void {
  try {
    const items: TauriMenuItemData[] = JSON.parse(jsonData);
    const menuId = `popup_${Date.now()}`;
    this.menus.set(menuId, items);

    const target: TargetInfo = { id: "__tauri_internal_menu_popup_anchor__" };

    const params = new MenuParams(items, this);
    const contentNode = new ComponentContent(
      this.uiContext,
      wrapBuilder(buildMenuContent),
      params,
      { nestingBuilderSupported: true }
    );
    params.contentNode = contentNode;

    const options: MenuOptions = {
      anchorPosition: { x: x ?? 0, y: y ?? 0 },
      placement: Placement.BottomLeft
    };

    this.uiContext.getPromptAction().openMenu(contentNode, target, options)
      .then(() => { this.menus.delete(menuId); })
      .catch((err: BusinessError) => {
        console.error(`[Menu] ${menuId} error: ${err.code} ${err.message}`);
        this.menus.delete(menuId);
      });
  } catch (e) {
    console.error('[Menu] Failed to parse menu JSON:', e);
  }
}
```

#### 3.3 修复 separator 判断

**修改前**（错误 — 永远不命中）：
```typescript
if (item.type === 'separator') { ... }
```

**修改后**（正确 — muda 发送 `{type:"predefined", predefinedType:"separator"}`）：
```typescript
if (item.type === 'predefined' && item.predefinedType === 'separator') {
  MenuItemGroup({ header: '' }) {
    MenuItem({ content: '' }).height(0).enabled(false)
  }
}
```

> ArkUI Menu 组件仅支持 MenuItem/MenuItemGroup 作为子组件，不能用 Divider()。空 MenuItemGroup 触发组间分隔线效果。

#### 3.4 handleItemClick 调用 emit_menu_event

```typescript
import { emit_menu_event } from 'libnative_ability.so';

handleItemClick(item: TauriMenuItemData): void {
  if (item.type === 'predefined' && item.predefinedType) {
    this.executor.execute(item.predefinedType);
  } else {
    emit_menu_event(item.id);
  }
}
```

#### 3.5 PredefinedActionExecutor 添加 'close' 分支

```typescript
case 'close':
  await this.win?.destroyWindow();
  break;
```

同时在 `menu_types.ets` 的 PredefinedType 联合类型中补充 `'close'`。

### Step 4: muda — strip `&` 助记符

```rust
// muda/src/platform_impl/ohos/mod.rs — to_menu_item_data()
text: Some(self.text.replace("&", "")),
```

### Step 5: plugin.rs — 无坐标 popup 的位置获取（OHOS cfg）

#### 5.1 注入 js_init_script

```rust
#[cfg(target_env = "ohos")]
{
  builder = builder.js_init_script(
    r#"(function(){
      var pos = {x:0, y:0};
      document.addEventListener('pointerdown', function(e){
        pos.x = e.clientX; pos.y = e.clientY;
      }, true);
      window.__TAURI_MENU_LAST_POINTER__ = pos;
    })()"#
  );
}
```

#### 5.2 popup handler 异步获取坐标（含超时）

```rust
#[cfg(target_env = "ohos")]
let position = if position.is_none() {
  let (tx, rx) = tokio::sync::oneshot::channel::<Option<Position>>();
  webview.eval_with_callback(
    "JSON.stringify(window.__TAURI_MENU_LAST_POINTER__)",
    move |result| {
      let pos = serde_json::from_str::<serde_json::Value>(&result)
        .ok()
        .and_then(|v| {
          let x = v.get("x")?.as_f64()?;
          let y = v.get("y")?.as_f64()?;
          Some(Position::Logical(crate::LogicalPosition::new(x, y)))
        });
      let _ = tx.send(pos);
    }
  )?;
  tokio::time::timeout(std::time::Duration::from_millis(100), rx)
    .await
    .ok()
    .and_then(|r| r.ok())
    .flatten()
} else {
  position
};
```

超时后 position 为 None，popup 使用 (0,0) 作为 fallback。

---

## 根因分析

### 核心根因：openMenu 的 TargetInfo 无效

**位置**: `menu.ets:201`

```typescript
const target: TargetInfo = { id: this.uniqueId };  // uniqueId = Date.now() + random
```

`TargetInfo.id` 为 number 时，OHOS 解释为组件 UniqueID（系统内部 ID）。arbitrary number 不对应任何组件 → `openMenu` 静默失败。

**修复**: 改用 string id（对应组件 `.id()` 属性值），指向 MainPage 根 Row。

### 次要根因：Separator 判断逻辑错误

muda 序列化 Separator 为 `{type: "predefined", predefinedType: "separator"}`，但 Builder 中判断 `item.type === 'separator'` 永远不命中。

### 关联问题（一并修复）

| 问题 | 根因 | 修复 |
|------|------|------|
| CloseWindow 点击无效 | muda 发 `"close"`，executor 只有 `"destroyWindow"` | 添加 `'close'` case |
| 文本显示 `&Minimize` | `PredefinedMenuItemType::text()` 带 `&` | muda 序列化时 strip |
| 点击无事件回传 | handleItemClick 未调 emit_menu_event | 添加调用 |
| 无坐标时弹在 (0,0) | OHOS 无 cursor_position API | js_init_script + eval_with_callback |

---

## 跨平台对比

| 维度 | Windows | macOS | OHOS（修复后） |
|------|---------|-------|--------------|
| API | `TrackPopupMenu` | `NSMenu.popUpMenu` | `openMenu` |
| 定位 | 屏幕绝对坐标 | 窗口坐标 | anchorPosition（相对全屏锚点 = 窗口坐标） |
| 需要锚点 | 否（只要 HWND） | 否（只要 NSView） | 是（string id） |
| 无坐标时 | `GetCursorPos()` | `NSEvent::mouseLocation()` | eval_with_callback 读 pointerdown |
| 事件回传 | muda 内部 dispatch | muda 内部 dispatch | emit_menu_event NAPI |

三个平台对外 API 行为一致：`popup(at?)` 传坐标则用坐标，不传则用最后交互位置。

---

## 数据流

```
popup(at) 有坐标:
  Rust popup_inner(Some(pos)) → muda popup(x, y) → ArkTS anchorPosition: {x, y}

popup() 无坐标 (OHOS cfg):
  用户 pointerdown → JS 存 {x,y} → popup handler eval_with_callback 读取
  → popup_inner(Some(pos)) → muda popup(x, y) → ArkTS anchorPosition: {x, y}
```

---

## Builder 完整渲染逻辑

```typescript
@Builder
function buildMenuContent(params: MenuParams) {
  Menu() {
    ForEach(params.items, (item: TauriMenuItemData) => {
      if (item.type === 'predefined' && item.predefinedType === 'separator') {
        // 分隔线：空 MenuItemGroup 触发组间分隔
        MenuItemGroup({ header: '' }) {
          MenuItem({ content: '' }).height(0).enabled(false)
        }
      } else if (item.type === 'submenu') {
        MenuItem({
          content: item.text ?? '',
          builder: () => buildSubmenuContent(item.submenuItems ?? [], params.manager, params.contentNode)
        })
      } else if (item.type === 'check') {
        MenuItem({ content: item.text ?? '' })
          .enabled(item.enabled ?? true)
          .selected(item.checked ?? false)
          .selectIcon(true)
          .onChange((checked: boolean) => {
            item.checked = checked;
            params.manager.handleItemClick(item);
          })
      } else {
        // item / icon / predefined(non-separator)
        // text 中的 & 已在 muda 序列化时去除
        MenuItem({ content: item.text ?? item.predefinedType ?? '', labelInfo: item.accelerator })
          .enabled(item.enabled ?? true)
          .onClick(() => {
            params.manager.handleItemClick(item);
            if (params.contentNode) {
              params.manager.uiContext.getPromptAction().closeMenu(params.contentNode);
            }
          })
      }
    })
  }
}
```

---

## 风险与缓解

| 风险 | 严重度 | 缓解措施 |
|------|--------|---------|
| `anchorPosition` 需 API 20+ | 中 | 编译 SDK ≥ 20 即可；保留 placement 作 fallback |
| `eval_with_callback` WebView 异常时不返回 | 中 | 100ms 超时，fallback 到 (0,0) |
| `emit_menu_event` NAPI 导入失败 | 低 | handleItemClick 在用户点击回调中执行，非 render() 上下文，风险低 |
| 锚点 id 冲突 | 极低 | 用户代码在 WebView 内，不设置 ArkUI 组件 id |
| `.id()` 影响布局/事件 | 无 | 通用属性，不影响布局计算和事件分发 |

### eval_with_callback 死锁分析（来自 tray 经验）

tray 调试中发现两类死锁：
- Fix 19: Chrome_IOThread 持 Mutex → 主线程等 Mutex → 循环等待
- Fix 20: `run_on_main_thread` + `rx.recv()` 阻塞 Chrome_IOThread → TSFN 需要该线程

本方案**不会**触发这两类死锁：
- popup handler 是 async，`rx.await` 只挂起 Future 不阻塞线程
- eval_with_callback 回调在 WebView JS 线程执行，不需要 ArkTS 主线程

**降级方案**（如果设备验证中 eval_with_callback 不可靠）：
```rust
// Rust 侧 AtomicU64 对 + IPC command 从 JS 推送坐标
static LAST_POINTER_X: AtomicU64 = AtomicU64::new(0);
static LAST_POINTER_Y: AtomicU64 = AtomicU64::new(0);
```

---

## API 版本要求

| API | 最低版本 | 说明 |
|-----|---------|------|
| `openMenu` | API 18+ | PromptAction 菜单弹出 |
| TargetInfo string id | API 18+ | 通过 `.id()` 字符串定位 |
| `anchorPosition` | API 20+ | 精确定位（覆盖 placement） |

- 模板 `compatibleSdkVersion` = API 12（最低运行时版本）
- 编译 SDK 需 ≥ API 20
- 当前实现假设运行时 API ≥ 20

---

## 验证清单

| 测试项 | 预期结果 |
|--------|---------|
| 基本 Popup | 菜单弹出，submenu 展开显示 item |
| 文本显示 | "Minimize" 而非 "&Minimize" |
| Separator | 两个 item 之间显示分隔线 |
| Normal 点击 | 菜单关闭，Rust 收到 MenuEvent |
| Check 切换 | 勾选状态切换，Rust 收到 MenuEvent |
| CloseWindow | 窗口关闭 |
| Minimize | 窗口最小化 |
| 无坐标 popup | 菜单弹出在最后 pointerdown 位置 |
| 子菜单展开 | hover 展开子菜单 |
| 点击外部 | 菜单关闭 |
| 回归: WebView | MainPage 加 .id() 后正常工作 |
| 回归: 触摸事件 | 根 Row 加 .id() 不影响事件穿透 |

---

## 来自 Tray 调试的关键教训

| Tray Bug | 教训 | 本阶段应用 |
|----------|------|-----------|
| Fix 19: Mutex 持锁死锁 | 不在持锁时执行跨线程操作 | eval_with_callback 是 async await，不持锁 |
| Fix 20: run_on_main_thread 死锁 | 避免阻塞 Chrome_IOThread | popup handler 是 async |
| Fix 23: Function::call() 静默失败 | render() 上下文中 NAPI 调用无效 | emit_menu_event 在点击回调中调用 |
| Phase 7: strip_mnemonics | `&` 需在序列化时去除 | muda to_menu_item_data() 中 strip |
| Phase 7: executePredefinedAction | predefined 通过 TSFN NonBlocking | menu executor 在 ArkTS 主线程直接执行 |

---

## 实施顺序

1. **核心修复**（让菜单弹出）：Step 1-3.3（string id + separator）
2. **必要补充**（让菜单可用）：Step 3.4-3.5 + Step 4（事件 + close + strip &）
3. **设备验证**：popup 弹出 + 位置 + 事件 + predefined
4. **无坐标 popup**：Step 5（eval_with_callback + 超时，可后续单独验证）

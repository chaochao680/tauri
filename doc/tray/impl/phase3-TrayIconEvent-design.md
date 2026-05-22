# API 设计：TrayIconEvent

> 职责：定义托盘图标事件类型，提供事件接收机制
> 代码位置：`tray-icon/src/lib.rs` (类型定义) + `tray-icon/src/platform_impl/ohos/mod.rs` (事件转发)
> 独立性：✓ 可独立验证（依赖 phase2 TrayIconBuilder 创建图标）

## 参考资料

- [../reference/status_bar_api.md](../reference/status_bar_api.md) - statusBarIconClick / rightMenuClick 事件定义
- [../reference/tray-reference.md](../reference/tray-reference.md) - 事件监听示例代码（第213-226行）

---

## 一、接口定义

### 1.1 TrayIconEvent 枚举（由 tray-icon lib.rs 定义）

```rust
// tray-icon/src/lib.rs 已定义，无需修改

#[derive(Debug, Clone)]
pub enum TrayIconEvent {
    /// 点击事件
    Click {
        id: TrayIconId,
        position: PhysicalPosition<f64>,
        rect: Rect,
        button: MouseButton,
        button_state: MouseButtonState,
    },
    /// 双击事件（Windows Only）
    DoubleClick {
        id: TrayIconId,
        position: PhysicalPosition<f64>,
        rect: Rect,
        button: MouseButton,
    },
    /// 鼠标进入
    Enter {
        id: TrayIconId,
        position: PhysicalPosition<f64>,
        rect: Rect,
    },
    /// 鼠标移动
    Move {
        id: TrayIconId,
        position: PhysicalPosition<f64>,
        rect: Rect,
    },
    /// 鼠标离开
    Leave {
        id: TrayIconId,
        position: PhysicalPosition<f64>,
        rect: Rect,
    },
}
```

### 1.2 TrayIconEvent 方法

```rust
// tray-icon/src/lib.rs 已定义

impl TrayIconEvent {
    /// 获取事件 ID
    pub fn id(&self) -> &TrayIconId;
    
    /// 获取事件接收器
    pub fn receiver() -> &'static TrayIconEventReceiver;
    
    /// 设置事件处理器
    pub fn set_event_handler<F: Fn(TrayIconEvent) + Send + Sync + 'static>(f: Option<F>);
    
    /// 发送事件（内部使用）
    pub(crate) fn send(event: TrayIconEvent);
}
```

---

## 二、MouseButton 定义

### 2.1 MouseButton 枚举

```rust
// tray-icon/src/lib.rs 已定义，无需修改

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum MouseButton {
    #[default]
    Left,
    Right,
    Middle,
}
```

### 2.2 OHOS 支持情况

| MouseButton | OHOS 支持 | 事件来源 | OHOS 返回值 |
|-------------|-----------|----------|-------------|
| Left | ✓ | `statusBarIconClick` | `{ iconClickType: "leftClick" }` |
| Right | ✓ | `rightMenuClick` | `{ menuCode: "xxx" }` |
| Middle | ✗ | 无中键概念 | 不发送事件 |

**重要区别**：
- **Left**：来自 `statusBarIconClick` 事件（左键点击图标）
- **Right**：来自 `rightMenuClick` 事件（右键点击菜单项，**不是右键点击图标本身**）
- **右键点击图标本身** → OHOS **不发送事件**，只弹出菜单

---

## 三、MouseButtonState 定义

### 3.1 MouseButtonState 枚举

```rust
// tray-icon/src/lib.rs 已定义，无需修改

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum MouseButtonState {
    #[default]
    Up,
    Down,
}
```

### 3.2 OHOS 支持情况

| MouseButtonState | OHOS 支持 | 说明 |
|------------------|-----------|------|
| Up | ✓ | OHOS 只发送点击完成事件（不区分按下/释放） |
| Down | ✗ | OHOS 不提供此状态 |

**OHOS 实现策略**：所有 Click 事件 `button_state` 固定为 `Up`

---

## 四、OHOS 事件源对照

### 4.1 两类独立事件

OHOS 提供两个独立的事件监听接口：

| OHOS 事件 | 触发条件 | 返回数据 | 转换为 TrayIconEvent |
|-----------|----------|----------|---------------------|
| `statusBarIconClick` | 左键点击托盘图标 | `{ iconClickType: "leftClick" }` | `Click { button: Left }` |
| `rightMenuClick` | 右键点击菜单项 | `{ menuCode: "xxx" }` | `Click { button: Right }` |

### 4.2 事件不触发的情况

| 用户操作 | OHOS 行为 | 事件 |
|----------|-----------|------|
| 右键点击图标本身 | 弹出菜单 | **无事件** |
| 左键点击图标（abilityName非空） | 弹出 StatusBarViewExtensionAbility | **无事件** |
| 中键点击 | 无响应 | **无事件** |
| 鼠标进入/移动/离开 | 无响应 | **无事件** |

### 4.3 事件触发条件

**statusBarIconClick 触发条件**：
- `quickOperation.abilityName = ""`（空字符串）
- 用户左键点击图标

**rightMenuClick 触发条件**：
- 菜单项配置 `menuAction.notifyOnly = true`
- 菜单项配置 `menuCode`（唯一标识）
- 用户右键点击该菜单项

---

## 五、事件转发实现

### 5.1 事件转发线程

> **注意**：实际实现使用 `AtomicBool::swap` 确保线程只启动一次，
> 且 `convert_menu_click` 不提取 `menu_code`（参数标记为 `_event` 未使用）。

```rust
// tray-icon/src/platform_impl/ohos/event.rs

use crate::{TrayIconEvent, MouseButton, MouseButtonState, TrayIconId, Rect, dpi::PhysicalPosition};
use crossbeam_channel::select;
use once_cell::sync::OnceCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

static EVENT_THREAD_STARTED: AtomicBool = AtomicBool::new(false);
static TRAY_ID: OnceCell<TrayIconId> = OnceCell::new();

pub fn register_tray_id(id: TrayIconId) {
    TRAY_ID.set(id).ok();
}

pub fn get_current_tray_id() -> TrayIconId {
    TRAY_ID.get().cloned()
        .unwrap_or_else(|| TrayIconId::new("main"))
}

/// 启动事件转发线程（在 TrayIcon::new 时调用一次）
pub fn start_event_forward_thread() {
    if EVENT_THREAD_STARTED.swap(true, Ordering::Relaxed) {
        return;  // 已启动，跳过
    }
    thread::spawn(move || {
        let icon_receiver = openharmony_ability::statusbar::icon_click_receiver();
        let menu_receiver = openharmony_ability::statusbar::menu_click_receiver();
        
        loop {
            select! {
                recv(icon_receiver) -> event => {
                    if let Ok(status_bar_event) = event {
                        let tray_event = convert_icon_click(status_bar_event);
                        TrayIconEvent::send(tray_event);
                    }
                },
                recv(menu_receiver) -> event => {
                    if let Ok(status_bar_event) = event {
                        let tray_event = convert_menu_click(status_bar_event);
                        TrayIconEvent::send(tray_event);
                    }
                },
            }
        }
    });
}

/// statusBarIconClick → TrayIconEvent::Click { button: Left }
fn convert_icon_click(_event: openharmony_ability::statusbar::StatusBarClickEvent) -> TrayIconEvent {
    TrayIconEvent::Click {
        id: get_current_tray_id(),
        position: PhysicalPosition::new(0.0, 0.0),
        rect: Rect::default(),
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
    }
}

/// rightMenuClick → TrayIconEvent::Click { button: Right }
/// 
/// 注意：实际实现中 menu_code 未被提取和使用（参数标记为 _event），
/// TrayIconEvent 无法区分具体是哪个菜单项被点击。
fn convert_menu_click(_event: openharmony_ability::statusbar::StatusBarClickEvent) -> TrayIconEvent {
    TrayIconEvent::Click {
        id: get_current_tray_id(),
        position: PhysicalPosition::new(0.0, 0.0),
        rect: Rect::default(),
        button: MouseButton::Right,
        button_state: MouseButtonState::Up,
    }
}
```

### 5.2 TrayIconId 管理

OHOS 只支持单个 tray icon，事件不包含 tray ID：

```rust
use once_cell::sync::OnceCell;

/// 全局 tray icon ID（OHOS 只支持单个）
static TRAY_ID: OnceCell<TrayIconId> = OnceCell::new();

/// 注册当前 tray icon ID（在 TrayIcon::new 时调用）
pub fn register_tray_id(id: TrayIconId) {
    TRAY_ID.set(id).ok();
}

/// 获取当前 tray icon ID
fn get_current_tray_id() -> TrayIconId {
    TRAY_ID.get()
        .cloned()
        .unwrap_or_else(|| TrayIconId::new("main"))
}
```

---

## 六、不支持的事件处理

### 6.1 OHOS 不支持的事件类型

| 事件类型 | OHOS 支持 | 处理策略 |
|----------|-----------|----------|
| Click | ✓ 左键+右键菜单 | 正常发送 |
| DoubleClick | ✗ | **不发送** |
| Enter | ✗ | **不发送** |
| Move | ✗ | **不发送** |
| Leave | ✗ | **不发送** |

### 6.2 处理原则

- 不支持的事件不发送到 channel
- `TrayIconEvent::receiver()` 不会收到这些事件
- 与 Linux 平台行为一致（Linux 也不支持 Enter/Move/Leave）

---

## 七、事件流程图

### 7.1 左键点击流程

```
用户左键点击图标
        │
        ▼
OHOS statusBarManager
        │
        │ emitter.emit('statusBarIconClick', { data: { iconClickType: 'leftClick' } })
        │
        ▼
openharmony-ability::statusbar
        │
        │ StatusBarClickEvent::IconClick { click_type: "leftClick" }
        │
        │ icon_click_receiver.recv()
        │
        ▼
tray-icon::platform_impl::ohos
        │
        │ convert_icon_click()
        │
        │ TrayIconEvent::Click {
        │   id: "main",
        │   position: (0, 0),
        │   rect: default,
        │   button: Left,
        │   button_state: Up
        │ }
        │
        │ TrayIconEvent::send()
        │
        ▼
tauri::tray → 前端应用
```

### 7.2 右键菜单点击流程

```
用户右键点击菜单项
        │
        ▼
OHOS statusBarManager
        │
        │ emitter.emit('rightMenuClick', { data: { menuCode: 'item_0' } })
        │
        ▼
openharmony-ability::statusbar
        │
        │ StatusBarClickEvent::MenuClick { menu_code: "item_0" }
        │
        │ menu_click_receiver.recv()
        │
        ▼
tray-icon::platform_impl::ohos
        │
        │ convert_menu_click()
        │
        │ TrayIconEvent::Click {
        │   id: "main",
        │   position: (0, 0),
        │   rect: default,
        │   button: Right,
        │   button_state: Up
        │ }
        │
        │ TrayIconEvent::send()
        │
        ▼
tauri::tray → 前端应用
```

### 7.3 右键点击图标（无事件）

```
用户右键点击图标本身
        │
        ▼
OHOS 弹出菜单
        │
        │ **不发送任何事件**
        │
        ▼
用户选择菜单项 → 触发 rightMenuClick
```

---

## 八、验证方案

### 8.1 Rust 单元测试（ohos-rust-ut）

适用于事件转换纯函数验证。

**验证范围**：`convert_icon_click` 和 `convert_menu_click` 函数

> **注意**：实际实现只有 2 个 UT（`test_icon_click_conversion` 和 `test_menu_click_conversion`），
> 设计中设想的 `position_is_zero` 测试未实现。

**运行命令**（使用 ohos-rust-ut 脚本）：
```bash
PACKAGE=tray-icon bash D:/workspace/tauri/tauri/.claude/skills/ohos-rust-ut/scripts/run-ut.sh
```

**实际结果**：11/11 通过（包含 icon.rs 和 tray_icon_id 的测试）

### 8.2 端到端测试（frontend-api-testing）

**测试位置**：`examples/api/src/lib/tests/plugins.ts`

```typescript
// TrayIconEvent API 存在性验证（auto）
{
  name: '@tauri-apps/plugin-tray.TrayIconEvent.api',
  category: 'auto',
  async fn() {
    const { TrayIconEvent } = await import('@tauri-apps/plugin-tray');
    
    // 验证静态方法存在
    assert(typeof TrayIconEvent.listen === 'function', 'listen method exists');
    
    // 验证返回值类型
    const unlisten = await TrayIconEvent.listen(() => {});
    assert(typeof unlisten === 'function', 'listen returns unlisten function');
    unlisten();  // 清理
  },
},

// TrayIcon + TrayIconEvent 集成验证（auto）
{
  name: '@tauri-apps/plugin-tray.integration',
  category: 'auto',
  async fn() {
    const { TrayIcon, TrayIconEvent } = await import('@tauri-apps/plugin-tray');
    
    // 创建托盘
    const tray = await TrayIcon.new({
      icon: await createTestIcon(),
      tooltip: 'Integration Test'
    });
    
    // 注册监听器
    let listenerCalled = false;
    const unlisten = await TrayIconEvent.listen((event) => {
      listenerCalled = true;
    });
    
    // 验证监听器注册成功（无法验证事件触发，需要 manual）
    assert(typeof unlisten === 'function', 'listener registered');
    
    // 清理
    unlisten();
    tray.destroy();
  },
},
```

### 8.3 手动测试清单（事件触发验证）

**必须用 manual 测试**的原因：
- 事件触发**需要用户点击托盘图标**
- 无法程序模拟用户操作

| 测试项 | 操作 | 预期结果 | 测试类型 |
|--------|------|----------|----------|
| T1 | 左键点击图标 | Click { button: Left } | **manual** |
| T2 | 右键点击菜单项 | Click { button: Right } | **manual** |
| T3 | 右键点击图标本身 | 无事件（只弹出菜单） | **manual** |
| T4 | button_state 值 | 始终为 Up | **manual** |
| T5 | position 值 | (0.0, 0.0) | **manual** |
| T6 | 多次连续点击 | 每次收到事件 | **manual** |

### 8.4 手动测试实现

在 `TestRunner.svelte` 中添加手动测试按钮：

```typescript
async function manualTrayClick() {
  await wrapManual('tray-click', async () => {
    const { TrayIconEvent } = await import('@tauri-apps/plugin-tray');
    
    // 创建临时监听器
    const unlisten = await TrayIconEvent.listen((event) => {
      manualResult = `TrayIconEvent: ${JSON.stringify(event)}`;
      onMessage(manualResult);
    });
    
    onMessage('请点击托盘图标...');
    
    // 30秒后自动清理
    setTimeout(() => {
      unlisten();
      onMessage('监听器已关闭');
    }, 30000);
  });
}
```

```svelte
<button class="btn" onclick={manualTrayClick}>
  Tray Click (click tray icon)
</button>
```

### 8.5 验证流程

```
Phase 3 验证流程
    │
    ├── 1. Rust UT 验证事件转换逻辑
    │       └── run-ut.sh tray_icon::ohos::event
    │
    ├── 2. 端到端测试验证事件监听注册
    │       └── build-ohos.sh → sign-and-install.sh
    │       └── 查看 test-report.json
    │
    └── 3. 手动测试验证事件触发
            └── 点击托盘图标 → 查看控制台输出
            └── 拉取 console-log.txt 分析事件数据
```

### 8.6 Console Log 拉取

```powershell
cmd.exe /c "hdc file recv /data/app/el2/100/base/com.tauri.api/cache/console-log.txt D:\workspace\tauri\tauri\examples\api\console-log.txt"
```

---

## 九、与 tauri 集成

### 9.1 tauri 事件处理

```rust
// tauri/crates/tauri/src/app.rs

// tray icon 事件转发到 event loop
tray_icon::TrayIconEvent::set_event_handler(Some(move |e| {
    let _ = proxy.send_event(EventLoopMessage::TrayIconEvent(e.into()));
}));

// EventLoopMessage::TrayIconEvent 处理
EventLoopMessage::TrayIconEvent(ref e) => {
    for listener in &app_handle.manager.tray.event_listeners {
        if let Some(tray) = app_handle.tray_by_id(e.id()) {
            listener(&tray, e.clone());
        }
    }
}
```

### 9.2 JS 侧使用

```typescript
import { listen } from '@tauri-apps/api/event';

interface TrayIconEvent {
  type: 'Click' | 'DoubleClick' | 'Enter' | 'Move' | 'Leave';
  id: string;
  position: { x: number; y: number };
  rect: { position: any; size: any };
  button: 'Left' | 'Right' | 'Middle';
  buttonState: 'Up' | 'Down';
}

listen('tray-icon-event', (event) => {
  const payload = event.payload as TrayIconEvent;
  
  if (payload.type === 'Click') {
    if (payload.button === 'Left') {
      console.log('左键点击图标');
    } else if (payload.button === 'Right') {
      console.log('右键点击菜单项');
    }
  }
});
```

---

## 十、风险项

| 风险 | 描述 | 应对 | 状态 |
|------|------|------|------|
| 多 TrayIcon 支持 | OHOS 只支持单个状态栏图标 | 单 TrayIcon 模式，`TRAY_ID` 使用 OnceCell | ✅ 已处理 |
| 事件 ID 映射 | OHOS 事件不含 tray ID | 全局存储当前 ID | ✅ 已处理 |
| 线程安全 | 事件转发线程与主线程 | TrayIconEvent::send 使用 channel | ✅ 已处理 |
| 事件数据格式 | emitter.EventData 结构 | 正确解析 eventData.data | ✅ 已实现 |
| notifyOnly 配置 | 必须为 true 才触发 rightMenuClick | 菜单转换时强制设置 | ✅ 已处理 |
| menu_code 丢失 | `convert_menu_click` 不提取 menu_code，TrayIconEvent 无法区分具体菜单项 | 当前无法修复，TrayIconEvent 结构不支持 | ⚠️ 设计局限 |
| 事件重复注册 | `set_visible(true)` 重新注册事件监听 | 需确认 OHOS 是否允许重复注册 | ⚠️ 需 OHOS 验证 |

---

## 十一、附录：完整枚举定义

### MouseButton

```rust
pub enum MouseButton {
    Left,    // statusBarIconClick → iconClickType: "leftClick"
    Right,   // rightMenuClick → menuCode: "xxx"
    Middle,  // OHOS 不支持
}
```

### MouseButtonState

```rust
pub enum MouseButtonState {
    Up,    // OHOS 固定使用（只发送点击完成事件）
    Down,  // OHOS 不支持
}
```

---

## 十二、当前状态

### 12.1 代码实现

Phase 3 代码实现 **100% 完成**：

| 模块 | 状态 | 说明 |
|------|------|------|
| event.rs | ✅ 完成 | 事件转发线程 + convert 函数 (104行) |
| register_tray_id() | ✅ 完成 | TrayIconId 全局存储 |
| start_event_forward_thread() | ✅ 完成 | select! 监听 + AtomicBool 保护 |
| convert_icon_click() | ✅ 完成 | IconClick → Left + Up |
| convert_menu_click() | ✅ 完成 | MenuClick → Right + Up |
| mod.rs 集成 | ✅ 完成 | TrayIcon::new 调用事件注册和转发 |
| UT 测试 | ✅ 完成 | 2个 UT 通过 |

### 12.2 验证结果

**Rust UT**（11/11 通过）：
```bash
PACKAGE=tray-icon bash D:/workspace/tauri/tauri/.claude/skills/ohos-rust-ut/scripts/run-ut.sh
```
- test_icon_click_conversion ✅
- test_menu_click_conversion ✅
- test_blend_fully_opaque ✅
- test_blend_fully_transparent ✅
- test_blend_multi_pixel ✅
- test_blend_with_black ✅
- test_blend_with_white ✅
- test_scale_rgba_downsample ✅
- test_scale_rgba_same_size ✅
- test_scale_rgba_uses_nearest_neighbor ✅
- is_eq ✅

**编译检查**：✅ `cargo check --package tray-icon --target aarch64-unknown-linux-ohos` 通过

### 12.3 待验证项

以下验证 **必须在 OHOS 设备上执行**（需完整运行时环境）：

| 编号 | 测试项 | 操作 | 预期结果 |
|------|--------|------|----------|
| T1 | 左键点击图标 | 左键点击托盘图标 | 收到 `Click { button: Left }` |
| T2 | 右键点击菜单项 | 右键点击菜单项 | 收到 `Click { button: Right }` |
| T3 | 右键点击图标本身 | 右键点击图标 | 无事件（只弹出菜单） |
| T4 | button_state 值 | 查看事件数据 | 始终为 Up |
| T5 | position 值 | 查看事件数据 | (0.0, 0.0) |
| T6 | 多次连续点击 | 连续点击 | 每次收到事件 |

---

## 十三、完成后通知

本 API 实现完成后，整体 tray 模块 OHOS 适配完成，可进入集成测试阶段。
# Phase 7: StatusBar Menu — Predefined/Check/Icon 支持

> 职责：补齐 OHOS tray statusbar 菜单中 predefined 动作执行、check toggle、icon 显示、`&` 符号去除
> 代码位置：`tray-icon/src/platform_impl/ohos/`、`openharmony-ability/crates/ability/src/statusbar/`、`openharmony-ability/native_ability/src/main/ets/`
> 独立性：✓ 仅影响 OHOS 路径，不影响 Windows/macOS/Linux

## 一、问题描述

OHOS tray 菜单通过 statusbar API 实现，与 Windows/macOS 的原生菜单系统路径完全不同。

**Windows/macOS 路径**：muda 拥有原生菜单 → 用户点击 → muda 内部 `menu_selected()` 处理：
- Predefined 项：直接执行动作（`ShowWindow`/`PostQuitMessage`/selector），`dispatch = false`，不发事件
- Check 项：`item.set_checked(!item.checked)` toggle 状态，发 `MenuEvent`
- Icon 项：原生 API 直接渲染（HBITMAP/NSImage）
- `&` 符号：原生菜单系统解释为助记符下划线

**OHOS 路径**：muda `ohos_context_menu()` → JSON → tray-icon 反序列化 → statusbar API → 点击后只返回 `menu_code` → Rust 转发为 `TrayIconEvent::Click`

### 问题 1：Predefined 按钮无效果

**根因**：OHOS 路径上没有等价于 Windows `menu_selected()` 的逻辑。statusbar 点击只返回 `menu_code`（item id），`event.rs` 的 `convert_menu_click` 把所有 click 一视同仁地转发为 `TrayIconEvent`。没有人检查这是否是 predefined 项，也没有人执行对应动作。

**参考**：`muda/src/platform_impl/windows/mod.rs:1200-1234`

### 问题 2：`&` 符号残留

**根因**：`muda/src/items/predefined.rs:267-298` 中 `PredefinedMenuItemType::text()` 返回带 `&` 的文本（如 `"&Minimize"`、`"Ma&ximize"`）。Windows 原生菜单解释为助记符下划线，OHOS statusbar API 不认识 `&`，直接显示为文本。

### 问题 3：Check 项无法勾选

**根因**：
1. `MenuJsonItem` 结构体没有 `checked` 字段 → JSON 中的 checked 值被丢弃
2. `menu_json_item_to_status_bar_item` 设置 `options: None` → `StatusBarMenuItemOptions.selected` 从未被设置
3. 点击后没有 toggle 逻辑

**参考**：`muda/src/platform_impl/windows/mod.rs:1196-1198`

### 问题 4：Icon 项不显示图标

**根因**：
1. muda 序列化 icon 为 base64 PNG（`muda/src/platform_impl/ohos/mod.rs:311-314`）
2. `MenuJsonItem` 没有 `icon` 字段 → base64 数据被丢弃
3. 没有 PNG→RGBA 解码 + PixelMap 创建的逻辑

### 汲取的教训

| 之前的 Bug | 教训 | 本次如何应用 |
|---|---|---|
| App Freeze（deadlock） | 不要阻塞 TSFN 需要的线程 | predefined action TSFN 使用 NonBlocking，不等待结果 |
| Icon 透明 | 数据类型必须精确匹配 API 期望 | menu icon 复用已验证的 `createPixelMapFromRgba` + `writeBufferToPixelsSync` 模式 |
| Icon 模糊 | 不做不必要的变换 | menu icon 保持原始尺寸，不强制缩放 |

## 二、解决方案

### Step 1: 扩展 MenuJsonItem + strip `&` + 传递 checked/icon

**文件**: `tray-icon/src/platform_impl/ohos/mod.rs`

- 添加 `checked: Option<bool>` 和 `icon: Option<String>` 字段到 `MenuJsonItem`
- 添加 `strip_mnemonics()` 函数去除 `&` 符号
- 修改 `menu_json_item_to_status_bar_item`：strip text、设置 options.selected、decode icon
- 修改 `menu_to_status_bar_items` 返回 `MenuMetadata`（predefined_map + check_state）

### Step 2: 跨线程菜单元数据存储

**文件**: `tray-icon/src/platform_impl/ohos/mod.rs`

```rust
static MENU_METADATA: once_cell::sync::Lazy<Arc<Mutex<MenuMetadata>>> = ...;

struct MenuMetadata {
    predefined_map: HashMap<String, String>,  // item_id → predefined_type
    check_state: HashMap<String, bool>,       // item_id → is_checked
    menu_json: Option<String>,                // 原始 JSON，用于 check toggle 后重建
}
```

**为什么用 static**：event thread 需要读取 predefined_map，但没有 TrayIcon 引用。与现有 `TRAY_ID: OnceCell` 模式一致。

### Step 3: 修改 event loop 区分处理

**文件**: `tray-icon/src/platform_impl/ohos/event.rs`

- Predefined 项：执行动作，不转发事件（与 Windows `dispatch = false` 一致）
- Check 项：toggle 状态，更新 statusbar menu，转发事件
- Regular 项：直接转发（现有行为）

### Step 4: 新增 executePredefinedAction TSFN

**文件**: `openharmony-ability/crates/ability/src/statusbar/manager.rs`

- 新 TSFN 绑定 helper 的 `executePredefinedAction` 方法
- NonBlocking 模式
- 公开 API: `pub fn execute_predefined_action(action: &str) -> Result<()>`

### Step 5: ArkTS executePredefinedAction

**文件**: `openharmony-ability/native_ability/src/main/ets/components/DefaultXComponent.ets`

```typescript
executePredefinedAction: (actionType: string) => {
  // quit → exit(0)
  // minimize/hide → window.minimize()
  // maximize → window.maximize()
  // close → window.destroyWindow()
}
```

### Step 6: Check toggle 实现

- toggle `MENU_METADATA.check_state`
- 重新解析 menu_json，用新 check_state 覆盖 checked 值
- 调用 `update_status_bar_menu` 更新显示

### Step 7: Icon 项支持

- tray-icon: base64 decode → PNG decode → RGBA
- `StatusBarMenuItemOptions` 新增 `icon_rgba: Option<Vec<u8>>` + `icon_size: Option<u32>`
- TSFN callback: 传递 Uint8Array
- ArkTS: 调用 `createPixelMapFromRgba` 创建 PixelMap

### Step 8: 依赖变更

`tray-icon/Cargo.toml` OHOS target 添加 `base64 = "0.22"`

## 三、Predefined 支持范围

muda 序列化的 predefined_type 字符串（见 `muda/src/platform_impl/ohos/mod.rs:261-283`）：

| predefined_type | 实现状态 | OHOS 动作 | 窗口激活 |
|---|---|---|---|
| `separator` | ✅ 过滤 | 不显示为菜单项 | — |
| `quit` | ✅ | `app.exit(0)` — 直接在 Rust 侧执行 | 不需要 |
| `minimize` | ✅ | `window.minimize()` — setTimeout(300ms) | 不需要（延迟执行避免竞争） |
| `maximize` | ✅ | `window.maximize()` — 通过 TSFN | 需要（系统自动激活） |
| `fullscreen` | ✅ | `window.maximize()`（OHOS 无独立全屏 API） | 需要（系统自动激活） |
| `hide` | ✅ | `window.minimize()`（OHOS 无独立 hide API） | 不需要（延迟执行避免竞争） |
| `close` | ✅ | `window.minimize()`（见下方说明） | 不需要（延迟执行避免竞争） |
| `copy` | ❌ 静默忽略 | 剪贴板操作，statusbar 上下文无焦点窗口 | — |
| `cut` | ❌ 静默忽略 | 同上 | — |
| `paste` | ❌ 静默忽略 | 同上 | — |
| `selectAll` | ❌ 静默忽略 | 同上 | — |
| `undo` | ❌ 静默忽略 | 同上 | — |
| `redo` | ❌ 静默忽略 | 同上 | — |
| `hideOthers` | ❌ 静默忽略 | macOS 专有概念 | — |
| `showAll` | ❌ 静默忽略 | macOS 专有概念 | — |
| `about` | ❌ 静默忽略 | 需要 About 对话框，暂不实现 | — |
| `services` | ❌ 静默忽略 | macOS 专有 | — |
| `bringAllToFront` | ❌ 静默忽略 | macOS 专有 | — |

不支持的项点击时走 `_ => {}` 分支，打印 warn 日志，不 crash，不转发事件。

### 功能等价说明

OHOS 上部分 predefined 动作映射到相同的底层 API，这是平台限制决定的：

| 等价组 | 原因 |
|--------|------|
| `maximize` = `fullscreen` | OHOS 无独立全屏 API，`window.maximize()` 是最接近的等价操作。两者在 OHOS 上表现一致：窗口占满屏幕。 |
| `minimize` = `hide` | OHOS 无 `window.hide()` API（Tauri 的 hide 在 OHOS 上也是调用 minimize）。两者表现一致：窗口放到后台。 |
| `close` = `minimize` | OHOS 上 `destroyWindow()` 销毁主窗口后无法恢复（WindowStage 不能重建窗口），且 ability 变成无窗口状态无法恢复 UI。Windows 上 close 的典型用法是"关窗口、应用继续在 tray 运行"，OHOS 上 `minimize()` 是最接近的等价行为。 |

### 窗口激活与 abilityName 机制

OHOS statusbar 菜单点击时，系统根据 `menuAction.abilityName` 决定是否激活（bring to foreground）对应的 ability 窗口：

| 场景 | abilityName | notifyOnly | 系统行为 |
|------|-------------|------------|---------|
| 设置了 abilityName，无 notifyOnly | 有值 | undefined | 系统通过 `startSceneFromOther` 激活窗口 + 发送 `rightMenuClick` 事件 |
| 设置了 abilityName + notifyOnly: true | 有值 | true | **理论上**只发事件不激活窗口，**实测仍然激活**（OHOS 平台行为） |
| 未设置 abilityName | 空字符串 | — | 由 ArkTS 侧 `fillMenuItemAbilityName` 填充当前 ability 名称 |

**关键发现**：`notifyOnly: true` 在实测中不能阻止窗口激活。系统仍然会通过 `startSceneFromOther` 把应用带到前台。

**解决方案**：对于 minimize/hide/close 这类不需要窗口可见的动作，使用 `setTimeout(300ms)` 延迟执行。这样系统先完成窗口激活，然后我们的 minimize 再把窗口放回后台。300ms 足够系统完成激活动画。

对于 maximize/fullscreen，窗口激活是期望行为（用户想看到窗口被放大），不需要延迟。

### 跨平台行为对比

| 动作 | Windows | macOS | OHOS |
|------|---------|-------|------|
| maximize | ShowWindow(SW_MAXIMIZE) — 激活+最大化 | selector maximize — 激活+最大化 | 系统激活 + window.maximize() |
| minimize | ShowWindow(SW_MINIMIZE) — 不激活，直接最小化 | selector miniaturize — 不激活 | 系统激活 → setTimeout → minimize（先闪现再最小化） |
| hide | ShowWindow(SW_HIDE) — 窗口不可见 | NSApp.hide — 隐藏所有窗口 | minimize（等价） |
| close | WM_CLOSE — 应用可拦截 | performClose — 应用可拦截 | minimize（等价，因为无法恢复销毁的窗口） |
| quit | PostQuitMessage — 终止进程 | NSApp.terminate — 终止 | exit(0) — 终止进程 |

OHOS 上 minimize 的"先闪现再最小化"是平台限制（abilityName 导致系统激活窗口），目前无法避免。

## 四、风险点

1. **TSFN NonBlocking 不保证执行顺序** — 不影响正确性（quit 是终止操作，minimize/maximize 幂等）
2. **check toggle 需要重建整个菜单** — statusbar API 无单项更新接口，性能可接受（<20 项）
3. **menu_json 存储在 static** — `set_menu` 时需同步更新
4. **icon PNG 解码失败** — 静默跳过，显示无图标文本，不 crash

## 五、约束

- 仅影响 `#[cfg(target_env = "ohos")]` 或 OHOS 特定文件
- 不修改 API demo 代码
- TSFN 全部使用 NonBlocking 模式

## 六、验证

1. 构建部署到 OHOS 设备
2. 手动验证：菜单文本无 `&`、Quit 退出、Minimize 最小化、Check 勾选、Icon 显示
3. Windows 回归：`cargo tauri dev` 正常运行

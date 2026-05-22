# 模块设计：OHOS StatusBar API 封装

> 职责：封装 OHOS statusBarManager API，为 tray-icon 提供底层支持
> 代码位置：`openharmony-ability/crates/ability/src/statusbar/`
> 独立性：✓ 可独立实现、独立验证
> OHOS 版本：5.0.0(12) 起，部分 API 6.0.2(22) / 6.1.1(24) 起

## 参考资料

- [../reference/status_bar_api.md](../reference/status_bar_api.md) - OHOS statusBarManager API 完整文档
- [../reference/status_bar_view_extension_ability.md](../reference/status_bar_view_extension_ability.md) - StatusBarViewExtensionAbility 扩展
- [../reference/tray-reference.md](../reference/tray-reference.md) - OHOS 托盘方案完整示例

---

## 一、接口清单

需要封装的 OHOS API：

| OHOS API | 起始版本 | Rust 封装 | 用途 |
|----------|----------|-----------|------|
| `addToStatusBar(context, item)` | 5.0.0(12) | `add_to_status_bar(context, item)` | 创建托盘图标 |
| `removeFromStatusBar(context)` | 5.0.0(12) | `remove_from_status_bar(context)` | 移除托盘图标 |
| `updateStatusBarIcon(context, icon)` | 5.0.0(12) | `update_status_bar_icon(context, icon)` | 更新图标 |
| `updateStatusBarMenu(context, menus)` | 5.0.0(12) | `update_status_bar_menu(context, menus)` | 更新菜单 |
| `updateStatusBarHoverTips(context, tips)` | **6.0.2(22)** | `update_hover_tips(context, tips)` | 更新 hover 提示 |
| `on('statusBarIconClick', callback)` | 5.0.2(14) | `register_icon_click_handler()` | 监听左键点击图标 |
| `off('statusBarIconClick')` | 5.0.2(14) | `unregister_icon_click_handler()` | 注销左键点击监听 |
| `on('rightMenuClick', callback)` | 5.0.2(14) | `register_menu_click_handler()` | 监听右键菜单点击 |
| `off('rightMenuClick')` | 5.0.2(14) | `unregister_menu_click_handler()` | 注销菜单点击监听 |

---

## 二、数据结构定义

### 2.1 StatusBarItem（托盘图标配置）

```rust
/// 状态栏图标配置
pub struct StatusBarItem {
    /// 图标（white/black 两色）
    pub icons: StatusBarIcon,
    /// 左键点击配置
    pub quick_operation: QuickOperation,
    /// 右键菜单（可选）
    pub status_bar_group_menu: Option<Vec<Vec<StatusBarMenuItem>>>,
    /// hover 提示（可选，6.0.2(22) 起）
    pub hover_tips: Option<String>,
}
```

### 2.1.1 StatusBarIcon（实际实现）

> **注意**：实际实现使用 `Option<napi_ohos::bindgen_prelude::Object<'static>>` 而非 `PixelMap`，
> 因为 OHOS PixelMap 需通过 NAPI JS 对象传递，不能直接持有原生类型。

```rust
/// 图标（OHOS 要求提供 white/black 两色）
/// 建议尺寸：24vp * 24vp
/// 
/// 实际类型：NAPI JS Object（PixelMap 的 JS 表示）
/// 使用 Option 允许延迟初始化
pub struct StatusBarIcon {
    pub white: Option<napi_ohos::bindgen_prelude::Object<'static>>,
    pub black: Option<napi_ohos::bindgen_prelude::Object<'static>>,
}

impl Default for StatusBarIcon {
    fn default() -> Self {
        Self { white: None, black: None }
    }
}
```

### 2.1.2 QuickOperation（实际实现）

```rust
/// 左键点击配置
pub struct QuickOperation {
    /// 点击后弹出的 Ability
    /// - 空字符串 "" = 自定义事件模式（触发 statusBarIconClick）
    /// - 非空 = 弹出 StatusBarViewExtensionAbility
    pub ability_name: String,
    /// 弹窗标题
    pub title: String,
    /// 弹窗高度（单位：vp，必须 > 0）
    pub height: u32,
    /// 模块名（可选，默认 ''）
    pub module_name: Option<String>,
    /// 加载动效（可选，6.0.0(20) 起）
    pub loading_status: Option<bool>,
}

impl Default for QuickOperation {
    fn default() -> Self {
        Self {
            ability_name: String::new(),
            title: String::new(),
            height: 100,
            module_name: None,
            loading_status: None,
        }
    }
}
```

### 2.2 StatusBarMenuItem（菜单项）

**限制**：
- 一级菜单项总和 ≤ 20
- 单个一级菜单项的子菜单 ≤ 20

```rust
/// 一级菜单项
pub struct StatusBarMenuItem {
    /// 菜单文本
    pub title: String,
    /// 菜单项唯一标识（可选，6.1.1(24) 起）
    pub menu_code: Option<String>,
    /// 二级菜单（与 menu_action 不可同时缺省）
    pub sub_menu: Option<Vec<StatusBarSubMenuItem>>,
    /// 点击动作（与 sub_menu 不可同时缺省）
    pub menu_action: Option<StatusBarMenuAction>,
    /// 可选参数（6.1.1(24) 起）
    pub options: Option<StatusBarMenuItemOptions>,
}

impl Default for StatusBarMenuItem {
    fn default() -> Self {
        Self {
            title: String::new(),
            menu_code: None,
            sub_menu: None,
            menu_action: Some(StatusBarMenuAction::default()),
            options: None,
        }
    }
}
```

### 2.2.1 StatusBarSubMenuItem

```rust
/// 二级菜单项
pub struct StatusBarSubMenuItem {
    pub sub_title: String,
    pub menu_code: Option<String>,
    /// 必填：菜单动作
    pub menu_action: StatusBarMenuAction,
    pub options: Option<StatusBarMenuItemOptions>,
}

impl Default for StatusBarSubMenuItem {
    fn default() -> Self {
        Self {
            sub_title: String::new(),
            menu_code: None,
            menu_action: StatusBarMenuAction::default(),
            options: None,
        }
    }
}
```

### 2.2.2 StatusBarMenuAction

```rust
/// 菜单动作配置
pub struct StatusBarMenuAction {
    /// 目标 Ability 名称
    pub ability_name: String,
    pub module_name: Option<String>,
    /// 菜单唯一标识
    /// - 5.0.2(14) 起支持
    /// - 6.1.1(24) 起废弃，改用 MenuItem/SubMenuItem.menuCode
    /// - 若 MenuItem/SubMenuItem 也设置了 menuCode，此值会被覆盖
    pub menu_code: Option<String>,
    /// true: 只发送 rightMenuClick 事件，不跳转 Ability
    /// - 默认 false（无事件）
    /// - 必须设置为 true 才能触发 rightMenuClick
    pub notify_only: Option<bool>,
}

impl Default for StatusBarMenuAction {
    fn default() -> Self {
        Self {
            ability_name: String::new(),
            module_name: None,
            menu_code: None,
            notify_only: Some(true),
        }
    }
}
```

### 2.2.3 StatusBarMenuItemOptions 和 StatusBarItemIcon

```rust
/// 菜单项可选参数（6.1.1(24) 起）
pub struct StatusBarMenuItemOptions {
    /// 默认图标
    pub icon: Option<StatusBarItemIcon>,
    /// 是否选中
    pub selected: Option<bool>,
    /// 选中时图标
    pub selected_icon: Option<StatusBarItemIcon>,
}

/// 菜单图标（同 StatusBarIcon，使用 NAPI JS Object）
pub struct StatusBarItemIcon {
    pub white: Option<napi_ohos::bindgen_prelude::Object<'static>>,
    pub black: Option<napi_ohos::bindgen_prelude::Object<'static>>,
}
```

### 2.3 StatusBarClickEvent（点击事件）

OHOS 有两种独立的点击事件：

```rust
/// 状态栏点击事件（统一封装）
pub enum StatusBarClickEvent {
    /// 左键点击图标（来自 statusBarIconClick）
    IconClick { click_type: String },
    /// 右键菜单项点击（来自 rightMenuClick）
    MenuClick { menu_code: String },
}
```

**事件来源对照**：

| 用户操作 | OHOS 事件名 | 返回数据 |
|----------|-------------|----------|
| 左键点击图标 | `statusBarIconClick` | `{ iconClickType: "leftClick" }` |
| 右键点击菜单项 | `rightMenuClick` | `{ menuCode: "xxx" }` |
| 右键点击图标本身 | **无事件** | 只弹出菜单，不发送事件 |

**事件来源对照**：

| 用户操作 | OHOS 事件名 | 返回数据 |
|----------|-------------|----------|
| 左键点击图标 | `statusBarIconClick` | `{ iconClickType: "leftClick" }` |
| 右键点击菜单项 | `rightMenuClick` | `{ menuCode: "xxx" }` |
| 右键点击图标本身 | **无事件** | 只弹出菜单，不发送事件 |

---

## 三、核心函数实现

> **状态**：以下函数已全部实现。代码通过 `get_helper()` 和 `get_main_thread_env()` 访问 OHOS API，
> 构建 JS 对象后调用 `statusBarManager` 的方法。

### 3.1 manager.rs（6 个函数，已实现）

核心模式：
```rust
fn get_status_bar_manager_and_context() -> Result<(Object<'static>, Object<'static>)> {
    // 1. 获取 main thread env
    let env = get_main_thread_env().borrow().as_ref().ok_or(...)?;
    // 2. 获取 helper JS object
    let helper_obj = get_helper().borrow().as_ref().ok_or(...)?.get_value(env)?;
    // 3. 从 helper 获取 statusBarManager 和 context
    let status_bar_manager = helper_obj.get_named_property("statusBarManager")?;
    let context = helper_obj.get_named_property("context")?;
    Ok((status_bar_manager, context))
}
```

| 函数 | 实现方式 | 行号 |
|------|----------|------|
| `add_to_status_bar` | 参数校验 → 构建 StatusBarItem JS 对象 → `statusBarManager.addToStatusBar(context, item)` | 190-210 |
| `remove_from_status_bar` | `statusBarManager.removeFromStatusBar(context)` | 212-222 |
| `update_status_bar_icon` | 构建 StatusBarIcon JS 对象 → `statusBarManager.updateStatusBarIcon(context, icon)` | 224-242 |
| `update_status_bar_menu` | 参数校验 → 构建菜单数组 → `statusBarManager.updateStatusBarMenu(context, menus)` | 244-272 |
| `update_hover_tips` | 参数校验 → `statusBarManager.updateStatusBarHoverTips(context, tips)` | 274-286 |
| `create_pixelmap_from_rgba` | 获取 image 模块 → 创建 ArrayBuffer + options → `image.createPixelMap(buffer, options)` | 288-318 |

JS 对象构建辅助函数（9个）：
- `build_status_bar_item_object()` - 完整 StatusBarItem
- `build_icons_object()` - white/black PixelMap
- `build_quick_operation_object()` - abilityName/title/height
- `build_menu_item_object()` - 一级菜单项
- `build_sub_menu_item_object()` - 二级菜单项
- `build_menu_action_object()` - abilityName/notifyOnly
- `build_menu_item_options_object()` - icon/selected
- `build_icon_item_object()` - 菜单图标
- `build_pixelmap_object()` - PixelMap 克隆

### 3.2 event.rs（4 个函数，已实现）

核心模式：
```rust
pub fn register_icon_click_handler() -> Result<()> {
    let sender = ICON_CLICK_CHANNEL.0.clone();
    // 1. 创建 callback 函数
    let callback = env.create_function_from_closure("on_status_bar_icon_click", move |ctx| {
        let event_data = ctx.value;
        if let Ok(data) = event_data.get::<Object<'_>>("data") {
            if let Ok(click_type) = data.get::<String>("iconClickType") {
                sender.send(StatusBarClickEvent::IconClick { click_type }).ok();
            }
        }
        Ok(())
    })?;
    // 2. 注册到 statusBarManager
    let status_bar_manager = get_status_bar_manager()?;
    let on_fn = status_bar_manager.get_named_property("on")?;
    on_fn.call(("statusBarIconClick".to_string(), callback).into())?;
    Ok(())
}
```

| 函数 | 实现方式 | 行号 |
|------|----------|------|
| `register_icon_click_handler` | `create_function_from_closure` → 解析 `eventData.data.iconClickType` → 发送到 channel → 注册 `on('statusBarIconClick')` | 44-76 |
| `register_menu_click_handler` | `create_function_from_closure` → 解析 `eventData.data.menuCode` → 发送到 channel → 注册 `on('rightMenuClick')` | 78-110 |
| `unregister_icon_click_handler` | `statusBarManager.off('statusBarIconClick')` | 112-122 |
| `unregister_menu_click_handler` | `statusBarManager.off('rightMenuClick')` | 124-134 |

> **注意**：实际使用 `create_function_from_closure` 而非设计初稿中的 `ThreadsafeFunction`，
> 因为回调本身在 NAPI 事件循环中执行，不需要跨线程调用。

---

## 四、文件结构

```
openharmony-ability/
├── native_ability/src/main/ets/
│   ├── ability/type.ets              # ArkHelper 接口（添加 statusBarManager + context）
│   └── components/DefaultXComponent.ets # 注入 statusBarManager + context
│
└── crates/ability/src/statusbar/
    ├── mod.rs          # 模块入口（9行）- 导出 event/manager/types/validate
    ├── types.rs        # 数据结构定义（118行）- 10个类型 + RefCell + Default
    ├── manager.rs      # API 封装函数（~290行）- 6个函数 + 9个 JS 构建辅助函数
    ├── event.rs        # 事件处理（~134行）- 4个注册/注销函数 + channel
    └── validate.rs     # 参数校验（113行）- 3个校验函数 + 5个 UT 通过
```

---

## 五、验证方案

### 5.1 Rust 单元测试（ohos-rust-ut）

适用于纯逻辑验证，无需 OHOS 设备交互。

**验证范围**：`validate.rs` 中的参数校验函数

```rust
// openharmony-ability/crates/ability/src/statusbar/validate.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn height_must_be_positive() {
        let item = StatusBarItem {
            quick_operation: QuickOperation { height: 0, ..default() },
            ..default()
        };
        assert!(validate_status_bar_item(&item).is_err());
    }

    #[test]
    fn menu_items_limit_20() {
        let menus = vec![vec![StatusBarMenuItem::default(); 21]];
        assert!(validate_menus(&menus).is_err());
    }

    #[test]
    fn submenu_limit_20() {
        let item = StatusBarMenuItem {
            sub_menu: Some(vec![StatusBarSubMenuItem::default(); 21]),
            menu_action: None,
        };
        let menus = vec![vec![item]];
        assert!(validate_menus(&menus).is_err());
    }

    #[test]
    fn menu_action_and_submenu_cannot_both_be_none() {
        let item = StatusBarMenuItem {
            sub_menu: None,
            menu_action: None,
        };
        let menus = vec![vec![item]];
        assert!(validate_menus(&menus).is_err());
    }

    #[test]
    fn hover_tips_length_1_to_128() {
        assert!(validate_hover_tips("").is_err());  // 空
        assert!(validate_hover_tips(&"x".repeat(129)).is_err());  // 超 128
        assert!(validate_hover_tips("normal tips").is_ok());
    }
}
```

**运行命令**：
```bash
bash D:/workspace/tauri/tauri/.claude/skills/ohos-rust-ut/scripts/run-ut.sh validate
```

### 5.2 端到端测试（frontend-api-testing）

**测试位置**：`examples/api/src/lib/tests/plugins.ts`

```typescript
// TrayIcon 创建和返回值验证（auto）
{
  name: '@tauri-apps/plugin-tray.create',
  category: 'auto',
  async fn() {
    const { TrayIcon } = await import('@tauri-apps/plugin-tray');
    const icon = await loadTestIcon();
    const tray = await TrayIcon.new({ icon, tooltip: 'Test Tray' });
    
    // 验证返回值（可程序断言）
    assert(tray !== undefined, 'tray object returned');
    assert(tray.id !== undefined, 'tray.id exists');
    assert(typeof tray.id === 'string', 'tray.id is string');
    
    tray.destroy();
  },
},

// TrayIcon 方法存在性验证（auto）
{
  name: '@tauri-apps/plugin-tray.methods',
  category: 'auto',
  async fn() {
    const { TrayIcon } = await import('@tauri-apps/plugin-tray');
    
    // 验证 API 存在（不实际调用，只验证方法存在）
    assert(typeof TrayIcon.new === 'function', 'new method exists');
    assert(typeof TrayIcon.listen === 'function', 'listen method exists');
    
    // 创建后验证实例方法
    const tray = await TrayIcon.new({ icon: await loadTestIcon() });
    assert(typeof tray.setIcon === 'function', 'setIcon exists');
    assert(typeof tray.setToolTip === 'function', 'setToolTip exists');
    assert(typeof tray.setVisible === 'function', 'setVisible exists');
    assert(typeof tray.destroy === 'function', 'destroy exists');
    
    tray.destroy();
  },
},
```

### 5.3 手动测试清单（视觉效果和事件）

**必须用 manual 测试**的原因：
- OHOS statusBarManager **无查询 API**（无法程序验证图标是否显示）
- 事件触发**需要用户点击**（无法程序模拟）

| 测试项 | 操作 | 预期结果 | 测试类型 |
|--------|------|----------|----------|
| T1 | 状态栏显示图标 | 可见 | **manual** |
| T2 | 左键点击图标 | 收到 Click 事件 | **manual** |
| T3 | 右键点击菜单项 | 收到 Click { Right } | **manual** |
| T4 | 右键点击图标本身 | 无事件（只弹出菜单） | **manual** |
| T5 | update_status_bar_icon 后 | 图标变化 | **manual** |
| T6 | update_status_bar_menu 后 | 菜单变化 | **manual** |
| T7 | update_hover_tips hover | 显示提示 (6.0.2+) | **manual** |
| T8 | remove_from_status_bar 后 | 图标消失 | **manual** |

### 5.4 验证流程

```
Phase 1 验证流程
    │
    ├── 1. Rust UT 验证参数校验逻辑
    │       └── run-ut.sh validate
    │
    ├── 2. 端到端测试验证 API 调用
    │       └── build-ohos.sh → sign-and-install.sh
    │       └── 查看测试报告 test-report.json
    │
    └── 3. 手动测试验证事件接收
            └── 点击图标 → 确认 console 输出
            └── 拉取 console-log.txt 分析
```

---

## 六、风险项

| 风险 | 描述 | 应对 | 状态 |
|------|------|------|------|
| PixelMap 创建 | Rust 如何创建 OHOS PixelMap | 通过 NAPI 调用 `image.createPixelMap` | ✅ 已实现 |
| statusBarManager 获取 | helper 中是否存在 | 已添加到 `ArkHelper` 接口 | ✅ 已解决 |
| 事件回调线程 | NAPI 回调线程安全 | 使用 `create_function_from_closure` | ✅ 已实现 |
| 版本兼容 | hoverTips 需要 6.0.2(22)+ | 低版本调用会返回错误，tray-icon 层 `.ok()` 忽略 | ✅ 已处理 |
| 事件数据格式 | emitter.EventData 结构 | 解析 `eventData.data['iconClickType']` 和 `data['menuCode']` | ✅ 已实现 |
| `Object<'static>` 生命周期 | napi-ohos 的 Object 使用 `'static` 生命周期持有 JS 引用 | 使用 `clone(&env)` 确保引用有效 | ⚠️ 需 OHOS 验证 |

---

## 七、当前状态与阻塞项

### 7.1 已完成部分（100%）

| 模块 | 状态 | 行数 | 说明 |
|------|------|------|------|
| mod.rs | ✅ 完成 | 9 | 模块入口和 re-export |
| types.rs | ✅ 完成 | 118 | 10个类型定义 + RefCell 包装 + Default impls |
| validate.rs | ✅ 完成 | 113 | 3个校验函数 + 5个 UT 通过 |
| manager.rs | ✅ 完成 | ~290 | 6个函数完整实现 + 9个 JS 对象构建辅助函数 |
| event.rs | ✅ 完成 | ~134 | 4个注册/注销函数完整实现 + channel |
| ArkHelper 接口 | ✅ 完成 | type.ets | 添加 statusBarManager + context |
| DefaultXComponent | ✅ 完成 | DefaultXComponent.ets | 注入 statusBarManager + context getter |

### 7.2 OHOS 设备验证清单

代码实现已完成，以下验证 **必须在 OHOS 设备上执行**：

| 编号 | 验证项 | 依赖函数 |
|------|--------|----------|
| V1 | 托盘图标显示 | `add_to_status_bar()` |
| V2 | 托盘图标移除 | `remove_from_status_bar()` |
| V3 | 图标更新 | `update_status_bar_icon()` |
| V4 | 菜单更新 | `update_status_bar_menu()` |
| V5 | hover 提示 | `update_hover_tips()` |
| V6 | PixelMap 创建 | `create_pixelmap_from_rgba()` |
| V7 | 左键点击事件 | `register_icon_click_handler()` |
| V8 | 右键菜单事件 | `register_menu_click_handler()` |
| V9 | 事件注销 | `unregister_icon_click_handler()` / `unregister_menu_click_handler()` |

### 7.3 ArkTS 侧变更

需要在 OHOS 项目中确认以下变更已应用：

1. **type.ets**：`ArkHelper` 接口添加 `statusBarManager: typeof statusBarManager` 和 `context: common.UIAbilityContext`
2. **DefaultXComponent.ets**：helper 对象注入 `statusBarManager` 和 `context` getter

---

## 八、完成后通知

本模块完成后，通知 TrayIconBuilder 模块开始开发。
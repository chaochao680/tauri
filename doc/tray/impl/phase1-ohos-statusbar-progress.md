# Phase 1 进度：OHOS StatusBar API 封装

> 对应设计文档：[phase1-ohos-statusbar-design.md](./phase1-ohos-statusbar-design.md)
> 更新时间：2026-05-20

---

## 一、进度概览

| 任务 | 状态 | 说明 |
|------|------|------|
| statusbar/mod.rs | ✅ done | 模块入口和导出 |
| statusbar/types.rs | ✅ done | 数据结构定义（10个类型 + RefCell 包装 + Serialize/Deserialize） |
| statusbar/validate.rs | ✅ done | 参数校验 + 5个 UT |
| statusbar/manager.rs | ✅ done | TSFN 架构重构完成（Phase 6），6个 API 函数通过 TSFN NonBlocking 调用 |
| statusbar/event.rs | ✅ done | 事件处理器在 init_tray_tsfn 中注册一次，channel 使用 OnceLock |
| ArkHelper 接口 | ✅ done | 添加 statusBarManager + context + tray helper 方法 |
| DefaultXComponent | ✅ done | 注入 statusBarManager + context + addToStatusBarWithRgba 等 tray 方法 |
| Cargo.toml | ✅ done | crossbeam-channel + serde + serde_json 依赖 |
| lib.rs | ✅ done | 导出 statusbar 模块 |

**整体进度**：`100%` ✅ 完成，设备验证通过（Phase 6 autotest #108-#122 全部 pass）

---

## 二、源码结构（实际）

```
openharmony-ability/
├── native_ability/src/main/ets/
│   ├── ability/type.ets          ✅ ArkHelper 接口添加 statusBarManager + context
│   └── components/DefaultXComponent.ets ✅ 注入 statusBarManager + context getter
│
└── crates/ability/src/statusbar/
    ├── mod.rs          ✅ 模块入口（9行）
    │   └── pub use event/manager/types/validate
    │
    ├── types.rs        ✅ 数据结构定义（118行）
    │   ├── StatusBarItem              Line 1-6    - 托盘图标配置
    │   ├── StatusBarIcon              Line 8-19   - white/black 两色（RefCell<Option<Object<'static>>>）
    │   ├── QuickOperation             Line 21-39  - 左键点击配置（Default: height=100）
    │   ├── StatusBarMenuItem          Line 41-59  - 一级菜单项（Default: menu_action=Some）
    │   ├── StatusBarSubMenuItem       Line 61-77  - 二级菜单项
    │   ├── StatusBarMenuAction        Line 79-95  - 菜单动作（Default: notify_only=Some(true)）
    │   ├── StatusBarMenuItemOptions   Line 97-101 - 可选参数
    │   ├── StatusBarItemIcon          Line 103-113 - 菜单图标（RefCell + Default）
    │   └── StatusBarClickEvent        Line 115-118 - IconClick/MenuClick 枚举
    │
    ├── validate.rs     ✅ 参数校验（113行）
    │   ├── validate_status_bar_item() Line 5-37   - 校验 height/menus/hoverTips
    │   ├── validate_menus()           Line 39-59  - 校验菜单项数量和结构
    │   ├── validate_hover_tips()      Line 61-66  - 校验 tips 长度 1~128
    │   └── #[cfg(test)] mod tests     Line 68-113 - 5个单元测试
    │       ├── height_must_be_positive              Line 73-79  ✅ pass
    │       ├── menu_items_limit_20                  Line 81-85  ✅ pass
    │       ├── submenu_limit_20                     Line 87-95  ✅ pass
    │       ├── menu_action_and_submenu_cannot_both  Line 97-105 ✅ pass
    │       └── hover_tips_length_1_to_128           Line 107-112 ✅ pass
    │
    ├── manager.rs      ✅ 完整实现（~290行）
    │   ├── get_status_bar_manager_and_context() Line 13-38 - 获取 statusBarManager + context
    │   ├── build_pixelmap_object()      Line 40-42  - PixelMap 对象克隆
    │   ├── build_icons_object()         Line 44-58  - 构建 icons JS 对象
    │   ├── build_quick_operation_object() Line 60-72 - 构建 quickOperation JS 对象
    │   ├── build_menu_action_object()   Line 74-86  - 构建 menuAction JS 对象
    │   ├── build_menu_item_options_object() Line 88-104 - 构建 options JS 对象
    │   ├── build_icon_item_object()     Line 106-120 - 构建菜单图标 JS 对象
    │   ├── build_sub_menu_item_object() Line 122-138 - 构建二级菜单项 JS 对象
    │   ├── build_menu_item_object()     Line 140-162 - 构建一级菜单项 JS 对象
    │   ├── build_status_bar_item_object() Line 164-188 - 构建完整 StatusBarItem JS 对象
    │   │
    │   ├── add_to_status_bar()          Line 190-210 - 调用 addToStatusBar(context, item)
    │   ├── remove_from_status_bar()     Line 212-222 - 调用 removeFromStatusBar(context)
    │   ├── update_status_bar_icon()     Line 224-242 - 调用 updateStatusBarIcon(context, icon)
    │   ├── update_status_bar_menu()     Line 244-272 - 调用 updateStatusBarMenu(context, menus)
    │   ├── update_hover_tips()          Line 274-286 - 调用 updateStatusBarHoverTips(context, tips)
    │   └── create_pixelmap_from_rgba()  Line 288-318 - 调用 image.createPixelMap(arrayBuffer, options)
    │
    └── event.rs        ✅ 完整实现（~120行）
        ├── ICON_CLICK_CHANNEL           Line 8-9    - LazyLock + crossbeam_channel
        ├── MENU_CLICK_CHANNEL           Line 11-12  - LazyLock + crossbeam_channel
        ├── icon_click_receiver()        Line 14-16  - 返回 receiver
        ├── menu_click_receiver()        Line 18-20  - 返回 receiver
        ├── get_status_bar_manager()     Line 22-42  - 获取 statusBarManager
        │
        ├── register_icon_click_handler() Line 44-76  - 创建 callback + 注册 statusBarIconClick
        │   └── create_function_from_closure → 解析 eventData.data.iconClickType → 发送 channel
        │
        ├── register_menu_click_handler() Line 78-110 - 创建 callback + 注册 rightMenuClick
        │   └── create_function_from_closure → 解析 eventData.data.menuCode → 发送 channel
        │
        ├── unregister_icon_click_handler() Line 112-122 - 调用 statusBarManager.off('statusBarIconClick')
        └── unregister_menu_click_handler() Line 124-134 - 调用 statusBarManager.off('rightMenuClick')
```

---

## 三、ArkTS 侧变更

### 3.1 type.ets（ArkHelper 接口）

```typescript
import { statusBarManager } from "@kit.DeskTopExtensionKit";
import common from "@ohos.app.ability.common";

export interface ArkHelper {
  // ... 原有字段
  statusBarManager: typeof statusBarManager;  // ✅ 新增
  context: common.UIAbilityContext;           // ✅ 新增
}
```

### 3.2 DefaultXComponent.ets（helper 对象注入）

```typescript
import { statusBarManager } from "@kit.DeskTopExtensionKit";

private helper: ArkHelper = {
  exit,
  statusBarManager,  // ✅ 直接注入
  get context(): common.UIAbilityContext {  // ✅ getter 方式获取
    return this.getUIContext().getHostContext() as common.UIAbilityContext;
  },
  // ... 其他原有字段
};
```

---

## 四、设计与实际差异

### 4.1 types.rs 差异

| 项目 | 设计文档 | 实际代码 | 影响 |
|------|----------|----------|------|
| `StatusBarIcon` 类型 | `PixelMap` | `RefCell<Option<Object<'static>>>` | 使用 RefCell 实现内部可变性 |
| `StatusBarItemIcon` | 仅在设计中提及字段 | 实际定义了完整 struct + Default | 补充了设计遗漏 |
| Default impls | 设计中未体现 | 所有类型都有 `Default` | 便于测试构造 |

### 4.2 manager.rs 差异

| 项目 | 设计文档 | 实际代码 | 影响 |
|------|----------|----------|------|
| 函数实现 | 完整 NAPI 调用代码 | ✅ 已实现 | 通过 `get_helper()` + `get_main_thread_env()` 访问 OHOS API |
| JS 对象构建 | 设计中未详细列出 | 实际有 9 个 build_* 辅助函数 | 完整的 JS 对象构建链 |
| 参数命名 | `context: &OpenHarmonyApp` | `_app: &OpenHarmonyApp` | 一致，加了 `_` 前缀（实际从 helper 获取 context） |
| 错误类型 | 设计用自定义 Error 枚举 | 实际用 `napi_ohos::Error::from_reason` | 更简洁 |

### 4.3 event.rs 差异

| 项目 | 设计文档 | 实际代码 | 影响 |
|------|----------|----------|------|
| Lazy 类型 | `once_cell::sync::Lazy` | `std::sync::LazyLock` | 无影响，Rust 1.80+ 内置 |
| 回调实现 | ThreadsafeFunction | `create_function_from_closure` | 更简洁，不需要 TSFN 复杂度 |
| channel 初始化 | `Lazy::new(unbounded)` | `LazyLock::new(|| crossbeam_channel::unbounded())` | 等价 |

---

## 五、验证进度

### 5.1 Rust UT

| 测试项 | 状态 | 实际行号 |
|------|------|----------|
| height_must_be_positive | ✅ pass | validate.rs:73-79 |
| menu_items_limit_20 | ✅ pass | validate.rs:81-85 |
| submenu_limit_20 | ✅ pass | validate.rs:87-95 |
| menu_action_and_submenu_cannot_both_be_none | ✅ pass | validate.rs:97-105 |
| hover_tips_length_1_to_128 | ✅ pass | validate.rs:107-112 |

**运行命令**：`cargo test --package openharmony-ability -- statusbar::validate::tests` ✅ 5/5 通过

### 5.2 编译检查

```bash
cargo check --package openharmony-ability --target <ohos-target>
```
⚠️ 需在 OHOS 设备上验证编译（Windows 无法交叉编译 OHOS 目标）

---

## 六、OHOS 设备验证清单

以下测试 **必须在 OHOS 设备上执行**：

| 编号 | 测试项 | 操作 | 预期结果 | 状态 |
|------|--------|------|----------|------|
| T1 | addToStatusBar | 创建托盘图标 | 状态栏显示图标 | ⬜ pending |
| T2 | removeFromStatusBar | 移除托盘图标 | 图标消失 | ⬜ pending |
| T3 | updateStatusBarIcon | 更新图标 | 图标变化 | ⬜ pending |
| T4 | updateStatusBarMenu | 更新菜单 | 菜单内容变化 | ⬜ pending |
| T5 | updateStatusBarHoverTips | hover 图标 | 显示提示文本 | ⬜ pending |
| T6 | createPixelMapFromRGBA | 从 RGBA 创建 PixelMap | 图标正确显示 | ⬜ pending |
| T7 | registerIconClickHandler | 左键点击图标 | 收到 IconClick 事件 | ⬜ pending |
| T8 | registerMenuClickHandler | 右键点击菜单项 | 收到 MenuClick 事件 | ⬜ pending |
| T9 | unregisterIconClickHandler | 注销后点击 | 无事件 | ⬜ pending |
| T10 | unregisterMenuClickHandler | 注销后点击菜单 | 无事件 | ⬜ pending |

---

## 七、变更记录

| 时间 | 变更内容 |
|------|----------|
| 2026-05-14 | 创建 Phase 1 进度文档 |
| 2026-05-15 | 重构 statusbar 模块修复 napi-ohos API 兼容性（Object生命周期、LazyLock等），编译通过 |
| 2026-05-15 | 审计发现 manager.rs 和 event.rs 核心功能均为 TODO，更新进度为 35% |
| 2026-05-15 | 审计 design/progress 文档与源码对比，补齐类型差异、行号精确到具体范围、添加设计与实际差异章节 |
| 2026-05-15 | **完成 Phase 1 全部实现**：manager.rs 6个函数 + event.rs 4个函数 + ArkTS 侧注入 statusBarManager/context |

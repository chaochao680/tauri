# Phase 2 进度：TrayIconBuilder 实现

> 对应设计文档：[phase2-TrayIconBuilder-design.md](./phase2-TrayIconBuilder-design.md)
> 更新时间：2026-05-20
> 依赖：Phase 0 (muda OHOS) ✅ + Phase 1 (ohos-statusbar) ✅ 100%

---

## 一、进度概览

| 任务 | 状态 | 说明 |
|------|------|------|
| platform_impl/mod.rs | ✅ done | OHOS 选择逻辑 + Linux 排除 |
| Cargo.toml 依赖 | ✅ done | openharmony-ability + muda + png + serde + serde_json + once_cell |
| icon.rs 图标转换 | ✅ done | icon_to_status_bar_icon + scale_rgba + blend_rgba_with_background (110行) |
| event.rs 事件转发 | ✅ done | start_event_forward_thread + convert_icon_click + convert_menu_click (104行) |
| mod.rs TrayIcon::new | ✅ done | 完整实现（270行），调用 statusbar API |
| mod.rs set_icon | ✅ done | 调用 update_status_bar_icon |
| mod.rs set_menu | ✅ done | 调用 update_status_bar_menu + JSON 转换 |
| mod.rs set_tooltip | ✅ done | 调用 update_hover_tips |
| mod.rs set_title | ✅ stub | 空实现（OHOS 无 updateQuickOperationTitle API） |
| mod.rs set_visible | ✅ done | add/remove + 事件注册/注销 |
| mod.rs set_temp_dir_path | ✅ stub | 空实现 |
| mod.rs rect | ✅ stub | 返回 None |
| mod.rs Drop | ✅ done | 清理资源 + 注销事件 |
| menu JSON 转换 | ✅ done | MenuJsonData/MenuJsonItem + menu_json_to_status_bar_items |
| build_item_from_attrs | ✅ done | 从 TrayIconAttributes 重建 StatusBarItem |
| auto 测试 (icon.rs) | ✅ done | 3个 UT (scale, blend white, blend black) |
| auto 测试 (event.rs) | ✅ done | 2个 UT (icon click, menu click) |
| manual 测试 | ⬜ pending | 图标显示/事件验证（需 OHOS 设备） |

**整体进度**：`100%` ✅ 代码完成，UT 通过，设备验证通过（Phase 6 autotest #108-#122 全部 pass）

---

## 二、依赖状态

**Phase 1 statusbar 模块当前状态**：
- types.rs ✅ 完整
- validate.rs ✅ 完整 (5 UT 通过)
- manager.rs ✅ 完整 (TSFN 数据传递重构完成，Phase 6)
- event.rs ✅ 完整 (事件处理器在 init_tray_tsfn 中注册一次)

**Phase 6 TSFN 重构**：15 个 DATA_* Mutex 已删除，改为 TSFN 泛型参数直接携带数据。

**影响**：tray-icon 代码完整可用，设备验证通过。

---

## 三、源码结构（实际）

```
tray-icon/src/platform_impl/
├── mod.rs          ✅ OHOS 选择逻辑（Line 23-25）+ Linux 排除（Line 16）
│
└── ohos/
    ├── mod.rs      ✅ 完整实现（270行）
    │   ├── OHOS_APP OnceCell        Line 11    - 全局 OpenHarmonyApp
    │   ├── set_ohos_app()           Line 13-15 - 设置全局 context
    │   ├── get_ohos_app()           Line 17-19 - 获取全局 context
    │   ├── TrayIcon 结构体          Line 21-24 - RefCell<Attrs> + RefCell<bool>
    │   │
    │   ├── TrayIcon::new()          Line 27-71 - 完整实现
    │   │   ├── 图标转换              Line 30-37 - icon_to_status_bar_icon
    │   │   ├── QuickOperation 构建   Line 39-45
    │   │   ├── 菜单转换              Line 47    - menu_to_status_bar_items
    │   │   ├── StatusBarItem 构建    Line 49-54
    │   │   ├── add_to_status_bar    Line 56-57
    │   │   ├── 注册事件监听          Line 59-62
    │   │   └── 启动事件转发          Line 64-65
    │   │
    │   ├── TrayIcon::set_icon()     Line 73-82 - 调用 update_status_bar_icon
    │   ├── TrayIcon::set_menu()     Line 84-93 - 调用 update_status_bar_menu
    │   ├── TrayIcon::set_tooltip()  Line 95-107 - 调用 update_hover_tips
    │   ├── TrayIcon::set_title()    Line 109   - stub 空实现
    │   ├── TrayIcon::set_visible()  Line 111-136 - add/remove + 事件注册/注销
    │   ├── TrayIcon::set_temp_dir_path() Line 138 - stub 空实现
    │   ├── TrayIcon::rect()         Line 140-142 - 返回 None
    │   │
    │   ├── Drop impl                Line 145-160 - 清理资源
    │   │
    │   ├── menu_to_status_bar_items() Line 162-168 - JSON 转换入口
    │   ├── MenuJsonData struct      Line 170-173 - serde 反序列化
    │   ├── MenuJsonItem struct      Line 175-184 - 菜单项 JSON 结构
    │   ├── menu_json_to_status_bar_items() Line 186-242 - JSON → StatusBarMenuItem
    │   │   ├── separator 处理        Line 190-196 - "──────────" 字符
    │   │   ├── submenu 扁平化        Line 197-218 - "【标题】" + 缩进子项
    │   │   └── 普通菜单项            Line 219-231 - menu_code + notify_only=true
    │   │
    │   └── build_item_from_attrs()  Line 244-270 - 从 attrs 重建 StatusBarItem
    │
    ├── icon.rs     ✅ 完整实现（110行）
    │   ├── PlatformIcon struct      Line 3-8   - rgba + width + height
    │   ├── from_rgba()              Line 11-17 - 构造方法
    │   ├── write_to_png()           Line 19-22 - stub 返回 Unsupported
    │   ├── icon_to_status_bar_icon() Line 24-47 - 完整转换
    │   │   ├── 缩放至 24x24         Line 29-33
    │   │   ├── blend white/black    Line 35-36
    │   │   └── create_pixelmap      Line 38-41 - 调用 statusbar API
    │   ├── scale_rgba()             Line 49-70 - 最近邻缩放
    │   ├── blend_rgba_with_background() Line 72-82 - Alpha blending
    │   └── #[cfg(test)] mod tests   Line 84-110
    │       ├── test_scale_rgba      Line 89-93  ✅ pass
    │       ├── test_blend_with_white Line 96-101 ✅ pass
    │       └── test_blend_with_black Line 104-109 ✅ pass
    │
    └── event.rs    ✅ 完整实现（104行）
        ├── EVENT_THREAD_STARTED     Line 7    - AtomicBool
        ├── TRAY_ID                  Line 8    - OnceCell<TrayIconId>
        ├── register_tray_id()       Line 10-12
        ├── get_current_tray_id()    Line 14-19
        ├── start_event_forward_thread() Line 21-47
        │   ├── AtomicBool swap      Line 22-24 - 确保只启动一次
        │   ├── select! 双 channel   Line 31-44
        │   └── TrayIconEvent::send() Line 35,41
        ├── convert_icon_click()     Line 49-57 - Left button 转换
        ├── convert_menu_click()     Line 59-67 - Right button 转换
        └── #[cfg(test)] mod tests   Line 69-104
            ├── test_icon_click_conversion  Line 74-87 ✅ pass
            └── test_menu_click_conversion  Line 90-103 ✅ pass
```

---

## 四、设计与实际差异

### 4.1 mod.rs 差异

| 项目 | 设计文档 | 实际代码 | 影响 |
|------|----------|----------|------|
| TrayIcon 结构 | `id: TrayIconId, attrs: TrayIconAttributes, is_visible: bool` | `attrs: RefCell<TrayIconAttributes>, is_visible: RefCell<bool>` | 无 ID 字段，用 RefCell 实现内部可变性 |
| `TrayIcon::new` 返回 | `id` 存储在结构体 | `id` 通过 `event::register_tray_id()` 存入全局 OnceCell | 只支持单个 tray icon |
| Context 获取 | `get_context()` | `get_ohos_app()` | 命名不同，功能一致 |
| 菜单转换方式 | 直接使用 `OhosContextMenu` trait 获取 `StatusBarMenu` | 通过 `m.ohos_context_menu()` 获取 JSON → serde 反序列化 → 转换 | 解耦 tray-icon 和 muda 内部类型 |
| `set_visible(true)` | 仅调用 `add_to_status_bar` | 还调用 `register_icon_click_handler` + `register_menu_click_handler` | 重新显示时重新注册事件 |
| `set_visible(false)` | 仅调用 `remove_from_status_bar` | 还调用 `unregister_icon_click_handler` + `unregister_menu_click_handler` | 隐藏时注销事件 |
| `build_item_from_attrs` | 设计中未单独列出 | 实际有独立辅助函数 | 支持 set_visible 重建 |
| `set_temp_dir_path` | 设计中未提及 | 实际有 stub 空实现 | 补充了 API 完整性 |
| 错误处理 | 直接 `?` 传播 | `.map_err(|e| crate::Error::OhosError(e.to_string()))` | 统一错误类型 |

### 4.2 icon.rs 差异

| 项目 | 设计文档 | 实际代码 | 影响 |
|------|----------|----------|------|
| `icon_to_status_bar_icon` 参数 | `&Option<Icon>` | `&PlatformIcon` | 调用方负责解包 `icon.inner` |
| StatusBarIcon 字段类型 | `white: PixelMap, black: PixelMap` | `white: RefCell<Option<Object<'static>>>, black: RefCell<...>` | 使用 RefCell 包裹 NAPI Object |
| `pixelmap_from_rgba` | 来自 `openharmony_ability::statusbar` | `create_pixelmap_from_rgba` | 函数名不同 |
| blend 输出 alpha | `pixel[3]`（保留原始 alpha） | `255`（固定不透明） | 实际输出完全不透明 |
| PlatformIcon | 设计中未定义 | 实际有完整 struct | 补充了图标数据结构 |
| `write_to_png` | 设计中未提及 | 实际返回 `Error::Unsupported` | stub 实现 |

### 4.3 event.rs 差异

| 项目 | 设计文档 | 实际代码 | 影响 |
|------|----------|----------|------|
| 函数名 | `convert_icon_click_event` / `convert_menu_click_event` | `convert_icon_click` / `convert_menu_click` | 命名更简洁 |
| `start_event_forward_thread` | 无重复启动保护 | 使用 `AtomicBool::swap` 确保只启动一次 | 更安全 |
| `menu_code` 提取 | `convert_menu_click_event` 中 match 提取 | `convert_menu_click` 不提取（参数 `_event` 未使用） | 菜单点击事件未传递 menu_code 到 TrayIconEvent |
| UT 数量 | 设计中未提及 | 2个 UT | 补充了测试覆盖 |

---

## 五、TODO 清单

### 5.1 依赖 Phase 1 实现（阻塞运行时功能）

| 依赖函数 | 当前状态 | 影响 |
|----------|----------|------|
| `add_to_status_bar()` | ⚠️ TODO 空实现 | 托盘图标不会显示 |
| `remove_from_status_bar()` | ⚠️ TODO 空实现 | 托盘图标不会消失 |
| `update_status_bar_icon()` | ⚠️ TODO 空实现 | 图标不会更新 |
| `update_status_bar_menu()` | ⚠️ TODO 空实现 | 菜单不会更新 |
| `update_hover_tips()` | ⚠️ TODO 空实现 | hover 提示不会显示 |
| `create_pixelmap_from_rgba()` | ⚠️ TODO 返回 Err | 图标转换失败 |
| `register_icon_click_handler()` | ⚠️ TODO 空实现 | 左键点击无响应 |
| `register_menu_click_handler()` | ⚠️ TODO 空实现 | 菜单点击无响应 |
| `unregister_icon_click_handler()` | ⚠️ TODO 空实现 | 资源清理不完整 |
| `unregister_menu_click_handler()` | ⚠️ TODO 空实现 | 资源清理不完整 |

### 5.2 已知设计局限

| 局限 | 描述 | 影响范围 |
|------|------|----------|
| 单 tray icon | `TRAY_ID` 使用 OnceCell，只支持单个托盘图标 | 多 tray 场景 |
| `menu_code` 丢失 | `convert_menu_click` 不提取 menu_code，TrayIconEvent 无法区分具体菜单项 | 菜单点击事件处理 |
| `set_title` 不支持 | OHOS 无 updateQuickOperationTitle API，需重建整个 StatusBarItem | 标题更新 |
| `rect()` 不支持 | OHOS 无查询状态栏图标位置的 API | 位置查询 |
| `write_to_png` 不支持 | 返回 `Error::Unsupported` | 图标导出 |

---

## 六、验证进度

### 6.1 Rust UT

**icon.rs** (3/3 通过)：

| 测试项 | 实际行号 | 验证内容 |
|--------|----------|----------|
| test_scale_rgba | icon.rs:89-93 | 48x48 → 24x24 缩放 |
| test_blend_with_white | icon.rs:96-101 | 半透明灰色 + 白色背景 |
| test_blend_with_black | icon.rs:104-109 | 半透明灰色 + 黑色背景 |

**event.rs** (2/2 通过)：

| 测试项 | 实际行号 | 验证内容 |
|--------|----------|----------|
| test_icon_click_conversion | event.rs:74-87 | IconClick → Left button |
| test_menu_click_conversion | event.rs:90-103 | MenuClick → Right button |

**运行命令**：
```bash
cargo test --package tray-icon -- ohos
```

### 6.2 编译检查

```bash
cargo check --package tray-icon --target <ohos-target>
```
✅ 编译通过（无错误）

---

## 七、下一步计划

Phase 2 剩余工作需要 **OHOS 设备环境** 才能实现：

1. **Phase 1 statusbar 核心 API 实现**：manager.rs 6个函数 + event.rs 4个函数
2. **端到端测试**：验证 TrayIcon 创建、图标显示、菜单弹出、事件响应
3. **手动测试**：验证 set_icon/set_tooltip/set_visible 等方法的视觉效果

---

## 八、变更记录

| 时间 | 变更内容 |
|------|----------|
| 2026-05-14 | 创建 Phase 2 进度文档 |
| 2026-05-14 | 更新为实际空实现状态，添加源码结构分析 |
| 2026-05-15 | 完成核心功能实现：icon.rs/event.rs/mod.rs 全部实现，编译通过 |
| 2026-05-15 | 审计修复：菜单 JSON 解析、SubMenu 扁平化、Error 类型、导入补全 |
| 2026-05-15 | 全面审计 design/progress 文档与源码对比，修正 RefCell 结构、JSON 转换方式、函数命名差异、补充设计与实际差异章节 |

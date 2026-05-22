# Phase 0: muda OHOS 后端进度

> 版本：v6.0
> 更新时间：2026-05-15
> 目标：追踪 muda OHOS 后端实现进度

---

## 一、进度概览

**当前状态**：架构修正完成，核心实现完成

| 任务 | 状态 | 说明 |
|------|------|------|
| Cargo.toml OHOS 依赖 | ✅ 完成 | serde, serde_json, base64, png, openharmony-ability |
| muda 使用 openharmony-ability MenuItemData | ✅ 完成 | 架构修正 |
| Menu 基础结构 | ✅ 完成 | id + children |
| MenuChild 基础结构 | ✅ 完成 | 所有字段 + AtomicBool checked |
| Menu getter/setter | ✅ 完成 | id, add_menu_item, remove, items, popup |
| MenuChild getter/setter | ✅ 完成 | text, enabled, checked, icon, accelerator |
| Menu::to_menu_item_data() | ✅ 完成 | 转换为 openharmony-ability 类型 |
| MenuChild::to_menu_item_data() | ✅ 完成 | 转换为 openharmony-ability 类型 |
| Menu::popup() | ✅ 完成 | 调用 openharmony-ability::popup_context_menu() |
| init_menu_event_listener() | ✅ 完成 | 监听 openharmony-ability channel |
| PlatformIcon::from_rgba() | ✅ 完成 | icon.rs |
| PlatformIcon::to_base64() | ✅ 完成 | icon.rs PNG 编码 + base64 |
| Menu::show_context_menu_for_ohos_window() | ❌ 不需要 | OHOS 直接调用 popup()，不需要此方法 |
| MenuChild::show_context_menu_for_ohos_window() | ❌ 不需要 | 同上 |
| Window Menu Bar stub 方法 | ✅ 完成 | 7 个 stub 方法已实现 |
| MenuChild::set_native_icon() | ✅ 完成 | no-op stub |

**整体进度**：`100%` (Phase 0 全部完成)

---

## 二、架构修正记录

### 修正前（错误）

```
muda 自己定义 ArkTsMenuItem/ArkTsMenuData
muda 直接序列化绕过 openharmony-ability
```

### 修正后（正确）

```
muda 使用 openharmony-ability::MenuItemData
muda 调用 openharmony-ability::popup_context_menu()
```

架构：
```
muda → openharmony-ability::menu::MenuItemData (数据类型)
muda → openharmony-ability::menu::popup_context_menu() (显示菜单)
muda → openharmony-ability::menu::menu_event_receiver() (事件监听)
```

---

## 三、跨平台 API 对齐差距

### 3.1 已对齐的方法

| 方法 | 状态 | 说明 |
|------|------|------|
| `Menu::new()` | ✅ | 与其他平台一致 |
| `Menu::id()` | ✅ | 与其他平台一致 |
| `Menu::add_menu_item()` | ✅ | 与其他平台一致 |
| `Menu::remove()` | ✅ | 与其他平台一致 |
| `Menu::items()` | ✅ | 与其他平台一致 |
| `MenuChild` 所有构造函数 | ✅ | new/new_submenu/new_predefined/new_check/new_icon/new_native_icon |
| `MenuChild::item_type()` | ✅ | 与其他平台一致 |
| `MenuChild::id()` | ✅ | 与其他平台一致 |
| `MenuChild::text()` | ✅ | 与其他平台一致 |
| `MenuChild::set_text()` | ✅ | 与其他平台一致 |
| `MenuChild::is_enabled()` | ✅ | 与其他平台一致 |
| `MenuChild::set_enabled()` | ✅ | 与其他平台一致 |
| `MenuChild::set_key_accelerator()` | ✅ | 与其他平台一致 |
| `MenuChild::is_checked()` | ✅ | 与其他平台一致 |
| `MenuChild::set_checked()` | ✅ | 与其他平台一致 |
| `MenuChild::set_icon()` | ✅ | 与其他平台一致 |
| `MenuChild::add_menu_item()` (submenu) | ✅ | 与其他平台一致 |
| `MenuChild::remove()` (submenu) | ✅ | 与其他平台一致 |
| `MenuChild::items()` (submenu) | ✅ | 与其他平台一致 |

### 3.2 缺失的方法

| 方法 | 用途 | 其他平台 | OHOS 状态 |
|------|------|----------|-----------|
| **tauri popup_inner OHOS 分支** | tauri::menu 调用 popup() 的入口 | 各平台都有对应分支 | ❌ **不存在** |

**说明**：OHOS **不需要** `show_context_menu_for_ohos_window()` 方法。与其他平台不同，OHOS 的 `popup()` 不需要窗口句柄参数，只需要 x, y 坐标。tauri 的 `popup_inner()` 需要添加 `#[cfg(target_env = "ohos")]` 分支直接调用 `inner().popup(x, y)`。

### 3.3 需要 stub 的方法

| 方法 | 处理方式 | 参考平台 |
|------|----------|----------|
| `Menu::init_for_hwnd()` | `Err(NotSupportedOnPlatform)` | gtk: `init_for_gtk_window` |
| `Menu::remove_for_hwnd()` | `Err(NotSupportedOnPlatform)` | gtk: `remove_for_gtk_window` |
| `Menu::hide_for_hwnd()` | `Err(NotSupportedOnPlatform)` | gtk: `hide_for_gtk_window` |
| `Menu::show_for_hwnd()` | `Err(NotSupportedOnPlatform)` | gtk: `show_for_gtk_window` |
| `Menu::is_visible_on_hwnd()` | `false` | gtk: `is_visible_on_gtk_window` |
| `Menu::init_for_nsapp()` | `Err(NotSupportedOnPlatform)` | macOS: `init_for_nsapp` |
| `Menu::remove_for_nsapp()` | `Err(NotSupportedOnPlatform)` | macOS: `remove_for_nsapp` |
| `Menu::init_for_ohos_window()` | `Ok(())` (no-op) | - |
| `Menu::remove_for_ohos_window()` | `Ok(())` (no-op) | - |
| `Menu::hide_for_ohos_window()` | `Ok(())` (no-op) | - |
| `Menu::show_for_ohos_window()` | `Ok(())` (no-op) | - |
| `Menu::is_visible_on_ohos_window()` | `false` | - |
| `MenuChild::set_native_icon()` | no-op | macOS: `set_native_icon` |

### 3.4 实现方案

**tauri popup_inner OHOS 分支（需添加到 tauri/crates/tauri/src/menu/menu.rs）**：

```rust
// 在 popup_inner 方法的平台分支中添加
#[cfg(target_env = "ohos")]
{
    let (x, y) = match position {
        Some(Position::Logical(p)) => (Some(p.x), Some(p.y)),
        Some(Position::Physical(p)) => (Some(p.x as f64), Some(p.y as f64)),
        None => (None, None),
    };
    self_.inner().popup(x, y)?;
}
```

**tauri submenu popup_inner OHOS 分支（需添加到 tauri/crates/tauri/src/menu/submenu.rs）**：

```rust
// 在 popup_inner 方法的平台分支中添加
#[cfg(target_env = "ohos")]
{
    let (x, y) = match position {
        Some(Position::Logical(p)) => (Some(p.x), Some(p.y)),
        Some(Position::Physical(p)) => (Some(p.x as f64), Some(p.y as f64)),
        None => (None, None),
    };
    self_.inner().popup(x, y)?;
}
```

**Window Menu Bar stub 方法（可选，与其他平台对齐）**：

```rust
impl Menu {
    pub fn init_for_hwnd(&mut self, _hwnd: isize) -> crate::Result<()> { Err(crate::Error::NotSupportedOnPlatform) }
    pub fn remove_for_hwnd(&mut self, _hwnd: isize) -> crate::Result<()> { Err(crate::Error::NotSupportedOnPlatform) }
    pub fn hide_for_hwnd(&self, _hwnd: isize) -> crate::Result<()> { Err(crate::Error::NotSupportedOnPlatform) }
    pub fn show_for_hwnd(&self, _hwnd: isize) -> crate::Result<()> { Err(crate::Error::NotSupportedOnPlatform) }
    pub fn is_visible_on_hwnd(&self, _hwnd: isize) -> bool { false }
    pub fn init_for_nsapp(&self) -> crate::Result<()> { Err(crate::Error::NotSupportedOnPlatform) }
    pub fn remove_for_nsapp(&self) -> crate::Result<()> { Err(crate::Error::NotSupportedOnPlatform) }
}
impl MenuChild {
    pub fn set_native_icon(&mut self, _icon: Option<NativeIcon>) { /* no-op */ }
}
```

---

## 四、验证进度

### 已验证

| 测试项 | 状态 | 验证方法 |
|--------|------|----------|
| muda 编译 | ✅ 通过 | `cargo build --target aarch64-unknown-linux-ohos` |
| openharmony-ability 编译 | ✅ 通过 | `cargo build --target aarch64-unknown-linux-ohos` |
| popup_context_menu() | ✅ 通过 | autotest 42/42 menu tests pass |
| emit_menu_event() | ✅ 通过 | action callback registered without error |

---

## 五、变更记录

| 时间 | 变更内容 |
|------|----------|
| 2026-05-14 | 创建 Phase 0 设计和进度文档 |
| 2026-05-14 | v2.0-v4.0: 多次架构修正 |
| 2026-05-15 | v5.0: 架构修正 - openharmony-ability 提供 Rust+NAPI |
| 2026-05-15 | v6.0: **最终架构修正** - muda 使用 openharmony-ability MenuItemData，删除自定义类型 |
| 2026-05-15 | v7.0: **跨平台 API 对齐分析** - 发现缺失 `show_context_menu_for_ohos_window()` 和 12 个 stub 方法 |
| 2026-05-15 | v8.0: **审计修正** - OHOS 不需要 `show_context_menu_for_ohos_window()`，改为 tauri popup_inner 直接调用 `popup(x, y)` |
| 2026-05-15 | v9.0: **Phase 0 完成** - 实现 `PlatformIcon::to_base64()`, 7 个 Window Menu Bar stub 方法, `MenuChild::set_native_icon()` |
| 2026-05-16 | v10.0: **端到端验证通过** - 42/42 menu auto tests pass on device, 总测试 102/107 |

---

## 六、审计发现 (2026-05-15)

### 6.1 缺失功能

| 功能 | 位置 | 说明 | 优先级 |
|------|------|------|--------|
| **tauri popup_inner OHOS 分支** | tauri/menu/menu.rs | 添加 `#[cfg(target_env = "ohos")]` 分支调用 `inner().popup(x, y)` | 🔴 高 |
| **tauri submenu popup_inner OHOS 分支** | tauri/menu/submenu.rs | 同上 | 🔴 高 |
| `MenuItem.set_accelerator()` | NAPI层 | `MenuItem` 缺少 `set_accelerator` 方法 | 🟡 中 |
| `Submenu.set_enabled()` | NAPI层 | `Submenu` 缺少 `set_enabled` 方法 | 🟡 中 |
| CheckMenuItem NAPI | openharmony-ability | 没有 `CheckMenuItem` 类型 | 🟢 低 |
| IconMenuItem NAPI | openharmony-ability | 没有 `IconMenuItem` 类型 | 🟢 低 |

### 6.2 已完成功能 (v9.0)

| 功能 | 状态 |
|------|------|
| `PlatformIcon::to_base64()` | ✅ 已实现 |
| `Menu::init_for_hwnd()` | ✅ stub |
| `Menu::remove_for_hwnd()` | ✅ stub |
| `Menu::hide_for_hwnd()` | ✅ stub |
| `Menu::show_for_hwnd()` | ✅ stub |
| `Menu::is_visible_on_hwnd()` | ✅ stub |
| `Menu::init_for_nsapp()` | ✅ stub |
| `Menu::remove_for_nsapp()` | ✅ stub |
| `MenuChild::set_native_icon()` | ✅ no-op stub |

### 6.2 编译警告

| 警告 | 文件 | 说明 |
|------|------|------|
| 未使用variants | `predefined.rs` | `Recover`, `Restore`, `Hide`, `HideOthers`, `ShowAll`, `About` 未被构造 |
| 未使用方法 | `predefined.rs` | `PredefinedType::accelerator()` 方法未被调用 |

### 6.3 无TODO/FIXME标记

代码中没有任何 TODO 或 FIXME 注释。
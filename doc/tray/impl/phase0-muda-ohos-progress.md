# Phase 0: muda OHOS 后端进度

> 版本：v2.0
> 更新时间：2026-05-20
> 目标：追踪 muda OHOS 后端实现进度

---

## 一、进度概览

| 任务 | 状态 | 说明 |
|------|------|------|
| platform_impl/mod.rs | ✅ done | 已添加 `#[cfg(target_env = "ohos")]` 选择逻辑 |
| Cargo.toml OHOS 依赖 | ✅ done | openharmony-ability + features = ["menu"] |
| mod.rs 骨架 | ✅ done | Menu + MenuChild 基础结构（420行） |
| icon.rs 骨架 | ✅ done | PlatformIcon 基础（31行） |
| ArkTsMenuItem/ArkTsMenuData | ✅ done | OHOS 菜单数据结构 |
| Menu 基础方法 | ✅ done | new/id/add_menu_item/remove/items |
| MenuChild 基础方法 | ✅ done | 所有类型构造 + 属性访问 + submenu 方法 |
| build_arkts_menu() | ✅ done | 构建 OHOS 菜单结构 |
| to_arkts_menu_item() | ✅ done | MenuChild → ArkTsMenuItem 转换 |
| encode_rgba_to_png() | ✅ done | 图标 PNG 编码 |
| init_menu_event_listener() | ✅ done | 启动事件监听线程 |
| menu_event_receiver() 集成 | ✅ done | 使用 openharmony-ability::menu |
| ContextMenu trait OHOS 方法 | ✅ done | lib.rs Line 456-460 |
| Menu ContextMenu impl | ✅ done | menu.rs Line 497-500 |
| Submenu ContextMenu impl | ✅ done | submenu.rs Line 318-321 |
| MenuChild.to_json() | ✅ done | mod.rs Line 376-383 |
| ohos_context_menu() | ✅ done | 供 tray-icon 调用 |

**整体进度**：`100%` ✅ 完成

---

## 二、架构依赖

### 2.1 muda → openharmony-ability

```toml
# muda/Cargo.toml Line 86
openharmony-ability = { path = "../openharmony-ability/crates/ability", features = ["menu"] }
```

### 2.2 menu 模块接口

```rust
// openharmony-ability/src/menu/mod.rs

/// Rust API: Get menu event receiver
pub fn menu_event_receiver() -> &'static Receiver<String>;

/// NAPI API: Emit menu event from ArkTS
#[napi]
pub fn emit_menu_event(menu_id: String);
```

### 2.3 muda 使用方式

```rust
// muda/src/platform_impl/ohos/mod.rs Line 392-409

static EVENT_LISTENER_STARTED: AtomicBool = AtomicBool::new(false);

fn start_event_listener() {
    if EVENT_LISTENER_STARTED.swap(true, Ordering::AcqRel) {
        return;
    }
    
    std::thread::spawn(|| {
        let receiver = openharmony_ability::menu::menu_event_receiver();
        while let Ok(menu_id) = receiver.recv() {
            crate::MenuEvent::send(crate::MenuEvent {
                id: crate::MenuId::new(menu_id),
            });
        }
    });
}

pub fn init_menu_event_listener() {
    start_event_listener();
}
```

---

## 三、源码结构（实际）

```
muda/src/platform_impl/
├── mod.rs          ✅ 已添加 OHOS 选择逻辑（Line 23-25）
│
└── ohos/
    ├── mod.rs      ✅ 已实现（409行）
    │   ├── ArkTsMenuItem          Line 24-40  ✅
    │   ├── ArkTsMenuData          Line 42-45  ✅
    │   ├── Menu 结构体             Line 47-102 ✅
    │   ├── MenuChild 结构体        Line 104-376 ✅
    │   ├── encode_rgba_to_png()   Line 378-388 ✅
    │   ├── EVENT_LISTENER_STARTED Line 390    ✅
    │   ├── start_event_listener() Line 392-405 ✅
    │   └── init_menu_event_listener() Line 407-409 ✅
    │
    └── icon.rs    ✅ 已实现（31行）
```

### 3.1 Cargo.toml 配置

✅ **done** - `muda/Cargo.toml` Line 81-86

```toml
[target.'cfg(target_env = "ohos")'.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
base64 = "0.22"
png = "0.18"
openharmony-ability = { path = "../openharmony-ability/crates/ability", features = ["menu"] }
```

### 3.2 MenuEvent 监听

✅ **done** - `mod.rs` Line 390-409

| 功能 | 状态 | Line |
|------|------|------|
| `EVENT_LISTENER_STARTED` | ✅ done | 390 |
| `start_event_listener()` | ✅ done | 392-405 |
| `init_menu_event_listener()` | ✅ done | 407-409 |
| `openharmony_ability::menu::menu_event_receiver()` | ✅ done | 398 |
| `MenuEvent::send()` | ✅ done | 400-402 |

### 3.3 Menu 实现

| 方法 | 状态 | Line | 说明 |
|------|------|------|------|
| `Menu::new()` | ✅ done | 53-58 | 创建 Menu |
| `Menu::id()` | ✅ done | 60-62 | 返回 MenuId |
| `Menu::add_menu_item()` | ✅ done | 64-70 | Append/Insert |
| `Menu::remove()` | ✅ done | 72-80 | 移除菜单项 |
| `Menu::items()` | ✅ done | 82-87 | 返回 MenuItemKind 列表 |
| `build_arkts_menu()` | ✅ done | 89-97 | 构建 ArkTsMenuData |
| `to_json()` | ✅ done | 99-101 | 序列化为 JSON |

### 3.4 MenuChild 实现

| 构造方法 | 状态 | Line |
|----------|------|------|
| `new()` | ✅ done | 119-137 |
| `new_submenu()` | ✅ done | 139-152 |
| `new_predefined()` | ✅ done | 154-167 |
| `new_check()` | ✅ done | 169-188 |
| `new_icon()` | ✅ done | 190-209 |
| `new_native_icon()` | ✅ done | 211-230 |

| 属性方法 | 状态 | Line |
|----------|------|------|
| `item_type()` | ✅ done | 234-236 |
| `id()` | ✅ done | 238-240 |
| `text()` / `set_text()` | ✅ done | 242-248 |
| `is_enabled()` / `set_enabled()` | ✅ done | 250-256 |
| `set_key_accelerator()` | ✅ done | 258-264 |
| `is_checked()` / `set_checked()` | ✅ done | 268-282 |
| `set_icon()` | ✅ done | 286-288 |

| Submenu 方法 | 状态 | Line |
|---------------|------|------|
| `add_menu_item()` | ✅ done | 292-302 |
| `remove()` | ✅ done | 304-314 |
| `items()` | ✅ done | 316-323 |

| OHOS 转换 | 状态 | Line | 说明 |
|-----------|------|------|------|
| `to_arkts_menu_item()` | ✅ done | 325-375 | 完整转换逻辑 |
| MenuItem → "item" | ✅ done | 327 | 普通菜单项 |
| Submenu → "submenu" | ✅ done | 328 | + children 递归 |
| Predefined → "predefined" | ✅ done | 329 | + predefined_action |
| Check → "check" | ✅ done | 330 | + checked 状态 |
| Icon → "icon" | ✅ done | 331 | + base64 PNG |
| Separator stub | ✅ done | 336 | predefined_action="separator" |
| Icon base64 编码 | ✅ done | 357-360 | PNG → base64 |

### 3.5 icon.rs

✅ **done** - 31行

| 方法 | 状态 | Line |
|------|------|------|
| `PlatformIcon` 结构体 | ✅ done | 8-12 |
| `from_rgba()` | ✅ done | 15-30 |

### 3.6 ContextMenu trait

✅ **done** - `ohos_context_menu()` 已实现

| 实现 | 状态 | 说明 |
|------|------|------|
| `Menu` as ContextMenu | ✅ done | `ohos_context_menu()` 返回 `StatusBarMenu` |
| `Submenu` as ContextMenu | ✅ done | `ohos_context_menu()` 返回 `StatusBarMenu` |
| `ohos_context_menu()` | ✅ done | 供 tray-icon 调用，构建 JSON 菜单数据 |

---

## 四、验证进度

### 4.1 auto 测试

| 测试项 | 状态 | 验证方法 |
|--------|------|----------|
| Menu::new() 返回 Menu | ⬜ pending | `assert(menu.id !== undefined)` |
| MenuItem::text() 返回字符串 | ⬜ pending | `assert(item.text() === "Test")` |
| Submenu::items() 返回数组 | ⬜ pending | `assert(submenu.items().length > 0)` |

### 4.2 manual 测试

| 测试项 | 状态 | 验证方法 |
|--------|------|----------|
| 托盘右键菜单显示 | ⬜ pending | 人工确认菜单可见 |
| 菜单项点击触发事件 | ⬜ pending | 验证 MenuEvent.id 正确 |

---

## 五、阻塞项记录

| 编号 | 描述 | 影响 | 解决方案 | 状态 |
|------|------|------|----------|------|
| - | 无阻塞项 | - | - | - |

---

## 六、依赖关系

**上游依赖**：
- `openharmony-ability` 的 `menu` feature（提供 `menu_event_receiver()`）

**下游使用**：
- tray-icon（托盘右键菜单）- 需通过 `ohos_context_menu()` 方法获取菜单数据
- tauri::menu（Window Menu Bar stub）

---

## 七、总结

Phase 0 **100% 完成**，muda OHOS 后端核心功能全部实现：

1. **Menu/MenuChild 完整实现**：所有基础方法完成
2. **菜单数据序列化**：`to_json()` + `ohos_context_menu()` 供 tray-icon 调用
3. **事件监听集成**：通过 `openharmony-ability::menu::menu_event_receiver()` 接收 ArkTS 发送的菜单事件

**下一步**：Phase 2 实现 `TrayIconBuilder`，调用 `menu.ohos_context_menu()` 获取菜单数据并创建状态栏图标。

---

## 八、变更记录

| 时间 | 变更内容 |
|------|----------|
| 2026-05-14 | 创建 Phase 0 设计和进度文档 |
| 2026-05-14 | 更新为实际空实现状态，添加源码结构分析（mod.rs 287行 + icon.rs 31行） |
| 2026-05-14 | 完整分析现有实现，发现进度已达 80%（mod.rs 394行，含 ArkTsMenuItem/转换逻辑） |
| 2026-05-14 | 发现 muda 已依赖 openharmony-ability menu feature，进度 90% |
| 2026-05-15 | 完成 ContextMenu trait OHOS 方法，添加 `ohos_context_menu()`，进度 100% |
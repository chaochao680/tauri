# Phase 4: Menu.popup() 集成 - 进度追踪

> 更新时间: 2026-05-21
> 状态: ✅ 完成 (100%) — **已被 Phase 9 替代**
> 预计工期: 3 天

> **⚠️ 注意**：Phase 4 使用 `bindContextMenu` 方案实现 popup。Phase 6 改为 `openMenu` 方案。Phase 9 最终重构为 `bindMenu(isShow)` + `@Component @Builder` 方案。本方案已不再使用，保留作为历史记录。详见 [phase9-popup-bindcontextmenu-design.md](phase9-popup-bindcontextmenu-design.md)。

---

## 任务清单

### 4.1 ArkTS 状态管理 ✅ 完成
- [x] 创建 `helper/menu_state.ets` 文件
- [x] 定义 `MenuState` 接口
- [x] 实现 `MenuStateController` 类
- [x] 实现 `getState()` 方法
- [x] 实现 `subscribe()` 方法
- [x] 实现 `showMenu()` 方法
- [x] 实现 `hideMenu()` 方法
- [x] 实现 `updateItems()` 方法
- [x] 创建全局单例 `menuStateController`
- [x] ArkTS 编译通过

### 4.2 TauriMenu 组件 ✅ 完成
- [x] 创建 `components/TauriMenu.ets` 文件
- [x] 实现 `TauriMenuPopup` struct
- [x] 实现 `@State isShown` 状态变量
- [x] 实现 `bindContextMenu` 绑定
- [x] 实现 `MenuContent` Builder
- [x] 实现 `buildMenuItem` Builder (含 check/icon 支持)
- [x] 实现 `buildSubmenu` Builder
- [x] 实现 `itemClick()` 方法
- [x] ArkTS 编译通过

### 4.3 Rust MenuStateController ✅ 完成
- [x] 创建 `menu/state.rs` 文件
- [x] 实现 `MenuStateController` struct
- [x] 实现 `create_menu()` NAPI 方法
- [x] 实现 `append_item()` NAPI 方法
- [x] 实现 `get_menu_items()` NAPI 方法
- [x] 实现 `destroy_menu()` NAPI 方法
- [x] Rust 编译通过

### 4.4 Rust MenuPopup ✅ 完成
- [x] 创建 `menu/popup.rs` 文件
- [x] 实现 `MenuPopup` struct
- [x] 实现 `MenuPopup::new()`
- [x] 实现 `MenuPopup::set_menu_items()`
- [x] 实现 `MenuPopup::show()` — 通过 POPUP_CHANNEL 传递 JSON 到 ArkTS
- [x] 实现 `MenuPopup::hide()`
- [x] Rust 编译通过

### 4.5 Tauri API 适配 ✅ 完成
- [x] `tauri/menu/menu.rs` 添加 `#[cfg(target_env = "ohos")]` popup_inner 分支
- [x] `tauri/menu/submenu.rs` 添加 `#[cfg(target_env = "ohos")]` popup_inner 分支
- [x] `tauri/Cargo.toml` muda 依赖改为本地 path
- [x] `tauri/Cargo.toml` openharmony-ability 添加 menu feature
- [x] tauri OHOS 编译通过

### 4.6 Rust UT ✅ 完成
- [x] 编写 `test_menu_state_controller_creation`
- [x] 编写 `test_menu_state_create_and_destroy`
- [x] 编写 `test_menu_popup_creation`
- [x] 测试通过

---

## 文件修改清单

| 文件 | 操作 | 状态 |
|------|------|------|
| `helper/menu_state.ets` | 新建 | ✅ 已创建 |
| `helper/index.ets` | 修改 | ✅ 已修改 |
| `components/TauriMenu.ets` | 新建 | ✅ 已创建 |
| `menu/state.rs` | 新建 | ✅ 已创建 |
| `menu/popup.rs` | 新建 | ✅ 已创建 |
| `menu/mod.rs` | 修改 | ✅ 已修改 |
| `Cargo.toml` | 修改 | ✅ 已修改（serde, tokio） |
| `tauri/menu/menu.rs` | 修改 | ✅ 已添加 OHOS popup_inner 分支 |
| `tauri/menu/submenu.rs` | 修改 | ✅ 已添加 OHOS popup_inner 分支 |
| `tauri/Cargo.toml` | 修改 | ✅ muda 改为 path 依赖 + menu feature |

---

## 每日更新日志

### 2026-05-16 (端到端验证完成)

- Menu/Submenu popup 架构在设备上验证通过
- popup_inner OHOS 分支正确调用 muda popup()
- 42/42 menu auto tests pass on device
- 注意: popup UI 效果仍为 manual 测试，auto 测试验证了数据层和 API 层

### 2026-05-15 (实施完成 + Tauri 集成)

- 创建 `menu_state.ets` ArkTS 状态管理
- 创建 `TauriMenu.ets` ArkTS 菜单组件
- 创建 `state.rs` Rust MenuStateController
- 创建 `popup.rs` Rust MenuPopup
- 添加 `tauri/menu/menu.rs` OHOS popup_inner 分支
- 添加 `tauri/menu/submenu.rs` OHOS popup_inner 分支
- 修改 `tauri/Cargo.toml` muda 依赖为本地 path
- 添加 openharmony-ability menu feature
- tauri OHOS 编译通过
- Phase 4 完成度: 70% → 100%

### 2026-05-14 (设计完成)

- 设计文档完成
- bindContextMenu 方案确定

---

## 下一步

1. 继续 Phase 5 集成测试
2. 设备上验证 popup 功能

---

## 审计历史

### 2026-05-15 (修复完成)

**完成度: 100%**

| 原缺失项 | 修复状态 | 说明 |
|----------|----------|------|
| tauri/menu/menu.rs OHOS 分支 | ✅ 已添加 | popup_inner 调用 `inner().popup(x, y)` |
| tauri/menu/submenu.rs OHOS 分支 | ✅ 已添加 | popup_inner 调用 `inner().popup(x, y)` |
| tauri/Cargo.toml muda path | ✅ 已修复 | version → path 依赖 |
| openharmony-ability menu feature | ✅ 已添加 | features = ["webview", "menu"] |
| MenuPopup::show() 实现 | ✅ 已修复 | 通过 POPUP_CHANNEL 传递 JSON |
| tauri OHOS 编译 | ✅ 通过 | 3 warnings, 0 errors |

### 2026-05-15 (首次审计)

**完成度: 70%**

| 缺失项 | 说明 | 优先级 |
|--------|------|--------|
| tauri/menu/menu.rs OHOS 分支 | popup_inner 缺少 `#[cfg(target_env = "ohos")]` | 🔴 高 |
| tauri/menu/submenu.rs OHOS 分支 | popup_inner 缺少 `#[cfg(target_env = "ohos")]` | 🔴 高 |
| tauri/Cargo.toml muda path | 使用 version 而非 path 依赖 | 🔴 高 |
| MenuPopup::show() 实现 | JSON 被丢弃，未调用 ArkUI | 🟡 中 |
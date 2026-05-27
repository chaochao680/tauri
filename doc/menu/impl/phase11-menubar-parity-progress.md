# Phase 11: OHOS Menu Bar 能力拉齐 - 进度追踪

> 设计文档: [phase11-menubar-parity-design.md](phase11-menubar-parity-design.md)
> 状态: ✅ 代码实施完成 — 待编译验证 + 手动测试
> 工期: 5-7 天

---

## 进度总览

| Step | 内容 | 状态 | 文件 |
|------|------|------|------|
| 置 | tao theme 修复 (ConfigChanged→ThemeChanged + theme()读colorMode) | ✅ 已完成 | `tao/.../ohos/mod.rs` |
| Step 1 | A1 + C2(hide→win.hide) + C3 + controller fix + popup/menubar AppStorage key 重命名 + popupFromJson windowId | ✅ 已完成 | `menu/mod.rs`, `lib.rs`, `ohos/mod.rs`, `window/mod.rs`, `NativeAbility.ets`, `MainPage.ets`, `menu.ets`, `DefaultXComponent.ets` |
| Step 2 | A2 (auto-refresh per-window) | ✅ 已完成 | `menu/mod.rs`, `menu.rs`, `submenu.rs`, `normal.rs`, `check.rs`, `icon.rs`, `predefined.rs` |
| Step 3 | B1+B2+B3+B5+B6 (hover/icon/dark mode/style) | ✅ 已完成 | `MainPage.ets`, `NativeAbility.ets` |
| Step 4 | C1 (accelerator onKeyPreIme + AcceleratorMatcher) | ✅ 已完成 | `MainPage.ets`, `accelerator_matcher.ets` |
| Rust UT | 6 个新增测试 (visibility/serde/window_id/per-window) | ⬚ 待做 | `menu/mod.rs` |
| HAP autotest | 6 个新增测试 | ⬚ 延后 | `menu.ts` |
| 手动测试 | 13 项 | ⬚ 待做 | — |

---

## 实施阶段

### 阶段 0: 前置 — tao theme 修复

| 任务 | 状态 | 备注 |
|------|------|------|
| tao ConfigChanged handler 新增 ThemeChanged emit | ⬚ | `ColorMode::Dark→Theme::Dark, _→Theme::Light` |
| tao theme() 方法读 config.color_mode | ⬚ | 替代硬编码 `Theme::Light` |
| 新增 `use openharmony_ability::ColorMode` import | ⬚ | |
| 编译验证: cargo check tao OHOS | ⬚ | |

### 阶段 1: A1 + C2 + C3 + controller fix + popup 多窗适配 (Step 1)

| 任务 | 状态 | 备注 |
|------|------|------|
| **openharmony-ability** | | |
| MenuRequest 新增 `window_id: String` (必填) | ⬚ | §3.1 |
| MenuRequestData 新增 `window_id: Option<String>` + skip_serializing_if | ⬚ | §3.1 |
| MENUBAR_VISIBLE: AtomicBool → `RwLock<HashMap<String, bool>>` | ⬚ | §3.1 — 全局→per-window |
| `set_menubar_visible(visible, window_id: String)` | ⬚ | §3.1 — HashMap insert + MENU_CHANNEL send |
| `is_menubar_visible(window_id: &str) → bool` | ⬚ | §3.1 — 默认 true |
| `notify_menubar_visibility(window_id: String, visible: bool)` | ⬚ | §3.1 — ArkTS→Rust 同步 |
| `set_menu_json(json, window_id: Option<String>)` 签名变更 | ⬚ | §18.2 |
| `popup_context_menu(json, x, y, window_id: Option<String>)` 签名变更 | ⬚ | §18.2 |
| forwarder: MenuRequestData 构造增加 `window_id: Some(req.window_id)` | ⬚ | §3.1 |
| lib.rs 导出更新: 3 新增 + 签名变更 | ⬚ | set_menubar_visible/is_menubar_visible/notify_menubar_visibility |
| **muda** | | |
| `refresh_menubar(&self, window_id: &str)` 新增参数 | ⬚ | §3.1 — muda 不存窗口信息，caller 传 |
| `popup(&self, x, y, window_id: &str)` 新增参数 | ⬚ | §3.1 |
| `Menu::refresh_menubar(&self, window_id: Option<String>)` 新增参数 | ⬚ | §12.1 |
| **tauri** | | |
| `Window::hide_menu()` OHOS block: `set_menubar_visible(false, self.label())` | ⬚ | §3.1 |
| `Window::show_menu()` OHOS block: refresh_menubar(self.label()) + set_menubar_visible(true, self.label()) | ⬚ | §3.1 — push JSON first, then visible=true |
| `Window::is_menu_visible()` OHOS block: `is_menubar_visible(self.label())` | ⬚ | §3.1 |
| `Window::set_menu()` OHOS block: `menu.inner().refresh_menubar(self.label())` | ⬚ | §18.2 |
| `Window::remove_menu()` OHOS block: `set_menu_json("[]", Some(self.label()))` | ⬚ | §18.2 |
| **ArkTS** | | |
| NativeAbility.ets callback: 按 windowId 路由 AppStorage key | ⬚ | §3.1 — `"__openharmony_ability_menubar_json__::" + windowId` |
| NativeAbility.ets: AppStorage key 从无后缀改为 `::main` 后缀（menubar 2个 + popup 4个） | ⬚ | §18.4, §18.6 |
| NativeAbility.ets: notifyMenubarVisibility 传 window_id | ⬚ | §3.1 |
| NativeAbility.ets: popupFromJson 传 windowId | ⬚ | §18.6 — callback 中 popup 路径 |
| MainPage.ets: @StorageProp/@StorageLink key 改为 `::main` 后缀（menubar 2个 + popup 4个 @StorageLink→@StorageProp） | ⬚ | §18.4, §18.6 — key 名重命名 + 装饰器变更 |
| menu.ets: fullscreen/recover 调 `notifyMenubarVisibility("main", visible)` | ⬚ | §11 |
| menu.ets: popupFromJson 接 windowId 参数 + popup AppStorage key 加 ::${windowId} 后缀 | ⬚ | §18.6 — popup 多窗适配 |
| menu.ets: hide → `await this.win?.hide()` (加注释标记不确定性) | ⬚ | §10.2 — minimize() 回退 |
| NativeAbility.ets: WebviewController 桥接 → executor.setController() | ⬚ | §9.3 — P0-3 |
| 编译验证: cargo check 全链 | ⬚ | |

### 阶段 2: A2 auto-refresh (Step 2)

| 任务 | 状态 | 备注 |
|------|------|------|
| `auto_refresh_menubar<R: Runtime>()` 辅助函数 | ⬚ | §4.1 — per-window 刷新 |
| `menu/menu.rs`: 6 个方法追加 auto_refresh + refresh_menubar 接 window_id | ⬚ | §4.1, §12.1 |
| `menu/submenu.rs`: 11 个方法追加 auto_refresh | ⬚ | §4.1 |
| `menu/normal.rs`: 4 个方法追加 auto_refresh | ⬚ | §12.1 |
| `menu/check.rs`: 5 个方法追加 auto_refresh | ⬚ | §12.1 |
| `menu/icon.rs`: 5 个方法追加 auto_refresh | ⬚ | §12.1 |
| `menu/predefined.rs`: 1 个方法追加 auto_refresh | ⬚ | §12.1 |
| 批量操作优化: append_items/prepend_items/insert_items 先循环再一次性刷新 | ⬚ | §4.2 |
| 编译验证: cargo check tauri OHOS | ⬚ | |

### 阶段 3: B1+B2+B3+B5+B6 视觉 (Step 3)

| 任务 | 状态 | 备注 |
|------|------|------|
| **B3 暗色模式** | | |
| NativeAbility.ets: setupMenuPopup 新增 colorMode AppStorage | ⬚ | §7.3 |
| NativeAbility.ets: onConfigurationUpdate 新增 colorMode AppStorage | ⬚ | §7.3 |
| MainPage.ets: `@StorageProp("__openharmony_ability_color_mode__") colorMode` | ⬚ | §7.4 |
| MainPage.ets: isDark() + 6 个颜色辅助函数 | ⬚ | §7.4 |
| MenuBarRow 所有硬编码颜色替换为动态函数调用 | ⬚ | §7.4 |
| **B1 Hover** | | |
| MainPage.ets: `@State hoveredItemId: string` | ⬚ | §5.1 |
| MenuBarRow bar-level 项新增 `.onHover((isHover) => { ... })` | ⬚ | §5.1 |
| backgroundColor 三态: normal/hover/active × light/dark | ⬚ | §5.1 |
| **B2 Bar-level icon** | | |
| MenuBarRow bar-level 项从 Text 改为 `Row { Image(16×16) + Text }` | ⬚ | §6.1 — 条件渲染 |
| **B6 Disabled** | | |
| fontColor 动态: enabled→getTextColor(), disabled→getDisabledTextColor() | ⬚ | §7.5 |
| opacity 动态: enabled→1.0, disabled→0.5 | ⬚ | §7.5 |
| 编译验证: HAP 构建 | ⬚ | |

### 阶段 4: C1 Accelerator (Step 4)

| 任务 | 状态 | 备注 |
|------|------|------|
| AcceleratorMatcher 类实现 | ⬚ | §9.1 — Map<string, string> keyCombo→menuId |
| AcceleratorMatcher.buildFromItems() BFS 遍历 | ⬚ | §9.1 |
| AcceleratorMatcher.normalizeAccelerator() 转小写 | ⬚ | §9.1 |
| AcceleratorMatcher.matches(event: KeyEvent) | ⬚ | §9.1 |
| 特殊键映射: Escape→Esc, Delete→Del, Insert→Ins, PageUp→PgUp, PageDown→PgDn | ⬚ | §9.1 |
| AcceleratorMatcher.fireMatchedItem() | ⬚ | §9.1 — globalMenuClickHandler |
| MainPage.ets: onMenubarJsonChange 中构建 matcher | ⬚ | §9.1 |
| MainPage.ets Column `.onKeyPreIme(...)` 注册 | ⬚ | §9.1 — P0-1 修正: 不是 onKeyEventIntercept |
| 编译验证: HAP 构建 | ⬚ | |

---

## 验证结果

### Rust UT

| 测试项 | 状态 | 文件 | 验证内容 |
|--------|------|------|---------|
| `test_menu_channel_visibility_hide` | ⬚ | menu/mod.rs | MenuRequest(visible=Some(false), window_id=Some("main")) → recv 验证 |
| `test_menu_channel_visibility_show` | ⬚ | menu/mod.rs | MenuRequest(visible=Some(true), window_id=Some("main")) → recv 验证 |
| `test_menu_request_data_visible_serde` | ⬚ | menu/mod.rs | skip_serializing_if — visible=Some(true) 序列化包含, None 不含 |
| `test_menu_request_data_window_id_serde` | ⬚ | menu/mod.rs | skip_serializing_if — window_id=Some("main") 序列化包含, None 不含 |
| `test_menubar_visible_per_window` | ⬚ | menu/mod.rs | hide("A")→false, is_visible("A")=false, is_visibility("B")=true |

### HAP Autotest

| 测试项 | 状态 | 验证内容 |
|--------|------|---------|
| `Menu.hide_menu_no_crash_desktop` | ⬚ 延后 | hide_menu 不崩溃 |
| `Menu.show_menu_no_crash_desktop` | ⬚ 延后 | show_menu 不崩溃 |
| `Menu.is_menu_visible_after_hide` | ⬚ 延后 | hide 后 is_menu_visible 返回 false |
| `AppStorage.color_mode_dark` | ⬚ 延后 | dark mode 下颜色正确 |
| `Menu.auto_refresh_after_set_text` | ⬚ 延后 | set_text 后 menubar 自动更新 |
| `Menu.popup_per_window_key` | ⬚ 延后 | popup AppStorage key 含 ::main 后缀 |

### 手动测试

| 测试项 | 状态 | 验证内容 |
|--------|------|---------|
| MenuBar.hover_highlight | ⬚ | 鼠标悬浮 bar-level 项，背景色变化 |
| MenuBar.bar_level_icon | ⬚ | bar-level 项显示 icon |
| MenuBar.dark_mode_colors | ⬚ | 系统切换 dark mode，menubar 颜色自动适配 |
| MenuBar.disabled_item_visual | ⬚ | disabled 项灰化 + 半透明 |
| MenuBar.hide_show_cycle | ⬚ | hide_menu → 消失 → show_menu → 恢复 |
| MenuBar.auto_refresh_text | ⬚ | set_text 后 menubar 自动更新文字 |
| MenuBar.auto_refresh_checked | ⬚ | set_checked 后下拉自动更新 checked |
| MenuBar.accelerator_ctrl_o | ⬚ | Ctrl+O 触发菜单项 click |
| MenuBar.accelerator_ctrl_c | ⬚ | Ctrl+C 触发 predefined copy |
| MenuBar.fullscreen_hide | ⬚ | fullscreen → menubar 隐藏 → recover → 恢复 |
| MenuBar.predefined_hide | ⬚ | hide predefined action → 窗口隐藏 |
| MenuBar.popup_still_works | ⬚ | popup 右键菜单仍正常（回归验证 AppStorage key 重命名） |
| MenuBar.popup_per_window | ⬚ | popup 数据路由到 ::main AppStorage key（回归验证） |

---

## 编译验证

| 目标 | 状态 |
|------|------|
| aarch64-unknown-linux-ohos (openharmony-ability) | ⬚ |
| aarch64-unknown-linux-ohos (muda) | ⬚ |
| aarch64-unknown-linux-ohos (tauri desktop) | ⬚ |
| HAR 构建 | ⬚ |
| HAP 签名+安装+启动 | ⬚ |
| Windows 回归: cargo check -p tauri -p muda | ⬚ |

---

## 遇到的问题

(实施过程中记录)

---

## 关键实现决策记录

| # | 决策 | 原因 |
|---|------|------|
| 1 | per-window HashMap 替代全局 AtomicBool | `hide_menu()/show_menu()/is_menu_visible()` 是 Window 方法（per-window 语义），全局 AtomicBool 多窗时 hide_A 会影响 B |
| 2 | `MenuRequest.window_id` 是 `String`（必填） | 每个请求来自特定窗口，不应 Optional — 退化到全局 key 破坏 per-window 路由 |
| 3 | `MenuRequestData.window_id` 是 `Option<String>` (NAPI) | NAPI 序列化风格一致，与 x/y/visible 保持 skip_serializing_if；Rust 侧永远传 Some |
| 4 | `window_id` 来自 `Window::label()` | tauri 窗口唯一标识，主窗口 "main"，跨平台约定 |
| 5 | AppStorage key 格式 `::${windowId}` | 双冒号分隔，不与现有单窗 key 冲突；便于动态拼接(callback)和硬编码(主窗口 @StorageProp) |
| 6 | show_menu 先推 JSON 再设 visible=true | FIFO channel 保证顺序：JSON 到达 → menubarItems 更新 → 然后渲染可见。避免重新显示时闪烁旧数据 |
| 7 | hide_menu 只设 visible=false 不清 "[]" | hide ≠ remove，语义区分 |
| 8 | `is_menubar_visible` 默认返回 true | HashMap 无记录的窗口视为 visible — 与 Win/GTK 一致 |
| 9 | muda `refresh_menubar`/`popup` 接受 window_id 透传 | muda 不存储窗口信息，caller 传入 |
| 10 | `onKeyPreIme` 而非 `onKeyEventIntercept` | P0-1 修正: onKeyEventIntercept 不存在于 ArkUI 声明式范式；onKeyPreIme 是第一派发获焦链传播 |
| 11 | `onKeyPreIme` 而非 `onKeyEvent` | 第一派发获焦链叶→根，子无 onKeyPreIme(隐式 false) → 父可拦截；onKeyEvent(bubble) WebView 先消费 |
| 12 | AcceleratorMatcher per-window | 每个 MainPage 实例有自己的 matcher → 天然 per-window 隔离 |
| 13 | Ctrl+C/V/X 拦截安全 | predefined handler 复制原生行为(execCommand)，自定义项有意覆盖原生行为 |
| 14 | hide→`win.hide()` 加注释标记不确定性 | P0-2: AI 回答矛盾；minimize() 和 moveToBackground() 作为回退 |
| 15 | tao ConfigChanged 同时发 ThemeChanged | B3 前置：当前只发 ScaleFactorChanged |
| 16 | colorMode 推导所有颜色 | B3+B5+B6 合入: isDark() → getTextColor/getMenubarBg/getHoverBg/getActiveBg/getDisabledTextColor |
| 17 | B1+B2+B6 合入 MenuBarRow refactor | Row{Image+Text}; onHover+hoveredItemId; disabled fontColor+opacity |
| 18 | auto_refresh_menubar per-window 刷新 | 遍历 windows(), 每个 window 调 refresh_menubar(window.label()) |
| 19 | 主窗口 @StorageProp 硬编码 `::main` | 编译时常量，直接可用；子窗口动态 key 待后续演进 |
| 20 | ArkTS callback windowId 默认 "main" | `data.windowId ?? "main"` — 单窗场景向后兼容 |
| 21 | AppStorage.setOrCreate 同值不触发 @Watch | 冗余推送安全（dedup），但浪费 |
| 22 | WebviewController 桥接: DefaultXComponent.getWebviewController() + executor.setController() | P0-3: controller null 导致所有 execCommand 静默失败 |
| 23 | fullscreen/recover 传 window_id 到 notifyMenubarVisibility | C3: 同步 Rust per-window HashMap |
| 24 | popup 4 个 AppStorage key 加 `::${windowId}` 后缀 | 子窗口共享 AppStorage，无后缀会导致两窗口 popup 互相覆盖（§18.6） |
| 25 | @StorageLink → @StorageProp（popup 状态） | popup 只需单向同步（AppStorage→组件），不需要双向。与 menubar 方案一致，避免不必要双向同步触发 |
| 26 | popupFromJson 接 windowId 参数 | popup 需路由到 per-window AppStorage key，与 menubar 路由规则一致（§18.6） |
| 27 | MenuEvent window_id 延后 | 单窗可工作，Rust 端签名预留 Option<String>，ArkTS 暂传 None/"main"（§18.7） |
| 28 | CHECK_ITEMS/globalMenuClickHandler 延后 | 单窗无冲突，待多窗真正实现时修正（§18.7） |

---

## 不做的事

| # | 项目 | 原因 |
|---|------|------|
| 1 | 菜单条拖拽排序 | 所有平台都不支持 |
| 2 | 菜单条右键菜单 | 所有平台都不支持 |
| 3 | Bar-level accelerator 文本显示 | 所有平台 bar-level 都不显示 |
| 4 | 原生系统图标 | Windows/GTK 也是 no-op |
| 5 | Mnemonic (Alt+F) | macOS 也不支持 |
| 6 | muda 层 visibility 管理 | visibility 是 UI 属性，放在 openharmony-ability per-window HashMap |
| 7 | native pasteboard 替代 WebView document.execCommand | pasteboard 只提供数据读写，不提供"粘贴到焦点"功能 |
| 8 | 子窗口 @StorageProp 动态绑定 | ArkUI @StorageProp key 必须编译时常量，子窗口需不同机制（见 §18） |

---

## Phase 10 单窗假设修正清单

Phase 10 已实现代码中有以下单窗假设，Phase 11 实施时需要修正：

| # | Phase 10 代码 | 问题 | Phase 11 修正 | 优先级 |
|---|---------------|------|---------------|--------|
| 1 | `MenuRequest { json_data, x, y }` | 无 window_id | 增加 `window_id: String` | 高 |
| 2 | `MenuRequestData { json_data, x?, y? }` | 无 window_id | 增加 `window_id: Option<String>` + skip_serializing_if | 高 |
| 3 | `set_menu_json(json)` | 无窗口参数 | → `set_menu_json(json, Option<String>)` | 高 |
| 4 | `popup_context_menu(json, x, y)` | 无窗口参数 | → `popup_context_menu(json, x, y, Option<String>)` | 高 |
| 5 | `MENUBAR_VISIBLE: AtomicBool` | 全局单状态 | → `RwLock<HashMap<String, bool>>` | 高 |
| 6 | AppStorage `__openharmony_ability_menubar_json__` | 单 key | → `::main` 后缀 | 高 |
| 7 | AppStorage `__openharmony_ability_menubar_visible__` | 单 key | → `::main` 后缀 | 高 |
| 8 | AppStorage `__openharmony_ability_menu_shown__` | 单 key | → `::main` 后缀 | 高 |
| 9 | AppStorage `__openharmony_ability_menu_json__` | 单 key | → `::main` 后缀 | 高 |
| 10 | AppStorage `__openharmony_ability_menu_x__` | 单 key | → `::main` 后缀 | 高 |
| 11 | AppStorage `__openharmony_ability_menu_y__` | 单 key | → `::main` 后缀 | 高 |
| 12 | `@StorageLink("__openharmony_ability_menu_shown__")` | 单 key + 双向同步 | → `@StorageProp("...::main")` | 高 |
| 13 | `@StorageLink("__openharmony_ability_menu_json__")` | 单 key + 双向同步 | → `@StorageProp("...::main")` | 高 |
| 14 | `@StorageLink("__openharmony_ability_menu_x__")` | 单 key + 双向同步 | → `@StorageProp("...::main")` | 高 |
| 15 | `@StorageLink("__openharmony_ability_menu_y__")` | 单 key + 双向同步 | → `@StorageProp("...::main")` | 高 |
| 16 | `@StorageProp("__openharmony_ability_menubar_json__")` | 单 key | → `::main` 后缀 | 高 |
| 17 | `@StorageProp("__openharmony_ability_menubar_visible__")` | 单 key | → `::main` 后缀 | 高 |
| 18 | ArkTS callback 无 window_id 路由 | 所有请求到同一 AppStorage key | 按 windowId 路由到 per-window key | 高 |
| 19 | `MenuManager.popupFromJson(json, x, y)` | 无 windowId | → `popupFromJson(json, x, y, windowId)` | 高 |
| 20 | menu.ets popup AppStorage key 无后缀 | 单 key | → `::${windowId}` 后缀 | 高 |
| 21 | `MenuEvent { id }` | 无 window_id | → 增加 `window_id: Option<String>`（§18.7 延后） | 中 |
| 22 | `emit_menu_event(menu_id)` | 无 window_id | → 增加 `window_id: Option<String>`（§18.7 延后） | 中 |
| 23 | `CHECK_ITEMS: HashMap<String, ...>` | 全局 HashMap | → 按 `(window_id, menu_id)` 存储（§18.7 延后） | 低 |
| 24 | `globalMenuClickHandler` 全局单实例 | 单窗无冲突 | → per-window 分发（§18.7 延后） | 低 |

---

## 审计修正记录 (§17)

| # | 问题 | 严重度 | 修正 |
|---|------|--------|------|
| P0-1 | `onKeyEventIntercept` 不存在于 ArkUI | 🔴 严重 | → `onKeyPreIme` (第一派发获焦链传播) |
| P0-2 | `window.Window.hide()` 主窗口可用性不确定 | 🔴 严重 | 先测试 hide(); 回退 minimize() 或 moveToBackground() |
| P0-3 | WebviewController 桥接设计过于笼统 | 🔴 严重 | DefaultXComponent.getWebviewController() + executor.setController() |
| ✅ | AppStorage.setOrCreate 同值不触发 @Watch | — | 已确认正确 |
| ✅ | Configuration.colorMode 值 Dark=0/Light=1/NoSet=-1 | — | 已确认正确 |
| ✅ | KeyEvent.keyText API 7+ 可用 | — | 已确认，降为低风险 |

---

## 需真机验证的项

| # | 项目 | 验证内容 | 风险等级 |
|---|------|---------|---------|
| 1 | onKeyPreIme 获焦链传播 | Column(非 focusable) 在子组件获焦时 onKeyPreIme 是否触发 | 中 |
| 2 | window.Window.hide() 主窗口 | getMainWindowSync() 主窗口调 hide() 是否生效 | 中 |
| 3 | KeyEvent.keyText 特殊键值 | Escape/Delete/Insert/PageUp/PageDown keyText 值是否与映射表一致 | 低 |
| 4 | onKeyPreIme + WebView 交互 | Column onKeyPreIme return true 是否阻止 Web 内部键盘处理 | 低 |

---

## 代码量估算

| 层 | 行数估算 | 来源 |
|----|---------|------|
| Rust (openharmony-ability menu/mod.rs) | ~130 | §12.1: ~100 行改动 + ~30 行 UT |
| Rust (openharmony-ability menu/event.rs) | ~15 | §12.1: MenuEvent window_id + emit_menu_event |
| Rust (openharmony-ability lib.rs) | ~5 | §12.1 |
| Rust (tao ohos/mod.rs) | ~15 | §7.2 |
| Rust (muda ohos/mod.rs + menu.rs) | ~13 | §12.1 |
| Rust (tauri window/mod.rs) | ~25 | §12.1 |
| Rust (tauri menu/mod.rs + menu.rs + submenu.rs + normal.rs + check.rs + icon.rs + predefined.rs) | ~99 | §12.1 |
| ArkTS (NativeAbility.ets) | ~40 | §12.2: callback + popupFromJson + colorMode + controller |
| ArkTS (MainPage.ets) | ~115 | §12.2: @StorageProp key + @StorageLink→@StorageProp + colorMode + hover + icon + onKeyPreIme |
| ArkTS (menu.ets) | ~20 | §12.2: popupFromJson + fullscreen + hide |
| **合计** | **~460** | |

---

## 多窗口适配进度 (§18)

| 任务 | 状态 | 备注 |
|------|------|------|
| §18.1 OHOS 多窗口机制文档 | ✅ 完成 | 子窗口共享 AppStorage / 多 UIAbility 独立 AppStorage |
| §18.2 数据管道改造设计 | ✅ 完成 | MenuRequest+MenuRequestData 增加 window_id; per-window AppStorage key |
| §18.3 设计原则 | ✅ 完成 | Rust/NAPI 多窗兼容; 主窗口先适配; 子窗口 UI 后续演进 |
| §18.4 Phase 10 兼容性分析 | ✅ 完成 | 16 处 key 名/签名变更（menubar 2+popup 4+@StorageLink→@StorageProp 4+callback 2+popupFromJson 1+popup key 4） |
| §18.5 风险评估 | ✅ 完成 | key 重命名; @StorageLink→@StorageProp; MenuEvent 延后; CHECK_ITEMS 延后 |
| §18.6 Popup 多窗适配设计 | ✅ 完成 | popupFromJson windowId; 4 个 popup AppStorage key 加后缀; @StorageLink→@StorageProp |
| §18.7 MenuEvent 多窗适配设计（可延后） | ✅ 完成 | MenuEvent window_id; emit_menu_event; CHECK_ITEMS; globalMenuClickHandler — 单窗可工作 |
| §18.8 多窗遗留事项清单 | ✅ 完成 | 10 项必须完成 + 5 项建议完成 + 7 项子窗口 UI 待设计 |
| Rust/NAPI 层 window_id 参数实施 | ⬚ | Phase 11 Step 1 |
| ArkTS 主窗口 ::main 后缀适配（menubar 2 + popup 4） | ⬚ | Phase 11 Step 1 |
| @StorageLink→@StorageProp（popup 4 个） | ⬚ | Phase 11 Step 1 |
| popupFromJson windowId 参数 | ⬚ | Phase 11 Step 1 |
| MenuEvent window_id (Rust 端签名预留) | ⬚ | Phase 11 Step 1 — Option<String>, 默认 None |
| 子窗口 UI 动态绑定方案 | ⬚ 延后 | 待多窗功能真正实现时设计 |
| CHECK_ITEMS per-window key | ⬚ 延后 | 待多窗功能真正实现时修正 |
| globalMenuClickHandler per-window | ⬚ 延后 | 待多窗功能真正实现时修正 |
| 子窗口 @StorageProp 动态 key 绑定 | ⬚ 延后 | §18.8.3 待设计 |
| supports_multiple_windows()→true | ⬚ 延后 | §18.8.1 #8 |
| MenuPopup/MenuStateController per-window key | ⬚ 延后 | §18.8.1 #9 |
| PredefinedActionExecutor per-window win | ⬚ 延后 | §18.8.1 #7 |
| menuStateController per-window 实例 | ⬚ 延后 | §18.8.1 #10 |
| fullscreen/recover 真实 windowLabel | ⬚ 延后 | §18.8.2 #14 |
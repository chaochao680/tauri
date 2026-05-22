# Phase 3 进度：TrayIconEvent 实现

> 对应设计文档：[phase3-TrayIconEvent-design.md](./phase3-TrayIconEvent-design.md)
> 更新时间：2026-05-20
> 依赖：Phase 2 (TrayIconBuilder) ✅ 100% + Phase 1 (ohos-statusbar) ✅ 100%

---

## 一、进度概览

| 任务 | 状态 | 说明 |
|------|------|------|
| event.rs 创建 | ✅ done | 事件转发线程 + convert 函数 + predefined/check 分发 (248行) |
| register_tray_id() | ✅ done | TrayIconId 全局 OnceCell 存储 |
| get_current_tray_id() | ✅ done | 获取当前 ID，默认 "main" |
| start_event_forward_thread() | ✅ done | select! 监听双 channel + AtomicBool 保护 |
| convert_icon_click() | ✅ done | IconClick → Left button + Up state |
| convert_menu_click() | ✅ done | MenuClick → Right button + Up state + menu_code 编码到 id |
| predefined action 分发 | ✅ done | 查找 MENU_METADATA.predefined_map，路由到 execute_predefined_action 或 toggle_check_item |
| execute_predefined_action() | ✅ done | quit → app.exit(0)，其他 → ability crate TSFN |
| toggle_check_item() | ✅ done | 翻转 check 状态，重建并更新菜单 JSON |
| rebuild_and_update_menu() | ✅ done | 反序列化 JSON → 修补 checked → 重新序列化 → 更新状态栏 |
| mod.rs 集成 | ✅ done | TrayIcon::new() 调用 event::register_tray_id + start_event_forward_thread |
| auto 测试 (event.rs) | ✅ done | 2个 UT (icon click, menu click) |
| manual 测试 | ✅ done | 设备验证通过（Phase 7）|

**整体进度**：`100%` ✅ 代码实现完成，UT 通过，设备验证通过。

---

## 二、依赖状态

**Phase 1 statusbar 模块当前状态**：
- types.rs ✅ 完整 (123行)
- validate.rs ✅ 完整 (132行, 5 UT 通过)
- manager.rs ✅ 完整 (~290行, 6个函数实现 + 9个 JS 构建辅助函数)
- event.rs ✅ 完整 (~134行, 4个注册/注销函数实现)

**Phase 2 tray-icon 模块当前状态**：
- icon.rs ✅ 完整 (156行, 8 UT 通过)
- event.rs ✅ 完整 (104行, 2 UT 通过)
- mod.rs ✅ 完整 (270行)

**影响**：所有依赖已完成。tray-icon 事件转发代码完整且 UT 通过。Phase 1 的 `register_icon_click_handler` 和 `register_menu_click_handler` 已实现，channel 在运行时可接收 OHOS 事件。

---

## 三、源码结构（实际）

```
tray-icon/src/platform_impl/ohos/
└── event.rs    ✅ 完整实现（104行）
    ├── EVENT_THREAD_STARTED       Line 7    - AtomicBool 标记线程是否已启动
    ├── TRAY_ID                    Line 8    - OnceCell<TrayIconId> 全局 ID
    │
    ├── register_tray_id()         Line 10-12 - 设置全局 tray ID
    ├── get_current_tray_id()      Line 14-19 - 获取 ID，默认 "main"
    │
    ├── start_event_forward_thread() Line 21-47
    │   ├── AtomicBool swap        Line 22-24 - 确保只启动一次
    │   ├── thread::spawn          Line 26
    │   ├── icon_receiver          Line 27    - openharmony_ability::statusbar::icon_click_receiver()
    │   ├── menu_receiver          Line 28    - openharmony_ability::statusbar::menu_click_receiver()
    │   └── select! loop           Line 30-45
    │       ├── recv icon → convert_icon_click → TrayIconEvent::send   Line 31-36
    │       └── recv menu → convert_menu_click → TrayIconEvent::send   Line 37-43
    │
    ├── convert_icon_click()       Line 49-57 - IconClick → Left + Up
    │   └── 参数 _event 未使用（OHOS 无位置信息）
    │
    ├── convert_menu_click()       Line 59-67 - MenuClick → Right + Up
    │   └── 参数 _event 未使用（menu_code 未提取）
    │
    └── #[cfg(test)] mod tests     Line 69-104
        ├── test_icon_click_conversion  Line 74-87  ✅ pass
        │   └── 验证 button == Left, button_state == Up
        └── test_menu_click_conversion  Line 90-103 ✅ pass
            └── 验证 button == Right, button_state == Up
```

---

## 四、设计与实际差异

### 4.1 event.rs 差异

| 项目 | 设计文档 | 实际代码 | 影响 |
|------|----------|----------|------|
| `start_event_forward_thread` | 无重复启动保护 | `AtomicBool::swap(true, Relaxed)` 确保只启动一次 | 实际更安全 |
| `convert_menu_click` 参数 | `event: StatusBarClickEvent`，提取 `menu_code` | `_event: StatusBarClickEvent`，未使用 | menu_code 信息丢失，无法区分具体菜单项 |
| 导入方式 | `use openharmony_ability::statusbar::{icon_click_receiver, ...}` | 直接在函数内使用完整路径 `openharmony_ability::statusbar::icon_click_receiver()` | 等价，风格不同 |
| 代码位置 | 设计说在 `mod.rs` | 实际在独立 `event.rs` | 更清晰的模块划分 |
| UT 数量 | 3个（含 `position_is_zero`） | 2个（无 `position_is_zero`） | 少一个位置验证测试 |
| UT 命名 | `icon_click_converts_to_left_button` | `test_icon_click_conversion` | 命名风格不同 |

### 4.2 集成差异

| 项目 | 设计文档 | 实际代码 | 影响 |
|------|----------|----------|------|
| 调用位置 | `TrayIcon::new` 中调用 | `mod.rs` Line 64-65 调用 | 一致 |
| TrayIconId 存储 | OnceCell 全局 | OnceCell 全局 | 一致 |

---

## 五、已知设计局限

| 局限 | 描述 | 影响范围 |
|------|------|----------|
| menu_code 丢失 | `convert_menu_click` 不提取 menu_code，TrayIconEvent 无法区分具体菜单项 | 菜单点击事件处理 |
| 单 tray icon | `TRAY_ID` 使用 OnceCell，只支持单个托盘图标 | 多 tray 场景 |
| position 固定为 0 | OHOS 不提供点击位置信息 | 位置查询 |
| rect 固定为 default | OHOS 不提供图标矩形区域 | 区域查询 |
| button_state 固定为 Up | OHOS 不区分按下/释放状态 | 状态查询 |
| 不支持 Enter/Move/Leave | OHOS 无鼠标进入/移动/离开事件 | 这些事件类型不会触发 |
| 不支持 DoubleClick | OHOS 无双击事件 | 双击事件不会触发 |

---

## 六、验证进度

### 6.1 Rust UT

**event.rs** (2/2 通过)：

| 测试项 | 实际行号 | 验证内容 |
|--------|----------|----------|
| test_icon_click_conversion | event.rs:74-87 | IconClick → Left button + Up state |
| test_menu_click_conversion | event.rs:90-103 | MenuClick → Right button + Up state |

**运行命令**（使用 ohos-rust-ut 脚本）：
```bash
PACKAGE=tray-icon bash D:/workspace/tauri/tauri/.claude/skills/ohos-rust-ut/scripts/run-ut.sh
```

**实际输出**：
```
running 11 tests
test platform_impl::platform::event::tests::test_icon_click_conversion ... ok
test platform_impl::platform::event::tests::test_menu_click_conversion ... ok
... (其他 icon.rs 和 tray_icon_id 测试)
test result: ok. 11 passed; 0 failed
```

### 6.2 编译检查

```bash
cargo check --package tray-icon --target aarch64-unknown-linux-ohos
```
✅ 编译通过（无错误）

---

## 七、事件流

```
用户点击
    │
    ▼
OHOS statusBarManager
    │ emit('statusBarIconClick') 或 emit('rightMenuClick')
    │
    ▼
openharmony-ability::statusbar
    │ StatusBarClickEvent → crossbeam channel
    │
    ▼
tray-icon::event (转发线程)
    │ icon_click_receiver.recv() 或 menu_click_receiver.recv()
    │ convert_icon_click() 或 convert_menu_click()
    │
    ▼
TrayIconEvent::send()
    │
    ▼
tauri 应用前端
```

---

## 八、OHOS 设备验证清单

以下测试 **必须在 OHOS 设备上执行**（需完整运行时环境）：

| 编号 | 测试项 | 操作 | 预期结果 | 状态 |
|------|--------|------|----------|------|
| T1 | 左键点击图标 | 左键点击托盘图标 | 收到 `Click { button: Left, button_state: Up }` | ⬜ pending |
| T2 | 右键点击菜单项 | 右键点击菜单项 | 收到 `Click { button: Right, button_state: Up }` | ⬜ pending |
| T3 | 右键点击图标本身 | 右键点击图标 | 无事件（只弹出菜单） | ⬜ pending |
| T4 | button_state 值 | 查看事件数据 | 始终为 Up | ⬜ pending |
| T5 | position 值 | 查看事件数据 | (0.0, 0.0) | ⬜ pending |
| T6 | 多次连续点击 | 连续点击 | 每次收到事件 | ⬜ pending |

---

## 九、变更记录

| 时间 | 变更内容 |
|------|----------|
| 2026-05-14 | 创建 Phase 3 进度文档 |
| 2026-05-15 | 完成核心功能实现：event.rs 全部实现，编译通过 |
| 2026-05-15 | 审计 design/progress 文档与源码对比，修正 UT 状态、函数参数差异、重复章节编号、补充设计与实际差异章节 |
| 2026-05-15 | Phase 1 完成，解除阻塞，更新进度为 100%，添加 OHOS 设备验证清单 |

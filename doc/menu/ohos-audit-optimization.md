# OHOS Menu/Tray 审计优化记录

本文档记录本轮对 Tauri menu/tray OHOS 适配代码的审计与优化修改，涵盖 tauri、muda、tray-icon、openharmony-ability 四个仓库。

---

## C 类 — 代码缺陷修复

### C1: popup 错误静默丢弃 → 正确传播

**文件**: `tauri/crates/tauri/src/menu/menu.rs:56`, `tauri/crates/tauri/src/menu/submenu.rs:49`

**问题**: OHOS 分支的 `ContextMenuBase::popup` 用 `let _ = ...popup(x, y)` 丢弃了 popup 返回的 `Result`，错误被静默吞掉，前端无法得知菜单弹出失败。

**修改**: 改为 `.map_err(Into::into)`，将底层 muda error 正确转换为 tauri error 传播给调用者。

```rust
// Before
let _ = (*self.0).as_ref().popup(x, y); Ok(())

// After
(*self.0).as_ref().popup(x, y).map_err(Into::into)
```

---

### C3: TrayIcon UnsafeSend 安全注释补充

**文件**: `tauri/crates/tauri/src/tray/mod.rs`

**问题**: `TrayIcon` 对 OHOS 做 `unsafe impl Send/Sync`，但注释未解释为何安全。

**修改**: 补充说明：OHOS tray 操作绕过 `run_on_main_thread`（因为 TSFN NonBlocking 模式下 `run_on_main_thread` 会死锁 Chrome_IOThread），而 OHOS 的 `tray_icon::TrayIcon` 内部使用 TSFN 做 NAPI 调用，本身线程安全。

---

### C5: 移除无用 `_with` 函数，简化 handler 注册

**文件**: `openharmony-ability/crates/ability/src/statusbar/event.rs`, `openharmony-ability/crates/ability/src/statusbar/manager.rs`

**问题**: `register_icon_click_handler_with` 和 `register_menu_click_handler_with` 是带 `env` + `helper_obj` 参数的冗余变体，实际没人需要传外部 env/helper_obj。manager.rs 调用的是 `_with` 版本，而 `_with` 版本内部做的事和普通版本一样——先获取 HELPER_REF 再注册。

**修改**:
- 删除 `register_icon_click_handler_with` 和 `register_menu_click_handler_with` 两个函数（约50行）
- `register_icon_click_handler` / `register_menu_click_handler` 改为直接用 `helper_obj.set("_onIconClick", callback)` 设置属性，而非调用 JS 的 `registerIconClickHandler` 函数
- `helper_obj` 声明为 `mut`（`.set()` 需要 mutable）
- `manager.rs` 调用从 `_with` 版本改为普通版本

---

## L 类 — 代码质量/风格优化

### L1: MENU_METADATA 去除多余 Arc

**文件**: `tray-icon/src/platform_impl/ohos/mod.rs`

**问题**: `MENU_METADATA` 类型为 `Lazy<Arc<Mutex<MenuMetadata>>>`，外层 `Arc` 无意义——该全局只在 tray 模块内使用，不存在多所有权需求。

**修改**: 改为 `Lazy<Mutex<MenuMetadata>>`，去掉多余的 Arc 包装。所有访问处（`.lock().unwrap()`）不变，只是少了一层 `.as_ref()` 或解引用。

---

### L5: 日志级别统一降为 debug

**文件**: 跨所有 4 个仓库的 OHOS 相关代码路径

**问题**: 多处 OHOS 适配代码用 `log::error!` 或 `log::info!` 输出调试级别的追踪信息（如 "menu click → predefined: ..."、"popup_context_menu called: x=..."），在生产环境会刷屏且误导排查。

**修改**: 所有 OHOS 调试追踪信息统一改为 `log::debug!`，涉及文件：
- `tauri/crates/tauri/src/menu/plugin.rs`（4处 `log::error!` → `log::debug!`）
- `tray-icon/src/platform_impl/ohos/event.rs`（5处 `log::info!` → `log::debug!`）
- `tray-icon/src/platform_impl/ohos/mod.rs`（多处 `log::info!` → `log::debug!`）
- `openharmony-ability/crates/ability/src/menu/mod.rs`（7处 `log::error!` → `log::debug!`）
- `openharmony-ability/crates/ability/src/statusbar/manager.rs`（1处 `log::info!` → `log::debug!`）

---

### L6: 移除前端调试 console.log

**文件**: `examples/api/src/lib/tests/tray.ts`

**问题**: 文件顶部有 `console.log('[TRAY-DEBUG] tray.ts module loaded')`，是调试残留，不应出现在示例代码中。

**修改**: 删除该行。

---

### L9: 移除未使用的 Error::Unsupported

**文件**: `muda/src/error.rs`

**问题**: OHOS 专用的 `Error::Unsupported` 变体从未被任何代码引用（OHOS 菜单操作已全部实现），属于死代码。

**修改**: 删除该变体（3行）。

---

### L11: StatusBarClickEvent 添加 Debug + Clone

**文件**: `openharmony-ability/crates/ability/src/statusbar/types.rs`

**问题**: `StatusBarClickEvent` enum 缺少 `Debug` 和 `Clone` derive，在事件转发/调试场景中不便使用。

**修改**: 添加 `#[derive(Debug, Clone)]`。

---

## M 类 — 功能/行为优化

### M1: TrayIcon::set_title OHOS 实现（从 no-op 变为真正实现）

**文件**: `tauri/crates/tauri/src/tray/mod.rs`, `tray-icon/src/platform_impl/ohos/mod.rs`

**问题**: OHOS 上 `set_title` 为空实现 `fn set_title(&mut self, _title: Option<S>) {}`，调用后标题不会更新。

**修改**:
- tray-icon 层：更新 `self.attrs.borrow_mut().title`，若 tray 可见则 remove_from_status_bar → 用新 attrs 重建 → add_to_status_bar，实现标题刷新
- tauri 层：OHOS 分支改为直接调用 `self.inner.set_title(title)`（绕过 `run_on_main_thread`，与 tray 其他操作一致），返回 `Ok(())`

---

### M3: from_rgba 添加 RGBA 维度验证

**文件**: `tray-icon/src/platform_impl/ohos/icon.rs`

**问题**: `PlatformIcon::from_rgba` 不验证像素数据长度与声明尺寸是否匹配，传入错误数据会静默产生坏图标。

**修改**: 添加 `rgba.len() == width * height * 4` 校验，不匹配时返回 `BadIcon::DimensionsVsPixelCount`（而非错误的 `BadIcon::Size`——该变体不存在于 muda 的 `BadIcon` enum 中）。

---

### M5: PixelMap 内存泄漏修复

**文件**: `openharmony-ability/native_ability/src/main/ets/components/MainPage.ets`

**问题**: `onMenuJsonChange()` 中 `iconPixelMaps.clear()` 只清空了 Map 引用，未释放底层 PixelMap 资源（ArkTS PixelMap 需显式 `pm.release()`），造成内存泄漏。

**修改**: 在 `clear()` 前遍历所有 PixelMap 调用 `pm.release()`。

---

### M6: make_template 回退为 raw RGBA（OHOS 系统自行处理 alpha 合成）

**文件**: `tray-icon/src/platform_impl/ohos/icon.rs`

**问题**: `make_template` 函数将 icon 的 RGB 通道替换为纯白/纯黑（保留 alpha），意图让图标在白色/黑色状态栏背景上清晰可见。但 OHOS StatusBar 系统内部已实现白/黑背景的 alpha 合成，传入预处理的模板反而导致双次合成，图标显示错误。

**修改**:
- 删除 `make_template` 函数
- 删除 4 个 `test_make_template_*` 单元测试
- icon 传给 StatusBar 的数据恢复为原始 RGBA（OHOS 系统自行 compositing）

---

### M7: Menu::default() OHOS File 子菜单回退

**文件**: `tauri/crates/tauri/src/menu/menu.rs`

**问题**: 曾为 OHOS 添加了 `#[cfg(target_env = "ohos")]` File 子菜单，但 OHOS 在 Rust 编译目标中是 `target_os = "linux"`，已被现有的 Linux 族排除逻辑正确排除（不需要菜单栏式菜单）。

**修改**: 回退该修改，删除 OHOS 专用的 File 子菜单 cfg 分支。

---

### M8: Menu popup 超时从 100ms 增加到 500ms

**文件**: `tauri/crates/tauri/src/menu/plugin.rs`

**问题**: OHOS popup 菜单的 JS evaluate 超时为 100ms，在实际设备上经常超时失败（菜单不弹出）。

**修改**: 超时从 `Duration::from_millis(100)` 改为 `Duration::from_millis(500)`。

---

### M9: tray plugin cfg 合并简化

**文件**: `tauri/crates/tauri/src/app.rs`

**问题**: tray plugin 初始化有两个分开的 cfg gate：
- `#[cfg(all(desktop, feature = "tray-icon", not(target_env = "ohos")))]` — 非 OHOS desktop
- `#[cfg(all(target_env = "ohos", feature = "tray-icon"))]` — OHOS

两块代码完全相同（都调用 `tray::plugin()`），OHOS 不需要特殊 cfg gate，因为 `cfg(desktop)` 在 OHOS desktop 上也为 true（由 build.rs 设置）。

**修改**: 合并为单一 `#[cfg(all(desktop, feature = "tray-icon"))]`。

---

### cfg 简化: `any(desktop, target_env = "ohos")` → `cfg(desktop)`

**文件**: `tauri/crates/tauri/src/window/plugin.rs`

**问题**: `current_monitor`、`primary_monitor`、`available_monitors` 三个命令及 `mod desktop_commands` 的 cfg 为 `#[cfg(any(desktop, target_env = "ohos"))]`。OHOS desktop 上 `cfg(desktop)` 已为 true，不需要单独 exemption。

**修改**: 改为 `#[cfg(desktop)]`。

---

### cfg 保留: `any(mobile, target_env = "ohos")` 不变

**文件**: `tauri/crates/tauri/src/lib.rs:78`, `tauri/crates/tauri/src/plugin.rs:33`, `tauri/crates/tauri/src/plugin/mobile.rs`, `tauri/crates/tauri-runtime-wry/src/window/mod.rs:57`, `tauri/crates/tauri-runtime-wry/src/monitor/mod.rs:32`

**说明**: 以下位置的 `any(mobile, target_env = "ohos")` 必须保留，因为 OHOS desktop 上 `cfg(mobile)` 为 false，但这些功能 OHOS desktop 也需要：
- `mobile_entry_point`（OHOS 入口函数）
- `plugin/mobile` 模块（移动端插件注册，OHOS desktop 也用）
- `WindowExt` / `MonitorExt` stubs（OHOS 需要这些 trait 的空实现）

---

## Bug 回退类 — 之前错误修复的纠正

### C2 回退: tray event thread 不应 break on recv error

**文件**: `tray-icon/src/platform_impl/ohos/event.rs`

**问题**: 曾添加 `SHUTDOWN` AtomicBool + `else { break }` on recv errors + `EVENT_THREAD_STARTED.store(false)` on exit，意图让 tray event thread 可优雅退出。但 `else { break }` 在 TSFN channel 的 transient recv error 时会导致线程永久退出，**后续所有 tray/menu 交互全部失效**（最大化、checked items、predefined actions 等）。

**修改**:
- 删除 `SHUTDOWN` AtomicBool
- 删除 `else { break }` 分支
- 删除 `EVENT_THREAD_STARTED.store(false)` on exit
- 恢复原始的无限循环 + `Ordering::Relaxed`
- 删除 `shutdown_event_thread()`，从 `Drop` impl 中移除调用

---

### M4 回退: make_template alpha 预合成是错误的

**说明**: 即上面的 M6。make_template 将 icon RGB 替换为模板色是错误的——OHOS StatusBar 系统自行做白/黑背景 alpha 合成，传入原始 RGBA 才是正确的。

---

## Menu check item 功能实现

### Menu check item toggle（跨线程 Arc<AtomicBool>）

**文件**: `muda/src/platform_impl/ohos/mod.rs`

**背景**: Tray 的 check item 通过 `MENU_METADATA.check_state: HashMap<String, bool>` 存储 checked 状态，点击后 toggle bool + rebuild StatusBar 菜单让 UI 立即反映变化。Menu popup 的 check item 也需要类似机制。

**实现**:
- 添加 `CHECK_ITEMS` 全局: `Lazy<Mutex<HashMap<String, Arc<AtomicBool>>>>`，存储 check item ID → checked state
- `MenuChild.checked` 从 `Option<Rc<AtomicBool>>` 改为 `Option<Arc<AtomicBool>>`（跨线程共享）
- popup 前调用 `collect_check_items()` 递归收集所有 check item 的 ID 和 `Arc<AtomicBool>` 到 `CHECK_ITEMS` map
- event listener 收到 menu_id 时查 `CHECK_ITEMS`，用 `store(!old, Ordering::Release)` 翻转状态
- **不重建菜单**：popup 是瞬态的，点击后消失，下次 popup 自然从 AtomicBool 读取新值

**与 Tray check 的差异对比**:

| 特性 | Tray check | Menu check |
|------|-----------|------------|
| 存储 | `MENU_METADATA.check_state: HashMap<String, bool>` | `CHECK_ITEMS: HashMap<String, Arc<AtomicBool>>` |
| 点击后 | toggle bool + rebuild StatusBar 菜单 | toggle AtomicBool，不重建 |
| UI 更新 | 立即（常驻菜单需实时反映） | 下次 popup 时（瞬态菜单点击后消失） |
| 事件发送 | TrayIconEvent | MenuEvent |

---

## 其他已知限制（本轮不做修改）

- `TauriMenu.ets` 是死代码，保留不删
- `StatusBarMenuItemOptions` Clone 时 icon=None，已知限制
- `Box::leak` in `helper/webview.rs:349`，轻微泄漏，保留
- `supports_multiple_windows` OHOS 上始终返回 false，故意设计
- OHOS desktop 测试：128 个测试，124 通过，4 失败（Channel 性能、http/autostart/clipboard 未注册——均为已知遗留问题）

---

## 审计验证结果

### 编译验证

| 仓库 | 目标 | 结果 | 备注 |
|------|------|------|------|
| muda | `aarch64-unknown-linux-ohos` | ✅ 通过 | 6 warnings（unused methods，已知） |
| tray-icon | `aarch64-unknown-linux-ohos` | ✅ 通过 | 2 warnings（unused methods） |
| openharmony-ability | `aarch64-unknown-linux-ohos` | ✅ 通过 | — |
| tauri | `aarch64-unknown-linux-ohos` | ✅ 通过 | tauri crate 本身无错误 |
| tauri-runtime-wry | `aarch64-unknown-linux-ohos` | ❌ 预存错误 | openharmony-ability 模块重构导致 wry 找不到 WebView 类型，与本次审计无关 |

### 代码审查发现

#### 问题 1: muda `start_event_listener` 缩进错误（低优先级）

**文件**: `muda/src/platform_impl/ohos/mod.rs:473-478`

```rust
        }
});      // ← 缩进错误，应为 `    });`
    }

    pub fn init_menu_event_listener() {  // ← 缩进错误，应为顶层
        start_event_listener();
    }
```

**影响**: 纯格式问题，编译正确，不影响功能。`});` 关闭 `std::thread::spawn`，`}` 关闭 `start_event_listener()`，`pub fn init_menu_event_listener()` 是独立顶层函数。

**建议**: 下次格式化时修正缩进。

#### 问题 2: muda 文件末尾缺少换行符

**文件**: `muda/src/platform_impl/ohos/mod.rs` 最后一行无 `\n`

**影响**: 无功能影响，但不符合 POSIX 规范，git diff 会显示 `\ No newline at end of file`。

#### 问题 3: check item toggle 的 load/store 非原子操作

**文件**: `muda/src/platform_impl/ohos/mod.rs:465-466`

```rust
let old = checked.load(Ordering::Relaxed);
checked.store(!old, Ordering::Release);
```

**影响**: 如果两个线程同时 toggle 同一个 check item，可能丢失一次翻转。但实际场景中 menu event listener 是单线程（单一 `recv()` 循环），不会并发 toggle 同一 item，所以安全。若未来改为多线程消费，应改用 `fetch_xor(true, Ordering::AcqRel)` 或 CAS 循环。

**建议**: 当前安全，无需修改。

#### 问题 4: `collect_check_items` 每次 popup 都 clear + rebuild

**文件**: `muda/src/platform_impl/ohos/mod.rs:433-438`

每次 `popup()` 调用都会 `guard.clear()` 然后重新收集。如果菜单结构未变，这是冗余操作。

**影响**: 性能微乎其微（菜单项通常 <50 个），且保证了 CHECK_ITEMS 始终与当前菜单结构同步。设计正确。

**建议**: 无需修改。

### 跨平台安全性确认

| 修改 | 隔离机制 | 确认 |
|------|---------|------|
| C1: popup error propagation | `#[cfg(target_env = "ohos")]` 分支内 | ✅ 不影响其他平台 |
| C3: UnsafeSend 注释 | 仅注释变更 | ✅ 无代码影响 |
| C5: handler 注册简化 | openharmony-ability crate（仅 OHOS 编译） | ✅ 不影响其他平台 |
| L1: Arc 移除 | tray-icon ohos mod（`#[cfg]` 隔离） | ✅ 不影响其他平台 |
| L5: log 级别 | 各仓 OHOS 代码路径 | ✅ 不影响其他平台 |
| L9: Error::Unsupported 移除 | `#[cfg(target_env = "ohos")]` 变体 | ✅ 不影响其他平台 |
| M1: set_title 实现 | `#[cfg(target_env = "ohos")]` 分支 | ✅ 不影响其他平台 |
| M3: from_rgba 验证 | tray-icon ohos icon（`#[cfg]` 隔离） | ✅ 不影响其他平台 |
| M6: make_template 删除 | tray-icon ohos icon（`#[cfg]` 隔离） | ✅ 不影响其他平台 |
| M8: popup timeout 500ms | `#[cfg(target_env = "ohos")]` 分支内 | ✅ 不影响其他平台 |
| M9: tray cfg 合并 | `cfg(desktop)` 包含 OHOS desktop | ✅ 语义等价 |
| cfg(desktop) 简化 | window/plugin.rs | ✅ OHOS desktop 上 `cfg(desktop)=true`，语义等价 |
| check item toggle | muda ohos mod（`#[cfg]` 隔离） | ✅ 不影响其他平台 |

### 总结

本轮审计优化修改**正确且安全**：
- 所有修改均通过 `#[cfg(target_env = "ohos")]` 或 OHOS 专用 crate 隔离，不影响 Windows/macOS/Linux
- 编译验证通过（muda、tray-icon、tauri crate 均无错误）
- 发现 3 个低优先级格式/风格问题，不影响功能
- `tauri-runtime-wry` 编译错误为预存问题（openharmony-ability 模块重构导致），与本次审计无关
# Tray 模块 OHOS 适配 - 总体设计

> 版本：v2.0
> 更新时间：2026-05-20
> 目标：为 Tauri tray 模块提供 OpenHarmony/HarmonyOS 平台支持

---

## 一、适配目标

### 1.1 需要支持的接口

根据 Tauri tray 模块定义，本轮适配需支持以下 4 个核心接口：

| 接口 | 职责 | OHOS 支持度 |
|------|------|-------------|
| `TrayIconBuilder` | 构建托盘图标，配置图标/菜单/提示 | ✓ 完整支持 |
| `TrayIconEvent` | 托盘事件类型，提供事件接收机制 | ✓ 部分支持（仅 Click） |
| `MouseButton` | 鼠标按钮枚举 | ✓ 部分支持（Left/Right） |
| `MouseButtonState` | 鼠标按钮状态 | ✓ 固定为 Up |

详细接口定义见 [api-to-support.md](./api-to-support.md)。

### 1.2 OHOS 平台限制

| 限制项 | 说明 | 影响范围 |
|--------|------|----------|
| 设备类型 | 仅 **PC/2in1** 设备支持 statusBarManager | 移动设备无托盘功能 |
| API 版本 | statusBarManager 起始版本 **5.0.0(12)** | 低版本系统不支持 |
| 单图标限制 | 同一应用只能添加一个状态栏图标 | 不支持多 TrayIcon |
| 事件类型 | 仅支持左键点击图标、右键点击菜单项 | 不支持 DoubleClick/Enter/Move/Leave |
| hoverTips | 需要 **6.0.2(22)** 起才支持 | 低版本 set_tooltip 静默失败 |

---

## 二、架构设计

### 2.1 分层架构

```
┌─────────────────────────────────────────────────────────────┐
│  Layer 4: Application (tauri::tray)                         │
│    ├── TrayIconManager                                       │
│    ├── Event Listener                                        │
│    └── JS API (@tauri-apps/api/tray)                        │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│  Layer 3: Cross-Platform Abstraction (tray-icon crate)      │
│    ├── TrayIconBuilder (公共接口)                            │
│    ├── TrayIcon                                              │
│    ├── TrayIconEvent                                         │
│    ├── MouseButton / MouseButtonState                        │
│    └── platform_impl::ohos (OHOS 实现)                       │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│  Layer 2: OHOS API Wrapper (openharmony-ability::statusbar) │
│    ├── add_to_status_bar()                                   │
│    ├── remove_from_status_bar()                              │
│    ├── update_status_bar_icon/menu()                         │
│    ├── update_hover_tips()                                   │
│    ├── register_icon_click_handler()                         │
│    ├── register_menu_click_handler()                         │
│    ├── icon_click_receiver()                                 │
│    └── menu_click_receiver()                                 │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│  Layer 1: OHOS SDK (@kit.DeskTopExtensionKit)               │
│    ├── statusBarManager.addToStatusBar()                     │
│    ├── statusBarManager.removeFromStatusBar()                │
│    ├── statusBarManager.updateStatusBarIcon()                │
│    ├── statusBarManager.updateStatusBarMenu()                │
│    ├── statusBarManager.updateStatusBarHoverTips()           │
│    ├── statusBarManager.on('statusBarIconClick')             │
│    └── statusBarManager.on('rightMenuClick')                 │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 实现阶段划分

| Phase | 模块 | 职责 | 设计文档 |
|-------|------|------|----------|
| **Phase 0** | muda OHOS 后端 | 为 muda crate 实现 OHOS 平台支持 | [impl/phase0-muda-ohos-design.md](./impl/phase0-muda-ohos-design.md) |
| **Phase 1** | OHOS StatusBar API 封装 | 封装 statusBarManager 为 Rust API | [impl/phase1-ohos-statusbar-design.md](./impl/phase1-ohos-statusbar-design.md) |
| **Phase 2** | TrayIconBuilder | 实现 tray-icon 的 OHOS platform_impl | [impl/phase2-TrayIconBuilder-design.md](./impl/phase2-TrayIconBuilder-design.md) |
| **Phase 3** | TrayIconEvent | 实现事件转发机制 | [impl/phase3-TrayIconEvent-design.md](./impl/phase3-TrayIconEvent-design.md) |
| **Phase 4** | StatusBar API 修正与增强 | 修正 createPixelMap、添加 set_ohos_app、修复死锁 | [impl/phase4-ohos-statusbar-fix-design.md](./impl/phase4-ohos-statusbar-fix-design.md) |
| **Phase 5** | 端到端测试 | 编写 auto 和 manual 测试用例 | [impl/phase5-tray-testing-design.md](./impl/phase5-tray-testing-design.md) |
| **Phase 6** | StatusBar TSFN 数据传递重构 | 将全局 Mutex 中转改为 TSFN 直接携带数据 | [impl/phase6-statusbar-tsfn-refactor-design.md](./impl/phase6-statusbar-tsfn-refactor-design.md) |
| **Phase 7** | Predefined/Check/Icon 支持 | 预定义动作、check toggle、icon 项 | [impl/phase7-predefined-check-icon-design.md](./impl/phase7-predefined-check-icon-design.md) |

### 2.3 muda OHOS 后端决策

**决策**：在 **muda crate** 中实现 OHOS 平台支持，而非在 tray-icon 中转换菜单。

**理由**：

| 维度 | muda 实现 | tray-icon 转换 |
|------|-----------|----------------|
| **架构一致性** | ✓ 符合 muda 现有架构（windows/macos/gtk/ohos） | ✗ tray-icon 特有实现 |
| **复用性** | ✓ tauri menu module、tray-icon 都可使用 | ✗ 仅 tray-icon 可用 |
| **维护性** | ✓ 菜单逻辑集中一处 | ✗ 维护分散 |
| **事件统一** | ✓ 使用 muda::MenuEvent 现有机制 | ✗ 需额外映射 |

**muda 使用范围**：
- `tauri::tray` → tray-icon → muda (托盘右键菜单)
- `tauri::menu` → muda (Window Menu Bar，OHOS 不支持但需 stub)

**OHOS 菜单能力限制**：
- ✓ 托盘右键菜单（statusBarManager）
- ✗ Window Menu Bar（OHOS 无传统桌面菜单栏）
- ✗ CheckMenuItem、IconMenuItem（OHOS 不支持，需 stub）
- ✗ accelerator（OHOS 不支持，需 stub）

---

## 三、数据流设计

### 3.1 创建流程

```
TrayIconBuilder::build()
        │
        │  1. 获取 OHOS Context (OHOS_APP)
        │
        │  2. Icon → StatusBarIcon (white/black PixelMap)
        │     ├── RGBA → Alpha blending with white bg
        │     ├── RGBA → Alpha blending with black bg
        │     └── 缩放至 24x24 vp
        │
        │  3. 构建 QuickOperation
        │     └── ability_name = "" (触发事件模式)
        │
        │  4. Menu → StatusBarMenuItem[]
        │     └── notifyOnly = true, menuCode 设置
        │
        │  5. 调用 add_to_status_bar()
        │
        │  6. 注册事件监听
        │     ├── register_icon_click_handler()
        │     └── register_menu_click_handler()
        │
        │  7. 启动事件转发线程
        │
        ▼
    TrayIcon 实例
```

### 3.2 事件流程

```
┌─────────────────────────────────────────────────────────────┐
│  用户操作                                                    │
│    ├── 左键点击图标                                          │
│    └── 右键点击菜单项                                        │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│  OHOS statusBarManager                                      │
│    ├── statusBarIconClick → { iconClickType: "leftClick" }  │
│    └── rightMenuClick → { menuCode: "xxx" }                 │
└─────────────────────────────────────────────────────────────┘
                          │
                          │ NAPI callback
                          ▼
┌─────────────────────────────────────────────────────────────┐
│  openharmony-ability::statusbar                             │
│    ├── StatusBarClickEvent::IconClick                       │
│    ├── StatusBarClickEvent::MenuClick                       │
│    ├── icon_click_receiver() (channel)                      │
│    └── menu_click_receiver() (channel)                      │
└─────────────────────────────────────────────────────────────┘
                          │
                          │ crossbeam channel
                          ▼
┌─────────────────────────────────────────────────────────────┐
│  tray-icon::platform_impl::ohos                             │
│    ├── 事件转发线程 (select! 监听两个 channel)              │
│    ├── convert_icon_click() → Click { button: Left }        │
│    ├── convert_menu_click() → Click { button: Right }       │
│    └── TrayIconEvent::send()                                │
└─────────────────────────────────────────────────────────────┘
                          │
                          │ TRAY_CHANNEL
                          ▼
┌─────────────────────────────────────────────────────────────┐
│  tauri::tray                                                │
│    ├── TrayIconEvent::receiver()                            │
│    ├── set_event_handler()                                  │
│    └── EventLoopMessage::TrayIconEvent                      │
└─────────────────────────────────────────────────────────────┘
                          │
                          │ IPC
                          ▼
┌─────────────────────────────────────────────────────────────┐
│  JS Application                                             │
│    └── listen('tray-icon-event', callback)                  │
└─────────────────────────────────────────────────────────────┘
```

---

## 四、接口映射对照

### 4.1 TrayIconBuilder → OHOS API

| TrayIconBuilder 方法 | OHOS API | 备注 |
|----------------------|----------|------|
| `build()` | `addToStatusBar()` | 创建图标 |
| `set_icon()` | `updateStatusBarIcon()` | 更新图标 |
| `set_menu()` | `updateStatusBarMenu()` | 更新菜单 |
| `set_tooltip()` | `updateStatusBarHoverTips()` | △ 部分：6.0.2(22) 起，hoverTips 非标准 tooltip |
| `set_title()` | **需重建** `addToStatusBar()` | 无 `updateQuickOperationTitle` API |
| `set_visible(false)` | `removeFromStatusBar()` | 移除图标 |
| `set_visible(true)` | `addToStatusBar()` | 重新创建 |
| `rect()` | **不支持** | 返回 None |

### 4.2 TrayIconEvent → OHOS 事件

| TrayIconEvent 类型 | OHOS 事件 | 触发条件 |
|---------------------|-----------|----------|
| `Click { button: Left }` | `statusBarIconClick` | 左键点击图标 |
| `Click { button: Right }` | `rightMenuClick` | 右键点击菜单项 |
| `DoubleClick` | **不支持** | 不发送 |
| `Enter` | **不支持** | 不发送 |
| `Move` | **不支持** | 不发送 |
| `Leave` | **不支持** | 不发送 |

### 4.3 MouseButton 映射

| MouseButton | OHOS 对应 | 来源 |
|-------------|-----------|------|
| `Left` | `leftClick` | statusBarIconClick.iconClickType |
| `Right` | `menuCode` | rightMenuClick.menuCode |
| `Middle` | **不支持** | 无中键概念 |

### 4.4 MouseButtonState 映射

| MouseButtonState | OHOS 对应 | 说明 |
|------------------|-----------|------|
| `Up` | 固定使用 | OHOS 只发送点击完成事件 |
| `Down` | **不支持** | 不区分按下/释放 |

---

## 五、关键设计决策

### 5.1 Context 管理

OHOS statusBarManager 需要 `common.Context` 参数，但 tray-icon crate 是独立模块，无法直接获取 context。

**决策**：使用全局 `OnceCell<OpenHarmonyApp>` 存储 context，由 tauri 初始化时调用 `set_ohos_app()` 设置。

```rust
static OHOS_APP: OnceCell<OpenHarmonyApp> = OnceCell::new();

pub fn set_ohos_app(app: OpenHarmonyApp) {
    OHOS_APP.set(app).expect("Already set");
}
```

### 5.2 图标转换策略

OHOS 要求提供 **white/black** 两个 PixelMap：
- white：深色壁纸下展示（建议纯白色）
- black：浅色壁纸下展示（建议纯黑色）

**决策**：对原始 RGBA 图标进行 Alpha blending：

```rust
// 与白色背景混合
let white_rgba = blend_rgba_with_background(rgba, [255, 255, 255]);
// 与黑色背景混合
let black_rgba = blend_rgba_with_background(rgba, [0, 0, 0]);
```

### 5.3 事件模式触发条件

OHOS 默认行为是左键点击图标弹出 StatusBarViewExtensionAbility。

**决策**：设置 `ability_name = ""`（空字符串）触发自定义事件模式，使 statusBarIconClick 事件生效。

### 5.4 右键菜单事件触发条件

OHOS 右键菜单点击默认跳转 Ability，不发送事件。

**决策**：设置 `notifyOnly = true` + `menuCode`，使 rightMenuClick 事件生效。

### 5.5 版本兼容策略

| API | 版本要求 | 低版本处理 |
|-----|----------|------------|
| `addToStatusBar` | 5.0.0(12) | 不支持，编译时检查 |
| `updateHoverTips` | 6.0.2(22) | 静默失败 `.ok()` |
| `statusBarIconClick` | 5.0.2(14) | 不支持，事件不触发 |

---

## 六、风险与限制

### 6.1 技术风险

| 风险项 | 描述 | 应对措施 | 状态 |
|--------|------|----------|------|
| PixelMap 创建 | Rust 通过 NAPI 创建 OHOS PixelMap | ArkTS 侧 createPixelMapSync + writeBufferToPixelsSync | ✅ 已修正 |
| NAPI 回调线程 | 事件回调在非主线程 | 使用 ThreadsafeFunction | ✅ 已验证 |
| muda menu 依赖 | tray-icon 依赖 muda crate | **在 muda 中实现 OHOS 后端**（Phase 0） | ✅ 已完成 |
| Icon 内部结构 | 无法直接访问 RGBA 数据 | 扩展 PlatformIcon 添加 rgba 字段 | ✅ 已完成 |
| `run_item_main_thread!` 死锁 | OHOS 上主线程同步等待导致应用 freeze | 宏级别 `#[cfg(target_env = "ohos")]` 直接执行路径 | ✅ 已修复 |
| `set_ohos_app()` 未初始化 | tray-icon 的 `OHOS_APP OnceCell` 为空导致 panic | 在 tauri 初始化时调用 `set_ohos_app()` | ✅ 已修复 |
| TSFN 并发竞争 | 全局 Mutex 中转模式下快速调用交叉写入 | Phase 6: TSFN 泛型参数直接携带数据 | ✅ 已修复 |
| IPC 回调 Mutex 死锁 | webview.rs 中 cbs Mutex 持锁时间过长 | 复制引用后立即释放锁 | ✅ 已修复 |

### 6.2 功能限制

| 限制项 | 影响 | 用户须知 |
|--------|------|----------|
| 单 TrayIcon | 不支持多图标 | 文档说明限制 |
| 无位置信息 | `position` 固定 (0, 0) | 文档说明 |
| 无尺寸信息 | `rect` 固定 default | 文档说明 |
| 无双击事件 | `DoubleClick` 不触发 | 与 Linux 一致 |
| 无悬停事件 | `Enter/Move/Leave` 不触发 | 与 Linux 一致 |
| 右键点击图标无事件 | 只弹出菜单 | 仅菜单项点击有事件 |

### 6.3 设备限制

| 限制项 | 说明 |
|--------|------|
| 仅 PC/2in1 | 移动设备不支持 statusBarManager |
| API 5.0.0+ | 低版本 HarmonyOS 不支持 |

---

## 七、实现路线图

### 7.0 验证方式概览

本项目使用两种验证方式，优先使用 **auto**，不行才用 **manual**：

| 验证方式 | 适用场景 | 工具/Skill | 测试分类 |
|----------|----------|------------|----------|
| **Rust UT** | 纯逻辑函数（参数校验、转换计算） | `ohos-rust-ut` skill | `#[test]` |
| **auto** | API 返回值、类型、方法存在性 | `frontend-api-testing` skill | `category: 'auto'` |
| **manual** | 视觉效果、用户交互、事件触发 | `frontend-api-testing` skill | `category: 'manual'` |

**auto vs manual 判断原则**：

| 判断依据 | auto | manual |
|----------|------|--------|
| 有返回值可断言？ | ✓ 使用 | - |
| 无查询 API 验证状态？ | - | ✓ 使用 |
| 需用户操作触发？ | - | ✓ 使用 |

**tray 模块测试分类示例**：

```typescript
// auto: 验证返回值和类型
{
  name: '@tauri-apps/plugin-tray.TrayIcon.new',
  category: 'auto',
  async fn() {
    const tray = await TrayIcon.new({ icon });
    assert(tray !== undefined, 'tray returned');      // ✓ 有返回值
    assert(typeof tray.id === 'string', 'id is string'); // ✓ 类型可断言
    tray.destroy();
  },
}

// manual: 验证视觉效果（无返回值、无查询 API）
// 用户操作：查看状态栏是否显示图标
```

**Rust UT 特点**：
- 用于 `#[cfg(target_env = "ohos")]` 门控代码
- 交叉编译到 OHOS target，在设备上执行
- 只能写纯逻辑测试（不能用 mock runtime）

**端到端测试特点**：
- `auto`：纯函数验证，自动执行，报告写入 `test-report.json`
- `manual`：需用户确认，通过 `wrapManual()` 包装，console log 自动捕获到 `console-log.txt`

### 7.1 Phase 0：muda OHOS 后端

**目标**：为 muda crate 实现 OHOS 平台支持。

**交付物**：
- `muda/src/platform_impl/ohos/` 模块

**实现范围**：

| 实现项 | OHOS API | 状态 |
|--------|----------|------|
| `Menu` | statusBarManager.publish() | 基础实现 |
| `MenuItem` | StatusBarMenuItem | 基础实现 |
| `Submenu` | StatusBarMenuItem.subMenu | 基础实现 |
| `Separator` | 空标题 + disabled | 基础实现 |
| `ContextMenu` trait | 菜单显示/隐藏 | 基础实现 |
| `CheckMenuItem` | ✗ 不支持 | stub（忽略或渲染为文本） |
| `IconMenuItem` | ✗ 不支持 | stub（忽略图标） |
| `accelerator` | ✗ 不支持 | stub（忽略快捷键） |

**验证方式**：

| 验证项 | 方式 | 说明 |
|--------|------|------|
| Menu.new() 返回值 | **auto** | 验证 menu.id 存在 |
| MenuItem.text() 返回值 | **auto** | 验证文本正确 |
| 托盘菜单显示 | **manual** | 需人工确认右键菜单可见 |
| 菜单项点击事件 | **manual** | 需用户点击菜单项 |

### 7.2 Phase 1：底层 API 封装

**目标**：封装 statusBarManager 为 Rust API。

**交付物**：
- `openharmony-ability/crates/ability/src/statusbar/` 模块

**验证方式**：

| 验证项 | 方式 | 说明 |
|--------|------|------|
| 参数校验逻辑 | **Rust UT** | `validate.rs` 纯函数测试 |
| TrayIcon.new() 返回值 | **auto** | 验证 tray.id 存在且类型正确 |
| API 方法存在性 | **auto** | 验证 setIcon/setToolTip/destroy 方法存在 |
| 状态栏显示图标 | **manual** | 无查询 API，需人工确认可见 |
| 点击事件触发 | **manual** | 需用户点击，无法程序模拟 |

### 7.3 Phase 2：TrayIconBuilder 实现

**目标**：实现 tray-icon crate 的 OHOS platform_impl。

**交付物**：
- `tray-icon/src/platform_impl/ohos/mod.rs`
- `tray-icon/src/platform_impl/ohos/icon.rs`

**验证方式**：

| 验证项 | 方式 | 说明 |
|--------|------|------|
| 图标转换逻辑 | **Rust UT** | `icon.rs` 纯函数（缩放、混合） |
| TrayIconBuilder.build() | **auto** | 验证返回 TrayIcon 对象 |
| 图标显示/变化 | **manual** | 无查询 API，需人工确认 |
| set_visible 效果 | **manual** | 需人工确认图标消失/重现 |

### 7.4 Phase 3：TrayIconEvent 实现

**目标**：实现事件转发机制。

**交付物**：
- 事件转发线程
- TrayIconId 管理机制

**验证方式**：

| 验证项 | 方式 | 说明 |
|--------|------|------|
| 事件转换函数 | **Rust UT** | `event.rs` 纯函数测试 |
| TrayIconEvent.listen() | **auto** | 验证返回 unlisten 函数 |
| 事件触发和数据 | **manual** | 需用户点击，验证 event 数据 |
| button_state 固定为 Up | **manual** | 验证事件数据字段 |

### 7.5 集成测试

**目标**：验证 tauri tray 模块完整功能。

**验证流程**：
```
1. Rust UT 验证纯逻辑
   └── bash run-ut.sh <filter>

2. 端到端测试验证 API
   └── bash build-ohos.sh
   └── bash sign-and-install.sh
   └── 等待测试完成

3. 拉取测试报告
   └── cmd.exe /c "hdc file recv ... test-report.json"
   └── cmd.exe /c "hdc file recv ... console-log.txt"

4. 分析报告确认通过
```

**验证项**：
- [ ] test-report.json 中 tray 相关测试全部 pass
- [ ] console-log.txt 显示正确的事件数据
- [ ] JS API 正常工作
- [ ] 事件正确传递到前端

---

## 八、代码结构

### 8.1 新增文件

```
muda/src/platform_impl/ohos/
├── mod.rs          # OHOS 平台入口（Menu, MenuChild）
├── menu.rs         # Menu 实现
├── menu_child.rs   # MenuChild 实现（MenuItem, Submenu, etc）
├── context.rs      # ContextMenu trait 实现
└── event.rs        # MenuEvent OHOS 处理

openharmony-ability/crates/ability/src/statusbar/
├── mod.rs          # 模块入口
├── types.rs        # StatusBarItem/Icon/Menu 类型
├── manager.rs      # API 封装函数
├── event.rs        # 事件处理
└── validate.rs     # 参数校验

tray-icon/src/platform_impl/ohos/
├── mod.rs          # TrayIcon 实现
├── icon.rs         # 图标转换
└── event.rs        # 事件转发
```

### 8.2 修改文件

```
muda/Cargo.toml                     # 添加 OHOS target 配置
muda/src/platform_impl/mod.rs       # 添加 ohos 模块选择
muda/src/lib.rs                     # 添加 cfg(target_env = "ohos")

tray-icon/Cargo.toml               # 添加 OHOS 依赖
tray-icon/src/lib.rs               # 添加 cfg(target_env = "ohos")
tray-icon/src/platform_impl/mod.rs # 添加 ohos 模块

tauri/Cargo.toml                   # 修改 muda/tray-icon 路径依赖
tauri/src/app/mod.rs               # 添加 set_ohos_app() 调用
```

---

## 九、参考资料

### 9.1 OHOS 官方文档

| 文档 | 内容 | 链接 |
|------|------|------|
| statusBarManager API | 完整 API 定义 | [reference/status_bar_api.md](./reference/status_bar_api.md) |
| StatusBarViewExtensionAbility | 左键弹窗 Ability | [reference/status_bar_view_extension_ability.md](./reference/status_bar_view_extension_ability.md) |
| 托盘方案示例 | 后台保活、事件处理 | [reference/tray-reference.md](./reference/tray-reference.md) |

### 9.2 设计文档

| Phase | 文档 |
|-------|------|
| Phase 0 | [impl/phase0-muda-ohos-design.md](./impl/phase0-muda-ohos-design.md) |
| Phase 1 | [impl/phase1-ohos-statusbar-design.md](./impl/phase1-ohos-statusbar-design.md) |
| Phase 2 | [impl/phase2-TrayIconBuilder-design.md](./impl/phase2-TrayIconBuilder-design.md) |
| Phase 3 | [impl/phase3-TrayIconEvent-design.md](./impl/phase3-TrayIconEvent-design.md) |
| Phase 4 | [impl/phase4-ohos-statusbar-fix-design.md](./impl/phase4-ohos-statusbar-fix-design.md) |
| Phase 5 | [impl/phase5-tray-testing-design.md](./impl/phase5-tray-testing-design.md) |
| Phase 6 | [impl/phase6-statusbar-tsfn-refactor-design.md](./impl/phase6-statusbar-tsfn-refactor-design.md) |
| Phase 7 | [impl/phase7-predefined-check-icon-design.md](./impl/phase7-predefined-check-icon-design.md) |

### 9.3 验证工具 Skills

| Skill | 用途 | 主要命令 |
|-------|------|----------|
| `ohos-rust-ut` | Rust 单元测试（OHOS target） | `bash run-ut.sh <filter>` |
| `frontend-api-testing` | 端到端测试编写 | 修改 `plugins.ts` |
| `ohos-build` | OHOS 构建和部署 | `bash build-ohos.sh` + `sign-and-install.sh` |

**Skill 文档路径**：
- `../../../.claude/skills/ohos-rust-ut/SKILL.md`
- `../../../.claude/skills/frontend-api-testing/SKILL.md`
- `../../../.claude/skills/ohos-build/SKILL.md`

---

## 十、附录

### 10.1 OHOS API 版本一览

| API | 起始版本 |
|-----|----------|
| addToStatusBar | 5.0.0(12) |
| removeFromStatusBar | 5.0.0(12) |
| updateStatusBarIcon | 5.0.0(12) |
| updateStatusBarMenu | 5.0.0(12) |
| updateQuickOperationHeight | 5.0.0(12) |
| statusBarIconClick | 5.0.2(14) |
| rightMenuClick | 5.0.2(14) |
| updateStatusBarHoverTips | 6.0.2(22) |
| updateStatusBarMenuItem | 6.1.1(24) |
| updateStatusBarSubMenuItem | 6.1.1(24) |

### 10.2 错误码一览

| 错误码 | 说明 |
|--------|------|
| 401 | 参数检查失败 |
| 1010710001 | PixelMap 尺寸超限 |
| 1010710002 | 菜单项数量超限 |
| 1010710003 | API 调用过于频繁 |
| 1010710004 | 无窗口在前台时无法删除 |
| 1010710005 | 字符串长度超限 |
| 1010710006 | 菜单项未找到 |
| 1010710007 | menuCode 不唯一 |
| 1010720001 | menuAction 和 subMenu 同时缺省 |
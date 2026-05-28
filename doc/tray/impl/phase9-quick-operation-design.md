# Phase 9: QuickOperation 设计

## Context

对照 tauri tray 完整 API 和四平台实现（Windows/macOS/Linux/OHOS），审计 OHOS desktop tray 的能力覆盖情况，识别缺失项，并给出 QuickOperation（OHOS 独有左键弹窗能力）的嵌入方案。

---

## 一、Tray 功能全景

### 1.1 构建与配置（TrayIconBuilder / TrayIconAttributes）

| 能力 | Builder 方法 | Attributes 字段 | Windows | macOS | Linux | OHOS |
|------|-------------|----------------|---------|-------|-------|------|
| 图标 | `with_icon()` | `icon: Option<Icon>` | ✅ | ✅ | ✅ | ✅ |
| 菜单 | `with_menu()` | `menu: Option<Box<ContextMenu>>` | ✅ | ✅ | ✅ | ✅ |
| 提示 | `with_tooltip()` | `tooltip: Option<String>` | ✅ | ✅ | ❌ | ✅ |
| 标题 | `with_title()` | `title: Option<String>` | ❌ | ✅ | ✅ | ✅ |
| 图标模板 | `with_icon_as_template()` | `icon_is_template: bool` | — | ✅ | — | — |
| 临时目录 | `with_temp_dir_path()` | `temp_dir_path: Option<PathBuf>` | — | — | ✅ | — |
| 左键菜单 | `with_menu_on_left_click()` | `menu_on_left_click: bool` | ✅ | ✅ | ❌ | — |
| 右键菜单 | `with_menu_on_right_click()` | `menu_on_right_click: bool` | ✅ | ✅ | ❌ | — |
| **QuickOperation** | `with_quick_operation()` | `quick_operation: Option<QuickOperationConfig>` | — | — | — | **✅ 已实现** |

### 1.2 运行时操作（TrayIcon setter）

| 能力 | 方法 | Windows | macOS | Linux | OHOS |
|------|------|---------|-------|-------|------|
| 设置图标 | `set_icon()` | ✅ | ✅ | ✅ | ✅ |
| 设置菜单 | `set_menu()` | ✅ | ✅ | ✅ | ✅ |
| 设置提示 | `set_tooltip()` | ✅ | ✅ | ❌ | ✅ |
| 设置标题 | `set_title()` | ❌ | ✅ | ✅ | ✅ |
| 显示/隐藏 | `set_visible()` | ✅ | ✅ | ✅ | ✅ |
| 图标模板 | `set_icon_as_template()` | — | ✅ | — | — |
| 临时目录 | `set_temp_dir_path()` | — | — | ✅ | — |
| 左键菜单 | `set_show_menu_on_left_click()` | ✅ | ✅ | ❌ | — (no-op) |
| 右键菜单 | `set_show_menu_on_right_click()` | ✅ | ✅ | ❌ | — (no-op) |
| 程序化显示菜单 | `show_menu()` | ✅ | ✅ | ❌ | ❌ 已文档标注不支持 |
| 图标位置 | `rect()` | ✅ | ✅ | ❌ | ❌ (None) |
| 原生句柄 | `window_handle()` | ✅ | — | — | — |
| 原生句柄 | `ns_status_item()` | — | ✅ | — | — |
| 原生句柄 | `app_indicator()` | — | — | ✅ | — |
| **QuickOperation** | `set_quick_operation()` | — | — | — | **✅ 已实现** |

### 1.3 事件（TrayIconEvent）

| 事件 | Windows | macOS | Linux | OHOS |
|------|---------|-------|-------|------|
| Click (Left, Up) | ✅ | ✅ | ❌ | ✅ |
| Click (Left, Down) | ✅ | ✅ | ❌ | ❌ API只通知完成 |
| Click (Right, Up) | ✅ | ✅ | ❌ | ✅ |
| Click (Right, Down) | ✅ | ✅ | ❌ | ❌ API只通知完成 |
| Click (Middle) | ✅ | ✅ | ❌ | ❌ 无middleClick |
| DoubleClick | ✅ | ❌ | ❌ | ❌ 无doubleClick |
| Enter | ✅ | ✅ | ❌ | ❌ 无hover |
| Move | ✅ | ✅ | ❌ | ❌ 无hover |
| Leave | ✅ | ✅ | ❌ | ❌ 无hover |
| position (坐标) | ✅ | ✅ | ❌ | ❌ API不传坐标 |
| rect (图标区域) | ✅ | ✅ | ❌ | ❌ 始终None |

---

## 二、OHOS 缺失项汇总

### 2.1 唯一功能性缺失：QuickOperation（左键弹窗）— ✅ 已实现

**原始现状**：`tray-icon/platform_impl/ohos/mod.rs` 中 QuickOperation 硬编码：
```rust
let quick_operation = openharmony_ability::statusbar::QuickOperation {
    ability_name: String::new(),   // 空 = 不弹窗，仅触发事件
    title: attrs.title.clone().unwrap_or("Tauri App".to_string()),
    height: 200,
    module_name: Some("entry".to_string()),
    loading_status: None,
};
```

**影响**：左键点击只触发 `statusBarIconClick` 事件回调，不弹出任何 UI。而 OHOS `StatusBarViewExtensionAbility` 原生支持左键弹出系统级面板（标题栏 + 自定义内容区），面板在系统层渲染，应用最小化也能弹出。这是 Windows/macOS/Linux 都不具备的 OHOS 独有能力。

**其他平台左键行为对比**：
| 平台 | 左键行为 |
|------|---------|
| Windows | 弹出上下文菜单 (TrackPopupMenu) |
| macOS | 弹出 NSMenu 或 highlight 图标 |
| Linux | 弹出 GTK 菜单 |
| OHOS 实现后 | 弹出系统级面板 (StatusBarViewExtensionAbility) ✅ |

### 2.2 仅文档缺失 — ✅ 已修复

- `show_menu()` (`tray-icon/src/lib.rs`)：已添加 `"OHOS: Unsupported. statusBarManager has no API to programmatically trigger the menu."`

### 2.3 系统级不可实现（phase8 已确认）

DoubleClick / Enter / Move / Leave / MouseButton::Middle / MouseButtonState::Down / position / rect / NativeIcon — 均为 StatusBar API 系统级限制。

### 2.4 纯性能优化（暂不实现）

- `updateStatusBarMenuItem` (6.1.1(24))：单项更新 API，功能已被 `updateStatusBarMenu` 覆盖
- `updateQuickOperationHeight`：动态调整弹窗高度，依赖 QuickOperation 先实现

### 2.5 已确认无 bug

- `set_visible(true)` 不重新注册事件处理器：handler 在 `init_tray_tsfn()` 中注册一次后始终活跃，`removeFromStatusBar` 移除图标后系统不会触发事件，`addToStatusBar` 重新添加后正常触发。无需修复。

---

## 三、OHOS QuickOperation 平台架构

### 3.1 QuickOperation 官方结构体（statusBarManager API）

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `title` | string | 是 | 弹窗标题栏文字，超长显示省略号 |
| `height` | number | 是 | 弹窗自定义区域高度 (vp, >0) |
| `abilityName` | string | 是 | ExtensionAbility 名称。**空字符串 = 不弹窗，仅触发 `statusBarIconClick` 事件** |
| `moduleName` | string | 可选 | Ability 所在模块名，默认空字符串 |
| `loadingStatus` | boolean | 可选 | 控制是否显示加载动效（6.0.0(20)+ 版本支持） |

### 3.2 StatusBarViewExtensionAbility 生命周期

QuickOperation 弹窗内容由应用自定义的 `StatusBarViewExtensionAbility` 承载，继承自 `UIExtensionAbility`。生命周期方法：

| 方法 | 签名 | 说明 |
|------|------|------|
| `onCreate()` | `void` | Ability 创建时回调，可做初始化 |
| `onSessionCreate(want, session)` | `void` | **核心回调** — session 创建完成，在此加载页面 |
| `onForeground()` | `void` | Ability 转前台 |
| `onBackground()` | `void` | Ability 转后台 |
| `onSessionDestroy()` | `void` | 会话销毁，不能再使用 session（无参数） |
| `onDestroy()` | `void \| Promise<void>` | Ability 销毁，可做资源清理 |

### 3.3 UIExtensionContentSession 方法

`onSessionCreate` 中收到的 `session` 对象提供以下方法：

| 方法 | 说明 |
|------|------|
| `loadContent(path: string)` | 按路径加载页面（如 `'pages/MyTrayPage'`） |
| `loadContentByName(name: string, storage?: LocalStorage)` | 按命名路由加载，可传 LocalStorage 共享数据 |
| `terminateSelf()` | **主动关闭弹窗**（仅关闭 QuickOperation session，不影响 tray 图标和应用主体） |
| `terminateSelfWithResult(result)` | 关闭弹窗并返回结果给宿主 |

> **实现发现**：`UIExtensionContentSession` 没有 `hide()` 方法。关闭弹窗必须使用 `terminateSelf()`。

### 3.4 want 参数来源（系统自动构造）

`onSessionCreate(want, session)` 中的 `want` 由**系统自动构造**，包含：
- `bundleName` — 当前应用包名
- `abilityName` — QuickOperation 中指定的 Ability 名
- `moduleName` — QuickOperation 中指定的模块名

**不支持自定义数据**：QuickOperation 没有 `parameters` 字段，无法在 `addToStatusBar` 时注入自定义数据让 want 携带。

### 3.5 Rust → ArkTS 弹窗页面的数据通信

由于弹窗运行在 `StatusBarViewExtensionAbility` 进程中，与应用主进程隔离，数据传递方式：

| 方式 | 说明 | 适用场景 |
|------|------|---------|
| **AppStorage** | 跨 Ability 共享状态 | 简单键值对、开关状态 |
| **Preferences** | 持久化存储 | 配置项、需要持久化的数据 |
| **commonEventManager** | 事件订阅/发布 | 事件通知、状态变更 |
| **RelationalStore** | 关系型数据库 | 复杂结构化数据 |

### 3.6 架构分层：Rust API 配"壳"，用户写"肉"

```
Rust 侧 (我们的 API)              用户 ArkTS 侧 (应用自行创建)
─────────────────────             ──────────────────────────
QuickOperationConfig {            module.json5:
  ability_name: "MyTrayAbility" ──→ "name": "MyTrayAbility"
  title: "My App"                   "type": "statusBarView"
  height: 300                     }
  module_name: Some("entry")
  loading_status: Some(false)     MyTrayAbility.ets:
}                                 ┌──────────────────────────────┐
                                  │ class MyTrayAbility extends   │
                                  │   StatusBarViewExtension      │
                                  │   Ability {                   │
                                  │                               │
                                  │   onSessionCreate(want,       │
                                  │       session) {              │
                                  │     session.loadContent(      │
                                  │       'pages/MyTrayPage'      │ ← 用户决定加载哪个页面
                                  │     )                         │
                                  │   }                           │
                                  │ }                             │
                                  └──────────────────────────────┘
                                            │
                                            ▼
                                  pages/MyTrayPage.ets:
                                  ┌──────────────────────────────┐
                                  │ @Entry @Component             │
                                  │ struct MyTrayPage {           │
                                  │   build() {                   │
                                  │     Column() { ... }          │ ← 用户自定义内容
                                  │   }                           │
                                  │ }                             │
                                  └──────────────────────────────┘
```

连接点就是 `ability_name`。系统层面的流程：
1. 用户左键点击 tray 图标
2. 系统根据 `abilityName` 查找 `module.json5` 中注册的 ExtensionAbility
3. 系统启动该 Ability，调用 `onSessionCreate(want, session)`
4. **用户在自己的 Ability 代码里**调用 `session.loadContent('pages/xxx')` 加载页面
5. 页面渲染在系统级弹窗中，高度由 `height` 控制

---

## 四、QuickOperation 实现方案

### 4.1 设计决策：嵌入现有接口

代码库已有平台专属功能的成熟模式：
- `TrayIconAttributes.temp_dir_path` — 文档标注 "Linux only"，其他平台 `let _ = path` 忽略
- `TrayIconAttributes.icon_is_template` — 文档标注 "macOS only"，其他平台 `let _ = is_template` 忽略

QuickOperation 遵循完全相同的模式：字段始终存在于 `TrayIconAttributes` 中，文档标注 "OHOS only"，非 OHOS 平台忽略。

### 4.2 对外接口（应用侧视角）

#### 新增公开类型

```rust
// tray-icon/src/lib.rs
/// QuickOperation 配置。**OHOS only**。
#[derive(Debug, Clone)]
pub struct QuickOperationConfig {
    /// 弹窗标题栏文字
    pub title: String,
    /// 弹窗内容区高度 (vp, >0)
    pub height: u32,
    /// StatusBarViewExtensionAbility 名称（module.json5 中注册）。
    /// 空字符串 = 禁用弹窗（左键仅触发事件）。
    pub ability_name: String,
    /// ExtensionAbility 所在模块名（默认当前模块）
    pub module_name: Option<String>,
    /// 控制是否显示加载动效（6.0.0(20)+ 版本支持）
    pub loading_status: Option<bool>,
}
```

#### Builder 构建时配置

```rust
let tray = TrayIconBuilder::new()
    .icon(icon)
    .menu(&menu)
    .quick_operation(QuickOperationConfig {  // OHOS only，其他平台 no-op
        title: "My App".into(),
        height: 300,
        ability_name: "MyTrayAbility".into(),
        module_name: Some("entry".into()),
        loading_status: None,
    })
    .build()?;
```

#### 运行时更新 / 禁用

```rust
// 更新配置
tray.set_quick_operation(Some(QuickOperationConfig {
    title: "Updated".into(), height: 400,
    ability_name: "MyTrayAbility".into(), module_name: None,
    loading_status: Some(true),
}));

// 禁用左键弹窗（回退为仅触发事件）
tray.set_quick_operation(None);
```

#### 应用侧 ArkTS 配置（应用自行创建）

```typescript
// 1. MyTrayAbility.ets — 继承 StatusBarViewExtensionAbility
import { StatusBarViewExtensionAbility } from "@kit.StatusBarExtensionKit";
import { UIExtensionContentSession, Want } from "@kit.AbilityKit";

export default class MyTrayAbility extends StatusBarViewExtensionAbility {
  onCreate(): void {
    // Ability 创建时初始化（可选）
  }

  onSessionCreate(want: Want, session: UIExtensionContentSession): void {
    // 核心：加载自定义页面
    // want 由系统自动构造，包含 bundleName/abilityName/moduleName
    // 不支持自定义数据，需通过 AppStorage/Preferences 等间接传递
    session.loadContent('pages/MyTrayPage');
  }

  onSessionDestroy(): void {
    // 会话销毁时清理（可选，无参数）
  }

  onDestroy(): void {
    // Ability 销毁时清理（可选）
  }
}

// 2. pages/MyTrayPage.ets — 自定义面板内容（任意 ArkUI 组件）
@Entry
@Component
struct MyTrayPage {
  @State isPlaying: boolean = false

  build() {
    Column({ space: 12 }) {
      Text('Music Player').fontSize(16).fontWeight(FontWeight.Bold)
      Row({ space: 16 }) {
        Button('⏮').onClick(() => { /* prev */ })
        Button(this.isPlaying ? '⏸' : '▶')
          .onClick(() => { this.isPlaying = !this.isPlaying })
        Button('⏭').onClick(() => { /* next */ })
      }
    }
    .width('100%').height('100%')
    .justifyContent(FlexAlign.Center)
  }
}

// 3. module.json5 注册
"extensionAbilities": [{
  "name": "MyTrayAbility",
  "type": "statusBarView",
  "exported": true,
  "srcEntry": "./ets/MyTrayAbility.ets"
}]

// 4. main_pages.json 注册弹窗页面路由（必须，否则 loadContent 失败）
{ "src": ["pages/Index", "pages/MyTrayPage"] }
```

#### Rust → ArkTS 弹窗数据通信示例

由于 `want` 不支持自定义数据，应用需通过间接方式传递数据：

```typescript
// Rust 侧：通过 NAPI/TSFN 写入 AppStorage 或 Preferences
// （应用自定义实现，tauri 不提供内置桥接）

// ArkTS 侧：弹窗页面读取共享数据
@Entry
@Component
struct MyTrayPage {
  @StorageLink('tray_current_track') currentTrack: string = 'Unknown'
  @StorageLink('tray_is_playing') isPlaying: boolean = false

  build() {
    Column({ space: 8 }) {
      Text(this.currentTrack).fontSize(14)
      Row({ space: 16 }) {
        Button(this.isPlaying ? '⏸' : '▶')
          .onClick(() => {
            this.isPlaying = !this.isPlaying
            // 通知主进程状态变更
            AppStorage.setOrCreate('tray_is_playing', this.isPlaying)
          })
      }
    }
    .width('100%').height('100%')
    .justifyContent(FlexAlign.Center)
  }
}
```

### 4.3 内部实现（逐层数据流）

```
应用调用 set_quick_operation(config)
        │
        ▼
┌─ tauri TrayIcon<R> ─────────────────────────────────────┐
│  #[cfg(ohos)]     → self.inner.set_quick_operation(config) │
│  #[cfg(not ohos)] → let _ = config; (no-op)               │
└───────────────────────────┬───────────────────────────────┘
                            ▼
┌─ tray-icon TrayIcon (lib.rs) ────────────────────────────┐
│  #[cfg(ohos)]     → self.tray.borrow_mut().set_quick_operation(config) │
│  #[cfg(not ohos)] → let _ = config;                       │
└───────────────────────────┬───────────────────────────────┘
                            ▼
┌─ tray-icon OHOS platform (platform_impl/ohos/mod.rs) ────┐
│  1. self.attrs.quick_operation = config                   │
│  2. remove_from_status_bar(app)                           │
│  3. build_item_from_attrs(attrs)  ← 提取 QuickOperation 参数    │
│  4. add_to_status_bar(app, &item)                         │
└───────────────────────────┬───────────────────────────────┘
                            ▼
┌─ openharmony-ability statusbar manager ──────────────────┐
│  TSFN_ADD → ArkTS addToStatusBar(statusBarItem)          │
│    statusBarItem.quick_operation = {                      │
│      abilityName, title, height, moduleName,              │
│      loadingStatus                                        │
│    }                                                      │
└───────────────────────────────────────────────────────────┘
```

### 4.4 实现步骤

#### Step 1: 新增 `QuickOperationConfig` 类型 + `TrayIconAttributes` 字段

**文件**: `tray-icon/src/lib.rs`

1. 新增 `pub struct QuickOperationConfig { title, height, ability_name, module_name, loading_status }` 及 `Default` impl
2. `TrayIconAttributes` 添加字段：
   ```rust
   /// QuickOperation for left-click popup. **OHOS only**.
   pub quick_operation: Option<QuickOperationConfig>,
   ```
3. `Default` impl 添加 `quick_operation: None`

#### Step 2: `TrayIconBuilder::with_quick_operation()`

**文件**: `tray-icon/src/lib.rs` — `TrayIconBuilder` impl

```rust
/// Set QuickOperation for left-click popup. **OHOS only**.
pub fn with_quick_operation(mut self, config: QuickOperationConfig) -> Self {
    self.attrs.quick_operation = Some(config);
    self
}
```

无 `#[cfg]` 门控 — 与 `with_temp_dir_path()` / `with_icon_as_template()` 一致。

#### Step 3: `TrayIcon::set_quick_operation()` (lib.rs 层)

**文件**: `tray-icon/src/lib.rs` — `TrayIcon` impl

```rust
pub fn set_quick_operation(&self, config: Option<QuickOperationConfig>) {
    #[cfg(target_env = "ohos")]
    self.tray.borrow_mut().set_quick_operation(config);
    #[cfg(not(target_env = "ohos"))]
    let _ = config;
}
```

与 `set_icon_as_template()` 的 `#[cfg]` 模式完全一致。

#### Step 4: OHOS 平台层实现

**文件**: `tray-icon/src/platform_impl/ohos/mod.rs`

**改动 4a**：提取 `fn build_item_from_attrs(attrs: &TrayIconAttributes) -> Result<StatusBarItem>`

将 `new()` 中的 StatusBarItem 构建逻辑提取为独立方法。`attrs.quick_operation` 决定 QuickOperation 参数：
- `Some(config)` → 使用 config 中的 ability_name / title / height / module_name / loading_status
- `None` → 当前行为：ability_name 空字符串，title 取 attrs.title 或 "Tauri App"，height 200

供 `new()`、`set_title()`、`set_icon()`、`set_quick_operation()` 共用，消除重复代码。

**改动 4b**：`new()` 使用 `build_item_from_attrs()`

```rust
let item = Self::build_item_from_attrs(&attrs)?;
openharmony_ability::statusbar::add_to_status_bar(app, &item)
    .map_err(|e| crate::Error::OhosError(e.to_string()))?;
```

**改动 4c**：`set_quick_operation()` 实现

```rust
pub fn set_quick_operation(&mut self, config: Option<QuickOperationConfig>) {
    self.attrs.borrow_mut().quick_operation = config;
    if *self.is_visible.borrow() {
        let app = get_ohos_app();
        openharmony_ability::statusbar::remove_from_status_bar(app).ok();
        let item = Self::build_item_from_attrs(&self.attrs.borrow()).unwrap();
        openharmony_ability::statusbar::add_to_status_bar(app, &item).ok();
    }
}
```

**改动 4d**：`set_title()` / `set_icon()` 重构使用 `build_item_from_attrs()`

当前 `set_title()` 和 `set_icon()` 有独立的 remove+rebuild 逻辑，改为调用 `build_item_from_attrs()` 以统一。

#### Step 5: tauri `TrayIconBuilder<R>::quick_operation()`

**文件**: `tauri/crates/tauri/src/tray/mod.rs` — `TrayIconBuilder<R>` impl

```rust
/// Set QuickOperation for left-click popup. **OHOS only**.
pub fn quick_operation(mut self, config: tray_icon::QuickOperationConfig) -> Self {
    self.inner = self.inner.with_quick_operation(config);
    self
}
```

#### Step 6: tauri `TrayIcon<R>::set_quick_operation()`

**文件**: `tauri/crates/tauri/src/tray/mod.rs` — `TrayIcon<R>` impl

```rust
pub fn set_quick_operation(&self, config: Option<tray_icon::QuickOperationConfig>) {
    #[cfg(target_env = "ohos")]
    self.inner.set_quick_operation(config);
    #[cfg(not(target_env = "ohos"))]
    let _ = config;
}
```

#### Step 7: tauri tray plugin 命令注册 + ACL 权限

**文件**: `tauri/crates/tauri/src/tray/plugin.rs` + `tauri/crates/tauri/build.rs`

1. 新增 `fn set_quick_operation<R: Runtime>(app, rid, config) -> crate::Result<()>` 命令
2. 注册到 `generate_handler![...]` 列表
3. `build.rs` PLUGINS 数组 `core:tray` 条目添加 `("set_quick_operation", true)`

> **实现注意**：步骤 3 在原始设计中遗漏，导致前端调用返回 `Command plugin:tray|set_quick_operation not allowed by ACL`。所有新 tray plugin 命令都需要此注册。

#### Step 8: show_menu() 文档注释

**文件**: `tray-icon/src/lib.rs`

添加：`/// - **OHOS**: Unsupported. statusBarManager has no API to programmatically trigger the menu.`

---

## 五、验证方案

### 5.1 编译验证（无需设备）

```bash
# tray-icon 层
cargo check -p tray-icon

# tauri 层
cargo check -p tauri --features tray-icon

# OHOS 交叉编译
cargo check -p tray-icon --target aarch64-unknown-linux-ohos
```

### 5.2 Rust 单元测试

#### 新增测试用例（6 个）

| # | 测试名 | 文件 | 验证内容 |
|---|--------|------|----------|
| 1 | `quick_operation_config_default` | `tray-icon/src/lib.rs` | `QuickOperationConfig::default()` → `title` 空、`height` 200、`ability_name` 空、`module_name` None、`loading_status` None |
| 2 | `builder_with_quick_operation` | `tray-icon/src/lib.rs` | `TrayIconBuilder::new().with_quick_operation(config)` → `attrs.quick_operation` 为 Some，字段值正确 |
| 3 | `build_item_with_quick_operation` | `tray-icon/src/platform_impl/ohos/mod.rs` | `build_item_from_attrs(attrs)` 当 `attrs.quick_operation = Some(config)` → QuickOperation 字段与 config 一致 |
| 4 | `build_item_without_quick_operation` | `tray-icon/src/platform_impl/ohos/mod.rs` | `build_item_from_attrs(attrs)` 当 `attrs.quick_operation = None` → 默认值：ability_name 空、title 取 attrs.title 或 "Tauri App"、height 200 |
| 5 | `quick_operation_empty_title_falls_back_to_attrs_title` | `tray-icon/src/platform_impl/ohos/mod.rs` | config.title 空 → 回退到 attrs.title |
| 6 | `quick_operation_no_attrs_title_falls_back_to_default` | `tray-icon/src/platform_impl/ohos/mod.rs` | config.title 空 + attrs.title None → 回退到 "Tauri App" |

#### 回归测试（现有用例不受影响）

| 测试名 | 文件 | 验证内容 |
|--------|------|----------|
| `tray_icon_id::test::is_eq` | `lib.rs` | TrayIconId Eq 实现 |
| `test_regular_item_becomes_top_level_menu_item` | `ohos/mod.rs` | build_item_from_attrs() 重构不影响事件转换 |
| `test_submenu_becomes_item_with_sub_menu` | `ohos/mod.rs` | 同上 |
| `test_separators_filtered_out` | `ohos/mod.rs` | 同上 |
| `test_menu_json_deserialization_from_muda_format` | `ohos/mod.rs` | 同上 |
| `test_split_items_into_groups_at_separator` | `ohos/mod.rs` | 同上 |

#### 运行命令

```bash
# tray-icon lib UT（新增 2 + 现有 1 = 3 个）
cargo test -p tray-icon --lib

# tray-icon OHOS UT（新增 4 + 现有 5 = 9 个，需 OHOS 交叉编译环境）
# 通过 run-tests.sh desktop 在设备上执行

# 合计：新增 6 + 现有 6 = 12 个
```

### 5.3 API Demo 自动测试

#### JS API 变更（`packages/api/src/tray.ts`）

```typescript
// 新增类型
export interface QuickOperationConfig {
  title: string
  height: number
  abilityName: string
  moduleName?: string
  loadingStatus?: boolean
}

// TrayIconOptions 新增字段
export interface TrayIconOptions {
  // ... existing fields ...
  /** QuickOperation for left-click popup. **OHOS only**. */
  quickOperation?: QuickOperationConfig
}

// TrayIcon 新增方法
class TrayIcon {
  /** Set QuickOperation for left-click popup. **OHOS only**. */
  async setQuickOperation(config: QuickOperationConfig | null): Promise<void> {
    return invoke('plugin:tray|set_quick_operation', { rid: this.rid, config })
  }
}
```

#### 自动测试用例（3 个，追加到 `trayTests[]`）

| # | 测试名 | category | 文件 | 验证内容 |
|---|--------|----------|------|----------|
| 1 | `TrayIcon.setQuickOperation` | auto | `examples/api/src/lib/tests/tray.ts` | 调用 `setQuickOperation(config)` 不抛异常，invoke 正常返回 |
| 2 | `TrayIcon.setQuickOperation_null` | auto | 同上 | 调用 `setQuickOperation(null)` 不抛异常（禁用场景） |
| 3 | `TrayIcon.setQuickOperation_update` | auto | 同上 | 先设 config → 再设 null → 两次 invoke 均正常返回 |

```typescript
// 示例代码
{
  name: '@tauri-apps/api/tray.TrayIcon.setQuickOperation',
  category: 'auto',
  async fn() {
    assert(sharedTray !== null, 'sharedTray not initialized');
    await sharedTray.setQuickOperation({
      title: 'Test Panel',
      height: 250,
      abilityName: 'TestTrayAbility',
      moduleName: 'entry',
    });
  },
},
{
  name: '@tauri-apps/api/tray.TrayIcon.setQuickOperation_null',
  category: 'auto',
  async fn() {
    assert(sharedTray !== null, 'sharedTray not initialized');
    await sharedTray.setQuickOperation(null);
  },
},
{
  name: '@tauri-apps/api/tray.TrayIcon.setQuickOperation_update',
  category: 'auto',
  async fn() {
    assert(sharedTray !== null, 'sharedTray not initialized');
    await sharedTray.setQuickOperation({
      title: 'Updated Panel',
      height: 350,
      abilityName: 'TestTrayAbility',
    });
    await sharedTray.setQuickOperation(null);
  },
},
```

**跨平台行为**：
- 非 OHOS 平台：setter 为 no-op，测试通过（验证 API 不抛异常）
- OHOS 平台：实际更新 QuickOperation 配置（弹窗效果需手动验证）

### 5.4 API Demo 手动测试

#### 手动测试用例（3 个）

| # | 名称 | 按钮文案 | 文件 | 操作 | 预期 |
|---|------|----------|------|------|------|
| 1 | `quickOperationEnable` | Enable QuickOp (click tray icon) | `TestRunner.svelte` | 点击按钮 → 左键点击 tray 图标 | 系统面板弹出，标题 "Test Panel"，高度 250vp |
| 2 | `quickOperationUpdate` | Update QuickOp (title/height) | `TestRunner.svelte` | 点击按钮 → 左键点击 tray 图标 | 面板标题变为 "Updated Title"，高度 400vp |
| 3 | `quickOperationDisable` | Disable QuickOp (event only) | `TestRunner.svelte` | 点击按钮 → 左键点击 tray 图标 | 无面板弹出，仅触发事件 |

```typescript
// 示例代码
async function manualQuickOperationEnable() {
  await wrapManual('quickOperationEnable', async () => {
    const tray = await TrayIcon.getById('tray-1');
    if (!tray) { manualResult = 'tray-1 not found'; onMessage(manualResult); return; }
    await tray.setQuickOperation({
      title: 'Test Panel',
      height: 250,
      abilityName: 'TestTrayAbility',
      moduleName: 'entry',
    });
    manualResult = 'QuickOperation enabled.\nLeft-click tray icon → system popup should appear.';
    onMessage(manualResult);
  });
}

async function manualQuickOperationDisable() {
  await wrapManual('quickOperationDisable', async () => {
    const tray = await TrayIcon.getById('tray-1');
    if (!tray) { manualResult = 'tray-1 not found'; onMessage(manualResult); return; }
    await tray.setQuickOperation(null);
    manualResult = 'QuickOperation disabled.\nLeft-click tray icon → no popup, event only.';
    onMessage(manualResult);
  });
}

async function manualQuickOperationUpdate() {
  await wrapManual('quickOperationUpdate', async () => {
    const tray = await TrayIcon.getById('tray-1');
    if (!tray) { manualResult = 'tray-1 not found'; onMessage(manualResult); return; }
    await tray.setQuickOperation({
      title: 'Updated Title',
      height: 400,
      abilityName: 'TestTrayAbility',
    });
    manualResult = 'QuickOperation updated: title="Updated Title", height=400.';
    onMessage(manualResult);
  });
}
```

Svelte 按钮（OHOS desktop 条件渲染）：

```svelte
<!-- QuickOperation (OHOS only) -->
{#if isDesktop}
  <button class="btn" onclick={manualQuickOperationEnable}>Enable QuickOp (click tray icon)</button>
  <button class="btn" onclick={manualQuickOperationUpdate}>Update QuickOp (title/height)</button>
  <button class="btn" onclick={manualQuickOperationDisable}>Disable QuickOp (event only)</button>
{/if}
```

#### Rust 侧示例（`examples/api/src-tauri/src/tray.rs`）

Builder 中配置 QuickOperation：

```rust
let _ = TrayIconBuilder::with_id("tray-1")
    .tooltip("Tauri")
    .icon(app.default_window_icon().unwrap().clone())
    .menu(&menu1)
    .show_menu_on_left_click(false)
    .quick_operation(tauri::tray::QuickOperationConfig {
        title: "Tauri API".into(),
        height: 300,
        ability_name: "TestTrayAbility".into(),
        module_name: Some("entry".into()),
        loading_status: None,
    })
    // ...
```

Menu 中添加 "Toggle QuickOp" 菜单项用于运行时测试：

```rust
let toggle_qo = MenuItem::with_id(app, "toggle-qo", "Toggle QuickOp", true, None::<&str>)?;

// on_menu_event handler:
"toggle-qo" => {
    if let Some(tray) = app.tray_by_id("tray-1") {
        let _ = tray.set_quick_operation(None);
    }
}
```

### 5.5 OHOS 设备手动验证清单

| # | 场景 | 操作 | 预期 | 验证来源 |
|---|------|------|------|----------|
| 1 | 不配置 QuickOperation | Builder 不调 `.quick_operation()` | 左键点击仅触发事件，不弹窗 | Rust 示例 |
| 2 | 配置 QuickOperation | Builder 调 `.quick_operation(config)` | 左键点击弹出系统面板，标题和高度正确 | Rust 示例 + auto test #1 |
| 3 | 禁用弹窗 | 手动测试 "Disable QuickOp" 按钮 | 左键回退为仅触发事件 | manual test #3 |
| 4 | 运行时更新 | 手动测试 "Update QuickOp" 按钮 | 下次左键点击弹出新配置的面板 | manual test #2 |
| 5 | 面板内容 | 用户写的 ArkTS 页面加载 | 面板显示自定义 UI，交互正常 | Rust 示例 |
| 6 | 数据通信 | Rust 写 AppStorage → 弹窗页面读 | 弹窗页面拿到最新数据 | Rust 示例 |
| 7 | set_title 兼容 | 右键菜单 "Switch Menu" 触发 set_tooltip | 标题更新，QuickOperation 不变 | 现有功能回归 |
| 8 | set_visible 兼容 | `setVisible(false)` → `setVisible(true)` | 图标消失再出现，QuickOperation 保持 | 现有功能回归 |
| 9 | 右键菜单不受影响 | 右键点击 | 菜单正常弹出 | 现有功能回归 |

### 5.6 验证总览

| 层级 | 类型 | 用例数 | 运行环境 | 状态 |
|------|------|--------|----------|------|
| Rust UT (lib) | 新增 | 2 | 当前机器 `cargo test --lib` | ✅ 2/2 |
| Rust UT (lib) | 回归 | 1 | 当前机器 `cargo test --lib` | ✅ 1/1 |
| Rust UT (ohos) | 新增 | 4 | OHOS 交叉编译 | ✅ 4/4 |
| Rust UT (ohos) | 回归 | 5 | OHOS 交叉编译 | ✅ 5/5 |
| API Demo auto | 新增 | 3 | Windows + OHOS 设备 | ✅ 3/3 |
| API Demo manual | 新增 | 3 | OHOS 设备 | ⬜ 待人工验证 |
| 设备手动验证 | 清单 | 9 | OHOS 设备 | 🔶 5/9（#2 #3 #4 #5 #9 已验证） |
| **合计** | | **27** | | |

---

## 六、总结

| 缺失项 | 能否实现 | 处理状态 | 优先级 |
|--------|---------|---------|--------|
| QuickOperation | ✅ 能 | ✅ Steps 1-7 已实现 | **P1** |
| show_menu() 文档 | ✅ 仅文档 | ✅ Step 8 已添加 OHOS 注释 | P2 |
| DoubleClick/Enter/Move/Leave | ❌ 不能 | API 限制，不处理 | — |
| Middle/Down/position/rect | ❌ 不能 | API 限制，不处理 | — |
| updateStatusBarMenuItem (6.1.1) | ✅ 能 | 纯性能优化，暂不 | — |
| updateQuickOperationHeight | ✅ 能 | 随 QuickOperation 后续 | — |
| set_visible 事件重注册 | — | 确认无 bug，不处理 | — |

**涉及文件（实际 23 个）**：

tray-icon 仓库（2 文件，~275 行插入）：
1. `tray-icon/src/lib.rs` — 新增 QuickOperationConfig 类型 + Attributes 字段 + Builder 方法 + setter + show_menu 文档 + 2 UT (Steps 1-3, 8)
2. `tray-icon/src/platform_impl/ohos/mod.rs` — 提取 build_item_from_attrs() + set_quick_operation() + 重构 set_title()/set_icon() + 4 UT (Step 4)

tauri 仓库（21 文件，~1077 行插入）：
3. `tauri/crates/tauri/build.rs` — ACL 权限注册 (Step 7c)
4. `tauri/crates/tauri/src/tray/mod.rs` — Builder/setter 透传 + QuickOperationConfig re-export (Steps 5-6)
5. `tauri/crates/tauri/src/tray/plugin.rs` — 命令注册 (Step 7a-b)
6. `packages/api/src/tray.ts` — QuickOperationConfig interface + setQuickOperation()
7. `examples/api/src/lib/tests/tray.ts` — 3 个 auto test
8. `examples/api/src/views/TestRunner.svelte` — 3 个 manual handlers + 按钮
9. `examples/api/src/views/Tray.svelte` — QuickOperation 配置 UI 面板
10. `examples/api/src-tauri/src/tray.rs` — quick_operation 配置 + toggle 菜单项
11. `examples/api/src-tauri/gen/ohos/.../TestTrayAbility.ets` — StatusBarViewExtensionAbility 示例（新文件）
12. `examples/api/src-tauri/gen/ohos/.../TestTrayPage.ets` — 弹窗页面示例（新文件）
13. `examples/api/src-tauri/gen/ohos/.../module.json5` — 注册 TestTrayAbility extensionAbility
14. `examples/api/src-tauri/gen/ohos/.../main_pages.json` — 注册 TestTrayPage 路由
15-19. `examples/api/src-tauri/gen/schemas/*.json` (5 个) — ACL schema 自动更新
20. `doc/tray/guides/quick-operation-guide.md` — 用户使用指南（新文件）
21. 本文档 `tauri/doc/tray/impl/phase9-quick-operation-design.md`

> **实现备注**：原设计预估 5 个文件，实际涉及 23 个。主要增量来自：ArkTS 示例文件（TestTrayAbility/TestTrayPage）、配置文件（module.json5/main_pages.json）、Tray.svelte UI、schema 自动生成、ACL 注册。

### 实现过程中发现的关键事项

| 事项 | 说明 |
|------|------|
| ACL 权限注册 | 新 tray plugin 命令必须在 `build.rs` PLUGINS 数组中注册，否则前端调用被拦截 |
| packages/api 重建 | 修改 `packages/api/src/tray.ts` 后必须 `pnpm build`，否则设备运行时报方法不存在 |
| main_pages.json | 弹窗页面必须在 `main_pages.json` 中注册路由，否则 `loadContent()` 失败 |
| OHOS 单 tray 限制 | StatusBar API 是全局操作（无 tray ID），一个应用只能有一个 tray 图标 |
| terminateSelf() | `UIExtensionContentSession` 没有 `hide()` 方法，关闭弹窗必须用 `terminateSelf()` |
| onSessionDestroy 签名 | 无参数 `(): void`，非 `(session): void` |
| build_item_from_attrs | 函数名为 `build_item_from_attrs`（非设计中的 `build_item`），更准确反映其职责 |

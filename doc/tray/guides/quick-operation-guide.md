# QuickOperation — OHOS 托盘左键弹窗

> **平台**：OHOS only
> **API 版本**：6.0.0(20)+（`loadingStatus` 字段）
> **对应设计文档**：[phase9-quick-operation-design.md](../impl/phase9-quick-operation-design.md)

---

## 概述

QuickOperation 是 OHOS 平台独有的系统级托盘弹窗能力。用户在状态栏**左键点击**托盘图标时，系统会弹出一个带有标题栏和自定义内容区的面板。面板在系统层渲染，即使应用最小化也能弹出。

其他平台的左键行为对比：

| 平台 | 左键行为 |
|------|---------|
| Windows | 弹出上下文菜单 (TrackPopupMenu) |
| macOS | 弹出 NSMenu 或高亮图标 |
| Linux | 弹出 GTK 菜单 |
| **OHOS (QuickOperation)** | **弹出系统级面板 (StatusBarViewExtensionAbility)** |

> **Note**
> 在非 OHOS 平台调用 QuickOperation 相关 API 不会报错，但也不会产生任何效果（no-op）。这与 `set_icon_as_template()` (macOS only) 和 `set_temp_dir_path()` (Linux only) 的行为一致。

---

## 前置条件

### 1. Cargo.toml

确保 `tauri` 启用了 `tray-icon` feature：

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
```

### 2. module.json5

在你的 OHOS 应用模块中注册一个 `StatusBarViewExtensionAbility`：

```json5
// entry/src/main/module.json5
{
  "module": {
    "extensionAbilities": [
      {
        "name": "MyTrayAbility",
        "type": "statusBarView",
        "exported": true,
        "srcEntry": "./ets/MyTrayAbility.ets"
      }
    ]
  }
}
```

| 字段 | 说明 |
|------|------|
| `name` | ExtensionAbility 名称，必须与 Rust 侧 `ability_name` 匹配 |
| `type` | 固定为 `"statusBarView"` |
| `exported` | 必须为 `true`，系统需要能启动该 Ability |
| `srcEntry` | 指向你的 Ability 入口文件 |

### 3. main_pages.json

在页面路由配置中注册你的弹窗页面：

```json
// entry/src/main/resources/base/profile/main_pages.json
{
  "src": [
    "pages/Index",
    "pages/MyTrayPage"
  ]
}
```

### 4. API 版本

- 基础 QuickOperation：OHOS 5.0+
- `loadingStatus` 字段：OHOS 6.0.0(20)+

---

## 注意事项

### OHOS 单 Tray 限制

OHOS 系统**一个应用只能有一个托盘图标**（StatusBar API 是全局操作，没有 tray ID 概念）。如果你通过 JS 侧 `TrayIcon.new()` 创建了新的 tray，它会**替换**之前 Rust 侧创建的 tray。autotest 运行期间也会替换，autotest 结束后 tray 消失。

### QuickOperation 与右键菜单

QuickOperation 仅影响**左键点击**行为。右键点击仍然弹出上下文菜单（由 `menu` 配置），两者互不干扰：

| 操作 | 行为 |
|------|------|
| 左键点击 | 弹出 QuickOperation 面板（如果配置了 `ability_name`），否则仅触发事件 |
| 右键点击 | 弹出上下文菜单（由 `menu` 配置） |

### 关闭面板

面板可以通过以下方式关闭：
- 点击面板外部区域
- 按返回键
- 编程式关闭：调用 `session.terminateSelf()`

```typescript
// 在 Ability 中保存 session 引用
let activeSession: UIExtensionContentSession | null = null;

export function closePanel(): void {
  activeSession?.terminateSelf();
}

export default class MyTrayAbility extends StatusBarViewExtensionAbility {
  onSessionCreate(want: Want, session: UIExtensionContentSession): void {
    activeSession = session;
    session.loadContent('pages/MyTrayPage');
  }

  onSessionDestroy(): void {
    activeSession = null;
  }
}
```

`terminateSelf()` 只关闭面板（ExtensionAbility 会话），**不影响 tray 图标和主应用**。下次左键点击会重新创建会话并弹出面板。

---

## 快速开始

以下是最小可运行示例：创建一个托盘图标，左键点击弹出系统面板。

### Rust 侧

```rust
use tauri::tray::{TrayIconBuilder, QuickOperationConfig};
use tauri::menu::{Menu, MenuItem};

fn create_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit])?;

    let _tray = TrayIconBuilder::with_id("main-tray")
        .tooltip("My App")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        // OHOS: 配置左键弹窗（其他平台 no-op）
        .quick_operation(QuickOperationConfig {
            title: "My App".into(),
            height: 300,
            ability_name: "MyTrayAbility".into(),
            module_name: Some("entry".into()),
            loading_status: None,
        })
        .build()?;

    Ok(())
}
```

### ArkTS 侧

创建 `MyTrayAbility.ets`：

```typescript
import { StatusBarViewExtensionAbility } from "@kit.StatusBarExtensionKit";
import { UIExtensionContentSession, Want } from "@kit.AbilityKit";

export default class MyTrayAbility extends StatusBarViewExtensionAbility {
  onSessionCreate(want: Want, session: UIExtensionContentSession): void {
    // 加载自定义页面
    session.loadContent('pages/MyTrayPage');
  }

  onSessionDestroy(): void {
    // session 结束回调（面板关闭时触发）
  }
}
```

创建 `pages/MyTrayPage.ets`（别忘了在 `main_pages.json` 中注册路由）：

```typescript
import { closePanel } from '../MyTrayAbility'

@Entry
@Component
struct MyTrayPage {
  build() {
    Column({ space: 12 }) {
      Text('Hello from Tray!').fontSize(16).fontWeight(FontWeight.Bold)
      Button('Open Main Window').onClick(() => {
        // 你的业务逻辑
      })
      Button('Close').backgroundColor('#ff6666').onClick(() => {
        closePanel()  // 编程式关闭面板
      })
    }
    .width('100%')
    .height('100%')
    .justifyContent(FlexAlign.Center)
  }
}
```

完成后，左键点击托盘图标即可看到弹出的系统面板。

---

## 详细示例

### 完整 Rust 配置

```rust
use tauri::tray::{TrayIconBuilder, TrayIconEvent, QuickOperationConfig};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};

fn setup_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    // 菜单项
    let show = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&show, &separator, &quit])?;

    // 创建托盘图标
    let tray = TrayIconBuilder::with_id("main-tray")
        .tooltip("My Tauri App")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(false) // 左键留给 QuickOperation
        .quick_operation(QuickOperationConfig {
            title: "My Tauri App".into(),
            height: 300,                       // 弹窗高度 (vp)
            ability_name: "MyTrayAbility".into(), // 对应 module.json5 中的 name
            module_name: Some("entry".into()), // Ability 所在模块
            loading_status: Some(false),       // 6.0.0(20)+: 是否显示加载动效
        })
        .on_menu_event(|app, event| {
            match event.id().as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        window.show().ok();
                        window.set_focus().ok();
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: tauri::tray::MouseButton::Right, .. } = event {
                // 右键点击时也可以做额外处理
            }
        })
        .build()?;

    Ok(())
}
```

### 运行时更新

```rust
// 更新 QuickOperation 配置
if let Some(tray) = app.tray_by_id("main-tray") {
    tray.set_quick_operation(Some(QuickOperationConfig {
        title: "Now Playing: Song Title".into(),
        height: 400,
        ability_name: "MyTrayAbility".into(),
        module_name: Some("entry".into()),
        loading_status: None,
    }));
}

// 禁用左键弹窗（回退为仅触发事件）
if let Some(tray) = app.tray_by_id("main-tray") {
    tray.set_quick_operation(None);
}
```

### JavaScript/TypeScript 侧

```typescript
import { TrayIcon } from '@tauri-apps/api/tray';

// 创建带 QuickOperation 的托盘图标
const tray = await TrayIcon.new({
  id: 'main-tray',
  tooltip: 'My App',
  quickOperation: {
    title: 'My App',
    height: 300,
    abilityName: 'MyTrayAbility',
    moduleName: 'entry',
  },
});

// 运行时更新
await tray.setQuickOperation({
  title: 'Updated Title',
  height: 400,
  abilityName: 'MyTrayAbility',
});

// 禁用
await tray.setQuickOperation(null);
```

### ArkTS ExtensionAbility 完整示例

```typescript
// MyTrayAbility.ets
import { StatusBarViewExtensionAbility } from "@kit.StatusBarExtensionKit";
import { UIExtensionContentSession, Want } from "@kit.AbilityKit";
import { hilog } from '@kit.PerformanceAnalysisKit';

export default class MyTrayAbility extends StatusBarViewExtensionAbility {
  onCreate(): void {
    hilog.info(0x0000, 'MyTrayAbility', 'onCreate');
  }

  onSessionCreate(want: Want, session: UIExtensionContentSession): void {
    hilog.info(0x0000, 'MyTrayAbility', 'onSessionCreate: %{public}s', JSON.stringify(want));
    // want 由系统自动构造，包含:
    // - bundleName: 当前应用包名
    // - abilityName: "MyTrayAbility"
    // - moduleName: "entry"
    //
    // 不支持自定义数据，需通过 AppStorage/Preferences 等间接传递
    session.loadContent('pages/MyTrayPage');
  }

  onForeground(): void {
    hilog.info(0x0000, 'MyTrayAbility', 'onForeground');
  }

  onBackground(): void {
    hilog.info(0x0000, 'MyTrayAbility', 'onBackground');
  }

  onSessionDestroy(): void {
    hilog.info(0x0000, 'MyTrayAbility', 'onSessionDestroy');
  }

  onDestroy(): void {
    hilog.info(0x0000, 'MyTrayAbility', 'onDestroy');
  }
}
```

### 弹窗页面 + 数据通信

由于 `want` 参数不支持自定义数据，弹窗页面需要通过 **AppStorage** 或 **Preferences** 与应用主进程通信：

```typescript
// pages/MyTrayPage.ets — 音乐播放器示例
@Entry
@Component
struct MyTrayPage {
  // 通过 AppStorage 与主进程共享状态
  @StorageLink('tray_current_track') currentTrack: string = 'Unknown'
  @StorageLink('tray_is_playing') isPlaying: boolean = false
  @StorageLink('tray_volume') volume: number = 50

  build() {
    Column({ space: 8 }) {
      // 当前曲目
      Text(this.currentTrack)
        .fontSize(14)
        .fontWeight(FontWeight.Medium)
        .maxLines(1)
        .textOverflow({ overflow: TextOverflow.Ellipsis })

      // 播放控制
      Row({ space: 16 }) {
        Button('⏮').fontSize(20).onClick(() => {
          // 上一曲 — 通过 AppStorage 通知主进程
          AppStorage.setOrCreate('tray_action', 'prev')
        })
        Button(this.isPlaying ? '⏸' : '▶')
          .fontSize(24)
          .onClick(() => {
            this.isPlaying = !this.isPlaying
            AppStorage.setOrCreate('tray_is_playing', this.isPlaying)
          })
        Button('⏭').fontSize(20).onClick(() => {
          AppStorage.setOrCreate('tray_action', 'next')
        })
      }

      // 音量调节
      Row() {
        Text('🔈').fontSize(16)
        Slider({
          value: this.volume,
          min: 0,
          max: 100
        })
          .onChange((value: number) => {
            this.volume = value
            AppStorage.setOrCreate('tray_volume', value)
          })
          .layoutWeight(1)
        Text('🔊').fontSize(16)
      }
    }
    .width('100%')
    .height('100%')
    .padding(16)
    .justifyContent(FlexAlign.Center)
  }
}
```

Rust 侧通过 TSFN 写入 AppStorage：

```rust
// 更新弹窗数据（需要通过自定义 NAPI/TSFN 桥接到 ArkTS 侧 AppStorage）
// tauri 不提供内置桥接，应用需自行实现
```

---

## API 参考

### QuickOperationConfig

```rust
pub struct QuickOperationConfig {
    /// 弹窗标题栏文字，超长显示省略号
    pub title: String,

    /// 弹窗自定义区域高度 (vp, 必须 > 0)
    pub height: u32,

    /// StatusBarViewExtensionAbility 名称（module.json5 中注册）。
    /// 空字符串 = 禁用弹窗（左键仅触发 statusBarIconClick 事件）。
    pub ability_name: String,

    /// ExtensionAbility 所在模块名。
    /// None = 使用默认模块。
    pub module_name: Option<String>,

    /// 控制是否显示加载动效。
    /// 需要 OHOS 6.0.0(20)+ 版本支持。
    pub loading_status: Option<bool>,
}
```

### Builder 方法

```rust
// 构建时配置
TrayIconBuilder::new()
    .quick_operation(config)  // OHOS only
    .build()?
```

### 运行时方法

```rust
// 更新或禁用
tray.set_quick_operation(Some(config));  // 更新配置
tray.set_quick_operation(None);          // 禁用弹窗
```

### JavaScript API

```typescript
interface QuickOperationConfig {
  title: string          // 标题栏文字
  height: number         // 内容区高度 (vp)
  abilityName: string    // ExtensionAbility 名称
  moduleName?: string    // 模块名
  loadingStatus?: boolean // 加载动效 (6.0.0(20)+)
}

// 创建时
TrayIcon.new({ quickOperation: config })

// 运行时
tray.setQuickOperation(config | null)
```

---

## 架构说明

QuickOperation 的工作原理是 Rust API 配"壳"，用户写"肉"：

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

1. 用户左键点击托盘图标
2. 系统根据 `abilityName` 查找 `module.json5` 中注册的 ExtensionAbility
3. 系统启动该 Ability，调用 `onSessionCreate(want, session)`
4. **用户在自己的 Ability 代码里**调用 `session.loadContent('pages/xxx')` 加载页面
5. 页面渲染在系统级弹窗中，高度由 `height` 控制

---

## 数据通信

弹窗运行在 `StatusBarViewExtensionAbility` 进程中，与应用主进程隔离。`want` 参数由系统自动构造，**不支持自定义数据**。

### 推荐的通信方式

| 方式 | 说明 | 适用场景 |
|------|------|---------|
| **AppStorage** | 跨 Ability 共享状态 | 简单键值对、开关状态 |
| **Preferences** | 持久化存储 | 配置项、需要持久化的数据 |
| **commonEventManager** | 事件订阅/发布 | 事件通知、状态变更 |
| **RelationalStore** | 关系型数据库 | 复杂结构化数据 |

### AppStorage 示例

```typescript
// 主进程写入
AppStorage.setOrCreate('tray_current_track', 'Song Title');
AppStorage.setOrCreate('tray_is_playing', true);

// 弹窗页面读取（@StorageLink 自动同步）
@StorageLink('tray_current_track') currentTrack: string = 'Unknown'
@StorageLink('tray_is_playing') isPlaying: boolean = false
```

---

## 平台差异

| API | Windows | macOS | Linux | OHOS |
|-----|---------|-------|-------|------|
| `quick_operation()` | no-op | no-op | no-op | ✅ 生效 |
| `set_quick_operation()` | no-op | no-op | no-op | ✅ 生效 |
| 左键默认行为 | 弹出菜单 | 弹出菜单 | 弹出菜单 | 触发事件 (无 QuickOp 时) |

> **Note**
> 在非 OHOS 平台，`quick_operation()` 和 `set_quick_operation()` 会被静默忽略。你可以安全地在跨平台代码中使用这些 API，无需 `#[cfg]` 条件编译。

---

## FAQ

### 左键点击后没有弹窗？

1. 确认 `ability_name` 与 `module.json5` 中注册的 ExtensionAbility `name` 完全匹配
2. 确认 `module.json5` 中 `type` 为 `"statusBarView"`，`exported` 为 `true`
3. 确认 `srcEntry` 指向的文件存在且导出了继承 `StatusBarViewExtensionAbility` 的默认类
4. 确认 `height` > 0

### 弹窗出来了但是空白？

1. 确认 `onSessionCreate` 中调用了 `session.loadContent('pages/xxx')`
2. 确认页面路径正确（相对于 `src/main/resources/base/profile/main_pages.json` 中注册的路由）
3. 确认页面使用了 `@Entry` 和 `@Component` 装饰器

### 弹窗里读不到 AppStorage 数据？

`StatusBarViewExtensionAbility` 运行在独立进程中，`AppStorage` 在跨进程场景下可能不支持。此时需要改用 `Preferences` 或 `commonEventManager` 进行数据通信。

### `loadingStatus` 不生效？

`loadingStatus` 需要 OHOS 6.0.0(20)+ 版本支持。低版本会静默忽略此字段。

### 可以在弹窗里放 WebView 吗？

可以。QuickOperation 弹窗内容区和普通 ArkUI 页面完全一致，支持所有 ArkUI 组件，包括 `Web` 组件。注意高度由 `height` 控制，内容超出会截断。

### 运行时更新 QuickOperation 后什么时候生效？

实现层面是 remove + rebuild + add 整个 StatusBarItem，因此：
- 更新后的配置在**下次左键点击**时生效
- 当前已打开的弹窗不受影响（弹窗是独立的 ExtensionAbility 会话）

### 为什么 autotest 运行后 tray 图标消失了？

OHOS 一个应用只能有一个 tray 图标。autotest 通过 JS 侧 `TrayIcon.new()` 创建测试用 tray，这会**替换**掉 Rust 侧 `setup()` 中创建的 tray。autotest 结束后调用 `tray.close()` 移除 tray，状态栏就空了。这是 OHOS 系统限制，不是 bug。

如果你需要在 autotest 后保留 tray，可以在测试结束后重新创建，或者避免在 autotest 中创建 tray。

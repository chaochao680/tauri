## Context

### 问题背景

在 Tauri OHOS 适配过程中，观察到三个插件的适配复杂度存在巨大差异：

| 插件 | PR | 修改文件数 | 是否修改 tauri 核心 |
|------|-----|-----------|-------------------|
| process | plugins-workspace #4 | 2 个 | ❌ 否 |
| updater | plugins-workspace #4 | 3 个 | ❌ 否 |
| dialog | plugins-workspace #6 + tauri #20 | 7+ 个（插件）+ 50+ 个（tauri 核心） | ✅ 是 |

这种差异引发了根本性的架构审视。

### 两个 HAR 的发现

通过代码分析，发现 Tauri OHOS 架构中存在**两个不同职责的 HAR 包**：

```
┌───────────────────────────────────────────────────────────────────┐
│                   openharmony-ability 仓库                         │
│   Rust crate: openharmony-ability (crates.io)                     │
│   ohpm 包: @ohos-rs/ability                                       │
│                                                                   │
│   职责：OHOS 平台原生能力桥接（NAPI + TSFN）                         │
│   包含：生命周期、窗口管理、WebView、菜单、updater、statusbar 等       │
│                                                                   │
│   定位：Rust/C 原生模块与 ArkTS 的底层交互层                         │
└──────────────────────────────┬────────────────────────────────────┘
                               │ tauri 核心 pub use
                               ▼
┌───────────────────────────────────────────────────────────────────┐
│                        tauri 核心仓库                              │
│   路径: crates/tauri/mobile/ohos/                                 │
│   ohpm 包: @tauri/app                                             │
│                                                                   │
│   职责：插件注册 + IPC 分发框架                                      │
│   包含：Plugin.ets, PluginManager.ets                             │
│                                                                   │
│   定位：tauri 插件系统的 OHOS 基础设施                               │
└───────────────────────────────────────────────────────────────────┘
```

### tauri ohos.rs 的直接引用

```rust
// crates/tauri/src/ohos.rs
pub use openharmony_ability;
pub static APP: Mutex<Option<openharmony_ability::OpenHarmonyApp>> = Mutex::new(None);
```

tauri 核心通过 `pub use` 直接暴露 openharmony-ability 的 API，供插件使用。

## Goals / Non-Goals

**Goals:**
- 澄清 tauri 核心 HAR 与 openharmony-ability HAR 的职责边界
- 解释为什么不同插件的适配复杂度差异巨大
- 总结三种 OHOS 插件适配模式的适用场景
- 评估当前 PluginManager 架构的合理性
- 为后续插件适配提供架构指导

**Non-Goals:**
- 不提出新的架构方案（当前方案已合理）
- 不修改现有代码（这是分析文档，不是实现 PR）
- 不评估 Android/iOS 插件系统（仅关注 OHOS）

## Decisions

### Decision 1: process 插件为什么可以简单适配

**发现**：process 插件采用**直接调用模式**，复用 openharmony-ability 已有的桥接。

**调用链**：
```
Rust command → tauri::ohos::APP（全局 Mutex）→ openharmony_ability::OpenHarmonyApp
                → NAPI/TSFN → ArkTS appRecovery.restartApp()
```

**代码示例**：
```rust
// plugins/process/src/ohos.rs
#[tauri::command]
pub fn restart<R: Runtime>(_app: AppHandle<R>) {
    let guard = tauri::ohos::APP.lock().unwrap();
    let app = guard.as_ref().unwrap();
    app.restart();  // 直接调用 openharmony-ability 的方法
}
```

**为什么不需要 PluginManager**：
- `appRecovery` 是 OHOS 系统 API，已在 openharmony-ability 的 `NativeAbility.ets` 中桥接
- process 插件只是复用这个已有的桥接，不需要自己的 ArkTS 插件模块
- 不需要注册独立的插件标识符，不需要 IPC 分发

**适用场景**：
- 插件功能已被 openharmony-ability 覆盖（如重启、基础生命周期）
- 插件的公共 API 无平台分支（process 只有 `exit()` 和 `restart()`）

### Decision 2: updater 插件为什么可以简单适配

**发现**：updater 插件同样采用**直接调用模式**，但采用了**粗粒度平行替换**策略。

**调用链**：
```
Rust command → tauri::ohos::APP → openharmony_ability::Updater
                → NAPI/TSFN → ArkTS updateManager.checkAppUpdate()
```

**代码示例**：
```rust
// plugins/updater/src/ohos.rs
pub(crate) async fn check<R: Runtime>(...) -> Result<Option<Metadata>> {
    let guard = tauri::ohos::APP.lock().unwrap();
    let app = guard.as_ref().unwrap();
    let updater = app.updater();  // openharmony-ability 提供的 Updater
    updater.check().await;        // 内部走 TSFN → ArkTS updateManager
}
```

**为什么不需要 PluginManager**：
- `updateManager` 是 OHOS AppGallery SDK，已在 openharmony-ability 的 `updater.ets` 中桥接
- updater 插件只是调用 `OpenHarmonyApp::updater()` 方法

**为什么可以采用"粗粒度平行替换"**：
- updater 的桌面端实现（HTTP manifest 更新、签名验证、安装器）与 OHOS 端（AppGallery）完全不同
- OHOS 用户**根本不需要也不应该看到**桌面版 Builder 的方法（`pubkey()`、`installer_args()` 等）
- 可以用一个 `#[cfg(not(target_env = "ohos"))]` 门控整个桌面端，然后写独立的 OHOS 模块

**代码结构**：
```rust
// plugins/updater/src/lib.rs

// 整个桌面端用一个 cfg 门控
#[cfg(not(target_env = "ohos"))]
mod commands;
#[cfg(not(target_env = "ohos"))]
mod updater;
#[cfg(not(target_env = "ohos"))]
pub trait UpdaterExt<R: Runtime> { ... }

// OHOS 端用独立模块
#[cfg(target_env = "ohos")]
mod ohos;

// Builder 的 build() 方法有两个版本
#[cfg(not(target_env = "ohos"))]
pub fn build(self) -> TauriPlugin<R, Config> { /* 桌面版 */ }

#[cfg(target_env = "ohos")]
pub fn build(self) -> TauriPlugin<R, Config> { /* OHOS 版 */ }
```

**适用场景**：
- 插件功能已被 openharmony-ability 覆盖（如 AppGallery 更新）
- 插件的 OHOS 实现与桌面端实现**完全不同**，可以整体替换
- 插件的公共 API 在 OHOS 上可以**大幅缩减**（OHOS 不需要桌面版的那些方法）

### Decision 3: dialog 插件为什么不能简单适配

**发现**：dialog 插件必须采用**插件注册模式**，需要独立的 ArkTS 插件模块。

**调用链**：
```
Rust command → run_mobile_plugin(handle, "open", payload)
                    ↓
               IPC 消息 → PluginManager.runCommand()
                    ↓
               "@tauri/plugin-dialog" 插件模块 → DialogPlugin.ets
                    ↓                                                    ↓
               独立的 ArkTS HAR 模块                            @ohos.file.picker
                                                                @ohos.promptAction
```

**为什么不能走 openharmony-ability**：
- dialog 的功能（文件选择器 `@ohos.file.picker`、消息弹框 `@ohos.promptAction`）是**插件级能力**，不是框架级能力
- 如果塞进 openharmony-ability，会导致：
  1. **体积膨胀**：每个 Tauri app 都要编译所有插件的原生桥接
  2. **发布耦合**：修改任何一个插件都要发新版 openharmony-ability → 再更新 tauri 依赖
  3. **职责模糊**：openharmony-ability 变成了"所有 OHOS 原生代码的大杂烩"

**为什么不能像 updater 那样整体门控 + 平行替换**：
- dialog 的公共 API 在桌面和移动端是"同一套类型、不同字段"，而非"完全不同的类型"
- `FileDialogBuilder` 和 `MessageDialogBuilder` 是**同一个 struct**，但部分字段/方法仅桌面有
- 用户代码在 `#[cfg(desktop)]` 下调用 `set_parent()`，OHOS 编译时这些代码会被编译但不应存在

**核心矛盾**：
```
OHOS 平台特征：
- target_os = "linux"  → cfg(desktop) = true    ✅ 被识别为桌面
- target_env = "ohos"  → 但实际无 X11/Wayland，无 rfd，无窗口 parent

原代码中 dialog 插件的平台分支：
- cfg(desktop) → 使用 rfd crate（原生文件对话框） + raw_window_handle parent
- cfg(mobile)  → 使用 run_mobile_plugin 调用 Kotlin/Swift 原生插件
```

**必须逐行修改 cfg 的原因**：

```rust
// FileDialogBuilder 是用户在 Rust 端直接使用的公共 API
pub struct FileDialogBuilder<R: Runtime> {
    pub(crate) file_name: Option<String>,         // 全平台
    pub(crate) filters: Vec<DialogFilter>,         // 全平台
    pub(crate) default_path: Option<PathBuf>,       // 全平台
    
    // 仅桌面有，但 OHOS 上不存在
    #[cfg(all(desktop, not(target_env = "ohos")))]
    pub(crate) parent: Option<crate::desktop::WindowHandle>,
}

// commands.rs 中共享的命令函数
pub(crate) async fn open<R: Runtime>(...) -> Result<OpenResponse> {
    let res = if options.directory {
        #[cfg(all(desktop, not(target_env = "ohos")))]
        { /* rfd folder picker — OHOS 上不存在 */ }
        
        #[cfg(any(mobile, target_env = "ohos"))]
        return Err(Error::FolderPickerNotImplemented);
    }
}
```

**cfg 修改量不可避免**：
- `OpenResponse::Folders/Folder` 枚举变体（OHOS 不支持文件夹选择）
- `rfd` 依赖排除（OHOS 不支持）
- `FileDialogBuilder::parent` 字段/方法（OHOS 无 raw_window_handle）
- `MessageDialogBuilder::parent` 字段/方法
- `set_default_path` 两个版本
- `init()` 中的 `invoke_handler` 注册策略
- `mobile::init()` 需要调用 `register_ohos_plugin()`

**为什么需要 tauri 核心的 PluginManager**：
- dialog 的 ArkTS 实现不在 openharmony-ability 中，需要独立的插件模块
- 需要注册机制让 Rust 侧知道怎么找到 ArkTS 插件
- PluginManager 提供 IPC 分发框架，与 Android/iOS 的插件架构一致

**适用场景**：
- 插件功能是**插件级能力**，不是框架级能力
- 插件的公共 API 在桌面和移动端是"同一套类型、不同字段"
- 插件需要独立的 ArkTS 模块，使用 OHOS 专属 API（如 `@ohos.file.picker`）

### Decision 4: 三种适配模式的对比

| 模式 | 适用插件 | 调用链 | 是否需要 PluginManager | 修改文件数 | 是否修改 tauri 核心 |
|------|---------|--------|----------------------|-----------|-------------------|
| **直接调用** | process | `tauri::ohos::APP → openharmony_ability → NAPI` | ❌ 否 | 2 个 | ❌ 否 |
| **平行替换** | updater | `tauri::ohos::APP → openharmony_ability → NAPI` | ❌ 否 | 3 个 | ❌ 否 |
| **插件注册** | dialog | `run_mobile_plugin → PluginManager → ArkTS HAR` | ✅ 是 | 7+ 个 | ✅ 是（首次需要） |

**选择标准**：

```
问：插件功能是否已被 openharmony-ability 覆盖？
├─ 是 → 直接调用模式（process）
└─ 否 → 问：插件的 OHOS 实现与桌面端是否完全不同？
         ├─ 是 → 平行替换模式（updater）
         └─ 否 → 插件注册模式（dialog）
```

### Decision 5: PluginManager 架构的合理性

**当前架构**：
```
┌─────────────────────────────────────────────────────────────────┐
│                        ArkTS 侧架构                              │
│                                                                  │
│  ┌──────────────────────────────────────┐                       │
│  │     tauri 核心 HAR (每个 app 都有)    │                       │
│  │                                      │                       │
│  │  • Plugin.ets (抽象基类)              │                       │
│  │  • PluginManager.ets (注册 + IPC)     │                       │
│  └──────────────────────────────────────┘                       │
│                                                                  │
│  ┌──────────────────────────────────────┐                       │
│  │  dialog 插件 HAR (只有用了才有)        │                       │
│  │                                      │                       │
│  │  • DialogPlugin.ets                   │                       │
│  │  • @ohos.file.picker                  │                       │
│  │  • @ohos.promptAction                 │                       │
│  └──────────────────────────────────────┘                       │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**与 Android/iOS 的对比**：

```
plugins-workspace/
├── plugins/dialog/
│   ├── android/     ← dialog 自己的 Kotlin 模块，独立 AAR
│   ├── ios/         ← dialog 自己的 Swift 模块，独立 framework
│   └── src/         ← Rust 代码
├── plugins/notification/
│   ├── android/     ← 独立 AAR
│   ├── ios/         ← 独立 framework
│   └── ...
```

每个插件的原生代码是**独立的包**，不是塞进 tauri 核心。OHOS 上的 PluginManager + 独立插件 HAR 模式，是**对齐 Android/iOS 的既有架构**。

**替代方案评估**：

| 方案 | 做法 | 优点 | 缺点 |
|------|------|------|------|
| **A. 把 dialog 加进 openharmony-ability** | 在 openharmony-ability 里加 `dialog.rs` + `dialog.ets` | 简单（不需要 PluginManager） | 体积膨胀、发布耦合、职责模糊 |
| **B. 保持独立插件 HAR（当前方案）** | tauri 核心提供 PluginManager 框架，dialog 有自己的 HAR | 与 Android/iOS 一致、长期可扩展 | 需要 tauri 核心的插件基础设施（50+ 文件） |

**结论**：当前方案（B）是合理的，虽然改动量大，但架构清晰、职责分离、长期可维护。

### Decision 6: openharmony-ability 的职责边界

**当前包含的功能**（框架级）：

| 模块 | 理由 |
|------|------|
| `app.rs` / `lifecycle.rs` | 每个 OHOS app 都需要 |
| `window.rs` | 每个 app 都有窗口 |
| `webview.rs` | 每个 Tauri app 都有 webview |
| `menu.rs` | 桌面端 app 基本都有 |
| `updater.rs` | 大多数 app 都需要（AppGallery SDK） |
| `statusbar.rs` | 移动端通用 |

**不应该包含的功能**（插件级）：

| 模块 | 理由 |
|------|------|
| `dialog.rs` | 只有用了 dialog 插件才需要 |
| `notification.rs` | 只有用了 notification 插件才需要 |
| `camera.rs` | 只有用了 camera 插件才需要 |
| `geolocation.rs` | 只有用了 geolocation 插件才需要 |

**判断标准**：
```
问：这个功能是"每个 Tauri app 都需要"还是"只有用了某个插件才需要"？
├─ 每个 app 都需要 → 框架级 → openharmony-ability
└─ 只有用了某个插件才需要 → 插件级 → 独立插件 HAR
```

### Decision 7: dialog 适配的改进空间

虽然 dialog 的 cfg 修改量不可避免，但**组织方式可以改进**：

**改进 1：基础设施应先行独立 PR**

当前 tauri PR #20 混合了：
- OHOS 插件基础设施（ohos.rs, ohos_plugin.rs, plugins.rs）← 应该先合
- dialog 插件 ArkTS 模板 ← 应该后合
- 构建脚本改进（template detection, plugin injection）← 应该先合
- 杂项修复（.gitattributes, build env vars）← 应该分开

如果 OHOS 插件基础设施先作为独立 PR 合入 tauri，那 dialog PR 就只需要改 plugins-workspace 中的文件。

**改进 2：workspace 级别统一 tauri-plugin 依赖**

当前 dialog PR 中有一个不优雅的修改：

```toml
# Cargo.toml — workspace 级别
- tauri-plugin = "2.5"
+ tauri-plugin = { path = "../tauri/crates/tauri-plugin" }  # ← 改为本地路径
```

这是为了使用 `ohos_path()` 和 `register_ohos_plugin()` 这些还未发布的 API。**理想做法**是先发布包含这些 API 的 tauri-plugin 版本。

**改进 3：dialog 的 PR 可以更聚焦**

当前 PR #6 的 design.md 写得很详细，这是好事。但可以考虑拆分：
- `Cargo.toml` 平台配置 + `build.rs` + `schema.json` → 一个 PR
- `commands.rs` + `lib.rs` cfg 修改 → 一个 PR
- `mobile.rs` OHOS 注册 → 一个 PR（依赖 tauri 核心已合入）

## Risks / Trade-offs

**[Trade-off] 架构清晰性 vs 短期改动量**
- 当前方案（独立插件 HAR）需要 50+ 文件改动，但架构清晰、职责分离
- 替代方案（塞进 openharmony-ability）改动量小，但长期维护成本高
- 选择当前方案是正确的

**[Risk] 后续插件适配可能重复 dialog 的路径**
- 任何使用 `cfg(desktop)` / `cfg(mobile)` 分支的插件，都需要类似 dialog 的 cfg 修改
- 需要文档指导，避免每个插件都重新发明轮子

**[Trade-off] openharmony-ability 的边界模糊**
- `updater` 被放进 openharmony-ability，但它是"大多数 app 需要"还是"插件级"？
- 边界并不绝对清晰，但 updater 的 AppGallery SDK 是系统级 API，更接近框架级

**[Risk] PluginManager 的 IPC 开销**
- `run_mobile_plugin` → PluginManager → ArkTS 插件的调用链有 IPC 开销
- 直接调用模式（process/updater）没有这个开销
- 但对于 dialog 这种低频操作（用户点击按钮才触发），开销可忽略

## Open Questions

### Q1: 后续插件适配的最佳实践文档

是否应该编写一份"OHOS 插件适配指南"，总结三种模式的适用场景和具体步骤？

**建议**：是。应该包含：
1. 如何判断插件应该采用哪种模式
2. 每种模式的具体步骤和代码模板
3. 常见陷阱（如 `cfg(desktop)` 在 OHOS 上的行为）

### Q2: openharmony-ability 的边界是否需要正式化

是否应该在 openharmony-ability 的 README 中明确声明"只包含框架级能力，不包含插件级能力"？

**建议**：是。可以避免后续开发者误将插件功能塞进 openharmony-ability。

### Q3: PluginManager 的性能监控

是否需要在 PluginManager 中添加性能追踪（如命令执行耗时），以便后续优化？

**建议**：暂不需要。当前插件（dialog）是低频操作，性能不是瓶颈。等出现性能问题再加。

## Summary

### 核心结论

1. **process/updater 可以简单适配**，因为它们复用 openharmony-ability 已有的框架级桥接
2. **dialog 不能简单适配**，因为它是插件级能力，需要独立的 ArkTS 模块和 PluginManager 注册
3. **当前架构是合理的**，与 Android/iOS 的插件架构一致，长期可扩展
4. **cfg 修改量不可避免**，但可以通过更好的 PR 组织方式降低认知负担

### 三种适配模式

| 模式 | 适用场景 | 示例 |
|------|---------|------|
| **直接调用** | 插件功能已被 openharmony-ability 覆盖 | process |
| **平行替换** | 插件的 OHOS 实现与桌面端完全不同 | updater |
| **插件注册** | 插件是插件级能力，需要独立 ArkTS 模块 | dialog |

### openharmony-ability vs tauri 核心 HAR

| | openharmony-ability | tauri 核心 HAR |
|---|---------------------|----------------|
| **职责** | 框架级原生能力桥接 | 插件注册框架 |
| **包含** | 生命周期、窗口、WebView、菜单、updater | Plugin.ets, PluginManager.ets |
| **定位** | Rust/C 与 ArkTS 的底层交互层 | tauri 插件系统的 OHOS 基础设施 |
| **ohpm 包** | `@ohos-rs/ability` | `@tauri/app` |

### 架构决策的合理性

- ✅ PluginManager 模式与 Android/iOS 一致
- ✅ 职责分离清晰（框架级 vs 插件级）
- ✅ 长期可扩展性好
- ⚠️ 短期改动量大（但不可避免）
- ⚠️ 需要文档指导后续插件适配

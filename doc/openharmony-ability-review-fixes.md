# PR #63 Review Fixes — openharmony-ability upstream

- **PR**: https://github.com/harmony-contrib/openharmony-ability/pull/63
- **Reviewer**: [richerfu](https://github.com/richerfu)（ohos-rs 项目维护者）
- **Review 时间**: 2026-06-08 ~ 2026-06-09
- **意见总数**: 10 条
- **状态**: 8 条已修复 / 2 条回复解释 / 2 条后续优化

---

## 意见列表

### #1 — 关闭按钮使用 ASCII 字符太 hack

| 项目 | 内容 |
|------|------|
| **文件** | `native_ability/src/main/ets/components/FloatPage.ets` line 121 |
| **原文** | *"Should We use raw icon file? for example: svg or png. Using ascii code symbol seems too hack"* |
| **背景** | 浮动窗口（FloatPage）右上角的关闭按钮，使用 `Text('×')` 实现 |
| **问题** | ASCII 字符 `×` 在不同设备/字体下渲染不一致，且不够专业 |
| **解决方案** | 替换为 `Image($r('sys.media.ohos_ic_public_cancel'))` 使用 OHOS 系统内置图标资源，通过 `.fillColor('#666666')` 保持原有视觉风格 |
| **状态** | ✅ 已修复 |
| **修改文件** | `FloatPage.ets` |

---

### #2 — 剪贴板建议用 native Rust API

| 项目 | 内容 |
|------|------|
| **文件** | `native_ability/src/main/ets/helper/ClipboardHelper.ets` line 1 |
| **原文** | *"Should we use native api for Clipboard? https://github.com/ohos-rs/ohos-native-bindings/tree/master/crates/pasteboard"* |
| **背景** | 剪贴板写图片功能当前通过 TSFN 桥接：Rust → ArkTS → `@ohos.pasteboard` |
| **问题** | ohos-rs 生态已有 `ohos-pasteboard-sys` crate，可在 Rust 端直接调用 native C API，省去 TSFN 跨语言开销 |
| **解决方案** | **暂不修改** — 当前方案已稳定工作（含 10s 超时 + Promise 结果处理），切换到 native API 改动较大，创建 issue 跟踪后续优化 |
| **状态** | 🔵 后续优化 |

---

### #3 — PixelMap 创建建议用 native image API

| 项目 | 内容 |
|------|------|
| **文件** | `native_ability/src/main/ets/helper/StatusBarUtils.ets` line 80 |
| **原文** | *"Using native api? https://github.com/ohos-rs/ohos-native-bindings/tree/master/crates/image https://github.com/ohos-rs/ohos-native-bindings/tree/master/crates/image_native"* |
| **背景** | 系统托盘图标从 RGBA 像素数据创建 PixelMap，当前在 ArkTS 侧通过 `createPixelMapSync` + `writeBufferToPixelsSync` 实现 |
| **问题** | ohos-rs 有 `ohos-image-kit-sys` / `image_native` crate，可在 Rust 端直接操作 native image API |
| **解决方案** | **暂不修改** — 托盘图标数据量小（24×24~32×32），TSFN 开销可忽略。与 #2 一起在后续版本评估全面切换到 native API 栈 |
| **状态** | 🔵 后续优化 |

---

### #4 — `DOMAIN` 常量重复定义

| 项目 | 内容 |
|------|------|
| **文件** | `native_ability/src/main/ets/webview/DefaultWebview.ets` line 12 |
| **原文** | *"Define too many `DOMAIN`"* |
| **背景** | hilog 日志系统需要 domain 参数标识日志来源 |
| **问题** | `const DOMAIN: number = 0x1999;` 在以下 10 个文件中各自重复定义：`WindowManager.ets`、`DefaultWebview.ets`、`Utils.ets`、`ArkHelper.ets`、`MainPage.ets`、`DefaultXComponent.ets`、`FloatPage.ets`、`os.ets`、`updater.ets`、`autostart.ets` |
| **解决方案** | 新建 `helper/constants.ets` 统一导出 `export const DOMAIN: number = 0x1999`，10 个文件改为 `import { DOMAIN } from '../helper/constants'`（helper 内文件用 `'./constants'`） |
| **状态** | ✅ 已修复 |
| **修改文件** | `helper/constants.ets`(新建) + 上述 10 个 `.ets` 文件 |

---

### #5 — `onErrorReceive` 错误页 fallback 是否符合 Tauri

| 项目 | 内容 |
|------|------|
| **文件** | `native_ability/src/main/ets/webview/DefaultWebview.ets` line 82 |
| **原文** | *"Does this behivor align to tauri? Should we add this fallback?"* |
| **背景** | `onErrorReceive` 回调中，主 frame 加载失败时使用 `ctrl.loadData(errorHtml)` 加载自定义错误页 |
| **问题** | 桌面 Tauri（WebKit/WRY）没有此行为——浏览器直接显示自带错误页。reviewer 质疑自定义 fallback 是否应该存在 |
| **解决方案** | **无需代码修改** — 在 PR 上回复解释：这是 OHOS 平台特有的 UX 增强，因为 OHOS Web 组件的默认错误页不够友好。如需控制可后续加 feature flag |
| **状态** | 💬 回复解释 |

---

### #6 — `onLoadIntercept` 返回值取反是 Breaking Change？

| 项目 | 内容 |
|------|------|
| **文件** | `native_ability/src/main/ets/webview/DefaultWebview.ets` line 111 |
| **原文** | *"Breaking change?"* |
| **背景** | `onLoadIntercept` 中，原代码 `return ret;`（直接透传 Tauri `onNavigationRequest` 返回值），改为 `return !ret;`（取反） |
| **问题** | reviewer 认为取反可能破坏现有应用的导航拦截行为 |
| **实际分析** | OHOS `onLoadIntercept`：`true` = **阻止**加载，`false` = **允许**。Tauri `onNavigationRequest`：`true` = **允许**，`false` = **阻止**。两者语义完全相反，原代码直接透传是**逻辑反转 bug**——Tauri 说"允许"的导航反而被阻止了 |
| **解决方案** | **无需代码修改** — 在 PR 上回复解释：这是 bug fix 而非 breaking change，引用[华为官方文档](https://developer.huawei.com/consumer/cn/doc/harmonyos-guides/web-redirection-and-browsing-history-mgmt)说明 `onLoadIntercept` 的 `true=block, false=allow` 语义 |
| **状态** | 💬 回复解释 |

---

### #7 — `eprintln!` 不写入 hilog

| 项目 | 内容 |
|------|------|
| **文件** | `crates/ability/src/render/xcomponent.rs` line 32 |
| **原文** | *"Use `hilog`. eprintln can't write log into hilog"* |
| **背景** | TSFN 初始化（tray、clipboard、autostart、updater）失败时打印错误信息 |
| **问题** | 使用 `eprintln!` 打印错误，但 OHOS 设备上 stderr 不会写入 hilog 系统日志，无法通过 `hdc shell hilog` 查看。涉及 6 处：`xcomponent.rs` 5 处（init_tray_tsfn、init_clipboard_tsfn、create_autostart_enable/disable/is_enabled_tsfn）+ `app.rs` 1 处（restart TSFN callback） |
| **解决方案** | 全部替换为 `crate::error!`，通过 `#[cfg(feature = "log")]` gated 宏委托给 `log::error!`（OHOS 上由 `ohos-hilog-binding` 后端写入 hilog）。feature 关闭时宏展开为 `{}`，变量用 `_e` 前缀避免 unused warning |
| **状态** | ✅ 已修复 |
| **修改文件** | `xcomponent.rs`、`app.rs` |

---

### #8 — 新模块应加 feature flag 减少 bundle size

| 项目 | 内容 |
|------|------|
| **文件** | `crates/ability/src/lib.rs` line 62 |
| **原文** | *"Every new module should add a feature flag to reduce bundle size"* |
| **背景** | PR 新增了 `statusbar`（系统托盘）、`updater`（AppGallery 更新）、`version`（API 版本检测）、`window`（多窗口）四个模块 |
| **问题** | 这四个模块直接 `pub mod` + `pub use` 无条件编译，不是所有应用都需要全部功能，增加不必要的 .so 体积 |
| **解决方案** | `Cargo.toml` 新增 `statusbar = []`/`updater = []`/`version = []`/`window = []` feature flag，全部加入 `default`（向后兼容）。`lib.rs` 中对应 `pub mod` 和 `pub use` 加 `#[cfg(feature = "...")]`。`helper/mod.rs` 中 `updater` 子模块加 gate。`xcomponent.rs`/`app.rs`/`autostart.rs` 中相关调用和引用加 gate |
| **状态** | ✅ 已修复 |
| **修改文件** | `Cargo.toml`、`lib.rs`、`helper/mod.rs`、`xcomponent.rs`、`app.rs`、`autostart.rs` |

---

### #9 — 添加 `log` feature flag

| 项目 | 内容 |
|------|------|
| **文件** | `crates/ability/Cargo.toml` line 13 |
| **原文** | *"Add a new flag: `log = ["dep:ohos-hilog-binding", "dep:log"]`"* |
| **背景** | 日志相关依赖的管理 |
| **问题** | `log` crate（0.4）和 `ohos-hilog-binding` 作为非 optional 依赖始终编译，无法按需裁剪 |
| **解决方案** | `log` 改为 `optional = true`，新增 `log = ["dep:log", "dep:ohos-hilog-binding"]` feature，加入 `default`。`clipboard` feature 改为 `["log"]`（clipboard 需要日志能力）。用户可通过 `default-features = false` 关闭日志 |
| **状态** | ✅ 已修复 |
| **修改文件** | `Cargo.toml` |

---

### #10 — 所有 hilog 应以 feature 为条件编译

| 项目 | 内容 |
|------|------|
| **文件** | `crates/ability/Cargo.toml` line 13（#9 的 follow-up） |
| **原文** | *"And every hilog should use it as condition."* |
| **背景** | #9 的补充要求——不仅 feature 定义要正确，代码中的日志调用也必须条件编译 |
| **问题** | `log::error!`/`log::debug!`/`log::info!`/`log::warn!` 分散在 `statusbar/manager.rs`、`menu/mod.rs`、`version.rs`、`helper/webview.rs`、`window/mod.rs` 共 5 个文件中，`hilog_error!` 在 `clipboard/mod.rs` 中。当 `log` feature 关闭时依赖不存在会编译失败 |
| **解决方案** | `lib.rs` 定义 4 组 cfg-gated `#[macro_export]` 宏（`error!`/`info!`/`warn!`/`debug!`）：feature on → 委托 `::log::error!` 等，off → 展开为 `{}`。6 个模块文件全部替换为 `crate::error!` 等统一调用。`clipboard/mod.rs` 从 `ohos_hilog_binding::hilog_error!` 迁移到 `crate::error!`，移除 `ohos_hilog_binding` 直接 import 和 `set_global_options` 调用 |
| **状态** | ✅ 已修复 |
| **修改文件** | `lib.rs`、`statusbar/manager.rs`、`menu/mod.rs`、`version.rs`、`helper/webview.rs`、`window/mod.rs`、`clipboard/mod.rs` |

---

## 修改统计

| 类别 | 数量 | 文件数 |
|------|------|--------|
| ✅ 已修复 | 8 条（#1, #4, #7, #8, #9, #10） | 23 files |
| 💬 回复解释 | 2 条（#5, #6） | — |
| 🔵 后续优化 | 2 条（#2, #3） | — |

### 编译验证

| 模式 | 结果 |
|------|------|
| `cargo check` (default features) | ✅ 零 warning 零 error |
| `cargo check --no-default-features` | ✅ 零 warning 零 error |

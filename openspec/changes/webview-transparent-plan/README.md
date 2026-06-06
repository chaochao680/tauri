# WebView 容器透明支持 适配计划

**创建时间**：2026-06-04
**审计时间**：2026-06-04（全 Phase 设计审查，修复 14 个问题）
**实施时间**：2026-06-04（Phase 1-3 代码实施完成，待设备端验证）
**补充时间**：2026-06-05（Phase 5: tauri webview plugin 命令注册修复）
**归档时间**：2026-06-06（Phase 1-5 全部归档）
**功能描述**：WebView 容器透明支持（透明背景 + 无标题栏 + 窗口背景透明）
**判断依据**：涉及 3 个代码层（openharmony-ability + tao + wry），预估 20+ 个文件（跨 4 个 Phase）

## Phase 列表

| Phase | 名称 | openspec change | 状态 | 涉及层 | 预估文件 | 验证方式 |
|-------|------|----------------|------|--------|---------|---------|
| 1 | WebView 透明背景 | p1-webview-transparent | ✓ 已归档 | openharmony-ability + wry | 7 | 设备端测试 |
| 2 | 无标题栏窗口 | p2-window-decorations | ✓ 已归档 | tao + openharmony-ability + ArkTS | 8 | 设备端测试 |
| 3 | 窗口背景透明 | p3-window-background | ✓ 已归档 | tao + openharmony-ability | 5 | 设备端测试 |
| 4 | 差距修复 | p4-gap-fix | ✓ 已归档 | 各层 | 待定 | 端到端测试 |
| 5 | WebView 透明命令注册修复 | p5-webview-transparent-cmd | ✓ 已归档 | tauri webview plugin | 1 | 设备端测试 |

## 实施总结

### 已完成的功能
- ✅ Float 子窗口完整透明穿透（decorations=false + transparent=true）
- ✅ 主窗口 decorations 运行时切换（标题栏显示/隐藏）
- ✅ 主窗口窗口壳层背景色运行时修改（setWindowBackgroundColor）
- ✅ WebView 组件创建时透明（Color.Transparent + SYNC_RENDER）
- ✅ webview.setBackgroundColor() 命令全平台注册（Phase 5 修复）
- ✅ Phase 4 差距修复：initMainWindow + WindowCreateParams + rgba_to_ohos_color

### OHOS 平台限制（已确认）
- ❌ **主窗口 Web 引擎渲染表面不支持透明** — OHOS 的 Web 组件在 UIAbility 主窗口中，即使设置 `.backgroundColor(Color.Transparent)` + `RenderMode.SYNC_RENDER`，Web 引擎内部渲染面仍为不透明。这是 OHOS 平台限制，非代码问题。
- ❌ **setWindowContainerColor inactive alpha 强制 FF** — OHOS 系统强制 inactive 颜色的 alpha 为不透明（#FF），无法设置透明。
- ✅ Float 子窗口不受此限制，可完整穿透到桌面。

### 关键文件修改清单
| 仓库 | 文件 | 修改内容 |
|------|------|---------|
| openharmony-ability | `crates/ability/src/window/mod.rs` | WindowCreateParams 结构体 + init_main_window NAPI |
| openharmony-ability | `native_ability/.../ability/ArkHelper.ets` | WebView 始终透明 + initMainWindow handler |
| openharmony-ability | `native_ability/.../ability/type.ets` | ArkHelper 接口新增 initMainWindow |
| openharmony-ability | `native_ability/.../window/WindowManager.ets` | initMainWindow 方法 + containerColor 修复 |
| openharmony-ability | `native_ability/.../components/MainPage.ets` | 4 层容器透明背景 |
| tao | `src/platform_impl/ohos/mod.rs` | set_background_color 不被 transparent 锁死 |
| tauri | `examples/api/src-tauri/src/lib.rs` | 主窗口 .transparent(true) |
| tauri | `crates/tauri/src/webview/plugin.rs` | commands 模块（全平台 set_webview_background_color） |

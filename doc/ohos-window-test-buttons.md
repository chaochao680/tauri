# OHOS 窗口能力 — 手动测试按钮清单

> 创建时间: 2026-08-10
> 最后更新: 2026-08-20(IME 位置双重误判纠正 + inner_position 标题栏补偿;此前 2026-08-19 光标抓取实现)
> 数据来源: [ohos-window-test-mapping.md](ohos-window-test-mapping.md)(能力表)+ TestRunner.svelte 实际按钮
> 用途: 对照本清单在设备上挨个点按钮测试窗口能力

## ⚠️ 两个重要提醒

1. **mapping 文档的 `#NN` 编号已失效** — git pull 后测试集从 220 项增长,编号整体错位。**按测试名找,别按号**。
2. **不要跑 Run All 全量自动测试** — 会触发 OnSizeChange 事件风暴导致 appfreeze(THREAD_BLOCK_6S)。手动点单个按钮测试是安全的。

## 测试前提

- 设备已连接(`hdc list targets` 可见,HUAWEI MateBook Pro HAD-W32)
- api 应用已构建安装启动(包名 `com.tauri.api`)
- 打开 TestRunner 页面,滚动到底部 **Manual Tests** 区

---

## A. 有手动按钮的能力(直接点按钮测)

全部在 TestRunner 页面底部 Manual Tests 区,按分区列出。每个按钮点完会在按钮下方显示 `manualResult`(操作结果 + 预期判据)。

### 🟦 顶部通用区

| 能力 | 按钮 | 预期 |
|------|------|------|
| 窗口大小/位置获取 | `Window DPI (resize/drag to verify)` | 显示 innerSize/outerSize/outerPosition |
| 配置获取(scaleFactor) | `currentMonitor` | 返回分辨率 + scaleFactor + position |
| 窗口聚焦(isFocused) | `isFocused (should be true)` | 返回 true |
| 窗口聚焦(onFocusChanged) | `Watch onFocusChanged`(toggle) | 切后台再回来,停止看事件数 > 0 |

### 🟦 Mouse Events (OHOS desktop / 2in1)

| 能力 | 按钮 | 预期 |
|------|------|------|
| 光标位置(读) | `Get Cursor Position` | 返回当前光标 X/Y |
| 光标位置(实时) | `Start Mouse Tracking`(toggle) | 移动鼠标,坐标实时刷新 |

### 🟦 Window Decorations & Transparency (Phase 1+2+3)

| 能力 | 按钮 | 预期 |
|------|------|------|
| 窗口装饰 | `Toggle Decorations (main window)` | 装饰开关 |
| 窗口创建(无装饰) | `Create Borderless Window (decorations=false)` | 弹出无装饰子窗口 |
| 窗口透明度(创建期) | `Create Transparent+Borderless` | 弹出透明无装饰子窗口 |
| 窗口创建(有装饰) | `Create Decorated Window (title bar)` | 弹出有标题栏+按钮的子窗口 |

### 🟦 Window Background Color (Phase 3)

> 先创建子窗口(Create Borderless/Decorated),再点 BG 按钮改该子窗口背景色。
> 2026-08-27 修复说明:按钮在 OHOS 上同时设置**窗口层**(setWindowBackgroundColor)与
> **webview 层**(ArkWeb 组件 backgroundColor,对齐 Rust API `WebviewWindow::
> set_background_color` 双层语义);测试子窗口页面背景已透明化(cmd.rs init script),
> 否则不透明 CSS 会盖住背景色。真机验证 Set BG Red 变红通过。

| 能力 | 按钮 | 预期 |
|------|------|------|
| 窗口背景色(不透明) | `Set BG Red (opaque)` | 子窗口背景变红 |
| 窗口背景色(半透明) | `Set BG Blue (alpha=128)` | 半透明蓝 |
| 窗口背景色(全透明) | `Set BG Green (alpha=0)` | 全透明(不可见) |
| 窗口背景色(重置) | `Reset BG (null)` | 恢复默认 |

### 🟦 Vibrancy (Window Effects) — OHOS

| 能力 | 按钮 | 预期 |
|------|------|------|
| 窗口效果 Blur | `vibrancy: Blur effect visible` | 新窗口背景模糊 |
| 窗口效果 Acrylic | `vibrancy: Acrylic effect visible` | 模糊 + 半透明黑 |
| 清除效果 | `vibrancy: clearEffects removes blur` | 新窗口无模糊 |
| 构建期效果 | `vibrancy: build-time Blur` | 验证不崩溃 |

### 🟦 OHOS Window Ops — 几何/状态

> setOuterPosition/setInnerSize 作用在最后创建的子窗口(需先 Create)。

| 能力 | 按钮 | 预期 |
|------|------|------|
| 窗口位置设置 | `setOuterPosition (toggle 100/400)` | 子窗口移动到 (100,100) 或 (400,400) |
| 窗口大小调整 | `setInnerSize (half size, restore)` | 子窗口缩到一半再还原 |
| 窗口最大化 | `Toggle Maximize` | 最大化/还原 |
| 窗口最小化 | `Minimize (2s restore)` | 最小化 2 秒后恢复 |
| 全屏模式 | `Toggle Fullscreen` | 全屏/退出(隐藏系统栏)。✅ 2026-08-27 修复:① WindowPlugin `set-fullscreen` action 迁移降级——pluginize 重构(ec27af6)把 action 迁到插件时写成 inline 纯手机路径(setWindowLayoutFullScreen),桌面 2in1 上视觉 no-op;已改委托 `WindowManager.setFullscreen`(双路径:桌面 maximize(ENTER_IMMERSIVE)+隐藏标题栏/Dock,手机沉浸式) ② tao `fullscreen()` rebase 时取了本地旧版硬编码返回 None→`isFullscreen` 恒 false→只能进不能退;已对齐 upstream 读镜像位(Borderless(None)) |
| 窗口可见性 | `Hide/Show (2s restore)` | ✅ 已修(主窗口:hide=minimize,show=startAbility instanceKey='main' 复用实例;2 秒后恢复) |
| 窗口聚焦 | `setFocus` | 子窗口 raiseToAppTop |
| 窗口置顶 | `Toggle AlwaysOnTop` | ✅ 已实现(setWindowTopmost API14+,跨应用常驻最前) |

### 🟦 OHOS Window Ops — 多 UIAbility 实例 (startAbility)

> ⚠️ 2026-08-27 定性:**两个按钮均为已知 deferred gap**(openspec upstream-ohdev-rebase-window-ops design.md 偏差 c),不是回归。upstream 的 `start_ui_ability` 多 UIAbility 建窗路径未移植,本地 tao 保留 single-UIAbility guard;当前 `launchType: singleton` 下 startAbility 只触发 onNewWant,不产生新实例窗口。点击表现为无新窗口(命令返回错误诊断)。留作后续专项移植。

| 能力 | 按钮 | 预期 |
|------|------|------|
| 窗口创建(多实例) | `Create UIAbility Instance Window` | ⏸️ deferred:多 UIAbility 建窗未移植,无新窗口为预期行为 |
| 窗口透明度(UIAbility) | `Create Transparent UIAbility` | ⏸️ deferred:同上(依赖多 UIAbility 路径) |

### 🟦 OHOS Window Ops — 装饰按钮 (子窗口生效)

> 需先创建子窗口(Create Decorated)。按钮始终显示,flag=false 时点击被 API 层拦截(问题四已修复:双层拦截 tao+ArkTS)。

| 能力 | 按钮 | 预期 |
|------|------|------|
| 窗口可关闭 | `Toggle Closable` | flag=false 时点关闭按钮关不掉(拦截) |
| 窗口可最大化 | `Toggle Maximizable` | flag=false 时点最大化按钮无效(拦截) |
| 窗口可最小化 | `Toggle Minimizable` | flag=false 时点最小化按钮无效(拦截) |
| 窗口可调整大小 | `Toggle Resizable` | flag=false 时 setInnerSize 被拦截 |
| 窗口可聚焦 | `setFocusable(false) (3s)` | 子窗口 3 秒内不可聚焦 |

### 🟦 OHOS Window Ops — 光标

| 能力 | 按钮 | 预期 |
|------|------|------|
| 光标可见性 | `setCursorVisible(false) (3s)` | ⏸️ deferred:upstream 自身 TODO-untested,本地维持 no-op(design.md 偏差 c),点击无效果为预期行为 |
| 光标图标 | `Cycle CursorIcon` | 循环切换光标样式(已修:用真实 windowId) |
| 忽略光标事件 | `Toggle IgnoreCursor (3s)` | 3 秒内鼠标穿透 |

### 🟦 OHOS Window Ops — 自动测试补充(无按钮能力)

| 能力 | 按钮 | 预期 |
|------|------|------|
| 窗口 ID | `Window ID (getCurrentWindow)` | label 非空(主窗口 "main") |
| 窗口销毁 | `CloseRequested (close sub-window)` | 建临时子窗口→关闭→收到事件 |
| 多窗口 | `on_new_window: Allow (window.open)` | 弹出新子窗口 |
| 光标抓取 | `setCursorGrab(true) 5s (Lock to window)` | ✅ 已实现(OH_WindowManager_LockCursor/UnlockCursor,NDK C API 22+,`LOCK_WINDOW_CURSOR` normal 权限已在 module.json5 声明)。点击锁定 5 秒:光标被限制在窗口内无法移出(窗口内仍可移动);期间点击其他窗口验证失焦自动解锁;5 秒后自动解锁恢复自由移动 |
| 窗口事件 | `Watch Window Events`(toggle) | 切后台触发 FocusChanged,停止看事件数 |
| 窗口状态持久化 | `window-state save+restore` | filename 非空,save+restore 幂等(已修:补偿标题栏高度) |
| set_bounds | `set_bounds round-trip (webview)` | set_ok === true |
| 窗口标题 | `Set Title (main window)` | ✅ 已实现(setWindowTitle API15+,主窗口标题栏+任务栏可见) |
| 窗口大小限制 | `Set Min Size 1600×1200 (main window)` | ✅ 已实现(setWindowLimits API11+,min 生效) |
| 窗口大小限制(重置) | `Reset Min Size (null)` | 设 min=1×1 恢复自由缩放 |
| 窗口主题 | `Set Theme (toggle Light/Dark/System)` | ✅ 已实现(setColorMode: LIGHT/DARK/NOT_SET 系统跟随) |
| 用户注意力请求 | `Request User Attention (notification)` | ✅ 已实现(notificationManager.publish + requestEnableNotification 授权)。首次点弹授权框,允许后右下角弹 "Tauri App / 请查看应用窗口" 通知 |
| IME 位置 | `Set IME Position (200,400)` | ✅ 已实现(`inputMethod.getController().updateCursor(CursorInfo)` API10+)。A/B 实测(2026-08-19):无聚焦输入框报 `12800009`(client detached);聚焦 HTML input 后返回 OK — **webview 场景完全可用,非架构限制**,唯一前置是调用时窗口内有聚焦的编辑框。按钮自验:自动聚焦输入框后上报,真实结果经 `get_ime_position_result` 回读显示在 manualResult |

---

## B. 只有自动测试、无独立按钮的能力

> **已全部补充手动按钮**(2026-08-10)。原 7 个"只有自动测试"的能力均已移入 A 区。
>
> **不建议跑 Run All 全量**(会触发 sizeChange 事件风暴导致 appfreeze)。

---

## C. 无测试入口的能力(空实现/平台限制,不用测)

### 真平台限制(系统无 API,空实现合理)

| 能力 | 原因 |
|------|------|
| 窗口图标 | 窗口层无运行时 API,只能 module.json5 静态配置 |
| 窗口置底 | 无置底 API(只有置顶 setWindowTopmost) |
| 窗口嵌入(子 WebView) | ✅ 已实现并真机验证(`WebViewBuilder::build_as_child` → `InnerWebView::new_as_child`,`is_child=true` 走 `WebViewStyle{x,y,w,h}`)。⚠️ 2026-08-19 修复:0cac4c3 曾把 ArkTS Web 组件宽高改回 "100%" 导致子 webview 实际为全窗口尺寸+位置偏移 → 右下溢出被窗口裁切(现象:右下角被挡显示不出);恢复显式宽高后真机验证通过(`create_webview (multi-webview)` 按钮,300×200@(50,50) 矩形完整)。`naturalLayout` 标记保证主 webview(创建时无 style.width)运行期 set_bounds 剥离宽高、保持 "100%" 跟随窗口 resize(0cac4c3 修复不回归) |
| 跨应用窗口嵌入 | 平台限制(把别的应用窗口/widget 嵌进本窗口:OHOS 无公开 API,`HalfScreenLaunchComponent` 仅元服务;与子 WebView 嵌入是两回事,勿混) |
| 折叠屏支持 | 只加了 Rust 侧框架（Event 变体 + tao 处理），但 ArkTS 侧还没注册监听。当前设备（MateBook Pro 2in1）不是折叠屏，无法验证 |

### ⚠️ 桥接已通但系统自动处理(no-op 合理)

| 能力 | API 情况 | 测试结果 |
|------|---------|----------|
| 请求重绘 | 无需 API(OHOS 由系统 vsync 自动驱动,每帧重绘) | no-op 合理。tao 调 ArkTS log,系统 VSyncGenerator 已自动驱动 MainEvent::WindowRedraw |

### 已实现(从 C 区移到 A 区)

| 能力 | 原状态 | 现状态 | 按钮 |
|------|--------|--------|------|
| 窗口标题 | `set_title(){}` 空 | ✅ setWindowTitle API15+ | `Set Title (main window)` |
| 窗口大小限制 | `set_min/max_inner_size(){}` 空 | ✅ setWindowLimits API11+ | `Set Min Size` / `Reset Min Size` |
| 窗口主题 | 部分实现 | ✅ 完整(含系统跟随 NOT_SET) | `Set Theme` |
| 窗口置顶 | no-op(只记 AtomicBool) | ✅ setWindowTopmost API14+ | `Toggle AlwaysOnTop` |
| 用户注意力请求 | no-op(空) | ✅ notificationManager.publish + requestEnableNotification 授权重试 | `Request User Attention (notification)` |
| 拖拽窗口 | Err(NotSupported) | ✅ FloatPage onTouch(Down) → win.startMoving() API14+ | (Create Decorated Window 后拖标题栏) |
| 拖拽调整大小 | Err(NotSupported) | ✅ set_window_draggable → enableDrag(主窗口边缘拖拽缩放) | (主窗口边缘直接拖) |
| 光标抓取 | Err(NotSupported)「平台限制」(误判:只 grep 了 ArkTS .d.ts,C API 仅在 NDK 暴露) | ✅ OH_WindowManager_LockCursor/UnlockCursor(NDK API22+,normal 权限;dlopen 弱加载,<22 设备降级 NotSupported;失焦自动解锁) | `setCursorGrab(true) 5s (Lock to window)` |
| IME 位置 | 「平台限制」双重误判(①"inputMethod 无位置 API";②"webview 不绑定 OHOS IMF client,走 Chromium 自己的输入法栈") | ✅ `inputMethod.getController().updateCursor(CursorInfo)` API10+。A/B 实测(2026-08-19):无聚焦输入框报 `12800009`(client detached,任何应用无聚焦编辑框皆如此,非 webview 特有);程序化聚焦 HTML input 后返回 OK — ArkWeb HTML input 走系统输入法(官方《Web 组件对接软键盘》),webview 场景可用 | `Set IME Position (200,400)`(自验:manualResult 显示真实结果) |
---

## 已修复的遗留问题

| 问题 | 状态 | 修复内容 |
|------|------|---------|
| 问题二:inner/outer 语义错位 | ✅ 已修(双侧闭环) | setter:`set_inner_size` 补偿标题栏高度(window_rect−content_rect);getter:`inner_position` 补 decor_height(2026-08-20,真机验证 inner(598,754)=outer(598,608)+146) |
| 问题三:hide/show 不对称 | ✅ 已修 | 主窗口:hide=`win.minimize()`(hideAbility 在 PC/2in1 不支持),show=`startAbility(instanceKey='main')` + AbilityStage onAcceptWant 复用实例(specified launchType,不爆发新窗口)。子窗口:minimize + showWindow 对称 |
| 问题四:装饰 flag 语义错位 | ✅ 已修 | 双层拦截(tao Rust + ArkTS WindowManager),flag=false 时 API 被拦截 |
| 问题五 5.1:僵尸字段 | ✅ 已修 | 删除 maximized/minimized 僵尸字段(is_* 直接查系统 API) |
| 问题五 5.2/5.3:状态不同步 | ⚠️ 待修 | visible/fullscreen 等单向写不回读,需补系统状态回灌 |

---

## 测试顺序建议

1. 先点顶部 `Window DPI` + `currentMonitor` 确认窗口基础几何读取正常
2. 安全测试(不改尺寸):`Set Title` / `Set Theme` / `Window ID` / `set_bounds`
3. 子窗口测试:先 `Create Decorated Window` → 测装饰按钮 / BG / setOuterPosition / setInnerSize / Hide-Show
4. 拖拽测试:`Create Decorated Window` 后拖动子窗口标题栏(startMoving);主窗口边缘拖拽缩放(enableDrag)
5. 通知测试:`Request User Attention` → 首次弹授权框→允许→再点弹通知
6. 光标测试:`Cycle CursorIcon` / `setCursorVisible` / `Toggle IgnoreCursor` / `setCursorGrab(true) 5s`(锁定期间移动鼠标验证无法移出窗口)
7. **避免**:`Set Min Size`(主窗口改尺寸触发 sizeChange 风暴)、`Run All` 全量自动测试
8. C 区跳过(空实现/平台限制)。IME 位置可测:点 `Set IME Position (200,400)`,manualResult 显示 `updateCursor 返回: OK ✅ → PASS` 即通过(按钮自动注入并聚焦输入框,真实结果经回读命令显示,无需看 hilog)

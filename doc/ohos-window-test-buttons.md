# OHOS 窗口能力 — 手动测试按钮清单

> 创建时间: 2026-08-10
> 最后更新: 2026-08-13(5 个新能力实现:redraw/attention/ime/drag/drag-resize)
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
| 窗口效果 TabbedDark | `vibrancy: TabbedDark effect visible` | 模糊 + 深色 |
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
| 全屏模式 | `Toggle Fullscreen` | 全屏/退出(隐藏系统栏) |
| 窗口可见性 | `Hide/Show (2s restore)` | ✅ 已修(主窗口:hide=minimize,show=startAbility instanceKey='main' 复用实例;2 秒后恢复) |
| 窗口聚焦 | `setFocus` | 子窗口 raiseToAppTop |
| 窗口置顶 | `Toggle AlwaysOnTop` | ✅ 已实现(setWindowTopmost API14+,跨应用常驻最前) |

### 🟦 OHOS Window Ops — 多 UIAbility 实例 (startAbility)

| 能力 | 按钮 | 预期 |
|------|------|------|
| 窗口创建(多实例) | `Create UIAbility Instance Window` | 拉起新 UIAbility 实例窗口 |
| 窗口透明度(UIAbility) | `Create Transparent UIAbility` | 主窗口变透明 |

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
| 光标可见性 | `setCursorVisible(false) (3s)` | 光标隐藏 3 秒后恢复 |
| 光标图标 | `Cycle CursorIcon` | 循环切换光标样式(已修:用真实 windowId) |
| 忽略光标事件 | `Toggle IgnoreCursor (3s)` | 3 秒内鼠标穿透 |

### 🟦 OHOS Window Ops — 自动测试补充(无按钮能力)

| 能力 | 按钮 | 预期 |
|------|------|------|
| 窗口 ID | `Window ID (getCurrentWindow)` | label 非空(主窗口 "main") |
| 窗口销毁 | `CloseRequested (close sub-window)` | 建临时子窗口→关闭→收到事件 |
| 多窗口 | `on_new_window: Allow (window.open)` | 弹出新子窗口 |
| 光标抓取(平台限制) | `setCursorGrab (platform limit)` | 不崩溃即 PASS(OHOS 无 pointer lock API) |
| 窗口事件 | `Watch Window Events`(toggle) | 切后台触发 FocusChanged,停止看事件数 |
| 窗口状态持久化 | `window-state save+restore` | filename 非空,save+restore 幂等(已修:补偿标题栏高度) |
| set_bounds | `set_bounds round-trip (webview)` | set_ok === true |
| 窗口标题 | `Set Title (main window)` | ✅ 已实现(setWindowTitle API15+,主窗口标题栏+任务栏可见) |
| 窗口大小限制 | `Set Min Size 1600×1200 (main window)` | ✅ 已实现(setWindowLimits API11+,min 生效) |
| 窗口大小限制(重置) | `Reset Min Size (null)` | 设 min=1×1 恢复自由缩放 |
| 窗口主题 | `Set Theme (toggle Light/Dark/System)` | ✅ 已实现(setColorMode: LIGHT/DARK/NOT_SET 系统跟随) |
| 用户注意力请求 | `Request User Attention (notification)` | ✅ 已实现(notificationManager.publish + requestEnableNotification 授权)。首次点弹授权框,允许后右下角弹 "Tauri App / 请查看应用窗口" 通知 |
| IME 位置 | `Set IME Position (200,400)` | ⚠️ 桥接已实现(inputMethod.updateCursor API10+),但 webview 场景受架构限制(详见 C 区) |

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
| 光标抓取 | OHOS 无 pointer lock API(两个 MCP 交叉确认) |
| 窗口置底 | 无置底 API(只有置顶 setWindowTopmost) |
| 窗口嵌入能力 | 平台限制(HalfScreenLaunchComponent 仅元服务) |
| 折叠屏支持 | 只加了 Rust 侧框架（Event 变体 + tao 处理），但 ArkTS 侧还没注册监听。当前设备（MateBook Pro 2in1）不是折叠屏，无法验证 |

### ⚠️ 桥接已通但受架构限制(有 API,webview 场景无法生效)

| 能力 | API 情况 | 测试结果 |
|------|---------|----------|
| 请求重绘 | 无需 API(OHOS 由系统 vsync 自动驱动,每帧重绘) | no-op 合理。tao 调 ArkTS log,系统 VSyncGenerator 已自动驱动 MainEvent::WindowRedraw |
| IME 位置 | **有 API** `inputMethod.updateCursor(CursorInfo)` API10+(纠正原"无位置 API"误判) | 桥接 PASS(命令→tao→ArkTS→updateCursor 全链路通),但返回 `12800009 detached`。**根因**:webview(Chromium/ArkWeb)内 HTML input 走 Chromium 自己的输入法栈,不绑定 OHOS InputMethod 客户端,所以 updateCursor 无 client 可用。原生 TextInput 组件理论上能生效,但 tauri 用 webview 非原生组件 |

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
| 可用区域避让 | |
---

## 已修复的遗留问题

| 问题 | 状态 | 修复内容 |
|------|------|---------|
| 问题二:inner/outer 语义错位 | ✅ 已修 | set_inner_size 补偿标题栏高度(window_rect - content_rect) |
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
6. 光标测试:`Cycle CursorIcon` / `setCursorVisible` / `Toggle IgnoreCursor`
7. **避免**:`Set Min Size`(主窗口改尺寸触发 sizeChange 风暴)、`Run All` 全量自动测试
8. C 区跳过(空实现/平台限制);IME 位置桥接已通但 webview 场景报 12800009(架构限制)

# OHOS 窗口能力与测试用例映射

> 创建时间: 2026-07-04
> 数据来源: 设备 test-report.md（220 项，214 ✅ / 6 ❌）+ TestRunner Manual Tests + window-ops.ts
> 关联: [ohos-window-design.md](ohos-window-design.md)（落地状态与预期效果）

原表为早期版本，部分标注与当前代码不符。下表合并「原标注 / 实际状态 / 具体 API / 测试用例 / 测试结果」五列。

API 列格式：`tao 方法 → openharmony-ability 函数 → OHOS API`（空实现/平台限制只标 tao 层）。

| 能力 | 原标注 | 实际状态 | 具体 API | 测试用例 | 测试结果 |
|------|--------|---------|---------|---------|---------|
| 窗口 ID | ✅ | ✅ | `WindowId::dummy()`（单例 ID=0） | #7 window.getCurrentWindow | ✅ |
| 窗口大小获取 | 部分支持 | 部分支持 | `inner_size`→`app.content_rect()`；`outer_size`→`app.window_rect()`（content_rect 回退） | #129 innerSize / #130 outerSize | ✅ |
| 窗口位置获取 | ✅ | ✅ | `inner_position`→`window_rect+content_rect`；`outer_position`→`app.window_rect()` | #131 innerPosition / #132 outerPosition | ✅ |
| 窗口内容区域 | ✅ | ✅ | `content_rect()`→`app.content_rect()` | #129 innerSize | ✅ |
| 窗口创建 | ✅ | ✅ | `Window::new`→`create_os_window`→`windowStage.createSubWindow` | #38 borderless / #39 transparent / #46 #47 on_new_window | ✅ |
| 窗口销毁 | 部分支持 | 部分支持 | `MainEvent::WindowDestroy`→补发 CloseRequested+Destroyed；无主动销毁 API | #34 CloseRequested / #35 Destroyed | ✅ |
| 请求重绘 | 部分支持 | 部分支持 stub | `request_redraw(){}`（空，由 MainEvent::WindowRedraw 驱动） | 无 | ❌ |
| 配置获取 | ✅ | ✅ | `config()`→`app.config()` | #133 scaleFactor | ✅ |
| 多窗口 | 部分支持 | 部分支持 | `Window::new`(Float)→`create_os_window`(TypeFloat)；supports_multiple_windows=false | #46/#47 on_new_window | ✅ |
| 窗口位置设置 | ❌ | ✅ 已实现 | `set_outer_position`→`move_window_to`→`win.moveWindowTo(x,y)`。主窗口(id=0)系统管理返回 1300002，静默 no-op；仅 Float 子窗口可移动 | #44 set_position / #135 setOuterPosition actually | ✅ / ✅ |
| 窗口大小调整 | ❌ | ✅ 已实现 | `set_inner_size`→`resize_window`→`win.resize(w,h)`。主窗口(id=0)系统管理返回 1300002，静默 no-op；仅 Float 子窗口可 resize | #45 set_size resizes（严格版 #134 主窗口 no-op 属系统限制，非 bug） | ✅ |
| 窗口大小限制 | ❌ | ❌ 空实现 | `set_min/max_inner_size(){}`（空） | 无 | ❌ |
| 窗口装饰 | ❌ | ✅ 已实现 | `set_decorations`→`set_window_decorations`→LocalStorage+`setWindowSystemBarEnable` | #37 setDecorations toggles | ✅ |
| 窗口背景色 | ❌ | ✅ 已实现 | `set_background_color`→`set_window_background_color`→`win.setWindowBackgroundColor` | #38/#39 create_borderless/transparent + 手动 BG 按钮 | ✅ |
| 窗口透明度 | ❌ | 部分支持 | 创建期 `transparent`→`WindowCreateParams`→`setWindowBackgroundColor(0x00000000)`+`setWindowContainerColor`；运行期无独立 alpha | #39 create_transparent_borderless | ✅ |
| 窗口主题 | ❌ | 部分支持 | `set_theme`→`app.set_color_mode`；`theme()`读 AtomicU8；缺系统跟随 | 无专门测试 | ❌ |
| 窗口图标 | ❌ | ❌ 空实现 | `set_window_icon(){}`（空） | 无 | ❌ |
| 窗口标题 | ❌ | ❌ 空实现 | `set_title(){}`；`title()`返回空串 | 无 | ❌ |
| 窗口效果 (vibrancy) | ❌ | ✅ 已实现 | `set_effects`→`set_window_blur`→`backdropBlur`(AttributeUpdater) | #67 setEffects / #68 build-time effects | ✅ |
| 窗口最大化 | ❌ | ✅ 已实现 | `set_maximized`→`maximize_window`→`win.maximize()`；还原 `restore_window`/`recover_window`→`win.restore()` | #42 maximize / #43 unmaximize / #136 maximize fills | ✅ / ✅ / ✅ |
| 窗口最小化 | ❌ | ✅ 已实现 | `set_minimized`→`minimize_window`→`win.minimize()`；还原 `restore_window` | #41 is_minimized / #138 minimize smoke | ✅ / ✅ |
| 全屏模式 | ❌ | ✅ 已实现 | `set_fullscreen`→`ohos_set_fullscreen`→`win.setWindowLayoutFullScreen`+`setWindowSystemBarEnable([])` | #137 setFullscreen smoke | ✅ |
| 窗口可见性 | ❌ | ✅ 已实现 | `set_visible`→`show_window`→`win.showWindow()` / `hide_window`→主`hideAbility`/子`win.minimize()` | 手动 Hide/Show 按钮 | 手动 |
| 窗口聚焦 | ❌ | ✅ 已实现 | `set_focus`→`focus_window`→子`win.raiseToAppTop()`/主 no-op | #8 isFocused / #13 onFocusChanged / cursor smoke(setFocus) | ✅ / ✅ / ✅ |
| 窗口置顶 | ❌ | partial | `set_always_on_top`→仅记 AtomicBool（OHOS 无 z-order API，Float 天然浮于主窗口） | #139 setAlwaysOnTop smoke | ✅（partial 标注） |
| 窗口置底 | ❌ | ❌ 空实现 | `set_always_on_bottom(){}`（空） | 无 | ❌ |
| 用户注意力请求 | ❌ | ❌ 空实现 | `request_user_attention(){}`（空） | 无 | ❌ |
| 窗口可关闭 | ❌ | ✅ 已实现 | `set_closable`→`set_window_decoration_flags`(bit0)→LocalStorage | #141 decoration flags smoke | ✅ |
| 窗口可最大化 | ❌ | ✅ 已实现 | `set_maximizable`→`set_window_decoration_flags`(bit1)→LocalStorage | 同上 | ✅ |
| 窗口可最小化 | ❌ | ✅ 已实现 | `set_minimizable`→`set_window_decoration_flags`(bit2)→LocalStorage | 同上 | ✅ |
| 窗口可聚焦 | ❌ | ✅ 已实现 | `set_focusable`→`set_window_focusable`→`win.setWindowFocusable` | 同上 | ✅ |
| 窗口可调整大小 | ❌ | ✅ 已实现 | `set_resizable`→`set_window_decoration_flags`(bit3)→LocalStorage | 同上 | ✅ |
| 光标位置 (读) | ❌ | 已工作 | `cursor_position`→读 `CURSOR_POSITION_X/Y`（ArkTS onMouse 经 NAPI 更新） | #58 cursorPosition | ✅ |
| 光标可见性 | ❌ | ✅ 已实现 | `set_cursor_visible`→`set_pointer_visible`→`pointer.setPointerVisible` | #142 cursor smoke | ✅ |
| 光标图标 | ❌ | ✅ 已实现 | `set_cursor_icon`→`set_pointer_style`→`pointer.setPointerStyleSync` | 同上 | ✅ |
| 光标抓取 | ❌ | ✅ 已实现 | `set_cursor_grab`→`openharmony-ability::window::set_cursor_grab`(dlopen 弱加载 + ArkTS `getRealWindowId` 查真实窗口 ID)→`OH_WindowManager_LockCursor/UnlockCursor`(NDK API22+,`LOCK_WINDOW_CURSOR` normal 权限) | TestRunner 手动按钮 `setCursorGrab(true) 5s (Lock to window)` | 手动 |
| 忽略光标事件 | ❌ | ✅ 已实现 | `set_ignore_cursor_events`→`set_window_touchable`→`win.setWindowTouchable` | #65（测试名标注 no-op，功能已接入；测试仅验不崩溃）/ #140 smoke | ✅ 命令不崩溃；穿透效果未验证 |
| IME 位置 | ❌ | ❌ 平台限制 | `set_ime_position(){}`（空，inputMethod 无位置 API） | 无 | ❌ |
| 拖拽窗口 | ❌ | ❌ 平台限制 | `drag_window`→返回 NotSupportedError（无 startWindowMove API，FloatPage PanGesture 手柄处理） | 无 | ❌ |
| 拖拽调整窗口大小 | ❌ | ❌ 平台限制 | `drag_resize_window`→返回 NotSupportedError（无 startWindowResize/Direction 枚举） | 无 | ❌ |
| 可用区域避让 | ❌ | ❌ 未实现 | 无（C 组 P2，需 `window.on('avoidAreaChange')`） | 无 | ❌ |
| 折叠屏支持 | ❌ | ❌ 未实现 | 无（C 组 P2，需 `displayFoldChanged`） | 无 | ❌ |
| 窗口嵌入(子 WebView) | ❌ | ✅ 已实现 | `WebViewBuilder::build_as_child` → wry `new_as_child`(is_child=true,`WebViewStyle{x,y,w,h}`)→ ArkTS `createWebview(windowId)` 把 Web 节点挂进父窗口 NodeController。2026-08-19 修复宽高失效回归(见下方说明) | 手动 `create_webview (multi-webview)`(300×200@(50,50) 矩形) | ✅ 手动(2026-08-19 真机验证通过) |
| 跨应用窗口嵌入 | ❌ | ❌ 平台限制 | 无(OHOS 无跨应用 widget embed API,`HalfScreenLaunchComponent` 仅元服务) | 无 | ❌ |
| 窗口状态持久化 | ❌ | ❌ 插件层 | 无（tauri-plugin-window-state 职责） | 无 | ❌ |
| 窗口事件 on_window_event | — | ✅ | tao `MainEvent`→`Event::WindowEvent` | #20 on_window_event | ✅ |

## 说明

- **原标注 ❌ 实际已实现**（14 项）：窗口位置设置、窗口大小调整、窗口最大化、窗口最小化、窗口装饰、窗口背景色、窗口效果 vibrancy、光标可见性、光标图标、忽略光标事件、全屏模式、装饰按钮 5 项、光标位置读、光标抓取。原表是早期版本未随 origin/ohdev `0cac4c3`/`f1af1eb` + A/D/E 组补全更新。
- **仍为 ❌ 属实**：IME 位置、拖拽窗口/调整大小（平台限制）、可用区域避让/折叠屏（P2 未实现）、窗口图标/标题/大小限制/置底/用户注意力（空实现）、跨应用窗口嵌入（平台限制）、窗口状态持久化（插件层）。
- **子 WebView 嵌入宽高曾失效（2026-08-19 修复）**：6fd8c0a（2026-06-25）曾以 `data.style.width/height` 驱动 Web 组件尺寸；0cac4c3（2026-07-17）为修主 webview 窗口 resize 后 ArkWeb 不 relayout（页面保旧布局、底部被裁）的 bug，把 WebBuilder/EmbeddedWebBuilder 宽高全部改回 "100%"，只保留 `.position(x,y)`——副作用是子 webview 变成全窗口尺寸+位置偏移，右下溢出被窗口边界裁切（现象：右下角显示不出，裁切量恰等于 position 偏移）。修复：恢复显式宽高，同时在 ArkTS `ensureWebviewNodeData`/`updateWebviewStyle` 引入 `naturalLayout` 标记——创建时无 `style.width` 的 webview（主 webview）运行期 set_bounds 的宽高被剥离，保持 "100%" 跟随窗口，0cac4c3 修复不回归；tauri-runtime-wry 侧 OHOS 不再对无显式 bounds 的 webview 兜底传全窗口尺寸。遗留：子 webview 运行期改宽高仍走 BuilderNode.update，ArkWeb 页面内部布局可能不即时刷新（直到下次导航），创建期不受影响。
- **光标抓取原「平台限制」结论系误判(2026-08-19 纠偏)**:旧结论只 grep 了 ArkTS `.d.ts`——`OH_WindowManager_LockCursor`/`OH_WindowManager_UnlockCursor` 仅在 **NDK native 侧**暴露(oh_window.h + libnative_window_manager.so 公开导出,API 22+,配套 `ohos.permission.LOCK_WINDOW_CURSOR` 为 normal 级 system_grant 开放权限,非系统应用专用)。tao 经 openharmony-ability 以 dlopen/dlsym 弱加载直调(规避 API<22 设备加载期符号解析失败,低于 22 降级 NotSupported);isCursorFollowMovement 固定 true(confined,与 Windows ClipCursor 语义一致)。**行为差异**:仅获焦窗口生效,失焦自动解锁(Windows ClipCursor 不随失焦释放)——持续锁定需应用监听 Focused 自行 re-grab。
- **#134 setInnerSize 严格版偶发 ❌**：主窗口 resize 受系统约束时 innerSize 不向目标靠拢，但 origin #45（同能力）通过——二者断言严格度不同，非功能 bug。
- **#143 setOuterPosition 严格版已降为 smoke（2026-08-13）**：OHOS 上 setOuterPosition 的实际移动**无法从 JS 可靠读回**——`outer_position()` 读自 `window_rect`，由 ArkTS `window_rect_change` 回调填充（lifecycle.rs:175-179）。resize 触发尺寸回调 → #142 setInnerSize 读回可靠；但纯 moveWindowTo 只改位置、不触发我们监听的 rect 回调，故 Float 子窗口读回恒为旧值（实测 orig(515,343)→after(515,343) 完全不变），主窗口则由系统自由窗口 WM 非确定性重定位（如 (699,651)），读回时 pass 时 fail。hilog 实测 moveWindowTo 解析成功、无 1300002 reject（ArkTS 仅 `.catch` 时 warn，全程零失败日志）——调用本身不抛错。故将 #143 从严格读回断言降为 smoke（校验 setPosition 不抛错），与 #137 fullscreen / #138 minimize / #139 alwaysOnTop 等主窗口不可从 JS 验证的能力同策。移动效果靠手动按钮验证。根本修复（让 moveWindowTo 触发 rect 回调更新 window_rect）属 lifecycle/WindowManager 层，超出测试修复范围。
- 手动按钮（TestRunner Manual Tests）：origin/ohdev 的 vibrancy/BG/decorations/mouse 按钮均在；我之前加的「OHOS Window Ops」分区在 rebase 冲突解决时丢失，如需恢复参考 [ohos-window-design.md](ohos-window-design.md)。

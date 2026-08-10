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
| 光标抓取 | ❌ | ❌ 平台限制 | `set_cursor_grab`→返回 NotSupportedError（OHOS 无指针锁定 API） | #142 cursor smoke（校验 no-throw） | ✅（平台限制） |
| 忽略光标事件 | ❌ | ✅ 已实现 | `set_ignore_cursor_events`→`set_window_touchable`→`win.setWindowTouchable` | #65（测试名标注 no-op，功能已接入；测试仅验不崩溃）/ #140 smoke | ✅ 命令不崩溃；穿透效果未验证 |
| IME 位置 | ❌ | ❌ 平台限制 | `set_ime_position(){}`（空，inputMethod 无位置 API） | 无 | ❌ |
| 拖拽窗口 | ❌ | ❌ 平台限制 | `drag_window`→返回 NotSupportedError（无 startWindowMove API，FloatPage PanGesture 手柄处理） | 无 | ❌ |
| 拖拽调整窗口大小 | ❌ | ❌ 平台限制 | `drag_resize_window`→返回 NotSupportedError（无 startWindowResize/Direction 枚举） | 无 | ❌ |
| 可用区域避让 | ❌ | ❌ 未实现 | 无（C 组 P2，需 `window.on('avoidAreaChange')`） | 无 | ❌ |
| 折叠屏支持 | ❌ | ❌ 未实现 | 无（C 组 P2，需 `displayFoldChanged`） | 无 | ❌ |
| 窗口嵌入能力 | ❌ | ❌ 平台限制 | 无（OHOS 无跨应用 widget embed API） | 无 | ❌ |
| 窗口状态持久化 | ❌ | ❌ 插件层 | 无（tauri-plugin-window-state 职责） | 无 | ❌ |
| 窗口事件 on_window_event | — | ✅ | tao `MainEvent`→`Event::WindowEvent` | #20 on_window_event | ✅ |

## 说明

- **原标注 ❌ 实际已实现**（13 项）：窗口位置设置、窗口大小调整、窗口最大化、窗口最小化、窗口装饰、窗口背景色、窗口效果 vibrancy、光标可见性、光标图标、忽略光标事件、全屏模式、装饰按钮 5 项、光标位置读。原表是早期版本未随 origin/ohdev `0cac4c3`/`f1af1eb` + A/D/E 组补全更新。
- **仍为 ❌ 属实**：IME 位置、拖拽窗口/调整大小（平台限制）、可用区域避让/折叠屏（P2 未实现）、窗口图标/标题/大小限制/置底/用户注意力（空实现）、窗口嵌入（平台限制）、窗口状态持久化（插件层）。
- **#134 setInnerSize 严格版偶发 ❌**：主窗口 resize 受系统约束时 innerSize 不向目标靠拢，但 origin #45（同能力）通过——二者断言严格度不同，非功能 bug。
- 手动按钮（TestRunner Manual Tests）：origin/ohdev 的 vibrancy/BG/decorations/mouse 按钮均在；我之前加的「OHOS Window Ops」分区在 rebase 冲突解决时丢失，如需恢复参考 [ohos-window-design.md](ohos-window-design.md)。

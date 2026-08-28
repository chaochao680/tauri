# Tauri OHOS 适配手动测试用例清单

> **生成日期**: 2026-06-01
>
> **测试入口**: `examples/api` 应用
>
> **级别说明**: T0 = 冒烟必测（核心功能/主流程）；T1 = 重要回归（辅助功能/边界场景）
>
> **用途**: 本文档归档 Tauri OHOS 适配的所有手动测试用例，涵盖各模块的必测场景。新模块适配完成后将用例追加至对应章节。

---

## 一、Tray（系统托盘）手动用例

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| core | tray | 创建与图标 | Full Test Tray — 创建托盘与图标显示 | **T0** | 应用已启动，进入 Tray 页面 | 1. 点击 "Full Test Tray" 按钮 2. 确认状态栏出现托盘图标 3. 左键点击托盘图标 | ① UI 输出 `Full test tray created` ② 状态栏显示托盘图标（32×32 默认图标） ③ 左键点击弹出 QuickOperation 系统面板，标题 "Tauri API"（QuickOp 面板拦截左键点击；验证 TrayIconEvent 输出需清空 abilityName，见 icon-click 用例 L308） | QuickOperation 配置：title="Tauri API"，height=300，abilityName="TestTrayAbility" |
| core | tray | 右键菜单显示 | Full Test Tray — 右键菜单结构与项类型 | **T0** | 已创建 Full Test Tray | 1. 右键点击（或长按）状态栏托盘图标 2. 检查菜单整体结构 3. 逐项检查各类型菜单项显示 | ① 弹出上下文菜单 ② 自定义项正确显示：Normal Item（普通文字）、Check Item（未勾选状态）、Icon Item（带图标+文字）、Another Normal（普通文字） ③ 分隔符正确渲染为分隔线 ④ 预定义项正确显示：Copy/Cut/SelectAll/Undo/Redo/Minimize/Maximize/Fullscreen/CloseWindow/Hide/Quit | 菜单共含 4 个自定义项 + 4 个分隔符 + 11 个预定义项（不含 Paste 和 3 个分隔符预定义项） |
| core | tray | 菜单项点击事件 | Full Test Tray — 自定义菜单项点击 | **T0** | 已创建 Full Test Tray；已右键打开菜单 | 1. 点击菜单中的 "Normal Item" 2. 重新打开菜单，点击 "Check Item" 3. 重新打开菜单，点击 "Icon Item" | ① 点击 Normal Item → Menu Event Log 输出 `[menu-event #N lid=1] global:normal-item at <时间>` ② 点击 Check Item → 输出 `[menu-event #N lid=1] global:check-item at <时间>` ③ 点击 Icon Item → 输出 `[menu-event #N lid=1] global:icon-item at <时间>` ④ 每次点击后菜单自动关闭 | 验证自定义 MenuItem action 回调 + Rust 全局事件转发 |
| core | tray | 预定义菜单项功能 | Full Test Tray — 预定义菜单项操作验证 | **T0** | 已创建 Full Test Tray；输入框有文本可用于剪贴板测试 | 1. 在输入框中选中一段文本 2. 右键打开托盘菜单，点击 Copy → 在另一处粘贴，验证复制成功 3. 重新选中输入框文本 4. 打开菜单，点击 Cut → 粘贴验证剪切成功 5. 打开菜单，点击 Minimize → 窗口最小化到任务栏，点击任务栏图标恢复窗口 6. 打开菜单，点击 Maximize → 窗口铺满全屏 7. 打开菜单，点击 Fullscreen → 进入沉浸式全屏，按 Esc 退出 8. 打开菜单，点击 Hide → 窗口隐藏，从任务栏点击恢复 9. 打开菜单，点击 CloseWindow → 窗口关闭 | ① Copy：文本被复制到剪贴板，Menu Event Log 输出 `global:copy` ② Cut：文本从输入框消失且被复制到剪贴板，输出 `global:cut` ③ Minimize：窗口最小化到任务栏，无闪烁（窗口不弹回前台） ④ Maximize：窗口铺满全屏 ⑤ Fullscreen：进入沉浸式全屏，菜单栏隐藏，Esc 恢复 ⑥ Hide：窗口隐藏，从任务栏点击可恢复 ⑦ CloseWindow：窗口关闭 ⑧ 每个操作 Menu Event Log 均有对应 id 输出 | **不测试 Paste**（OHOS 剪贴板读权限限制）；Quit 会退出应用，建议最后测试；Minimize 验证 minimizeWithRestoreGuard 已恢复（WINDOW_ACTIVE 竞态保护，hilog 标记 `minimizeWithRestoreGuard: minimizing (settled)`） |
| core | tray | 托盘创建 | Tray Page — 自定义参数创建托盘 | **T1** | 应用已启动，进入 Tray 页面 | 1. 填写 Title/Tooltip/Icon 等参数 2. 点击 "Create tray" 按钮 | 托盘图标按配置参数创建成功；状态栏显示对应图标；悬停显示 tooltip | 会先移除已有的 tray-1 和 manual-tray；OHOS 有 500ms 延迟 |
| core | tray | 托盘清理 | Tray Page — Remove All Trays | **T1** | 已创建过托盘图标 | 1. 点击 "Remove All Trays" 按钮 | 所有托盘图标（tray-1、manual-tray、full-test-tray）从状态栏消失 | 验证批量移除能力 |
| core | tray | QuickOperation | Enable QuickOp — 启用快速操作面板 | **T1** | 应用已启动；tray-1 已创建（Tray 页 "Create tray"）；TestTrayAbility 已在 module.json5 注册 | 1. 在 TestRunner 页 Manual Tests 区域点击 "Enable QuickOp" 按钮 2. 左键点击状态栏托盘图标 | 系统弹出快速操作面板，标题 "Test Panel"，高度 250vp | **仅 OHOS 平台**；需预注册 abilityName；按钮内部 `getById('tray-1')`，只对 tray-1 生效 |
| core | tray | QuickOperation | Update QuickOp — 更新快速操作参数 | **T1** | QuickOperation 已启用 | 1. 点击 "Update QuickOp" 按钮 2. 左键点击托盘图标 | 弹出面板标题变为 "Updated Title"，高度变为 400vp | **仅 OHOS 平台** |
| core | tray | QuickOperation | Disable QuickOp — 禁用快速操作 | **T1** | QuickOperation 已启用 | 1. 点击 "Disable QuickOp" 按钮 2. 左键点击托盘图标 | 不再弹出面板，仅触发点击事件 | **仅 OHOS 平台**；setQuickOperation(null) |
| core | tray | icon_as_template | Icon as Template — template 模式下深色/浅色壁纸适配 | **T0** | 应用已启动，进入 Manual Tests 区域 | 1. 点击 "Icon as Template (check wallpaper)" 按钮 2. 确认状态栏出现托盘图标 3. 切换系统深色/浅色壁纸 4. 观察状态栏图标颜色变化 | ① 托盘图标创建成功（iconAsTemplate=true） ② 深色壁纸下图标为白色版本（保持可见） ③ 浅色壁纸下图标为黑色版本（保持可见） ④ 切换后图标颜色自动适配，无需重建托盘 | **仅 OHOS 平台**；验证 `to_monochrome()` 生成的白/黑双色 PixelMap 正确工作 |
| core | tray | icon_as_template | White Icon NO Template — 非 template 模式对比验证 | **T1** | 应用已启动，进入 Manual Tests 区域 | 1. 点击 "White Icon NO Template (compare)" 按钮 2. 确认状态栏出现纯白托盘图标 3. 切换系统深色/浅色壁纸 4. 观察图标是否有变化 | ① 托盘图标创建成功（32×32 纯白 PNG，iconAsTemplate=false） ② 切换壁纸后图标**不变**，始终保持纯白色 ③ 与 "Icon as Template" 对比：template 模式图标会变，非 template 不变 | 验证系统**不会**自动对非 template 图标做色反；确认 `icon_as_template` 功能的必要性 |

> **⚠️ 平台坑：托盘菜单/图标点击全部无反应（2026-08-27 定论）**
>
> **症状**: 右键菜单能正常弹出、显示完全正常，但点击任何菜单项或图标都无反应；app 侧日志无任何报错（onNewWant 触发但参数为空）。
>
> **根因**: SCB（com.ohos.sceneboard）`AppClientNotifier.handleClientRegistration` 的 `clientProxyMap` 容量为 50，进程死后条目不自动清理。开发期反复 deploy/force-stop 会用僵尸 pid 把 50 个坑占满，新 app 的 receiver 代理注册被拒 → 点击降级为无载荷 startAbility。**属平台缺陷，app 侧无法自救。**
>
> **识别**: SCB 日志出现 `Register client pid fail: out of range`（hilog 默认 INFO 级即可看到；正常应为 `Register client pid success: <pid>`）。
>
> **恢复**: `hdc shell "kill <sceneboard_pid>"` 杀掉 SCB（约 30 秒后自动重生，clientProxyMap 清空）或重启设备，然后重启 app。正常 force-stop / install -r 不泄漏，日常开发不会复现。

---

## 二、Menu（菜单）手动用例

### 2.1 菜单栏（MenuBar）

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| core | menu | menubar/基础 | MenuBar Visible — 菜单栏可见性 | **T0** | 应用已启动 | 1. 点击 "MenuBar Visible" 按钮 | `is_menu_visible()` 返回 `true`；菜单栏在窗口顶部可见 | 应用启动时已自带默认菜单栏 |
| core | menu | menubar/基础 | MenuBar Dropdown — 菜单栏下拉菜单 | **T0** | 应用已启动 | 1. 点击 "MenuBar Dropdown" 按钮 2. 点击菜单栏 "Click Me" | 下拉菜单显示 "Item A" 和 "Item B" 两个选项 | 验证基本下拉功能 |
| core | menu | menubar/基础 | MenuBar Hide — 隐藏菜单栏 | **T0** | 应用已启动 | 1. 点击 "MenuBar Hide" 按钮 | 菜单栏从窗口顶部消失；`is_menu_visible()` 返回 `false` | 调用 `plugin:app-menu\|hide_menu` |
| core | menu | menubar/基础 | MenuBar Show — 显示菜单栏 | **T0** | 菜单栏已隐藏 | 1. 点击 "MenuBar Show" 按钮 | 菜单栏重新出现（恢复默认 File/Edit/Window/Help）；`is_menu_visible()` 返回 `true` | 调用 `plugin:app-menu\|show_menu`；会先恢复默认菜单 |
| core | menu | menubar/快捷键 | MenuBar Accelerator Ctrl+O — 自定义快捷键 | **T0** | 应用已启动 | 1. 点击 "MenuBar Accelerator Ctrl+O" 按钮 2. 按下 Ctrl+O（或点击 Accel → Accel Test） | action 回调触发，结果区显示 `Accelerator Ctrl+O FIRED! id=<id>` | 验证 `setAccelerator('Ctrl+O')` |
| core | menu | menubar/事件 | MenuBar Action Event — 菜单项点击事件 | **T0** | 应用已启动 | 1. 点击 "MenuBar Action Event" 按钮 2. 点击 EventTest → Click Me | 结果区显示 `action callback fired! id=menu-event-test`；Menu Event Log 输出 `[menu-event #N lid=1] global:menu-event-test at <时间>` | 验证 JS action 回调 + Rust 全局事件同时触发 |
| core | menu | menubar/预定义项 | Menu Edit→Copy — 预定义复制 | **T0** | 应用已启动；输入框有文本 | 1. 点击 "Menu Edit→Copy" 按钮 2. 选中输入框文本 3. 点击 Edit → Copy | 选中文本被复制到剪贴板 | 验证 PredefinedMenuItem Copy 功能 |
| core | menu | menubar/点击交互 | MenuBar Check Item — 勾选菜单项点击切换 | **T0** | 应用已启动 | 1. 点击 "MenuBar Auto Refresh Checked" 按钮 2. 展开 "Refresh" 下拉菜单 3. 点击 "Check Me" 项 | ① 初始状态未勾选，500ms 后自动变为勾选 ✓ ② 点击后勾选状态切换 ③ Menu Event Log 输出 `[menu-event #N lid=1] global:check_me at <时间>` | 验证 CheckMenuItem 点击行为 |
| core | menu | menubar/点击交互 | MenuBar Fullscreen — 预定义全屏窗口操作 | **T0** | 应用已启动 | 1. 点击 "MenuBar Fullscreen" 按钮 2. 展开 "View" 下拉菜单 3. 点击 Fullscreen 项 4. 按 Esc 退出全屏 | ① 窗口进入全屏，菜单栏隐藏 ② Menu Event Log 输出 `[menu-event #N lid=1] global:fullscreen at <时间>` ③ 按 Esc 退出全屏，菜单栏恢复显示 | 验证预定义项执行原生窗口操作 |
| core | menu | menubar/基础 | MenuBar Remove — 移除菜单栏 | **T1** | 应用已启动 | 1. 点击 "MenuBar Remove Menu" 按钮 | 菜单栏消失（设置空菜单） | `Menu.new({ items: [] })` + `setAsWindowMenu()` |
| core | menu | menubar/基础 | MenuBar is_menu_visible — 可见性查询 | **T1** | 菜单栏处于已知状态 | 1. 点击 "MenuBar is_menu_visible" 按钮 | 返回当前菜单可见性布尔值；默认 `true`，Hide 后 `false` | 验证 API 返回值与实际状态一致 |
| core | menu | menubar/嵌套 | MenuBar Nested Submenu — 嵌套子菜单 | **T1** | 应用已启动 | 1. 点击 "MenuBar Nested Submenu" 按钮 2. 点击 "Outer" → 悬停 "Inner" | 级联菜单：Outer 下拉显示 "Top Item" + "Inner"；悬停 Inner 展开显示 "Deep Item" | 验证多层嵌套菜单展开 |
| core | menu | menubar/交互 | MenuBar Hover — 菜单项悬停效果 | **T1** | 应用已启动 | 1. 点击 "MenuBar Hover" 按钮 2. 鼠标悬停到 "HoverTest" | 悬停时背景色高亮变化；鼠标移开后恢复正常 | 验证 UI 交互反馈 |
| core | menu | menubar/图标 | MenuBar Bar-Level Icon — 菜单栏级图标 | **T1** | 应用已启动 | 1. 点击 "MenuBar Bar-Level Icon" 按钮 | "IconMenu" 在菜单栏级别文字旁显示一个小图标 | MB_TEST_ICON 为 1×1 透明 PNG |
| core | menu | menubar/状态 | MenuBar Disabled Item — 禁用菜单项 | **T1** | 应用已启动 | 1. 点击 "MenuBar Disabled Item" 按钮 2. 点击 "DisTest" 下拉 | "Disabled" 项灰显/半透明且不可点击；"Normal" 项全色可点击 | 验证 `enabled: false` 的视觉表现 |
| core | menu | menubar/快捷键 | MenuBar Accelerator Ctrl+C — 预定义复制快捷键 | **T1** | 应用已启动；有可选择的文本 | 1. 点击 "MenuBar Accelerator Ctrl+C" 按钮 2. 在输入框输入文本并选中 3. 按 Ctrl+C | 选中文本被复制到剪贴板；粘贴可验证 | 使用 PredefinedMenuItem Copy |
| core | menu | menubar/自动刷新 | MenuBar Auto Refresh Text — 文本自动刷新 | **T1** | 应用已启动 | 1. 点击 "MenuBar Auto Refresh Text" 按钮 2. 展开 "Refresh" 下拉菜单 | 下拉菜单显示 "Updated!" 而非 "Original" | 先创建 text='Original'，500ms 后 setText('Updated!')；验证 auto_refresh 机制 |
| core | menu | menubar/自动刷新 | MenuBar Auto Refresh Checked — 勾选状态自动刷新 | **T1** | 应用已启动 | 1. 点击 "MenuBar Auto Refresh Checked" 按钮 2. **不点击**，等待 500ms 3. 展开 "Refresh" 下拉菜单 | "Check Me" 项前自动出现勾选标记 ✓（无需手动点击） | 验证 auto_refresh 机制在 500ms 后自动推送 checked 状态变更到原生菜单栏 |
| core | menu | menubar/预定义项 | MenuBar Predefined Hide — 预定义隐藏窗口 | **T1** | 应用已启动 | 1. 点击 "MenuBar Predefined Hide" 按钮 2. 点击 Window → Hide | 窗口最小化；从任务栏恢复后窗口重新出现 | PredefinedMenuItem 'Hide' |
| core | menu | menubar/事件 | MenuBar Popup Regression — popup 回归测试 | **T1** | 应用已启动 | 1. 点击 "MenuBar Popup Regression" 按钮 | 光标位置弹出上下文菜单，显示 "Popup Test" | 验证 AppStorage key 重命名后 `menu.popup()` 仍正常工作 |
| core | menu | menubar/NativeIcon | MenuBar NativeIcon Symbols — 原生图标映射 | **T1** | 应用已启动 | 1. 点击 "MenuBar NativeIcon Symbols" 按钮 2. 分别展开 "Mapped" 和 "Unmapped" 子菜单 | Mapped 组：Add→★、LockLocked→🔒、Network→📶 显示对应系统图标；Unmapped 组：Home/Folder/Share 等仅显示文字无图标 | **仅 OHOS 平台**有映射效果 |
| core | menu | menubar/预定义项 | Menu Edit→Paste — 预定义粘贴 | **T1** | 应用已启动；剪贴板有内容 | 1. 点击 "Menu Edit→Paste" 按钮 2. 在外部复制文本 3. 聚焦输入框 4. 点击 Edit → Paste | 剪贴板内容被粘贴到输入框中 | OHOS 剪贴板读权限限制，当前无法验证 |
| core | menu | menubar/预定义项 | Menu Edit→Cut — 预定义剪切 | **T1** | 应用已启动；输入框有选中文本 | 1. 点击 "Menu Edit→Cut" 按钮 2. 选中输入框文本 3. 点击 Edit → Cut | 选中文本从输入框消失，同时被复制到剪贴板 | 验证 PredefinedMenuItem Cut 功能 |

### 2.2 弹出菜单（PopupMenu）

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| core | menu | popupmenu/基础 | Menu Page — Popup 弹出菜单 | **T0** | 应用已启动，进入 Menu 页面 | 1. 在 MenuBuilder 中配置菜单项 2. 点击 "Popup" 按钮 | 光标位置弹出上下文菜单，显示配置的所有菜单项 | `menu.popup()` 弹出 |
| core | menu | popupmenu/点击交互 | Popup Click Item — 弹出菜单点击菜单项 | **T0** | 应用已启动，进入 Menu 页面 | 1. 在 MenuBuilder 中添加一个 Normal 项（如 "Test Item"） 2. 点击 "Popup" 按钮 3. 在弹出菜单中点击 "Test Item" | ① 光标位置弹出上下文菜单 ② 点击后菜单消失 ③ UI 输出 `Item Test Item clicked` | 验证 MenuItem action 回调 |
| core | menu | popupmenu/点击交互 | Popup Predefined Copy — 弹出菜单预定义复制 | **T0** | 应用已启动，进入 Menu 页面；输入框有文本 | 1. 在 MenuBuilder 中添加一个 Predefined Copy 项 2. 选中输入框文本 3. 点击 "Popup" 按钮 4. 在弹出菜单中点击 Copy | 选中文本被复制到剪贴板；UI 输出 `Item Copy clicked` | 验证弹出菜单中预定义项的原生操作 |
| core | menu | popupmenu/图标 | Menu Page — Create menu with NativeIcon | **T1** | 应用已启动，进入 Menu 页面 | 1. 点击 "Create menu with NativeIcon" 按钮 | 菜单栏显示带 NativeIcon.Folder 图标的子菜单 | 验证 Submenu 级别的 NativeIcon |
| core | menu | popupmenu/图标 | Menu Page — Create menu with Image icon | **T1** | 应用已启动，进入 Menu 页面 | 1. 点击 "Create menu with Image icon" 按钮 | 菜单栏显示带 defaultWindowIcon 图标的子菜单 | 使用 `defaultWindowIcon()` 获取应用窗口图标 |
| core | menu | popupmenu/基础 | Menu Page — Create menu 创建应用菜单 | **T1** | 应用已启动，进入 Menu 页面；MenuBuilder 已配置菜单项 | 1. 在 MenuBuilder 中选择菜单项类型并创建 2. 点击 "Create menu" 按钮 | 窗口菜单栏出现 "app" 子菜单，包含所有配置的菜单项 | macOS 设为 AppMenu，其他平台设为 WindowMenu |

---

## 三、Clipboard（剪贴板）手动用例

### 3.1 writeImage 全参数类型

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| core | clipboard | writeImage/rgba | writeImage(rgba) — { rgba, width, height } 对象 | **T0** | 应用已启动，进入 Tests 页面 | 1. 点击 "writeImage(rgba)" 按钮 2. 切换到备忘录或其他应用 3. 粘贴 | ① Console 输出 `writeImage({ rgba: … }) OK` ② 粘贴后出现 1×1 红色图像 | 验证 visit_map → JsImage::Rgba 路径 |
| core | clipboard | writeImage/data-uri | writeImage(data-uri) — data URI 字符串 | **T0** | 应用已启动，进入 Tests 页面 | 1. 点击 "writeImage(data-uri)" 按钮 2. 切换到其他应用 3. 粘贴 | ① Console 输出 `writeImage(dataUri) OK` ② 粘贴后出现图像 | 验证 visit_str → JsImage::DataUri 路径 |
| core | clipboard | writeImage/rid | writeImage(Image rid) — Image 资源对象 | **T0** | 应用已启动，进入 Tests 页面 | 1. 点击 "writeImage(Image rid)" 按钮 2. 切换到其他应用 3. 粘贴 | ① Console 输出 `writeImage(Image rid=N) OK` ② 粘贴后出现 1×1 红色图像 | 验证 duck-type rid → JsImage::Resource 路径 |
| core | clipboard | writeImage/bytes | writeImage(Uint8Array) — PNG 字节数组 | **T0** | 应用已启动，进入 Tests 页面 | 1. 点击 "writeImage(Uint8Array)" 按钮 2. 切换到其他应用 3. 粘贴 | ① Console 输出 `writeImage(Uint8Array) OK` ② 粘贴后出现 1×1 红色图像 | 验证 visit_seq → JsImage::Bytes 路径 |
| core | clipboard | writeImage/path | writeImage(filePath) — 文件路径字符串 | **T1** | 应用已启动，进入 Tests 页面 | 1. 点击 "writeImage(filePath)" 按钮 2. 切换到其他应用 3. 粘贴 | ① Console 输出 `writeImage(filePath) OK` + 路径信息 ② 粘贴后出现 1×1 红色图像 | 验证 visit_str → JsImage::Path 路径；使用 fs plugin + path API 写临时文件 |
| core | clipboard | writeImage/number-array | writeImage(number[]) — 数字数组 | **T1** | 应用已启动，进入 Tests 页面 | 1. 点击 "writeImage(number[])" 按钮 2. 切换到其他应用 3. 粘贴 | ① Console 输出 `writeImage(number[]) OK` ② 粘贴后出现 1×1 红色图像 | 验证 visit_seq → JsImage::Bytes 路径（OHOS IPC：Array → sequence） |
| core | clipboard | writeImage/arraybuffer | writeImage(ArrayBuffer) — ArrayBuffer | **T1** | 应用已启动，进入 Tests 页面 | 1. 点击 "writeImage(ArrayBuffer)" 按钮 2. 切换到其他应用 3. 粘贴 | ① Console 输出 `writeImage(ArrayBuffer) OK` ② 粘贴后出现 1×1 红色图像 | 验证 visit_seq → JsImage::Bytes 路径（OHOS IPC：buffer → sequence） |

---

## 四、Dialog（对话框）手动用例

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| plugin | dialog | open/基础 | Dialog.open (single) — 单文件选择 | **T0** | 应用已启动，进入 Tests 页面 | 1. 点击 "Dialog.open (single)" 按钮 2. 在弹出的文件选择器中选择一个文件 3. 点击确认 | ① 弹出系统文件选择器 ② 选择文件后 UI 显示所选文件路径（字符串） ③ 路径非空 | `open({ multiple: false })` |
| plugin | dialog | open/多选 | Dialog.open (multiple) — 多文件选择 | **T0** | 应用已启动，进入 Tests 页面 | 1. 点击 "Dialog.open (multiple)" 按钮 2. 在选择器中选择多个文件 3. 点击确认 | ① 弹出系统文件选择器 ② UI 显示所有选中文件的路径列表（字符串数组） ③ 数组长度 ≥ 2 | `open({ multiple: true })` |
| plugin | dialog | save/基础 | Dialog.save — 保存文件对话框 | **T0** | 应用已启动，进入 Tests 页面 | 1. 点击 "Dialog.save" 按钮 2. 在保存对话框中输入文件名 3. 点击保存 | ① 弹出系统保存文件对话框 ② 默认文件名为 `test.txt` ③ 确认后 UI 显示所选保存路径 | `save({ defaultPath: 'test.txt' })` |
| plugin | dialog | confirm/基础 | Dialog.confirm — Ok/Cancel 确认 | **T0** | 应用已启动，进入 Tests 页面 | 1. 点击 "Dialog.confirm" 按钮 2. 分别点击 OK 和 Cancel | ① 弹出对话框，标题 "Confirm Action"，warning 图标 ② 包含 OK/Cancel 两个按钮 ③ 点击 OK → 返回 `true` ④ 点击 Cancel → 返回 `false` | `confirm('...', { title: 'Confirm Action', kind: 'warning' })` |
| plugin | dialog | message/info | Dialog.message (info) — 信息对话框 | **T0** | 应用已启动，进入 Tests 页面 | 1. 点击 "Dialog.message (info)" 按钮 2. 查看弹出的对话框 3. 点击 OK | ① 弹出消息对话框 ② 标题为 "Info Dialog" ③ 显示 info 类型图标 ④ 包含 "OK" 按钮 ⑤ 点击 OK 后对话框关闭 | `message('...', { title: 'Info Dialog', kind: 'info' })` |
| plugin | dialog | message/warning | Dialog.message (warning) — 警告对话框 | **T0** | 应用已启动，进入 Tests 页面 | 1. 点击 "Dialog.message (warning)" 按钮 2. 查看弹出的对话框 3. 点击 OK | ① 弹出消息对话框 ② 标题为 "Warning Dialog" ③ 显示 warning 类型图标 ④ 包含 "OK" 按钮 ⑤ 点击 OK 后对话框关闭 | `message('...', { title: 'Warning Dialog', kind: 'warning' })` |
| plugin | dialog | message/error | Dialog.message (error) — 错误对话框 | **T0** | 应用已启动，进入 Tests 页面 | 1. 点击 "Dialog.message (error)" 按钮 2. 查看弹出的对话框 3. 点击 OK | ① 弹出消息对话框 ② 标题为 "Error Dialog" ③ 显示 error 类型图标 ④ 包含 "OK" 按钮 ⑤ 点击 OK 后对话框关闭 | `message('...', { title: 'Error Dialog', kind: 'error' })` |

---

## 五、plugin-os（平台检测）手动用例

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| plugin | os | platform/基础 | platform() — 平台标识返回值 | **T0** | 应用已启动，进入 OS/Platform 页面或控制台 | 1. 点击`OS info(platform/type/version)`按钮 2. 调用 `platform()` API | ① `platform()` 返回 `"ohos"`（非 `"linux"`） ② 前端 TypeScript 类型包含 `'ohos'` | 编译期通过 `cfg(target_env = "ohos")` 覆盖 `std::env::consts::OS` |
| plugin | os | type/基础 | type() — OS 类型返回值 | **T0** | 应用已启动 | 1. 调用 `type()` API 2. 观察返回值 | ① `type()` 返回 `"ohos"`（非 `"linux"`） ② 前端 TypeScript `OsType` 类型包含 `'ohos'` | `OsType::Ohos` 在 `cfg(target_env = "ohos")` 下优先于 Linux 分支 |
| plugin | os | version/基础 | version() — 版本号返回值 | **T1** | 应用已启动 | 1. 调用 `version()` API 2. 观察返回值 | ① `version()` 返回 `"0.0.0"` ② 不崩溃、不报错 | OHOS 上 `os_info` 不支持，使用 `Version::Semantic(0,0,0)` 占位 |
| plugin | os | family/基础 | family() — 系统家族返回值 | **T1** | 应用已启动 | 1. 调用 `family()` API | `family()` 返回 `"unix"` | OHOS 属于 unix 家族，无需覆盖 |
| plugin | os | arch/基础 | arch() — 架构返回值 | **T1** | 应用已启动 | 1. 调用 `arch()` API | `arch()` 返回 `"aarch64"` | OHOS 目标为 aarch64，无需覆盖 |
| plugin | os | eol/基础 | eol() — 行尾标记返回值 | **T1** | 应用已启动 | 1. 调用 `eol()` API | `eol()` 返回 `"\n"` | OHOS 非 Windows，使用 POSIX 行尾 |

---

## 六、Autostart（开机自启动）手动用例

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| core | autostart | enable/跳转设置 | enable() — 跳转到应用启动管理页面 | **T0** | 应用已启动，进入 Tests 页面，滚动到 Manual Tests 区域 | 1. 找到 "Autostart Manual Tests" 分组 2. 点击 "enable() (opens settings)" 按钮 3. 观察应用是否跳转到系统设置页面 4. 在系统设置页面中找到当前应用（com.tauri.api） 5. 确认该应用旁有"自启动"开关 | ① 点击按钮后，应用无报错，Console 区输出 `enable() called. On OHOS: System "App launch management" settings page should open now.` ② 系统自动跳转到"应用启动管理"设置页面（bundleName: `com.huawei.hmos.settings`，URI: `pc_app_setup_settings`） ③ 设置页面中可见当前应用名称及其自启动开关 ④ 应用未崩溃、未卡死 | OHOS 平台限制：普通应用无法程序化开启自启动，只能引导用户到设置页面手动操作 |
| core | autostart | isEnabled/状态查询 | isEnabled() — 查询自启动状态 | **T0** | 应用已启动，进入 Tests 页面；设备 API ≥ 21 | 1. 找到 "Autostart Manual Tests" 分组 2. 点击 "isEnabled()" 按钮 3. 观察 Console 输出结果（应为 `false` 或 `true`，取决于当前设置） 4. 切换到系统设置 → 应用启动管理 → 找到当前应用 5. 手动开启自启动开关 6. 返回应用，再次点击 "isEnabled()" 按钮 7. 再次切换到系统设置，手动关闭自启动开关 8. 返回应用，第三次点击 "isEnabled()" 按钮 | ① 步骤 2 后 Console 输出 `isEnabled() → <布尔值>`，并提示 `Verify: Go to Settings → App launch management` ② 步骤 6 后 Console 输出 `isEnabled() → true`（与步骤 5 手动开启一致） ③ 步骤 8 后 Console 输出 `isEnabled() → false`（与步骤 7 手动关闭一致） ④ 每次调用无报错、无超时（5s 内返回） | 需要 API 21+ 支持 `autoStartupManager.getAutoStartupStatusForSelf()`；API < 21 设备始终返回 `false` |
| core | autostart | disable/跳转设置 | disable() — 跳转到应用启动管理页面 | **T1** | 应用已启动，进入 Tests 页面 | 1. 找到 "Autostart Manual Tests" 分组 2. 点击 "disable() (opens settings)" 按钮 3. 观察应用是否跳转到系统设置页面 4. 确认设置页面中可见当前应用及其自启动开关 | ① 点击按钮后，应用无报错，Console 区输出 `disable() called. On OHOS: System "App launch management" settings page should open now.` ② 系统自动跳转到"应用启动管理"设置页面（与 enable() 相同的目标页面） ③ 设置页面中可见当前应用名称及其自启动开关（可手动关闭） | OHOS 平台限制：disable() 与 enable() 行为一致，都是跳转到设置页面，由用户手动操作 |
| core | autostart | 完整流程 | enable → 手动开启 → isEnabled → disable → 手动关闭 → isEnabled | **T1** | 应用已启动，进入 Tests 页面；设备 API ≥ 21 | 1. 点击 "enable() (opens settings)" 按钮 2. 系统跳转到设置页面，手动开启当前应用的自启动开关 3. 返回应用，点击 "isEnabled()" 按钮 4. 点击 "disable() (opens settings)" 按钮 5. 系统跳转到设置页面，手动关闭当前应用的自启动开关 6. 返回应用，点击 "isEnabled()" 按钮 | ① 步骤 3 Console 输出 `isEnabled() → true`（与步骤 2 手动开启一致） ② 步骤 6 Console 输出 `isEnabled() → false`（与步骤 5 手动关闭一致） ③ 整个 enable→check→disable→check 流程无报错、无崩溃 ④ 每次操作均在 5s 内完成 | 验证完整的用户操作流程：引导设置 → 手动操作 → 状态查询一致性 |

| 模块 | T0 | T1 | 合计 |
|------|-----|-----|------|
| Autostart | 2 | 2 | **4** |

---

## 七、Webview（WebView）手动用例

### 7.1 createPdf（PDF 生成）

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| core | webview | createPdf/默认 | Create PDF A4 — 默认 A4 尺寸生成 PDF | **T0** | 应用已启动；WebView 已加载页面 | 1. 滚动到 "Create PDF Manual Test" 区域 2. 点击 "Create PDF A4 (default)" 按钮 | ① 页面显示 `SUCCESS ✅` ② 设备 `/data/storage/el2/base/cache/test.pdf` 文件生成 ③ `hdc file recv` 拉取后可打开查看，内容为 A4 尺寸 | 默认配置: 8.27×11.69in, 无边距, 含背景 |
| core | webview | createPdf/自定义 | Create PDF Square — 正方形自定义尺寸 | **T1** | 应用已启动；WebView 已加载页面 | 1. 滚动到 "Create PDF Manual Test" 区域 2. 点击 "Create PDF Square (8.27×8.27)" 按钮 | ① 页面显示 `SUCCESS ✅` ② 设备 `/data/storage/el2/base/cache/test-square.pdf` 文件生成 ③ 拉取后打开，页面为正方形尺寸 | 验证 PdfConfig 透传: width=8.27, height=8.27 |

| 模块 | T0 | T1 | 合计 |
|------|-----|-----|------|
| Webview — createPdf | 1 | 1 | **2** |

### 7.2 Cookie（Cookie 管理真实生效）

> **背景**: 自动用例（cookie_test）只验证 WebCookieManager API 契约（configCookieSync 写入 → fetchCookieSync 读回）。本手动用例补全"set_cookie 写入的 cookie 真实随请求发送到服务端"的真实浏览行为。
>
> **日志监控命令**: `hdc shell hilog | grep tauritest`

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| core | webview | cookie/真实生效 | Cookie Live (httpbin echo) — set_cookie 后服务端收到 | **T0** | 应用已启动；设备可访问 `https://httpbin.org` | 1. 滚动到 "webview.cookie Manual Tests" 区域 2. 点击 "Cookie Live (httpbin echo)" 按钮 3. 观察弹出的子窗口中 `https://httpbin.org/cookies` 的 JSON 响应 | ① 子窗口成功打开并加载 `https://httpbin.org/cookies` ② JSON 响应包含 `"tauri_test_cookie": "ManualTest123"`（证明 `set_cookie` 写入的 cookie 真实发送到服务端） | 验证 `set_cookie`（`WebCookieManager.configCookieSync`）端到端真实生效；cookie 域 `httpbin.org`、Path `/`、值 `ManualTest123` 由 `cookie_manual_test` 命令预设。注：子窗口为外部页（无 Tauri 工具栏），仅验证首次加载的 cookie 回显，不做刷新/持久化验证 |

| 模块 | T0 | T1 | 合计 |
|------|-----|-----|------|
| Webview — Cookie | 1 | 0 | **1** |

### 7.3 DevTools（调试访问开关）

> **背景**: wry OHOS 的 `open_devtools`/`close_devtools` 映射为 `WebviewController.setWebDebuggingAccess` 全局开关，`is_devtools_open` 返回 ArkTS 侧自跟踪状态（OHOS 无 getter）。三方法受 `#[cfg(any(debug_assertions, feature="devtools"))]` 门控，**仅在 devtools feature 构建可测**（标准 release 不编译）。本用例在 devtools 构建下验证 open→true、close→false 的 toggle 行为。

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| core | webview | devtools/toggle | DevTools (open/is_open/close) — 调试访问开关 toggle | **T1** | ① 用 devtools feature 构建并部署：临时把 `examples/api/src-tauri/Cargo.toml` 的 `prod` 改为 `["tauri/custom-protocol", "devtools"]`（或 build-ohos.sh 加 `--features prod,devtools`），跑 `run-tests.sh` 部署；验证后回退该改动 ② 设备屏幕已唤醒（`hdc shell "power-shell setmode 602"`）| 1. 打开 app，进入 Tests 页 2. 滚动到 "webview.devtools Manual Test (OHOS only, needs devtools build)" 区域 3. 点击 "DevTools (open/is_open/close)" 按钮 | 屏幕显示如下即成功：`devtools_test: PASS ✅` 换行 `initial=<true|false>, after_open=true, after_close=false`。关键判定：`after_open=true`（open_devtools 后调试访问开）且 `after_close=false`（close_devtools 后关）。若显示 `FAIL ❌` 或 `devtools feature not enabled` 则失败 | `initial` **有状态、非判定项**：`webDebuggingEnabled` 是进程级全局变量，跨调用持久——首次调用（app 刚启动无 open/close 历史）反映 init 标志（tauri 默认 devtools=true → 通常 true）；若之前已跑过 close_devtools（如自动用例 test 53 先跑）则 initial=false。判定只看 after_open/after_close；标准 release 构建（未加 devtools feature）点击提示 "devtools feature not enabled"，属预期（dormant）|

| 模块 | T0 | T1 | 合计 |
|------|-----|-----|------|
| Webview — DevTools | 0 | 1 | **1** |

### 7.4 全屏无黑边（set_bounds resize 传播回归防护）

> **背景**: 修复了主 webview `set_bounds` 全屏黑边问题。根因是 tao 不传播 `ContentRectChange` 为 `Resized` 事件 + `WindowIdStore` 的 ZST key 被子窗口覆盖。修复后 set_bounds 在每次窗口 resize 时被正确调用，Web 组件按新尺寸重渲染。本用例防护此回归。

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| core | webview | fullscreen/no-black-bars | Fullscreen No Black Bars — 全屏无黑边 | **T0** | 应用已启动 | 1. 将应用窗口最大化或全屏 2. 观察屏幕四个方向是否有黑边 3. 恢复窗口化 4. 再次观察 | ① 全屏时 Web 内容填满整个窗口，四方向无黑边 ② 窗口化时 Web 内容填满窗口，无黑边 ③ 若出现黑边说明 tao ContentRectChange 传播 / WindowIdStore or_insert / wry set_bounds 链断裂 | 防护三修复链：tao 传播 ContentRectChange→Resized + tauri-runtime-wry or_insert + wry set_bounds 移除 cache-only |

| 模块 | T0 | T1 | 合计 |
|------|-----|-----|------|
| Webview — Fullscreen | 1 | 0 | **1** |

---

## 八、WebView User-Agent 自定义 手动用例

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| core | webview | userAgent/custom | 自定义 User-Agent 设置验证 | **T0** | 应用已启动，进入 Tests 页面 | 1. 滚动到 "WebView User-Agent" 测试分组 2. 点击 "userAgent (custom)" 按钮 3. 观察新打开的窗口中 useragent-test.html 页面显示结果 | ① 新窗口成功打开并加载 `useragent-test.html` 页面 ② 页面显示绿色 "✓ PASS: Custom UA detected" ③ `navigator.userAgent` 包含 `MyApp/1.0 Tauri/2.0` | OHOS 平台通过 `WebviewController.setCustomUserAgent()` 实现；在 `onControllerAttached` 回调中设置；Rust 侧通过 `eval_with_callback` 将 UA 输出到 hilog |
| core | webview | userAgent/default | 默认 User-Agent 验证 | **T1** | 应用已启动，进入 Tests 页面 | 1. 滚动到 "WebView User-Agent" 测试分组 2. 点击 "userAgent (default)" 按钮 3. 观察新打开的窗口中 useragent-test.html 页面显示结果 | ① 新窗口成功打开并加载 `useragent-test.html` 页面 ② 页面显示蓝色 "ℹ System default UA (no custom UA set)" ③ `navigator.userAgent` 为系统默认值（如 `Mozilla/5.0 (Phone; OpenHarmony 5.0) AppleWebKit/537.36 ...`） | 未提供自定义 User-Agent 时，WebView 使用系统默认值 |
| core | webview | userAgent/多窗口隔离 | 多窗口 User-Agent 隔离验证 | **T1** | 应用已启动，进入 Tests 页面 | 1. 点击 "userAgent (multi-window)" 按钮 2. 观察两个新打开的窗口中 useragent-test.html 页面分别显示的结果 3. 可通过 `hdc shell "hilog \| grep UA-TEST"` 查看 Rust 侧日志 | ① 两个新窗口成功打开 ② 窗口 A 页面显示 "Multi-window UA detected" ③ 窗口 B 页面显示 "Multi-window UA detected" ④ hilog 中两个窗口的 `navigator.userAgent` 值分别包含各自的自定义标识 | 验证 OHOS 平台上多个 WebView 实例的 User-Agent 设置互不干扰 |

| 模块 | T0 | T1 | 合计 |
|------|-----|-----|------|
| WebView User-Agent | 1 | 2 | **3** |

---

## 九、RunEvent（生命周期事件）手动用例

> **背景**: 修复了 `ExitRequested`/`Exit` 在 `LoopDestroyed` 路径上的触发；修复了子窗口 `Destroyed` 事件缺失和 `WindowsStore` 清理问题。
>
> **日志监控命令**: `hdc shell hilog | grep tauritest`

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| core | runevent | ExitRequested/LoopDestroyed | 系统关闭应用 — ExitRequested + prevent_exit | **T0** | 应用已启动，打开 DevEco Studio 观察日志，关键词runevent | 1. 关闭应用 2. 观察日志输出 | ① 日志依次出现 `LoopDestroyed received` → `ExitRequested, code=None` → `prevent_exit() called` → `Exit` ② 应用仍然退出（`LoopDestroyed` 时系统已开始销毁，`prevent_exit()` 无法阻止） | 验证：LoopDestroyed handler 先触发 ExitRequested 再触发 Exit；OHOS 平台限制：prevent_exit 仅通知清理，无法阻止退出 |
| core | runevent | ExitRequested/防重复 | ExitRequested 防重复触发 | **T1** | 应用已启动，打开 DevEco Studio 观察日志，关键词runevent；已创建多个子窗口 | 1. 逐个关闭子窗口（每个观察日志） 2. 关闭最后一个窗口（主窗口） 3. 统计 `ExitRequested` 出现次数 | ① 每个子窗口关闭时：`CloseRequested` → `Destroyed` ② 最后一个窗口关闭时：`ExitRequested` **仅一次** ③ 随后 LoopDestroyed 时**不再重复** ExitRequested，直接发送 `Exit` | 验证 `ExitState(AtomicBool)` 防重复机制 |
| core | runevent | Resumed/跨平台遗留 | Resumed 事件 — 不触发（预期行为） | **T1** | 自动测试报告已生成 | 1. 查看 Test #29 `RunEvent::Resumed fires on startup` 结果 | ① 状态为 ❌ ② 预期失败，跨平台遗留问题 | 不在本次修复范围内 |
| core | runevent | Opened/深度链接 | Opened 事件 — 深度链接触发 | **T1** | 应用已启动，打开 DevEco Studio 观察日志 | 1. 执行 `hdc shell aa start -a EntryAbility -b com.tauri.api -U myapp://test/path` 2. 观察日志输出和 UI 响应 | ① 日志出现 `[RunEvent] Opened, urls=["myapp://test/path"]` ② UI 显示深度链接信息（如有处理逻辑） | 验证：OHOS 平台 Opened 事件已启用（代码 511-515 行），通过深度链接触发 |

| 模块 | T0 | T1 | 合计 |
|------|-----|-----|------|
| RunEvent（生命周期事件） | 1 | 3 | **4** |

---

## 十、Transparent（透明窗口）手动用例

> **背景**: OHOS 平台 Web 引擎渲染表面不支持透明穿透，主窗口设置 `transparent: true` 后 Web 内容区仍不透明（详见 `doc/ohos-main-window-transparent-analysis.md`）。仅 Float 子窗口（`transparent: true` + `decorations: false`）可实现完整穿透效果。
>
> **测试入口**: `examples/api` 应用 → TransparencyTest 页面

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| core | transparent | float-window/透明+无边框 | 创建透明无边框 Float 子窗口 | **T0** | 应用已启动，进入 TransparencyTest 页面 | 1. 点击 "创建透明无边框窗口" 按钮 2. 观察新弹出的子窗口外观 3. 验证窗口是否可穿透看到桌面内容 4. 点击窗口内关闭链接关闭子窗口 | ① 新窗口弹出，无标题栏（`decorations: false`） ② 窗口背景透明，可穿透看到桌面内容（`transparent: true`） ③ 窗口内显示半透明黑色卡片，标题 "✨ Transparent + Borderless"，文字 `decorations: false + transparent: true` ④ 窗口底部状态栏显示 `isDecorated: false` ⑤ 点击关闭链接后窗口正常关闭 | Float 子窗口类型；`WindowMode.Float` + `transparent(true)` + `decorations(false)`；内部分配 `window_id = transparency_test_<timestamp>` |
| core | transparent | float-window/仅透明有边框 | 创建透明有边框 Float 子窗口 | **T1** | 应用已启动，进入 TransparencyTest 页面 | 1. 点击 "创建透明有边框窗口" 按钮 2. 观察新弹出的子窗口外观 3. 确认窗口有标题栏、背景透明效果可见 4. 点击窗口内关闭链接关闭子窗口 | ① 子窗口有标题栏（`decorations: true` 默认） ② 窗口内容区背景透明，可穿透看到桌面 ③ 窗口内显示 "🪟 Transparent Window" 卡片，底部状态栏显示 `isDecorated: true` ④ 点击关闭链接后窗口正常关闭 | 验证 `transparent: true` 单独使用（不加 `decorations: false`）时 OHOS 表现；标题栏由系统渲染不受 transparent 影响 |

| 模块 | T0 | T1 | 合计 |
|------|-----|-----|------|
| Transparent（透明窗口） | 1 | 1 | **2** |

---

## 十一、on_new_window（新窗口拦截）手动用例

> **背景**: OHOS 平台通过 ArkWeb `onWindowNew` 事件拦截 `window.open()` / `target="_blank"` 等新窗口请求，Rust 侧 `on_new_window` handler 可返回 Allow（弹出 dialog）或 Deny（阻止）。
>
> **测试入口**: `examples/api` 应用 → Tests 页面 → Manual Tests 区域

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| core | on_new_window | Allow/弹窗关闭 | Allow dialog 关闭按钮验证 | **T0** | 应用已启动，进入 Tests 页面 | 1. 点击 "on_new_window: Allow dialog has close button (manual)" 2. 观察弹窗外观 3. 点击标题栏 ✕ 按钮 | ① 弹出非模态对话框，标题栏显示 URL ② 标题栏右上角有 ✕ 关闭按钮 ③ 点击 ✕ 对话框关闭 ④ 点击对话框内嵌 Web 组件加载对应 URL | `promptAction.openCustomDialog` + `setTimeout` 延迟打开避免阻塞事件循环 |
| core | on_new_window | Deny/无弹窗 | Deny 模式阻止弹窗验证 | **T1** | 应用已启动，进入 Tests 页面 | 1. 点击 "on_new_window: Deny prevents dialog (manual)" 2. 观察屏幕 | ① 不弹出任何对话框 ② 页面保持不变，无导航跳转 ③ hilog 可见 `DENY` 日志 | `setWebController(null)` 阻止新窗口 |
| core | on_new_window | Create/真窗口 | Create real OS window 验证 | **T0** | 应用已启动，进入 Tests 页面 | 1. 点击 "Create (real OS window)" 2. 观察 3. 验证窗口行为 | ① 弹出独立 OS 子窗口（非页内对话框）② 窗口加载目标 URL ③ 关闭子窗口不影响主应用 ④ 再次点击不弹对话框（setWebController(null)） | `NewWindowResponse::Create` → `WebviewWindowBuilder::build()` → `createOSWindow` → Float 子窗口 |

| 模块 | T0 | T1 | 合计 |
|------|-----|-----|------|
| on_new_window（新窗口拦截） | 2 | 1 | **3** |

---

## 十二、Notification（通知）手动用例

> **测试入口**: `examples/api` 应用 → Tests 页面 → **Notification Manual Tests** 区域
>
> **自动测试已覆盖**: `isPermissionGranted`、`createChannel+channels`、`removeChannel`、`cancel+cancelAll`、`pending+active`、`sendNotification`、`sendWithChannel` 共 7 个自动测试已在 `plugins.ts` 中，每次构建自动运行。
>
> **以下 3 个用例需要人眼确认通知中心的视觉显示**，已集成为 Tests 页面的按钮：

| 一级场景 | 二级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|------|
| plugin | notification | Send Notification — 通知中心视觉确认 | **T0** | 应用已启动，通知权限已授予 | 1. 进入 Tests 页面，滚动到 "Notification Manual Tests" 区域 2. 点击 **"Send Notification"** 按钮 3. 点击屏幕右上角系统通知图标，打开通知中心 | ① 结果区域显示 `sendNotification() 调用成功` ② 系统通知中心（屏幕右下角系统托盘🔔图标）出现通知，标题 "Tauri 手动测试"，内容 "如果你在通知中心看到这条消息，测试通过！" ③ 点击通知后通知消失 | 自动测试只验证 API 不报错，通知是否真正显示必须人眼确认 |
| plugin | notification | Send With Channel — 渠道通知视觉确认 | **T1** | 应用已启动，通知权限已授予 | 1. 点击 **"Send With Channel"** 按钮 2. 打开系统通知中心 | ① 结果区域显示 `createChannel() + sendNotification(channelId) 调用成功` ② 屏幕右下角出现通知，标题 "渠道通知测试"| 按钮自动创建渠道 `manual-test-ch` 并通过该渠道发送 |
| plugin | notification | Request Permission — 系统弹窗确认 | **T1** | **需卸载重装应用**（权限弹窗仅首次弹出） | 1. 卸载应用：`hdc shell bm uninstall -n com.tauri.api`（首次执行不用） 2. 重新构建安装 3. 点击 **"Request Permission"** 按钮 | ① 系统弹出通知权限授权对话框 ② 点击"允许"后结果区域显示 `requestPermission() → "granted"` ③ 再次点击不再弹窗 | 此测试需要干净环境，日常回归可跳过 |

| 模块 | T0 | T1 | 合计 |
|------|-----|-----|------|
| Notification manual（手动测试） | 1 | 2 | **3** |

---

## 十三、Single-Instance（单实例）手动用例

> **前置条件**: example app 已集成 `tauri-plugin-single-instance`，callback 中通过 `log::info!("[single-instance] callback fired! args={:?}, cwd={:?}", args, cwd)` 输出日志。
>
> **验证方法**: 在宿主机执行 `hdc shell` 命令触发二次启动，通过 `hilog` 观察 callback 是否触发。

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| core | single-instance | 首次启动 | App Normal Launch — 首次启动不触发 callback | **T0** | 设备已连接；app 未运行 | 1. `hdc shell hilog -r`（清空日志） 2. 启动 app（点击图标或 `hdc shell aa start -a EntryAbility -b com.tauri.api`） 3. `hdc shell "hilog -x \| grep single-instance"` | hilog 中**无** `[single-instance] callback fired` 日志输出；app 正常启动显示主界面 | 首次启动走 `onCreate` 路径，不触发 `onNewWant` |
| core | single-instance | 二次启动 | Second Launch Callback — 再次启动触发 callback | **T0** | app 已在运行 | 1. `hdc shell hilog -r`（清空日志） 2. `hdc shell "aa start -a EntryAbility -b com.tauri.api -U 'tauri://test'"` 3. `hdc shell "hilog -x \| grep single-instance"` | ① hilog 输出 `[single-instance] callback fired! args=["tauri://test", "{...}"], cwd=""` ② app 回到前台 ③ 不会创建新的 app 实例 | OHOS 默认 `launchType: singleton`，OS 层面阻止新实例 |
| core | single-instance | 参数传递 | Want Parameters — 二次启动携带 URI | **T0** | app 已在运行 | 1. `hdc shell hilog -r` 2. `hdc shell "aa start -a EntryAbility -b com.tauri.api -U 'myapp://action?key=value'"` 3. `hdc shell "hilog -x \| grep single-instance"` | ① args 第一个元素为 `"myapp://action?key=value"`（want.uri） ② args 第二个元素为 JSON 字符串，包含系统注入的 want.parameters（具体字段因 API 版本和设备而异，验证重点为非空 JSON 字符串） ③ cwd 为空字符串 `""` | `aa start -U` 仅设置 want.uri，want.parameters 由系统自动注入 |
| core | single-instance | 无 URI 启动 | Second Launch Without URI — 无 URI 二次启动 | **T1** | app 已在运行 | 1. `hdc shell hilog -r` 2. `hdc shell "aa start -a EntryAbility -b com.tauri.api"` 3. `hdc shell "hilog -x \| grep NativeAbility"` 4. `hdc shell "hilog -x \| grep single-instance"` | ① `hilog \| grep NativeAbility` 有 `onNewWant - uri: , parametersJson.length: <N>` 日志（URI 为空，length > 0） ② `hilog \| grep single-instance` 有 `[single-instance] callback fired!` 日志，args 仅包含系统注入的 want.parameters JSON（空 URI 被过滤） | 与 macOS/Windows 行为对齐：第二次启动无论有无参数，callback 均触发 |

| 模块 | T0 | T1 | 合计 |
|------|-----|-----|------|
| Single-Instance（单实例） | 3 | 1 | **4** |

---

## 十四、Predefined Multi-Window（预定义操作多窗口支持）手动用例

> **背景**: 修复 predefined menu 操作在多窗口场景下的目标窗口解析：hide/close/minimize 语义修正、showAll/bringAllToFront 恢复应用、剪贴板/编辑操作使用目标窗口 webview controller、onTouch 迁移到页面根容器。
>
> **测试入口**: `examples/api` 应用，需创建 Full Test Tray 后操作。涉及左键点击托盘图标的用例需先清空 QuickOperation abilityName。

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| core | predefined-multi-window | clipboard/copy | Tray Copy 子窗口 — 复制子窗口选中文本 | **T0** | 应用已启动；已创建子窗口（如 Hello World）；子窗口有可选择的文本 | 1. 在子窗口中选中一段文本 2. 右键点击状态栏托盘图标打开菜单 3. 点击 Copy 4. 在主窗口或其他位置粘贴验证 | ① 粘贴得到的文本为子窗口中选中的文本 ② 不是主窗口的文本 ③ hilog 无 `Clipboard copy failed` 错误 | 验证：剪贴板操作使用目标窗口的 webview controller |
| core | predefined-multi-window | clipboard/cut | Tray Cut 子窗口 — 剪切子窗口选中文本 | **T1** | 应用已启动；已创建子窗口；子窗口有可编辑的文本输入框 | 1. 在子窗口的输入框中选中一段文本 2. 右键点击托盘图标打开菜单 3. 点击 Cut 4. 观察子窗口输入框 5. 在其他位置粘贴验证 | ① 子窗口输入框中选中的文本被删除 ② 粘贴得到的文本为被剪切的文本 | 验证 Cut 操作在目标窗口 webview 上执行 JS |
| core | predefined-multi-window | clipboard/selectAll | Tray SelectAll 子窗口 — 全选子窗口内容 | **T1** | 应用已启动；已创建子窗口；子窗口有文本内容 | 1. 确保子窗口有焦点 2. 右键点击托盘图标打开菜单 3. 点击 SelectAll 4. 观察子窗口文本选中状态 | ① 子窗口中所有文本被选中 ② 主窗口的文本未被选中 | 验证 SelectAll 操作在目标窗口 webview 上执行 |
| core | predefined-multi-window | clipboard/copy | Tray Copy 主窗口 — 复制主窗口选中文本 | **T1** | 应用已启动；主窗口有可选择的文本 | 1. 点击主窗口使其成为焦点 2. 在主窗口中选中一段文本 3. 右键点击托盘图标打开菜单 4. 点击 Copy 5. 在其他位置粘贴验证 | ① 粘贴得到的文本为主窗口中选中的文本 | 验证 fallback 到主窗口 controller 仍然正常工作 |
| core | predefined-multi-window | hide-restore | Menu Hide → 托盘左键恢复 | **T0** | 应用已启动；已创建 Full Test Tray；QuickOperation 的 abilityName 已清空（点击 "Disable QuickOp" 或将 abilityName 置空），确保左键点击托盘图标触发 icon click 事件 | 1. 右键点击托盘图标打开菜单 2. 点击 Hide 3. 确认应用隐藏到后台 4. 左键点击状态栏托盘图标 | ① 步骤 3 应用隐藏，所有窗口不可见 ② 步骤 4 应用恢复到前台，窗口重新可见 ③ hilog 输出 `startAbility succeeded` | 验证：hide → hideAbility() + 托盘 startAbility() 恢复；QuickOperation abilityName 必须清空，否则左键点击打开 QuickOp 面板而非触发恢复 |
| core | predefined-multi-window | hide-restore | Menu Close 主窗口 → 托盘左键恢复 | **T0** | 应用已启动；已创建 Full Test Tray；QuickOperation 的 abilityName 已清空 | 1. 点击主窗口使其成为焦点 2. 右键点击托盘图标打开菜单 3. 点击 CloseWindow 4. 确认应用隐藏到后台 5. 左键点击状态栏托盘图标 | ① 步骤 4 应用隐藏（主窗口 close 等价于 hideAbility），所有窗口不可见 ② 步骤 5 应用恢复到前台 ③ hilog 无 crash 或 freeze | 验证：closeWindow(id=0) → hideAbility()；主窗口不可 destroyWindow（WindowStage 会失效） |
| core | predefined-multi-window | window-lifecycle | Menu Minimize — 最小化到最近任务 | **T1** | 应用已启动 | 1. 右键点击托盘图标打开菜单 2. 点击 Minimize | ① 窗口最小化到最近任务列表 ② 从最近任务列表点击可恢复应用 ③ 行为与修改前一致（未回归） | 验证：minimize 行为不变 |
| core | predefined-multi-window | window-lifecycle | Menu Quit — 应用退出 | **T1** | 应用已启动 | 1. 右键点击托盘图标打开菜单 2. 点击 Quit | ① 应用完全退出 ② 不在最近任务列表中 ③ 行为与修改前一致（未回归） | 验证：quit 使用 terminateSelf()，行为不变 |
| core | predefined-multi-window | icon-click | 前台点击托盘图标 — 无副作用 | **T1** | 应用已启动且在前台；已创建 Full Test Tray；QuickOperation 的 abilityName 已清空 | 1. 确保应用在前台显示 2. 左键点击状态栏托盘图标 | ① 应用保持在前台，无闪烁或抖动 ② Tray 页面消息输出 `tray event: {"type":"click",...,"button":"Left","buttonState":"Up"}`（TrayIconEvent 已转发到前端） ③ hilog 无错误日志 | 验证：startAbility() 幂等安全 + iconClickHandler → bridge icon-click → Rust TrayIconEvent 事件链完整 |
| core | predefined-multi-window | restore | Tray ShowAll — 隐藏后恢复应用 | **T0** | 应用已启动；已创建 Full Test Tray（含 ShowAll 菜单项） | 1. 右键点击托盘图标打开菜单 2. 点击 Hide 3. 确认应用隐藏 4. 右键点击托盘图标打开菜单 5. 点击 ShowAll | ① 步骤 3 应用隐藏到后台 ② 步骤 5 应用恢复到前台 ③ 所有窗口可见 | 验证：showAll → showAbility() + 遍历窗口 showWindow() |
| core | predefined-multi-window | restore | Tray BringAllToFront — 隐藏后恢复应用 | **T0** | 应用已启动；已创建 Full Test Tray（含 BringAllToFront 菜单项） | 1. 右键点击托盘图标打开菜单 2. 点击 Hide 3. 确认应用隐藏 4. 右键点击托盘图标打开菜单 5. 点击 BringAllToFront | ① 步骤 3 应用隐藏到后台 ② 步骤 5 应用恢复到前台 ③ 所有窗口可见 | 验证：bringAllToFront 在 OHOS 上等价于 showAll（无跨应用置顶权限） |
| core | predefined-multi-window | restore | BringAllToFront 子窗口恢复 | **T1** | 应用已启动；已创建子窗口；子窗口处于最小化状态 | 1. 确保主窗口可见 2. 右键点击托盘图标打开菜单 3. 点击 BringAllToFront | ① 主窗口保持可见 ② 被最小化的子窗口恢复显示 | 验证：遍历 WindowManager 所有窗口调用 showWindow() 可恢复最小化子窗口 |
| core | predefined-multi-window | restore | 前台点击 ShowAll — 无副作用 | **T1** | 应用已启动且在前台；已创建 Full Test Tray（含 ShowAll 菜单项） | 1. 确保应用在前台，所有窗口可见 2. 右键点击托盘图标打开菜单 3. 点击 ShowAll | ① 应用保持在前台，无闪烁或异常 ② 所有窗口保持可见 ③ hilog 无错误 | 验证：showAbility() 幂等安全，showWindow() 对已可见窗口不产生副作用 |
| core | predefined-multi-window | clipboard/copy | MenuBar Copy 主窗口 — 通过 MenuBar 触发 Copy | **T0** | 应用已启动；主窗口有可选择的文本 | 1. 点击主窗口 MenuBar 打开菜单 2. 点击 Edit → Copy 3. 在其他位置粘贴验证 | ① 粘贴得到的文本为主窗口中选中的文本 ② 操作目标为主窗口 webview | 验证：Window Menu Bar 路径 targetWindowId 有值，直接操作菜单所属窗口 |

| 模块 | T0 | T1 | 合计 |
|------|-----|-----|------|
| Predefined Multi-Window（预定义操作多窗口支持） | 6 | 8 | **15** |

---

## 十五、Sentry（错误追踪）手动用例

> **测试应用**: `examples/api`（主测试应用）
>
> **前提**: sentry 插件已注册（`tauri_plugin_sentry::init`），DSN 已配置；设备已联网
>
> **测试入口**: TestRunner.svelte → "Sentry (错误追踪) Manual Tests" 区域
>
> **验证方式**: 优先通过自动测试报告 + 设备日志判断，Sentry 仪表盘为可选增强验证

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| core | sentry | JS Error 捕获 | JS Error Capture — WebView JS 异常捕获 | **T0** | 应用已启动；点击 "JS Error Capture" 按钮 | 1. 打开 DevEco Studio 检查日志 2. 点击 "JS Error Capture" 按钮 | ① 日志 输出 `[Sentry Test] Caught error: Error: OHOS test error from examples/api` ② `[ManualTest] Completed: sentryJsError` 确认测试完成 | 若 js_init_script 未注入，JS error 仍会被 WebView console.error 记录；注入验证：在 WebView 中执行 `typeof Sentry !== 'undefined'` |
| core | sentry | Rust Panic 捕获 | Rust Panic Capture — Rust panic 导致 app 崩溃 | **T1** | 应用已启动；点击 "Rust Panic (may crash)" 按钮 | 1.1. 打开 DevEco Studio 检查日志 2. 点击 "Rust Panic (may crash)" 按钮 3. 等待 2 秒，app 崩溃退出 4. 查看crash日志 | ① app 崩溃退出（预期行为，SIGABRT） ② cppcrash 日志 `Reason` 行包含 `Signal:SIGABRT(SI_TKILL)` ③ 栈回溯中 `libapi_lib.so` 出现在顶层帧（Rust panic → abort） ④ 崩溃时间与按钮点击时间吻合 | 仅在DEBUG模式下支持，sentry-panic crate 在 panic 时捕获事件并尝试上报；panic 导致进程退出需重启应用；breadcrumb/envelope/rust_breadcrumb 的 IPC 通路由自动测试 #74-#76 覆盖 |

| 模块 | T0 | T1 | 合计 |
|------|-----|-----|------|
| Sentry（错误追踪） | 1 | 1 | **2** |

---

## 十六、Unstable Feature（窗口与 Webview 解耦）手动用例

> **背景**: 补齐 wry OHOS `set_bounds`/`set_visible`/`bounds` 实现 + ProxyJsHelper pending path 修复；添加 Reparent OHOS 安全返回防死锁；移除 `add_child` 的 OHOS 排除。
>
> **测试入口**: TestRunner.svelte → "Unstable Feature (窗口与 Webview 解耦) Manual Tests" 区域

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| core | unstable | phase2/reparent | webview.reparent returns error — 防死锁验证 | **T0** | 应用已启动，进入 TestRunner 页面 | 1. 找到 `reparent returns error (no deadlock)` 2. 点击运行 3. 观察测试是否在 5 秒内完成 | ① 测试状态 PASS ② 查看日志 `webview.reparent(window)` 返回 Error ③ 不卡住（无 timeout） | 验证：`#[cfg(target_env = "ohos")]` Reparent handler 调用 `tx.send(Err(...))` 解除 `rx.recv()` 阻塞 |
| core | unstable | phase2/reparent | webview operations after failed reparent — 无级联死锁 | **T1** | 应用已启动 | 1. 找到 `reparent cascade check` 2. 点击运行 | ① 测试状态 PASS ② 查看日志 `webview.size()` 正常返回非零值 | 验证 reparent 失败后 `current_window_id` Mutex 锁被释放 |
| core | unstable | phase3/multi-webview | webview.create_webview — multi-webview 创建验证 | **T0** | 应用已启动；**Cargo.toml 需启用 `unstable` feature** | 1. 找到 `create_webview (multi-webview)` 2. 点击运行 3. 观察是否出现 300x200 子 webview 4. 等待 1 秒后子 webview 自动关闭 | ① 测试状态 PASS ② 子 webview 在 (50,50) 位置出现，显示 "Child Webview" ③ 1 秒后子 webview 关闭 | **需要 `unstable` feature**；验证 `add_child` + `dispose_child` 完整链路 |

| 模块 | T0 | T1 | 合计 |
|------|-----|-----|------|
| **合计** | **2** | **1** | **3** |

---

## 十七、Global Shortcut（全局快捷键）手动用例

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| plugin | global-shortcut | 注册与触发 | Register Shortcut — 注册快捷键并物理键盘触发 | **T0** | 应用已启动；设备连接物理键盘；进入 Tests 页面底部 Global Shortcut Manual Tests 区域 | 1. 点击 "Register Ctrl+Shift+T" 按钮 2. 确认状态显示 "Registered: CommandOrControl+Shift+T" 3. 用物理键盘按下 Ctrl+Shift+T | ① 状态变为 "Triggered! id=xxx, state=Released" ② 控制台输出 `[global-shortcut] Shortcut triggered: id=xxx, state=Released` | OHOS 使用 inputConsumer API（API 14+），仅在 key-down 时触发 Pressed 回调；代码合成 Released 事件以匹配 global-hotkey 合约，UI 最终显示 Released；最多支持 2 个修饰键 |
| plugin | global-shortcut | 注销验证 | Unregister All — 注销后快捷键不再触发 | **T0** | 已注册 Ctrl+Shift+T 且已验证触发成功 | 1. 点击 "Unregister All" 按钮 2. 确认状态显示 "All shortcuts unregistered" 3. 用物理键盘再次按下 Ctrl+Shift+T | ① 状态不再变为 "Triggered" ② 快捷键已被注销，系统不再拦截该组合键 | 验证 inputConsumer.off() 精确注销，不影响其他应用的快捷键 |

---

## 十八、窗口聚焦与热键缩放 手动用例

> **背景**: 窗口聚焦（set_focus）和热键缩放（Ctrl+/-/=）需要人眼确认的手动测试。
>
> **测试入口**: `examples/api` 应用 → Tests 页面 → **Window Focus + Hotkey Zoom Manual Tests** 区域

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| core | 窗口聚焦 | 多窗口层级 | Window Focus 多窗口层级验证 | **T0** | 应用已启动，进入 Tests 页面 | 1. 点击 "Window Focus" 创建子窗口 2. 手动将其他子窗口拖到该窗口上方 3. 再次点击 "Window Focus" | ① 首次点击创建 Float 子窗口 ② 再次点击调用 `setFocus()` → `raiseToAppTop()` ③ 窗口回到所有 Float 窗口最上方 | `Message::Task` 派发到主线程 → `focus_window(id)` → NAPI → `WindowManager.focusWindow` → `win.raiseToAppTop()` |
| core | 热键缩放 | Ctrl+/- | Ctrl+/- 缩放验证 | **T1** | 应用已启动，进入 Tests 页面 | 1. 点击 "Hotkey Zoom" 查看说明 2. 聚焦 webview 区域 3. 按 Ctrl + = 放大 4. 按 Ctrl + - 缩小 | ① 页面内容随快捷键放大/缩小 ② 缩放级别在 0.2~10 之间 | `zoom-hotkey.js` 通过 `cfg(desktop)` 注入。Ctrl+0 被 ArkWeb 引擎拦截，不生效 |

---

## 十九、Vibrancy（窗口模糊）手动用例

> 自动用例 2 个（side-effect）：
> 1. `window.setEffects(Blur/Acrylic) + clearEffects` 不抛错（运行时 setEffects，AttributeUpdater 刷新 backdropBlur/backgroundColor；Mica/Tabbed 系列在 OHOS 上为 no-op 跳过）
> 2. `create_transparent_window(effect=Blur)` build 时 effects 不抛错（WindowBuilder::effects，registerController inject）
>
> 以下为手动用例，通过 Tests 视图的手动按钮触发。vibrancy 窗口用 create_transparent_window（Float 子窗口，避开 UIAbility singleton 冲突）。

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| core | vibrancy | Blur | Blur effect visible | **T0** | 应用已启动，进入 Tests 视图 | 1. 点击 "vibrancy: Blur effect visible" 手动测试按钮 2. 观察弹出的透明窗口 | 窗口背景呈磨砂模糊（backdropBlur(25)），能透出背后内容且带模糊 | 窗口加载 vibrancy.html 透明页，Effect::Blur radius=25 |
| core | vibrancy | Acrylic | Acrylic effect visible | **T1** | 应用已启动，进入 Tests 视图 | 1. 点击 "vibrancy: Acrylic effect visible" 手动测试按钮 2. 观察弹出的透明窗口 | 窗口背景呈模糊 + 半透明深色 tint（blur + color） | Effect::Acrylic radius=25, color=[0,0,0,128] |
| core | vibrancy | clearEffects | clearEffects removes blur | **T0** | 应用已启动，进入 Tests 视图 | 1. 点击 "vibrancy: clearEffects removes blur" 手动测试按钮 2. 观察：先模糊 1s，然后 clearEffects 后模糊消失 | ① 初始窗口背景呈磨砂模糊 ② clearEffects 后窗口背景变清晰，且无半透明颜色遮罩（完全透出背后内容，不发暗/无色调） | 验证 clearEffects 同时移除 backdropBlur 和 backgroundColor tint |
| core | vibrancy | build-time effects | build-time Blur effect visible | **T0** | 应用已启动，进入 Tests 视图 | 1. 点击 "vibrancy: build-time Blur (WindowBuilder::effects)" 手动测试按钮 2. 观察弹出的透明窗口 | 窗口出现时即呈磨砂模糊（build 时 effects，非运行时 setEffects） | create_transparent_window(effect=Blur, radius=25)，WindowBuilder::effects 在窗口创建时 apply |

---

## 二十、Deep-Link 手动用例

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| core | deep-link | onOpenUrl | onOpenUrl 事件触发 — 运行中收到外部链接 | **T0** | app 已运行 | 1. 在 TestRunner UI manual 区点击 "onOpenUrl (trigger with hdc)" 按钮注册监听 2. 执行 `hdc shell "aa start -U taurideeplink://manualtest"` | UI 消息区显示 `[deep-link] onOpenUrl received: ["taurideeplink://manualtest"]` | RunEvent::Opened urls 非空时触发 |
| core | deep-link | getCurrent | getCurrent 冷启动 — 首启动链接拉起 | **T0** | app 未运行 | 1. `hdc shell "aa force-stop com.tauri.api"` 2. `hdc shell "aa start -U taurideeplink://coldstart"` 3. 等 app 冷启动后在 TestRunner UI manual 区点击 "getCurrent" 按钮 | UI 消息区显示 `[deep-link] getCurrent → ["taurideeplink://coldstart"]` | 冷启动 onCreate want.uri 经 lazy take 注入 |
| core | deep-link | 外部唤起 | 外部链接唤起 app — 跨 app 跳转 | **T0** | app 已安装 | 1. `hdc shell "aa force-stop com.tauri.api"` 2. `hdc shell "aa start -U taurideeplink://foreground-test"` | app 唤起到前台（onCreate 冷启动或 onNewWant 运行中） | aa start -U 与浏览器点击 `<a href>` 走相同系统 Want 路由（module.json5 skills 匹配）；浏览器地址栏直接输入 scheme 会被当搜索词 |

> **验证记录**: 2026-08-28 真机（HUAWEI MateBook Pro HAD-W32, desktop 形态）3 例全 **PASS**：
> - onOpenUrl 运行中触发 **PASS**：app 运行中点 "onOpenUrl (trigger with hdc)" 注册监听后 `aa start -U taurideeplink://manualtest`（11:20:54），hilog 全链路：`OnNewWant` → `NativeAbility onNewWant - uri: taurideeplink://manualtest` → `[single-instance] callback fired! args=["taurideeplink://manualtest", ...]` → `[RunEvent] Opened, urls=[taurideeplink://manualtest]` → `opened urls: [...]`。后端 RunEvent::Opened urls 非空触发，前端 onOpenUrl 回调显示该 url。
> - getCurrent 冷启动 **PASS**：force-stop → `aa start -U taurideeplink://coldstart` 冷启动（pid 55083，11:21:31），hilog：`onCreate` → `NativeAbility onCreate - moduleName: "api_lib"` → **`[deep-link] init_deep_link OHOS branch`**（onCreate 时把 want.uri 存入 INITIAL_WANT_URI 的 lazy-take 注入点）→ `[RunEvent] Ready/Resumed`。app 就绪后点 "getCurrent" 按钮，前端显示 `[deep-link] getCurrent → ["taurideeplink://coldstart"]`（ARKWEB-CONSOLE 域 A01194）。lazy-take 注入链路实锤。
> - 外部唤起 **PASS**：force-stop → `aa start -U taurideeplink://foreground-test`，app 冷启动唤起到前台（pid 55505，11:23:09 `onCreate` → `[RunEvent] Resumed`）。force-stop 杀死后外部 scheme 唤起成功拉起 app，跨 app 跳转路径正常（module.json5 skills 匹配 taurideeplink scheme）。

## 二十一、Window Operations（窗口操作）手动用例

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| core | window-ops | minimize | 窗口最小化与恢复 | **T0** | app 已运行，进入 Tests 视图 | 1. 在 TestRunner UI 底部 "Window Operations" 区找到 "Minimize then is_minimized" 按钮，点击 2. 窗口最小化到任务栏 3. 从任务栏点击 app 图标恢复窗口 4. 查看按钮下方显示的测试结果 | ① 窗口成功最小化到任务栏 ② 按钮下方显示 `isMinimized() = true` → PASS ③ 从任务栏恢复窗口后底部内容完整无缺失 | `win.minimize()` 调 `window.Window.minimize()`（API11）；`is_minimized()` 调 `getWindowStatus() === MINIMIZE`。恢复通过任务栏点击（系统行为），非 API 调用。 |
| core | window-ops | window-state | 窗口位置记忆与恢复 | **T0** | app 已运行 | 1. 将窗口拖动到一个明显的位置（如左上角） 2. 在 TestRunner UI 底部 "Window Operations" 区找到 "Window-State Save" 按钮，点击（保存当前位置） 3. 重启 app：终端执行 `hdc shell aa force-stop com.tauri.api` 后重新启动 4. 观察重启后窗口的位置变化 5. 也可在重启后点击 "Window-State Restore" 按钮手动恢复 | ① 重启后窗口先出现在屏幕中心（OS 默认位置） ② 随后窗口自动闪现到步骤 1 保存的位置 ③ 注意：自动测试中的 "set_position moves window" 用例会调 `setPosition(100,100)` 移动主窗口，可能覆盖恢复结果——请等自动测试跑完后（约 30 秒）再观察窗口最终位置 | OHOS 适配要点：① restore_state 从文件读取保存的位置（绕过被 Moved 事件覆盖的内存缓存）② 在 `RunEvent::Ready` 时对主窗口触发 restore（OHOS 的 `on_window_ready` 不对主窗口触发）③ `moveWindowTo` 对主窗口（id=0）用 `windowStage.getMainWindowSync()` 获取窗口句柄（主窗口不在 WindowManager 的 Map 中）④ NAPI 调用用 Object 传参（napi-ohos 不支持 3 元素元组）⑤ `inner_size()` 返回 window_rect（外尺寸）使 save→resize 循环幂等 ⑥ OHOS 跳过 `RunEvent::Exit` 自动保存（用户通过 Save 按钮显式控制）|
| core | window-ops | resize | 窗口缩放后底部内容完整 | **T0** | app 已运行，页面有可滚动内容 | 1. 用鼠标拖动窗口右边缘或下边缘向内缩小窗口 2. 松开鼠标后观察页面底部内容是否完整显示 3. 再拖动边缘向外放大窗口 4. 松开鼠标后再次观察 5. 重复缩放操作 3-5 次 | ① 缩小窗口后底部内容完整可见，无裁剪 ② 放大窗口后底部内容完整可见 ③ 多次缩放均正常 | 根因：commit `6fd8c0a` 把 Web 组件尺寸从 `.width("100%")`（自然布局）改为 `.width(data.style.width)`（set_bounds/BuilderNode.update），而 BuilderNode.update 不通知 ArkWeb 重新布局 → 缩放后底部被裁。修复：Web `.width/.height` 改回 `"100%"`，让 ArkUI 自然布局驱动 ArkWeb relayout。 |
| core | persisted-scope | save | fs scope 保存到文件 | **T0** | app 已运行（建议先点 "Persisted-Scope Clear" 清掉旧 `.persisted-scope` 避免残留干扰） | 1. 在 TestRunner UI 底部 "Window Operations & Persisted-Scope Manual Tests" 区点击 "Persisted-Scope Test" 按钮 2. 查看按钮下方显示的结果 3.（可选）`hdc shell ls -l <结果中的 state_file 路径>` 核对文件落盘 | ① `allow_directory: ✅ 成功` ② `.persisted-scope 文件: ✅ 已生成 (N bytes)` ③ `路径:` 显示 state_file 完整路径 | 因 OHOS 不支持 DragDrop（tao OHOS 未实现 DragDrop 事件），通过自定义 `test_persisted_scope` command 直接调 `scope.allow_directory(test_path, true)` 触发 PathAllowed 事件 → persisted-scope 插件监听该事件并把 `allowed_patterns()` 写入 `.persisted-scope`（bincode 二进制）。注意：该 command 返回 `allow_ok / test_path / state_file / state_file_exists / state_file_size`，**不返回 allowed_patterns 数量**，故本步只验证文件生成。 |
| core | persisted-scope | restore | 重启后 fs scope 自动恢复 | **T0** | 已执行 save 用例（`.persisted-scope` 文件已生成） | 1. 重启 app：`hdc shell aa force-stop com.tauri.api` 后重新启动 2. 重启后**先不要点 Test**（点 Test 会再次 `allow_directory` 同一路径，使 count 恒定，掩盖 restore 是否生效，见备注） 3. 直接点击 "Persisted-Scope Clear" 按钮 4. 查看按钮下方**结果框**（mono 字体 div）的 `remaining_patterns_count`（注意：消息区会被随后的 "Console log saved" 覆盖，看结果框或 hilog） | ① `文件删除: ✅ 已删除`（证明 `.persisted-scope` 跨重启留存）+ `remaining_patterns_count > 0` → ✅ restore 生效 ② `remaining_patterns_count = 0` → ❌ restore 失败（文件未读 / app_data_dir 在 setup 时不可用 / 反序列化失败） | persisted-scope 插件 setup 时读取 `.persisted-scope`（bincode 反序列化）并对每个 allowed_paths 调 `allow_path`→`scope.allow_directory` 恢复 fs scope。**count 判据修正**：`allow_directory(path, true)` 一次实际产生 **4 个 pattern**（非 2 个）——`push_pattern`（`crates/tauri/src/scope/fs.rs:87,126-128`）除 pattern 本身外还插入 `canonicalize_parent` 规范化变体，故 path+变体 + 各自 `/**` 递归 = 4。fs scope 是 HashSet 对同路径幂等去重，save→restore 对固定 test-path 幂等不累积。**实测 count=7 非泄漏**：4（test-persisted-scope 的 4 pattern）+ 3（.persisted-scope 启动恢复的先前会话固定路径，如先前 app_cache_dir alias 不同），全库无任何时间戳/变长路径来源（见 leak 调查子agent 结论）。故判据应为 `count > 0`（典型 ≥4），而非固定 2。`clear_persisted_scope` 是唯一返回 count 的入口（读 `scope.allowed_patterns().len()`，**只返回 .len() 不返回具体 pattern**——这是排查盲点），但会删 `.persisted-scope`，重复验证需先点 Test 重新保存。 |

## 二十二、Opener（打开文件/URL）手动用例

> autotest 已移除（原 `category:'manual'` 被运行器一律 skip，零覆盖）。opener 的 OHOS 实现走 `openharmony_ability::open_with_system` / plugin-url bridge `reveal-in-dir`（系统意图），行为依赖系统，必须人眼验证。测试入口：TestRunner 底部 "Plugins Manual Tests" 区按钮。
>
> **revealItemInDir 平台限制说明**：OHOS 文件管理器**不支持高亮选中文件**（无此 API），只能打开目标路径的**父目录**。**应用沙箱路径**（appCacheDir、`/data/storage/` 等）无法在文件管理器打开（平台限制，非 bug），会返回 documented 错误；只有**公共目录**（`/storage/media/100/local/files/<顶层>` 且顶层可映射为 FM 虚拟名）可 reveal。
>
> **验证记录**: 2026-08-28 真机（HUAWEI MateBook Pro HAD-W32）4 例：
> - openPath **PASS**：`openPath(/data/storage/el2/base/haps/entry_desktop/cache/opener-<ts>.txt) called.`（159ms）。文件落盘（病毒扫描印证），系统弹"选择打开方式"AppSelector（`isFileOpenScene:true reqMimeType:.txt`，10:49:43.966）——OHOS 无默认 txt handler 故弹选择器，符合预期②"或文件管理器"兜底。`plugin:opener|open_path` ok=true。
> - revealItemInDir 沙箱 **PASS**：`→ documented error (expected)` 错误含 `cannot open app-sandbox paths` + `This is a platform limitation`（59ms），`respond ok=false` 未拉起 FM。文件落盘 `opener-reveal-<ts>.txt`。与 [[reveal 文件管理器 uri 形态]] 一致。
> - revealItemInDir 公共目录 **PASS**（2026-08-28，Want 参数修复后真机验证）：根因是 Want 参数放错位置——代码把 URI 放 `want.uri`（顶层）但 FM MainAbility 只读 `want.parameters.fileUri` + `want.parameters.external_storage_uuid`，`want.uri` 被忽略→FM 默认导航 home（停主页）。修复（UrlPlugin.ets:195-209）：URI 改放 `parameters.fileUri` + `external_storage_uuid="LOCAL"`。真机验证：`initAddressByWant startAbilityFileUri: file://docs/storage/Users/currentUser/Documents`（修复前是 `home`）→ 双阶段 `setCurrentUri`（先 `myPC` 中间态，最终 `= file://docs/.../Documents`）= 导航成功。注：Rust 侧 `reveal_item_in_dir.rs:98` 取 `path.parent()`，故无论输入 `Docs` 或 `Docs/IDEProjects`，parent 均为 `.../Docs` → 映射 `Documents`（app 语义下 reveal 总打开文件所在父目录）。2026-08-20 旧"PASS"实为误报（FM 停首页/我的电脑被误读）；`empty uri when get uuid` 是 FM 每次启动的噪音非 URI 拒绝。见 [[reveal 文件管理器 uri 形态]]。
> - openUrl **PASS**：`openUrl('https://tauri.app') called.`（76ms）+ 浏览器 `com.huawei.hmos.browser` 拉起（pid 43906），DNS `tauri.app`→2 A 记录→TCP→SSL 证书校验→加载页面。`plugin:opener|open_url` ok=true。

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| core | opener | openPath | Opener openPath — 打开文件 ✅ | **T0** | app 已运行，进入 TestRunner → "Plugins Manual Tests" 区 | 1. 点击 "Opener openPath (open file)" 按钮 2. 观察系统反应 3. 查看按钮下方 manualResult 输出 | ① manualResult 输出 `openPath(<appCacheDir>/opener-<ts>.txt) called.` ② 系统弹出默认文本查看器/编辑器打开该文件（或文件管理器） ③ 无 `OpenharmonyAbility` 错误 | 2026-08-28 PASS。OHOS 实现：`commands.rs:84` `open_path` → `openharmony_ability::open_with_system(file_uri)`。文件写入 appCacheDir。`open with` 参数在 OHOS 被忽略。OHOS 无默认 txt handler→弹"选择打开方式"AppSelector（符合②"或文件管理器"兜底） |
| core | opener | revealItemInDir | Opener revealItemInDir — 沙箱路径返回 documented 错误 ✅ | **T0** | app 已运行 | 1. 点击 "Opener revealItemInDir (sandbox→err)" 按钮 2. 查看 manualResult 3. 观察系统反应 | ① manualResult 输出 `revealItemInDir(<appCacheDir>/opener-reveal-<ts>.txt) → documented error (expected):` ② 错误信息含 `app-sandbox paths` / `platform limitation` ③ 文件管理器/备忘录**不**打开 | 2026-08-28 PASS（`cannot open app-sandbox paths` + `platform limitation`，respond ok=false）。OHOS 平台限制：FM 无法打开应用沙箱路径。实现：`reveal_item_in_dir.rs` OHOS imp 传父目录真实路径 → `UrlPlugin.ets` `mapToVirtualUri` 沙箱检测 → `[reveal-in-dir]` 错误上抛 |
| core | opener | revealItemInDir | Opener revealItemInDir — 公共目录打开 FM ✅ | **T0** | app 已运行；输入框路径默认 `/storage/media/100/local/files/Docs/IDEProjects`（该目录需真实存在，可改为 Docs 下任意已存在文件/目录） | 1. 在输入框确认/填入公共目录下真实存在的路径 2. 点击 "Opener revealItemInDir (public dir→FM)" 3. 观察 FM | ① FM 打开所填路径的**父目录**（地址栏显示 `我的电脑>文档>...`） ② **不**高亮选中文件（OHOS 无此能力，平台限制，非 FAIL 项） ③ 无错误 | 2026-08-28 PASS（Want 参数修复后）。根因：原代码把 URI 放 `want.uri`（顶层）但 FM 只读 `want.parameters.fileUri`+`external_storage_uuid`，`want.uri` 被忽略→停主页。修复（UrlPlugin.ets:195-209）：URI 改放 `parameters.fileUri`+`external_storage_uuid="LOCAL"`。真机：`initAddressByWant startAbilityFileUri: file://docs/.../Documents`（修复前 `home`）→ 双阶段 `setCurrentUri`（先 `myPC` 中间态，最终 `= file://docs/.../Documents`）导航成功。Rust `reveal_item_in_dir.rs:98` 取 `path.parent()`，故 reveal 总打开文件父目录。实证形态：显式 Want `{bundleName:com.huawei.hmos.filemanager, abilityName:MainAbility, moduleName:pc, parameters:{fileUri:<虚拟uri>, external_storage_uuid:"LOCAL"}}`。仅 Documents 虚拟名实证；Desktop/Download 等推断待验。**公共路径在应用命名空间不可见**（/storage/media 未挂载 + hmdfs hmmac 拒绝，均 ENOENT），Rust 侧 canonicalize 失败透传。注：2026-08-20 旧"PASS"是误报（FM 停首页被误读）；`empty uri when get uuid` 是 FM 每次启动噪音非 URI 拒绝。 |
| core | opener | openUrl | Opener openUrl — 打开 URL ✅ | **T0** | app 已运行；设备已联网 | 1. 点击 "Opener openUrl (open browser)" 按钮 2. 观察系统反应 3. 查看 manualResult | ① manualResult 输出 `openUrl('https://tauri.app') called.` ② 系统浏览器打开 https://tauri.app ③ 无错误 | 2026-08-28 PASS。OHOS 实现：`commands.rs:42` `open_url` → `openharmony_ability::open_with_system(url)`。浏览器 DNS→TCP→SSL→加载全链路印证。**autotest 从未覆盖 openUrl**，仅手动验证 |

---

## 二十三、Store（持久化存储）手动用例

> autotest 仅覆盖内存 CRUD（set/get/has/keys/entries/delete/close），**刻意不碰 Exit/Drop 路径**。store timeout 修复（OHOS Drop-skip `store.rs:644`、Exit `save_or_skip` `store.rs:555`/`lib.rs:454`）是 defense-in-depth，autotest 不覆盖；磁盘持久化（set→退出→重开→数据在）也需手动验证。测试入口：TestRunner "Plugins Manual Tests" 区。
>
> **验证记录**: 2026-08-28 真机（HUAWEI MateBook Pro HAD-W32）3 例全 **PASS**：
> - 持久化-写入 **PASS**：`store.save() done. key='manual-sentinel' value='persisted-1787884832317' → manual-store.json.`（61ms）。病毒扫描日志印证落盘 `/data/app/el2/100/base/com.tauri.api/haps/entry_desktop/files/manual-store.json`。IPC `plugin:store|load→set→save` 全 `ok=true`。
> - 持久化-恢复 **PASS**：force-stop（pid 30862→35681）重启后 `store.get('manual-sentinel') → {"value":"persisted-1787884832317"}` + `PASS: value persisted across restart.`（44ms）。sentinel 值跨重启完全一致，磁盘反序列化恢复。
> - Exit 不阻塞 **PASS**：关主窗 CloseRequested(10:44:17.024)→Destroyed→ExitRequested→`[RunEvent] Exit`(10:44:17.033)→进程 `exit with code:0`(10:44:17.149)，全程 9ms 干净退出。零条 `StoreInner locked`/`save_or_skip` 异常、零 `appfreeze`/`THREAD_BLOCK`/ANR。OHOS Drop-skip（`store.rs:644`）+ Exit `save_or_skip` 降级（`store.rs:555`）未阻塞退出。

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| core | store | 持久化-写入 | Store Persist — set+save 落盘 ✅ | **T0** | app 已运行 | 1. 点击 "Store Persist (set+save)" 按钮 2. 查看 manualResult | ① manualResult 输出 `store.save() done. key='manual-sentinel' value='persisted-<ts>' → manual-store.json.` ② 无错误 | 2026-08-28 PASS。路径 `manual-store.json` 经 `resolve_store_path`（`store.rs:30`，`BaseDirectory::AppData`）解析到 AppData 目录落盘。autotest 不调 `save()`，本用例补此路径 |
| core | store | 持久化-恢复 | Store Verify — 重启后数据留存 ✅ | **T0** | 已执行 Persist 用例 | 1. force-stop app：`hdc shell aa force-stop com.tauri.api` 2. 重新启动 app，进入 TestRunner "Plugins Manual Tests" 区 3. 点击 "Store Verify (after restart)" 按钮 4. 查看 manualResult | ① manualResult 输出 `store.get('manual-sentinel') → {"value":"persisted-<ts>"}` ② 输出 `PASS: value persisted across restart.` ③ 若输出 `FAIL: value missing` → 持久化失败 | 2026-08-28 PASS（sentinel 1787884832317 跨 pid 30862→35681 一致）。验证 `manual-store.json` 跨进程重启留存 + 反序列化恢复。force-stop 模拟进程退出后重启 |
| core | store | Exit 不阻塞 | Store Exit — 退出无 appfreeze ✅ | **T1** | app 已运行，已 load 过 store（如执行过 Persist 用例） | 1. 正常关闭 app 主窗口（触发 `RunEvent::Exit`）2. 观察窗口是否立即关闭、无卡顿/超时 3. 重新启动 app 确认正常 | ① 窗口立即关闭，无 5s 卡顿/ANR ② 重启正常 ③ hilog 无 `store: StoreInner locked on exit, skipping save` 之外的异常 | 2026-08-28 PASS（CloseRequested→Exit 9ms，进程 exit code:0 干净退出）。验证 OHOS Drop-skip（`store.rs:644` OHOS 下 `Drop` 完全跳过 auto-save）+ Exit `save_or_skip` 降级（`store.rs:555`，mutex 争用时 try_lock 跳过）。这是 store timeout 三处修复的核心，autotest 无法触发 Exit/Drop |

---

## 二十四、Upload（文件上传）手动用例

> autotest 调 upload 并注册 progress 回调，但**只断言响应体非空，未断言 progress 回调触发**。本用例验证 progress 事件确实触发。测试入口：TestRunner "Plugins Manual Tests" 区。依赖 app 内 3003 端口 echo server（autotest upload 已验证可用）。
>
> **验证记录**: 2026-08-28 真机（HUAWEI MateBook Pro HAD-W32）**PASS**：`upload response: xxx...xxx (65536 bytes)` + `progress events: 8`（progressTotal 32768→40960→49152→57344→65536 分块递增）+ `PASS: upload succeeded`，165ms 完成。`writeFile` 64KB 文件 `upload-<ts>.txt` 落盘（病毒扫描日志印证）。备注担心的"fast small uploads 可能不触发 progress"未发生——64KB 在 localhost 触发了 8 次 progress 回调。

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| core | upload | progress 回调 | Upload — echo + progress 触发 ✅ | **T0** | app 已运行；3003 echo server 已起（autotest upload 通过即满足） | 1. 点击 "Upload (echo+progress)" 按钮 2. 查看 manualResult | ① manualResult 输出 `upload response: <非空>` ② 输出 `progress events: N`（N ≥ 1）③ 列出最近 5 条 progress 事件 ④ 输出 `PASS: upload succeeded` ⑤ 若 `FAIL: no progress events` → progress 回调未触发 | 2026-08-28 PASS（8 次 progress）。上传 64KB 文件到 `http://localhost:3003/up`，progress 回调收 `ProgressPayload{progress, progressTotal}`。autotest 仅 `Math.max(lastProgress, p.progress)` 抓了未断言，本用例补断言 |

---

## 二十五、Localhost（本地资源服务）手动用例

> autotest fetch `127.0.0.1:3005/index.html` 断言 200 + body，但**未直接断言 CORS 头**。本用例显式检查 `Access-Control-Allow-Origin`。测试入口：TestRunner "Plugins Manual Tests" 区。
>
> **验证记录**: 2026-08-28 真机（HUAWEI MateBook Pro HAD-W32）**PASS**：`status=200 bodyLen=968 ACAO=null` + `PASS: localhost serve OK.`。服务端确认设了 `Access-Control-Allow-Origin: *`（`localhost/src/lib.rs:134-137`，`#[cfg(target_env="ohos")]` 专属）+ Allow-Methods + Allow-Headers，**但前端 `resp.headers.get('access-control-allow-origin')` 读到 null**。根因：ArkWeb 对跨源响应只暴露 safelisted response headers（Cache-Control/Content-Language/Content-Type/Expires/Last-Modified/Pragma），ACAO 不在其中且服务端未设 `Access-Control-Expose-Headers` → 前端 JS 读不到该头。**但 fetch 跨源成功拿到 200+968 字节 body 本身即证明 CORS 实际放行生效**（否则 ArkWeb 会拦截跨源响应、拿不到 body）。故 ACAO=null 是 ArkWeb 响应头过滤行为，非服务端缺头、非 CORS 失效。

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| core | localhost | serve+CORS | Localhost fetch — 200 + CORS 放行 ✅ | **T0** | app 已运行；localhost 插件已在 3005 起服务 | 1. 点击 "Localhost fetch (CORS)" 按钮 2. 查看 manualResult | ① manualResult 输出 `fetch 127.0.0.1:3005/index.html → status=200 bodyLen=<N>` ② 输出 `PASS: localhost serve OK.` ③ **CORS 放行判据 = fetch 跨源成功拿到 body**（status=200 + bodyLen>0），而非前端读 ACAO 头 ④ `ACAO=null` 属正常（ArkWeb 不向跨源 JS 暴露此头，非服务端缺失）；仅当 status≠200 或 bodyLen=0 才算 FAIL | OHOS 绑 `127.0.0.1`（`lib.rs:110`，非 `localhost`）。服务端 CORS 头在 `lib.rs:132-146`（OHOS 专属：ACAO=`*` + Allow-Methods + Allow-Headers）。前端读 ACAO=null 因 ArkWeb 跨源响应头白名单过滤（未设 `Access-Control-Expose-Headers`），不代表 CORS 失效——fetch 拿到 body 即放行证据。 |

---

## 二十六、OHOS 适配真 gap 功能 手动用例

> **验证记录**: 2026-08-28 真机（HUAWEI MateBook Pro HAD-W32，desktop 形态）：
> - drag-overlay drag-in **PASS**（2026-08-28）：点 "Drag Overlay (§二十六)" 按钮弹出测试窗口（overlay 模式，`drag_drop_overlay=true`），从文件管理器拖入 `CodeAgentCLI-develop-green/.../package.json`，hilog `[DRAG-TEST]` 完整序列：Enter(358,268)→Over 持续(267,218→227,205)→Drop paths=["docs/storage/Users/currentUser/Desktop/CodeAgentCLI-develop-green/.../package.json"]@(227,205)。Drop 后无 Leave 属正常（Drop 即终态，ArkUI 不补 Leave；Leave 仅在拖出窗口不释放时触发）。事件链 Rust→ArkUI Stack→wry drag_drop_handler→WindowEvent→`[DRAG-TEST]` 不经页面 JS。
> - drag-overlay pointer-passthrough **PASS**（2026-08-28）：overlay 窗口内点击/滚动/选中文本均正常，`HitTestMode.Transparent`（`WebviewPlugin.ets:1341`）透传指针事件到下方 Web。
> - https-scheme 4 例（page-load/secure-context/external-https/subresource）**全 PASS**（2026-08-28，两层修复后真机验证）：点 "HTTPS Scheme" 按钮，hilog 全链路证据：
>   - create 时种子协议集成功：`[wry https-intercept] URL rewrite: tauri://localhost → https://tauri.localhost (use_https=true, protocols=["asset","myapp-async","tauri","isolation","myapp","ipc"])`——protocols 非空（修复前空集导致 onInterceptRequest early-return null）。
>   - **page-load PASS**：`[bridge https-intercept] received url=https://tauri.localhost/` → `[wry https-intercept] enter/extracted protocol='tauri'/reverted/calling handler/success status=200 mime=text/html body_len=968`。子资源 `index.js body_len=450853`、`rolldown-runtime.js`、`index.html` 均拦截成功。`location.href=https://tauri.localhost/`（非 arkweb-error 占位）。
>   - **secure-context PASS**：`[https-scheme] isSecureContext=true`（修复前 false）+ `[https-scheme] crypto.subtle OK, bytes=32`（修复前 undefined）。
>   - **subresource PASS**：`[https-scheme] subresource fetch OK: status=200 bytes=968`——子资源 fetch 被 onInterceptRequest 拦截返回内容。
>   - **external-https PASS**：`[https-scheme] external fetch resolved: type=opaque status=0`——外部 example.com fetch 未被误拦截（走真实网络栈，opaque 是 no-cors 正常返回），拦截器只处理 tauri.localhost 不误伤外部 https。
>   - 两层修复：①create 时种子 https_intercept_protocol_list（`wry/src/ohos/mod.rs:663-672`）治 create-vs-register 竞态；②`dispatch_https_intercept_sync` 用 `Arc<Mutex<Option<Response>>>` 非阻塞替 `rx.recv_timeout(3s)`（`mod.rs:1065-1149`）治主线程阻塞。见 [[OHOS https scheme issecurecontext=false]]。

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| ohos | drag-overlay | drag-in | Overlay 拖拽接收 — 文件拖入 webview ✅ | **T0** | desktop 形态。点 "Drag Overlay (§二十六)" 按钮即运行时创建 overlay 测试窗口（`create_ohos_test_webview` 传 `dragDropOverlay:true`），**无需改 app 配置重建** | 1. 点 "Drag Overlay (§二十六)" 按钮弹出测试窗口 2. 从文件管理器拖拽文件到测试窗口 webview 区域 3. 释放 4. hilog 搜 `[DRAG-TEST]` | ① `Enter` → `Over` → `Drop(paths)` 事件序列（Drop 后无 Leave 属正常，Drop 即终态）② paths 含拖入文件的 URI ③ Web 级 handler 被抑制（不双发） | 2026-08-28 PASS。事件链 `cmd.rs:1922-1942` `on_window_event`→`WindowEvent::DragDrop` match 打 `[DRAG-TEST]`。overlay 模式 = `WebviewPlugin.ets:1248` `data.dragDropOverlay===true` 分支，透明 Stack（`HitTestMode.Transparent`）接收 ArkUI drag 事件；主窗口走 direct 模式（`dragDropOverlay=false` 默认），ArkWeb 抢占 drop→靠 onLoadIntercept 拦 file:// 兜底恢复，机制不同，故必须用测试窗口。 |
| ohos | drag-overlay | pointer-passthrough | Overlay 透传 — 鼠标/触摸不受影响 ✅ | **T0** | 同上（overlay 窗口已渲染） | 1. 在测试窗口 webview 区域点击、滚动、选中文本 2. 页内 HTML5 拖拽（DOM 元素间拖动） | ① 鼠标点击/滚动/触摸正常响应 ② 文本选择正常 ③ HTML5 DnD 不被 overlay 干扰 | 2026-08-28 PASS。`HitTestMode.Transparent`（`WebviewPlugin.ets:1341`）透传指针事件到下方 Web。 |
| ohos | https-scheme | page-load | HTTPS Scheme — 页面加载 ✅ | **T0** | 应用已启动，进入 Tests 页面 | 1. 点击 "HTTPS Scheme" 按钮 2. 观察弹出的测试窗口页面是否渲染 3. hilog 搜 `onInterceptRequest` / `[wry https-intercept]` | ① `onInterceptRequest` 触发 ② custom_protocol 闭包被调用 ③ 页面 HTML 正常渲染 | 2026-08-28 PASS（两层修复后）。`[wry https-intercept] success status=200 mime=text/html body_len=968`，子资源 index.js/rolldown-runtime.js 均拦截成功。`location.href=https://tauri.localhost/`（非 arkweb-error）。修复：create 时种子协议集（`mod.rs:663-672`）+ 非阻塞 dispatch（`mod.rs:1065-1149`）。 |
| ohos | https-scheme | secure-context | HTTPS Scheme — Secure Context 验证 ✅ | **T0** | 同上；页面加载成功 | 1. 点 "HTTPS Scheme" 按钮（代码已自动执行探针，无需 DevTools）2. hilog 搜 `[https-scheme] isSecureContext=` 和 `crypto.subtle` | ① `[https-scheme] isSecureContext=true` ② `[https-scheme] crypto.subtle OK, bytes=32` ③ 不抛异常 | 2026-08-28 PASS（修复后）。`isSecureContext=true`（修复前 false）+ `crypto.subtle OK, bytes=32`（修复前 undefined）。`cmd.rs:1874-1900` init script 自动执行探针→hilog ARKWEB-CONSOLE。最终验收门槛已过。 |
| ohos | https-scheme | external-https | HTTPS Scheme — 外部 HTTPS 不被误拦截 ✅ | **T1** | 同上 | 1. 点 "HTTPS Scheme" 按钮 2. hilog 搜 `[https-scheme] external fetch` | ① `[https-scheme] external fetch resolved`（不被误拦截） | 2026-08-28 PASS。`external fetch resolved: type=opaque status=0`——外部 example.com 走真实网络栈（opaque 是 no-cors 正常返回），拦截器只处理 tauri.localhost 不误伤外部 https。 |
| ohos | https-scheme | subresource | HTTPS Scheme — 子资源 fetch/XHR 拦截 ✅ | **T1** | 同上 | 1. 点 "HTTPS Scheme" 按钮 2. hilog 搜 `[https-scheme] subresource fetch` | ① `[https-scheme] subresource fetch OK`（被 onInterceptRequest 拦截） | 2026-08-28 PASS。`subresource fetch OK: status=200 bytes=968`——子资源 fetch 被 onInterceptRequest 拦截返回内容。注：另有 CSP `default-src 'none'` 拒绝 connect（Refused to connect），但那是 CSP 策略非拦截器失败，探针本身 OK。 |

## 二十七、OHOS 适配 8 项功能 手动用例

> **背景**: openspec 8 项适配（clipboard/zoom flag、dialog error 降级、event 转发、monitor 真实值、dialog folder-picker、webview print、drag-drop）的手动验证。**2026-08-27 起约定：已由自动测试覆盖并验证的用例不保留在本文档**——monitor 刷新率用例（其测试步骤本身即"等 auto 运行"）由 `ohos-adapter.monitor.real-size`（auto，报告 #254 PASS）覆盖后移除，断言已收紧（连续两次调用 size 一致）；注：refreshRate 是 Rust `video_modes()` 字段，JS Monitor API 未暴露（name/size/position/workArea/scaleFactor），无手动验证路径。测试入口：TestRunner「OHOS Adapter Manual Tests」区按钮。
>
> **验证记录**: 2026-08-27 真机（HUAWEI MateBook Pro HAD-W32）：
> - monitor from-point **PASS**（3120×2080 物理像素，hilog 无 warn）
> - webview print **判据①②PASS、③FAIL=实锤缺陷**：系统打印对话框弹出（CreatePrintTask/Initialize success/job 1787833426439_812/CallSpooler 全链路 + printkit 弹窗）；用户点弹窗右上角 ✕ 取消。**临时 PDF `wry_print_*.pdf`（273KB）残留未清理**。根因：清理逻辑（`fileIo.unlinkSync` on succeed/fail/cancel 终态）只存在于遗留路径 `DefaultWebview.ets printPage`，实际运行的桥接路径 `WebviewPlugin.printPdf` 只注册终态监听转发 print-state + 维持 activePrintTasks 强引用，**从未删除文件**；wry Rust 侧（`PendingOp::Print`）也不删。权限拒绝等早退路径同样泄漏。属 [[bridge 重构丢失注入点]] 同族（print 迁移 bridge 时丢清理），待专项修。
> - event start-resumed **转发链 PASS、按钮判据不适用**：hilog `[RunEvent] Resumed` 在切前台瞬间触发（20:27:31.033）证明 Rust 转发正确；但按钮监听 `tauri://resumed` 永远 FAIL——**该事件在所有平台都不存在**（TauriEvent enum 无 resumed 成员，tauri core 从不向 JS emit RunEvent::Resumed），是按钮设计缺陷非适配缺陷。建议按钮改走 Rust probe 或标注平台语义。
> - event save-state **定性通过**：无法真机触发内存回收；代码路径保证（tao mod.rs:668-673 `debug!` drop，无 warn、不转发 Event）。
> - dialog error-degrade **PASS**（按钮显示说明信息 ×2，无崩溃；该函数 Windows-only，OHOS 分支 log::error! 不 panic）。
> - clipboard OFF/ON **PASS**：OFF 窗口选中文本 Ctrl+C 后粘贴，剪贴板内容不变（拦截生效）；ON 窗口复制粘贴正常（2026-08-27 用户确认）。
> - zoom OFF/ON **PASS**：OFF 窗口 Ctrl+=/-/0 页面缩放不变（拦截生效）；ON 窗口缩放/重置正常（2026-08-27 用户确认）。
>
> **§二十七 结论**：9 例中 8 例 PASS/定性通过，1 例判据 FAIL（webview print 判据③临时 PDF 清理，实锤缺陷待修，见上）；start-resumed 按钮监听 `tauri://resumed` 的判据设计缺陷已注明（事件在所有平台都不存在，Rust 转发本身正确）。

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| ohos | monitor | from-point | monitor_from_point — 边界判定 | **T1** | 应用已启动，进入 Tests 页面 | 1. 点击 "Monitor Info" 按钮 2. 查看输出的 monitor size + 测试点说明 3. hilog 确认 `monitor_from_point` 无 warn 日志 | ① 显示 monitor size（DisplayManager 物理像素） ② 屏幕内坐标返回 `Some(primary)` ③ 屏幕外坐标返回 `None` ④ 无 warn | OHOS 单显示器，边界判定 `0<=x<w && 0<=y<h`；JS API 不暴露 from_point，通过 hilog/Rust 验证 |
| ohos | webview | print | WebView 打印 — 系统打印对话框 | **T0** | 应用已启动；页面已加载（onPageEnd）；进入 Tests 页面 | 1. 点击 "WebView Print" 按钮 2. 观察系统打印对话框 3. 检查临时 PDF 清理（hilog 搜 `print`） | ① 弹出系统打印对话框 ② 打印任务提交后 `log.info('print: job submitted')` ③ 临时 PDF 文件清理（`fileIo.unlinkSync`） ④ 页面未加载时返回 Err | `@ohos.print` + `createPdf` 降级 |
| ohos | event | start-resumed | MainEvent::Start → Event::Resumed 转发 | **T0** | 应用已启动 | 1. 点 `RunEvent::Resumed` 按钮（监听 tauri://resumed）2. 按 Home 键将应用切到后台 3. 从最近任务列表切回应用 4. 看按钮结果（30s 内 PASS/FAIL） | ① 切回时 `Resumed` 事件触发，按钮显示 PASS ② hilog 无 `warn: TODO: forward onStart` ③ 与 SurfaceCreate/Resume 的重复 Resumed 可接受（幂等） | tao `MainEvent::Start`（SHOWN）转发为 `Event::Resumed`；按钮自动监听 30s |
| ohos | event | save-state | MainEvent::SaveState 降级 | **T1** | 应用已启动 | 1. 触发系统内存回收（打开多个应用占用内存后切回） 2. hilog 搜 `SaveState` | ① `debug: SaveState has no tao Event equivalent; dropped` ② 无 `warn` 噪音 ③ 不转发任何 Event | tao 无 SaveState 变体，降级为 debug log |
| ohos | clipboard | flag-off | with_clipboard(false) — 拦截 Ctrl+C | **T0** | 应用已启动，进入 Tests 页面 | 1. 点击 "Clipboard OFF" 按钮 2. 在弹出的测试窗口选中文本 3. 按 Ctrl+C 4. 在输入框 Ctrl+V 粘贴 | ① 剪贴板内容**不变**（未复制） ② hilog 无错误 | ArkUI `onKeyPreIme` 拦截 CLIPBOARD_ACCELERATORS |
| ohos | clipboard | flag-on | with_clipboard(true) — 正常复制 | **T0** | 应用已启动，进入 Tests 页面 | 1. 点击 "Clipboard ON" 按钮 2. 在弹出的测试窗口选中文本 3. 按 Ctrl+C 4. 在输入框 Ctrl+V 粘贴 | ① 剪贴板内容**已更新**（复制成功） | ArkWeb 原生处理（flag=true 不拦截） |
| ohos | zoom | flag-off | with_zoom_hotkeys(false) — 拦截 Ctrl+= | **T0** | 应用已启动，进入 Tests 页面 | 1. 点击 "Zoom OFF" 按钮 2. 在弹出的测试窗口按 Ctrl+= 3. 按 Ctrl+- 4. 按 Ctrl+0 | ① 页面缩放**不变** ② hilog 无错误 | `onKeyPreIme` 拦截 ZOOM_HOTKEY_ACCELERATORS |
| ohos | zoom | flag-on | with_zoom_hotkeys(true) — 正常缩放 | **T0** | 应用已启动，进入 Tests 页面 | 1. 点击 "Zoom ON" 按钮 2. 在弹出的测试窗口按 Ctrl+= 3. 按 Ctrl+- 4. 按 Ctrl+0 | ① Ctrl+= 放大 ② Ctrl+- 缩小 ③ Ctrl+0 重置 | ArkWeb 原生缩放（flag=true 不拦截） |
| ohos | dialog | error-degrade | dialog::error() 降级 — 不 panic | **T1** | 应用已启动；进入 Tests 页面 | 1. 点击 "Dialog Error (degrade)" 按钮 2. 查看 hilog 搜 `dialog::error` | ① 按钮显示说明信息（该函数仅 Windows 运行时调用） ② OHOS 分支为 `log::error!` 不 panic ③ 应用不崩溃 | `log::error!` 替代 `unimplemented!()`；实际运行时不触发（仅 Windows 调用） |

---

## 二十八、Window Ignore Cursor Events（窗口事件穿透）手动用例

> **背景**: Tauri `Window::set_ignore_cursor_events(ignore)` 在 OHOS 映射到 `ohos.window.setWindowTouchable(!ignore)`（`ignore=true` 穿透 ↔ `touchable=false` 不消费事件，取反在 tao 层）。**桥接走 WindowClient（plugin-window `set_window_touchable`，"set-touchable" typed bridge call）**，tao 层 `runtime.spawn` fire-and-forget 不等结果（防主线程死锁模式）；失败仅 hilog warn，不反向通知 Rust。
>
> **API 版本矛盾（待真机定论）**: 本地缓存文档标注 setWindowTouchable API 9+/12+，但华为官方智能问答确认为 **API 15+（HarmonyOS 5.0.0+）**。tauri api demo 默认 `compatibleSdkVersion = API 12`。若设备 API < 15，`win.setWindowTouchable` 为 undefined → ArkTS 同步抛 TypeError → 被 ArkHelper `safeLogError` 捕获，**不闪退**，仅穿透不生效。真机验证设备实际 API level 为定论步骤（design R5）。
>
> **测试入口**: `examples/api` 应用 → Tests 页面 → Manual Tests 区域 → `setIgnoreCursorEvents (3s toggle)` 按钮（smoke：toggle true→false 验证 TSFN 桥接 + 3s 穿透观察）。完整穿透验证需手动创建 Float overlay 子窗口（见 T0 用例）。
>
> **日志监控**: `hdc shell hilog | grep -iE "setWindowTouchable|WindowManager"`
>
> **验证记录**: 2026-08-27 真机（HUAWEI MateBook Pro HAD-W32，API 23）通过。新增专用按钮 `Overlay Ignore Cursor (穿透, §二十八)`（TestRunner）：`create_transparent_window` 创建 800×600 透明 Float 子窗口（label `manual-ignore-cursor-overlay`，WMS id 239）→ 子窗口 `setIgnoreCursorEvents(true)` → hilog 实锤 `WMSEvent SetTouchable: id:239, 0`（穿透生效），T0 点击穿透 + T1 hover 穿透视觉确认（下层主窗口按钮可点、hover 高亮正常）；30s 定时恢复 `SetTouchable: id:239, 1` 正常。设备 API 23 ≥ 15，T1 落在分支①（单 setWindowTouchable 足够，无需 hitTestBehavior fallback 与版本守卫）。穿透期间 overlay 自身 UI（含 "✕ Close" 链接）不可点击为**预期内**——窗口级 touchable=false 使整个窗口（含内部 webview）不消费事件，跨平台一致语义；恢复入口必须在其他窗口/定时器（本按钮即 30s 自动恢复）。
>
> **新发现缺陷（关闭生命周期，2026-08-28 已修复）**: overlay 的 "✕ Close" 链接走 `WebviewPlugin.ets` onLoadIntercept → `onCloseWindow` → `WindowManager.destroyWindow()`，ArkTS 侧直接销毁子窗口但不通知 Rust/tauri manager 摘除条目。后果：① `getAllWebviews()` 仍含已关闭 webview → `getByLabel` 返回僵尸句柄 → 同 label 重建被跳过（再次点击按钮无法创建新窗口）；② 对僵尸窗口的任何 window op（set-touchable 等）报 `Unknown OS sub-window '1' for this plugin instance`。
>
> **根因（深两层）**：① `WebviewPlugin.onCloseWindow` 回调调 `WindowManager.destroyWindow` 前漏调 `notifyWindowClose`（FloatPage × 按钮/aboutToDisappear 路径本有，但 close-window URL 路径漏）；② **更深的真根因**：`ProcessInitializer.initialize()` 从未 `AppStorage.setOrCreate(NATIVE_MODULE_STORAGE_KEY, ...)`，导致全仓所有 `AppStorage.get(NATIVE_MODULE_STORAGE_KEY)` 读取方（FloatPage × 按钮/aboutToDisappear、menu.ets closeWindow、NativeAbility windowStatusChange seed）恒 undefined → hilog 报 `NAPI notifyWindowClose not available` → notifyWindowClose 从未真正调用 → Rust drain 队列恒空 → tauri manager 残留僵尸条目。
>
> **修法（2026-08-28 落地验证通过）**：① `ProcessInitializer.ets` initialize 末尾补 `AppStorage.setOrCreate(NATIVE_MODULE_STORAGE_KEY, this.nativeModules[0])`（一处写入修复所有读取方）；② `WebviewPlugin.ets` onCloseWindow 在 destroyWindow 前补 notifyWindowClose（镜像 FloatPage × 按钮模式）。真机 hilog 实锤 `WebviewSurface: onCloseWindow: destroying windowId=1` + `FloatPage: Window 1 close notified via NAPI (system close)`（不再 not available）；windowId 递增 1→2→3 同 label 反复创建关闭全正常。详见 memory `ohos-appstorage-native-module-never-set`。

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| ohos | ignore-cursor-events | touch-passthrough | setIgnoreCursorEvents(true) 触摸穿透 | **T0** | 应用已启动；已创建一个 Float 子窗口叠在主窗口上方（如透明 overlay）；设备 API ≥ 15 | 1. 在 overlay 子窗口上调用 `setIgnoreCursorEvents(true)` 2. 用手指/鼠标点击 overlay 覆盖区域 3. 观察主窗口是否收到点击 4. hilog 搜 `setWindowTouchable` 5. 调 `setIgnoreCursorEvents(false)` 恢复 | ① 点击穿透到下层主窗口（overlay 不消费触摸/鼠标事件）② hilog 输出 `setWindowTouchable: window N touchable=false`（debug）③ `setIgnoreCursorEvents(false)` 恢复后 overlay 重新消费事件 | `ignore=true` ↔ `touchable=false`（tao 层取反）；fire-and-forget，Rust 返回 Ok 不代表 ArkTS 成功，以 hilog + 视觉为准 |
| ohos | ignore-cursor-events | hover-passthrough | setIgnoreCursorEvents hover 穿透 + API 版本 | **T1** | 同上 | 1. overlay 调 `setIgnoreCursorEvents(true)` 2. 鼠标悬停 overlay 覆盖区域 3. 观察下层主窗口的 hover/光标交互是否生效 4. 若 hover 不穿透，确认触摸仍穿透 5. 确认设备 API level（`hdc shell param get const.ohos.apicomversion` 或 deviceInfo.sdkApiVersion） | ① **API ≥ 15 且 hover 穿透**：单 setWindowTouchable 足够 ② **hover 不穿透但触摸穿透**：需追加组件级 `hitTestBehavior(HitTestMode.Transparent)`（参考 R72 drag-drop-overlay，task 4.3）③ **API < 15**：hilog 输出 `setWindowTouchable failed: ...`（TypeError），穿透完全不生效，需在 WindowManager 加 `deviceInfo.sdkApiVersion >= 15` 版本守卫静默跳过 | 真机为定论（design R1/R5）；hover fallback 走 task 4.3；版本守卫属底层仓（openharmony-ability）职责，不加在 tao 层 |


<!-- §二十九（OHOS 初始化链 init-chain）已于 2026-08-27 移除：3 个用例（window/menu/tray T0）全部由自动测试 `ohos-init.chain.window-menu-tray`（ohos-init.ts，side-effect 类别）覆盖并真机验证（2026-08-27 报告 #255 PASS）。回归签名（not initialized / not installed / client not initialized）已内嵌为 INIT_BREAK_PATTERNS 断言，判据强于原手动 hilog grep（grep 会被无关来源的 "not installed" 日志误伤——如 huawei-account 注册缺陷）。节号保留空缺以维持既有交叉引用。 -->

## 三十、OHOS Gap 补测（notification 触发/shell/updater）手动用例

> **背景**: 测试覆盖率分析发现的零覆盖缺口补测。自动测试位于 `examples/api/src/lib/tests/ohos-gap.ts`。**2026-08-27 起约定：已由自动测试覆盖并验证的用例不保留在本文档**——os 插件（type/family/arch/eol/exeExtension/version/locale/hostname，断言已收紧为 OHOS 精确值）与 clipboard（writeHtml/clear/round-trip）共 4 个用例从本节移除，证据见自动测试报告（2026-08-27 真机 289 例标准集 #256-269 全 PASS）。本节仅保留无法自动化的部分：通知回调**触发**（register 路径已自动化）、shell sidecar/Command 与 updater check（环境前置条件无法满足，仅手动占位）。
>
> **版本兼容策略**: 任务1（os.version/locale、clipboard writeHtml/clear 实现）已落地（2026-08-27 验证：os.version 真实版本号非 0.0.0 占位，clipboard 三项 PASS）。
>
> **验证记录**: 2026-08-27 真机（HUAWEI MateBook Pro）：① os 七项 + clipboard 三项 auto 全 PASS（报告 #256-269），按上述约定移出本节；② onAction 触发（T0）随 §三十二 4/4 验证通过（同一按钮）；③ onNotificationReceived 触发（T1）2026-08-27 补按钮 `Send & Listen (onNotificationReceived)` 后真机验证：注册/发布链路正常（`registerListener` ok=true、`notify→show` ok），两轮 15s 窗口回调均未触发（fired=false）——**定性平台限制**：华为官方确认 `notificationManager.subscribe` 为 `@systemapi`，需 `ohos.permission.NOTIFICATION_CONTROLLER`（system_basic 级，三方不可申请），三方应用无法订阅"通知到达"事件；插件设计注释"registration succeeds but no events will be delivered"与实测一致，按用例预期②"记录形态"判定通过；④ shell sidecar / updater check 维持占位——sidecar 需外部二进制配置（examples/api 不集成）；updater 需 AppGallery 发布环境，且当日实锤 `UpdaterBridgePlugin` Rust 侧未注册（连带缺陷，见 §三十一 验证记录）。

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| plugin | notification | onAction/trigger | onAction 触发 — 展开通知点 Action 按钮 | **T0** | 应用已启动；通知权限已授予；进入 Tests 页面 | 1. 点 Notification Manual Tests 区 `Send With Action Button (onAction)` 按钮 2. 下拉通知栏，展开 "Action 手动测试" 通知 3. 点击 "Tap Me" Action 按钮 4. 等待回调（热启动即时；冷启动需先杀进程，见 manual_tests §三十二） | ① console 输出 `PASS: onAction callback fired` ② 回调 payload 含 action id | manual 类别；回调触发依赖真机通知交付；2026-08-27 已补专用手动按钮（原引用的 `onAction trigger (manual)` autotest 按钮不存在——manual 类别被 Run All 过滤且无独立运行入口） |
| plugin | notification | onNotificationReceived/trigger | onNotificationReceived 触发 — 发送后回调 | **T1** | 同上 | 1. 点 Notification Manual Tests 区 `Send & Listen (onNotificationReceived)` 按钮（2026-08-27 补齐：原引用的 `onNotificationReceived trigger (manual)` autotest 按钮不存在——manual 类别被 Run All 过滤且无独立运行入口）2. 等待最多 15s | ① console 输出 `PASS: callback fired` ② 回调 payload 含通知内容 | manual 类别；**平台限制（2026-08-27 定论）**：`notificationManager.subscribe` 为 @systemapi（需 NOTIFICATION_CONTROLLER，三方不可申请），三方无法订阅通知到达事件；注册成功但无事件投递是插件文档化设计，fired=false 即最终形态 |
| plugin | shell | sidecar/Command | shell Sidecar/Command.spawn — 外部二进制 | **T1** | 应用已配置 `externalBin` sidecar 二进制（tauri.conf.json）+ 重新构建部署 | 1. 配置 sidecar 二进制路径 2. 点击 `plugin-shell.sidecar (manual)` 占位测试 3. hilog 搜 `sidecar` | ① sidecar 进程启动并 stdout 回传 ② Command.spawn 能获取子进程输出 | 成本高（需外部二进制 + tauri.conf 配置）；仅手动占位 + 草稿，examples/api 不集成 |
| plugin | updater | check | updater.check — AppGallery 更新检查 | **T1** | 应用已发布到 AppGallery 且存在更高版本 | 1. 点击 `plugin-updater.check (manual)` 占位测试 2. 查看 console 输出 | ① check() 返回非 null Update 对象（有新版本）② 无 AppGallery 源时 reject（预期）| 仅手动占位；需 AppGallery 环境（T1，前置条件重）；另有 UpdaterBridgePlugin Rust 侧未注册的连带缺陷（2026-08-27 实锤） |

---

## 三十一、OHOS 移动原生插件（barcode/biometric/geolocation/haptics/nfc/huawei-account）手动用例

> **背景**: 任务3 新适配的 5 个移动原生插件 + huawei-account 集成。UI 交互类流程无法自动化，自动测试仅覆盖安全子集（`examples/api/src/lib/tests/ohos-mobile-plugins.ts`：biometric.status / nfc.is_available / barcode.check_permissions / geolocation.check_permissions / haptics.selection_feedback 路由冒烟）。本节为 UI 绑定流程的手动用例。
>
> **前置**: 集成落地后重新构建部署（5 插件已注册到 examples/api lib.rs OHOS builder 链；entry module.json5 已声明 VIBRATE/LOCATION/APPROXIMATELY_LOCATION/CAMERA 权限）。测试入口：TestRunner「Mobile Native Plugins Manual Tests」区按钮（2026-08-27 补齐：Barcode Scan / Biometric Authenticate / NFC / Haptics / Huawei Account；geolocation 两按钮在「Geolocation Manual Tests」区）。
>
> **验证记录**: 2026-08-27 真机（HUAWEI MateBook Pro HAD-W32，pid 17635）点击前 5 个按钮实测，逐项「设备问题 / 代码问题」判定（hilog 全链路证据）：
>
> - **barcode scan（T0）— 偏设备/环境，待定位**。权限链完整通过：check_permissions → request_permissions → 系统弹窗 → selfPermissionStateChange settle（3.1s，轮询 attempt=0，§三十二权限竞态修复在此同样生效），camera=granted。scan 本身失败：首次 `code=1000500001 Internal error`；第二次进到 HMS ScanFrameworkUIServiceExtension（SetInputOptions/ExecuteStartScanCenter 均执行）后 UIExtension 启动失败 `errorCode=1011, name=start_ability_fail`。失败点在华为扫码框架内部拉起 ScanUIExtAbility，非我方代码路径；2in1 桌面形态不支持 HMS 扫码 UI 或 context/want 传法问题均有可能，待定位。相机权限弹窗链路本身已验证通过。
> - **biometric（T0）— 设备/系统侧**。status 返回 `{isAvailable:false, biometryType:0, error:'biometryNotAvailable'}`。ArkTS 为真实现（`userAuth.getAvailableStatus` 查 FACE+FINGERPRINT@ATL1），两查询均抛异常，非桩代码。定性：userIAM 对三方应用未暴露生物认证（或未录入指纹）。已知可观测性缺陷：getAvailableStatus 原始错误码被 catch 吞掉，无法区分「未录入」vs「不支持」，待透传修复。
> - **nfc（T1）— 设备侧，符合预期**。is_available 真实查询 `nfcController.isNfcAvailable()` 返回 false（PC 无 NFC 读卡器）；scan 按文档化设计决策 reject（tag 发现需 Ability 级 ACTION_TAG_DISCOVERED intent 集成），报错含能力说明，属预期行为。
> - **haptics（T1）— 设备侧，符合预期**。4 命令（vibrate/impact_feedback/notification_feedback/selection_feedback）全部真实调到 vibrator，被拒 `Device operation failed.`（PC 无马达）；错误如实上报、无假成功，降级路径正确。
> - **huawei-account（T1）— 代码缺陷（实锤）**。login/silent_login/logout 全部 reject `Bridge plugin 'ohos.account' is not installed for 'api_lib'`。根因：ArkTS `AccountPlugin.ets`（id=`ohos.account`）存在且 EntryAbility bridgePlugins 工厂已注册，但 **Rust 侧 `BridgePluginRegistry` 无任何 `register_plugin::<AccountBridgePlugin>` 调用点**——对照：tauri-runtime-wry 注册 webview/window/url，tray-icon 注册 statusbar/menu，plugin-resource 注册 resource；account 与 updater 均无（pluginize 重构丢注入点同款系统性缺陷）。修法方向：仿 tray-icon `set_ohos_app` 模式在 huawei-account 插件 init 中注册（待修）；`UpdaterBridgePlugin` 疑似同款漏注册。
> - **geolocation get_current_position（T1）— 已验证通过（2026-08-27 补测）**。`Get Current Position` 按钮：getCurrentPosition 1445ms 返回 `lat=30.184877, lng=120.1998408, acc=4.3m, alt=0`（Wi-Fi/网络定位，坐标与 §三十二 watchPosition fix 同区域），链路（JS→runCommand getCurrentPosition→geolocationmanager）正常。本节 6 用例全部完成实测定性。

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| plugin | barcode-scanner | scan | 扫码 — 拉起相机扫码 | **T0** | 应用已启动；CAMERA 权限已授予（首次触发系统弹窗） | 1. 点 `Barcode Scan (camera)` 按钮（内部先 check/request 权限再 scan）2. 对准任意二维码 3. 观察 manualResult | ① scan resolve 返回 `{ content, format }`，content 为二维码内容 ② 相机扫码 UI 正常拉起与关闭 ③ 无摄像头时 reject 且报错清晰 | context 注入修复后 scan 才可用（audit B2）；2026-08-27 实测权限链通过、scan UIExtension 1011 启动失败——判定见验证记录 |
| plugin | barcode-scanner | vibrate | 扫码成功振动反馈 | **T1** | 设备有振动马达 | 1. 完成一次 scan 2. 触发 vibrate 命令 | ① 设备振动 ② 无马达设备 reject/静默 | 复用 @ohos.vibrator |
| plugin | biometric | authenticate | 生物认证 — 拉起系统认证框 | **T0** | 应用已启动；设备已录入指纹/人脸 | 1. 点 `Biometric Authenticate` 按钮（内部先 status 再 authenticate）2. 完成认证/取消 3. 观察 manualResult | ① 认证成功 resolve（result.success=true）② 取消/失败 reject 且 errorCode 清晰 ③ 系统认证 UI 正常显示 | userIAM getUserAuthInstance 链路；PC 无生物识别硬件时 status.isAvailable=false |
| plugin | geolocation | get_current_position | 定位 — 获取当前位置 | **T1** | LOCATION 权限已授予；设备定位服务开启 | 1. 点 Geolocation Manual Tests 区 `Get Current Position` 按钮 2. 观察 manualResult | ① 返回 `{ coords: { latitude, longitude, ... }, timestamp }` 数值合理（Wi-Fi/网络定位）② 无 fix 时超时 reject——记录形态即可 | ~~watchPosition 流式回推是已知架构限制~~ **已过时**：emit/Channel 已落地（§三十二），watchPosition 流式回传真机验证通过（2026-08-27 收到位置 fix 回调） |
| plugin | haptics | vibrate 效果 | 触觉反馈 — 三种效果 | **T1** | 设备有振动马达 | 1. 点 `Haptics (vibrate/impact/notification/selection)` 按钮（内部依次调 4 个命令）2. 观察 manualResult | ① 各命令 resolve ② 有马达设备产生对应振动模式 | PC 无马达时 BusinessError 801→测试 skip（路由链已验证） |
| plugin | nfc | scan/write | NFC 扫描/写入 | **T1** | 设备支持 NFC；备一张可写 NFC 标签 | 1. 点 `NFC isAvailable + scan` 按钮 2. 观察 manualResult | ① is_available 返回布尔 ② scan 当前明确 reject（未实现，设计决策）③ 报错信息含能力说明 | scan/write 属下一轮；Plugin 基类 emit/Channel 已落地（§三十二）但 nfc scan 尚未接入；本轮只验 is_available |
| plugin | huawei-account | login | 华为账号一键登录 | **T1** | 设备已登录华为账号；AppGallery Connect 配置完成 | 1. 点 `Huawei Account (login/silent/logout)` 按钮 2. 完成一键登录授权 3. 观察 manualResult | ① resolve 返回 { openId, unionId, ... } ② silent_login 免弹窗返回 ③ logout 后 silent_login reject | 需真实华为账号环境；零自动覆盖为已知缺口（任务5 结论），真机集成验证补；2026-08-27 实测全 reject——Rust 侧 AccountBridgePlugin 未注册（代码缺陷），判定见验证记录 |

---

## 三十二、OHOS Plugin 基类 emit/Channel 事件回传机制

> **背景**: 打通 ArkTS→webview 事件流：ArkTS `Plugin.emit(channelId, payload)` → NAPI `tauri_send_channel_data` → Rust CHANNELS 注册表 → `Channel.send` → webview.eval → JS 回调。对标 Android `send_channel_data` / iOS `send_channel_data_handler`。
>
> **改动范围**: Rust（channel.rs cfg + mobile.rs CHANNELS pub + ohos_plugin.rs NAPI）、ArkTS Plugin 基类（emit/setEmitHandler/parseChannelId/onNotificationAction）、PluginManager（getPlugin）、EntryAbility（setEmitHandler 注入 + onNewWant/handleNotificationAction）、geolocation（watchPosition channel emit）、notification（registerListener/removeListener + action dispatch）。
>
> **自动测试**（`examples/api/src/lib/tests/ohos-mobile-plugins.ts`）：notification.registerListener（注册/注销不报错即通过）。geolocation watchPosition 的 emit 事件流依赖设备位置开关与位置 fix，环境依赖强，转为手动用例（TestRunner「Geolocation Manual Tests」两按钮：①请求权限+打开定位设置 ②Watch Position (emit)）。
>
> **验证记录**: 2026-08-27 真机（HUAWEI MateBook Pro）4/4 用例通过。① 权限链：requestPermissionsFromUser 发起→settle 3.3s（selfPermissionStateChange 兜底胜出，四路竞合去重正常，轮询 attempt=1 双 granted），无挂起；② watchPosition：10s 窗口收到 1 次位置 fix（lat=30.1849/lng=120.1998/acc=3.6m，Wi-Fi 定位），clearWatch 正常，emit 端到端验证通过；③ 冷启动：`aa force-stop` 后点 action → 新 pid onCreate 拉起 + `Notification action: id=9001` 派发，emit 被吞（`No listener registered` warn，无 crash，文档预告限制精确复现）；④ 热启动：onNewWant 派发 + `evaluate-script`（webview.eval）注入回调链 hilog 闭环。另补手动按钮 `Send With Action Button (onAction)`（TestRunner Notification Manual Tests 区，热/冷启动共用，监听常驻跨后台）。

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| plugin | geolocation | 权限+开关 | 请求权限 + 打开定位设置（按钮一） | **T1** | 应用已安装 | 1. 点击「请求权限 + 打开定位设置」按钮 2. 系统弹权限对话框时选"允许" 3. 跳转设置页后开启「定位服务」总开关 4. 返回应用 | ① 弹出位置权限对话框（LOCATION + APPROXIMATELY_LOCATION）② 跳转到系统定位设置页（uri=location_manager_settings；失败则兜底跳应用详情页 application_info_settings）③ requestPermissions 返回 granted ④ 弹窗授权后数秒内完成（不挂起） | 平台坑（2026-08-22 修复+真机验证）：`requestPermissionsFromUser` 的 Promise 在地图预览弹窗形态下可能永不 resolve（事件循环不冻结，是 Promise 本身不结算）→ ArkTS 侧 fire-and-forget + 四路兜底 settle（onForeground 生命周期 / on('selfPermissionStateChange') 事件（API 18+）/ 60s setTimeout 安全网 / Promise 本身）；且 selfPermissionStateChange 事件在 ATM 提交前触发，同步 checkAccessTokenSync 读到旧 denied → settle 后轮询（立即首查+300ms×6 次直到全 granted）；应用级权限与系统总开关是两道独立门槛，总开关关闭时 locManager 报 3301100 |
| plugin | geolocation | watchPosition | Watch Position 位置流回传（按钮二） | **T1** | 按钮一已完成（权限 granted + 定位服务开启） | 1. 点击「Watch Position (emit)」按钮 2. 观察 10s 内结果区的位置更新计数 3. 自动 clearWatch 结束 | ① watchPosition resolve 返回 channelId ② 设备产生位置 fix 时收到 `{ coords: { latitude, longitude, accuracy, ... }, timestamp }` 回调（计数递增）③ 结果区显示「emit 端到端链路验证通过」④ clearWatch 后不再有回调 ⑤ 无位置 fix 时提示注册/注销链路已通过，事件流待有 fix 设备验证 | 验证链路：locationChange → Plugin.emit(channelId, position) → NAPI tauri_send_channel_data → Rust CHANNELS → Channel.send → JS 回调；MateBook Pro 无 GPS，事件依赖 Wi-Fi/网络定位 fix |
| plugin | notification | actionPerformed | 通知 action 按钮 — 冷启动 | **T0** | 通知权限已授予；已 registerActionTypes；前台发一条带 actionTypeId 的通知 | 1. `onAction(cb)` 注册监听 2. 发通知 `notify({ id, title, body, actionTypeId })` 3. 切到后台 4. 点击通知 action 按钮 5. App 冷启动拉起 | ① App 被拉起 ② cb 收到 `{ id, actionId }` ③ actionId 与点击的按钮一致 | 冷启动 webview 可能未就绪→emit 被 warn 吞（不 crash）；热启动更可靠 |
| plugin | notification | actionPerformed | 通知 action 按钮 — 热启动 | **T1** | 同上；App 在后台运行 | 1. `onAction(cb)` 注册监听 2. 发通知 3. 点击通知 action 按钮 4. App 回到前台（onNewWant） | ① cb 收到 `{ id, actionId }` ② actionId 与点击的按钮一致 ③ `removeListener` 注销后不再收到回调 | 热启动走 onNewWant→handleNotificationAction→onNotificationAction→emit 链路 |

---

## 三十三、Key Repeat Detection（键盘连发检测）手动用例

> **背景**: ArkWeb 物理键盘走 IME 插入管线，原生 DOM keydown 为空壳事件（无 key/code、不连发、e.repeat 恒 false，且每个重复周期合成一对假 keydown/keyup）。`ohos-webview-key-synthesis` 在 MainPage.onKeyPreIme 检测连发并派发合成 KeyboardEvent（带按键身份与 repeat），shim 抑制原生退化事件。测试入口：Tests 页面 → Key Repeat Detection (OHOS desktop / 2in1)。

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| webview | key-synthesis | 长按连发 | Hold Key — 长按按键触发连发 repeat | **T0** | 应用已启动；设备连接物理键盘；进入 Tests 页面 Key Repeat Detection 区域 | 1. 点击 "Start" 2. 点击输入框获取焦点（光标闪烁） 3. **长按**字母键 j 约 3 秒（手指不松开） 4. 松开 | ① 首行 `D key="j" code=KeyJ repeat=false` ② 长按期间连续多行 `D ... repeat=true`（绿色高亮），中间**无 U 行、无灰色空壳 D/U 对** ③ 松开后单行 `U key="j" code=KeyJ` ④ 输入框正常连出 `jjjj`（无翻倍） | 关键判定：`code=KeyJ` 有值 + `repeat=true` 连发；灰色行=shim 失效（scriptRules 匹配问题） |
| webview | key-synthesis | 点按对照 | Tap Key — 快速点按不触发 repeat | **T0** | 同上；已完成长按用例 | 1. 快速点按字母键 j 一次（按下立即松开） 2. 再快速点按一次 | ① 每次 `D ... repeat=false` ② 每次点按配对一行 `U` ③ 无 repeat=true 出现 | 验证 Up 正确清除按下集合，不误报 |
| webview | key-synthesis | 快捷键拦截 | Accelerator Interception — 被消费按键不泄入页面 | **T1** | 应用配置了菜单快捷键（如 Ctrl+Shift+T 类 accelerator） | 1. 长按快捷键组合 2 秒 | ① 页面**不**收到该组合键的 keydown/keyup（被 onKeyPreIme 拦截链消费） ② 松开修饰键后页面收到修饰键事件正常 | 验证 fall-through 接线：拦截优先于合成 |
| webview | key-synthesis | 告警检查 | No Warnings — hilog 无派发失败 | **T1** | 已完成长按用例 | 1. `hdc shell "hilog -x \| grep KeySynthesis"` | ① 无 `runJavaScript failed` 告警 | 派发失败仅 warn 不刷屏（20Hz 连发下静默成功） |

---

## 三十四、手动用例统计汇总

| 模块 | T0 | T1 | 合计 |
|------|-----|-----|------|
| Tray（系统托盘） | 5 | 6 | **11** |
| Menu — MenuBar | 9 | 14 | **23** |
| Menu — PopupMenu | 3 | 3 | **6** |
| Clipboard — writeImage | 4 | 3 | **7** |
| Dialog | 7 | 0 | **7** |
| plugin-os（平台检测） | 2 | 4 | **6** |
| Autostart（开机自启动） | 2 | 2 | **4** |
| Webview — createPdf | 1 | 1 | **2** |
| Webview — Cookie | 1 | 0 | **1** |
| Webview — DevTools | 0 | 1 | **1** |
| Webview — Fullscreen | 1 | 0 | **1** |
| WebView User-Agent | 1 | 2 | **3** |
| RunEvent（生命周期事件） | 1 | 3 | **4** |
| Transparent（透明窗口） | 1 | 1 | **2** |
| on_new_window（新窗口拦截） | 2 | 1 | **3** |
| Single-Instance（单实例） | 3 | 1 | **4** |
| Predefined Multi-Window（预定义操作多窗口支持） | 6 | 8 | **14** |
| Notification（通知） | 1 | 2 | **3** |
| Sentry（错误追踪） | 1 | 1 | **2** |
| Unstable Feature（窗口与 Webview 解耦） | 2 | 1 | **3** |
| Global Shortcut（全局快捷键） | 2 | 0 | **2** |
| 窗口聚焦与热键缩放 | 1 | 1 | **2** |
| Vibrancy（窗口模糊） | 3 | 1 | **4** |
| Deep-Link（深度链接） | 3 | 0 | **3** |
| Window Operations（窗口操作） | 3 | 0 | **3** |
| Persisted Scope（fs scope 持久化） | 2 | 0 | **2** |
| Opener（打开文件/URL） | 4 | 0 | **4** |
| Store（持久化存储） | 2 | 1 | **3** |
| Upload（文件上传） | 1 | 0 | **1** |
| Localhost（本地资源服务） | 1 | 0 | **1** |
| OHOS — Drag Overlay（拖拽降级） | 2 | 0 | **2** |
| OHOS — HTTPS Scheme（安全上下文） | 2 | 2 | **4** |
| OHOS — Monitor（真实值 + from-point） | 0 | 1 | **1** |
| OHOS — WebView Print（打印） | 1 | 0 | **1** |
| OHOS — Event Lifecycle（Start→Resumed + SaveState） | 1 | 1 | **2** |
| OHOS — Clipboard Flag（with_clipboard 开/关） | 2 | 0 | **2** |
| OHOS — Zoom Flag（with_zoom_hotkeys 开/关） | 2 | 0 | **2** |
| OHOS — Dialog Error（降级不 panic） | 0 | 1 | **1** |
| OHOS — Window Ignore Cursor Events（事件穿透） | 1 | 1 | **2** |
| OHOS Gap — notification 触发（onAction/onNotificationReceived） | 1 | 1 | **2** |
| OHOS Gap — shell sidecar/Command（占位） | 0 | 1 | **1** |
| OHOS Gap — updater check（AppGallery 占位） | 0 | 1 | **1** |
| OHOS 移动原生插件 — barcode-scanner（scan/vibrate） | 1 | 1 | **2** |
| OHOS 移动原生插件 — biometric（authenticate） | 1 | 0 | **1** |
| OHOS 移动原生插件 — geolocation（定位/权限） | 0 | 1 | **1** |
| OHOS 移动原生插件 — haptics（三种效果） | 0 | 1 | **1** |
| OHOS 移动原生插件 — nfc（is_available/scan/write） | 0 | 1 | **1** |
| OHOS 移动原生插件 — huawei-account（一键登录） | 0 | 1 | **1** |
| OHOS Plugin emit/Channel（geolocation watch/notification action） | 1 | 3 | **4** |
| OHOS — Accessibility 插件（fontScale/屏幕阅读器） | 0 | 3 | **3** |
| OHOS — Screenshot 插件（截图预览/色块取色） | 1 | 1 | **2** |
| OHOS — Continuation 插件（接续边界/源端保存/双设备往返） | 0 | 3 | **3** |
| Key Repeat Detection（key-synthesis 长按/点按） | 2 | 2 | **4** |
| **合计** | **93** | **83** | **176** |

> **统计口径（2026-08-27 起）**: 已由自动测试覆盖并验证的用例不保留在本文档（从 §三十 移除 os 七项 + clipboard 三项，断言收紧进 `ohos-gap.ts`）。2026-08-27 逐行实核：此前合计含 4 个幽灵 T0（声称 96/78/174，实际 92/77/173），已按逐节表格行修正为 92/79/171（含本日新增 continuation T1 一例）。同日 Phase 3c 二次实核又发现分项表 6 处与表格行不符（Opener 少计 1 T0、Monitor 多计 1 T0、webPageSnapshot 幽灵行、emit/Channel 多计 1 T1、Accessibility 错记 1 T0 为 T1），已全部修正；当前合计 93 T0 / 83 T1 / 176（含 Phase 3c 新增 continuation 源端保存 + 双设备往返 2 T1，及 key-synthesis 新增 2 T0 + 2 T1）。以逐行 grep 实数为准（`grep -cE "\*\*T0\*\*"` / `"\*\*T1\*\*"` 校验行数，勿用 -o 计出现次数）。

---

## 三十五、OHOS P1/P2 新插件（无障碍/截图取色/应用接续）手动用例

> **背景**: 2026-08-27 交付的无障碍、截图取色与应用接续（被动恢复）插件。只收录自动测试**未覆盖**的维度（系统设置联动、事件端到端、人眼比对、接续边界）；已被自动用例覆盖的断言（getFontScale 正数、红块阈值、base64 前缀+宽高、接续普通启动 false/null）不再重复。splash/字体（P0）已有 hilog/静态验证结论，不设手动用例。
>
> **自动测试**: 无障碍 `plugins.ts`（getFontScale auto / 查询 auto / onAccessibilityStateChange manual-console）；截图 `ohos-screenshot.ts`（captureWebview auto / 红块取色 side-effect / 越界 auto / demo manual-console）；接续 `ohos-continuation.ts`（isContinuationRestoreLaunch false+peek 幂等 auto / getContinuationData null+take 幂等空 auto / setContinuationData 保存+清空+超限拒绝 auto / demo manual-console）。
>
> **手动入口**: 无障碍三按钮在 TestRunner「Accessibility Manual Tests」区；截图在 App 侧栏「Screenshot」demo 页；接续在 App 侧栏「Continuation」demo 页。
>
> **双设备接续迁移流用例**: 见下表 continuation source-restore 行（Phase 3c 已交付源端保存 setContinuationData + onContinue 快照 + continuable 门控）。

| 一级场景 | 二级场景 | 三级场景 | 用例名称 | 用例级别 | 预置条件 | 测试步骤 | 预期结果 | 备注 |
|---------|---------|---------|---------|---------|---------|---------|---------|------|
| plugin | accessibility | fontScale | 字号缩放跟随系统设置 | **T1** | 应用已安装 | 1. TestRunner 点「Font Scale 查询」记录当前值 2. 系统设置 → 显示和亮度 → 字体大小与显示大小，调大字号 3. 返回应用重新点击按钮 | ① 第一次返回正数（默认 1.0）② 第二次返回值 > 第一次（fontSizeScale 跟随系统变化） | 零权限；auto 用例只断言正数，联动变化需手动 |
| plugin | accessibility | screenReader | 屏幕阅读器查询与开关对照 | **T1** | 应用已安装 | 1. TestRunner 点「Screen Reader 查询对照」记录两查询值 2. 设置 → 辅助功能，开关屏幕阅读器 3. 返回重新点击按钮 | ① `isScreenReaderEnabled()` 与系统开关一致 ② 切换后查询值跟随变化 ③ 全程无权限错误（真机实测 ACCESSIBILITY 只读不设防） | `isTouchExploreEnabled()` 同理（触屏设备）；2in1 桌面形态 touchExplore 恒 false 属正常 |
| plugin | accessibility | stateChange | 屏幕阅读器状态事件端到端 | **T1** | 应用已安装 | 1. TestRunner 点「State Change Watch (20s)」 2. 20s 内：设置 → 辅助功能，开关屏幕阅读器 | ① 结果区显示「✅ 状态事件链路验证通过：共 N 次事件」② 事件 payload enabled 与开关动作一致 | 订阅注册已 hilog 实锤（Observer has subscribed）；本用例验证 emit→listen 端到端事件流，auto 测试无法触达（需操作系统设置） |
| plugin | screenshot | preview | 截图预览与页面一致 | **T0** | 应用已安装 | 1. App 侧栏切到「Screenshot」页 2. 点「📷 截图预览」 3. 检查预览 img | ① 预览图完整显示当前页面（含 5 个色块）② 无空白/截断 ③ 尺寸信息 ≈ 视口物理分辨率（实测 2092×1249） | auto 只断言 base64 前缀+宽高；图像内容正确性需人眼 |
| plugin | screenshot | pickColor | 全色块取色人眼比对 | **T1** | Screenshot 页已打开 | 1. 依次点击 5 个色块（#FF0000/#00FF00/#0000FF/#FFFFFF/#000000） 2. 核对每次显示的 rgba | ① 各色块通道与色值一致（±极小偏差；红块实测像素级精确 rgba(255,0,0,255)）② 显示的 snapshot 坐标与色块位置对应 | auto 已覆盖红块阈值断言；本用例补全其余 4 色块+人眼维度 |
| plugin | continuation | boundary | 非 CONTINUATION 启动的参数不误判为接续 | **T1** | 应用已运行， hdc 可用 | 1. App 侧栏「Continuation」页点「查询恢复状态+数据」记录基线（普通启动：false/null） 2. `hdc shell aa start -b com.tauri.api -m entry_desktop -a EntryAbility --ps customKey customValue` 触发 onNewWant（带 parameters 的普通 want） 3. 回到 Continuation 页再点查询 | ① `isContinuationRestoreLaunch()` 仍为 false（launchReason 非 CONTINUATION）② `getContinuationData()` 仍为 null ③ hilog 无 onNewWant 异常 | 边界验证：带参数的普通启动走 deep-link/WANT_PARAMETERS 通道，不误入接续存储；真接续（launchReason=CONTINUATION）单设备不可注入，双设备流见下行 |
| plugin | continuation | source-save | 源端快照保存与 onContinue AGREE | **T1** | 双设备（同华为账号、均安装 app、`tauri.conf.json` 开 `bundle.openHarmony.continuable: true` 后 build 部署） | 1. 源设备 Continuation 页输入 payload（如 `{"route":"/article/42"}`），点「💾 保存快照」 2. `hdc shell hilog | grep -i onContinue` 开始监听 3. 从系统迁移入口（超级终端/接续）把 app 迁移到目标设备 | ① 源端 hilog 出现 `onContinue - AGREE, snapshot length: N`（N 为 payload 长度）② 目标设备 app 启动且迁移完成 | 空快照对照：先点「🧹 清空快照」再迁移，应出现 `onContinue - empty snapshot, refusing (MISMATCH)` 且系统提示不可接续 |
| plugin | continuation | source-restore | 双设备完整往返（set → 迁移 → 目标端恢复） | **T1** | 上一用例通过（源端已 set 快照） | 1. 在源设备完成迁移 2. 目标设备 app 内进入 Continuation 页，点「isContinuationRestoreLaunch」 3. 点「getContinuationData」查看 payload | ① 目标端 `isContinuationRestoreLaunch()` === true（launchReason 为 CONTINUATION）② `getContinuationData()` 返回 JSON 串且 `JSON.parse(...).continuationData` 与源端 set 的 payload 逐字一致 ③ 再次调用返回 null（消费型） | 往返约定：源端 setContinuationData ↔ 目标端 JSON.parse(getContinuationData()).continuationData；continueType 缺省回退 `[identifier]`（com.tauri.api），同 app 双设备自动匹配 |


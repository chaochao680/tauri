# Phase 12: OHOS Menu Bar 手动测试

> 状态: 🔄 验证中 — 已部署精简版实现，dark theme 暂不支持
> 前置: Phase 10 (单窗 Menu Bar 基础渲染) + Phase 11 (能力拉齐: auto-refresh/hover/accelerator/per-window visibility)
> 工期: 1-2 天

---

## 1. 目标

为 Phase 10-11 实现的 OHOS Menu Bar 功能编写 **手动测试按钮**，放在 Tests 视图 (TestRunner.svelte) 的 Manual Tests 区域下，作为一个 **Menu Bar 子区域**。每个按钮有独立 handler，用 `wrapManual` 包装，结果通过 `manualResult` 展示 + `console.log` 捕获到 `console-log.txt` 供远程拉取分析。

---

## 2. 设计原则

1. **每个测试自包含** — 每个 handler 动态创建所需菜单，不依赖默认菜单或其他测试的状态
2. **不依赖默认菜单条** — Menu::default 在启动时设置，但测试过程中会被替换
3. **提供 Restore Default 按钮** — 让用户可以恢复到 File/Edit/Window/Help 默认菜单条
4. **分离 Hide/Show** — 不用 toggle（状态依赖），改为独立按钮 + 新增 Rust 命令
5. **覆盖 Phase 10+11 全部手动测试项** — 不遗漏 bar_level_icon、accelerator_ctrl_c、popup_per_window 等

---

## 3. 现状分析

### 3.1 已有测试覆盖

| 来源 | 覆盖内容 | 缺口 |
|------|---------|------|
| `menu.ts` 65 个 auto test | popup Menu CRUD | 无 menu bar (setAsWindowMenu/hide_menu/show_menu/is_menu_visible) |
| `tray.ts` 21 个 auto test | TrayIcon + tray menu | 无 |
| `menu_plugin.rs` | toggle + popup 命令 | 无独立 hide/show/is_menu_visible 命令 |
| `Menu::default(app)` | 启动时设 File/Edit/Window/Help | 动态替换后无法恢复 |
| TestRunner.svelte Manual Tests | isFocused/monitor/dpi + Tray 3 个 | 无 Menu Bar 手动按钮 |

### 3.2 cfg gate 验证

`#[cfg(all(desktop, not(test)))]` 在 OHOS release 构建 (cargo build --release) 时 `cfg!(desktop)=true, cfg!(test)=false` → 条件满足 → menu_plugin/PopupMenu/Menu::default **全部注册**。`plugin:app-menu|toggle` 命令可用。

---

## 4. 测试项清单 (16 个按钮)

对照 Phase 10 (9项) + Phase 11 (13项) 的所有手动测试，合并去重后为 16 个:

| # | 按钮名 | handler | 验证内容 | 来源 |
|---|--------|---------|---------|------|
| M1 | Restore Default Menu | `manualMenuBarRestore` | 恢复 File/Edit/Window/Help 默认菜单条 | 新增(基础设施) |
| M2 | MenuBar Visible | `manualMenuBarVisible` | 菜单条可见 (File/Edit/Window/Help) | Phase 10 |
| M3 | MenuBar Dropdown | `manualMenuBarDropdown` | 点击 bar-level 项出现下拉 | Phase 10 |
| M4 | MenuBar Nested Submenu | `manualMenuBarNested` | 嵌套 submenu (Sub→SubSub→Item) | Phase 10 |
| M5 | MenuBar Hover Highlight | `manualMenuBarHover` | 悬浮 bar-level 项背景色变化 | Phase 11 §5.1 |
| M6 | MenuBar Dark Mode | `manualMenuBarDarkMode` | ⚠️ 暂不支持: OHOS desktop 不传播 colorMode | Phase 11 §7.3 |
| M7 | MenuBar Bar-Level Icon | `manualMenuBarBarIcon` | bar-level 项显示 icon (Submenu 带 icon) | Phase 11 §6.1 |
| M8 | MenuBar Disabled Item | `manualMenuBarDisabledItem` | disabled 项灰化+半透明 | Phase 11 §7.5 |
| M9 | MenuBar Hide | `manualMenuBarHide` | hide_menu → 菜单条消失 | Phase 11 §3.1 |
| M10 | MenuBar Show | `manualMenuBarShow` | show_menu → 菜单条恢复 | Phase 11 §3.1 |
| M11 | MenuBar is_menu_visible | `manualMenuBarIsMenuVisible` | 查询 is_menu_visible 状态 | Phase 11 §3.1 |
| M12 | MenuBar Remove Menu | `manualMenuBarRemove` | remove_menu → 菜单条消失 | Phase 10 |
| M13 | MenuBar Auto Refresh Text | `manualMenuBarAutoRefreshText` | setText 后文字自动更新 | Phase 11 §4.1 |
| M14 | MenuBar Auto Refresh Checked | `manualMenuBarAutoRefreshChecked` | setChecked 后勾选状态更新 | Phase 11 §4.1 |
| M15 | MenuBar Accelerator Ctrl+O | `manualMenuBarAccelerator` | Ctrl+O 触发菜单项 click | Phase 11 §9.1 |
| M16 | MenuBar Accelerator Ctrl+C | `manualMenuBarAcceleratorCopy` | Ctrl+C 触发 predefined copy | Phase 11 §9.1 |
| M17 | MenuBar Fullscreen | `manualMenuBarFullscreen` | fullscreen → 菜单条隐藏 → recover → 恢复 | Phase 11 §11 |
| M18 | MenuBar Predefined Hide | `manualMenuBarPredefinedHide` | Hide → 窗口最小化 | Phase 11 §10.2 |
| M19 | MenuBar Popup Regression | `manualMenuBarPopupRegression` | 右键 popup 正常 (AppStorage key 重命名回归) | Phase 11 回归 |

---

## 5. Rust 端变更

### 5.1 menu_plugin.rs 新增 3 个命令

当前 menu_plugin 只有 `toggle` 和 `popup`。需要新增独立 hide/show/is_menu_visible:

```rust
#[cfg(not(target_os = "macos"))]
#[command]
pub fn hide_menu<R: tauri::Runtime>(window: tauri::Window<R>) -> crate::Result<()> {
    window.hide_menu()
}

#[cfg(not(target_os = "macos"))]
#[command]
pub fn show_menu<R: tauri::Runtime>(window: tauri::Window<R>) -> crate::Result<()> {
    window.show_menu()
}

#[cfg(not(target_os = "macos"))]
#[command]
pub fn is_menu_visible<R: tauri::Runtime>(window: tauri::Window<R>) -> bool {
    window.is_menu_visible().unwrap_or(true)
}
```

注册到 invoke_handler:

```rust
.invoke_handler(tauri::generate_handler![
    #![plugin(app_menu)]
    popup,
    toggle,
    hide_menu,        // 新增
    show_menu,        // 新增
    is_menu_visible,  // 新增
])
```

### 5.2 macOS 分支

macOS 的 hide/show/is_menu_visible 走 AppHandle (全局菜单)，不在本次 OHOS 测试范围内。`#[cfg(not(target_os = "macos"))]` gate 确保只在非 macOS 注册。

---

## 6. Handler 详细设计

所有 handler 用 `wrapManual` 包装，遵循 TestRunner.svelte 现有模式:
- `manualResult = '...'` + `onMessage(manualResult)` 展示
- `console.log('[MenuBarTest] ...')` 自动捕获到 console-log.txt
- 动态创建菜单 → `menu.setAsWindowMenu()` 设为窗口菜单条

### M1: Restore Default Menu

恢复一个与 Menu::default 近似的菜单条 (File/Edit/Window/Help):

```typescript
async function manualMenuBarRestore() {
  await wrapManual('menuBarRestore', async () => {
    const { Menu, Submenu, PredefinedMenuItem } = await import('@tauri-apps/api/menu');
    const fileSub = await Submenu.new({ text: 'File', items: [
      await PredefinedMenuItem.new({ item: 'CloseWindow' }),
      await PredefinedMenuItem.new({ item: 'Quit' }),
    ]});
    const editSub = await Submenu.new({ text: 'Edit', items: [
      await PredefinedMenuItem.new({ item: 'Undo' }),
      await PredefinedMenuItem.new({ item: 'Redo' }),
      await PredefinedMenuItem.new({ item: 'Separator' }),
      await PredefinedMenuItem.new({ item: 'Cut' }),
      await PredefinedMenuItem.new({ item: 'Copy' }),
      await PredefinedMenuItem.new({ item: 'Paste' }),
      await PredefinedMenuItem.new({ item: 'SelectAll' }),
    ]});
    const windowSub = await Submenu.new({ text: 'Window', items: [
      await PredefinedMenuItem.new({ item: 'Minimize' }),
      await PredefinedMenuItem.new({ item: 'Maximize' }),
      await PredefinedMenuItem.new({ item: 'CloseWindow' }),
    ]});
    const helpSub = await Submenu.new({ text: 'Help', items: [
      await PredefinedMenuItem.new({ item: 'About', text: `About Tauri API Validation` }),
    ]});
    const menu = await Menu.new({ items: [fileSub, editSub, windowSub, helpSub] });
    await menu.setAsWindowMenu();
    manualResult = 'Default menu restored: File | Edit | Window | Help';
    onMessage(manualResult);
  });
}
```

### M2: MenuBar Visible

观察当前菜单条是否可见:

```typescript
async function manualMenuBarVisible() {
  await wrapManual('menuBarVisible', async () => {
    manualResult = 'Check: Top of window should show a menu bar with submenu labels.\nIf visible → PASS. If missing → FAIL.\nTip: Click "Restore Default Menu" first if menu bar is missing.';
    onMessage(manualResult);
  });
}
```

### M3: MenuBar Dropdown

创建简单菜单并点击观察下拉:

```typescript
async function manualMenuBarDropdown() {
  await wrapManual('menuBarDropdown', async () => {
    const { Menu, Submenu, MenuItem } = await import('@tauri-apps/api/menu');
    const sub = await Submenu.new({ text: 'Click Me', items: [
      await MenuItem.new({ text: 'Item A' }),
      await MenuItem.new({ text: 'Item B' }),
    ]});
    const menu = await Menu.new({ items: [sub] });
    await menu.setAsWindowMenu();
    manualResult = 'Menu bar: "Click Me" submenu.\nClick "Click Me" → dropdown should appear with "Item A" and "Item B".\nIf dropdown appears → PASS.';
    onMessage(manualResult);
  });
}
```

### M4: MenuBar Nested Submenu

测试嵌套 submenu:

```typescript
async function manualMenuBarNested() {
  await wrapManual('menuBarNested', async () => {
    const { Menu, Submenu, MenuItem } = await import('@tauri-apps/api/menu');
    const inner = await Submenu.new({ text: 'Inner', items: [
      await MenuItem.new({ text: 'Deep Item' }),
    ]});
    const outer = await Submenu.new({ text: 'Outer', items: [
      await MenuItem.new({ text: 'Top Item' }),
      inner,
    ]});
    const menu = await Menu.new({ items: [outer] });
    await menu.setAsWindowMenu();
    manualResult = 'Menu bar: "Outer → Top Item + Inner → Deep Item".\nClick Outer → hover Inner → should show nested dropdown with "Deep Item".\nIf nested submenu works → PASS.';
    onMessage(manualResult);
  });
}
```

### M5: MenuBar Hover Highlight

创建菜单观察 hover 效果:

```typescript
async function manualMenuBarHover() {
  await wrapManual('menuBarHover', async () => {
    const { Menu, Submenu, MenuItem } = await import('@tauri-apps/api/menu');
    const sub = await Submenu.new({ text: 'HoverTest', items: [
      await MenuItem.new({ text: 'Item' }),
    ]});
    const menu = await Menu.new({ items: [sub] });
    await menu.setAsWindowMenu();
    manualResult = 'Menu bar: "HoverTest".\nHover mouse over "HoverTest" → background should change color.\nMove away → background returns to normal.\nIf hover effect visible → PASS.';
    onMessage(manualResult);
  });
}
```

### M6: MenuBar Dark Mode ⚠️ 暂不支持

OHOS desktop 设备不通过 Configuration/ResourceManager API 传播系统 colorMode 变化。已尝试三种方案均无效:
- `context.config.colorMode` → 返回 -1 (NOT_SET)
- `resourceManager.getConfigurationSync().colorMode` → 静默失败
- `onConfigurationUpdate` → 不触发

Menubar 颜色目前固定为浅色模式 (hardcoded)。此测试项暂时跳过。

```typescript
async function manualMenuBarDarkMode() {
  await wrapManual('menuBarDarkMode', async () => {
    manualResult = '⚠️ Dark mode is NOT supported on OHOS desktop currently.\nOHOS desktop does not propagate colorMode through Configuration or ResourceManager APIs.\nMenu bar colors are fixed to light mode (hardcoded).\nThis test is SKIPPED — will be revisited when OHOS SDK provides colorMode support.';
    onMessage(manualResult);
  });
}
```

### M7: MenuBar Bar-Level Icon

测试 bar-level 项显示 icon (Submenu 带 icon):

```typescript
async function manualMenuBarBarIcon() {
  await wrapManual('menuBarBarIcon', async () => {
    const { Menu, Submenu, MenuItem } = await import('@tauri-apps/api/menu');
    const TEST_ICON = 'data:image/png;base64,iVBORW0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==';
    const sub = await Submenu.new({ text: 'IconMenu', icon: TEST_ICON, items: [
      await MenuItem.new({ text: 'Item' }),
    ]});
    const menu = await Menu.new({ items: [sub] });
    await menu.setAsWindowMenu();
    manualResult = 'Menu bar: "IconMenu" submenu WITH icon.\nBar-level "IconMenu" should show a small icon next to the text.\nIf icon visible at bar level → PASS. If only text → FAIL.';
    onMessage(manualResult);
  });
}
```

### M8: MenuBar Disabled Item

创建 disabled + normal 对比:

```typescript
async function manualMenuBarDisabledItem() {
  await wrapManual('menuBarDisabledItem', async () => {
    const { Menu, Submenu, MenuItem } = await import('@tauri-apps/api/menu');
    const sub = await Submenu.new({ text: 'DisTest', items: [
      await MenuItem.new({ text: 'Disabled', enabled: false }),
      await MenuItem.new({ text: 'Normal', enabled: true }),
    ]});
    const menu = await Menu.new({ items: [sub] });
    await menu.setAsWindowMenu();
    manualResult = 'Menu bar: "DisTest → Disabled + Normal".\nClick DisTest → "Disabled" should appear grayed out + semi-transparent.\n"Normal" should appear with full color.\nIf disabled visual correct → PASS.';
    onMessage(manualResult);
  });
}
```

### M9: MenuBar Hide

独立 hide_menu 按钮 (不依赖 toggle):

```typescript
async function manualMenuBarHide() {
  await wrapManual('menuBarHide', async () => {
    await invoke('plugin:app-menu|hide_menu');
    manualResult = 'hide_menu() called.\nCheck: Menu bar should disappear from top of window.\nIf disappeared → PASS.\nClick "Show" button to restore.';
    onMessage(manualResult);
  });
}
```

### M10: MenuBar Show

独立 show_menu 按钮:

```typescript
async function manualMenuBarShow() {
  await wrapManual('menuBarShow', async () => {
    // 先确保菜单条有内容 (如果之前 remove 过)
    await manualMenuBarRestore();
    await invoke('plugin:app-menu|show_menu');
    manualResult = 'show_menu() called (default menu restored + show).\nCheck: Menu bar should reappear at top of window.\nIf visible → PASS.';
    onMessage(manualResult);
  });
}
```

### M11: MenuBar is_menu_visible

查询当前状态:

```typescript
async function manualMenuBarIsMenuVisible() {
  await wrapManual('menuBarIsMenuVisible', async () => {
    const visible = await invoke('plugin:app-menu|is_menu_visible');
    manualResult = `is_menu_visible() = ${visible}\nExpected: true (menu bar should be visible by default).\nIf true → PASS.\nTip: Click "Hide" first, then click this button again → should return false.`;
    onMessage(manualResult);
  });
}
```

### M12: MenuBar Remove Menu

remove_menu → 菜单条清空消失:

```typescript
async function manualMenuBarRemove() {
  await wrapManual('menuBarRemove', async () => {
    const { Menu } = await import('@tauri-apps/api/menu');
    const emptyMenu = await Menu.new({ items: [] });
    await emptyMenu.setAsWindowMenu();
    manualResult = 'Empty menu set as window menu (equivalent to remove_menu).\nCheck: Menu bar should disappear (no items to show).\nIf disappeared → PASS.\nClick "Restore Default Menu" to restore.';
    onMessage(manualResult);
  });
}
```

> 注: `Window::remove_menu()` 在 Rust 端调用 `set_menu_json("[]", window_id)` 清空。
> JS 端没有直接的 `removeMenu()` API，所以用空菜单 `setAsWindowMenu()` 等效。
> 如果 `setAsWindowMenu([])` 不触发 menubar 渲染清空，需要新增 Rust 命令 `remove_menu`。

### M13: MenuBar Auto Refresh Text ✅ 已修复

根因: ForEach 闭包捕获 stale `item.submenuItems`，改用 `getActiveDropdownItem()?.submenuItems` 从最新状态读取。同时在 `onMenubarJsonChange` 中 close/reopen dropdown 强制 re-render。

setText 后菜单条自动更新:

```typescript
async function manualMenuBarAutoRefreshText() {
  await wrapManual('menuBarAutoRefreshText', async () => {
    const { Menu, Submenu, MenuItem } = await import('@tauri-apps/api/menu');
    const item = await MenuItem.new({ text: 'Original' });
    const sub = await Submenu.new({ text: 'Refresh', items: [item] });
    const menu = await Menu.new({ items: [sub] });
    await menu.setAsWindowMenu();
    await new Promise(r => setTimeout(r, 500));
    await item.setText('Updated!');
    manualResult = 'Menu bar: "Refresh → Original".\nsetText("Updated!") called → auto_refresh should push new JSON.\nClick "Refresh" dropdown → should show "Updated!" (not "Original").\nIf text updated → PASS.';
    onMessage(manualResult);
  });
}
```

### M14: MenuBar Auto Refresh Checked

setChecked 后勾选状态更新:

```typescript
async function manualMenuBarAutoRefreshChecked() {
  await wrapManual('menuBarAutoRefreshChecked', async () => {
    const { Menu, Submenu, CheckMenuItem } = await import('@tauri-apps/api/menu');
    const check = await CheckMenuItem.new({ text: 'Check Me', checked: false });
    const sub = await Submenu.new({ text: 'Refresh', items: [check] });
    const menu = await Menu.new({ items: [sub] });
    await menu.setAsWindowMenu();
    await new Promise(r => setTimeout(r, 500));
    await check.setChecked(true);
    manualResult = 'Menu bar: "Refresh → Check Me" (unchecked).\nsetChecked(true) called → auto_refresh should update.\nClick "Refresh" dropdown → "Check Me" should show a checkmark.\nIf checked state updated → PASS.';
    onMessage(manualResult);
  });
}
```

### M15: MenuBar Accelerator Ctrl+O

注册 Ctrl+O 快捷键，通过 action callback 验证触发:

```typescript
async function manualMenuBarAccelerator() {
  await wrapManual('menuBarAccelerator', async () => {
    const { Menu, Submenu, MenuItem } = await import('@tauri-apps/api/menu');
    const item = await MenuItem.new({
      text: 'Accel Test',
      action: (id) => {
        console.log('[MenuBarTest] Accelerator fired! id:', id);
        manualResult = `Accelerator Ctrl+O FIRED! id=${id}`;
        onMessage(manualResult);
      }
    });
    await item.setAccelerator('Ctrl+O');
    const sub = await Submenu.new({ text: 'Accel', items: [item] });
    const menu = await Menu.new({ items: [sub] });
    await menu.setAsWindowMenu();
    manualResult = 'Menu bar: "Accel → Accel Test" (Ctrl+O).\nPress Ctrl+O → should trigger action callback → show "FIRED" message.\nAlso try clicking "Accel Test" in dropdown → should also fire.\nIf Ctrl+O triggers → PASS.';
    onMessage(manualResult);
  });
}
```

### M16: MenuBar Accelerator Ctrl+C

测试 predefined Copy 的 Ctrl+C 快捷键:

```typescript
async function manualMenuBarAcceleratorCopy() {
  await wrapManual('menuBarAcceleratorCopy', async () => {
    const { Menu, Submenu, PredefinedMenuItem } = await import('@tauri-apps/api/menu');
    const copyItem = await PredefinedMenuItem.new({ item: 'Copy' });
    const sub = await Submenu.new({ text: 'Edit', items: [copyItem] });
    const menu = await Menu.new({ items: [sub] });
    await menu.setAsWindowMenu();
    manualResult = 'Menu bar: "Edit → Copy" (Ctrl+C built-in).\nType some text in the webview → select it → press Ctrl+C.\nThen try pasting somewhere → should paste the copied text.\nIf Ctrl+C copies → PASS.';
    onMessage(manualResult);
  });
}
```

> 注: PredefinedMenuItem.Copy 在 OHOS 内部有 accelerator Ctrl+C (通过 AcceleratorMatcher 匹配)。
> PredefinedMenuItem 没有 `setAccelerator()` 方法，accelerator 是内置的。

### M17: MenuBar Fullscreen ✅ 已修复

实现: `windowRectChange` RECOVER 检测 + Escape 键恢复 + recover 菜单项。退出全屏时 menubar 自动恢复 (已 hilog 确认)。

fullscreen → 菜单条隐藏 → recover → 恢复:

```typescript
async function manualMenuBarFullscreen() {
  await wrapManual('menuBarFullscreen', async () => {
    const { Menu, Submenu, PredefinedMenuItem } = await import('@tauri-apps/api/menu');
    const fsItem = await PredefinedMenuItem.new({ item: 'Fullscreen' });
    const sub = await Submenu.new({ text: 'View', items: [fsItem] });
    const menu = await Menu.new({ items: [sub] });
    await menu.setAsWindowMenu();
    manualResult = 'Menu bar: "View → Fullscreen".\nClick "View → Fullscreen" → window enters fullscreen, menu bar should disappear.\nPress Esc or click "View → Fullscreen" again → exit fullscreen, menu bar should recover.\nIf menu bar hides in fullscreen and recovers → PASS.';
    onMessage(manualResult);
  });
}
```

### M18: MenuBar Predefined Hide

Hide → 窗口最小化:

```typescript
async function manualMenuBarPredefinedHide() {
  await wrapManual('menuBarPredefinedHide', async () => {
    const { Menu, Submenu, PredefinedMenuItem } = await import('@tauri-apps/api/menu');
    const hideItem = await PredefinedMenuItem.new({ item: 'Hide' });
    const sub = await Submenu.new({ text: 'Window', items: [hideItem] });
    const menu = await Menu.new({ items: [sub] });
    await menu.setAsWindowMenu();
    manualResult = 'Menu bar: "Window → Hide".\nClick "Window → Hide" → window should minimize.\nRestore window from taskbar → confirm window reappears.\nIf window minimizes on Hide → PASS.';
    onMessage(manualResult);
  });
}
```

### M19: MenuBar Popup Regression

验证 popup 右键菜单仍正常 (AppStorage key 重命名回归):

```typescript
async function manualMenuBarPopupRegression() {
  await wrapManual('menuBarPopupRegression', async () => {
    const { Menu, MenuItem } = await import('@tauri-apps/api/menu');
    const item = await MenuItem.new({ text: 'Popup Test' });
    const menu = await Menu.new({ items: [item] });
    await menu.popup();
    manualResult = 'Popup menu triggered at cursor position.\nCheck: Context menu should appear with "Popup Test".\nThis verifies AppStorage key renaming (::main suffix) did not break popup path.\nIf popup appears → PASS.';
    onMessage(manualResult);
  });
}
```

---

## 7. 前端 UI: TestRunner.svelte

在 Tests 视图的 Manual Tests 区域下，新增 **Menu Bar Tests 子区域**，与现有 General Tests 和 Tray Tests 并列:

```svelte
<!-- General Tests (已有) -->
<div class="flex gap-2 flex-wrap">
  <button class="btn" onclick={manualIsFocused}>isFocused (should be true)</button>
  <button class="btn" onclick={toggleFocusWatch}>...</button>
  <button class="btn" onclick={manualMonitor}>currentMonitor</button>
  <button class="btn" onclick={manualAppCacheDir}>appCacheDir</button>
  <button class="btn" onclick={manualWindowDpi}>Window DPI</button>
</div>

<!-- Tray Manual Tests (已有) -->
<div class="mt-2 pt-2 border-t-1 border-solid border-code">
  <h5 class="my-1 text-xs text-gray-500">Tray Manual Tests</h5>
  <div class="flex gap-2 flex-wrap">
    <button class="btn" onclick={manualTrayIconShow}>...</button>
    <button class="btn" onclick={manualTrayEvent}>...</button>
    <button class="btn" onclick={manualTrayMenu}>...</button>
  </div>
</div>

<!-- Menu Bar Tests (新增) -->
<div class="mt-2 pt-2 border-t-1 border-solid border-code">
  <h5 class="my-1 text-xs text-gray-500">Menu Bar Tests</h5>
  <div class="flex gap-2 flex-wrap">
    <button class="btn" onclick={manualMenuBarRestore}>Restore Default Menu</button>
    <button class="btn" onclick={manualMenuBarVisible}>MenuBar Visible</button>
    <button class="btn" onclick={manualMenuBarDropdown}>MenuBar Dropdown</button>
    <button class="btn" onclick={manualMenuBarNested}>MenuBar Nested Submenu</button>
    <button class="btn" onclick={manualMenuBarHover}>MenuBar Hover</button>
    <button class="btn" onclick={manualMenuBarDarkMode}>MenuBar Dark Mode</button>
    <button class="btn" onclick={manualMenuBarBarIcon}>MenuBar Bar-Level Icon</button>
    <button class="btn" onclick={manualMenuBarDisabledItem}>MenuBar Disabled Item</button>
    <button class="btn" onclick={manualMenuBarHide}>MenuBar Hide</button>
    <button class="btn" onclick={manualMenuBarShow}>MenuBar Show</button>
    <button class="btn" onclick={manualMenuBarIsMenuVisible}>MenuBar is_menu_visible</button>
    <button class="btn" onclick={manualMenuBarRemove}>MenuBar Remove Menu</button>
    <button class="btn" onclick={manualMenuBarAutoRefreshText}>MenuBar Auto Refresh Text</button>
    <button class="btn" onclick={manualMenuBarAutoRefreshChecked}>MenuBar Auto Refresh Checked</button>
    <button class="btn" onclick={manualMenuBarAccelerator}>MenuBar Accelerator Ctrl+O</button>
    <button class="btn" onclick={manualMenuBarAcceleratorCopy}>MenuBar Accelerator Ctrl+C</button>
    <button class="btn" onclick={manualMenuBarFullscreen}>MenuBar Fullscreen</button>
    <button class="btn" onclick={manualMenuBarPredefinedHide}>MenuBar Predefined Hide</button>
    <button class="btn" onclick={manualMenuBarPopupRegression}>MenuBar Popup Regression</button>
  </div>
</div>
```

---

## 8. 变更清单

| 文件 | 变更 |
|------|------|
| `examples/api/src-tauri/src/menu_plugin.rs` | 新增 `hide_menu`/`show_menu`/`is_menu_visible` 3 个命令 + 注册到 invoke_handler |
| `examples/api/src/views/TestRunner.svelte` | 新增 19 个 handler 函数 + Menu Bar Tests 子区域 UI |

无其他文件变更。menu.ts 不改 (手动测试不进 TestCase 数组)。

---

## 9. 测试流程

### 9.1 构建部署

使用 `ohos-build` skill 的 `run-tests.sh` 构建 HAP 并安装到 OHOS 设备。

### 9.2 手动验证步骤

1. 打开 API demo → 自动进入 Tests 视图 → autotest 自动运行
2. 滚动到 Manual Tests → Menu Bar Tests 子区域
3. 先点 **Restore Default Menu** 确认菜单条可见
4. 逐个点击按钮，观察设备上 UI 变化
5. 每次点击后 console.log 自动捕获

### 9.3 Console Log 拉取

```powershell
cmd.exe /c "hdc file recv /data/app/el2/100/base/com.tauri.api/cache/console-log.txt D:\workspace\tauri\tauri\examples\api\console-log.txt"
```

---

## 10. 风险与注意事项

| # | 风险 | 影响 | 对策 |
|---|------|------|------|
| 1 | M12 空菜单 setAsWindowMenu 可能不清空 menubar | 菜单条仍显示旧内容 | 如果 `Menu.new({items:[]})` + setAsWindowMenu 不清空，需新增 Rust `remove_menu` 命令 |
| 2 | accelerator Ctrl+O 可能被 WebView 拦截 | Ctrl+O 事件不到达 onKeyPreIme | 改用 Ctrl+Shift+T 或其他组合 |
| 3 | accelerator 路径依赖 onKeyPreIme 获焦链 | Column 非 focusable 时 onKeyPreIme 可能不触发 | Phase 11 标记为中风险，真机验证 |
| 4 | M18 Hide → minimize() 后窗口消失 | 用户需从任务栏恢复窗口 | manualResult 在点击前已展示，用户先读指令 |
| 5 | 500ms 延迟可能不够 (TSFN 推送) | setAsWindowMenu 后 menubar 未渲染完就 setText | 增加到 1000ms 或观察后手动确认 |
| 6 | PredefinedMenuItem.Copy 的 accelerator 是内置的 | 无法通过 setAccelerator() 外部设置 | Ctrl+C 由 AcceleratorMatcher 内置映射，无需手动注册 |
| 7 | M10 Show 需要先恢复菜单内容 | 如果菜单被 remove 后只 show_menu 不会恢复内容 | Show handler 先调用 Restore 再调用 show_menu |

---

## 11. 不做的事

| # | 项目 | 原因 |
|---|------|------|
| 1 | 添加 auto test 到 menu.ts | 用户要求全部写成手动测试 |
| 2 | 多窗口 menu bar 测试 | OHOS 当前单窗 |
| 3 | popup_per_window AppStorage key 验证 | popup 的 ::main 后缀是内部路由逻辑，手动测试无法直接观察 AppStorage key；M19 popup regression 间接验证 |
| 4 | 性能基准测试 | 非本阶段目标 |

---

## 12. 实施步骤

| Step | 内容 | 文件 |
|------|------|------|
| 1 | menu_plugin.rs: 新增 hide_menu/show_menu/is_menu_visible 3 个命令 + 注册 | `examples/api/src-tauri/src/menu_plugin.rs` |
| 2 | TestRunner.svelte: 新增 19 个 handler + Menu Bar Tests 子区域 UI | `examples/api/src/views/TestRunner.svelte` |
| 3 | HAP 构建+安装+启动 | ohos-build skill |
| 4 | 手动逐项验证 | 设备操作 |
| 5 | 拉取 console-log.txt 分析 | hdc file recv |
| 6 | 更新 phase11-menubar-parity-progress.md 手动测试状态 | doc/menu/impl |

---

## 13. 验证完成标准

| 条件 | 标准 |
|------|------|
| M1 Restore | 默认菜单条恢复 (File/Edit/Window/Help) |
| M2 Visible | 菜单条可见 |
| M3 Dropdown | 点击 bar-level 项出现下拉 |
| M4 Nested | 嵌套 submenu 正常展开 |
| M5 Hover | 悬浮背景色变化 |
| M6 Dark Mode | ⚠️ 暂不支持 (OHOS desktop 不传播 colorMode) — SKIP |
| M7 Bar-Level Icon | bar-level 项显示 icon |
| M8 Disabled | disabled 项灰化+半透明 |
| M9 Hide | hide_menu → 菜单条消失 |
| M10 Show | show_menu → 菜单条恢复 |
| M11 is_menu_visible | 状态查询正确 |
| M12 Remove | remove_menu → 菜单条消失 |
| M13 Auto Refresh Text | setText 后文字更新 — ✅ 已修复验证 |
| M14 Auto Refresh Checked | setChecked 后勾选更新 |
| M15 Accelerator Ctrl+O | 快捷键触发 click |
| M16 Accelerator Ctrl+C | Ctrl+C 复制功能 |
| M17 Fullscreen | fullscreen 隐藏/恢复 — ✅ 已修复验证 |
| M18 Predefined Hide | Hide → 窗口最小化 |
| M19 Popup Regression | 右键 popup 正常 |

全部 18 项通过 (M6 跳过) = Phase 12 完成。
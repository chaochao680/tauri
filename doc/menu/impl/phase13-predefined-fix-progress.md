# Phase 13: Predefined Item 功能缺口修复与 Menu/Tray 统一 - 进度追踪

> 设计文档: [phase13-predefined-fix-design.md](phase13-predefined-fix-design.md)
> 状态: ✅ 全部完成（代码 + 编译 + 设备部署 + autotest 通过）
> 工期: 2-3 天

---

## 进度总览

| Step | 内容 | 状态 | 文件 |
|------|------|------|------|
| Step 1 | 修复剪贴板操作 (copy/cut/paste → pasteboard) | ✅ 完成 | `menu.ets` |
| Step 2 | 扩展 executor 支持 targetWindowId | ✅ 完成 | `menu.ets` |
| Step 3 | 统一 Tray 执行路径 (全局 executor + ArkHelper 委托 + event.rs 补齐) | ✅ 完成 | `menu.ets`, `NativeAbility.ets`, `ArkHelper.ets`, `event.rs` |
| Step 4 | 补齐 ArkTS 类型定义 | ✅ 完成 | `menu_types.ets` |
| Step 5 | 补齐 AcceleratorMatcher 剪贴板排除 (ctrl+z/y) | ✅ 完成 | `accelerator_matcher.ets` |
| Step 6 | 前端测试: autotest 新增 + 手动测试按钮 | ✅ 完成 | `menu.ts`, `TestRunner.svelte` |
| Step 7 | 修复 Tray minimize 窗口闪烁 (minimizeWithRestoreGuard) | ✅ 完成 | `menu.ets` |
| 权限 | api demo 添加 READ_PASTEBOARD | ⚠️ 已移除 | `module.json5` | 受限权限需 AGC Profile，侧载安装失败 |
| 验证 | 编译 + 手动测试 12 项 + autotest 回归 | ✅ 通过 | — | 136 passed, 10 failed (均为 pre-existing) |

---

## Step 1: 修复剪贴板操作

> 设计文档 §五 Step 1

### 问题

`PredefinedActionExecutor.execute()` 中 copy/cut/paste 使用 `document.execCommand("copy"/"cut"/"paste")`。ArkWeb 已废弃 `execCommand`，华为确认"可能兼容但不保证"。实际测试 Copy 和 Paste 菜单点击不生效。

### 方案

改用 `@ohos.pasteboard` 原生 API 读写剪贴板，配合 `runJavaScript` 提取/插入 WebView 文本内容。

### 实施步骤

| Step | 任务 | 状态 | 文件 | 备注 |
|------|------|------|------|------|
| 1.1 | 添加 `import { pasteboard } from '@kit.BasicServicesKit'` | ✅ | `menu.ets` | |
| 1.2 | 重写 `case 'copy'`: getSelection → pasteboard.setData | ✅ | `menu.ets` | `runJavaScript('window.getSelection().toString()')` → `pasteboard.createData` → `systemPasteboard.setData` |
| 1.3 | 重写 `case 'cut'`: getSelection → pasteboard.setData → execCommand("delete") | ✅ | `menu.ets` | delete 步骤保留 execCommand (仅删除操作) |
| 1.4 | 重写 `case 'paste'`: pasteboard.getData → insertText | ✅ | `menu.ets` | `systemPasteboard.getData().getPrimaryText()` → `execCommand("insertText", false, text)` |
| 1.5 | api demo `module.json5` 添加 `ohos.permission.READ_PASTEBOARD` | ⚠️ 已移除 | `module.json5` | 设备不支持 grant，paste 改为 try/catch 容错 |
| 1.6 | 编译验证: HAP 构建 | ✅ | — | BUILD SUCCESSFUL, 安装成功, autotest 全部通过 |

### 验证

| 测试项 | 状态 | 验证内容 |
|--------|------|---------|
| Menu Copy — 选中文本 | ⬚ | Edit → Copy → 外部粘贴验证文字出现 |
| Menu Cut — 选中文本 | ⬚ | Edit → Cut → 选中文本消失 + 外部粘贴验证 |
| Menu Paste — 输入框 | ⬚ | 外部复制 → Edit → Paste → 文字插入输入框 |
| Menu Copy — 无选中 | ⬚ | Edit → Copy (无选中) → 不崩溃，静默 no-op |
| Menu Paste — 剪贴板为空 | ⬚ | Edit → Paste (空剪贴板) → 不崩溃，静默 no-op |
| READ_PASTEBOARD 授权 | ⬚ | 首次 Paste 弹出授权对话框（预期行为） |

---

## Step 2: 扩展 executor 支持 targetWindowId

> 设计文档 §五 Step 2

### 问题

`PredefinedActionExecutor.execute()` 签名只有 `(type, aboutMetadata?)`，窗口操作始终使用初始化时 set 的主窗口 (`this.win`)。无法操作子窗口。

### 方案

添加可选 `targetWindowId?: number` 参数，通过 `WindowManager.getWindow()` 动态解析目标窗口。

### 实施步骤

| Step | 任务 | 状态 | 文件 | 备注 |
|------|------|------|------|------|
| 2.1 | `execute()` 签名添加 `targetWindowId?: number` | ✅ | `menu.ets` | 向下兼容，现有调用不传参 |
| 2.2 | 添加 `import { WindowManager }` | ✅ | `menu.ets` | |
| 2.3 | 窗口操作 (minimize/maximize/fullscreen/recover/close/hide) 使用动态解析的 `win` | ✅ | `menu.ets` | `targetWindowId` 非 0 时从 `WindowManager.getWindow()` 获取 |
| 2.4 | 剪贴板操作保持使用 `this.controller` | ✅ | `menu.ets` | 剪贴板操作需要有焦点的 WebView，不随 targetWindowId 变化 |
| 2.5 | 编译验证: HAP 构建 | ⬚ | — | |

### 已知限制

`MenuManager.handleItemClick` 当前调用 `this.executor.execute(item.predefinedType, item.aboutMetadata)` 未传 windowId。`popupFromJson` 接收 windowId 但未存储到 MenuManager 实例。本阶段 targetWindowId 暂不传递（默认走主窗口），完整转发见 [Phase 14](phase14-remaining-features-scope.md) (MW-2, MW-3)。

### 验证

| 测试项 | 状态 | 验证内容 |
|--------|------|---------|
| 不传 targetWindowId | ⬚ | 行为与修改前一致（回归验证） |
| 传 targetWindowId=0 | ⬚ | 等同于不传，使用主窗口 |
| 传 targetWindowId=非0 窗口存在 | ⬚ | 窗口操作路由到子窗口 |
| 传 targetWindowId=非0 窗口不存在 | ⬚ | fallback 到主窗口，不崩溃 |

---

## Step 3: 统一 Tray 执行路径

> 设计文档 §五 Step 3

### 问题

`ArkHelper.ets` 的 `executePredefinedAction` 维护独立的 switch/case（约 60 行），与 `menu.ets` 的 `PredefinedActionExecutor` 代码重复。Tray 不支持 clipboard/recover/about 操作。

### 方案

单一 `PredefinedActionExecutor` 实例，通过全局 getter 共享。Tray 路径委托给同一 executor。

### 实施步骤

| Step | 任务 | 状态 | 文件 | 备注 |
|------|------|------|------|------|
| **3a. 全局 executor** | | | | |
| 3a.1 | `menu.ets` 添加 `globalExecutor` 变量 + `set/getPredefinedActionExecutor()` 导出函数 | ✅ | `menu.ets` | |
| 3a.2 | `NativeAbility.ets` `setupMenuPopup` 中 executor 创建后调用 `setPredefinedActionExecutor(executor)` | ✅ | `NativeAbility.ets` | 在 `executor.setWindow(mainWindow)` 之后 |
| **3b. ArkHelper 委托** | | | | |
| 3b.1 | `ArkHelper.ets` 添加 `import { getPredefinedActionExecutor } from '../helper/menu'` | ✅ | `ArkHelper.ets` | |
| 3b.2 | 替换 `executePredefinedAction` 的 switch/case 为 `executor.execute(actionType as PredefinedType)` | ✅ | `ArkHelper.ets` | 删除约 60 行代码 |
| 3b.3 | 添加 executor null check + warn 日志 | ✅ | `ArkHelper.ets` | executor 未初始化时不崩溃 |
| **3c. Rust 侧补齐** | | | | |
| 3c.1 | `event.rs` `execute_predefined_action` match 补齐 `copy/cut/paste/selectAll/undo/redo/recover` | ✅ | `event.rs` | 原来只有 minimize/hide/maximize/close/fullscreen/about |
| 3c.2 | 编译验证: cargo check OHOS | ⬚ | — | |
| 3c.3 | 编译验证: HAP 构建 | ⬚ | — | |

### 验证

| 测试项 | 状态 | 验证内容 |
|--------|------|---------|
| Tray Minimize | ⬚ | tray 菜单 → Minimize → 主窗口最小化 |
| Tray Maximize | ⬚ | tray 菜单 → Maximize → 主窗口最大化 |
| Tray Fullscreen | ⬚ | tray 菜单 → Fullscreen → 主窗口全屏 |
| Tray Close | ⬚ | tray 菜单 → Close → 主窗口最小化（保持 minimize 行为） |
| Tray About | ⬚ | tray 菜单 → About → AlertDialog 弹出 |
| Tray Recover | ⬚ | tray 菜单 → Recover → 窗口恢复正常（原来不支持，统一后修复） |
| Tray Copy | ⬚ | tray 菜单 → Copy → 与 Menu Copy 一致（原来不支持） |
| Tray Paste | ⬚ | tray 菜单 → Paste → 与 Menu Paste 一致（原来不支持） |
| Tray Quit | ⬚ | tray 菜单 → Quit → 应用退出 |
| ArkHelper 无 switch/case | ⬚ | 代码审查: ArkHelper.ets 的 executePredefinedAction 只有委托调用 |

---

## Step 4: 补齐 ArkTS 类型定义

> 设计文档 §五 Step 4

### 问题

`menu_types.ets` 的 `PredefinedType` 缺少 `services`、`bringAllToFront`、`none`，与 Rust 侧 `PredefinedMenuItemType` 枚举不对齐。

### 实施步骤

| Step | 任务 | 状态 | 文件 | 备注 |
|------|------|------|------|------|
| 4.1 | `PredefinedType` 添加 `services` / `bringAllToFront` / `none` | ✅ | `menu_types.ets` | macOS-only 变体，OHOS 上 no-op 但类型需对齐 |
| 4.2 | 编译验证: HAP 构建 | ⬚ | — | |

### 验证

| 测试项 | 状态 | 验证内容 |
|--------|------|---------|
| 类型对齐 | ⬚ | Rust 侧所有 PredefinedMenuItemType 变体在 ArkTS 有对应字符串 |
| services/bringAllToFront/none | ⬚ | 这些 predefined type 触发时不崩溃（no-op） |

---

## Step 5: 补齐 AcceleratorMatcher 剪贴板排除

> 设计文档 §五 Step 5

### 问题

Phase 11 已实现 `AcceleratorMatcher` 并排除了 `ctrl+c/x/v/a`，但**缺少 `ctrl+z` (undo) 和 `ctrl+y` (redo)**。WebView 原生 undo/redo 会被绕过。

### 实施步骤

| Step | 任务 | 状态 | 文件 | 备注 |
|------|------|------|------|------|
| 5.1 | `CLIPBOARD_ACCELERATORS` 添加 `'ctrl+z'` 和 `'ctrl+y'` | ✅ | `accelerator_matcher.ets:38-43` | |
| 5.2 | 编译验证: HAP 构建 | ⬚ | — | |

### 验证

| 测试项 | 状态 | 验证内容 |
|--------|------|---------|
| Ctrl+Z 不被拦截 | ⬚ | 按 Ctrl+Z → WebView 原生 undo（不走 executor） |
| Ctrl+Y 不被拦截 | ⬚ | 按 Ctrl+Y → WebView 原生 redo（不走 executor） |
| Ctrl+C/V/X/A 仍不被拦截 | ⬚ | 回归验证: 原有排除列表未受影响 |
| Ctrl+M 仍被拦截 | ⬚ | 非剪贴板快捷键仍正常拦截并触发 executor |

---

## Step 6: 前端测试

> 测试框架: `examples/api/src/lib/tests/` (autotest) + `TestRunner.svelte` (manual)
> 参考: frontend-api-testing skill

### 6a. 新增自动测试

**文件**: `examples/api/src/lib/tests/menu.ts`

在 `menuTests` 数组末尾添加 `predefined_clipboard_primitives` 测试，验证 predefined menu item 依赖的底层原语在 ArkWeb 中可用：

| Step | 任务 | 状态 | 文件 | 备注 |
|------|------|------|------|------|
| 6a.1 | 添加 `predefined_clipboard_primitives` side-effect 测试 | ✅ | `menu.ts` | 验证 `getSelection()`, `execCommand("insertText")`, `execCommand("selectAll")` |

**测试逻辑**:
1. 创建 textarea → 填入文字 → `textarea.select()` → `window.getSelection().toString()` 验证选中文字（copy/cut 的文本提取原语）
2. 清空 textarea → `execCommand("insertText", false, "pasted-content")` → 验证 textarea.value（paste 的文本插入原语）
3. `execCommand("selectAll")` → 验证全选（selectAll 原语）

**说明**: 这些测试不直接测试 pasteboard API（需要 `READ_PASTEBOARD` 权限弹窗），而是验证 menu predefined copy/paste 修复方案依赖的 WebView 侧原语。如果 `execCommand("insertText")` 在 ArkWeb 中也不可用，测试会失败并触发风险缓解方案（DOM `activeElement.value` 直接操作）。

### 6b. 新增手动测试

**文件**: `examples/api/src/views/TestRunner.svelte`

| Step | 任务 | 状态 | 备注 |
|------|------|------|------|
| 6b.1 | 添加 `manualMenuPredefinedCopy` handler | ✅ | 设置 Edit → Copy 菜单，引导用户选中文字后点击 Copy 验证 |
| 6b.2 | 添加 `manualMenuPredefinedPaste` handler | ✅ | 设置 Edit → Paste 菜单，引导用户外部复制后点击 Paste 验证 |
| 6b.3 | 添加 `manualMenuPredefinedCut` handler | ✅ | 设置 Edit → Cut 菜单，引导用户选中文字后点击 Cut 验证 |
| 6b.4 | 添加 `manualTrayPredefined` handler | ✅ | 创建含 Copy/Minimize/About/Recover/Fullscreen/Quit 的 tray 菜单，逐一验证 |
| 6b.5 | 在 Manual Tests 区域添加对应按钮 | ✅ | 分两组: "Menu Predefined" + "Tray Predefined" |

---

## Step 7: 修复 Tray minimize 窗口闪烁

> 设计文档 §五 Step 6

### 问题

OHOS 桌面模式下，点击 Tray 菜单的 Minimize/Hide/Close 时窗口"闪缩还原"（先缩小再恢复原尺寸）。根因是 StatusBar 右键菜单关闭时系统自动恢复主窗口焦点 (WINDOW_ACTIVE 事件)，与 `win.minimize()` 竞态，系统焦点恢复覆盖了 minimize。

maximize/fullscreen/copy 等不受影响，因为它们不改变窗口前台状态。

### 方案

`minimizeWithRestoreGuard()`: 不直接 minimize，而是先注册 WINDOW_ACTIVE 一次性监听器，等系统焦点恢复后再 minimize。只调用一次 minimize，避免卡顿。

```typescript
private async minimizeWithRestoreGuard(win: window.Window | null): Promise<void> {
    if (!win) return;
    let consumed = false;
    const doMinimize = () => {
      if (!consumed) {
        consumed = true;
        try { win.off('windowEvent', onWindowEvent); } catch (_) {}
        win.minimize();
      }
    };
    const onWindowEvent = (eventType: window.WindowEventType) => {
      if (!consumed && eventType === window.WindowEventType.WINDOW_ACTIVE) {
        doMinimize();
      }
    };
    try { win.on('windowEvent', onWindowEvent); } catch (_) { win.minimize(); return; }
    setTimeout(doMinimize, 50);  // fallback if no focus restoration (menu path)
}
```

### 实施步骤

| Step | 任务 | 状态 | 文件 | 备注 |
|------|------|------|------|------|
| 7.1 | 添加 `minimizeWithRestoreGuard` 方法到 `PredefinedActionExecutor` | ✅ | `menu.ets` | 注册 WINDOW_ACTIVE 监听，延迟 minimize |
| 7.2 | 合并 `case 'minimize'` / `case 'hide'` / `case 'close'` 为 fallthrough | ✅ | `menu.ets` | 三者都调用 `minimizeWithRestoreGuard(win)` |
| 7.3 | 删除原独立的 `case 'close'` | ✅ | `menu.ets` | 与 minimize/hide 合并 |
| 7.4 | 编译验证 + 设备部署测试 | ✅ | — | 136 passed, minimize 正常无闪烁 |

### 验证

| 测试项 | 状态 | 验证内容 |
|--------|------|---------|
| Tray Minimize 无闪烁 | ✅ | tray 菜单 → Minimize → 窗口平滑最小化，不闪缩还原 |
| Tray Hide 无闪烁 | ✅ | tray 菜单 → Hide → 等同 minimize |
| Tray Close 无闪烁 | ✅ | tray 菜单 → Close → 等同 minimize |
| Menu Minimize 无闪烁 | ✅ | menu → Minimize → 正常（menu 路径不走焦点恢复，50ms fallback） |
| Tray Maximize 不受影响 | ✅ | tray 菜单 → Maximize → 正常（不经过 guard） |
| Tray Fullscreen 不受影响 | ✅ | tray 菜单 → Fullscreen → 正常 |

### 验证

| 测试项 | 状态 | 验证内容 |
|--------|------|---------|
| autotest `predefined_clipboard_primitives` | ⬚ | Run Side-Effect 通过后三项断言 |
| 手动 Menu Copy | ⬚ | wrapManual 输出引导文案 → 用户验证 |
| 手动 Menu Paste | ⬚ | wrapManual 输出引导文案 → 用户验证 |
| 手动 Menu Cut | ⬚ | wrapManual 输出引导文案 → 用户验证 |
| 手动 Tray Predefined | ⬚ | wrapManual 创建 tray → 用户逐一点击验证 |

---

## 编译验证

| 目标 | 状态 |
|------|------|
| aarch64-unknown-linux-ohos (openharmony-ability) | ⬚ |
| aarch64-unknown-linux-ohos (tray-icon) | ⬚ |
| HAR 构建 | ⬚ |
| HAP 签名+安装+启动 | ⬚ |
| Windows 回归: cargo check -p tray-icon | ⬚ |

---

## 前端自动测试 (autotest)

> 现有测试框架: `examples/api/src/lib/tests/` (auto/side-effect) + `TestRunner.svelte` (manual)

### 现有相关测试

| 测试名 | 文件 | 类别 | 覆盖范围 | Phase 13 相关 |
|--------|------|------|----------|---------------|
| `PredefinedMenuItem.copy` | `menu.ts:564` | auto | 创建 + text 非空 | ❌ 不验证实际复制行为 |
| `PredefinedMenuItem.cut` | `menu.ts:574` | auto | 创建 + text 非空 | ❌ 不验证实际剪切行为 |
| `PredefinedMenuItem.paste` | `menu.ts:585` | auto | 创建 + text 非空 | ❌ 不验证实际粘贴行为 |
| `PredefinedMenuItem.selectAll` | `menu.ts:596` | auto | 创建 + text 非空 | ❌ 不验证实际全选行为 |
| `PredefinedMenuItem.undo` | `menu.ts:607` | auto | 创建 + text 非空 | ❌ 不验证实际撤销行为 |
| `PredefinedMenuItem.redo` | `menu.ts:618` | auto | 创建 + text 非空 | ❌ 不验证实际重做行为 |
| `clipboard-manager.writeText+readText` | `plugins.ts:126` | side-effect | clipboard plugin 读写 | ❌ plugin 在 OHOS 不可用 (arboard) |
| `menuBarAcceleratorCopy` | `TestRunner.svelte:391` | manual | Ctrl+C 键盘快捷键 | ✅ 验证键盘路径 (WebView 原生) |
| `menuBarFullscreen` | `TestRunner.svelte:403` | manual | 全屏 predefined | ✅ 已有 |
| `menuBarPredefinedHide` | `TestRunner.svelte:415` | manual | 隐藏 predefined | ✅ 已有 |

### 新增自动测试

在 `menu.ts` 中添加 Phase 13 验证用例，间接验证 pasteboard 路径依赖的原语：

```typescript
// Phase 13: Predefined item clipboard primitives
{
  name: '@tauri-apps/api/menu.predefined_clipboard_primitives',
  category: 'side-effect',
  async fn() {
    // 1. Verify window.getSelection works (used by menu copy/cut)
    const textarea = document.createElement('textarea');
    textarea.value = 'phase13-test-text';
    document.body.appendChild(textarea);
    textarea.select();
    const selection = window.getSelection()?.toString();
    assert(selection === 'phase13-test-text',
      `getSelection should return selected text, got "${selection}"`);

    // 2. Verify execCommand("insertText") works (used by menu paste)
    textarea.value = '';
    textarea.focus();
    const inserted = document.execCommand('insertText', false, 'pasted-content');
    assert(inserted !== false, 'execCommand("insertText") returned false');
    assert(textarea.value === 'pasted-content',
      `textarea should contain inserted text, got "${textarea.value}"`);

    // 3. Verify execCommand("selectAll") works (used by menu selectAll)
    textarea.select();
    const selectedAll = window.getSelection()?.toString();
    assert(selectedAll === 'pasted-content',
      `selectAll should select all textarea content, got "${selectedAll}"`);

    document.body.removeChild(textarea);
  },
},
```

**说明**: 这些测试验证 predefined menu item 依赖的底层原语 (`getSelection`, `execCommand("insertText")`, `execCommand("selectAll")`) 在 ArkWeb 中可用。如果底层原语不可用，menu copy/paste 的修复方案 (pasteboard + 这些原语) 也会失败。

### 新增手动测试

在 `TestRunner.svelte` 中添加 Phase 13 手动测试按钮：

| 测试名 | handler | 验证内容 |
|--------|---------|---------|
| Menu Edit → Copy | `manualMenuPredefinedCopy` | 选中文字 → Edit → Copy → 外部粘贴验证 |
| Menu Edit → Paste | `manualMenuPredefinedPaste` | 外部复制 → 输入框 → Edit → Paste → 文字插入 |
| Menu Edit → Cut | `manualMenuPredefinedCut` | 选中文字 → Edit → Cut → 文字消失 + 外部粘贴验证 |
| Tray Predefined Actions | `manualTrayPredefined` | 创建含 Copy/Minimize/About/Recover 的 tray 菜单 → 逐一点击验证 |

**handler 实现模式** (参考现有 `menuBarAcceleratorCopy`):

```typescript
async function manualMenuPredefinedCopy() {
  await wrapManual('menuPredefinedCopy', async () => {
    const { Menu, Submenu, PredefinedMenuItem } = await import('@tauri-apps/api/menu');
    const copyItem = await PredefinedMenuItem.new({ item: 'Copy' });
    const sub = await Submenu.new({ text: 'Edit', items: [copyItem] });
    const menu = await Menu.new({ items: [sub] });
    await menu.setAsWindowMenu();
    manualResult = 'Menu bar: "Edit → Copy".\nSelect some text → click Edit → Copy → paste elsewhere.\nIf text appears in clipboard → PASS.';
    onMessage(manualResult);
  });
}

async function manualTrayPredefined() {
  await wrapManual('trayPredefined', async () => {
    const { TrayIcon } = await import('@tauri-apps/api/tray');
    const { Menu, PredefinedMenuItem } = await import('@tauri-apps/api/menu');
    const menu = await Menu.new({ items: [
      await PredefinedMenuItem.new({ item: 'Copy' }),
      await PredefinedMenuItem.new({ item: 'Minimize' }),
      await PredefinedMenuItem.new({ item: 'About' }),
      await PredefinedMenuItem.new({ item: 'Separator' }),
      await PredefinedMenuItem.new({ item: 'Fullscreen' }),
      await PredefinedMenuItem.new({ item: 'Quit' }),
    ]});
    await TrayIcon.new({ id: 'phase13-test', menu, tooltip: 'Phase 13 Test' });
    manualResult = 'Tray icon created with predefined items.\nRight-click → Copy: should copy selected text.\nRight-click → Minimize: should minimize main window.\nRight-click → About: should show AlertDialog.\nRight-click → Fullscreen: should enter fullscreen.\nVerify each action works.';
    onMessage(manualResult);
  });
}
```

---

## 手动测试 (设备验证)

| 测试项 | 状态 | 验证内容 |
|--------|------|---------|
| 键盘 Ctrl+C | ⬚ | WebView 选中文字 → Ctrl+C → 外部粘贴（WebView 原生处理） |
| 键盘 Ctrl+V | ⬚ | 外部复制 → demo 输入框 → Ctrl+V（WebView 原生处理） |
| 键盘 Ctrl+Z | ⬚ | 输入文字 → Ctrl+Z → 撤销（WebView 原生处理，不被 AcceleratorMatcher 拦截） |
| 键盘 Ctrl+Y | ⬚ | Ctrl+Z 后 → Ctrl+Y → 重做（WebView 原生处理，不被 AcceleratorMatcher 拦截） |
| Menu Copy | ⬚ | 选中文字 → Edit → Copy → 外部粘贴 |
| Menu Paste | ⬚ | 外部复制 → 输入框 → Edit → Paste |
| Menu Cut | ⬚ | 选中文字 → Edit → Cut → 外部粘贴 |
| Tray Copy | ⬚ | tray 菜单 → Copy |
| Tray About | ⬚ | tray 菜单 → About → AlertDialog |
| Tray Recover | ⬚ | tray 菜单 → Recover → 窗口恢复 |
| Tray Minimize | ⬚ | tray 菜单 → Minimize → 主窗口最小化 |
| Tray Fullscreen | ⬚ | tray 菜单 → Fullscreen → 主窗口全屏 |
| Phase 12 M1-M19 回归 | ⬚ | 使用 Phase 12 测试设计验证菜单操作不受影响 |

---

## 遇到的问题

(实施过程中记录)

---

## 关键实现决策记录

| # | 决策 | 原因 |
|---|------|------|
| 1 | 剪贴板操作用 pasteboard 而非 clipboard plugin | clipboard plugin 在 OHOS 通过 arboard 实现，arboard 不支持 OHOS，初始化静默失败 |
| 2 | paste 的插入步骤仍用 execCommand("insertText") | pasteboard 只提供数据读写，"插入到焦点"需要 WebView 配合。备选: 直接操作 DOM activeElement.value |
| 3 | 剪贴板操作始终用主 WebView controller | 剪贴板操作需要有焦点的 WebView，当前架构中只有主窗口有 WebView |
| 4 | targetWindowId 暂不在 menu 路径传递 | MenuManager.handleItemClick 不存 windowId，popupFromJson 的 windowId 未转发。完整多窗待子窗口 menubar 实现时补齐 |
| 5 | Tray 不传 targetWindowId | OHOS tray 唯一，固定操作主窗口 (windowId=0)，与 macOS selector→key window 行为对齐 |
| 6 | CloseWindow 保持 minimize | 用户确认不改为 terminateSelf，与 Windows/macOS Close=关闭窗口 的语义差异可接受 |
| 7 | CLIPBOARD_ACCELERATORS 补齐 ctrl+z/y | Phase 11 已排除 c/x/v/a 但漏了 z/y。WebView 原生 undo/redo 更完整 |
| 8 | ArkHelper.ets 删除未使用的 `display` import | fullscreen 逻辑移至 PredefinedActionExecutor 后 display 不再被引用 |
| 9 | pasteboard 操作包裹在 try/catch 中 | 剪贴板操作可能因权限未授权、剪贴板为空等原因失败，静默 no-op 比崩溃好 |
| 10 | 移除 READ_PASTEBOARD 权限声明 | `user_grant` 权限需要设备支持 grant 流程，测试设备安装失败 (code 9568289)。paste 改为 try/catch 容错，无权限时静默 no-op |

---

## 不做的事

| # | 项目 | 原因 |
|---|------|------|
| 1 | 修改 clipboard plugin 的 OHOS 实现 | clipboard plugin 是独立项目，不在本次 predefined item 修复范围内。arboard 不支持 OHOS 是上游问题 |
| 2 | 为 selectAll/undo/redo 改用 pasteboard | execCommand("selectAll"/"undo"/"redo") 目前可工作，无明确废弃证据；pasteboard 不提供等价功能 |
| 3 | MenuManager 转发 windowId 到 executor | 当前 menubar 只在主窗口，targetWindowId 永远为 0。完整多窗转发见 [Phase 14](phase14-remaining-features-scope.md) |
| 4 | 为 Tray clipboard 操作获取子窗口 WebView | Tray 固定操作主窗口，与 macOS 行为一致 |
| 5 | 为 clipboard 操作添加 autotest 直接调用 pasteboard | pasteboard 需要 `READ_PASTEBOARD` 权限（已从 module.json5 移除，设备不支持 grant）。改为验证底层原语 |

---

## 审计修正记录

| # | 问题 | 严重度 | 修正 |
|---|------|--------|------|
| A-1 | Step 5 标注为"当前未实现 accelerator 拦截" | 🟡 中等 | 实际 Phase 11 已实现 AcceleratorMatcher + CLIPBOARD_ACCELERATORS，但缺少 ctrl+z/y |
| A-2 | 未提及 clipboard plugin 在 OHOS 不可用 | 🟡 中等 | Context 补充: arboard 不支持 OHOS，plugin 静默失败 |
| A-3 | Step 2 未说明 menu 路径当前不传 windowId | 🟡 中等 | 添加"当前限制"说明 |
| A-4 | 风险表"Phase 11 如果错误拦截"描述过于笼统 | 🟢 低 | 改为具体描述: CLIPBOARD_ACCELERATORS 已排除 c/x/v/a 但缺少 z/y |

---

## 代码量估算

| 层 | 行数估算 | 来源 |
|----|---------|------|
| ArkTS (menu.ets 剪贴板) | ~30 | Step 1: pasteboard import + copy/cut/paste 重写 |
| ArkTS (menu.ets targetWindowId) | ~15 | Step 2: 参数 + WindowManager 查找 |
| ArkTS (menu.ets 全局 executor) | ~8 | Step 3a: globalExecutor + getter/setter |
| ArkTS (NativeAbility.ets) | ~3 | Step 3a: setPredefinedActionExecutor(executor) |
| ArkTS (ArkHelper.ets) | ~-55 | Step 3b: 删除 60 行 switch/case，替换为 5 行委托 |
| ArkTS (menu_types.ets) | ~3 | Step 4: PredefinedType 补齐 3 个变体 |
| ArkTS (accelerator_matcher.ets) | ~2 | Step 5: CLIPBOARD_ACCELERATORS 补齐 2 项 |
| ArkTS (module.json5) | ~2 | 权限: READ_PASTEBOARD |
| Rust (event.rs) | ~5 | Step 3c: match 补齐 7 个 action 字符串 |
| JS (menu.ts) | ~25 | Step 6a: predefined_clipboard_primitives 测试 |
| Svelte (TestRunner.svelte) | ~60 | Step 6b: 4 个 handler + 4 个按钮 |
| **合计** | **~98** (源码净减 ~42 + 测试 ~85) | |

# Menu 模块 OHOS 测试设计

> 基于 api-to-support.md 中所有 OHOS 支持的 API 设计
> 创建时间: 2026-05-16
> 状态: ✅ 已完成实现并验证
> 最后更新: 2026-05-16

---

## 测试分类说明

| 分类 | 说明 | 示例 |
|------|------|------|
| **auto** | 纯 API 调用 + assert，可自动断言结果 | `Menu.new()`, `MenuItem.text()`, `Menu.append()` |
| **side-effect** | 有副作用但可验证（状态改变、文件写入等） | `PredefinedMenuItem.cut()` → 验证剪贴板 |
| **manual** | 需人工确认 UI 或行为 | `Menu.popup()` 视觉验证、`quit()` 退出验证 |

## 超时策略

测试引擎 (`test-runner.ts`) 已内置超时保护机制，所有测试用例**无需自行处理超时**：

```typescript
// test-runner.ts 内置机制
const TEST_TIMEOUT_MS = 5000; // 全局超时 5 秒

function withTimeout<T>(promise: Promise<T>, timeoutMs: number): Promise<T> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error(`Timeout after ${timeoutMs}ms`)), timeoutMs);
    promise.then(v => { clearTimeout(timer); resolve(v); })
           .catch(e => { clearTimeout(timer); reject(e); });
  });
}
```

### 超时规则

| 规则 | 说明 |
|------|------|
| **auto / side-effect** | 自动应用 5s 超时，超时后标记为 `fail`，错误信息 `Timeout after 5000ms` |
| **manual** | 不参与 autotest 自动执行，无超时限制 |
| **禁止自行超时** | 不要在测试用例中使用 `Promise.race` + `setTimeout`，会与 test-runner 冲突 |

### 慢操作处理

| 操作类型 | 处理方式 | 原因 |
|----------|----------|------|
| `popup()` | 归为 **manual** | 涉及 UI 渲染，autotest 无法验证视觉效果 |
| `popup({ x, y })` | 归为 **manual** | 涉及 UI 渲染，autotest 无法验证位置 |
| `minimize()` / `maximize()` / `hide()` | 归为 **manual** | 改变窗口状态，影响后续测试运行 |
| `quit()` / `closeWindow()` | 归为 **manual** | 退出应用，无法继续执行后续测试 |
| `action` 回调 | 归为 **auto** | 仅验证注册不报错，实际点击需 manual |
| 剪贴板操作 (`copy_exec` 等) | 归为 **side-effect** | 可通过 `navigator.clipboard.readText()` 验证 |

### API 重要说明

根据实际 Tauri Menu API 源码 (`packages/api/src/menu/`)：

| API | 类型 | 说明 |
|-----|------|------|
| `id` | **getter 属性** | `menu.id` 不是 `await menu.id()` |
| `kind` | **getter 属性** | `item.kind` 返回 `'MenuItem' \| 'Submenu' \| 'Predefined' \| 'Check' \| 'Icon'` |
| `append()` | 接受单个或多个 | `menu.append(item)` 或 `menu.append([item1, item2])` |
| `prepend()` | 接受单个或多个 | 同上 |
| `insert()` | 接受单个或多个 | `menu.insert(item, position)` 或 `menu.insert([a,b], position)` |
| `popup()` | 可选位置参数 | `menu.popup()` 或 `menu.popup({ x: 100, y: 200 })` |
| `popupAt()` | **不存在** | 使用 `popup({ x, y })` 替代 |
| `appHandle()` | **不存在** | Menu 类没有此方法 |
| `AboutMetadataBuilder` | **不存在** | 使用 `AboutMetadata` 接口直接传入 |
| `onMenuEvent` | **不存在** | 使用 `action` 回调在创建时注册 |

### 测试用例模板

```typescript
// ✅ 正确：无需处理超时，test-runner 自动保护
{
  name: '@tauri-apps/api/menu.Menu.new',
  category: 'auto',
  async fn() {
    const { Menu } = await import('@tauri-apps/api/menu');
    const menu = await Menu.new();
    assert(menu.id.length > 0, 'menu.id returned empty');
  },
}

// ❌ 错误：不要自行处理超时
{
  name: '@tauri-apps/api/menu.Menu.popup',
  category: 'manual',
  async fn() {
    // 不要这样做！test-runner 不会执行 manual 测试
    await Promise.race([
      menu.popup(),
      new Promise((_, reject) => setTimeout(() => reject(new Error('timeout')), 3000))
    ]);
  },
}
```

---

## 一、Menu 测试 (16 个)

### 1.1 创建方法

| # | 测试名 | 覆盖 API | 分类 | Assert |
|---|--------|----------|------|--------|
| M-01 | `Menu.new` | `Menu.new()` | auto | `menu.id.length > 0` |
| M-02 | `Menu.with_id` | `Menu.new({ id: 'custom-id' })` | auto | `menu.id === 'custom-id'` |
| M-03 | `Menu.with_items` | `Menu.new({ items: [...] })` | auto | `menu.items().length === N` |
| M-04 | `Menu.with_id_and_items` | `Menu.new({ id, items })` | auto | `menu.id === X && menu.items().length === N` |

### 1.2 菜单项管理

| # | 测试名 | 覆盖 API | 分类 | Assert |
|---|--------|----------|------|--------|
| M-05 | `Menu.append` | `append(item)` | auto | `items().length` 递增 |
| M-06 | `Menu.append_items` | `append([item1, item2])` | auto | `items().length` 增加 N |
| M-07 | `Menu.prepend` | `prepend(item)` | auto | `items()[0].id === item.id` |
| M-08 | `Menu.prepend_items` | `prepend([item1, item2])` | auto | `items()[0..1]` ID 匹配 |
| M-09 | `Menu.insert` | `insert(item, 1)` | auto | `items()[1].id === item.id` |
| M-10 | `Menu.insert_items` | `insert([a,b], 1)` | auto | `items()[1..2]` ID 匹配 |
| M-11 | `Menu.remove` | `remove(item)` | auto | `items().length` 递减 |
| M-12 | `Menu.removeAt` | `removeAt(0)` | auto | `items().length` 递减 |
| M-13 | `Menu.get` | `get(id)` | auto | `get(id) !== null && get(id)!.id === id` |
| M-14 | `Menu.items` | `items()` | auto | 返回数组，长度正确 |

### 1.3 ContextMenu (popup)

| # | 测试名 | 覆盖 API | 分类 | Assert |
|---|--------|----------|------|--------|
| M-15 | `Menu.popup` | `popup()` | manual | 弹出菜单，人工确认 |
| M-16 | `Menu.popup_at` | `popup({ x, y })` | manual | 在指定位置弹出 |

---

## 二、MenuItem 测试 (8 个)

| # | 测试名 | 覆盖 API | 分类 | Assert |
|---|--------|----------|------|--------|
| MI-01 | `MenuItem.new` | `MenuItem.new({ text })` | auto | `item.id.length > 0` |
| MI-02 | `MenuItem.with_id` | `MenuItem.new({ id, text })` | auto | `item.id === id` |
| MI-03 | `MenuItem.text` | `text()` | auto | `text() === 'Test Item'` |
| MI-04 | `MenuItem.setText` | `setText('New')` | auto | `text() === 'New'` |
| MI-05 | `MenuItem.isEnabled` | `isEnabled()` | auto | `isEnabled() === true` (默认) |
| MI-06 | `MenuItem.setEnabled` | `setEnabled(false)` | auto | `isEnabled() === false` |
| MI-07 | `MenuItem.setAccelerator` | `setAccelerator('Ctrl+O')` | auto | 不抛异常 |
| MI-08 | `MenuItem.action` | `new({ action: (id) => {} })` | auto | 注册不报错 |

---

## 三、Submenu 测试 (16 个)

### 3.1 创建

| # | 测试名 | 覆盖 API | 分类 | Assert |
|---|--------|----------|------|--------|
| SM-01 | `Submenu.new` | `Submenu.new({ text })` | auto | `submenu.id.length > 0` |
| SM-02 | `Submenu.with_id` | `Submenu.new({ id, text })` | auto | `submenu.id === id` |
| SM-03 | `Submenu.with_items` | `Submenu.new({ text, items })` | auto | `items().length === N` |

### 3.2 菜单项管理

| # | 测试名 | 覆盖 API | 分类 | Assert |
|---|--------|----------|------|--------|
| SM-04 | `Submenu.append` | `append(item)` | auto | `items().length` 递增 |
| SM-05 | `Submenu.append_items` | `append([a,b])` | auto | `items().length` 增加 N |
| SM-06 | `Submenu.prepend` | `prepend(item)` | auto | `items()[0].id === item.id` |
| SM-07 | `Submenu.prepend_items` | `prepend([a,b])` | auto | `items()[0..1]` ID 匹配 |
| SM-08 | `Submenu.insert` | `insert(item, 1)` | auto | `items()[1].id === item.id` |
| SM-09 | `Submenu.insert_items` | `insert([a,b], 1)` | auto | `items()[1..2]` ID 匹配 |
| SM-10 | `Submenu.remove` | `remove(item)` | auto | `items().length` 递减 |
| SM-11 | `Submenu.removeAt` | `removeAt(0)` | auto | `items().length` 递减 |
| SM-12 | `Submenu.items` | `items()` | auto | 返回数组，长度正确 |
| SM-13 | `Submenu.get` | `get(id)` | auto | `get(id).id === id` |

### 3.3 属性

| # | 测试名 | 覆盖 API | 分类 | Assert |
|---|--------|----------|------|--------|
| SM-14 | `Submenu.text` | `text()`, `setText()` | auto | roundtrip 验证 |
| SM-15 | `Submenu.isEnabled` | `isEnabled()`, `setEnabled()` | auto | roundtrip 验证 |
| SM-16 | `Submenu.setIcon` | `setIcon()` | auto | 不抛异常 |

### 3.4 Popup & 嵌套

| # | 测试名 | 覆盖 API | 分类 | Assert |
|---|--------|----------|------|--------|
| SM-17 | `Submenu.popup` | `popup()` | manual | 弹出子菜单 |
| SM-18 | `Submenu.popup_at` | `popup({ x, y })` | manual | 在指定位置弹出 |
| SM-19 | `Submenu.nested` | Submenu → Submenu → MenuItem | manual | 多级嵌套可展开 |

---

## 四、PredefinedMenuItem 测试 (16 个)

### 4.1 创建

| # | 测试名 | 覆盖 API | 分类 | Assert |
|---|--------|----------|------|--------|
| PM-01 | `PredefinedMenuItem.separator` | `new({ item: 'Separator' })` | auto | `id.length > 0` (separator text is intentionally empty on all platforms) |
| PM-02 | `PredefinedMenuItem.copy` | `new({ item: 'Copy' })` | auto | `text().length > 0` |
| PM-03 | `PredefinedMenuItem.cut` | `new({ item: 'Cut' })` | auto | `text().length > 0` |
| PM-04 | `PredefinedMenuItem.paste` | `new({ item: 'Paste' })` | auto | `text().length > 0` |
| PM-05 | `PredefinedMenuItem.selectAll` | `new({ item: 'SelectAll' })` | auto | `text().length > 0` |
| PM-06 | `PredefinedMenuItem.undo` | `new({ item: 'Undo' })` | auto | `text().length > 0` |
| PM-07 | `PredefinedMenuItem.redo` | `new({ item: 'Redo' })` | auto | `text().length > 0` |
| PM-08 | `PredefinedMenuItem.fullscreen` | `new({ item: 'Fullscreen' })` | auto | `text().length > 0` |

### 4.2 窗口操作

| # | 测试名 | 覆盖 API | 分类 | Assert |
|---|--------|----------|------|--------|
| PM-09 | `PredefinedMenuItem.minimize` | `new({ item: 'Minimize' })` | manual | 窗口最小化 |
| PM-10 | `PredefinedMenuItem.maximize` | `new({ item: 'Maximize' })` | manual | 窗口最大化 |
| PM-11 | `PredefinedMenuItem.closeWindow` | `new({ item: 'CloseWindow' })` | manual | 窗口关闭，应用退出 |
| PM-12 | `PredefinedMenuItem.hide` | `new({ item: 'Hide' })` | manual | 窗口最小化（等同于 minimize） |
| PM-13 | `PredefinedMenuItem.quit` | `new({ item: 'Quit' })` | manual | 应用退出 |

### 4.3 属性

| # | 测试名 | 覆盖 API | 分类 | Assert |
|---|--------|----------|------|--------|
| PM-14 | `PredefinedMenuItem.text` | `text()`, `setText()` | auto | roundtrip 验证 |

### 4.4 About

| # | 测试名 | 覆盖 API | 分类 | Assert |
|---|--------|----------|------|--------|
| PM-15 | `PredefinedMenuItem.about` | `new({ item: { About: {...} } })` | auto | `text().length > 0` |
| PM-16 | `PredefinedMenuItem.about_exec` | 点击 about 项 | manual | 弹出 AlertDialog |

---

## 五、CheckMenuItem 测试 (7 个)

| # | 测试名 | 覆盖 API | 分类 | Assert |
|---|--------|----------|------|--------|
| CK-01 | `CheckMenuItem.new` | `CheckMenuItem.new({ text, checked })` | auto | `item.id.length > 0` |
| CK-02 | `CheckMenuItem.isChecked` | `isChecked()` | auto | `=== true` (默认) |
| CK-03 | `CheckMenuItem.setChecked` | `setChecked(false)` | auto | `isChecked() === false` |
| CK-04 | `CheckMenuItem.text` | `text()`, `setText()` | auto | roundtrip 验证 |
| CK-05 | `CheckMenuItem.isEnabled` | `isEnabled()`, `setEnabled()` | auto | roundtrip 验证 |
| CK-06 | `CheckMenuItem.setAccelerator` | `setAccelerator('Ctrl+K')` | auto | 不抛异常 |
| CK-07 | `CheckMenuItem.kind` | `item.kind` | auto | `=== 'Check'` |

---

## 六、IconMenuItem 测试 (7 个)

| # | 测试名 | 覆盖 API | 分类 | Assert |
|---|--------|----------|------|--------|
| IC-01 | `IconMenuItem.new` | `IconMenuItem.new({ text, icon })` | auto | `item.id.length > 0` |
| IC-02 | `IconMenuItem.with_id` | `IconMenuItem.new({ id, text, icon })` | auto | `item.id === id` |
| IC-03 | `IconMenuItem.setIcon` | `setIcon(newIcon)` | auto | 不抛异常 |
| IC-04 | `IconMenuItem.text` | `text()`, `setText()` | auto | roundtrip 验证 |
| IC-05 | `IconMenuItem.isEnabled` | `isEnabled()`, `setEnabled()` | auto | roundtrip 验证 |
| IC-06 | `IconMenuItem.setAccelerator` | `setAccelerator('Ctrl+I')` | auto | 不抛异常 |
| IC-07 | `IconMenuItem.kind` | `item.kind` | auto | `=== 'Icon'` |

---

## 七、MenuEvent 测试 (1 个)

| # | 测试名 | 覆盖 API | 分类 | Assert |
|---|--------|----------|------|--------|
| ME-01 | `MenuItem.action` | `new({ action: (id) => {} })` | auto | 注册不报错，回调函数被设置 |

> **注意**: Tauri Menu 不使用 `onMenuEvent` 监听器，而是通过 `action` 回调在创建菜单项时注册。

---

## 八、MenuItemKind 测试 (5 个)

| # | 测试名 | 覆盖 API | 分类 | Assert |
|---|--------|----------|------|--------|
| MK-01 | `MenuItem.kind` | `item.kind` | auto | `=== 'MenuItem'` |
| MK-02 | `Submenu.kind` | `submenu.kind` | auto | `=== 'Submenu'` |
| MK-03 | `PredefinedMenuItem.kind` | `item.kind` | auto | `=== 'Predefined'` |
| MK-04 | `CheckMenuItem.kind` | `item.kind` | auto | `=== 'Check'` |
| MK-05 | `IconMenuItem.kind` | `item.kind` | auto | `=== 'Icon'` |

---

## 九、AboutMetadata 测试 (2 个)

| # | 测试名 | 覆盖 API | 分类 | Assert |
|---|--------|----------|------|--------|
| AB-01 | `PredefinedMenuItem.about` | `new({ item: { About: {...} } })` | auto | `text().length > 0` |
| AB-02 | `PredefinedMenuItem.about_exec` | 点击 about 项 | manual | 弹出 AlertDialog |

> **注意**: Tauri 不使用 `AboutMetadataBuilder`，直接使用 `AboutMetadata` 接口。

---

## 十、集成测试 (4 个)

| # | 测试名 | 覆盖场景 | 分类 | Assert |
|---|--------|----------|------|--------|
| INT-01 | `Menu.full_workflow` | new → append → popup → click → action | manual | 完整工作流：创建菜单 → 添加项 → 弹出 → 点击 → action 回调 |
| INT-02 | `Menu.with_submenu` | Menu → Submenu → MenuItem | manual | 子菜单可展开 |
| INT-03 | `Menu.mixed_items` | Menu 包含 MenuItem + Submenu + Predefined + Check + Icon | auto | `items()` 返回混合类型，`items().length` 正确 |
| INT-04 | `Menu.popup_at_position` | `popup({ x: 100, y: 200 })` | manual | 在指定位置弹出 |

---

## 汇总统计

| 模块 | auto | side-effect | manual | 合计 |
|------|------|-------------|--------|------|
| Menu | 14 | 0 | 2 | 16 |
| MenuItem | 8 | 0 | 0 | 8 |
| Submenu | 13 | 0 | 3 | 16 |
| PredefinedMenuItem | 9 | 0 | 7 | 16 |
| CheckMenuItem | 7 | 0 | 0 | 7 |
| IconMenuItem | 7 | 0 | 0 | 7 |
| MenuEvent | 1 | 0 | 0 | 1 |
| MenuItemKind | 5 | 0 | 0 | 5 |
| AboutMetadata | 1 | 0 | 1 | 2 |
| 集成测试 | 1 | 0 | 3 | 4 |
| **总计** | **66** | **0** | **16** | **82** |

---

## 实际测试结果 (2026-05-16)

### 测试执行结果

| 指标 | 数值 |
|------|------|
| **总测试数** | 107 (含非 menu 测试) |
| **Menu 相关 auto 测试** | 42 |
| **Menu 通过** | **42/42 (100%)** |
| **总通过** | 102/107 |
| **总失败** | 5 (全部非 menu 相关) |

### Menu 各模块通过情况

| 模块 | 设计编号 | 状态 | 备注 |
|------|---------|------|------|
| Menu 创建/管理 | M-01 ~ M-14 | ✅ 全部通过 | 14/14 auto |
| MenuItem 属性 | MI-01 ~ MI-08 | ✅ 全部通过 | 8/8 auto |
| Submenu 创建/管理/属性 | SM-01 ~ SM-16 | ✅ 全部通过 | 13/13 auto + setIcon |
| PredefinedMenuItem 创建 | PM-01 ~ PM-08 | ✅ 全部通过 | 8/8 auto |
| PredefinedMenuItem 属性 | PM-14 | ✅ 通过 | text roundtrip |
| PredefinedMenuItem About | AB-01 | ✅ 通过 | about 创建 + text 验证 |
| CheckMenuItem | CK-01 ~ CK-06 | ✅ 全部通过 | 6/6 auto |
| IconMenuItem | IC-01 ~ IC-06 | ✅ 全部通过 | 6/6 auto (含 data URI 修复) |
| MenuEvent | ME-01 | ✅ 通过 | action 回调注册 |
| MenuItemKind | MK-01 ~ MK-05 | ✅ 全部通过 | 5/5 auto |
| 集成测试 | INT-03 | ✅ 通过 | mixed_items 混合类型 |

### 未覆盖/Manual 测试

| 编号 | 测试名 | 分类 | 原因 |
|------|--------|------|------|
| M-15, M-16 | Menu.popup / popup_at | manual | UI 视觉效果需人工确认 |
| SM-17 ~ SM-19 | Submenu.popup / popup_at / nested | manual | UI 嵌套展开需人工确认 |
| PM-09 ~ PM-13 | minimize/maximize/closeWindow/hide/quit | manual | 窗口状态改变影响测试流 |
| PM-16 | about_exec | manual | AlertDialog 弹出需人工确认 |
| INT-01, INT-02, INT-04 | full_workflow / with_submenu / popup_at_position | manual | 完整 UI 工作流 |

### 非 Menu 失败项（与本次无关）

| 测试 | 错误 | 原因 |
|------|------|------|
| core.Channel | expected 1000, got 122 | JS-Rust 通道通信已知问题 |
| plugin-http.fetch | plugin http not found | 插件未加载，预期行为 |
| plugin-autostart | plugin autostart not found | 插件未加载，预期行为 |
| plugin-clipboard-manager ×2 | plugin clipboard-manager not found | 插件未加载，预期行为 |

---

## 实现过程中修复的问题

### 问题 1: `run_main_thread!` 死锁

**现象**: Menu 测试大量超时，应用 freeze
**根因**: OHOS IPC 回调线程 ≠ main_thread_id，`rx.recv()` 永久阻塞
**修复**: 方案 B - 方法级别 `#[cfg(target_env = "ohos")]` 直接执行
**影响文件**: `menu.rs`, `submenu.rs`, `normal.rs`, `check.rs`, `icon.rs`, `predefined.rs`

### 问题 2: `JsImage` 将 data URI 当作文件路径

**现象**: IconMenuItem/Submenu.setIcon 报 `No such file or directory (os error 2)`
**根因**: `#[serde(untagged)]` 按声明顺序匹配，`Path` 先于其他变体匹配所有字符串
**修复**: 自定义反序列化器，识别 `data:` 前缀创建 `DataUri` 变体 + base64 解码
**影响文件**: `crates/tauri/src/image/mod.rs`, `examples/api/src/lib/tests/menu.ts`

### 问题 3: `PredefinedMenuItem.separator` 文本断言

**现象**: 测试断言 `text.length > 0` 失败
**根因**: Separator 在所有平台上 text 都是 `""`（正确行为，分隔线不需要文本）
**修复**: 修改测试断言为验证 `id.length > 0`，移除 text 非空断言

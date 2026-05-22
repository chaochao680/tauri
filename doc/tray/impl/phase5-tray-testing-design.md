# Phase 5 设计：Tray 模块端到端测试

> 版本：v1.0
> 创建时间：2026-05-17
> 目标：为 tray 模块编写 auto 和 manual 测试用例，验证 OHOS 托盘功能

---

## 一、测试策略

### 1.1 验证方式概览

| 验证方式 | 适用场景 | 工具 | 测试分类 |
|----------|----------|------|----------|
| **auto** | API 返回值、类型、方法存在性可断言 | `frontend-api-testing` skill | `category: 'auto'` |
| **side-effect** | 有副作用但可程序验证（创建/删除托盘） | `frontend-api-testing` skill | `category: 'side-effect'` |
| **manual** | 视觉效果、用户交互、事件触发需人工确认 | `frontend-api-testing` skill | `category: 'manual'` |

### 1.2 auto vs manual 判断原则

| 判断依据 | auto/side-effect | manual |
|----------|-----------------|--------|
| 有返回值可断言？ | ✓ 使用 | - |
| 无查询 API 验证状态？ | - | ✓ 使用 |
| 需用户操作触发？ | - | ✓ 使用 |
| 需人工观察视觉效果？ | - | ✓ 使用 |

### 1.3 OHOS 平台限制对测试的影响

| 限制 | 对测试的影响 | 处理方式 |
|------|-------------|---------|
| 仅 PC/2in1 支持 statusBarManager | 移动设备无托盘 | 测试仅在 desktop 设备运行 |
| 单图标限制 | 无法测试多图标 | 不编写多图标测试 |
| 仅 Click 事件（Left/Right） | DoubleClick/Enter/Move/Leave 不触发 | 不测试这些事件类型 |
| `position` 固定 (0, 0) | 位置断言无意义 | 仅验证事件结构，不验证位置值 |
| `rect` 固定 default | 尺寸断言无意义 | 仅验证事件结构，不验证 rect 值 |
| `button_state` 固定 Up | 状态断言无意义 | 仅验证 button_state === 'Up' |
| `set_title` 不支持 | OHOS 无 updateQuickOperationTitle | 标记为 OHOS skip |
| `set_temp_dir_path` Linux only | OHOS 不支持 | 标记为 OHOS skip |
| `set_icon_as_template` macOS only | OHOS 不支持 | 标记为 OHOS skip |
| `set_show_menu_on_left_click` | OHOS 不支持 | 标记为 OHOS skip |

---

## 二、Auto 测试用例设计

### 2.1 TrayIcon 创建与查询（有意义的断言）

| # | 测试名称 | 断言内容 | Category | 有意义原因 |
|---|---------|---------|----------|-----------|
| T1 | `TrayIcon.new` | `tray.id.length > 0` | auto | 验证 id 非空 |
| T2 | `TrayIcon.new({ id })` | `tray.id === 'custom-id'` | auto | 验证 id 精确匹配 |
| T3 | `TrayIcon.getById(存在)` | `found !== null && found.id === id` | auto | 验证查询返回正确对象 |
| T4 | `TrayIcon.getById(不存在)` | `found === null` | auto | 验证不存在返回 null |
| T5 | `TrayIcon.removeById` | 删除后 `getById` 返回 `null` | side-effect | 验证删除生效 |

### 2.2 TrayIcon 实例方法（无有意义断言）

> 以下方法均返回 `void`，且 OHOS 无对应的 getter API（`getTooltip`/`getMenu`/`isVisible` 等均不存在）。
> 这些测试仅验证"调用不抛异常"，在 OHOS 上大部分是 stub 实现，实际功能验证交给 manual 测试。

| # | 测试名称 | 返回值 | Category | 说明 |
|---|---------|--------|----------|------|
| T6 | `tray.setIcon(dataUri)` | void | auto | 验证不抛异常 |
| T7 | `tray.setIcon(null)` | void | auto | 验证不抛异常 |
| T8 | `tray.setMenu(menu)` | void | auto | 验证不抛异常 |
| T9 | `tray.setMenu(null)` | void | auto | 验证不抛异常 |
| T10 | `tray.setTooltip(text)` | void | auto | 验证不抛异常 |
| T11 | `tray.setTitle(text)` | void | auto | OHOS stub，验证不抛异常 |
| T12 | `tray.setVisible(false)` | void | auto | 验证不抛异常 |
| T13 | `tray.setVisible(true)` | void | auto | 验证不抛异常 |
| T14 | `tray.setTempDirPath(path)` | void | auto | OHOS stub，验证不抛异常 |
| T15 | `tray.setIconAsTemplate(true)` | void | auto | OHOS stub，验证不抛异常 |
| T16 | `tray.setShowMenuOnLeftClick(true)` | void | auto | OHOS stub，验证不抛异常 |

### 2.3 总结

| 类别 | 数量 | 说明 |
|------|------|------|
| 有意义断言 | 5 | id 验证、getById 验证、removeById 验证 |
| 无有意义断言 | 11 | 仅验证不抛异常，OHOS 上为 stub |
| **总计 auto** | **16** | |

---

## 三、Manual 测试用例设计

### 3.1 视觉效果验证

| # | 测试名称 | 操作步骤 | 预期行为 | 验证点 |
|---|---------|---------|---------|--------|
| M1 | 托盘图标显示 | 1. 点击 "Tray Icon Show" 按钮<br>2. 查看屏幕右下角状态栏 | 状态栏右侧出现一个新的托盘图标，图标为默认图标（非空白） | 图标可见、位置正确（状态栏右侧） |
| M2 | 图标更新 | 1. 创建托盘图标后<br>2. 调用 `setIcon` 传入不同的 PNG 图片<br>3. 观察状态栏图标变化 | 状态栏图标立即更新为新图片，无闪烁或延迟 | 图标变化即时生效 |
| M3 | 图标移除 | 1. 创建托盘图标后<br>2. 调用 `setIcon(null)` | 状态栏图标立即消失，不留下空白占位 | 图标完全移除 |
| M4 | 托盘隐藏 | 1. 创建托盘图标后<br>2. 调用 `setVisible(false)` | 状态栏图标消失，效果与 `setIcon(null)` 相同 | 图标不可见 |
| M5 | 托盘显示 | 1. 调用 `setVisible(false)` 隐藏后<br>2. 调用 `setVisible(true)` | 状态栏图标重新出现，与隐藏前一致 | 图标重现 |
| M6 | Tooltip 显示 | 1. 创建托盘图标时设置 `tooltip: "test tooltip"`<br>2. 鼠标悬停在托盘图标上 1-2 秒 | 图标上方出现浮提示文本 "test tooltip"，鼠标移开后消失 | tooltip 文本正确显示 |

### 3.2 菜单验证

| # | 测试名称 | 操作步骤 | 预期行为 | 验证点 |
|---|---------|---------|---------|--------|
| M7 | 右键菜单弹出 | 1. 创建带菜单的托盘图标<br>2. 右键点击托盘图标 | 在图标附近弹出上下文菜单，菜单项与创建时设置的一致 | 菜单弹出位置合理、内容正确 |
| M8 | 菜单项显示 | 1. 右键点击托盘图标弹出菜单<br>2. 查看菜单内容 | 显示所有设置的菜单项（如 "Test Item"），文本正确，无乱码 | 菜单项文本与设置一致 |
| M9 | 菜单更新 | 1. 创建托盘图标并设置初始菜单<br>2. 调用 `setMenu` 更换为新菜单<br>3. 右键点击托盘图标 | 弹出的菜单显示新菜单的内容，旧菜单项不再出现 | 菜单内容已更新 |
| M10 | 菜单项点击 | 1. 右键点击托盘图标弹出菜单<br>2. 点击任意菜单项 | 菜单关闭，同时应用收到菜单点击事件（通过 console log 验证） | 菜单点击后关闭且事件触发 |

### 3.3 事件验证

| # | 测试名称 | 操作步骤 | 预期行为 | 验证点 |
|---|---------|---------|---------|--------|
| M11 | 左键点击事件 | 1. 创建带 `action` 回调的托盘图标<br>2. 左键点击托盘图标<br>3. 查看 console log | 收到事件，`type` 为 `"Click"`，`button` 为 `"Left"`，`buttonState` 为 `"Up"`，`id` 与创建的 tray id 一致 | 事件结构正确，id 匹配 |
| M12 | 右键菜单点击事件 | 1. 创建带 `action` 回调和菜单的托盘图标<br>2. 右键点击托盘图标弹出菜单<br>3. 点击某个菜单项<br>4. 查看 console log | 收到事件，`type` 为 `"Click"`，`button` 为 `"Right"`，`buttonState` 为 `"Up"`，`id` 格式为 `"tray_id:menu_code"` | 事件包含 menu_code 信息 |
| M13 | 事件 id 正确性 | 1. 查看 M11 或 M12 收到的事件数据 | `id` 字段与创建托盘时指定的 id 一致（或包含该 id） | id 字段正确 |
| M14 | 事件 position | 1. 查看 M11 或 M12 收到的事件数据 | `position.x` 为 `0`，`position.y` 为 `0`（OHOS 不提供点击位置） | position 为 (0, 0) |
| M15 | 事件 rect | 1. 查看 M11 或 M12 收到的事件数据 | `rect.position` 和 `rect.size` 均为默认值（OHOS 不提供图标区域信息） | rect 为 default |

### 3.4 集成验证

| # | 测试名称 | 操作步骤 | 预期行为 | 验证点 |
|---|---------|---------|---------|--------|
| M16 | 菜单项点击触发应用逻辑 | 1. 使用 example app 的 tray 功能<br>2. 右键点击托盘图标<br>3. 点击 "Quit" 菜单项 | 应用完全退出，窗口关闭，进程终止 | 应用退出 |
| M17 | 菜单项点击触发窗口操作 | 1. 使用 example app 的 tray 功能<br>2. 右键点击托盘图标<br>3. 点击 "Toggle" 菜单项 | 主窗口在显示/隐藏之间切换，菜单项文本在 "Show"/"Hide" 之间变化 | 窗口状态切换 |
| M18 | 多次创建/删除 | 1. 循环执行创建托盘 → 操作 → 删除托盘 5 次以上<br>2. 观察应用是否崩溃或内存泄漏 | 每次创建和删除都正常完成，无崩溃，状态栏图标正确显示/消失 | 无崩溃、无泄漏 |

---

## 四、测试文件结构

### 4.1 新增文件

```
examples/api/src/
└── lib/tests/
    └── tray.ts          # 新增：tray 测试用例
```

### 4.2 修改文件

```
examples/api/src/views/TestRunner.svelte
├── 导入 trayTests
├── 添加到 allTests 数组
├── 添加 tray manual test handlers
└── 添加 tray manual test buttons

examples/api/src-tauri/src/lib.rs
└── 确认 OHOS tray 初始化路径已存在（Phase 4 已完成）

examples/api/src-tauri/Cargo.toml
└── 确认 tray-icon feature 已启用（Phase 4 已完成）
```

---

## 五、Auto 测试用例详细设计

### 5.1 有意义的断言测试（5 个）

```typescript
// T1: TrayIcon.new
{
  name: '@tauri-apps/api/tray.TrayIcon.new',
  category: 'auto',
  async fn() {
    const { TrayIcon } = await import('@tauri-apps/api/tray');
    const tray = await TrayIcon.new();
    assert(tray !== undefined, 'TrayIcon.new returned undefined');
    assert(tray.id.length > 0, `tray.id returned empty: ${tray.id}`);
    tray.close();
  },
}

// T2: TrayIcon.new({ id })
{
  name: '@tauri-apps/api/tray.TrayIcon.new_with_id',
  category: 'auto',
  async fn() {
    const { TrayIcon } = await import('@tauri-apps/api/tray');
    const tray = await TrayIcon.new({ id: 'my-custom-tray' });
    assert(tray.id === 'my-custom-tray', `tray.id mismatch: "${tray.id}"`);
    tray.close();
  },
}

// T3: TrayIcon.getById(存在)
{
  name: '@tauri-apps/api/tray.TrayIcon.getById',
  category: 'auto',
  async fn() {
    const { TrayIcon } = await import('@tauri-apps/api/tray');
    const tray = await TrayIcon.new({ id: 'test-get-by-id' });
    const found = await TrayIcon.getById('test-get-by-id');
    assert(found !== null, 'getById returned null for existing tray');
    assert(found.id === 'test-get-by-id', `getById id mismatch: "${found.id}"`);
    tray.close();
  },
}

// T4: TrayIcon.getById(不存在)
{
  name: '@tauri-apps/api/tray.TrayIcon.getById_not_found',
  category: 'auto',
  async fn() {
    const { TrayIcon } = await import('@tauri-apps/api/tray');
    const found = await TrayIcon.getById('non-existent-tray-id');
    assert(found === null, `getById should return null, got ${found}`);
  },
}

// T5: TrayIcon.removeById
{
  name: '@tauri-apps/api/tray.TrayIcon.removeById',
  category: 'side-effect',
  async fn() {
    const { TrayIcon } = await import('@tauri-apps/api/tray');
    const tray = await TrayIcon.new({ id: 'test-remove' });
    await TrayIcon.removeById('test-remove');
    const found = await TrayIcon.getById('test-remove');
    assert(found === null, 'getById should return null after removeById');
  },
}
```

### 5.2 无有意义断言的测试（11 个）

```typescript
// T6: tray.setIcon(dataUri)
{
  name: '@tauri-apps/api/tray.TrayIcon.setIcon',
  category: 'auto',
  async fn() {
    const { TrayIcon } = await import('@tauri-apps/api/tray');
    const tray = await TrayIcon.new();
    const icon = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==';
    await tray.setIcon(icon);
    tray.close();
  },
}

// T7: tray.setIcon(null)
{
  name: '@tauri-apps/api/tray.TrayIcon.setIcon_null',
  category: 'auto',
  async fn() {
    const { TrayIcon } = await import('@tauri-apps/api/tray');
    const tray = await TrayIcon.new();
    await tray.setIcon(null);
    tray.close();
  },
}

// T8: tray.setMenu(menu)
{
  name: '@tauri-apps/api/tray.TrayIcon.setMenu',
  category: 'auto',
  async fn() {
    const { TrayIcon } = await import('@tauri-apps/api/tray');
    const { Menu, MenuItem } = await import('@tauri-apps/api/menu');
    const tray = await TrayIcon.new();
    const item = await MenuItem.new({ text: 'Item' });
    const menu = await Menu.new({ items: [item] });
    await tray.setMenu(menu);
    tray.close();
  },
}

// T9: tray.setMenu(null)
{
  name: '@tauri-apps/api/tray.TrayIcon.setMenu_null',
  category: 'auto',
  async fn() {
    const { TrayIcon } = await import('@tauri-apps/api/tray');
    const tray = await TrayIcon.new();
    await tray.setMenu(null);
    tray.close();
  },
}

// T10: tray.setTooltip(text)
{
  name: '@tauri-apps/api/tray.TrayIcon.setTooltip',
  category: 'auto',
  async fn() {
    const { TrayIcon } = await import('@tauri-apps/api/tray');
    const tray = await TrayIcon.new();
    await tray.setTooltip('test tooltip');
    await tray.setTooltip(null);
    tray.close();
  },
}

// T11: tray.setTitle(text) — OHOS stub
{
  name: '@tauri-apps/api/tray.TrayIcon.setTitle',
  category: 'auto',
  async fn() {
    const { TrayIcon } = await import('@tauri-apps/api/tray');
    const tray = await TrayIcon.new();
    await tray.setTitle('test title');
    await tray.setTitle(null);
    tray.close();
  },
}

// T12: tray.setVisible
{
  name: '@tauri-apps/api/tray.TrayIcon.setVisible',
  category: 'auto',
  async fn() {
    const { TrayIcon } = await import('@tauri-apps/api/tray');
    const tray = await TrayIcon.new();
    await tray.setVisible(false);
    await tray.setVisible(true);
    tray.close();
  },
}

// T13-T16: 平台专属方法 — OHOS stub
{
  name: '@tauri-apps/api/tray.TrayIcon.setTempDirPath',
  category: 'auto',
  async fn() {
    const { TrayIcon } = await import('@tauri-apps/api/tray');
    const tray = await TrayIcon.new();
    await tray.setTempDirPath('/tmp');
    await tray.setTempDirPath(null);
    tray.close();
  },
}

{
  name: '@tauri-apps/api/tray.TrayIcon.setIconAsTemplate',
  category: 'auto',
  async fn() {
    const { TrayIcon } = await import('@tauri-apps/api/tray');
    const tray = await TrayIcon.new();
    await tray.setIconAsTemplate(true);
    await tray.setIconAsTemplate(false);
    tray.close();
  },
}

{
  name: '@tauri-apps/api/tray.TrayIcon.setShowMenuOnLeftClick',
  category: 'auto',
  async fn() {
    const { TrayIcon } = await import('@tauri-apps/api/tray');
    const tray = await TrayIcon.new();
    await tray.setShowMenuOnLeftClick(true);
    await tray.setShowMenuOnLeftClick(false);
    tray.close();
  },
}
```

---

## 六、Manual 测试详细设计

### 6.1 TestRunner.svelte 添加的 handlers

```typescript
// Tray manual test handlers
async function manualTrayIconShow() {
  await wrapManual('trayIconShow', async () => {
    const { TrayIcon } = await import('@tauri-apps/api/tray');
    const tray = await TrayIcon.new({ tooltip: 'Test Tray Icon' });
    manualResult = `Tray icon created with id: "${tray.id}". Check system tray for icon.`;
    onMessage(manualResult);
  });
}

async function manualTrayEvent() {
  await wrapManual('trayEvent', async () => {
    const { TrayIcon } = await import('@tauri-apps/api/tray');
    let lastEvent: string | null = null;
    const tray = await TrayIcon.new({
      tooltip: 'Click me!',
      action: (event) => {
        lastEvent = JSON.stringify(event);
        manualResult = `Tray event received: ${lastEvent}`;
        onMessage(manualResult);
      }
    });
    manualResult = `Tray created with id: "${tray.id}". Left-click or right-click the tray icon to trigger events.`;
    onMessage(manualResult);
  });
}

async function manualTrayMenu() {
  await wrapManual('trayMenu', async () => {
    const { TrayIcon } = await import('@tauri-apps/api/tray');
    const { Menu, MenuItem } = await import('@tauri-apps/api/menu');
    const item = await MenuItem.new({ text: 'Test Menu Item' });
    const menu = await Menu.new({ items: [item] });
    const tray = await TrayIcon.new({ menu, tooltip: 'Right-click me' });
    manualResult = `Tray created with menu. Right-click the tray icon to see the menu.`;
    onMessage(manualResult);
  });
}
```

### 6.2 Manual Tests 区域按钮

```svelte
<div class="flex gap-2 flex-wrap">
  <button class="btn" onclick={manualTrayIconShow}>Tray Icon Show (check system tray)</button>
  <button class="btn" onclick={manualTrayEvent}>Tray Event (click icon to trigger)</button>
  <button class="btn" onclick={manualTrayMenu}>Tray Menu (right-click to see menu)</button>
</div>
```

---

## 七、OHOS 特殊处理

### 7.1 需要 skip 的测试

| 测试 | 原因 | 处理方式 |
|------|------|---------|
| `setTempDirPath` | Linux only | auto 测试保留（OHOS stub 返回 Ok），不验证实际效果 |
| `setIconAsTemplate` | macOS only | auto 测试保留（OHOS stub 返回 Ok），不验证实际效果 |
| `setShowMenuOnLeftClick` | 不支持 | auto 测试保留（OHOS stub 返回 Ok），不验证实际效果 |
| `setTitle` | OHOS 无 updateQuickOperationTitle | auto 测试保留（OHOS stub 返回 Ok），不验证实际效果 |

### 7.2 事件验证注意事项

OHOS 事件限制：
- 仅 `Click` 事件，`DoubleClick`/`Enter`/`Move`/`Leave` 不触发
- `button` 仅 `Left`/`Right`，无 `Middle`
- `buttonState` 固定为 `Up`
- `position` 固定为 `(0, 0)`
- `rect` 固定为 default

Manual 测试中需明确标注这些限制，避免测试人员误判。

---

## 八、测试执行流程

### 8.1 自动测试

```bash
# 使用 ohos-build skill 一键测试
bash .claude/skills/ohos-build/scripts/run-tests.sh "" desktop
```

### 8.2 手动测试

1. 构建并安装到 OHOS 设备
2. 打开应用，进入 Tests 视图
3. 点击 Manual Tests 区域的 tray 相关按钮
4. 按照按钮文案提示操作
5. 查看 console-log.txt 确认事件数据

### 8.3 报告拉取

```bash
# 拉取测试报告
cmd.exe /c "hdc file recv /data/app/el2/100/base/com.tauri.api/cache/test-report.md D:\workspace\tauri\tauri\examples\api\test-report.md"

# 拉取 console log（包含 manual 测试结果）
cmd.exe /c "hdc file recv /data/app/el2/100/base/com.tauri.api/cache/console-log.txt D:\workspace\tauri\tauri\examples\api\console-log.txt"
```

---

## 九、验收标准

### 9.1 Auto 测试

| 指标 | 目标 | 说明 |
|------|------|------|
| 有意义断言测试 | 5/5 通过 | id 验证、getById 验证、removeById 验证 |
| 无有意义断言测试 | 11/11 通过 | 仅验证不抛异常，OHOS 上为 stub |
| **总计** | **16/16 通过** | |

### 9.2 Manual 测试

| 指标 | 目标 |
|------|------|
| 视觉效果验证 | 6/6 确认 |
| 菜单验证 | 4/4 确认 |
| 事件验证 | 5/5 确认 |
| 集成验证 | 3/3 确认 |
| **总计** | **18/18 确认** |

---

## 十、实现变更清单

| 文件 | 变更类型 | 变更内容 |
|------|----------|----------|
| `examples/api/src/lib/tests/tray.ts` | 新增 | 16 个 auto/side-effect 测试用例（5 个有意义断言 + 11 个无有意义断言） |
| `examples/api/src/views/TestRunner.svelte` | 修改 | 导入 trayTests，添加 3 个 manual test handlers 和按钮 |
| `examples/api/src-tauri/src/lib.rs` | 已存在 | OHOS tray 初始化路径（Phase 4 已完成） |
| `examples/api/src-tauri/Cargo.toml` | 已存在 | tray-icon feature（Phase 4 已完成） |

---

## 十一、OHOS 平台特殊处理

### 11.1 需要 skip 的测试

| 测试 | 原因 | 处理方式 |
|------|------|---------|
| `setTempDirPath` | Linux only | auto 测试保留（OHOS stub 返回 Ok），不验证实际效果 |
| `setIconAsTemplate` | macOS only | auto 测试保留（OHOS stub 返回 Ok），不验证实际效果 |
| `setShowMenuOnLeftClick` | 不支持 | auto 测试保留（OHOS stub 返回 Ok），不验证实际效果 |
| `setTitle` | OHOS 无 updateQuickOperationTitle | auto 测试保留（OHOS stub 返回 Ok），不验证实际效果 |

### 11.2 事件验证注意事项

OHOS 事件限制：
- 仅 `Click` 事件，`DoubleClick`/`Enter`/`Move`/`Leave` 不触发
- `button` 仅 `Left`/`Right`，无 `Middle`
- `buttonState` 固定为 `Up`
- `position` 固定为 `(0, 0)`
- `rect` 固定为 default

Manual 测试中需明确标注这些限制，避免测试人员误判。

---

## 十二、测试执行流程

### 12.1 自动测试

```bash
# 使用 ohos-build skill 一键测试
bash .claude/skills/ohos-build/scripts/run-tests.sh "" desktop
```

### 12.2 手动测试

1. 构建并安装到 OHOS 设备
2. 打开应用，进入 Tests 视图
3. 点击 Manual Tests 区域的 tray 相关按钮
4. 按照按钮文案提示操作
5. 查看 console-log.txt 确认事件数据

### 12.3 报告拉取

```bash
# 拉取测试报告
cmd.exe /c "hdc file recv /data/app/el2/100/base/com.tauri.api/cache/test-report.md D:\workspace\tauri\tauri\examples\api\test-report.md"

# 拉取 console log（包含 manual 测试结果）
cmd.exe /c "hdc file recv /data/app/el2/100/base/com.tauri.api/cache/console-log.txt D:\workspace\tauri\tauri\examples\api\console-log.txt"
```

---

## 十三、验收标准

### 13.1 Auto 测试

| 指标 | 目标 | 说明 |
|------|------|------|
| 有意义断言测试 | 5/5 通过 | id 验证、getById 验证、removeById 验证 |
| 无有意义断言测试 | 11/11 通过 | 仅验证不抛异常，OHOS 上为 stub |
| **总计** | **16/16 通过** | |

### 13.2 Manual 测试

| 指标 | 目标 |
|------|------|
| 视觉效果验证 | 6/6 确认 |
| 菜单验证 | 4/4 确认 |
| 事件验证 | 5/5 确认 |
| 集成验证 | 3/3 确认 |
| **总计** | **18/18 确认** |

---

## 十四、实现变更清单

| 文件 | 变更类型 | 变更内容 |
|------|----------|----------|
| `examples/api/src/lib/tests/tray.ts` | 新增 | 16 个 auto/side-effect 测试用例（5 个有意义断言 + 11 个无有意义断言） |
| `examples/api/src/views/TestRunner.svelte` | 修改 | 导入 trayTests，添加 3 个 manual test handlers 和按钮 |
| `examples/api/src-tauri/src/lib.rs` | 已存在 | OHOS tray 初始化路径（Phase 4 已完成） |
| `examples/api/src-tauri/Cargo.toml` | 已存在 | tray-icon feature（Phase 4 已完成） |
| `openharmony-ability/crates/ability/src/statusbar/manager.rs` | 修改 | ThreadsafeFunction 全局变量 + 初始化函数 + 参数构建函数 |
| `openharmony-ability/crates/ability/src/helper/mod.rs` | 修改 | 在 helper 初始化时调用 ThreadsafeFunction 初始化 |
| `crates/tauri/src/tray/mod.rs` | 修改 | `build_inner`、`set_icon`、`set_menu`、`set_tooltip`、`set_visible`、`set_title`、`rect` 加 OHOS 分支 |
| `crates/tauri/src/menu/mod.rs` | 修改 | 恢复宏（移除未提交的 OHOS 分支） |

---

## 十五、本地 HAR 包依赖方案

### 15.1 问题背景

Tauri demo 项目的 `oh-package.json5` 默认依赖 `@ohos-rs/ability: "0.4.0-beta.7"`（从 ohpm 中心仓下载）。该版本不包含 tray 相关的 ArkTS helper 方法（如 `addToStatusBarWithRgba`、`updateStatusBarIconWithRgba` 等）。

需要让项目使用本地 `openharmony-ability` 仓库的代码。

### 15.2 本地 HAR 构建流程

```bash
# 1. 在 openharmony-ability 仓库目录下构建
cd openharmony-ability
ohrs build --arch arm64

# 2. 运行 pack 脚本（最后会报错，已知问题，忽略）
bash scripts/pack.sh

# 3. 打包 HAR 文件
tar -czf ability.har package
```

### 15.3 依赖配置

修改 `examples/api/src-tauri/gen/ohos/entry/oh-package.json5`：

```json5
{
  "dependencies": {
    "libentry.so": "file:./src/main/cpp/types/libentry",
    "@ohos-rs/ability": "file:../../../../../../../openharmony-ability/ability.har"
  }
}
```

路径说明：从 `gen/ohos/entry/` 到 `openharmony-ability/` 需要上 7 层目录。

### 15.4 ArkTS 菜单 API 兼容性

本地 `openharmony-ability/package` 中的 menu 代码存在与 SDK 不兼容的问题。经查阅 OpenHarmony 官方文档，确认以下 API 在当前 SDK 5.0.0(12) 中**不存在**：

| 错误引用 | 位置 | SDK 实际情况 | 正确替代 |
|---------|------|------------|---------|
| `SymbolGlyphOptions` | `menu_types.ets:41-42` | ❌ 不存在此类型 | `SymbolGlyphModifier` (API 12+)，通过 `new SymbolGlyphModifier($r('sys.symbol.xxx')).fontSize('24vp')` 创建 |
| `MenuDivider` | `menu.ets:94,138` | ❌ 不存在此组件 | 使用 `MenuItemGroup({ header: '' }) { MenuItem... }` |
| `MenuItemType` | `menu.ets:108,151` | ❌ 不存在此枚举 | `.selected().selectIcon(true)` |

### 15.5 正确的 ArkUI 菜单 API 规范

根据 OpenHarmony 官方文档 `arkts-popup-and-menu-components-menu.md`：

**核心组件**：
- `Menu()` - 菜单容器组件，需配合 `bindMenu` 或 `bindContextMenu` 使用
- `MenuItem(options?: MenuItemOptions)` - 菜单项
- `MenuItemGroup(options?: MenuItemGroupOptions)` - 菜单项分组

**MenuItemOptions 参数**：
| 参数 | 类型 | 说明 |
|------|------|------|
| `startIcon` | `ResourceStr` | 起始图标（不支持 Symbol 图标） |
| `content` | `ResourceStr` | 菜单项文本 |
| `endIcon` | `ResourceStr` | 末尾图标（不支持 Symbol 图标） |
| `labelInfo` | `ResourceStr` | 标签信息（如快捷键 Ctrl+C） |
| `builder` | `CustomBuilder` | 二级菜单构建器 |
| `symbolStartIcon` (API 12+) | `SymbolGlyphModifier` | 起始 Symbol 图标 |
| `symbolEndIcon` (API 12+) | `SymbolGlyphModifier` | 末尾 Symbol 图标 |

**SymbolGlyphModifier 用法** (API 12+)：
```typescript
import { SymbolGlyphModifier } from '@kit.ArkUI';

const icon = new SymbolGlyphModifier($r('sys.symbol.checkmark'))
  .fontSize('24vp')
  .fontColor([Color.White]);
```

**分隔线**：
- `Menu().menuItemDivider(options)` - 设置 MenuItem 之间的分割线
- `Menu().menuItemGroupDivider(options)` - 设置 MenuItemGroup 上下分割线
- 或使用通用 `Divider()` 组件

**简单菜单项数据结构 (MenuElement)**：
```typescript
interface MenuElement {
  value: ResourceStr;      // 菜单项文本
  icon?: ResourceStr;      // 图标 (API 10+)
  enabled?: boolean;       // 是否可交互 (API 11+)
  action: () => void;      // 点击回调
  symbolIcon?: SymbolGlyphModifier;  // Symbol 图标 (API 12+)
}
```

### 15.6 需要修复的文件

本地 `openharmony-ability/package` 中需要修复的 ArkTS 文件：

| 文件 | 问题 | 修复方案 |
|------|------|---------|
| `src/main/ets/helper/menu_types.ets` | 使用 `SymbolGlyphOptions` | 改为 `SymbolGlyphModifier` |
| `src/main/ets/helper/menu.ets` | 使用 `MenuDivider()`、`MenuItemType` | 改为 `MenuItemGroup`，移除 `MenuItemType` 引用 |
| `src/main/ets/components/DefaultXComponent.ets` | `ArkHelper` 接口缺少 `updateStatusBarMenu` 等方法 | 补充接口定义 |
| `src/main/ets/ability/type.ets` | `ArkHelper` 接口定义不完整 | 补充所有 tray helper 方法签名 |

### 15.7 编译验证

修复后需通过以下验证：

```bash
# 1. 重新打包 HAR
cd openharmony-ability
bash scripts/pack.sh
tar -czf ability.har package

# 2. 安装本地 HAR
cd examples/api/src-tauri/gen/ohos/entry
ohpm install

# 3. 构建验证
bash .claude/skills/ohos-build/scripts/build-ohos.sh
```

预期结果：ArkTS 编译无 ERROR，仅有 WARN（如 ESObject 使用警告）。

---

## 十六、问题修复记录

所有问题修复的详细记录已移至 [`DEBUG.md`](../DEBUG.md)，按 Fix 1 / Fix 2 / ... 格式依次排列。

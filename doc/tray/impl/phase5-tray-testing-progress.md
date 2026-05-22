# Phase 5 进度：Tray 模块端到端测试

> 对应设计文档：[phase5-tray-testing-design.md](./phase5-tray-testing-design.md)
> 创建时间：2026-05-17
> 更新时间：2026-05-20

---

## 一、进度概览

| 任务 | 状态 | 说明 |
|------|------|------|
| 设计文档 | ✅ done | phase5-tray-testing-design.md 已创建 |
| tray.ts 测试文件 | ✅ done | 16 个 auto/side-effect 测试用例已创建 |
| TestRunner.svelte 修改 | ✅ done | 导入 trayTests + 3 个 manual handlers + 按钮 |
| Tray plugin 注册 | ✅ done | `app.rs` 添加 OHOS tray plugin 注册 |
| Helper/Env 全局化 | ✅ done | `helper/mod.rs` 改为全局静态存储，支持跨线程访问 |
| Auto 测试执行 | ✅ done | Phase 6 TSFN 重构后，tray 测试 #108-#122 全部通过 |
| Manual 测试执行 | ✅ done | OHOS 设备人工确认通过

**整体进度**：`100%` ✅ Auto + Manual 测试全部通过

---

## 二、测试结果

### 2.1 最新测试运行（2026-05-18 — Phase 6 重构后）

| 指标 | 数值 | 说明 |
|------|------|------|
| 总测试数 | 122 | 全量测试 |
| 通过 | 117 | 包含全部 15 项 tray 测试 |
| 失败 | 5 | Channel 消息丢失 + http/autostart/clipboard 未加载 |
| Tray 测试 | 15/15 | ✅ 全部通过 |

**Tray 相关用例（#108-#122）全部通过 ✅**：

| # | Test | Duration |
|---|------|----------|
| 108 | TrayIcon.new | 320ms |
| 109 | TrayIcon.new_with_id | 319ms |
| 110 | TrayIcon.getById | 338ms |
| 111 | TrayIcon.getById_not_found | 321ms |
| 112 | TrayIcon.removeById | 350ms |
| 113 | TrayIcon.setIcon | 333ms |
| 114 | TrayIcon.setIcon_null | 332ms |
| 115 | TrayIcon.setMenu | 358ms |
| 116 | TrayIcon.setMenu_null | 328ms |
| 117 | TrayIcon.setTooltip | 345ms |
| 118 | TrayIcon.setTitle | 348ms |
| 119 | TrayIcon.setVisible | 357ms |
| 120 | TrayIcon.setTempDirPath | 344ms |
| 121 | TrayIcon.setIconAsTemplate | 335ms |
| 122 | TrayIcon.setShowMenuOnLeftClick | 350ms |

**5 项失败均为预存问题，与 tray 无关**：
- #3: core.Channel — expected 1000 messages, got 79
- #21: plugin-http.fetch — plugin http not found
- #23: plugin-autostart — plugin autostart not found
- #24: plugin-clipboard-manager.writeText — plugin clipboard-manager not found
- #25: plugin-clipboard-manager.writeImage — plugin clipboard-manager not found

### 2.1.1 历史测试运行（2026-05-17 13:23）

**崩溃信息**: `Signal:SIGABRT(SI_TKILL)@0x01317bcc000033bf`，CppCrash 日志已生成

### 2.2 崩溃分析

| 时间 | 事件 | 说明 |
|------|------|------|
| 13:23:44 | 测试开始 | 应用启动，开始执行 auto 测试 |
| 13:23:47 | Menu 测试完成 | 最后一个通过的测试：`Menu.mixed_items` |
| 13:23:48 | **SIGABRT 崩溃** | `Signal:SIGABRT(SI_TKILL)`，CppCrash |
| 13:23:48+ | CreatePixelMap 成功 | 系统日志显示 `CreatePixelMap success` (65x65) |

**关键发现**:
- 应用在 menu 测试完成后、tray 测试开始前崩溃
- 系统日志显示 `CreatePixelMap success` — PixelMap 创建**成功**
- 崩溃原因：`SIGABRT(SI_TKILL)` — 可能是 statusBarManager 处理 PixelMap 时崩溃
- ArkTS 编译输出确认包含 `createPixelMapSync` 代码
- 错误处理已添加（try-catch 返回 null），但崩溃仍然发生

### 2.3 非 Tray 测试状态

| 模块 | 通过 | 失败 | 说明 |
|------|------|------|------|
| Menu (66 个) | 66 | 0 | ✅ 全部通过 |
| Core/API (14 个) | 13 | 1 | Channel 消息丢失（已知问题） |
| Plugins (5 个) | 1 | 4 | http/autostart/clipboard 未加载 |
| DPI/Window/Image (22 个) | 22 | 0 | ✅ 全部通过 |
| **总计** | **102** | **5** | |

### 2.4 测试历史

| 时间 | 错误信息 | 状态 |
|------|---------|------|
| 10:54 | `Failed to load @kit.ImageKit: GenericFailure` | ArkRuntime 方案失败 |
| 11:15 | `createPixelMapFromPng not found in helper` | ArkTS 函数未暴露 |
| 11:22 | `createPixelMapFromPng not found in helper` | 同上，清理缓存后仍失败 |
| 12:03 | **SIGABRT 崩溃** | PixelMap 创建成功，但 busy-wait 阻塞导致应用崩溃 |
| 12:15 | **方案制定** | 使用 `createPixelMapSync()` 替代异步回调，消除 busy-wait |
| 13:13 | **方案实施** | 修改 ArkTS 使用 `createPixelMapSync`，Rust 移除 busy-wait 代码 |
| 13:13 | **仍崩溃** | SIGABRT 仍然发生，PixelMap 创建成功但 statusBarManager 处理时崩溃 |
| 13:23 | **Clean Rebuild** | 清理构建缓存重新编译，确认 ArkTS 代码已更新，仍崩溃 |
| 14:15 | **新方案** | 改用 RGBA 字节传递而非 PixelMap 对象，避免 NAPI 序列化问题 |
| 14:27 | **首次成功** | 所有 122 个测试运行！但 tray 测试报错 `context not found in helper` |
| 14:33 | **context 修复** | 使用模块级变量存储 context，但仍崩溃 |
| 14:38 | **持续崩溃** | 即使完全 stub 掉 tray 功能和移除 ImageKit 导入，仍然崩溃 |
| 14:47 | **关键发现** | 使用原始未修改的 @ohos-rs/ability 包仍然崩溃！说明崩溃来自 Rust 侧代码变更 |
| 15:00 | **根因定位** | `StatusBarIcon` 类型从 `Object<'static>` 改为 `Vec<u8>` 可能导致内存问题 |
| 15:08 | **验证** | 重新安装原始包后仍崩溃，确认是 Rust 侧类型变更导致 |
| 15:15 | **文档更新** | 记录当前进展，待恢复 StatusBarIcon 原始类型 |

---

## 三、已知阻塞项

### B1: 根因分析 — demo 使用 ohpm 发布包而非本地代码

**问题根因**: Tauri demo 项目的 `oh-package.json5` 依赖 `@ohos-rs/ability: 0.4.0-beta.0`（从 ohpm 下载），而 `createPixelMapFromPng` 函数只存在于本地 `native_ability/` 代码中，尚未发布到 ohpm。

**证据**:
```json
// tauridemo/src-tauri/gen/ohos/entry/oh-package.json5
"@ohos-rs/ability": "0.4.0-beta.0"  // ← ohpm 发布包，不含 createPixelMapFromPng
```

**对比**:
```json
// openharmony-ability/demo/entry/oh-package.json5
"@ohos-rs/ability": "file:../../native_ability"  // ← 本地开发代码，含 createPixelMapFromPng
```

### B1: `image` 模块无法通过 NAPI 传递（✅ 已解决）

**解决方案**: 在 `@ohos-rs/ability` 包中添加 `createPixelMapFromPng` 包装函数，PixelMap 创建**成功**。

### B2: Busy-wait 阻塞事件循环导致应用崩溃（✅ 已找到方案）

**问题**: Rust 侧使用 `std::thread::sleep()` 进行 busy-wait 等待回调结果，阻塞了 ArkTS 事件循环，导致应用被系统看门狗杀死 (SIGABRT)。

**错误**: `Signal:SIGABRT(SI_TKILL)@0x01317bcc0000df38`

**系统日志**: 大量 `CreatePixelMap success` 证实 PixelMap 创建成功，但回调无法执行（事件循环被阻塞）。

**影响**: 所有 tray 测试无法执行，应用在 menu 测试后崩溃。

### B2.1 解决方案：使用 `createPixelMapSync()` 替代异步回调（✅ 已实施，但未解决崩溃）

**调研发现**: OpenHarmony API 12+ 提供 `imageSource.createPixelMapSync(options)` 同步 API。

| API | 同步/异步 | 说明 |
|-----|----------|------|
| `image.createImageSource(buffer)` | ✅ 同步 | 返回 ImageSource 立即 |
| `imageSource.createPixelMap()` | ❌ 异步 | 返回 Promise |
| `imageSource.createPixelMapSync(options)` | ✅ 同步 | 直接返回 PixelMap |

**修改方案**:

1. **ArkTS 侧** (`DefaultXComponent.ets`): 将 `createPixelMapFromPng` 改为同步函数
```typescript
// After (fully synchronous):
createPixelMapFromPng: (pngData: Uint8Array): image.PixelMap | null => {
  let imageSource: image.ImageSource | null = null;
  try {
    imageSource = image.createImageSource(pngData.buffer);
    const pixelMap = imageSource.createPixelMapSync({
      desiredPixelFormat: image.PixelMapFormat.RGBA_8888
    });
    return pixelMap;
  } catch (err) {
    return null;
  } finally {
    if (imageSource) {
      imageSource.release();
    }
  }
}
```

2. **接口定义** (`type.ets`): 修改签名
```typescript
createPixelMapFromPng: (pngData: Uint8Array) => image.PixelMap | null;
```

3. **Rust 侧** (`manager.rs`): 直接调用获取返回值，移除 busy-wait
```rust
let create_fn: Function<'_, Uint8Array, Object<'static>> = 
    helper_obj.get_named_property("createPixelMapFromPng")?;
let pixelmap: Object<'static> = create_fn.call(uint8_array)?;
Ok(pixelmap)
```

**实施结果**:
- ✅ ArkTS 编译通过，代码已确认包含 `createPixelMapSync`
- ✅ PixelMap 创建成功（系统日志确认 `CreatePixelMap success`）
- ❌ 应用仍然崩溃 (SIGABRT)
- ❌ 崩溃发生在 `statusBarManager.addToStatusBar()` 调用时

**下一步行动**:
1. **调查应用退出原因** — 检查 test 107 后应用退出的原因
   - 已确认：报告共 107 个测试，缺少 15 个 tray 测试（108-122）
   - 已确认：plugin 和 menu 的 'manual' 测试被正确过滤（8 + 14 = 22 个 manual 测试）
   - 预期非 manual 测试总数：14 + 11 + 6 + 5 + 5 + 66 + 15 = 122 个
   - 实际执行测试数：107 个（缺少 15 个 tray 测试）
   - 结论：应用在 test 107 完成后、test 108 开始前退出
2. **检查 tray 测试导入** — 确认 `trayTests` 被正确导入并执行
3. **增加日志** — 在 tray 测试中添加更多日志以定位问题
4. **验证 PixelMap 创建** — 确认 `createPixelMapFromRgba` 函数正常工作

**问题**: `@kit.ImageKit` 的 `image` 模块是模块命名空间对象，无法通过 NAPI 的 `ObjectRef` 传递给 Rust。

**影响**: `create_pixelmap_from_rgba` 函数无法创建 PixelMap，导致所有 tray 图标创建失败。

**错误信息**: `image module not found in helper: InvalidArg, Object property 'image' type mismatch. Expect value to be Object, but received Undefined`

### 3.1 调研发现

**为什么 `statusBarManager` 可以传递但 `image` 不行？**

| 特性 | `statusBarManager` | `image` (@kit.ImageKit) |
|------|-------------------|------------------------|
| 类型 | 单例对象 (Singleton Object) | 模块命名空间 (Module Namespace) |
| NAPI 传递 | ✅ 可以作为 `Object` 传递 | ❌ 模块命名空间无法作为普通 Object 传递 |
| 访问方式 | 直接注入到 helper | 需要通过 `import` 或 `requireModule` 访问 |

**关键区别**: `statusBarManager` 是一个具体的 JS 对象实例，而 `image` 是 ES6 模块的命名空间对象，NAPI 无法直接将其作为普通 Object 传递。

### 3.2 尝试的解决方案

#### 方案 1: 通过 helper 传递 `image` 模块 ❌
- **实现**: 在 `ArkHelper` 接口添加 `image: typeof image`，在 `DefaultXComponent.ets` 中注入
- **结果**: 失败，`image` 模块在 NAPI 中显示为 `Undefined`
- **原因**: 模块命名空间对象无法通过 NAPI 的 `ObjectRef` 传递

#### 方案 2: 使用 `ArkRuntime::new()` 加载 `@kit.ImageKit` ❌
- **实现**: 在 Rust 侧创建新的 ArkRuntime，使用 `load_without_info("@kit.ImageKit")` 加载模块
- **结果**: 失败，错误 `Failed to load @kit.ImageKit: GenericFailure`
- **原因**: `ArkRuntime::new()` 创建的是独立的虚拟机，无法访问主运行时的内置模块

#### 方案 3: 在 ArkTS 侧创建 PixelMap 并传递给 Rust ✅ 已实现
- **实现**: 在 `ArkHelper` 添加 `createPixelMapFromPng(pngData, successCb, errorCb)` 方法
- **状态**: 代码已正确实现，但 demo 使用的是 ohpm 发布包（不含此函数）
- **修复**: 修改 demo 的 `oh-package.json5` 使用本地 `native_ability/` 路径
- **待验证**: 需要重新构建并运行测试

#### 方案 4: 从 NAPI global object 直接获取 `image` ❌
- **实现**: 使用 `env.get_global()` 获取全局对象，然后尝试访问 `image` 或 `imageKit`
- **结果**: 失败，模块命名空间不在全局对象中

### 3.3 解决方案

#### 短期方案（测试用）✅ 部分成功
- 修改 `@ohos-rs/ability@0.4.0-beta.7` 包，添加 `createPixelMapFromPng` 函数
- PixelMap 创建**成功**（系统日志确认）
- **新问题**: Rust 侧 busy-wait (`std::thread::sleep`) 阻塞 ArkTS 事件循环，导致应用被看门狗杀死 (SIGABRT)
- **需要**: 改用异步模式或事件循环泵送机制

#### 长期方案（发布用）⬜ 待实施
1. 发布新版 `@ohos-rs/ability` 到 ohpm（包含 `createPixelMapFromPng`）
2. 更新 Tauri CLI 模板中的版本号
3. 更新所有 demo 项目使用新版本

**当前状态**: 根因已定位。`add_to_status_bar()` 中的 `Uint8Array::new()` 在 Chrome_IOThread 上执行，触发 ArkTS VM 跨线程保护（SIGABRT）。需要确保 NAPI 操作在主线程执行。

---

## 四、测试用例清单

### 4.1 Auto 测试（16 个）

| # | 测试名称 | Category | 有意义断言 | 状态 |
|---|---------|----------|-----------|------|
| T1 | `TrayIcon.new` | auto | ✅ id.length > 0 | ❌ blocked |
| T2 | `TrayIcon.new({ id })` | auto | ✅ id === 'custom-id' | ❌ blocked |
| T3 | `TrayIcon.getById(存在)` | auto | ✅ found !== null && id 匹配 | ❌ blocked |
| T4 | `TrayIcon.getById(不存在)` | auto | ✅ found === null | ✅ pass |
| T5 | `TrayIcon.removeById` | side-effect | ✅ 删除后 getById 返回 null | ❌ blocked |
| T6 | `tray.setIcon(dataUri)` | auto | ❌ 无 getter，仅验证不抛异常 | ❌ blocked |
| T7 | `tray.setIcon(null)` | auto | ❌ 无 getter，仅验证不抛异常 | ❌ blocked |
| T8 | `tray.setMenu(menu)` | auto | ❌ 无 getter，仅验证不抛异常 | ❌ blocked |
| T9 | `tray.setMenu(null)` | auto | ❌ 无 getter，仅验证不抛异常 | ❌ blocked |
| T10 | `tray.setTooltip` | auto | ❌ 无 getter，仅验证不抛异常 | ❌ blocked |
| T11 | `tray.setTitle` | auto | ❌ OHOS stub，仅验证不抛异常 | ❌ blocked |
| T12 | `tray.setVisible` | auto | ❌ 无 getter，仅验证不抛异常 | ❌ blocked |
| T13 | `tray.setTempDirPath` | auto | ❌ OHOS stub，仅验证不抛异常 | ❌ blocked |
| T14 | `tray.setIconAsTemplate` | auto | ❌ OHOS stub，仅验证不抛异常 | ❌ blocked |
| T15 | `tray.setShowMenuOnLeftClick` | auto | ❌ OHOS stub，仅验证不抛异常 | ❌ blocked |

**有意义断言**: 5/16 (31%)
**当前通过**: 1/16 (6%) — 仅 `getById_not_found` 通过

### 4.2 Manual 测试（18 个）

| # | 测试名称 | 分类 | 状态 |
|---|---------|------|------|
| M1 | 托盘图标显示 | 视觉效果 | ⬜ pending |
| M2 | 图标更新 | 视觉效果 | ⬜ pending |
| M3 | 图标移除 | 视觉效果 | ⬜ pending |
| M4 | 托盘隐藏 | 视觉效果 | ⬜ pending |
| M5 | 托盘显示 | 视觉效果 | ⬜ pending |
| M6 | Tooltip 显示 | 视觉效果 | ⬜ pending |
| M7 | 右键菜单弹出 | 菜单 | ⬜ pending |
| M8 | 菜单项显示 | 菜单 | ⬜ pending |
| M9 | 菜单更新 | 菜单 | ⬜ pending |
| M10 | 菜单项点击 | 菜单 | ⬜ pending |
| M11 | 左键点击事件 | 事件 | ⬜ pending |
| M12 | 右键菜单点击事件 | 事件 | ⬜ pending |
| M13 | 事件 id 正确性 | 事件 | ⬜ pending |
| M14 | 事件 position | 事件 | ⬜ pending |
| M15 | 事件 rect | 事件 | ⬜ pending |
| M16 | 菜单项触发应用逻辑 | 集成 | ⬜ pending |
| M17 | 菜单项触发窗口操作 | 集成 | ⬜ pending |
| M18 | 多次创建/删除 | 集成 | ⬜ pending |

---

## 五、有意义的断言分析

### 5.1 问题分析

Tray API 大部分方法返回 `void`，且 OHOS 无对应的 getter API：

| 方法 | 返回值 | 对应 getter | 可断言 |
|------|--------|------------|--------|
| `TrayIcon.new()` | TrayIcon | - | ✅ id 非空 |
| `TrayIcon.getById()` | TrayIcon \| null | - | ✅ null/not null |
| `TrayIcon.removeById()` | void | getById | ✅ 删除后验证 |
| `tray.setIcon()` | void | ❌ 无 getIcon | ❌ |
| `tray.setMenu()` | void | ❌ 无 getMenu | ❌ |
| `tray.setTooltip()` | void | ❌ 无 getTooltip | ❌ |
| `tray.setTitle()` | void | ❌ 无 getTitle | ❌ |
| `tray.setVisible()` | void | ❌ 无 isVisible | ❌ |
| `tray.setTempDirPath()` | void | ❌ 无 getter | ❌ |
| `tray.setIconAsTemplate()` | void | ❌ 无 getter | ❌ |
| `tray.setShowMenuOnLeftClick()` | void | ❌ 无 getter | ❌ |

### 5.2 结论

- **有意义断言**: 5/16 (31%) — id 验证、getById 验证、removeById 验证
- **无有意义断言**: 11/16 (69%) — 仅验证不抛异常，OHOS 上为 stub
- **功能验证**: 全部交给 18 个 manual 测试

---

## 六、实现详情

### 6.1 tray.ts 测试用例

| # | 测试名称 | Category | 断言内容 |
|---|---------|----------|---------|
| T1 | `TrayIcon.new` | auto | id.length > 0 |
| T2 | `TrayIcon.new_with_id` | auto | id === 'my-custom-tray' |
| T3 | `TrayIcon.getById` | auto | found !== null && id 匹配 |
| T4 | `TrayIcon.getById_not_found` | auto | found === null |
| T5 | `TrayIcon.removeById` | side-effect | 删除后 getById 返回 null |
| T6 | `TrayIcon.setIcon` | auto | 不抛异常 |
| T7 | `TrayIcon.setIcon_null` | auto | 不抛异常 |
| T8 | `TrayIcon.setMenu` | auto | 不抛异常 |
| T9 | `TrayIcon.setMenu_null` | auto | 不抛异常 |
| T10 | `TrayIcon.setTooltip` | auto | 不抛异常 |
| T11 | `TrayIcon.setTitle` | auto | 不抛异常（OHOS stub） |
| T12 | `TrayIcon.setVisible` | auto | 不抛异常 |
| T13 | `TrayIcon.setTempDirPath` | auto | 不抛异常（OHOS stub） |
| T14 | `TrayIcon.setIconAsTemplate` | auto | 不抛异常（OHOS stub） |
| T15 | `TrayIcon.setShowMenuOnLeftClick` | auto | 不抛异常（OHOS stub） |

### 6.2 Manual Test Handlers

| Handler | 功能 | 预期行为 |
|---------|------|---------|
| `manualTrayIconShow` | 创建托盘图标 | 状态栏显示图标 |
| `manualTrayEvent` | 创建带 action 回调的托盘 | 点击后收到事件，console log 记录 |
| `manualTrayMenu` | 创建带菜单的托盘 | 右键弹出菜单，点击菜单项触发事件 |

---

## 七、根因深度分析

### 7.1 崩溃精确位置

**`manager.rs:241-251`** — `add_to_status_bar()` 调用 `helper_obj.get_named_property("addToStatusBarWithRgba")`，该函数在 ArkTS helper 中不存在。

```rust
// manager.rs:241-251
let add_fn: Function<...> = helper_obj
    .get_named_property("addToStatusBarWithRgba")  // ← 崩溃点
    .map_err(|e| Error::from_reason(...))?;
```

ArkTS helper (`DefaultXComponent.ets:29-165`) 只暴露了：
- `statusBarManager` — OHOS 系统 API
- `context` — UIAbilityContext getter
- `createPixelMapFromPng` — 异步 PixelMap 创建器
- `createWebview`, `requestPermission` 等

**`addToStatusBarWithRgba` 和 `updateStatusBarIconWithRgba` 在整个 ArkTS 代码库中不存在。**

### 7.2 完整执行流程追踪

```
JS: TrayIcon.new({ icon: TEST_ICON })
  ↓
plugin.rs:38-95 — `new()` command
  ↓ 调用 builder.icon(icon).build_inner()
  ↓
tray/mod.rs:351-406 — `build_inner()`
  ↓ 调用 unsafe_builder.take().build()
  ↓
tray-icon/ohos/mod.rs:27-74 — `TrayIcon::new()`
  ↓ 调用 icon::icon_to_status_bar_icon(&icon)
  ↓
tray-icon/ohos/icon.rs:24-45 — `icon_to_status_bar_icon()`
  → 返回 StatusBarIcon {
      white: RefCell::new(Some(white_rgba_vec)),   // Vec<u8>
      black: RefCell::new(Some(black_rgba_vec)),   // Vec<u8>
      size: 24,
    }
  ↓
tray-icon/ohos/mod.rs:59 — add_to_status_bar(app, &item)
  ↓
ability/statusbar/manager.rs:197-254 — add_to_status_bar()
  1. validate_status_bar_item(item)        ✅ 通过
  2. get_main_thread_env()                  ✅ 成功
  3. get_helper() → helper_ref.get_value()  ✅ 成功
  4. 构建 icons_rgba Object (含 Uint8Array)  ✅ 成功
  5. 构建 qo_obj, menu_obj                  ✅ 成功
  6. helper_obj.get_named_property("addToStatusBarWithRgba")  💥 崩溃
```

### 7.3 为什么 `Object<'static>` → `Vec<u8>` 导致 SIGABRT

类型变更本意是**避免** PixelMap 崩溃，但产生了新问题：

**原始设计**（从未工作过）:
```
Rust: StatusBarIcon { white: RefCell<Option<Object<'static>>> }  // NAPI PixelMap
  → build_icons_object() → { white: PixelMap, black: PixelMap }
  → statusBarManager.addToStatusBar(context, { icons: {...} })
  → ArkTS 接收 PixelMap 对象 → SIGABRT（PixelMap 无法在 NAPI 往返中存活）
```

**当前混合状态**:
```
Rust: StatusBarIcon { white: RefCell<Option<Vec<u8>>> }  // RGBA 字节
  → add_to_status_bar() 构建 { white: Uint8Array, black: Uint8Array }
  → helper_obj.get_named_property("addToStatusBarWithRgba")  ← 不存在
  → SIGABRT（NAPI 属性查找在 ArkTS 对象上失败）
```

**关键缺口**: ArkTS helper 从未更新以匹配 Rust 侧的类型变更。Rust 代码被重写为调用 `addToStatusBarWithRgba`，但 `DefaultXComponent.ets` 中没有添加对应的 ArkTS 函数。

### 7.4 为什么"原始包"也崩溃

进度文档记录：*"使用原始未修改的 @ohos-rs/ability 包仍然崩溃"*。

这是因为**原始设计也从未工作过**。原始 `add_to_status_bar` 会：
1. 构建包含 PixelMap `Object<'static>` 引用的 JS 对象
2. 调用 `statusBarManager.addToStatusBar(context, itemObject)`
3. ArkTS 接收经过 NAPI 序列化的 PixelMap 对象
4. PixelMap 是 ArkTS 原生句柄 — 无法在 NAPI 往返中存活
5. `statusBarManager` 接收无效/损坏的 PixelMap → SIGABRT

日志显示 `CreatePixelMap success` 证实 PixelMap 创建成功，但通过 NAPI 传递给 `statusBarManager` 时崩溃。

### 7.5 二次问题：`build_icons_object` 是损坏的死代码

`manager.rs:37-49`:
```rust
fn build_icons_object<'a>(env: &'a Env, icons: &StatusBarIcon) -> Result<Object<'a>> {
    if let Some(white) = icons.white.borrow().as_ref() {
        obj.set("white", white)?;  // ← 期望 Object<'static>，得到 Vec<u8>
    }
    ...
}
```

此函数不再被 `add_to_status_bar()` 调用（后者在内联构建对象），但它仍然存在，如果被调用会因为 `Vec<u8>` 无法直接设置为 NAPI `Object` 属性而编译失败。

---

## 八、推荐解决方案

### 方案 A：在 ArkTS 中实现 `addToStatusBarWithRgba`（推荐）

**原理**: 让 ArkTS 侧完成 RGBA → PixelMap → statusBarManager 的完整流程，Rust 只传递字节数据。

**官方文档确认**（`status_bar_api.md:46-54`）:
- `StatusBarIcon.white`: `image.PixelMap` — 深色壁纸下展示的图标，建议 24vp × 24vp
- `StatusBarIcon.black`: `image.PixelMap` — 浅色壁纸下展示的图标，建议 24vp × 24vp
- **不支持** `Uint8Array`、`Resource` 或原始 buffer

**官方示例**（`status_bar_api.md:322-338`）:
```typescript
// 1. 加载 rawfile 中的 PNG 文件
const whiteFileData = resourceMgr.getRawFileContentSync('testWhite.png');
const whiteBuffer = whiteFileData.buffer;
// 2. 创建 ImageSource
const whiteImageSource = image.createImageSource(whiteBuffer);
// 3. 创建 PixelMap
let whitePixelMap = await whiteImageSource.createPixelMap();
// 4. 使用
let icon: statusBarManager.StatusBarIcon = {
  white: whitePixelMap,
  black: blackPixelMap
};
```

**需要修改的文件**:

1. **`type.ets`** — 添加接口定义:
```typescript
export interface ArkHelper {
  // 现有字段...
  addToStatusBarWithRgba: (
    iconsRgba: { white?: Uint8Array, black?: Uint8Array },
    iconSize: number,
    quickOperation: ESObject,
    statusBarGroupMenu?: ESObject[][],
    hoverTips?: string
  ) => void;
  updateStatusBarIconWithRgba: (
    iconsRgba: { white?: Uint8Array, black?: Uint8Array },
    iconSize: number
  ) => void;
}
```

2. **`DefaultXComponent.ets`** — 添加实现:
```typescript
addToStatusBarWithRgba: (iconsRgba, iconSize, quickOperation, statusBarGroupMenu?, hoverTips?) => {
  const context = this.getUIContext().getHostContext() as common.UIAbilityContext;
  const icons: ESObject = {};
  
  if (iconsRgba.white && iconsRgba.white.length > 0) {
    const opts: image.InitializationOptions = {
      editable: true,
      pixelFormat: image.PixelMapFormat.RGBA_8888,
      size: { width: iconSize, height: iconSize }
    };
    const pm = image.createPixelMapSync(opts);
    pm.writePixelsSync({
      pixels: iconsRgba.white,
      offset: 0,
      stride: iconSize * 4,
      region: { x: 0, y: 0, size: { width: iconSize, height: iconSize } }
    });
    icons.white = pm;
  }
  // black 同理...
  
  statusBarManager.addToStatusBar(context, {
    icons, quickOperation, statusBarGroupMenu, hoverTips
  });
}
```

3. **Rust 侧** (`manager.rs`) — 已实现，无需修改

**优点**:
- 完全消除 NAPI PixelMap 传递问题
- PixelMap 在 ArkTS 侧创建并直接使用，不跨边界
- 同步执行，无 busy-wait
- Rust 侧代码已就绪

**风险**:
- 需要 API 12+（`createPixelMapSync`）
- `this.getUIContext().getHostContext()` 在箭头函数中可能不可用，需要模块级变量存储

### 方案 B：恢复原始类型 + 使用 `createPixelMapFromPng`

**原理**: 恢复 `StatusBarIcon` 为 `Object<'static>`，通过 `createPixelMapFromPng` 创建 PixelMap。

**需要修改**:
1. 恢复 `types.rs` 中 `StatusBarIcon` 的 `white/black` 为 `Object<'static>`
2. 恢复 `icon.rs` 中调用 `create_pixelmap_from_rgba`
3. 实现 `createPixelMapFromPng` 的同步版本（或修复 busy-wait）

**缺点**:
- busy-wait 阻塞事件循环的问题仍未解决
- PixelMap 通过 NAPI 传递仍可能崩溃

### 方案 C：直接调用 statusBarManager 并传递 Uint8Array（❌ 不可行）

**官方文档确认**（`status_bar_api.md:46-54`）:
- `StatusBarIcon.white` 和 `black` 的类型是 **`image.PixelMap`**
- **不接受** `Uint8Array`、`Resource` 或原始 buffer

**结论**: 方案 C **不可行**。`statusBarManager.addToStatusBar` 和 `updateStatusBarIcon` 都要求 `icons.white/black` 为 `image.PixelMap` 类型。任何尝试传递 `Uint8Array` 的做法都会导致参数校验失败（错误码 401）或 SIGABRT。

---

## 九、方案对比

| 维度 | 方案 A | 方案 B | 方案 C |
|------|--------|--------|--------|
| 实现复杂度 | 中 | 低 | 低 |
| 需要 ArkTS 修改 | ✅ | ✅ | ❌ |
| 需要 Rust 修改 | ❌ | ✅ | ✅ |
| 解决 busy-wait | ✅ | ❌ | ✅ |
| 解决 NAPI PixelMap | ✅ | ❌ | ✅ |
| API 版本要求 | API 12+ | 无特殊要求 | N/A |
| 可行性 | ✅ **可行** | ⚠️ 可能崩溃 | ❌ **不可行** |
| 风险 | 中 | 高 | N/A |

**推荐**: **方案 A** 是唯一可行的方案。方案 C 已被官方文档明确排除（statusBarManager 仅接受 PixelMap）。

---

## 十、变更记录

| 时间 | 变更内容 |
|------|----------|
| 2026-05-17 | 创建 Phase 5 设计文档和进度文档 |
| 2026-05-17 | 分析 auto 测试可行性，确认仅 5/16 有有意义断言，其余为 stub 验证 |
| 2026-05-17 | **实现完成**：创建 tray.ts（16 个测试用例），修改 TestRunner.svelte（导入 + 3 个 manual handlers + 按钮） |
| 2026-05-17 | **Tray plugin 注册**：在 `app.rs` OHOS 初始化块中添加 tray plugin 注册 |
| 2026-05-17 | **Helper/Env 全局化**：修改 `helper/mod.rs` 使用全局静态存储，支持跨线程访问 |
| 2026-05-17 | **首次测试运行**：108-122 tray 测试全部失败，错误为 `image module not found in helper` |
| 2026-05-17 | **阻塞项确认**：`@kit.ImageKit` 的 `image` 模块无法通过 NAPI 传递给 Rust |
| 2026-05-17 10:54 | **方案 2 测试**：使用 `ArkRuntime::new()` 加载 `@kit.ImageKit`，失败 `Failed to load @kit.ImageKit: GenericFailure` |
| 2026-05-17 11:15 | **方案 3 实现**：在 `ArkHelper` 添加 `createPixelMapFromPng` 方法，ArkTS 侧实现异步 PixelMap 创建 |
| 2026-05-17 11:15 | **Rust 侧修改**：使用 `create_function_from_closure` 创建回调函数，通过 busy-wait 等待结果 |
| 2026-05-17 11:15 | **测试结果**：`createPixelMapFromPng not found in helper`，ArkTS 函数未正确暴露 |
| 2026-05-17 11:22 | **清理缓存重试**：清理 ArkTS 编译缓存后重新构建，仍失败 |
| 2026-05-17 11:30 | **调研完成**：确认 `statusBarManager` 是单例对象可传递，`image` 是模块命名空间不可传递 |
| 2026-05-17 11:30 | **方案 4 实现**：尝试从 NAPI global object 直接获取 `image` 模块，失败 |
| 2026-05-17 12:00 | **根因分析**：demo 使用 ohpm 发布包（0.4.0-beta.0），不含 `createPixelMapFromPng` |
| 2026-05-17 12:00 | **修复方案**：修改 demo `oh-package.json5` 使用本地 `native_ability/` 路径 |
| 2026-05-17 12:00 | **Rust 代码恢复**：`create_pixelmap_from_rgba` 改回使用 helper 的 `createPixelMapFromPng` |
| 2026-05-17 12:03 | **构建成功**：修改 `@ohos-rs/ability@0.4.0-beta.7` 包添加 `createPixelMapFromPng`，编译通过 |
| 2026-05-17 12:03 | **PixelMap 创建成功**：系统日志确认 `CreatePixelMap success` (65x65) |
| 2026-05-17 12:03 | **应用崩溃**：`SIGABRT(SI_TKILL)` — busy-wait 阻塞 ArkTS 事件循环，被看门狗杀死 |
| 2026-05-17 12:15 | **新阻塞项**：需要异步方案替代 busy-wait，避免阻塞事件循环 |
| 2026-05-17 15:15 | **根因深度分析**：完成崩溃精确位置定位、执行流程追踪、方案对比 |
| 2026-05-17 15:15 | **推荐方案**：方案 A（ArkTS 实现 addToStatusBarWithRgba）或方案 C（验证 statusBarManager 接受 Uint8Array） |
| 2026-05-17 15:30 | **方案 C 验证**：查阅官方文档 `status_bar_api.md`，确认 `StatusBarIcon.white/black` 类型为 `image.PixelMap`，方案 C **不可行** |
| 2026-05-17 15:30 | **最终推荐**：实施方案 A，为唯一可行方案 |
| 2026-05-17 18:30 | **方案 A 实施**：在 `@ohos-rs/ability` 包中添加 `addToStatusBarWithRgba` 和 `updateStatusBarIconWithRgba` |
| 2026-05-17 18:30 | **ArkTS 实现**：使用 `createPixelMapSync` + `writePixelsSync` 在 ArkTS 侧创建 PixelMap |
| 2026-05-17 18:32 | **构建成功**：Rust 和 ArkTS 编译通过 |
| 2026-05-17 18:32 | **关键进展**：应用**不再崩溃**（无 SIGABRT），但提前退出（terminateReason: 2） |
| 2026-05-17 18:44 | **重复验证**：应用仍然在 test 107 后退出，tray 测试未执行 |
| 2026-05-17 20:01 | **事件处理修复**：修复 `registerIconClickHandler` 等函数的回调类型 |
| 2026-05-17 20:01 | **构建成功**：Rust 和 ArkTS 编译通过 |
| 2026-05-17 20:01 | **应用退出**：应用在 test 107 后退出（无崩溃），tray 测试未执行 |
| 2026-05-17 23:32 | **Desktop 模式重新测试**：使用 `TAURI_OHOS_DEVICE_TYPE=desktop` 完整运行 ohos-build 流程 |
| 2026-05-17 23:32 | **关键发现 — tray 测试确实执行了**：hilog 确认 `[TRAY-DEBUG] Test 1 starting: TrayIcon.new` |
| 2026-05-17 23:32 | **SIGABRT 崩溃确认**：CppCrash 日志 `Signal:SIGABRT(SI_TKILL)@0x01317bcc0000b15d` |
| 2026-05-17 23:32 | **根因定位**：`[CheckThread] Fatal: ecma_vm cannot run in multi-thread! thread:45405 currentThread:45591` |
| 2026-05-17 23:32 | **崩溃线程**：`Chrome_IOThread`（tid 45591），非主线程（tid 45405） |
| 2026-05-17 23:32 | **崩溃操作**：`Uint8Array::new()` 在错误线程上创建 NAPI typed array |
| 2026-05-17 23:32 | **报告格式问题**：设备写入 `test-report.md` 而非 `test-report.json`，run-tests.sh 拉取失败 |
| 2026-05-18 | **方案设计**：恢复 `run_item_main_thread!` 宏 + 修复 `build_inner` + stub 方法绕过 |
| 2026-05-18 | **宏恢复**：移除 `menu/mod.rs` 中未提交的 OHOS 分支，恢复原始 `run_on_main_thread + channel` |
| 2026-05-18 | **build_inner 修复**：OHOS 路径改为 `run_on_main_thread + channel` |
| 2026-05-18 | **stub 方法修复**：`set_title` 和 `rect` 加 `#[cfg(ohos)]` 分支直接返回 |
| 2026-05-18 | **编译验证**：`cargo check --target aarch64-unknown-linux-ohos` 通过 |
| 2026-05-18 | **重新审计**：发现 `run_on_main_thread` 在 OHOS 上不可靠（event loop 在 Chrome_IOThread） |
| 2026-05-18 | **方案修正**：改用 ThreadsafeFunction 确保 ArkTS 主线程执行 |
| 2026-05-18 | **statusbar/manager.rs 重写**：5 个 ThreadsafeFunction 全局变量 + 数据全局存储 + 回调在主线程执行 NAPI |
| 2026-05-18 | **types.rs 修改**：添加 `#[derive(Serialize, Deserialize)]` 支持 JSON 序列化 |
| 2026-05-18 | **helper/mod.rs 修改**：`set_helper` 中调用 `init_tray_tsfn()` |
| 2026-05-18 | **tray/mod.rs 修改**：`set_title` 和 `rect` 加 `#[cfg(ohos)]` 分支绕过宏 |
| 2026-05-18 | **menu/mod.rs 修改**：恢复 `run_item_main_thread!` 宏（移除未提交的 OHOS 分支） |
| 2026-05-18 | **编译验证**：`cargo check --target aarch64-unknown-linux-ohos` 通过 |
| 2026-05-18 | **方案升级**：tray/mod.rs 中 `set_icon`、`set_menu`、`set_tooltip`、`set_visible`、`with_inner_tray_icon` 全部改用 `#[cfg(target_env = "ohos")]` 分支直接调用 TSFN |
| 2026-05-18 | **依赖修正**：不应修改 ohpm 中心仓下载的包，改为让本地 `openharmony-ability` 被编译进项目使用 |
| 2026-05-18 | **本地 HAR 构建**：需要构建本地 `@ohos-rs/ability` HAR 包并在 entry 的 oh-package.json5 中通过 `file:` 引用 |
| 2026-05-18 | **ArkTS API 修复**：`SymbolGlyphOptions` → `SymbolGlyphModifier`，`MenuDivider()` → `MenuItemGroup`，`MenuItemType` 移除 |
| 2026-05-18 | **TSFN 回调修复**：回调返回参数元组而非 `()`，让 TSFN 自动调用 JS 函数 |
| 2026-05-18 | **abilityName 获取修复**：`Want.abilityName` 直接使用，不需要 `elementName.abilityName` |
| 2026-05-18 | **TSFN 类型更新**：静态变量类型匹配回调返回类型 `(Object, f64, Object, ...)` |
| 2026-05-18 | **编译验证**：`cargo check --target aarch64-unknown-linux-ohos` 通过 |

## 十一、Desktop 模式测试审计（2026-05-17 23:32）

### 11.1 测试运行结果

| 指标 | 数值 | 说明 |
|------|------|------|
| 总测试数 | 107 | 报告写入时 tray 测试尚未开始 |
| 通过 | 102 | 非 tray 相关测试大部分通过 |
| 失败 | 5 | Channel 消息丢失 + 4 个 plugin 未加载 |
| Tray 测试 | 0/15 | 报告写入后 tray 测试才开始执行 |

**测试报告格式问题**：设备写入的是 `test-report.md`（Markdown 表格），而 `run-tests.sh` 脚本拉取的是 `test-report.json`（JSON 格式），导致脚本报告拉取失败。实际报告存在于 `/data/app/el2/100/base/com.tauri.api/cache/test-report.md`。

### 11.2 Tray 测试执行情况

**hilog 确认 tray 测试确实开始执行**：
```
23:32:26.927 [TRAY-DEBUG] Test 1 starting: TrayIcon.new
23:32:26.928 [TRAY-DEBUG] TrayIcon imported, calling new()
```

但之后没有 `TrayIcon.new returned` 日志，说明 `TrayIcon.new()` 调用过程中崩溃。

### 11.3 崩溃根因 — ArkTS VM 跨线程访问

**CppCrash 日志关键信息**：
```
Reason: Signal:SIGABRT(SI_TKILL)@0x01317bcc0000b15d
LastFatalMessage: [CheckThread] Fatal: ecma_vm cannot run in multi-thread!
  thread:45405 currentThread:45591

Fault thread:
  Tid: 45591, Name: Chrome_IOThread
```

**根因**：`add_to_status_bar()` 中的 `Uint8Array::new()` 在 **Chrome_IOThread** 上执行，但 ArkTS VM（ecma_vm）绑定在主线程（tid 45405）。ArkRuntime 检测到跨线程访问后调用 `abort()` 终止进程。

### 11.4 崩溃堆栈分析

```
#07  libark_jsruntime.so  — Uint8ArrayRef::New()
#08  libace_napi.z.so     — ArkNativeEngine::NapiNewTypedArray()
#09  libace_napi.z.so     — napi_create_typedarray()
#10  libapi_lib.so        — ← 我们的 Rust 代码（Uint8Array::new）
...
#29  libarkweb_engine.so  — ArkWeb_HttpBodyStream::OnReadComplete()
```

**调用链**：webview 的 HTTP body stream 回调（Chrome_IOThread）→ 我们的 Rust 代码 → `Uint8Array::new()` → ArkTS VM 跨线程检测 → SIGABRT。

### 11.5 为什么在主线程调用的代码跑到了 Chrome_IOThread？

`add_to_status_bar()` 中的 `get_main_thread_env()` 使用 thread-local 缓存：
```rust
// helper/mod.rs
pub fn get_main_thread_env() -> Rc<RefCell<Option<Env>>> {
    thread_local! {
        static CACHED_ENV: Rc<RefCell<Option<Env>>> = Rc::new(RefCell::new(None));
    }
    CACHED_ENV.with(|rc| {
        let guard = GLOBAL_MAIN_THREAD_ENV.lock().unwrap();
        *rc.borrow_mut() = guard.0;  // ← 从全局拷贝 Env 到 thread-local
        Rc::clone(rc)
    })
}
```

`Env` 被拷贝到 Chrome_IOThread 的 thread-local 后，在该线程上使用 NAPI 创建 `Uint8Array`，触发 ArkTS VM 的跨线程保护。

### 11.6 修复方向

`add_to_status_bar()` 和 `update_status_bar_icon()` 必须在主线程执行。OHOS 路径当前缺少主线程调度机制。

**已实施的修复**（3 个修改点）：

1. **恢复 `run_item_main_thread!` 宏**（`menu/mod.rs`）— 移除未提交的 OHOS 分支，恢复原始 `run_on_main_thread + channel`
2. **修复 `build_inner`**（`tray/mod.rs`）— OHOS 路径改为 `run_on_main_thread + channel`
3. **stub 方法绕过宏**（`tray/mod.rs`）— `set_title` 和 `rect` 加 `#[cfg(ohos)]` 分支直接返回

**方法分类**：

| 方法 | OHOS 行为 | 处理方式 |
|------|----------|---------|
| `build_inner` | `add_to_status_bar` → NAPI | ✅ 已改 `run_on_main_thread + channel` |
| `set_icon` | `update_status_bar_icon` → NAPI | ✅ 保持宏（自动调度） |
| `set_menu` | `update_status_bar_menu` → NAPI | ✅ 保持宏（自动调度） |
| `set_tooltip` | `update_hover_tips` → NAPI | ✅ 保持宏（自动调度） |
| `set_visible` | `add_to/remove_from_status_bar` → NAPI | ✅ 保持宏（自动调度） |
| `set_title` | stub `{}` | ✅ 已加 `#[cfg(ohos)]` 直接返回 |
| `rect` | 返回 `None` | ✅ 已加 `#[cfg(ohos)]` 直接返回 |

## 十二、本地依赖构建方案（2026-05-18 更新）

### 12.1 问题：不应修改 ohpm 中心仓

**错误做法**：直接修改 `oh_modules/@ohos-rs/ability/` 下的 type.ets 和 DefaultXComponent.ets 文件。
这些文件是从 ohpm 中心仓下载的，修改会在 `ohpm install` 时被覆盖，且不符合依赖管理规范。

**正确做法**：修改本地 `openharmony-ability` 仓库的 ArkTS 源码，然后构建为本地 HAR 包，在 entry 中通过 `file:` 路径引用。

### 12.2 本地 HAR 构建步骤

1. **修改本地 ArkTS 源码**：
   - `openharmony-ability/native_ability/src/main/ets/components/DefaultXComponent.ets`
   - 添加 `addToStatusBarWithRgba()`、`updateStatusBarIconWithRgba()`、`updateStatusBarMenu()`、`updateHoverTips()`、`removeFromStatusBar()` 等方法

2. **构建 HAR 包**：
   ```bash
   cd openharmony-ability/native_ability
   ohpm publish --local  # 或 hvigorw assembleHar
   ```

3. **修改 entry 的 oh-package.json5**：
   ```json5
   {
     "dependencies": {
       "libentry.so": "file:./src/main/cpp/types/libentry",
       "@ohos-rs/ability": "file:../../openharmony-ability/native_ability/har/ability.har"
     }
   }
   ```

### 12.3 当前状态

- `openharmony-ability` 仓库的 Rust 部分（statusbar/manager.rs 等）已更新为 ThreadsafeFunction 方案
- ArkTS 部分（DefaultXComponent.ets）已添加所有 tray helper 方法
- 本地 HAR 包已构建并配置到 entry/oh-package.json5
- ArkTS 菜单 API 兼容性问题已修复（SymbolGlyphModifier、MenuItemGroup 等）
- TSFN 回调参数传递问题已修复（返回参数元组而非 `()`）
- abilityName 获取方式已修复（直接使用 `Want.abilityName`）
- `cargo check --target aarch64-unknown-linux-ohos` 编译通过

### 12.4 下一步

1. ~~签名安装 HAP 到设备~~ ✅ 已完成
2. ~~运行 tray 测试验证功能~~ ✅ 已完成（Phase 6 后 #108-#122 全部 pass）
3. Manual 测试待执行

### 12.5 Phase 6 解决阻塞项（2026-05-18 更新）

Phase 6 TSFN 数据传递重构解决了 Phase 5 中记录的所有阻塞项：

| 阻塞项 | 解决方式 |
|--------|---------|
| B1: `image` 模块 NAPI 传递 | ArkTS 侧 `createPixelMapFromRgba` + `writeBufferToPixelsSync` |
| B2: Busy-wait 阻塞事件循环 | Phase 6: TSFN NonBlocking 直接携带数据，无需 busy-wait |
| SIGABRT 跨线程崩溃 | TSFN 确保 NAPI 操作在 ArkTS 主线程执行 |
| 测试 timeout/freeze | 删除 15 个 DATA_* Mutex，改为原子 struct 传递 |

**验证结果**：122 项 autotest，tray 相关 15 项（#108-#122）全部通过。

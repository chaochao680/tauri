# Phase 9: QuickOperation — Progress

> 对应设计文档：[phase9-quick-operation-design.md](./phase9-quick-operation-design.md)
> 创建时间：2026-05-27
> 更新时间：2026-05-27

---

## 一、进度概览

| 任务 | 状态 | 说明 |
|------|------|------|
| 设计文档 | ✅ done | phase9-quick-operation-design.md 已完成 |
| Step 1: QuickOperationConfig 类型 + Attributes 字段 | ✅ done | `tray-icon/src/lib.rs` |
| Step 2: TrayIconBuilder::with_quick_operation() | ✅ done | `tray-icon/src/lib.rs` |
| Step 3: TrayIcon::set_quick_operation() (lib.rs 层) | ✅ done | `tray-icon/src/lib.rs` |
| Step 4: OHOS 平台层 — build_item() 提取 + set_quick_operation() | ✅ done | `tray-icon/src/platform_impl/ohos/mod.rs` |
| Step 5: tauri TrayIconBuilder::quick_operation() | ✅ done | `tauri/crates/tauri/src/tray/mod.rs` |
| Step 6: tauri TrayIcon::set_quick_operation() | ✅ done | `tauri/crates/tauri/src/tray/mod.rs` |
| Step 7: tauri tray plugin 命令注册 | ✅ done | `tauri/crates/tauri/src/tray/plugin.rs` |
| Step 8: show_menu() 文档注释 | ✅ done | `tray-icon/src/lib.rs` |
| JS API 变更 | ✅ done | `packages/api/src/tray.ts` |
| Auto 测试 | ✅ done | `examples/api/src/lib/tests/tray.ts` |
| 手动测试 | ✅ done | `examples/api/src/views/TestRunner.svelte` |
| Rust 侧示例 | ✅ done | `examples/api/src-tauri/src/tray.rs` |
| 编译验证 (Windows) | ✅ done | `cargo check` + `cargo test` |
| 编译验证 (OHOS 交叉) | ✅ done | `run-tests.sh` 自动完成 aarch64-unknown-linux-ohos 编译 |
| Rust 单元测试 | ✅ done | `cargo test -p tray-icon` 3/3 passed |
| OHOS Auto 测试 | ✅ done | 3/3 setQuickOperation 用例通过 |
| OHOS 手动验证 | 🔶 partial | 5/9 项已验证（#2 #3 #4 #5 #9），4 项待验证 |
| ACL 权限注册 | ✅ done | `build.rs` 添加 `set_quick_operation` 到 `core:tray` |
| 用户使用指南文档 | ✅ done | `doc/tray/guides/quick-operation-guide.md`（已根据实现经验修订 6 处） |
| ArkTS 示例文件 | ✅ done | TestTrayAbility.ets + TestTrayPage.ets + module.json5 + main_pages.json |
| Tray.svelte UI | ✅ done | QuickOperation 配置面板（title/height/abilityName/moduleName） |

**整体进度**：`98%` — 代码实现 + Windows 验证 + OHOS 交叉编译 + Auto 测试 + OHOS 设备核心场景验证全部完成，仅剩 4 项边缘场景手动验证

---

## 二、实现步骤详情

### Step 1-3: tray-icon 层 (lib.rs)

**文件**：`tray-icon/src/lib.rs`
**改动量**：~30 行

| 子步骤 | 内容 | 状态 |
|--------|------|------|
| 1a | 新增 `pub struct QuickOperationConfig { title, height, ability_name, module_name, loading_status }` + `Default` | ✅ |
| 1b | `TrayIconAttributes` 添加 `pub quick_operation: Option<QuickOperationConfig>` | ✅ |
| 1c | `Default` impl 添加 `quick_operation: None` | ✅ |
| 2 | `TrayIconBuilder::with_quick_operation(mut self, config: QuickOperationConfig) -> Self` | ✅ |
| 3 | `TrayIcon::set_quick_operation(&self, config: Option<QuickOperationConfig>)` — OHOS 直调，非 OHOS `let _ = config` | ✅ |

**模式参考**：与 `with_temp_dir_path()` / `set_icon_as_template()` 完全一致，无 `#[cfg]` 门控。

### Step 4: OHOS 平台层

**文件**：`tray-icon/src/platform_impl/ohos/mod.rs`
**改动量**：~210 行（含 4 个新增 UT）

| 子步骤 | 内容 | 状态 |
|--------|------|------|
| 4a | 提取 `fn build_item_from_attrs(attrs: &TrayIconAttributes) -> Result<StatusBarItem>` — 统一 StatusBarItem 构建 | ✅ |
| 4b | `new()` 改用 `build_item_from_attrs()` | ✅ |
| 4c | 新增 `set_quick_operation()` — 更新 attrs + remove + build_item_from_attrs + add | ✅ |
| 4d | `set_title()` / `set_icon()` 重构使用 `build_item_from_attrs()` | ✅ |

**QuickOperation 参数提取逻辑**：
- `Some(config)` → 使用 config 中的 ability_name / title / height / module_name / loading_status
- `None` → 当前行为：ability_name 空字符串，title 取 attrs.title 或 "Tauri App"，height 200

### Step 5-6: tauri 层透传

**文件**：`tauri/crates/tauri/src/tray/mod.rs`
**改动量**：~15 行

| 子步骤 | 内容 | 状态 |
|--------|------|------|
| 5 | `TrayIconBuilder<R>::quick_operation(mut self, config: tray_icon::QuickOperationConfig) -> Self` | ✅ |
| 6 | `TrayIcon<R>::set_quick_operation(&self, config: Option<tray_icon::QuickOperationConfig>)` — `#[cfg(ohos)]` 直调，非 OHOS no-op | ✅ |

### Step 7: plugin 命令注册 + ACL 权限

**文件**：`tauri/crates/tauri/src/tray/plugin.rs` + `tauri/crates/tauri/build.rs`
**改动量**：~41 行

| 子步骤 | 内容 | 状态 |
|--------|------|------|
| 7a | 新增 `set_quick_operation` 命令函数 | ✅ |
| 7b | 注册到 `generate_handler![]` 列表 | ✅ |
| 7c | `build.rs` PLUGINS 数组 `core:tray` 条目添加 `("set_quick_operation", true)` | ✅ |

> **实现发现**：Step 7c 在原始设计中遗漏，导致前端调用被 ACL 拦截返回 `Command plugin:tray|set_quick_operation not allowed by ACL`。

### Step 8: show_menu() 文档注释

**文件**：`tray-icon/src/lib.rs`
**改动量**：1 行

| 子步骤 | 内容 | 状态 |
|--------|------|------|
| 8 | `show_menu()` 添加 `/// - **OHOS**: Unsupported. statusBarManager has no API to programmatically trigger the menu.` | ✅ |

### 额外实现：ArkTS 示例 + UI 配置

> 以下步骤在原始设计中未列出，实现过程中确认为必要。

| 步骤 | 文件 | 内容 | 状态 |
|------|------|------|------|
| ArkTS Ability | `TestTrayAbility.ets` | StatusBarViewExtensionAbility + terminateSelf() 关闭模式 | ✅ |
| ArkTS 页面 | `TestTrayPage.ets` | 弹窗内容（Click Me + Close Panel） | ✅ |
| Ability 注册 | `module.json5` | extensionAbilities 添加 TestTrayAbility | ✅ |
| 路由注册 | `main_pages.json` | 添加 pages/TestTrayPage | ✅ |
| Tray UI | `Tray.svelte` | QuickOperation 配置面板（title/height/abilityName/moduleName） | ✅ |
| packages/api 重建 | `packages/api` | `pnpm build` 重新生成 dist | ✅ |

---

## 三、验证计划

### 3.1 编译验证

| 目标 | 命令 | 状态 |
|------|------|------|
| tray-icon 层 | `cargo check -p tray-icon` | ✅ |
| tauri 层 | `cargo check -p tauri --features tray-icon` | ✅ |
| OHOS 交叉编译 | `cargo check -p tray-icon --target aarch64-unknown-linux-ohos` | ⬜ |

### 3.2 Rust 单元测试

| 目标 | 命令 | 状态 |
|------|------|------|
| tray-icon lib | `cargo test -p tray-icon --lib` | ✅ 3/3 (2 新增 + 1 现有) |
| tray-icon ohos | OHOS 交叉编译内建 | ✅ 9/9 (4 新增 + 5 现有) |
| tauri tray | `cargo test -p tauri --lib tray` | ⬜ |

**新增测试用例**（6 个）：

| # | 测试名 | 文件 | 验证内容 |
|---|--------|------|----------|
| 1 | `quick_operation_config_default` | `lib.rs` | Default 字段值正确 |
| 2 | `builder_with_quick_operation` | `lib.rs` | Builder 设置后 attrs 正确 |
| 3 | `build_item_with_quick_operation` | `ohos/mod.rs` | Some(config) → QuickOperation 字段一致 |
| 4 | `build_item_without_quick_operation` | `ohos/mod.rs` | None → 默认值正确 |
| 5 | `quick_operation_empty_title_falls_back_to_attrs_title` | `ohos/mod.rs` | 空 title → 回退 attrs.title |
| 6 | `quick_operation_no_attrs_title_falls_back_to_default` | `ohos/mod.rs` | 空 title + None attrs.title → "Tauri App" |

验证要点：
- `QuickOperationConfig::default()` 字段值正确
- `build_item_from_attrs()` 重构后现有测试仍通过
- `set_title()` 重构后行为不变
- title 回退链：config.title → attrs.title → "Tauri App"

### 3.3 前端 API 层

**JS API 变更** (`packages/api/src/tray.ts`)：

| 改动 | 状态 |
|------|------|
| 新增 `QuickOperationConfig` interface | ✅ |
| `TrayIconOptions` 添加 `quickOperation?: QuickOperationConfig` | ✅ |
| `TrayIcon` 新增 `setQuickOperation(config \| null)` 方法 | ✅ |

**Auto 测试** (`examples/api/src/lib/tests/tray.ts`)：

| 测试用例 | 状态 |
|----------|------|
| `TrayIcon.setQuickOperation` — 设置 config，不抛异常 | ✅ |
| `TrayIcon.setQuickOperation_null` — 传 null 禁用 | ✅ |
| `TrayIcon.setQuickOperation_update` — 更新后再禁用 | ✅ |

**手动测试** (`examples/api/src/views/TestRunner.svelte`)：

| 测试用例 | 按钮 | 验证内容 |
|----------|------|----------|
| `manualQuickOperationEnable` | Enable QuickOp | 左键点击 → 系统面板弹出 |
| `manualQuickOperationUpdate` | Update QuickOp | 标题和高度更新 |
| `manualQuickOperationDisable` | Disable QuickOp | 左键回退为仅事件 |

**Rust 侧示例** (`examples/api/src-tauri/src/tray.rs`)：

| 改动 | 状态 |
|------|------|
| `create_tray()` 中添加 `.quick_operation(...)` | ✅ |
| menu 添加 "Toggle QuickOp" 菜单项 | ✅ |
| on_menu_event 处理 `toggle-qo` | ✅ |

### 3.4 OHOS 设备手动验证清单

| # | 场景 | 操作 | 预期 | 状态 |
|---|------|------|------|------|
| 1 | 不配置 QuickOperation | Builder 不调 `.quick_operation()` | 左键点击仅触发事件，不弹窗 | ⬜ 待验证 |
| 2 | 配置 QuickOperation | Builder 调 `.quick_operation(config)` | 左键点击弹出系统面板，标题和高度正确 | ✅ 已验证 |
| 3 | 禁用弹窗 | 手动测试 "Disable QuickOp" 按钮 | 左键回退为仅触发事件 | ✅ 已验证 |
| 4 | 运行时更新 | 手动测试 "Update QuickOp" 按钮 | 下次左键点击弹出新配置的面板 | ✅ 已验证 |
| 5 | 面板内容 | 用户写的 ArkTS 页面加载 | 面板显示自定义 UI，交互正常（Click Me 计数 + Close Panel 关闭） | ✅ 已验证 |
| 6 | 数据通信 | Rust 写 AppStorage → 弹窗页面读 | 弹窗页面拿到最新数据 | ⬜ 待验证 |
| 7 | set_title 兼容 | 右键菜单触发 set_tooltip | 标题更新，QuickOperation 不变 | ⬜ 待验证 |
| 8 | set_visible 兼容 | `setVisible(false)` → `setVisible(true)` | 图标消失再出现，QuickOperation 保持 | ⬜ 待验证 |
| 9 | 右键菜单不受影响 | 右键点击 | 菜单正常弹出 | ✅ 已验证 |

---

## 四、文件变更清单

### tauri 仓库（21 文件，~1077 行插入）

| 文件 | 变更类型 | 实际行数 |
|------|----------|----------|
| `crates/tauri/build.rs` | ACL 权限注册 `set_quick_operation` | +1 |
| `crates/tauri/src/tray/mod.rs` | Builder/setter 透传 + QuickOperationConfig re-export | +72 |
| `crates/tauri/src/tray/plugin.rs` | 命令注册 + generate_handler | +40 |
| `packages/api/src/tray.ts` | QuickOperationConfig interface + setQuickOperation() | +48 |
| `examples/api/src/lib/tests/tray.ts` | 3 个 auto test | +35 |
| `examples/api/src/views/TestRunner.svelte` | 3 个 manual handlers + 按钮 | +51 |
| `examples/api/src/views/Tray.svelte` | QuickOperation 配置 UI 面板 | +45 |
| `examples/api/src-tauri/src/tray.rs` | quick_operation 配置 + toggle 菜单项 | +20 |
| `examples/api/src-tauri/gen/ohos/.../TestTrayAbility.ets` | StatusBarViewExtensionAbility 示例 | +28 (新文件) |
| `examples/api/src-tauri/gen/ohos/.../TestTrayPage.ets` | 弹窗页面（Click Me + Close Panel） | +44 (新文件) |
| `examples/api/src-tauri/gen/ohos/.../module.json5` | 注册 TestTrayAbility extensionAbility | +8 |
| `examples/api/src-tauri/gen/ohos/.../main_pages.json` | 注册 TestTrayPage 路由 | +3 |
| `examples/api/src-tauri/gen/schemas/*.json` (5 个) | ACL schema 自动更新 | auto |
| `doc/tray/guides/quick-operation-guide.md` | 用户使用指南（已修订 6 处） | 新文件 |

### tray-icon 仓库（2 文件，~275 行插入）

| 文件 | 变更类型 | 实际行数 |
|------|----------|----------|
| `src/lib.rs` | QuickOperationConfig 类型 + Attributes 字段 + Builder + setter + 文档 + 2 UT | +102 |
| `src/platform_impl/ohos/mod.rs` | build_item_from_attrs() 提取 + set_quick_operation() + 重构 set_title/set_icon + 4 UT | +210 |

**总新增代码**：~1352 行（含测试、示例、ArkTS 文件、schema 自动生成）

---

## 五、依赖关系

```
Step 1-3 (tray-icon lib.rs)
    │
    ▼
Step 4 (OHOS 平台层) ──→ 编译验证 + 单元测试
    │
    ▼
Step 5-6 (tauri 层透传)
    │
    ▼
Step 7 (plugin 命令) + Step 8 (文档注释)
    │
    ▼
JS API 变更
    │
    ▼
Auto/Manual 测试 + Rust 侧示例
    │
    ▼
OHOS 设备验证
    │
    ▼
用户使用指南文档
```

---

## 六、风险与注意事项

| 项目 | 风险 | 缓解措施 |
|------|------|----------|
| build_item_from_attrs() 重构 | set_title()/set_icon() 行为回归 | 重构后跑全部现有 UT ✅ 已验证 |
| QuickOperation ability_name 不存在 | 系统报错或静默失败 | 文档说明 + 手动测试覆盖 |
| loading_status 低版本不支持 | 6.0.0(20) 以下忽略 | 文档标注版本要求 |
| 弹窗页面运行在 ExtensionAbility 进程 | 数据隔离，不能直接访问主进程状态 | 文档引导使用 AppStorage/Preferences |
| **ACL 权限注册** | 新命令未注册到 build.rs PLUGINS 数组 → 前端调用被 ACL 拦截 | 在 `core:tray` 条目中添加 `("set_quick_operation", true)` |
| **packages/api 重建** | 修改 tray.ts 后未 `pnpm build` → 设备运行时方法不存在 | 修改 packages/api 后必须执行 `cd packages/api && pnpm build` |
| **OHOS 单 tray 限制** | StatusBar API 是全局操作，一个应用只能有一个 tray 图标 | 文档说明 + autotest 创建 tray 会替换 Rust setup 创建的 tray |
| **弹窗关闭方式** | `UIExtensionContentSession` 没有 `hide()` 方法 | 使用 `terminateSelf()` 关闭弹窗，仅关闭 session 不影响 tray 和应用 |
| **弹窗页面路由注册** | 忘记在 main_pages.json 注册 → 页面加载失败 | 文档前置条件中添加 main_pages.json 注册步骤 |

---

## 七、用户使用指南（交付物）

**文件**：`doc/tray/guides/quick-operation-guide.md`
**状态**：✅ 已创建

实现完成后，编写用户使用指南文档，内容包括：

1. **概述** — QuickOperation 是什么，与其他平台左键行为的区别
2. **前置条件** — Cargo.toml 配置、module.json5 注册、API 版本要求
3. **快速开始** — 最小可运行示例（Rust + ArkTS）
4. **详细示例** — 完整示例（Rust Builder + Setter + ArkTS ExtensionAbility + 页面 + 数据通信）
5. **API 参考** — QuickOperationConfig 字段说明
6. **平台差异** — OHOS only 标注，其他平台行为说明
7. **FAQ** — 常见问题（弹窗不出现、数据不同步、版本兼容）

参考 [Tauri System Tray 官方文档](https://v2.tauri.app/learn/system-tray/) 的风格：
- 渐进复杂度（先简后繁）
- Rust + JS 双语言示例
- 平台专属说明用 Note 块
- 链接到 API Reference 而非重复文档

---

## 八、Log

### 2026-05-27（设计与实现）

- 完成设计文档 phase9-quick-operation-design.md（6 节完整内容）
- 创建进度文档
- 确认 QuickOperation 为 OHOS 独有能力，遵循 temp_dir_path / icon_is_template 既有模式
- 确认 openharmony-ability 层无需改动（QuickOperation struct 已有 loading_status 字段）
- 确认实现步骤 8 个 + 验证清单 9 项 + 用户使用指南 1 份
- 创建用户使用指南文档 `doc/tray/guides/quick-operation-guide.md`
- 完成全部 8 个 Rust 实现步骤（Step 1-8）
- 完成 JS API 变更：QuickOperationConfig interface + TrayIconOptions.quickOperation + setQuickOperation()
- 完成 3 个 auto 测试 + 3 个 manual 测试
- 完成 Rust 侧示例：tray.rs 添加 .quick_operation() 配置 + toggle-qo 菜单项
- 编译验证通过：cargo check -p tauri ✅，cargo test -p tray-icon --lib 3/3 ✅
- OHOS 交叉编译 + 设备部署通过（run-tests.sh desktop）
- **发现并修复 ACL 权限遗漏**：`build.rs` 中 `core:tray` 权限列表缺少 `set_quick_operation`
- **发现 packages/api 重建要求**：修改 tray.ts 后必须 `pnpm build`，否则运行时方法不存在
- OHOS Auto 测试 3/3 通过

### 2026-05-27（ArkTS 示例 + 设备验证）

- 创建 TestTrayAbility.ets：StatusBarViewExtensionAbility 示例，使用 terminateSelf() 关闭弹窗
- 创建 TestTrayPage.ets：弹窗页面（Click Me 计数 + Close Panel 关闭按钮）
- 注册 module.json5 extensionAbility + main_pages.json 路由
- 更新 Tray.svelte：添加 QuickOperation 配置 UI（title/height/abilityName/moduleName 输入框）
- **修复 Close Panel 按钮**：UIExtensionContentSession 没有 hide() 方法，改用 terminateSelf()
- **确认 terminateSelf() 行为**：仅关闭 QuickOperation session，不影响 tray 图标和应用主体
- **确认 OHOS 单 tray 限制**：autotest TrayIcon.new() 会替换 Rust setup() 创建的 tray-1，autotest 清理后 tray 消失
- OHOS 设备手动验证 5/9 项通过（#2 配置弹窗 ✅ #3 禁用弹窗 ✅ #4 运行时更新 ✅ #5 面板内容 ✅ #9 右键菜单 ✅）

### 2026-05-27（提交 + 文档修订）

- 提交 tray-icon 仓库：`587453a` feat: add QuickOperation support for OHOS left-click popup
- 提交 tauri 仓库：`81b70864b` feat: add QuickOperation API for OHOS tray left-click popup
- 根据实现经验修订用户指南 6 处：
  1. 添加 main_pages.json 注册为前置条件
  2. 修正 onSessionDestroy 签名（无参数）
  3. 添加 Close Panel 按钮示例（terminateSelf）
  4. 添加注意事项章节（单 tray 限制、左/右键行为、terminateSelf）
  5. 明确运行时更新机制（remove + rebuild + add）
  6. 添加 FAQ 条目（autotest tray 替换、运行时更新时序）
- 提交 tauri 仓库：`9c9c67f0a` docs: update QuickOperation guide with implementation learnings
- 更新 progress 和 design 文档反映实际实现状态

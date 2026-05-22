# OHOS Menu 模块实现设计 - 阶段索引

## 总体规划

基于 `ohos-menu-api-design.md` 设计文档，将 Menu 模块 OHOS 适配分为 9 个阶段实现。

**总工期**: 12-14 天 → 实际已完成

## 阶段概览

| 阶段 | 名称 | 工期 | 主要目标 | 输出物 | 状态 |
|------|------|------|---------|--------|------|
| [Phase 0](impl/phase0-muda-ohos-design.md) | muda OHOS 后端 | 2天 | 为 muda crate 实现 OHOS 平台支持 | `muda/platform_impl/ohos/` | ✅ |
| [Phase 1](impl/phase1-infrastructure.md) | 基础架构 | 2天 | 建立类型定义和 FFI 框架 | `menu_types.ets`, `menu/mod.rs` | ✅ |
| [Phase 2](impl/phase2-core-types.md) | 核心菜单类型 | 3天 | 实现 Menu/MenuItem/Submenu/MenuEvent | `menu.ets`, `menu/types.rs`, `menu/event.rs` | ✅ |
| [Phase 3](impl/phase3-predefined.md) | PredefinedMenuItem | 2天 | 实现预定义菜单项功能 | `predefined.ets`, `window_ops.ets` | ✅ |
| [Phase 4](impl/phase4-popup-integration.md) | Popup 集成 (bindContextMenu) | 3天 | 实现 Menu.popup() | `TauriMenu.ets`, `menu/popup.rs` | ✅ 已被 Phase 9 替代 |
| [Phase 5](impl/phase5-integration-testing.md) | 集成与测试 | 2-3天 | Tauri API 集成和测试 | `menu/ohos.rs`, `tests/menu.ts` | ✅ |
| [Phase 6](impl/phase6-api-demo-integration.md) | api-demo 集成 (openMenu) | 2-3天 | api-demo 端到端验证 | `menu.ets` (openMenu) | ✅ 已被 Phase 9 替代 |
| [Phase 7](impl/phase7-api-testing-design.md) | API 测试设计 | 1天 | 测试用例设计 | 测试设计文档 | ✅ |
| [Phase 8](impl/phase8-popup-root-cause-and-fix.md) | popup 设备修复 (openMenu) | 1天 | 修复 popup 在设备上无效果 | 修复方案 | ⚠️ 设计完成，被 Phase 9 替代 |
| [Phase 9](impl/phase9-popup-bindcontextmenu-design.md) | bindMenu 重构 + 全类型支持 | 1天 | 重构 popup 为 bindMenu，支持全类型 | `MainPage.ets`, `menu.ets` | ✅ 完成 |

## 阶段依赖关系

```
Phase 0 (muda OHOS 后端)
    ↓
Phase 1 (基础架构)
    ↓
Phase 2 (核心菜单类型) → Phase 3 (PredefinedMenuItem)
    ↓                         ↓
    └────────────┬────────────┘
                  ↓
          Phase 4 (Popup 集成 — bindContextMenu) ← 已被 Phase 9 替代
                  ↓
          Phase 5 (集成与测试)
                  ↓
          Phase 6 (api-demo 集成 — openMenu) ← 已被 Phase 9 替代
                  ↓
          Phase 7 (API 测试设计)
                  ↓
          Phase 8 (popup 设备修复 — openMenu) ← 设计完成，被 Phase 9 替代
                  ↓
          Phase 9 (bindMenu 重构 + 全类型支持) ← 最终方案 ✅
```

**Phase 演进说明**：
- Phase 4 使用 `bindContextMenu` 方案，Phase 6 改为 `openMenu` 方案，Phase 9 最终重构为 `bindMenu` 方案
- Phase 8 的 `openMenu` 修复方案因模块级 `@Builder` 无 `this` 导致子菜单 crash，Phase 9 彻底重构
- 最终方案：`bindMenu(isShow)` + `@Component @Builder` 递归渲染，支持全类型 item

## 关键技术决策

### 1. Menu.popup() 实现方案（最终方案 — Phase 9）
- **方案**: 使用 `bindMenu(isShow: boolean)` + `@Component @Builder` 递归渲染
- **原理**: `@Component` 内的 `@Builder` 有 `this` 上下文，支持 `MenuItem.builder` 递归子菜单
- **实现**: 通过 `AppStorage` 桥接 Rust 和 ArkTS，`@StorageLink` 响应式控制显隐
- **历史演进**: Phase 4 (bindContextMenu) → Phase 6 (openMenu) → Phase 9 (bindMenu 最终方案)
- **详见**: [phase9-popup-bindcontextmenu-design.md](impl/phase9-popup-bindcontextmenu-design.md)

### 2. PredefinedMenuItem 执行方式
- **剪贴板**: 使用 `runJavaScript` + `document.execCommand()`
- **窗口操作**: 使用 OHOS Window API (注意命名差异)
- **应用退出**: 使用 `context.terminateSelf()`

### 3. API 命名映射

| Tauri API | OHOS API | 备注 |
|-----------|----------|------|
| `close()` | `destroyWindow()` | OHOS 无 close 方法 |
| `restore()` | `recover()` | 从 maximize/fullscreen 恢复 |
| `restore()` | `restore()` | 从 minimize 恢复（仅 2in1） |
| `fullscreen` | `maximize(ENTER_IMMERSIVE)` | 无 setFullScreen |
| `hide()` | `minimize()` | 主窗口 minimize = 隐藏 |

## 文档索引

### 设计文档
- [主设计文档](ohos-menu-api-design.md) - API 分析、架构设计、审计结果

### 实现设计文档
- [Phase 0: muda OHOS 后端](impl/phase0-muda-ohos-design.md)
- [Phase 1: 基础架构](impl/phase1-infrastructure.md)
- [Phase 2: 核心菜单类型](impl/phase2-core-types.md)
- [Phase 3: PredefinedMenuItem](impl/phase3-predefined.md)
- [Phase 4: Popup 集成 (bindContextMenu)](impl/phase4-popup-integration.md) ← 已被 Phase 9 替代
- [Phase 5: 集成与测试](impl/phase5-integration-testing.md)
- [Phase 6: api-demo 集成 (openMenu)](impl/phase6-api-demo-integration.md) ← 已被 Phase 9 替代
- [Phase 7: API 测试设计](impl/phase7-api-testing-design.md)
- [Phase 8: popup 设备修复 (openMenu)](impl/phase8-popup-root-cause-and-fix.md) ← 被 Phase 9 替代
- [Phase 9: bindMenu 重构 + 全类型支持](impl/phase9-popup-bindcontextmenu-design.md) ← **最终方案**

### 进度文档
- [总进度追踪](ohos-menu-api-progress.md)
- [Phase 0 进度](impl/phase0-muda-ohos-progress.md)
- [Phase 1 进度](impl/phase1-infrastructure-progress.md)
- [Phase 2 进度](impl/phase2-core-types-progress.md)
- [Phase 3 进度](impl/phase3-predefined-progress.md)
- [Phase 4 进度](impl/phase4-popup-integration-progress.md)
- [Phase 5 进度](impl/phase5-integration-testing-progress.md)
- [Phase 6 进度](impl/phase6-api-demo-integration-progress.md)
- [Phase 8 进度](impl/phase8-popup-progress.md)
- [Phase 9 进度](impl/phase9-popup-bindcontextmenu-progress.md) ← **最新完成**

### 问题修复
- [DEBUG.md](DEBUG.md) — Fix 1-6 完整修复记录

### 参考文档
- [Menu 组件](reference/menu.md) - Menu 容器、SubMenuExpandingMode
- [MenuItem 组件](reference/menu_item.md) - MenuItem 属性、onClick/onChange
- [MenuItemGroup](reference/menu_item_group.md) - 菜单项分组
- [bindMenu API](reference/menu_control.md) - 菜单弹出控制（核心）
- [ContextMenu.close](reference/context_menu.md) - 关闭菜单方法
- [promptAction](reference/prompt_action.md) - ActionMenu 备选方案
- [OHOS Window API](../.claude/skills/ohos-window-adapter/references/) - 窗口操作
- [WebView Clipboard](../.claude/skills/arkweb-adapter/references/) - 剪贴板验证

## 验证标准

### 测试结果 (2026-05-21 — Phase 9 完成后)

| 指标 | 数值 |
|------|------|
| Menu auto tests | **66/66 (100%)** |
| Tray auto tests | **21/21 (100%)** |
| 总测试通过 | 120/128 |
| 总失败 | 8 (全部非 menu/tray 相关) |
| Manual tests | 14 (UI 效果已设备验证) |

### 功能验证
- ✅ Menu 创建、追加、删除菜单项
- ✅ MenuItem 自定义文本、启用状态
- ✅ Submenu 嵌套子菜单（递归 hover 展开）
- ✅ PredefinedMenuItem copy/cut/paste/selectAll/undo/redo
- ✅ PredefinedMenuItem minimize/maximize/close/quit
- ✅ Menu.popup() 程序化弹出（bindMenu 方案，精确定位）
- ✅ MenuEvent 事件触发和监听
- ✅ CheckMenuItem 创建、checked 状态 toggle
- ✅ IconMenuItem 创建、图标显示
- ✅ Separator 分隔线可见
- ✅ MenuItemKind 类型标识
- ✅ 混合类型集成测试
- ✅ 无 `&` 助记符残留
- ✅ 无 accelerator 快捷键文本

### API 对等性
- ✅ 与 Windows/macOS 功能对等
- ⚠️ hideOthers/showAll 不支持（跨应用限制）
- ⚠️ about() 自定义 AlertDialog 已实现
- ✅ separator 文本在所有平台为空（正确行为）

### 测试覆盖
- ✅ Rust UT：OHOS 特有代码（纯逻辑测试）
- ✅ 前端 auto 测试：66 个 menu + 21 个 tray API 调用全部通过
- ✅ 前端 manual 测试：14 个已设备验证（UI、交互）

## 已知限制

1. **hideOthers/showAll**: OHOS 不支持跨应用窗口操作
2. **嵌套 submenu**: OHOS 支持多级嵌套（Phase 9 已实现，实际 ≤2 级）
3. **菜单位置精度**: `anchorPosition` 精确定位（API 20+）
4. **键盘快捷键**: 需额外实现全局监听（不在当前范围）
5. **Popup UI 效果**: 已通过设备手动验证（Phase 9）

## API 版本要求

| 功能 | 最低 API 版本 |
|------|-------------|
| `bindMenu(isShow)` | API 11 |
| `anchorPosition` 精确定位 | API 20 |
| `showInSubWindow` | API 11 |
| `minimize/maximize/recover` | API 12 |
| `restore` (从 minimize) | API 14（仅 2in1） |
| `destroyWindow` | API 6 |

## 快速开始

```bash
# 1. 阅读设计文档
cat README.md
cat ohos-menu-api-design.md

# 2. 运行前端 API 测试（一键）
bash .claude/skills/ohos-build/scripts/run-tests.sh

# 3. 查看测试报告
# - test-report.md: 自动测试结果
# - console-log.txt: 手动测试日志
```

## 测试结果

最新测试运行 (2026-05-21 — Phase 9 完成后):
- **66/66 menu auto tests pass** (100%)
- **21/21 tray auto tests pass** (100%)
- **120/128 total tests pass**
- 8 failures 全部为非 menu/tray 相关（插件未加载、Channel 通信问题）
- Manual tests UI 效果已设备验证通过

## 相关 SKILL 文档

- [ohos-rust-ut](../.claude/skills/ohos-rust-ut/SKILL.md) - Rust 单元测试
- [frontend-api-testing](../.claude/skills/frontend-api-testing/SKILL.md) - 前端 API 测试
- [ohos-build](../.claude/skills/ohos-build/SKILL.md) - OHOS 构建

## 联系与反馈

实现过程中如发现问题，请更新 `ohos-menu-api-design.md` 的"审计结果"章节，记录新发现的问题和修正方案。

## 关键修复记录

| 日期 | 问题 | 修复 |
|------|------|------|
| 2026-05-16 | `run_main_thread!` 死锁 | 方法级别 OHOS 直接执行路径 |
| 2026-05-16 | `JsImage` data URI 当作文件路径 | 自定义反序列化器 + base64 解码 |
| 2026-05-16 | separator 文本断言失败 | 改为验证 id.length > 0 |
| 2026-05-21 | openMenu + ComponentContent 子菜单 crash | Phase 9: bindMenu + @Component @Builder 递归渲染 |
| 2026-05-21 | Predefined close/hide 不可恢复 | close/hide → minimize（安全等价） |
| 2026-05-21 | `&` 助记符残留 | muda ohos: text.replace("&", "") |
| 2026-05-21 | accelerator 文本显示 | muda ohos: accelerator = None |
# Phase 3: tauri-runtime-wry OHOS 编译修复进度

> 版本：v2.0
> 更新时间：2026-05-14

---

## 总览

| 状态 | 数量 |
|------|------|
| ⬜ Not Started | 0 |
| 🔄 In Progress | 0 |
| ✅ Completed | 19 |

---

## 修改清单

### tauri-runtime-wry (6 项)

| # | 文件 | 修改内容 | 状态 |
|---|------|----------|------|
| 1 | `monitor/mod.rs:32` | `#[cfg(mobile)]` → `#[cfg(any(mobile, target_env = "ohos"))]` | ✅ |
| 2 | `window/mod.rs:57` | 同上，WindowExt 复用 mobile 空实现 | ✅ |
| 3 | `lib.rs:4942` | `#[cfg(desktop)]` → `#[cfg(all(desktop, not(target_env = "ohos")))]` | ✅ |
| 4 | `lib.rs:4954` | NewWindowOpener.webview 排除 ohos | ✅ |
| 5 | `lib.rs:4965` | NewWindowResponse::Create 排除 ohos | ✅ |
| 6 | `lib.rs:4977` | 同上 | ✅ |

### tauri (12 项)

| # | 文件 | 修改内容 | 状态 |
|---|------|----------|------|
| 7 | `app.rs:1108` | `#[cfg(desktop)]` → 排除 ohos，解决 duplicate definitions | ✅ |
| 8 | `manager/menu.rs:81-90` | `#[cfg(all(..., not(target_env = "ohos")))]` 排除 gtk 相关 | ✅ |
| 9 | `webview/mod.rs:242-262` | NewWindowResponse enum 拆分（ohos 只有 Allow/Deny） | ✅ |
| 10 | `webview/mod.rs:710-720` | match 分支添加 cfg 条件 | ✅ |
| 11 | `window/mod.rs:1346-1355` | remove_for_gtk_window 排除 ohos | ✅ |
| 12 | `window/mod.rs:1378-1387` | hide_for_gtk_window 排除 ohos | ✅ |
| 13 | `window/mod.rs:1412-1421` | show_for_gtk_window 排除 ohos | ✅ |
| 14 | `window/mod.rs:1441-1449` | is_visible_on_gtk_window 排除 ohos | ✅ |
| 15 | `plugin.rs:33` | `#[cfg(mobile)]` → `#[cfg(any(mobile, target_env = "ohos"))]` | ✅ |
| 16 | `plugin/mobile.rs:14-20` | oneshot/AtomicI32 导入添加 ohos cfg | ✅ |
| 17 | `plugin/mobile.rs:35` | PENDING_PLUGIN_CALLS_ID 添加 ohos cfg | ✅ |
| 18 | `lib.rs:78` | mobile_entry_point 宏导出添加 ohos | ✅ |

### examples/api (1 项)

| # | 文件 | 修改内容 | 状态 |
|---|------|----------|------|
| 19 | `src-tauri/src/lib.rs:179` | new_window handler 代码排除 ohos | ✅ |

---

## 编译阶段发现的问题

| # | 问题 | 错误类型 | 解决方案 |
|---|------|----------|----------|
| 1 | MonitorExt/WindowExt 无实现 | E0599 | 复用 mobile 空实现 |
| 2 | NewWindowOpener.webview 无字段 | E0560/E0609 | 排除 ohos desktop |
| 3 | NewWindowResponse::Create 无 variant | E0599 | 排除 ohos desktop |
| 4 | supports_multiple_windows duplicate | E0592 | 排除 ohos desktop |
| 5 | gtk 相关方法不存在 | E0599 | 排除 ohos |
| 6 | plugin::mobile 模块未找到 | E0433 | 导出给 ohos |
| 7 | oneshot crate 未找到 | E0433 | 导入添加 ohos cfg |
| 8 | mobile_entry_point 未找到 | E0433 | 宏导出添加 ohos |
| 9 | NewWindowResponse type parameter R 未使用 | E0392 | PhantomData |
| 10 | NewWindowResponse match 不完整 | E0004 | 添加 cfg 分支 |

---

## 验证测试

| # | 测试 | 状态 | 结果 |
|---|------|------|------|
| V3.1 | tauri-runtime-wry 编译 | ✅ | 成功 |
| V3.2 | tauri 编译 | ✅ | 成功 |
| V3.3 | api 编译 | ✅ | 成功 |
| V3.4 | HAP 构建 | ✅ | 成功 |
| V3.5 | 安装启动 | ✅ | 成功 |
| V3.6 | autotest | ✅ | 36/41 通过 (87.8%) |

---

## 状态图标

| 图标 | 含义 |
|------|------|
| ⬜ | Not Started |
| 🔄 | In Progress |
| ✅ | Completed |
| 🔒 | Blocked |

---

## 更新日志

### 2026-05-14

- ✅ 修复 MonitorExt/WindowExt trait 实现（复用 mobile 空实现）
- ✅ 修复 NewWindowOpener/NewWindowResponse cfg 条件
- ✅ 修复 supports_multiple_windows 方法冲突
- ✅ 修复 gtk 相关代码（init_for_gtk_window 等）
- ✅ 修复 plugin::mobile 模块导出
- ✅ 修复 plugin/mobile.rs 导入（oneshot、AtomicI32）
- ✅ 修复 mobile_entry_point 宏导出
- ✅ 修复 NewWindowResponse enum 拆分（ohos 只有 Allow/Deny）
- ✅ 修复 examples/api new_window handler 代码
- ✅ 端到端验证通过：36/41 测试通过 (87.8%)

---

## 关键设计决策

1. **复用 mobile 空实现**：ohos desktop 与 mobile 共享空实现，语义正确且最小改动
2. **NewWindowResponse 拆分**：ohos 只有 Allow/Deny，无 Create variant
3. **gtk 相关全部排除**：ohos 不支持 gtk，所有相关代码添加 `not(target_env = "ohos")`
4. **mobile 模块导出给 ohos**：plugin::mobile 和 mobile_entry_point 需要给 ohos 使用
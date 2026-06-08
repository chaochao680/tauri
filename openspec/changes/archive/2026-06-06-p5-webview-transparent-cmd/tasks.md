# Phase 5: WebView 透明命令注册修复 - 实施任务

## 状态
- **方案**: 新增 `commands` 模块 + 提取 `set_webview_background_color`
- **Skill 校验**: ✅ 已完成（cfg 隔离/平台隔离/桥接规则/NAPI/线程/ArkTS/版本隔离 全部通过）
- **代码实施**: ✓ 已完成（2026-06-05）
- **当前阶段**: 待设备端验证

---

## 1. 代码修改

### 1.1 新增 `commands` 模块（无平台限制）
- [x] 在 `tauri/crates/tauri/src/webview/plugin.rs` 中新增 `mod commands`：
  - 包含 `get_webview` 辅助函数
  - 包含 `set_webview_background_color` 命令定义

### 1.2 从 `desktop_commands` 中移除重复定义
- [x] 删除 `desktop_commands` 模块内的 `setter!(set_webview_background_color, set_background_color, Option<Color>)` 调用

### 1.3 修改命令注册
- [x] 在 `generate_handler!` 宏中：
  - 将 `#[cfg(desktop)] desktop_commands::set_webview_background_color` 改为 `commands::set_webview_background_color`

---

## 2. 验证

### 2.1 编译验证
- [x] Desktop 模式 `cargo check` 通过（2026-06-05，无新增 warning）
- [ ] OHOS mobile 模式交叉编译通过

### 2.2 功能验证（设备端）
- [ ] TransparencyTest.svelte 卡片 2（"当前 WebView 设为透明"）：`webviewWindow.setBackgroundColor([0,0,0,0])` 成功，不再报 `command not found`
- [ ] 卡片 3（半透明红色）、卡片 4（半透明绿色）、卡片 5（不透明黑色）：`setBackgroundColor` 调用成功
- [ ] 卡片 6（重置为默认）：`webview.setBackgroundColor(null)` 成功
- [ ] 自动测试 `window.setBackgroundColor does not throw` 在 OHOS mobile 上通过

### 2.3 回归验证（Desktop 平台）
- [ ] Windows/macOS/Linux 上 `setBackgroundColor` 行为不变
- [ ] 其他 `desktop_commands` 命令（zoom/hide/show/print 等）不受影响

---

## 依赖关系

```
Phase 1（wry OHOS 后端 set_background_color 实现）
  └─→ 1. 代码修改（命令注册修复）
      └─→ 2. 编译验证
          └─→ 3. 功能验证（设备端）
          └─→ 4. 回归验证（Desktop）
```

---

## 预估工作量

| 模块 | 任务数 | 预估时间 |
|------|--------|----------|
| 代码修改 | 3 | 0.2h |
| 编译验证 | 2 | 0.3h |
| 功能验证 | 5 | 1h |
| 回归验证 | 2 | 0.5h |
| **总计** | **12** | **~2h** |

---

## 验收标准

1. ✅ `commands` 模块已新增，包含 `set_webview_background_color` 命令
2. ✅ `desktop_commands` 中已删除重复的 `setter!` 调用
3. ✅ 命令注册已改为无 `#[cfg(desktop)]` 门控
4. ⬜ 编译验证通过（Desktop + OHOS mobile）
5. ⬜ 设备端卡片 2-6 全部成功
6. ⬜ Desktop 回归验证通过

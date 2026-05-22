# Phase 8: popup() 设备修复 - 进度追踪

> 设计文档: [phase8-popup-root-cause-and-fix.md](phase8-popup-root-cause-and-fix.md)
> 审计文档: [phase8-popup-review.md](phase8-popup-review.md)
> 状态: ⚠️ 设计完成 — **已被 Phase 9 替代**
> 工期: 1 天

> **⚠️ 注意**：Phase 8 的 `openMenu` 修复方案因模块级 `@Builder` 无 `this` 上下文导致子菜单渲染 crash，未能实施。Phase 9 彻底重构为 `bindMenu(isShow)` + `@Component @Builder` 方案，解决了所有问题。详见 [phase9-popup-bindcontextmenu-design.md](phase9-popup-bindcontextmenu-design.md)。

---

## 进度总览

| Step | 内容 | 状态 | 文件 |
|------|------|------|------|
| 1 | MainPage 添加锚点 id | ⬚ 待做 | `MainPage.ets` |
| 2 | NativeAbility 删除 uniqueId | ⬚ 待做 | `NativeAbility.ets` |
| 3.1 | TauriMenuManager 删除 uniqueId | ⬚ 待做 | `menu.ets` |
| 3.2 | popupFromJson 改用 string id + anchorPosition | ⬚ 待做 | `menu.ets` |
| 3.3 | 修复 separator 判断 | ⬚ 待做 | `menu.ets` |
| 3.4 | handleItemClick 调用 emit_menu_event | ⬚ 待做 | `menu.ets` |
| 3.5 | PredefinedActionExecutor 添加 'close' | ⬚ 待做 | `menu.ets` + `menu_types.ets` |
| 4 | muda strip `&` 助记符 | ⬚ 待做 | `muda/.../ohos/mod.rs` |
| 5.1 | js_init_script 注入 pointerdown 监听 | ⬚ 待做 | `plugin.rs` |
| 5.2 | popup handler 坐标获取（含超时） | ⬚ 待做 | `plugin.rs` |

---

## 实施阶段

### 阶段 1: 核心修复（让菜单弹出）

| 任务 | 状态 | 备注 |
|------|------|------|
| Step 1: `.id("__tauri_internal_menu_popup_anchor__")` | ⬚ | |
| Step 2: 删除 uniqueId 生成和传递 | ⬚ | |
| Step 3.1: 构造函数删除 uniqueId 参数 | ⬚ | |
| Step 3.2: target 改 string id + anchorPosition | ⬚ | |
| Step 3.3: separator 判断改为 `type=predefined && predefinedType=separator` | ⬚ | |
| 设备验证: 菜单能弹出 | ⬚ | |

### 阶段 2: 必要补充（让菜单可用）

| 任务 | 状态 | 备注 |
|------|------|------|
| Step 3.4: handleItemClick 调用 emit_menu_event | ⬚ | 需确认 libnative_ability.so 导入可用 |
| Step 3.5: executor 添加 'close' 分支 | ⬚ | menu_types.ets 同步更新 |
| Step 4: muda `self.text.replace("&", "")` | ⬚ | |
| 设备验证: 点击有事件 + predefined 动作执行 + 文本无 & | ⬚ | |

### 阶段 3: 无坐标 popup（可后续单独验证）

| 任务 | 状态 | 备注 |
|------|------|------|
| Step 5.1: js_init_script 注入 | ⬚ | `#[cfg(target_env = "ohos")]` |
| Step 5.2: eval_with_callback + 100ms 超时 | ⬚ | |
| 设备验证: 不传坐标时菜单弹在 pointerdown 位置 | ⬚ | 如失败切换 atomic 降级方案 |

---

## 验证结果

| 测试项 | 结果 | 备注 |
|--------|------|------|
| 基本 Popup | ⬚ | |
| 文本无 & 符号 | ⬚ | |
| Separator 显示 | ⬚ | |
| Normal 点击事件 | ⬚ | |
| Check 切换事件 | ⬚ | |
| CloseWindow 执行 | ⬚ | |
| Minimize 执行 | ⬚ | |
| 无坐标 popup 位置 | ⬚ | |
| 子菜单展开 | ⬚ | |
| 点击外部关闭 | ⬚ | |
| 回归: WebView 正常 | ⬚ | |
| 回归: 触摸事件正常 | ⬚ | |

---

## 遇到的问题

（实施过程中记录）

---

## 编译验证

```bash
# OHOS 交叉编译
cargo check --target aarch64-unknown-linux-ohos -p tauri
cargo check --target aarch64-unknown-linux-ohos -p muda

# Windows 回归
cargo check -p tauri
cargo check -p muda
```

| 目标 | 状态 |
|------|------|
| aarch64-unknown-linux-ohos (tauri) | ⬚ |
| aarch64-unknown-linux-ohos (muda) | ⬚ |
| Windows (tauri) | ⬚ |
| Windows (muda) | ⬚ |

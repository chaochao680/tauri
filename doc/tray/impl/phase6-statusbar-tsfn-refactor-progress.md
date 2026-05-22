# Phase 6 进度：StatusBar TSFN 数据传递重构

> 对应设计文档：[phase6-statusbar-tsfn-refactor-design.md](./phase6-statusbar-tsfn-refactor-design.md)
> 创建时间：2026-05-18
> 更新时间：2026-05-20

---

## 一、进度概览

| 任务 | 状态 | 说明 |
|------|------|------|
| 设计文档 | ✅ done | phase6-statusbar-tsfn-refactor-design.md 已创建 |
| 新增数据结构 | ✅ done | `AddStatusBarData`, `UpdateIconData`, `UpdateMenuData`, `UpdateTipsData` 已实现 |
| 修改 TSFN 类型别名 | ✅ done | 5 个 TSFN 类型别名泛型参数已从 `()` 改为对应数据结构 |
| 删除 15 个 `DATA_*` 静态变量 | ✅ done | 全部移除，仅保留 5 个 `TSFN_*` 静态变量 |
| 修改 `init_tray_tsfn` | ✅ done | 使用 `build_threadsafe_function::<T>()` 构建，回调接收 `ThreadsafeCallContext<T>` |
| 修改 `build_*` 函数 | ✅ done | `build_add_args`, `build_update_icon_args`, `build_update_menu_args` 改为接收数据参数 |
| 修改公开 API | ✅ done | `add_to_status_bar`, `remove_from_status_bar`, `update_status_bar_icon`, `update_status_bar_menu`, `update_hover_tips` 全部改为构造 struct + TSFN call |
| 构建验证 | ✅ done | `TAURI_OHOS_DEVICE_TYPE=desktop` 交叉编译通过，HAR 打包成功 |
| 设备部署 | ✅ done | 签名 + 安装成功，应用启动正常，无 crash/ANR |
| Autotest 验证 | ✅ done | 122 项用例，117 pass / 5 fail（tray 相关 #108-#122 全部通过） |

**整体进度**：`100%` ✅ 重构完成，验证通过

---

## 二、实现变更清单

| 文件 | 变更类型 | 变更内容 |
|------|----------|----------|
| `openharmony-ability/crates/ability/src/statusbar/manager.rs` | 重写 | 删除 15 个 `DATA_*` Mutex，新增 4 个数据结构，修改 TSFN 类型别名、`init_tray_tsfn`、`build_*` 函数、公开 API |
| `openharmony-ability/crates/ability/src/statusbar/types.rs` | 无需变更 | 类型定义不变 |
| `openharmony-ability/crates/ability/src/statusbar/event.rs` | 无需变更 | 事件处理不变 |
| `openharmony-ability/crates/ability/src/statusbar/validate.rs` | 无需变更 | 参数校验不变 |
| `tray-icon/src/platform_impl/ohos/mod.rs` | 无需变更 | 上层调用方无需修改 |

---

## 三、验证结果（2026-05-18）

### 3.1 构建

| 指标 | 状态 | 说明 |
|------|------|------|
| 交叉编译 | ✅ | `TAURI_OHOS_DEVICE_TYPE=desktop` 编译通过 |
| HAR 打包 | ✅ | `ohrs build --arch arm64` → `pack.sh` → `ability.har` |
| hvigor 构建 | ✅ | API Demo HAP 构建成功（3.2s） |

### 3.2 设备部署

| 指标 | 状态 | 说明 |
|------|------|------|
| 签名 + 安装 | ✅ | 成功部署到设备 |
| 应用启动 | ✅ | 无 crash / ANR |

### 3.3 Autotest 结果

| 指标 | 数值 | 说明 |
|------|------|------|
| 总测试数 | 122 | 全量测试 |
| 通过 | 117 | 包含全部 15 项 tray 测试 |
| 失败 | 5 | 预存问题，与本次重构无关 |

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

**5 项失败均为预存问题，与本次重构无关**：

| # | Test | Error |
|---|------|-------|
| 3 | core.Channel | expected 1000 messages, got 79 |
| 21 | plugin-http.fetch | plugin http not found |
| 23 | plugin-autostart | plugin autostart not found |
| 24 | plugin-clipboard-manager.writeText | plugin clipboard-manager not found |
| 25 | plugin-clipboard-manager.writeImage | plugin clipboard-manager not found |

---

## 四、变更对比

| 维度 | 重构前 | 重构后 |
|------|--------|--------|
| 静态全局变量 | 20 个（5 TSFN + 15 DATA） | 6 个（5 TSFN + 1 PredefinedAction） |
| 原子性 | 10 次独立 lock，可能交叉 | 单次构造 struct，天然原子 |
| 数据传递 | 全局 Mutex 中转 | TSFN 内部 Box 传递 |
| 代码行数 | ~415 行 | ~463 行（含 Phase 7 新增的 PredefinedAction） |
| 扩展性 | 新增字段 = 新增 Mutex + lock/take | 新增字段 = struct 加一个字段 |

---

## 五、源码审计（2026-05-20）

### 5.1 设计文档要求 vs 实际代码

| 设计要求 | 实际状态 | 文件位置 |
|---------|---------|---------|
| `AddStatusBarData` struct | ✅ 已实现 | `manager.rs:17-28` |
| `UpdateIconData` struct | ✅ 已实现 | `manager.rs:30-34` |
| `UpdateMenuData` struct | ✅ 已实现 | `manager.rs:36-38` |
| `UpdateTipsData` struct | ✅ 已实现 | `manager.rs:40-42` |
| `PredefinedActionData` struct | ✅ 额外新增 | `manager.rs:44-46`（Phase 7 扩展） |
| TSFN 类型别名泛型修改 | ✅ 已实现 | `manager.rs:50-61` |
| 删除 15 个 `DATA_*` 静态变量 | ✅ 已删除 | 代码中不存在 |
| `init_tray_tsfn` 使用泛型 TSFN | ✅ 已实现 | `manager.rs:71-207` |
| `build_add_args` 接收数据参数 | ✅ 已实现 | `manager.rs:211-243` |
| `build_update_icon_args` 接收数据参数 | ✅ 已实现 | `manager.rs:245-254` |
| `build_update_menu_args` 接收数据参数 | ✅ 已实现 | `manager.rs:256-266` |
| `add_to_status_bar` 构造 struct + TSFN call | ✅ 已实现 | `manager.rs:376-398` |
| `remove_from_status_bar` TSFN call | ✅ 已实现 | `manager.rs:400-406` |
| `update_status_bar_icon` 构造 struct + TSFN call | ✅ 已实现 | `manager.rs:408-420` |
| `update_status_bar_menu` 构造 struct + TSFN call | ✅ 已实现 | `manager.rs:422-437` |
| `update_hover_tips` 构造 struct + TSFN call | ✅ 已实现 | `manager.rs:439-451` |
| `execute_predefined_action` TSFN call | ✅ 额外新增 | `manager.rs:453-463`（Phase 7 扩展） |

### 5.2 代码质量评估

**优点**：
- ✅ 完全消除了 15 个 `DATA_*` Mutex，减少了全局状态
- ✅ TSFN 泛型使用正确，数据结构传递符合 napi-ohos API 规范
- ✅ 公开 API 签名不变，上层调用方无需修改
- ✅ `build_menu_item_object_static` 等辅助函数保持不变
- ✅ 代码结构清晰，注释完整

**额外变更**（Phase 7 扩展）：
- `PredefinedActionData` struct（`manager.rs:44-46`）
- `TrayTsfnPredefined` 类型别名（`manager.rs:61`）
- `TSFN_PREDEFINED` 静态变量（`manager.rs:68`）
- `execute_predefined_action` 函数（`manager.rs:453-463`）

这些扩展属于 Phase 7 的工作，不影响 Phase 6 的重构目标。

---

## 六、结论

Phase 6 重构**已 100% 完成**：

1. **代码实现**：所有设计文档要求的变更均已实施
2. **构建验证**：交叉编译和 HAR 打包通过
3. **设备验证**：应用启动正常，无 crash/ANR
4. **Autotest**：tray 相关 15 项用例全部通过
5. **代码质量**：结构清晰，符合 napi-ohos 最佳实践

**重构效果**：
- 静态全局变量从 20 个减少到 6 个（减少 70%）
- 数据传递从非原子 Mutex 操作改为原子 struct 传递
- 扩展性显著提升，新增字段只需修改 struct 定义

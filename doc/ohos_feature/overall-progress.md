# Menu/Tray 模块 OHOS 空实现 - 总体进度

> 版本：v1.2
> 更新时间：2026-05-14

---

## 总览

| Phase | 状态 | 进度 |
|-------|------|------|
| Phase 0: muda 空实现 | ✅ Completed | 5/5 |
| Phase 1: tray-icon 空实现 | ✅ Completed | 8/8 |
| Phase 2: tauri 适配 | ✅ Completed | 3/3 |
| Phase 3: tauri-runtime-wry 修复 | ✅ Completed | 15/15 |
| **总计** | ✅ Completed | **31/31** |

---

## Phase 0：muda 空实现

进度详情见 [impl/phase0-muda-stub-progress.md](./impl/phase0-muda-stub-progress.md)

| # | 文件 | 状态 |
|---|------|------|
| 0.1 | `muda/Cargo.toml` 第 46-48 行 - **仅 Linux** 排除 ohos | ✅ |
| 0.2 | `muda/src/platform_impl/mod.rs` - 添加 ohos 分支 | ✅ |
| 0.3 | `muda/src/platform_impl/ohos/mod.rs` - 新建空实现 | ✅ |
| 0.4 | `muda/src/platform_impl/ohos/icon.rs` - 新建 PlatformIcon | ✅ |
| 0.5 | `muda/src/error.rs` - 添加 Unsupported | ✅ |

---

## Phase 1：tray-icon 空实现

进度详情见 [impl/phase1-tray-icon-stub-progress.md](./impl/phase1-tray-icon-stub-progress.md)

| # | 文件 | 状态 |
|---|------|------|
| 1.1 | `tray-icon/Cargo.toml` 第 44-46 行 - **仅 Linux** 排除 ohos | ✅ |
| 1.2 | `tray-icon/Cargo.toml` 第 91-92 行 - png 排除 ohos | ✅ |
| 1.3 | `tray-icon/src/platform_impl/mod.rs` - 添加 ohos 分支 | ✅ |
| 1.4 | `tray-icon/src/platform_impl/ohos/mod.rs` - 新建空实现 | ✅ |
| 1.5 | `tray-icon/src/platform_impl/ohos/icon.rs` - 新建 PlatformIcon | ✅ |
| 1.6 | `tray-icon/src/error.rs` - Unsupported + PngEncodingError 排除 | ✅ |
| 1.7 | `tray-icon/src/lib.rs` - app_indicator 排除 ohos | ✅ |
| 1.8 | `tray-icon/Cargo.toml` - muda patch | ✅ |

---

## Phase 2：tauri 适配

进度详情见 [impl/phase2-tauri-adaptation-progress.md](./impl/phase2-tauri-adaptation-progress.md)

### 已完成的前置工作

| 文件 | 状态 |
|------|------|
| `tauri/Cargo.toml` [patch] muda/tray-icon | ✅ |
| `crates/tauri/build.rs` TAURI_OHOS_DEVICE_TYPE | ✅ |
| `crates/tauri-runtime/build.rs` | ✅ |
| `crates/tauri-runtime-wry/build.rs` | ✅ |
| `crates/tauri-build/src/lib.rs` | ✅ |

### Phase 2 任务

| # | 文件 | 状态 |
|---|------|------|
| 2.1 | `crates/tauri/Cargo.toml` desktop 排除 ohos + 添加 ohos 配置 | ✅ |
| 2.2 | `crates/tauri/Cargo.toml` 版本号升级 | ✅ |
| 2.3 | `crates/tauri/Cargo.toml` 合并 ohos 配置 | ✅ |

---

## Phase 3：tauri-runtime-wry 修复

进度详情见 [impl/phase3-tauri-runtime-wry-fix-progress.md](./impl/phase3-tauri-runtime-wry-fix-progress.md)

| # | 文件 | 状态 |
|---|------|------|
| 3.1 | `tauri-runtime-wry/monitor/mod.rs` 复用 mobile 空实现 | ✅ |
| 3.2 | `tauri-runtime-wry/window/mod.rs` 复用 mobile 空实现 | ✅ |
| 3.3 | `tauri-runtime-wry/lib.rs` NewWindowOpener 排除 ohos | ✅ |
| 3.4 | `tauri/app.rs` supports_multiple_windows 排除 ohos | ✅ |
| 3.5 | `tauri/manager/menu.rs` gtk 相关排除 ohos | ✅ |
| 3.6 | `tauri/webview/mod.rs` NewWindowResponse 拆分 | ✅ |
| 3.7 | `tauri/window/mod.rs` gtk 相关排除 ohos | ✅ |
| 3.8 | `tauri/plugin.rs` mobile 模块导出 ohos | ✅ |
| 3.9 | `tauri/plugin/mobile.rs` 导入添加 ohos cfg | ✅ |
| 3.10 | `tauri/lib.rs` mobile_entry_point 导出 ohos | ✅ |
| 3.11 | `examples/api/src-tauri/src/lib.rs` new_window 排除 ohos | ✅ |

---

## 验收测试

| # | 测试 | 状态 |
|---|------|------|
| 3.1 | muda ohos 编译 | ✅ |
| 3.2 | tray-icon ohos 编译 | ✅ |
| 3.3 | tauri ohos 编译 | ✅ |
| 3.4 | HAP 安装到鸿蒙 PC | ✅ |
| 3.5 | autotest 验收 | ✅ (36/41 通过) |

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

- **Phase 3 完成**：tauri-runtime-wry OHOS 编译修复完成
  - 关键修改：MonitorExt/WindowExt 复用 mobile 空实现
  - NewWindowOpener/NewWindowResponse 排除 ohos desktop
  - plugin::mobile 模块和 mobile_entry_point 导出 ohos
  - gtk 相关代码排除 ohos
- **端到端验证通过**：36/41 测试通过 (87.8%)
- **Phase 2 完成**：tauri 适配完成并通过编译验证
  - 编译命令：`cargo build -p tauri --target aarch64-unknown-linux-ohos`
  - 编译耗时：42.51s
  - 有 2 个 tauri warnings（unused_mut/unused_variable）
  - 关键修改：desktop 排除 ohos + ohos 单独配置（不带 gtk）
  - 版本号升级：muda 0.19 / tray-icon 0.24
- **Phase 1 完成**：tray-icon 空实现全部完成并通过编译验证
  - 编译命令：`cargo build -p tray-icon --target aarch64-unknown-linux-ohos --no-default-features`
  - 有 3 个 warnings（unused fields/method），属于空实现正常情况
  - 额外修改：error.rs PngEncodingError 排除 ohos，lib.rs app_indicator 排除 ohos，Cargo.toml 添加 muda patch
- **Phase 0 完成**：muda 空实现全部完成并通过编译验证
  - 编译命令：`cargo build -p muda --target aarch64-unknown-linux-ohos --no-default-features`
  - 有 6 个 warnings（unused imports/fields），属于空实现正常情况
- 确认 Windows/macOS 与 ohos 互斥，不需要排除
- 仅 Linux 依赖需要添加 `not(target_env = "ohos")`
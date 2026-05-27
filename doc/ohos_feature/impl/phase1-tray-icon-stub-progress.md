# Phase 1: tray-icon 空实现进度

> 版本：v1.1
> 更新时间：2026-05-14

---

## 总览

| 状态 | 数量 |
|------|------|
| ⬜ Not Started | 0 |
| 🔄 In Progress | 0 |
| ✅ Completed | 6 |

---

## 任务清单

| # | 文件 | 修改内容 | 状态 |
|---|------|----------|------|
| 1.1 | `tray-icon/Cargo.toml` 第 44-46 行 | **仅 Linux 依赖**添加 `not(target_env = "ohos")` | ✅ |
| 1.2 | `tray-icon/Cargo.toml` 第 91-92 行 | png 依赖添加 `not(target_env = "ohos")` | ✅ |
| 1.3 | `tray-icon/src/platform_impl/mod.rs` | 添加 ohos 分支并排除 Linux | ✅ |
| 1.4 | `tray-icon/src/platform_impl/ohos/mod.rs` | **新建**空实现 | ✅ |
| 1.5 | `tray-icon/src/platform_impl/ohos/icon.rs` | **新建**PlatformIcon 空实现 | ✅ |
| 1.6 | `tray-icon/src/error.rs` | 添加 Unsupported + PngEncodingError 排除 | ✅ |
| 1.7 | `tray-icon/src/lib.rs` | app_indicator 排除 ohos | ✅ |
| 1.8 | `tray-icon/Cargo.toml` | 添加 muda patch | ✅ |

---

## 注意事项

- Windows/macOS 依赖**不需要修改**（与 ohos 互斥）
- 仅 Linux 和 png 依赖需要排除 ohos
- error.rs PngEncodingError 需拆分为两个 cfg（Linux+排除ohos，macOS）
- lib.rs app_indicator 需排除 ohos

---

## 验证测试

| # | 测试 | 状态 | 命令 |
|---|------|------|------|
| V1.1 | tray-icon ohos 编译 | ✅ | `cargo build -p tray-icon --target aarch64-unknown-linux-ohos --no-default-features` |

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

- 创建进度文档
- 确认 Windows/macOS 依赖不需要修改
- ✅ 修改 Cargo.toml Linux/png 依赖排除 ohos
- ✅ 修改 Cargo.toml dev-dependencies 排除 ohos
- ✅ 添加 platform_impl/mod.rs ohos 分支 + Linux 排除 ohos
- ✅ 创建 ohos/mod.rs 空实现（TrayIcon）
- ✅ 创建 ohos/icon.rs 空实现（PlatformIcon）
- ✅ 修改 error.rs 添加 Unsupported + PngEncodingError 排除 ohos
- ✅ 修改 lib.rs app_indicator 排除 ohos
- ✅ 添加 Cargo.toml muda patch
- ✅ 编译验证通过（有 3 个 warnings，属于空实现正常情况）
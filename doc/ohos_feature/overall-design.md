# Menu/Tray 模块 OHOS 空实现 - 总体设计

> 版本：v1.0
> 更新时间：2026-05-14
> 审计状态：已审计

---

## 一、方案概述

**方案 B**：空实现在 muda/tray-icon 底层，tauri 层无需修改源码。

---

## 二、编译目标分析

ohos 编译目标：`aarch64-unknown-linux-ohos`
- `target_os = "linux"`（ohos 基于 linux）
- `target_env = "ohos"`（环境标识）

**关键结论**：

| 平台 | 与 ohos 是否互斥 | Cargo.toml 是否需要排除 |
|------|-----------------|------------------------|
| Windows | ✅ 互斥（`target_os = "windows"`） | ❌ 不需要 |
| macOS | ✅ 互斥（`target_os = "macos"`） | ❌ 不需要 |
| Linux | ❌ 不互斥（`target_os = "linux"`） | ✅ **需要排除** |

---

## 三、文件修改总清单

### 3.1 muda（仅 1 处 Cargo.toml 修改）

| 文件 | 修改内容 |
|------|----------|
| `Cargo.toml` 第 46-48 行 | **仅 Linux 依赖**添加 `not(target_env = "ohos")` |
| `src/platform_impl/mod.rs` | 添加 ohos 分支 |
| `src/platform_impl/ohos/mod.rs` | 新建空实现 |
| `src/error.rs` | 添加 Unsupported 错误 |

**注意**：Windows/macOS 依赖不需要修改。

### 3.2 tray-icon（仅 2 处 Cargo.toml 修改）

| 文件 | 修改内容 |
|------|----------|
| `Cargo.toml` 第 44-46 行 | **仅 Linux 依赖**添加 `not(target_env = "ohos")` |
| `Cargo.toml` 第 91-92 行 | png 依赖添加 `not(target_env = "ohos")` |
| `src/platform_impl/mod.rs` | 添加 ohos 分支 |
| `src/platform_impl/ohos/mod.rs` | 新建空实现 |
| `src/error.rs` | 添加 Unsupported 错误 |

**注意**：Windows/macOS 依赖不需要修改。

### 3.3 tauri（已部分完成）

| 文件 | 修改内容 | 状态 |
|------|----------|------|
| `Cargo.toml` [patch] | 添加 muda/tray-icon patch | ✅ 已完成 |
| `crates/tauri/Cargo.toml` 第 87-95 行 | 移除 desktop 依赖的 ohos 排除 | ⬜ 待完成 |
| `crates/tauri/build.rs` | TAURI_OHOS_DEVICE_TYPE 支持 | ✅ 已完成 |

---

## 四、验证流程

```bash
# Phase 0 验证
cargo build -p muda --target aarch64-unknown-linux-ohos --no-default-features

# Phase 1 验证
cargo build -p tray-icon --target aarch64-unknown-linux-ohos --no-default-features

# Phase 2 验证
cargo build -p tauri --target aarch64-unknown-linux-ohos

# 最终验收
bash D:/workspace/tauri/tauri/.claude/skills/ohos-build/scripts/build-ohos.sh "" desktop
hdc install entry-default-signed.hap
```
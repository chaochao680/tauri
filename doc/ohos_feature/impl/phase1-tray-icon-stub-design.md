# Phase 1: tray-icon 空实现设计

> 版本：v1.0
> 更新时间：2026-05-14
> 目标：在本地 tray-icon crate 添加 ohos 平台空实现

---

## 一、编译目标分析

ohos 编译目标：`aarch64-unknown-linux-ohos`
- `target_os = "linux"`（ohos 基于 linux）
- `target_env = "ohos"`（环境标识）

**关键**：`target_os = "linux"` 和 `target_env = "ohos"` 不互斥！

---

## 二、需要修改的文件

| 文件 | 位置 | 修改内容 |
|------|------|----------|
| Cargo.toml | `D:\workspace\tauri\tray-icon\Cargo.toml` | **仅 Linux 依赖**添加 `not(target_env = "ohos")` |
| platform_impl/mod.rs | `D:\workspace\tauri\tray-icon\src\platform_impl\mod.rs` | 添加 ohos 分支 |
| platform_impl/ohos/mod.rs | `D:\workspace\tauri\tray-icon\src\platform_impl\ohos\mod.rs` | **新建**空实现 |
| error.rs | `D:\workspace\tauri\tray-icon\src\error.rs` | 添加 Unsupported 错误 |

---

## 三、Cargo.toml 修改

### 3.1 Windows 依赖 - **不需要修改**

```toml
# 保持不变，target_os = "windows" 与 ohos 互斥
[target."cfg(target_os = \"windows\")".dependencies]
```

### 3.2 macOS 依赖 - **不需要修改**

```toml
# 保持不变，target_os = "macos" 与 ohos 互斥
[target."cfg(target_os = \"macos\")".dependencies]
```

### 3.3 Linux 依赖 - **需要修改**

```toml
# 当前（第 44-46 行）
[target."cfg(any(target_os = \"linux\", target_os = \"dragonfly\",
                 target_os = \"freebsd\", target_os = \"netbsd\",
                 target_os = \"openbsd\"))".dependencies]
libappindicator = { version = "0.9", optional = true }
dirs = "6"

# 修改后（添加 not(target_env = "ohos")）
[target.'cfg(all(any(target_os = "linux", target_os = "dragonfly",
                      target_os = "freebsd", target_os = "netbsd",
                      target_os = "openbsd"), not(target_env = "ohos")))'.dependencies]
libappindicator = { version = "0.9", optional = true }
dirs = "6"
```

### 3.4 png 依赖 - **需要修改**

```toml
# 当前（第 91-92 行）
[target."cfg(any(target_os = \"linux\", target_os = \"macos\", ...))".dependencies]
png = "0.18"

# 修改后（添加 not(target_env = "ohos")）
[target.'cfg(all(any(target_os = "linux", target_os = "macos",
                      target_os = "dragonfly", target_os = "freebsd",
                      target_os = "netbsd", target_os = "openbsd"),
                  not(target_env = "ohos")))'.dependencies]
png = "0.18"
```

---

## 四、platform_impl/mod.rs 修改

文件：`D:\workspace\tauri\tray-icon\src\platform_impl\mod.rs`

添加：

```rust
#[cfg(target_env = "ohos")]
#[path = "ohos/mod.rs"]
mod platform;
```

---

## 五、新建 platform_impl/ohos/mod.rs

文件：`D:\workspace\tauri\tray-icon\src\platform_impl\ohos\mod.rs`

需要导出的类型：

| 类型 | 说明 |
|------|------|
| `TrayIcon` | 托盘图标内部实现 |
| `PlatformIcon` | 图标 stub |

所有操作方法返回 `crate::Error::Unsupported`。

---

## 六、error.rs 修改

文件：`D:\workspace\tauri\tray-icon\src\error.rs`

添加：

```rust
#[cfg(target_env = "ohos")]
#[error("Operation not supported on OpenHarmony")]
Unsupported,
```

---

## 七、验证

```bash
cargo build -p tray-icon --target aarch64-unknown-linux-ohos --no-default-features
```
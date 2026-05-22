# Phase 2: tauri 适配设计

> 版本：v1.0
> 更新时间：2026-05-14
> 目标：移除 tauri/Cargo.toml 中对 muda/tray-icon 的 ohos 排除条件

---

## 一、已完成的修改（不需要回退）

### 1.1 Cargo.toml patch（已完成）

```toml
# tauri/Cargo.toml [patch.crates-io]（已添加）
muda = { path = "../muda" }
tray-icon = { path = "../tray-icon" }
```

### 1.2 build.rs TAURI_OHOS_DEVICE_TYPE 支持（已完成）

| 文件 | 修改 |
|------|------|
| `crates/tauri/build.rs` | 支持 `TAURI_OHOS_DEVICE_TYPE` 环境变量 |
| `crates/tauri-runtime/build.rs` | 同上 |
| `crates/tauri-runtime-wry/build.rs` | 同上 |
| `crates/tauri-build/src/lib.rs` | 同上 |

---

## 二、Phase 2 需要做的修改

### 2.1 crates/tauri/Cargo.toml（第 87-95 行）

**当前**：
```toml
[target.'cfg(all(any(target_os = "linux", target_os = "dragonfly", 
                     target_os = "freebsd", target_os = "openbsd", 
                     target_os = "netbsd", target_os = "windows", 
                     target_os = "macos"), not(target_env = "ohos")))'.dependencies]
muda = { version = "0.17", default-features = false, features = ["serde", "gtk"] }
tray-icon = { version = "0.22", default-features = false, features = ["serde"], optional = true }
```

**修改后**（移除 `all(..., not(target_env = "ohos"))` 改为 `any(...)`）：
```toml
[target.'cfg(any(target_os = "linux", target_os = "dragonfly", 
                  target_os = "freebsd", target_os = "openbsd", 
                  target_os = "netbsd", target_os = "windows", 
                  target_os = "macos"))'.dependencies]
muda = { version = "0.17", default-features = false, features = ["serde", "gtk"] }
tray-icon = { version = "0.22", default-features = false, features = ["serde"], optional = true }
```

---

## 三、不需要修改的部分

### 3.1 linux 依赖（第 97-100 行）- 保持不变

```toml
[target.'cfg(all(any(target_os = "linux", ...), not(target_env = "ohos")))'.dependencies]
gtk = { version = "0.18", features = ["v3_24"] }
webkit2gtk = { version = "=2.0", features = ["v2_40"], optional = true }
```

ohos 不需要 gtk/webkit2gtk，保持排除。

### 3.2 源码 - 不需要修改

tauri 的 menu/tray 代码是通用的，调用 muda/tray-icon 空实现即可。

---

## 四、修改清单

| 文件 | 位置 | 修改内容 | 状态 |
|------|------|----------|------|
| `crates/tauri/Cargo.toml` | 第 87-95 行 | 移除 `not(target_env = "ohos")` | ⬜ |

---

## 五、验证

```bash
# 编译测试
cargo build -p tauri --target aarch64-unknown-linux-ohos

# 最终验收
bash D:/workspace/tauri/tauri/.claude/skills/ohos-build/scripts/build-ohos.sh "" desktop
hdc install entry-default-signed.hap
```
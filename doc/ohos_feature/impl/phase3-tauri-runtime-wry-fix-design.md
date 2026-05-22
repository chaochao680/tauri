# Phase 3: tauri-runtime-wry OHOS 编译修复设计

> 版本：v2.0
> 更新时间：2026-05-14
> 目标：修复 tauri 全栈在 ohos desktop 编译时的错误

---

## 一、问题分析

### 1.1 编译目标

ohos desktop 编译（`TAURI_OHOS_DEVICE_TYPE=desktop`）：
- `cfg(desktop)` = true（由 build.rs 定义）
- `cfg(mobile)` = false
- `target_env = "ohos"`

### 1.2 编译阶段发现的错误

| # | 错误 | 位置 | 原因 |
|---|------|------|------|
| 1 | `MonitorHandle.work_area()` | tauri-runtime-wry/monitor/mod.rs | MonitorExt trait 无实现 |
| 2 | `Window.is_enabled()` | tauri-runtime-wry/lib.rs | WindowExt trait 无实现 |
| 3 | `Window.center()` | tauri-runtime-wry/lib.rs | WindowExt trait 无实现 |
| 4 | `Window.set_enabled()` | tauri-runtime-wry/lib.rs | WindowExt trait 无实现 |
| 5 | `NewWindowOpener.webview` field | tauri-runtime-wry/lib.rs:4955 | tauri-runtime 结构体无 ohos 字段 |
| 6 | `NewWindowResponse::Create` variant | tauri-runtime-wry/lib.rs:4966 | tauri-runtime enum 无 ohos variant |
| 7 | `supports_multiple_windows` duplicate definitions | tauri/app.rs:670+1109 | desktop cfg 包含 ohos 导致冲突 |
| 8 | `init_for_gtk_window` method not found | tauri/manager/menu.rs:90 | gtk 方法在 ohos 无实现 |
| 9 | `gtk_window` field not found | tauri/window/mod.rs | RawWindow 无 gtk 字段 |
| 10 | `NewWindowResponse::Create` not found | tauri/webview/mod.rs:716 | enum 拆分后 match 不匹配 |
| 11 | `plugin::mobile` module not found | tauri-plugin-shell/error.rs | mobile 模块未导出 ohos |
| 12 | `run_mobile_plugin` method not found | tauri-plugin-sample/mobile.rs | PluginHandle 无方法 |
| 13 | `oneshot` crate not found | tauri/plugin/mobile.rs:297 | 导入未包含 ohos |
| 14 | `mobile_entry_point` not found | api/src-tauri/src/lib.rs:45 | 宏未导出 ohos |
| 15 | `NewWindowResponse::Create` in api | api/src-tauri/src/lib.rs:206 | 使用 Create variant |

### 1.3 根因分析

#### 问题 1-4: MonitorExt / WindowExt trait

**现状**：
```rust
// tauri-runtime-wry/monitor/mod.rs
#[cfg(mobile)]  // mobile = Android/iOS，ohos desktop 不生效
impl MonitorExt for tao::monitor::MonitorHandle { ... }

// tauri-runtime-wry/window/mod.rs
#[cfg(mobile)]
impl WindowExt for tao::window::Window { ... }
```

**问题**：
- ohos desktop: `cfg(mobile)=false`，空实现不生效
- linux gtk 实现被 `not(target_env = "ohos")` 排除
- **结果**：无 trait 实现

#### 问题 5-6: NewWindowOpener / NewWindowResponse

**tauri-runtime/src/webview.rs**：
```rust
pub struct NewWindowOpener {
  #[cfg(all(any(target_os = "linux", ...), not(target_env = "ohos")))]
  pub webview: webkit2gtk::WebView,  // ohos 无此字段
}

pub enum NewWindowResponse {
  #[cfg(not(any(target_os = "android", target_os = "ios", target_env = "ohos")))]
  Create { window_id: WindowId },  // ohos 无此 variant
}
```

**tauri-runtime-wry/lib.rs** 使用 `#[cfg(desktop)]` 编译代码，但 ohos desktop 时结构体无对应字段。

#### 问题 7-10: tauri crate gtk 相关

**tauri/app.rs**：
```rust
#[cfg(desktop)]  // ohos desktop 也匹配，与 ohos 专用的 impl 冲突
pub fn supports_multiple_windows(&self) -> bool { true }
```

**tauri/manager/menu.rs / window/mod.rs**：
```rust
#[cfg(any(target_os = "linux", ...))]  // ohos 基于 linux，匹配此 cfg
let _ = menu.inner().init_for_gtk_window(raw.gtk_window, ...);  // 但 gtk 方法不存在
```

#### 问题 11-14: plugin::mobile 模块

**tauri/plugin.rs**：
```rust
#[cfg(mobile)]  // ohos desktop 不匹配
pub mod mobile;
```

**tauri/lib.rs**：
```rust
#[cfg(mobile)]
pub use tauri_macros::mobile_entry_point;  // ohos desktop 无法使用
```

---

## 二、解决方案

### 2.1 核心策略：复用 mobile 空实现 + 排除 gtk 相关

**理由**：
1. ohos 窗口管理能力与 mobile 相似（tao ohos 实现验证）
2. 最小改动原则：只修改 cfg 条件
3. 统一行为：ohos mobile/desktop 共享实现

### 2.2 分层修改

| 层级 | crate | 修改类型 |
|------|-------|----------|
| 运行时 | tauri-runtime-wry | 复用 mobile 空实现 + 排除 NewWindow |
| 核心 | tauri | 排除 gtk 相关 + 导出 mobile 模块 |
| 示例 | examples/api | 排除 new_window handler |

---

## 三、修改清单

### 3.1 tauri-runtime-wry

| # | 文件 | 修改 |
|---|------|------|
| 1 | `monitor/mod.rs:32` | `#[cfg(mobile)]` → `#[cfg(any(mobile, target_env = "ohos"))]` |
| 2 | `window/mod.rs:57` | 同上 |
| 3 | `lib.rs:4942` | `#[cfg(desktop)]` → `#[cfg(all(desktop, not(target_env = "ohos")))]` |
| 4 | `lib.rs:4954` | 同上 |
| 5 | `lib.rs:4965` | 同上 |
| 6 | `lib.rs:4977` | 同上 |

### 3.2 tauri

| # | 文件 | 修改 |
|---|------|------|
| 7 | `app.rs:1108` | `#[cfg(desktop)]` → `#[cfg(all(desktop, not(target_env = "ohos")))]` |
| 8 | `manager/menu.rs:81` | 同上 |
| 9 | `webview/mod.rs:242` | NewWindowResponse enum 拆分（ohos 只有 Allow/Deny） |
| 10 | `webview/mod.rs:710` | match 分支添加 cfg |
| 11 | `window/mod.rs:1346` | gtk 相关代码添加 `not(target_env = "ohos")` |
| 12 | `window/mod.rs:1378` | 同上 |
| 13 | `window/mod.rs:1412` | 同上 |
| 14 | `window/mod.rs:1441` | 同上 |
| 15 | `plugin.rs:33` | `#[cfg(mobile)]` → `#[cfg(any(mobile, target_env = "ohos"))]` |
| 16 | `plugin/mobile.rs:14` | oneshot 导入添加 ohos cfg |
| 17 | `plugin/mobile.rs:35` | PENDING_PLUGIN_CALLS_ID 添加 ohos cfg |
| 18 | `lib.rs:78` | mobile_entry_point 导出添加 ohos |

### 3.3 examples/api

| # | 文件 | 修改 |
|---|------|------|
| 19 | `src-tauri/src/lib.rs:179` | `#[cfg(all(desktop, not(test)))]` → 添加 `not(target_env = "ohos")` |

---

## 四、关键代码示例

### 4.1 NewWindowResponse 拆分

```rust
// tauri/webview/mod.rs

#[cfg(not(target_env = "ohos"))]
pub enum NewWindowResponse<R: Runtime> {
  Allow,
  Create { window: crate::WebviewWindow<R> },
  Deny,
}

#[cfg(target_env = "ohos")]
pub enum NewWindowResponse<R: Runtime> {
  Allow(std::marker::PhantomData<R>),
  Deny,
}
```

### 4.2 match 分支处理

```rust
// tauri/webview/mod.rs

match handler(url, features) {
  #[cfg(not(target_env = "ohos"))]
  NewWindowResponse::Allow => tauri_runtime::webview::NewWindowResponse::Allow,
  #[cfg(target_env = "ohos")]
  NewWindowResponse::Allow(_) => tauri_runtime::webview::NewWindowResponse::Allow,
  #[cfg(all(desktop, not(target_env = "ohos")))]
  NewWindowResponse::Create { window } => {
    tauri_runtime::webview::NewWindowResponse::Create {
      window_id: window.window.window.id,
    }
  }
  NewWindowResponse::Deny => tauri_runtime::webview::NewWindowResponse::Deny,
}
```

---

## 五、验证

```bash
# 编译验证
cargo build -p tauri --target aarch64-unknown-linux-ohos

# 端到端验证
bash D:/workspace/tauri/tauri/.claude/skills/ohos-build/scripts/run-tests.sh "" desktop
```

**结果**：36/41 测试通过 (87.8%)

---

## 六、注意事项

1. ohos desktop 的 `cfg(desktop)` 为 true，但很多桌面功能不支持
2. gtk 相关代码必须排除 ohos（`not(target_env = "ohos")`）
3. mobile 模块和 mobile_entry_point 需要导出给 ohos
4. NewWindowResponse 在 ohos 只有 Allow/Deny，无 Create
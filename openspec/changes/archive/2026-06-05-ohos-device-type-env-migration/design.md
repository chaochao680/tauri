## OHOS 设备形态检测机制：从 mobile 到 desktop

### 起点：OHOS 默认是 mobile

上游 Tauri 将 OHOS 与 iOS、Android 一视同仁——全部归为 mobile。这在 `tauri-plugin::setup()` 中体现得最为直接：

```rust
// 上游 setup() 的判断（我们的修改前）
let mobile = target_os == "ios" || target_os == "android" || target_env == "ohos";
//                                                            ^^^^^^^^^^^^^^^^^^^^
//                                                            OHOS 无条件归入 mobile
```

这个判断决定了每个 crate 的 `build.rs` 最终输出哪个 cfg alias：

```rust
alias("desktop", !mobile);  // mobile=true → 不输出 cfg(desktop)
alias("mobile", mobile);    // mobile=true → 输出 cargo:rustc-cfg=mobile
```

结果：OHOS 构建时，所有 `#[cfg(mobile)]` 代码块被编译，所有 `#[cfg(desktop)]` 代码块被排除。tray、menu bar 等 PC 功能不可用。

### 问题：鸿蒙 PC 需要 desktop 代码路径

鸿蒙不仅有手机/平板，还有 PC。PC 上需要 tray、menu bar 等 desktop 功能，这些代码都被 `#[cfg(desktop)]` gate 保护着。

但 `target_os`/`target_env` 无法区分鸿蒙 PC 和手机——两者的 target 三元组完全相同（`target_os="linux"`, `target_env="ohos"`）。我们需要一个额外的信号来告诉构建系统："这次编译的目标是 PC"。

### 解决方案：用环境变量覆盖默认判定

我们在 OHOS 分支中插入一个环境变量检查，用 `OHOS_DEVICE_TYPE` 的值覆盖上游的"OHOS 一律 mobile"判定：

```rust
// 我们修改后的判断
let mobile = if target_env == "ohos" {
    // 读取环境变量，未设置时默认为 "mobile"（保持上游行为）
    let device_type = std::env::var("OHOS_DEVICE_TYPE")
        .unwrap_or_else(|_| "mobile".to_string());
    // 只有明确设为 "desktop" 才翻转为非 mobile
    device_type != "desktop"
} else {
    target_os == "ios" || target_os == "android"
};
```

对比：

| 场景 | 上游行为 | 我们的行为 |
|------|---------|-----------|
| OHOS，未设置环境变量 | mobile | mobile（兼容上游） |
| OHOS，`OHOS_DEVICE_TYPE=mobile` | mobile | mobile |
| OHOS，`OHOS_DEVICE_TYPE=desktop` | mobile | **desktop**（翻转） |
| iOS | mobile | mobile（不变） |
| Windows/macOS | desktop | desktop（不变） |

**核心逻辑**：环境变量是唯一能将 OHOS 从 mobile 翻转为 desktop 的开关。不设置时保持上游默认的 mobile 行为，完全向后兼容。

### 翻转如何传播到编译产物

整个机制是一条四步的**编译期转换链**：

```
OHOS_DEVICE_TYPE=desktop
        │
        ▼
  ┌─ Step 1: 环境变量注入 ─────────────────────────────────────┐
  │  CLI: set_var("OHOS_DEVICE_TYPE", "desktop")              │
  │  或: export OHOS_DEVICE_TYPE=desktop                       │
  └────────────────────────────────────────────────────────────┘
        │
        ▼
  ┌─ Step 2: build.rs 读取环境变量，翻转 mobile 变量 ─────────┐
  │  device_type = "desktop"                                   │
  │  mobile = ("desktop" != "desktop") = false  ← 翻转！      │
  │  alias("desktop", !false) → cargo:rustc-cfg=desktop       │
  │  alias("mobile", false)   → 不输出                         │
  └────────────────────────────────────────────────────────────┘
        │
        ▼
  ┌─ Step 3: cfg(desktop) 生效 ──────────────────────────────┐
  │  #[cfg(desktop)] fn create_tray_icon() → 被编译 ✓         │
  │  #[cfg(mobile)]  fn setup_mobile_ui()  → 被排除 ✗         │
  └────────────────────────────────────────────────────────────┘
        │
        ▼
  ┌─ Step 4: 最终编译产物 ───────────────────────────────────┐
  │  OHOS PC 版：tray + menu bar + 窗口装饰                    │
  └────────────────────────────────────────────────────────────┘
```

### Step 1 详解：环境变量从哪来

三种入口，最终效果相同——在 cargo 子进程的环境中设置 `OHOS_DEVICE_TYPE`：

**① CLI `--device-type` 参数（最常用）**

```
cargo tauri ohos build --device-type desktop
                         │
                         ▼
```

```rust
// tauri-cli/src/mobile/open_harmony/build.rs
#[clap(long, default_value = "mobile", value_parser(["mobile", "desktop"]))]
pub device_type: String,

// 解析后立即注入环境变量
set_var("OHOS_DEVICE_TYPE", &options.device_type);
```

同时写入 `CliOptions.vars`，在 `mod.rs` 中再次 `set_var` 确保后续所有 cargo 子进程可访问：

```rust
// tauri-cli/src/mobile/open_harmony/mod.rs
if let Some(device_type) = cli_options.vars.get("OHOS_DEVICE_TYPE") {
    set_var("OHOS_DEVICE_TYPE", device_type);
}
```

**② 手动 export + cargo build**

```bash
export OHOS_DEVICE_TYPE=desktop
cargo build --target aarch64-unknown-linux-ohos
```

**③ 构建脚本/CI 内联**

```bash
OHOS_DEVICE_TYPE=desktop cargo build --target aarch64-unknown-linux-ohos
```

### Step 2 详解：build.rs 中的翻转逻辑

每个需要区分设备形态的 crate 在 `build.rs` 中执行相同的模式：

```rust
fn alias(alias: &str, has_feature: bool) {
    println!("cargo:rustc-check-cfg=cfg({alias})");
    if has_feature {
        println!("cargo:rustc-cfg={alias}");
    }
}

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    let mobile = if target_env == "ohos" {
        // ① 防止 cargo 缓存：环境变量变化时重新执行 build script
        println!("cargo:rerun-if-env-changed=OHOS_DEVICE_TYPE");

        // ② 读取环境变量，默认 "mobile"（与上游行为一致）
        let device_type = std::env::var("OHOS_DEVICE_TYPE")
            .unwrap_or_else(|_| "mobile".to_string());

        // ③ 翻转判定：只有 "desktop" 才输出 false
        device_type != "desktop"
    } else {
        target_os == "ios" || target_os == "android"
    };

    // ④ 输出 cfg alias（mobile=false 时 desktop=true）
    alias("desktop", !mobile);
    alias("mobile", mobile);
}
```

**这段代码存在于以下所有 crate 的 build.rs 中**（确保每个 crate 都能独立做出正确判断）：

| crate | 文件 |
|-------|------|
| tauri | `crates/tauri/build.rs` |
| tauri-runtime | `crates/tauri-runtime/build.rs` |
| tauri-runtime-wry | `crates/tauri-runtime-wry/build.rs` |
| tauri-build | `crates/tauri-build/src/lib.rs` |
| openharmony-ability | `openharmony-ability/crates/ability/build.rs` |
| opener 插件 | `plugins/opener/build.rs` |
| shell 插件 | `plugins/shell/build.rs` |
| updater 插件 | `plugins/updater/build.rs` |

### Step 3 详解：Plugin Layer — setup() 让所有插件自动翻转

`tauri-plugin::setup()` 是每个使用 `tauri_plugin::Builder` 的插件的共享构建入口。**上游实现将所有 OHOS 无条件视为 mobile**。我们对 `setup()` 做了同样的修改：

```rust
// tauri-plugin/src/build/mobile.rs :: setup()
// 上游版本：
//   let mobile = target_os == "ios" || target_os == "android" || target_env == "ohos";
// 我们的版本：
let mobile = if target_env == "ohos" {
    println!("cargo:rerun-if-env-changed=OHOS_DEVICE_TYPE");
    let device_type = std::env::var("OHOS_DEVICE_TYPE")
        .unwrap_or_else(|_| "mobile".to_string());
    device_type != "desktop"
} else {
    target_os == "ios" || target_os == "android"
};
cfg_alias("desktop", !mobile);
cfg_alias("mobile", mobile);
```

**这是最关键的一处修改**。没有它，即使核心 crate 翻转成了 desktop，插件仍然会是 mobile（因为 setup() 会把 OHOS 强制设为 mobile）。有了它，`OHOS_DEVICE_TYPE=desktop` 会自动传播到所有插件，无需每个插件单独处理。

### OHOS desktop 的边界：cfg(desktop) ≠ 真正的桌面平台

翻转为 `cfg(desktop)=true` 后，OHOS PC 版能编译 tray、menu bar 等代码。但 OHOS 仍然是鸿蒙系统，不是 Windows/macOS。某些 desktop 原生库（如 `rfd`）在 OHOS 上不存在。因此需要**双重条件** cfg gate 来处理这些边界：

```rust
// dialog/models.rs — OHOS 不参与 rfd 转换（即使 cfg(desktop) 为 true）
#[cfg(all(desktop, not(target_env = "ohos")))]
impl From<rfd::MessageDialogResult> for MessageDialogResult { ... }

// dialog/error.rs — OHOS 始终走 mobile plugin pathway（即使 cfg(desktop) 为 true）
#[cfg(any(mobile, target_env = "ohos"))]
PluginInvoke(#[from] tauri::plugin::mobile::PluginInvokeError),
```

这类 cfg gate 是翻转的**必要补充**：它们保护那些"虽然是 desktop 功能但依赖原生桌面库"的代码路径，防止在 OHOS desktop 模式下误编译。

### `rerun-if-env-changed` 的必要性

Cargo 会缓存 build script 的输出。如果连续两次构建只有 `OHOS_DEVICE_TYPE` 不同（如从 `mobile` 切换到 `desktop`），没有此声明时 cargo 会复用缓存的 `cfg(mobile)`，导致翻转失败。

每个读取 `OHOS_DEVICE_TYPE` 的 build script 都必须输出：
```
cargo:rerun-if-env-changed=OHOS_DEVICE_TYPE
```

### 完整文件清单

| 层级 | 文件 | 职责 |
|------|------|------|
| CLI 注入 | `tauri-cli/src/mobile/open_harmony/build.rs` | `--device-type` → `set_var` |
| CLI 注入 | `tauri-cli/src/mobile/open_harmony/dev.rs` | `--device-type` → `set_var` |
| CLI 传播 | `tauri-cli/src/mobile/open_harmony/mod.rs` | `CliOptions.vars` → `set_var`，过滤 cargo args |
| Build Layer | `crates/tauri/build.rs` | env → cfg alias |
| Build Layer | `crates/tauri-runtime/build.rs` | env → cfg alias |
| Build Layer | `crates/tauri-runtime-wry/build.rs` | env → cfg alias |
| Build Layer | `crates/tauri-build/src/lib.rs` | env → cfg alias |
| Build Layer | `openharmony-ability/crates/ability/build.rs` | env → cfg alias |
| Plugin Layer | `tauri-plugin/src/build/mobile.rs` | `setup()` env → cfg alias（影响所有插件） |
| Plugin Layer | `plugins/opener/build.rs` | env → cfg alias |
| Plugin Layer | `plugins/shell/build.rs` | env → cfg alias |
| Plugin Layer | `plugins/updater/build.rs` | env → cfg alias |
| cfg Gate 修正 | `plugins/dialog/src/models.rs` | 排除 OHOS 的 rfd 路径 |
| cfg Gate 修正 | `plugins/dialog/src/error.rs` | OHOS 始终走 mobile plugin pathway |

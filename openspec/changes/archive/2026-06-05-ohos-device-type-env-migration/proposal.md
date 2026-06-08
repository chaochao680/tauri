## Why

上游 Tauri 将所有 OHOS 设备统一视为 mobile（与 iOS、Android 同等）。但鸿蒙不仅有手机/平板，还有 PC。PC 上需要 tray、menu bar 等 desktop 功能，这些代码都被 `#[cfg(desktop)]` gate 保护着。

由于 OHOS 的 `target_os="linux"`, `target_env="ohos"` 在 PC 和手机上完全相同，无法从 target 信息区分设备形态。我们需要一个额外的信号来告诉构建系统："这次编译的目标是 PC"。

`OHOS_DEVICE_TYPE` 环境变量就是这个信号。本次变更将此前散落在各处的 `TAURI_OHOS_DEVICE_TYPE`（仅在 openharmony-ability 中使用）统一为标准化的 `OHOS_DEVICE_TYPE`，并确保所有 crate 的 build.rs、`setup()` 函数、CLI 工具链都一致地读取它，从而实现 OHOS 设备形态的编译期切换。

## What Changes

### 环境变量标准化
- `TAURI_OHOS_DEVICE_TYPE` → `OHOS_DEVICE_TYPE`（去除 `TAURI_` 前缀，更简洁）

### Build Scripts（cfg alias 发射）
所有需要区分设备形态的 crate 的 build.rs 统一读取 `OHOS_DEVICE_TYPE`：
- `crates/tauri/build.rs`
- `crates/tauri-runtime/build.rs`
- `crates/tauri-runtime-wry/build.rs`
- `crates/tauri-build/src/lib.rs`
- `crates/tauri-plugin/src/build/mobile.rs`（`setup()` 函数 — 影响所有插件）
- `openharmony-ability/crates/ability/build.rs`

每个 build.rs 都输出 `cargo:rerun-if-env-changed=OHOS_DEVICE_TYPE` 防止 cargo 缓存导致 cfg 不刷新。

### CLI 集成
- `crates/tauri-cli/src/mobile/open_harmony/build.rs` — `--device-type` 参数通过 `set_var` + `CliOptions.vars` 设置 `OHOS_DEVICE_TYPE`
- `crates/tauri-cli/src/mobile/open_harmony/dev.rs` — 同上
- `crates/tauri-cli/src/mobile/open_harmony/mod.rs` — 从 cargo args 中过滤 `--device-type`；从 `CliOptions.vars` 传播 `OHOS_DEVICE_TYPE` 到进程环境

### Plugin cfg Gate 修正
- `plugins/dialog/src/models.rs` — `#[cfg(desktop)]` → `#[cfg(all(desktop, not(target_env = "ohos")))]`，排除 OHOS 的 `rfd` 代码路径
- `plugins/dialog/src/error.rs` — `#[cfg(mobile)]` → `#[cfg(any(mobile, target_env = "ohos"))]`，确保 OHOS 在 desktop 模式下仍包含 mobile plugin pathway 的错误类型
- `plugins/opener/build.rs`、`plugins/shell/build.rs`、`plugins/updater/build.rs` — 添加 OHOS_DEVICE_TYPE 检测

## Impact

- **Code**: 跨 3 个仓库（tauri、plugins-workspace、openharmony-ability）修改约 15 个文件
- **APIs**: 无公开 API 变更 — `OHOS_DEVICE_TYPE` 是构建期环境变量
- **Dependencies**: 无依赖变更
- **Compatibility**: 未设置 `OHOS_DEVICE_TYPE` 时默认 `mobile`，与上游行为一致，完全向后兼容

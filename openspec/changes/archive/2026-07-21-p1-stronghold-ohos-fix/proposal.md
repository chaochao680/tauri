## Why

`tauri-plugin-stronghold` 的依赖链 `iota_stronghold → stronghold_engine → stronghold-runtime → libsodium-sys-stable v1.22.3` 在 Windows 主机交叉编译到 OHOS (`aarch64-linux-ohos`) 时构建失败：`libsodium-sys-stable` 的 `build.rs` 在未设置 `SODIUM_LIB_DIR` 时调用 `install_from_source()` 走 `./configure`，而 Windows `CreateProcess` 无法执行 shell 脚本，返回 `os error 193` (`ERROR_BAD_EXE_FORMAT`)；预编译分支仅覆盖 `*-pc-windows-{msvc,gnu}`，OHOS 目标落入 `_ => panic!` 分支。这阻断了 stronghold 插件在 OHOS 桌面端的可用性，需在 P1 阶段打通编译链。

## What Changes

- 新增 `plugins/stronghold/scripts/build-libsodium-ohos.sh`：使用 OHOS NDK clang (`aarch64-linux-ohos` sysroot) 预编译 libsodium 1.0.20 stable 静态库 (`libsodium.a`)，输出到 `plugins/stronghold/native/ohos/<target>/lib/`。脚本在 Git Bash 下运行，调用上游 `LATEST.tar.gz` 内的 `./configure --host=... --enable-minimal --disable-shared` + `make install`。
- 修改 `plugins/stronghold/build.rs`：在 `tauri_plugin::Builder::new(COMMANDS)` 之前增加 OHOS 目标分支（通过 `std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("ohos")` 检测，**禁止 `cfg!(target_env = "ohos")`** —— build.rs 编译运行在 HOST，`cfg!` 反映 host cfg 而非 target，从 Windows 交叉编译到 `aarch64-linux-ohos` 时 `cfg!(target_env = "ohos")` 恒为 false，OHOS 分支不执行，SODIUM_LIB_DIR 早期失败检测完全不生效，错误被推迟到 libsodium-sys-stable 的 os error 193；项目既有约定 `crates/tauri/build.rs:264`、`crates/tauri-runtime/build.rs:16`、`crates/tauri-runtime-wry/build.rs:16`、`crates/tauri-plugin/src/build/mobile.rs:71` 均用 env var 方式）做早期失败检测。因 `libsodium-sys-stable` 的 build.rs 与 stronghold 的 build.rs 运行在不同进程，build.rs 内 `env::set_var("SODIUM_LIB_DIR", ...)` 无法跨进程传递，故 `SODIUM_LIB_DIR` 必须由构建环境（shell / CI / `source scripts/env.sh`）导出。若 OHOS 目标且 `SODIUM_LIB_DIR` 未设，build.rs 立即 panic 并打印运行预编译脚本与导出 `SODIUM_LIB_DIR` 的指引（避免错误延迟到 libsodium-sys-stable 的 os error 193）。若 `SODIUM_LIB_DIR` 已设，build.rs 不重复输出链接指令（由 libsodium-sys-stable 的 `find_libsodium_env()` 负责）。OHOS 之外的目标完全不受影响。
- 修改 `plugins/stronghold/Cargo.toml`：在 `[package.metadata.platforms.support]` 增加 OHOS 平台元数据声明 (若 workspace schema 不识别则仅作文档)，不改依赖版本。
- 修改 `plugins/stronghold/README.md`：新增 "OHOS 构建" 小节，说明先运行预编译脚本再 `cargo build --target aarch64-linux-ohos`。

## Capabilities

### New Capabilities
- `stronghold-ohos-build`: stronghold 插件在 OHOS 目标 (`aarch64-linux-ohos` / `armv7-linux-ohos` / `x86_64-linux-ohos`) 上的 libsodium 静态库预编译与链接流程，确保 `cargo build/check --target <ohos-triple>` 成功，且运行期行为与 Windows/macOS/Linux 一致 (快照格式二进制兼容)。

### Modified Capabilities
<!-- 无既有 spec-level 行为变更。本变更仅打通编译，不改变 stronghold 的运行期 API/命令/权限语义。 -->

## Impact

- **代码**：`plugins/stronghold/build.rs`、`plugins/stronghold/Cargo.toml`、`plugins/stronghold/README.md`；新增 `plugins/stronghold/scripts/build-libsodium-ohos.sh` 及预编译产物目录 `plugins/stronghold/native/ohos/` (产物不入 git，加 `.gitignore`)。
- **依赖**：不新增 Rust crate 依赖；不修改 `iota_stronghold` / `libsodium-sys-stable` 版本。预编译脚本下载上游 libsodium 源码 tarball (与 libsodium-sys-stable 内置的 `LATEST.tar.gz` 同源同版本 1.0.20)。
- **平台**：OHOS (desktop + mobile 通用，libsodium 无设备形态差异)。Windows/macOS/Linux/Android/iOS 路径不变 (build.rs 通过 `std::env::var("CARGO_CFG_TARGET_ENV")` 检测 OHOS 目标做隔离，非 OHOS 目标不进入该分支)。
- **构建环境**：OHOS 交叉编译需 `OHOS_NDK_HOME` 指向 DevEco SDK 的 `openharmony` 目录 (不带 `/native` 后缀)，与现有 OHOS 构建约定一致。
- **验证**：`cargo check --target aarch64-linux-ohos -p tauri-plugin-stronghold` 通过；设备端 `cargo test` 中 OHOS UT 不依赖此插件 (插件为纯 Rust，UT 走 `cfg(not(target_env = "ohos"))` 排除)。

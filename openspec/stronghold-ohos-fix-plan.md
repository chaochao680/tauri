# Stronghold OHOS 适配计划

**创建时间**：2026-07-17
**功能描述**：解决 stronghold 插件依赖的 libsodium-sys-stable 在 Windows 主机交叉编译 OHOS 时 `./configure` 无法执行 (os error 193) 的构建阻断问题，打通 tauri-plugin-stronghold 在 OHOS 上的编译链。
**判断依据**：涉及 1 个代码层 (tauri-plugin-stronghold 构建配置 + 预构建脚本)，预估 4 个文件。不涉及 ArkTS / NAPI / openharmony-ability 桥接 (libsodium 是纯 C 库，stronghold 是纯 Rust + FFI，运行期无 OHOS 系统能力调用)。

## 问题根因

`libsodium-sys-stable v1.22.3` 的 `build.rs` 在未设置 `SODIUM_LIB_DIR` / `SODIUM_USE_PKG_CONFIG` 时调用 `install_from_source()` → `compile_libsodium_traditional()` → `Command::new(fs::canonicalize(source_dir.join("configure")))`。Windows 上 `CreateProcess` 无法直接执行 shell 脚本 (os error 193 = `ERROR_BAD_EXE_FORMAT`)。预编译分支仅覆盖 `i686/x86_64-pc-windows-{msvc,gnu}`，OHOS 目标 (`aarch64-linux-ohos` 等) 落入 `_ => panic!` 分支。

`build.rs` 已提供逃生舱：`SODIUM_LIB_DIR` 环境变量 → `find_libsodium_env()` 直接链接预构建静态库，跳过 `./configure`。`libsodium-sys-stable` 的 FFI 绑定 (`src/sodium_bindings.rs`) 为预生成并随 crate 发布，构建期不需要 `sodium.h` include 路径。因此方案是：用 OHOS NDK clang 预编译 libsodium 静态库，构建期通过 `SODIUM_LIB_DIR` 注入。

## Phase 列表

| Phase | 名称 | openspec change | 状态 | 涉及层 | 预估文件 | 验证方式 |
|-------|------|----------------|------|--------|---------|---------|
| 1 | 构建打通 (预编译 libsodium + SODIUM_LIB_DIR) | p1-stronghold-ohos-fix | ✓ 设计完成 | tauri-plugin-stronghold | 4 | `cargo check --target aarch64-linux-ohos` |

## Phase 详细说明

### Phase 1: 构建打通
- **目标**：在不修改 libsodium-sys-stable 上游源码、不影响其他平台构建的前提下，让 `tauri-plugin-stronghold` 在 Windows 主机交叉编译到 `aarch64-linux-ohos` (及 `armv7-linux-ohos` / `x86_64-linux-ohos`) 时成功链接 libsodium。
- **文件列表**：
  1. `plugins/stronghold/scripts/build-libsodium-ohos.sh` (新增) — 使用 OHOS NDK clang 预编译 libsodium 静态库的脚本 (Git Bash 运行)，输出到 `plugins/stronghold/native/ohos/<arch>/libsodium.a`。
  2. `plugins/stronghold/build.rs` (修改) — 在 `tauri_plugin::Builder` 之前通过 `std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("ohos")` 运行期检测 OHOS 目标（**禁止 `cfg!(target_env = "ohos")`**：build.rs 编译运行在 HOST，`cfg!` 反映 host cfg 而非 target，从 Windows 交叉编译到 `aarch64-linux-ohos` 时恒为 false，OHOS 分支不执行，SODIUM_LIB_DIR 早期失败检测完全不生效，错误被推迟到 libsodium-sys-stable 的 os error 193）。OHOS 分支内做**早期失败检测**：若 `SODIUM_LIB_DIR` 未设（无论本地产物是否存在），立即 `panic` 并打印运行预编译脚本与 `source scripts/env.sh` 的指引；若已设则正常返回（不再重复输出链接指令，由 libsodium-sys-stable 的 `find_libsodium_env()` 负责）。**不使用 `env::set_var("SODIUM_LIB_DIR", ...)`**：stronghold build.rs 与 libsodium-sys-stable build.rs 是不同进程，`env::set_var` 不跨进程传递，`SODIUM_LIB_DIR` 必须由构建环境（shell / CI / `source scripts/env.sh`）导出。非 OHOS 目标 `CARGO_CFG_TARGET_ENV` 非 "ohos"，分支不进入，其他平台路径字节级不变。
  3. `plugins/stronghold/Cargo.toml` (修改) — 补充 `[package.metadata.platforms.support]` 的 `ohos = { level = "full", notes = "requires prebuilt libsodium via scripts/build-libsodium-ohos.sh" }` 元数据 (若 schema 支持)；不改依赖。
  4. `plugins/stronghold/README.md` (修改) — 增加 "OHOS 构建" 小节，说明先运行预编译脚本再 `cargo build`。
- **依赖**：无

## 方案排除

**替代方案 A (替换为 OHOS cryptoFramework)**：用 `@ohos.security.cryptoFramework` (AES/HMAC/SHA) 重写 stronghold 的加密后端。排除原因：iota_stronghold 内部深度依赖 libsodium 的 `crypto_box`/`crypto_secretbox`/`crypto_pwhash` (Argon2) 等_primitive_，替换需 fork iota_stronghold + stronghold-runtime，工作量级 10x+，且破坏与 Windows/macOS/Linux 的快照格式二进制兼容性。留作未来 P2 探索。

**替代方案 B (vendored feature + Rust 移植)**：用 `sodiumoxide`/`libsodium-sys` 的 pure-Rust 后端。排除原因：上游 crate 固定依赖 `libsodium-sys-stable`，无法切换。

## OHOS 三铁律遵守

1. **OHOS 目标检测方式**：`build.rs` 中所有 OHOS 逻辑用 `std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("ohos")` 运行期分支包裹（**禁止 `cfg!(target_env = "ohos")`**：build.rs 编译运行在 HOST，`cfg!` 反映 host cfg 而非 target，从 Windows 交叉编译到 `aarch64-linux-ohos` 时恒为 false，OHOS 分支不执行，SODIUM_LIB_DIR 早期失败检测完全不生效，错误被推迟到 libsodium-sys-stable 的 `./configure` → os error 193）。`CARGO_CFG_TARGET_ENV` 是 Cargo 为 build script 注入的环境变量，反映**目标** triple 的 `target_env`，交叉编译安全，与项目既有约定一致（`crates/tauri/build.rs:264`、`crates/tauri-runtime/build.rs:16`、`crates/tauri-runtime-wry/build.rs:16`、`crates/tauri-plugin/src/build/mobile.rs:71`）。非 OHOS 目标该条件为 false，Windows/macOS/Linux 走原路径。
2. **不影响其他平台**：不修改任何现有依赖版本、不改 `build.rs` 中现有 `tauri_plugin::Builder` 调用、不引入新 crate 依赖。预编译脚本独立于 cargo 构建图。
3. **无 ArkTS 桥接**：本 Phase 不调用任何 OHOS 系统能力，不需要 openharmony-ability。libsodium 是用户态 C 库，stronghold 运行期纯 Rust + FFI。
4. **跨进程 env 不传递**：stronghold build.rs 与 libsodium-sys-stable build.rs 是不同进程，`env::set_var("SODIUM_LIB_DIR", ...)` 无法跨进程注入。`SODIUM_LIB_DIR` 必须由构建环境（shell / CI / `source scripts/env.sh`）导出，stronghold build.rs 仅负责检测与早期失败提示（未设则 panic，避免错误延迟到 libsodium-sys-stable 的 os error 193）。

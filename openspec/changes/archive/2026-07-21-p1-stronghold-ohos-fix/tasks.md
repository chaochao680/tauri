## 1. 预编译脚本

- [x] 1.1 创建 `plugins/stronghold/scripts/build-libsodium-ohos.sh`，参数为目标 triple (`aarch64-linux-ohos` / `armv7-linux-ohos` / `x86_64-linux-ohos`)，默认编译全部三个
- [x] 1.2 脚本校验 `OHOS_NDK_HOME` 环境变量，未设置则报错退出；拼接 clang 路径 `$OHOS_NDK_HOME/native/llvm/bin/clang` 与 sysroot `$OHOS_NDK_HOME/native/sysroot`
- [x] 1.3 脚本下载 libsodium 1.0.20 stable 源码 tarball (从 `https://download.libsodium.org/libsodium/releases/libsodium-1.0.20-stable.tar.gz` 或使用 `libsodium-sys-stable` crate 内置的 `LATEST.tar.gz` 副本)
- [x] 1.4 脚本对每个目标 triple 设置 `CC=<clang> --target=<triple> --sysroot=<sysroot> -D__MUSL__`，`CFLAGS` 同上，`AR=llvm-ar`，调用 `./configure --host=<host> --enable-minimal --disable-shared --disable-pie --prefix=<out>` + `make -j$(nproc)` + `make install`
- [x] 1.5 脚本将产物输出到 `plugins/stronghold/native/ohos/<triple>/lib/libsodium.a`，并生成 `plugins/stronghold/scripts/env.sh` 内含 `export SODIUM_LIB_DIR=...` 指令供 `source`
- [x] 1.6 在 Git Bash (Windows) 与 Linux 主机各验证一次脚本可运行，产物为正确架构 ELF 静态库 (`llvm-readelf -h libsodium.a` 校验 `Machine: AArch64` 等)

## 2. build.rs OHOS 分支

- [x] 2.1 修改 `plugins/stronghold/build.rs`，在 `tauri_plugin::Builder::new(COMMANDS)` 调用前增加 `if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("ohos") { ... }` 分支。**禁止使用 `cfg!(target_env = "ohos")`**：build.rs 编译运行在 HOST，`cfg!` 反映 host cfg 而非 target，从 Windows 交叉编译到 `aarch64-linux-ohos` 时 `cfg!(target_env = "ohos")` 恒为 false，OHOS 分支不会执行，SODIUM_LIB_DIR 早期失败检测完全不生效，错误被推迟到 libsodium-sys-stable 的 os error 193。`CARGO_CFG_TARGET_ENV` 是 Cargo 为 build script 注入的环境变量，反映**目标** triple 的 `target_env`，交叉编译安全。项目既有约定：`crates/tauri/build.rs:264`、`crates/tauri-runtime/build.rs:16`、`crates/tauri-runtime-wry/build.rs:16`、`crates/tauri-plugin/src/build/mobile.rs:71` 均用此方式
- [x] 2.2 OHOS 分支内：读取 `cargo:rustc-link-search` 所需的 `native/ohos/<target>/lib` 路径 (用 `env::var("CARGO_CFG_TARGET_ARCH")` + `env::var("TARGET")` 拼装)，检测 `libsodium.a` 是否存在
- [x] 2.3 若 `SODIUM_LIB_DIR` 已设置：stronghold build.rs 无需额外输出链接指令（libsodium-sys-stable 的 build.rs 会自行走 `find_libsodium_env()` 分支输出 `cargo:rustc-link-search` + `cargo:rustc-link-lib=static=sodium`）；stronghold build.rs 仅做友好性 `cargo:warning=` 提示当前使用预编译 libsodium 路径
- [x] 2.4 若 `SODIUM_LIB_DIR` 未设（无论本地产物是否存在）：`panic!("OHOS target requires SODIUM_LIB_DIR to be set. libsodium-sys-stable's build.rs runs in a separate process and cannot read env vars set here; without SODIUM_LIB_DIR it will invoke ./configure and fail with os error 193. Run `bash scripts/build-libsodium-ohos.sh <target>` first, then `source scripts/env.sh` (or export SODIUM_LIB_DIR=$(pwd)/native/ohos/<target>/lib). See README.md OHOS section.")` — 原因：stronghold build.rs 与 libsodium-sys-stable build.rs 是不同进程，`env::set_var` 无法跨进程注入，必须由构建环境（shell/CI）导出 `SODIUM_LIB_DIR` 才能让 libsodium-sys-stable 跳过 `./configure`
- [x] 2.5 验证：未设 `SODIUM_LIB_DIR` 时 `cargo check --target aarch64-linux-ohos` 在 stronghold build.rs 阶段即 panic 并打印上述指引（而非让错误延迟到 libsodium-sys-stable 的 os error 193）。重点回归：确认改用 `CARGO_CFG_TARGET_ENV` 后该 panic 在交叉编译时确实触发（若误用 `cfg!(target_env = "ohos")` 则此 panic 永不触发，是审计阻断性 bug）
- [x] 2.6 验证非 OHOS 目标 (`cargo check -p tauri-plugin-stronghold` 在 Windows 默认目标) 构建路径不受影响，OHOS 分支不进入（`CARGO_CFG_TARGET_ENV` 非 "ohos" 时 `if` 条件为 false，分支体不执行）

## 3. 元数据与文档

- [x] 3.1 修改 `plugins/stronghold/Cargo.toml` 的 `[package.metadata.platforms.support]`，增加 `ohos = { level = "full", notes = "requires prebuilt libsodium; run scripts/build-libsodium-ohos.sh" }` (若 workspace schema 不识别该键则跳过并记录)
- [x] 3.2 在 `plugins/stronghold/.gitignore` 增加 `native/` 行，排除预编译产物
- [x] 3.3 修改 `plugins/stronghold/README.md`，新增 "## OHOS Build" 小节：说明前置条件 (`OHOS_NDK_HOME`)、运行预编译脚本、`source scripts/env.sh`、然后 `cargo build --target aarch64-linux-ohos`
- [x] 3.4 在 README 中显式说明此为 P1 编译打通，运行期行为与其他平台一致，不支持的功能 (无)

## 4. 验证

- [ ] 4.1 Windows 主机执行 `cargo check --target aarch64-linux-ohos -p tauri-plugin-stronghold` 退出码 0
- [ ] 4.2 Windows 主机执行 `cargo check --target armv7-linux-ohos -p tauri-plugin-stronghold` 退出码 0
- [ ] 4.3 Windows 主机执行 `cargo check --target x86_64-linux-ohos -p tauri-plugin-stronghold` 退出码 0
- [ ] 4.4 Windows 主机默认目标 `cargo check -p tauri-plugin-stronghold` 退出码 0 (回归验证)
- [ ] 4.5 Linux/macOS 主机 `cargo check -p tauri-plugin-stronghold` 退出码 0 (回归验证，若有 CI 矩阵)
- [ ] 4.6 验证 `cargo build --target aarch64-linux-ohos -p tauri-plugin-stronghold` 产出的 `.rlib` 中包含 libsodium 符号 (`nm` / `llvm-nm` 检查 `crypto_secretbox_easy` 等符号存在)
- [ ] 4.7 (可选，设备端) 在 OHOS 设备上跑一个最小 stronghold demo (create client + save secret + reload)，确认运行期无 undefined symbol 崩溃

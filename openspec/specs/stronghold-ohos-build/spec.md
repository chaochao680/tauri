# stronghold-ohos-build Specification

## Purpose
TBD - created by archiving change p1-stronghold-ohos-fix. Update Purpose after archive.
## Requirements
### Requirement: OHOS 目标编译成功

`tauri-plugin-stronghold` SHALL 在 Windows 主机上交叉编译到 `aarch64-linux-ohos`、`armv7-linux-ohos`、`x86_64-linux-ohos` 三个 triple 时，`cargo check` 与 `cargo build` 均成功完成，不因 `libsodium-sys-stable` 的 `./configure` 执行失败 (os error 193) 而中断。

#### Scenario: aarch64-linux-ohos cargo check 通过
- **WHEN** 开发者在 Windows 主机执行 `cargo check --target aarch64-linux-ohos -p tauri-plugin-stronghold`，且已按 README 运行 `scripts/build-libsodium-ohos.sh` 并导出 `SODIUM_LIB_DIR`
- **THEN** 命令退出码为 0，无 `os error 193`、无 `Unable to compile or find precompiled libsodium` panic

#### Scenario: armv7-linux-ohos cargo check 通过
- **WHEN** 开发者执行 `cargo check --target armv7-linux-ohos -p tauri-plugin-stronghold`，且 `SODIUM_LIB_DIR` 指向 `native/ohos/armv7-linux-ohos/lib`
- **THEN** 命令退出码为 0

#### Scenario: x86_64-linux-ohos cargo check 通过
- **WHEN** 开发者执行 `cargo check --target x86_64-linux-ohos -p tauri-plugin-stronghold`，且 `SODIUM_LIB_DIR` 指向 `native/ohos/x86_64-linux-ohos/lib`
- **THEN** 命令退出码为 0

### Requirement: 预编译脚本可复现产出 libsodium 静态库

`scripts/build-libsodium-ohos.sh` SHALL 使用 `OHOS_NDK_HOME` 指定的 NDK clang 与 sysroot，从 libsodium 1.0.20 stable 源码编译 `libsodium.a` 静态库到 `native/ohos/<target-triple>/lib/` 目录，且对同一 NDK 版本可重复产出字节级一致的产物 (使用 `--enable-minimal --disable-shared --disable-pie` 配置)。

#### Scenario: 预编译 aarch64 静态库
- **WHEN** 开发者在 Git Bash 中执行 `OHOS_NDK_HOME=/path/to/openharmony bash scripts/build-libsodium-ohos.sh aarch64-linux-ohos`
- **THEN** 脚本退出码为 0，`native/ohos/aarch64-linux-ohos/lib/libsodium.a` 文件存在且大于 0 字节，且为 `aarch64` 架构 ELF (可通过 `file` 或 `llvm-readelf` 验证)

#### Scenario: 缺少 OHOS_NDK_HOME 报错
- **WHEN** 开发者未设置 `OHOS_NDK_HOME` 执行脚本
- **THEN** 脚本立即退出码非 0 并打印 "OHOS_NDK_HOME not set" 提示，不进行任何编译

#### Scenario: 产物不入版本库
- **WHEN** 脚本执行完毕
- **THEN** `native/ohos/` 目录被 `plugins/stronghold/.gitignore` 排除，`git status` 不显示该目录内文件

### Requirement: 不影响其他平台构建

`build.rs` 的 OHOS 分支 SHALL 通过 `std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("ohos")` 运行期检测隔离（**禁止 `cfg!(target_env = "ohos")`** —— build.rs 编译运行在 HOST，`cfg!` 反映 host cfg 而非 target，交叉编译到 OHOS 时恒为 false，OHOS 分支不执行），Windows、macOS、Linux、Android、iOS 目标的构建路径 SHALL 与变更前字节级一致，不引入新的环境变量依赖或 panic 路径。

#### Scenario: Windows 目标构建不变
- **WHEN** 开发者执行 `cargo check -p tauri-plugin-stronghold` (默认 x86_64-pc-windows-msvc)
- **THEN** 构建路径不读取 `native/ohos/` 目录，不检查 `SODIUM_LIB_DIR`，行为与变更前一致 (libsodium-sys-stable 走原有 MSVC 预编译或源码编译分支)

#### Scenario: Linux 目标构建不变
- **WHEN** 开发者执行 `cargo check --target x86_64-unknown-linux-gnu -p tauri-plugin-stronghold`
- **THEN** build.rs 中 `CARGO_CFG_TARGET_ENV` 不等于 `"ohos"`，OHOS 分支条件为 false 不进入，构建行为与变更前一致

### Requirement: 防御性检测与错误提示

当 OHOS 目标构建时 `SODIUM_LIB_DIR` 未设置，`build.rs` SHALL 以清晰的 panic 消息提示开发者运行预编译脚本并导出 `SODIUM_LIB_DIR`，而非让 `libsodium-sys-stable` 因 os error 193 失败并留下难以诊断的错误。因 `libsodium-sys-stable` 的 build.rs 与 stronghold 的 build.rs 运行在不同进程，stronghold build.rs 内 `env::set_var("SODIUM_LIB_DIR", ...)` 无法跨进程传递，故 `SODIUM_LIB_DIR` 必须由构建环境（shell / CI / `source scripts/env.sh`）导出；stronghold build.rs 仅负责检测与早期失败提示。

#### Scenario: 缺少 SODIUM_LIB_DIR 时清晰报错
- **WHEN** 开发者执行 `cargo check --target aarch64-linux-ohos -p tauri-plugin-stronghold`，且未设置 `SODIUM_LIB_DIR`（无论 `native/ohos/<target>/lib/libsodium.a` 是否已存在）
- **THEN** stronghold 的 build.rs 在 libsodium-sys-stable build.rs 运行之前 panic，消息包含 "run `bash scripts/build-libsodium-ohos.sh`" 与 "source scripts/env.sh" 指引，并说明 `SODIUM_LIB_DIR` 必须由构建环境导出（原因：跨进程 env 不传递）

#### Scenario: SODIUM_LIB_DIR 已设置时静默通过
- **WHEN** OHOS 目标构建时 `SODIUM_LIB_DIR` 已指向 `native/ohos/<target>/lib/libsodium.a` 所在目录
- **THEN** stronghold build.rs 正常完成（可打印 `cargo:warning=` 友好提示），libsodium-sys-stable build.rs 走 `find_libsodium_env()` 分支输出链接指令，`cargo check` 退出码为 0

### Requirement: 运行期行为与桌面平台一致

OHOS 上 stronghold 的运行期行为 (快照加解密、密钥派生、procedure 执行) SHALL 与 Windows/macOS/Linux 一致，快照文件格式 SHALL 跨平台二进制兼容 (同一密码可在不同平台加载同一快照)。

#### Scenario: 跨平台快照兼容
- **WHEN** 在 Windows 上用 stronghold 创建并保存快照 `vault.stronghold`，将其拷贝到 OHOS 设备并用相同密码加载
- **THEN** OHOS 端 `load_snapshot` 成功，可读取 store record 与 secret，与 Windows 端数据一致

#### Scenario: Argon2 KDF 在 OHOS 可用
- **WHEN** OHOS 上使用 `Builder::with_argon2(salt_path)` 初始化 stronghold (启用 `kdf` feature)
- **THEN** argon2 哈希计算成功 (rust-argon2 纯 Rust 实现，不依赖 libsodium)，salt 文件正确读写


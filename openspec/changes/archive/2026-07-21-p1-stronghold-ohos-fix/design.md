## Context

`tauri-plugin-stronghold` v2.3.1 通过 `iota_stronghold` v2.1.0 间接依赖 `libsodium-sys-stable` v1.22.3。该 sys crate 的 `build.rs` 提供三条 libsodium 获取路径，按优先级：

1. `SODIUM_LIB_DIR` 环境变量 → `find_libsodium_env()`：直接 `cargo:rustc-link-search` + `cargo:rustc-link-lib=static=sodium`，跳过一切编译。**不输出 `cargo:include`**，因为 FFI 绑定 `src/sodium_bindings.rs` 已预生成并随 crate 发布，构建期不需要 `sodium.h`。
2. `SODIUM_USE_PKG_CONFIG` (或 `use-pkg-config` feature) → `find_libsodium_pkgconfig()` / `find_libsodium_vpkg()`。
3. 都未设置 → `install_from_source()` → `compile_libsodium_traditional()` → `Command::new(source_dir.join("configure"))` 执行上游 autoconf `./configure && make`。失败后按目标 triple 匹配预编译 zip/tarball，但仅覆盖 `i686/x86_64-pc-windows-{msvc,gnu}` 四个 triple，其余目标 `_ => panic!`。

OHOS 目标 (`aarch64-linux-ohos`、`armv7-linux-ohos`、`x86_64-linux-ohos`) 在 Windows 主机上落入路径 3：`Command::new("configure")` 经 `CreateProcessW` 调用，因 `configure` 是 POSIX shell 脚本 (无 `.exe`/PE 头) 返回 `ERROR_BAD_EXE_FORMAT` (os error 193)。即便在 Git Bash 终端里运行 `cargo build`，`std::process::Command` 仍走 Win32 API 而非 bash，所以问题与终端无关。

stronghold 运行期为纯 Rust + libsodium FFI，不调用任何 OHOS 系统能力 (无 ArkTS、无 NAPI、无文件系统之外的平台 API)。快照文件格式由 iota_stronghold 定义，跨平台二进制兼容。因此本问题纯属构建期/工具链问题，运行期无需适配。

## Goals / Non-Goals

**Goals:**
- 让 `cargo check/build --target aarch64-linux-ohos -p tauri-plugin-stronghold` 在 Windows 主机成功。
- 同样支持 `armv7-linux-ohos` 与 `x86_64-linux-ohos`。
- 不修改 `libsodium-sys-stable` 上游源码 (位于 `~/.cargo/registry`，patch 不可移植)。
- 不修改 `iota_stronghold` / `stronghold_engine` / `stronghold-runtime` 任何版本。
- OHOS 之外的所有目标 triple 构建路径字节级不变。
- 预编译产物可复现、可审计 (使用与 `libsodium-sys-stable` 内置 `LATEST.tar.gz` 相同的 libsodium 1.0.20 stable 源码)。

**Non-Goals:**
- 不替换 iota_stronghold 的加密后端为 `@ohos.security.cryptoFramework` (留作未来 P2，会破坏快照格式兼容)。
- 不在 OHOS 上跑 stronghold 的设备端单元测试 (UT 依赖 `mock_runtime` + `tao::EventLoop`，OHOS 不可用，已由 `cfg(not(target_env = "ohos"))` 排除)。
- 不为 libsodium 启用硬件加速 (aarch64 NEON 等) ——P1 只求打通，用 `--enable-minimal` 默认实现。
- 不做动态库 (`libsodium.so`) 打包；P1 用静态链接，简化部署。
- 不修改 stronghold 的前端 JS API、权限模型、命令列表。

## Decisions

### Decision 1: 用预编译静态库 + `SODIUM_LIB_DIR`，而非 patch build.rs

**选择**：在 `plugins/stronghold/build.rs` 内对 OHOS 目标分支（通过 `std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("ohos")` 检测，**不能用 `cfg!(target_env = "ohos")`**，因为 build.rs 编译运行在 HOST，`cfg!` 反映 host cfg 而非 target —— 从 Windows 交叉编译到 `aarch64-linux-ohos` 时 `cfg!(target_env = "ohos")` 恒为 false，OHOS 分支不会执行）让 libsodium-sys-stable 的 build.rs 走 `find_libsodium_env()` 分支。

> ⚠️ **覆盖说明（以 Decision 4「最终采用」为准，实现者务必读本段）**：本 Decision 原始措辞提到的 `env::set_var("SODIUM_LIB_DIR", path)` 方案**已被否决**，仅保留作方案演进的历史记录。Decision 4 已证明 stronghold build.rs 与 libsodium-sys-stable build.rs 是 cargo 编译图中的**不同进程**，build.rs 内 `env::set_var` 设置的环境变量**不跨进程传递**，libsodium-sys-stable build.rs 读不到该变量，仍会落到 `install_from_source()` → `./configure` → os error 193。**实现者须按 Decision 4 的「最终采用」方案执行**：由构建环境（shell / CI / `source scripts/env.sh`）`export SODIUM_LIB_DIR=<path>`，stronghold build.rs 在 OHOS 分支**仅做早期失败检测**——`SODIUM_LIB_DIR` 未设则立即 `panic` 并打印运行预编译脚本与 `source scripts/env.sh` 的指引，已设则不再重复输出链接指令（由 libsodium-sys-stable 的 `find_libsodium_env()` 负责输出 `rustc-link-search` + `rustc-link-lib=static=sodium`）。**禁止**在 build.rs 内调用 `env::set_var("SODIUM_LIB_DIR", ...)` 企图跨 crate 注入。

**备选 A (patch libsodium-sys-stable)**：用 `[patch.crates-io]` 替换为本地 fork，在 build.rs 增加 OHOS 预编译分支。**否决**：patch 不可移植 (其他开发者需同步 fork)，且 `libsodium-sys-stable` 已提供 `SODIUM_LIB_DIR` 逃生舱，无需改上游。

**备选 B (cargo `[build-dependencies]` + `cc` 重写编译)**：在 stronghold build.rs 里直接用 `cc::Build` 编译 libsodium 源码。**否决**：libsodium 用 autoconf 生成 `config.h` 与 `libtool`，直接 cc 编译需手写 100+ 源文件列表与配置宏，维护成本高且易出错。

**备选 C (`SODIUM_USE_PKG_CONFIG` + sysroot 内安装)**：把 libsodium 装进 OHOS NDK sysroot 用 pkg-config 找。**否决**：OHOS NDK 不带 pkg-config，且污染 sysroot 影响其他 crate。

### Decision 2: 预编译脚本独立于 cargo 构建图

`scripts/build-libsodium-ohos.sh` 是开发者手动运行的一次性脚本 (类似 `npm install`)，不作为 cargo `build.rs` 的子进程。理由：
- cargo build.rs 内下载/编译 libsodium 会显著拖慢增量构建，且 `Command::new("configure")` 在 Windows 仍失败 (脚本本身要在 Git Bash 跑)。
- 产物存入 `native/ohos/<target>/lib/libsodium.a`，`.gitignore` 排除，每个开发者/CI 首次运行一次。
- build.rs 只做"检测 `SODIUM_LIB_DIR` 是否已由构建环境导出 → 未设则 panic"（见 Decision 4「最终采用」：build.rs 内 `env::set_var` 不跨进程传递，故 build.rs 不再设置该变量，仅做早期失败检测），无网络、无编译，O(1) 开销。

### Decision 3: 三架构统一脚本，sysroot 与 clang 从环境变量取

脚本读取 `OHOS_NDK_HOME` (约定：指向 `openharmony` 目录，不带 `/native` 后缀，与项目其他 OHOS 构建一致)，内部拼接 `$OHOS_NDK_HOME/native/llvm/bin/clang`、`--sysroot=$OHOS_NDK_HOME/native/sysroot`、`-D__MUSL__`。对每个目标 triple 设置：

| Rust triple | configure `--host` | clang `--target` |
|-------------|-------------------|------------------|
| `aarch64-linux-ohos` | `aarch64-linux-ohos` | `aarch64-linux-ohos` |
| `armv7-linux-ohos` | `arm-linux-ohos` | `arm-linux-ohos` |
| `x86_64-linux-ohos` | `x86_64-linux-ohos` | `x86_64-linux-ohos` |

调用上游 `./configure --host=<host> --enable-minimal --disable-shared --disable-pie --prefix=<out>` 然后 `make -j && make install`。`--enable-minimal` 与 `libsodium-sys-stable` 的 `compile_libsodium_traditional` 一致，保证符号集相同。

### Decision 4: build.rs 内 `env::set_var` 跨 crate 生效

Rust build scripts 在 cargo 编译图中作为独立进程运行，但 `libsodium-sys-stable` 的 build.rs 与 `tauri-plugin-stronghold` 的 build.rs 是**不同进程**。`env::set_var` 在 stronghold build.rs 进程内设置的环境变量**不会**自动传递给 libsodium-sys-stable build.rs 进程。

**解决方案**：用 cargo 的 `links` 机制 + `cargo:rustc-link-search`/`cargo:rustc-link-lib` 直接在 stronghold build.rs 输出链接指令；同时 libsodium-sys-stable build.rs 仍会运行 (它有自己的 `links = "sodium"`)，需要让它走 `find_libsodium_env()` 分支。两种手段组合：

1. **stronghold build.rs** 在 OHOS 分支 `println!("cargo:rustc-link-search=native={}", lib_dir)` + `println!("cargo:rustc-link-lib=static=sodium")`，保证最终链接能找到库。
2. **通过 `.cargo/config.toml` 或环境变量注入**：在 OHOS 构建环境 (CI/开发者 shell) 中导出 `SODIUM_LIB_DIR=<path>`，使 libsodium-sys-stable build.rs 走 `find_libsodium_env()` 而非 `install_from_source()`。README 文档化此步骤。

> 若 cargo `links` 冲突 (`libsodium-sys-stable` 已声明 `links = "sodium"`，stronghold 未声明)，则 stronghold build.rs 的 `rustc-link-lib=sodium` 是合法的补充链接指令，不冲突——`links` 只防止两个 crate 同时构建同一个原生库，此处 libsodium-sys-stable 已是唯一的 libsodium 构建者，stronghold 只是额外注入搜索路径。

**最终采用**：README + 脚本生成 `env.sh` 供 `source`，提示用户 `export SODIUM_LIB_DIR=...`。stronghold build.rs 在 OHOS 分支（`std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("ohos")`，**不可用 `cfg!(target_env = "ohos")`**，原因见 Decision 1）做**早期失败**检测：若 `SODIUM_LIB_DIR` 未设（无论本地产物是否存在），立即 `panic` 并打印运行预编译脚本与 `source scripts/env.sh` 的指引。原因：stronghold build.rs 与 libsodium-sys-stable build.rs 是不同进程，`env::set_var` 无法跨进程注入；不 panic 的话 libsodium-sys-stable build.rs 仍会走到 `./configure` 并以 os error 193 失败，错误信息难以诊断。若 `SODIUM_LIB_DIR` 已设，stronghold build.rs 不再重复输出链接指令（libsodium-sys-stable 的 `find_libsodium_env()` 已完整输出 `rustc-link-search` + `rustc-link-lib=static=sodium`），仅可选打印 `cargo:warning=` 友好提示。

> **OHOS 检测方式（铁律）**：build.rs 内检测目标是否为 OHOS 必须 `std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("ohos")`，禁止 `cfg!(target_env = "ohos")`。`CARGO_CFG_TARGET_ENV` 是 Cargo 为 build script 注入的环境变量，反映**目标** triple 的 `target_env`，交叉编译安全。`cfg!` 宏在 build.rs 中反映的是 **host** cfg 而非 target —— 从 Windows 交叉编译到 `aarch64-linux-ohos` 时 `cfg!(target_env = "ohos")` 恒为 false，OHOS 分支不会执行，`SODIUM_LIB_DIR` 早期失败检测完全不生效，错误会被推迟到 `libsodium-sys-stable` 的 `./configure` → os error 193。项目既有约定：`crates/tauri/build.rs:264`、`crates/tauri-runtime/build.rs:16`、`crates/tauri-runtime-wry/build.rs:16`、`crates/tauri-plugin/src/build/mobile.rs:71` 均用此 env var 方式。

### Decision 5: 版本对齐

预编译脚本下载 libsodium 1.0.20 stable (与 `libsodium-sys-stable v1.22.3` 内置 `LATEST.tar.gz` 同版本)，避免 FFI 符号缺失或布局差异。脚本校验 tarball 的 minisig (使用 `libsodium-sys-stable` 内置的 `LATEST.tar.gz.minisig` 公钥路径，或跳过验签但打印 warning)。

## Risks / Trade-offs

- **[风险] 预编译符号集与 `sodium_bindings.rs` 不匹配** → 缓解：固定 libsodium 1.0.20 (与 crate 内置源码同版本)；`--enable-minimal` 与 crate 配置一致；首次 `cargo check` 验证链接成功即可发现符号缺失。
- **[风险] aarch64 结构体布局差异** → 缓解：libsodium 的 `crypto_*_state` 结构体字段为固定宽度整型 (`unsigned char`/`uint32_t`/`uint64_t`)，aarch64 与 x86_64 布局一致；`bindgen_test_layout_*` 在测试期验证 (OHOS 上测试被排除，但 Windows/macOS 测试覆盖了相同布局)。
- **[风险] `SODIUM_LIB_DIR` 跨进程不传递** → 缓解：Decision 4 的文档化 + build.rs 防御性检测 + 脚本生成 `env.sh`。
- **[权衡] 静态库体积** → libsodium minimal 静态库约 200KB，可接受。
- **[权衡] 开发者需手动跑预编译脚本** → 类似 `npm install` 的一次性步骤，README 显式说明，CI 在构建前自动执行。
- **[风险] OHOS NDK clang 版本与 libsodium 兼容性** → 缓解：libsodium 1.0.20 对 clang 兼容性良好；脚本失败时输出 clang 版本与错误日志便于排查。

## Migration Plan

1. 合并本 change 后，OHOS 构建者在 `plugins/stronghold/` 下执行 `bash scripts/build-libsodium-ohos.sh` (需 `OHOS_NDK_HOME` 环境变量)。
2. `source scripts/env.sh` (脚本生成) 或手动 `export SODIUM_LIB_DIR=$(pwd)/native/ohos/aarch64-linux-ohos/lib`。
3. 执行 `cargo check --target aarch64-linux-ohos -p tauri-plugin-stronghold` 验证。
4. 回滚：删除 `native/ohos/` 目录、还原 `build.rs`/`Cargo.toml`/`README.md`，无数据迁移。

## Open Questions

- OHOS NDK 是否自带 `make`？若否，脚本需检测并提示安装 (Git Bash 通常带 `make`)。
- `armv7-linux-ohos` 的 `--host` triple 是 `arm-linux-ohos` 还是 `armv7-linux-ohos`？需在脚本中验证 (libsodium autoconf 接受任意 triple，但会影响 `config.sub` 匹配)。

---

## 实现期运行时修复 (2026-07-21，P1 编译打通后的运行时调试)

P1 编译打通后，examples/api stronghold 自动测试运行时连续暴露三个独立问题，逐一定位并修复（均 OHOS cfg 隔离，不影响其他平台）：

1. **`NCSizeNotAllowed`（key 长度 13≠32）**：根因 = `NCKey::load`（stronghold_engine crypto_box.rs:186）做 `key.len() == Provider::box_key_len()=32` 纯长度检查；examples/api 注册 `Builder::new(|p| p.as_bytes().to_vec())` 传 13 字节明文（"test-password"）→ None → `MemoryError::NCSizeNotAllowed`。平台无关（所有平台都会 fail，只是非 OHOS 走 isMissing skip）。修复 = examples/api OHOS block 改用 `Builder::with_argon2(salt_path)`（argon2 default hash_length=32 满足 box_key_len；salt 用 `/data/storage/el2/base/cache/stronghold-salt.key`，OHOS sandbox redirect 到 app cache）。
2. **`Permission denied (os error 13)`**：根因 = OHOS app CWD=`/`（root 拥有，app 只有 `--x`），快照相对路径解析到 `/` 不可写。修复 = 插件 `initialize` 改泛型 `<R: Runtime>(app: AppHandle<R>, ...)`，OHOS 下 `app.path().resolve(&snapshot_path, BaseDirectory::AppData)` 把相对快照路径 resolve 到 AppData（对齐 store 的 `resolve_store_path`）；collection 仍按 received path 做 key，`Stronghold` 内部持 resolved path，`save`/`destroy` 无需改。
3. **`5000ms timeout`**：根因 = age/scrypt KDF work factor 默认 19（N=2¹⁹），OHOS aarch64 ~4.2s/次，测试做 4 次 encrypt/decrypt → 17s > 5s。hilog 计时确认 `try_unlock` 1.4ms（LockedMemory 无辜），4.2s 在 `encrypt_file`/`decrypt_file`（scrypt）。修复 = examples/api OHOS setup 调 `iota_stronghold::engine::snapshot::try_set_encrypt_work_factor(10)`（N=2¹⁰，毫秒级；WF 写入快照头，decrypt 读文件 WF 故 encrypt+decrypt 都快）。该 API 源码注释明说"testing 用，production 勿改"，故放测试 app OHOS setup。

**手动测试 `loadClient` 修正**：重启后 Stronghold Verify 用 `createClient` 会新建空 client 覆盖快照数据 → 假失败；改用 `loadClient`（从快照恢复持久化数据）。

验证：test-report stronghold ✅ 390-603ms；手动 Persist/Verify 跨重启 ✅。

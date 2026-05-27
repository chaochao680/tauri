# ohos-build

编译 Tauri OpenHarmony 项目（examples/api），生成 HAP 包并签名安装到设备。

## 一键构建部署

```bash
source D:/workspace/tauri/tauri/.claude/skills/ohos-build/scripts/env.sh
bash D:/workspace/tauri/tauri/.claude/skills/ohos-build/scripts/run-tests.sh "" desktop
```

第二个参数为设备类型：`desktop`（PC/桌面，tray/menu 需要）或 `mobile`（手机/平板）。

脚本自动完成全部流程：
1. 检测 `openharmony-ability/` 源码变更，自动重建 HAR 包并 ohpm install
2. 前端构建（pnpm + vite，VITE_AUTOTEST=true）
3. Rust 交叉编译（aarch64-unknown-linux-ohos，release，--features prod）
4. 拷贝 .so → hvigorw assembleHap（自动禁用/恢复 tauriPlugin，使用 build-profile.json5 中的证书签名）
5. 卸载旧版 → 安装已签名 HAP → 启动
6. 等待 30s → 拉取 test-report → 分析结果

## 环境要求

- **运行环境**: Git Bash（`C:\Program Files (x86)\Git\bin\bash.exe`）
- DevEco Studio（含 OpenHarmony SDK、ohpm、hvigor、JBR）
- pnpm、Rust + `aarch64-unknown-linux-ohos` target
- hdc（设备连接工具，SDK 自带）
- 设备已通过 USB 连接（`hdc list targets` 可见）

首次使用如果 DevEco Studio 自动检测失败：
```bash
echo 'DEVECO_HOME="/d/app/DevEco-Studio"' > D:/workspace/tauri/tauri/.claude/skills/ohos-build/scripts/.env.local
```

## 设备类型（TAURI_OHOS_DEVICE_TYPE）

| 值 | 说明 | 编译特性 |
|---|------|---------|
| `desktop` | PC/桌面设备 | `cfg(desktop)` — Tray/Menu 功能需要此模式 |
| `mobile` | 手机/平板（默认） | `cfg(mobile)` |

## 脚本说明

| 脚本 | 功能 |
|------|------|
| `env.sh` | 环境配置：CC/linker/JAVA_HOME/PATH，必须在其他脚本前 source |
| `run-tests.sh` | 一键全流程（含 HAR 自动重建、tauriPlugin 自动禁用/恢复） |
| `build-ohos.sh` | 构建全流程（前端 → Rust → .so → hvigorw 签名打包），自动处理 tauriPlugin |
| `sign-and-install.sh` | 仅安装启动（使用 hvigorw 已签名的 HAP），不构建不签名 |

## 关键注意事项

1. **env.sh 必须先 source** — 设置 OHOS linker/CC/JAVA_HOME，否则 Rust 编译报 `cc not found`
2. **prod feature** — 不加则 app 连接 localhost:1420 而非加载打包前端。脚本已自动包含。
3. **hdc 路径转义** — Git Bash 把 `/data/...` 转为 Windows 路径。设备路径命令用 `hdc shell "cat ..."` 加引号。
4. **签名由 hvigorw 完成** — build-profile.json5 中配置了含 system_basic 权限的证书，hvigorw assembleHap 自动签名。无需手动签名。
5. **HAR 缓存** — 修改了 openharmony-ability 后必须重建 HAR。`run-tests.sh` Step 0 自动检测并处理。
6. **tauriPlugin** — hvigorfile.ts 中的 tauriPlugin 需要 TCP 回调 tauri CLI，独立构建时必须禁用。`build-ohos.sh` 和 `run-tests.sh` 自动处理。

## 设备日志与故障诊断

```bash

# 实时日志（持续输出，Ctrl+C 停止）
hdc shell "hilog | grep -i 关键词"

# 缓冲区日志（非阻塞，-z 取最后 N 行）
# Rust log::info/error 的 domain 为 A00000，tag 为 tauritest
hdc shell "hilog -x" | grep "com.tauri"

# hilog 归档文件（选 hilog.*.gz，忽略 hilog_kmsg.*.gz）
hdc shell "ls -lt /data/log/hilog/ | grep -v hilog_kmsg | grep hilog | head -3"
hdc shell "zcat /data/log/hilog/hilog.最新序号.时间戳.gz | grep 关键词"

# Crash 日志（jscrash / cppcrash）
hdc shell "ls -lt /data/log/faultlog/faultlogger/ | grep tauri | head -5"
hdc shell "cat /data/log/faultlog/faultlogger/jscrash-com.tauri.api-最新文件名"

# Freeze 日志（主线程阻塞 3s+）
hdc shell "ls -lt /data/log/faultlog/faultlogger/ | grep appfreeze | head -5"

# Rust panic
hdc shell "cat /data/app/el2/100/base/com.tauri.api/cache/panic.log"
```

## 文件系统路径

| 视角 | 路径 | 说明 |
|------|------|------|
| App 内部（Rust 代码中） | `/data/storage/el2/base/cache/` | 应用沙箱路径 |
| 外部（hdc 访问） | `/data/app/el2/100/base/com.tauri.api/cache/` | 实际物理路径 |

## 手动 HAR 重建（仅在脚本自动检测失效时需要）

```bash
cd D:/workspace/tauri/openharmony-ability
source D:/workspace/tauri/tauri/.claude/skills/ohos-build/scripts/env.sh
ohrs build --arch arm64          # 末尾 panic 是已知问题，不影响
bash scripts/pack.sh
tar -czf ability.har package
cmd.exe /c "rmdir /s /q D:\workspace\tauri\tauri\examples\api\src-tauri\gen\ohos\entry\oh_modules"
cmd.exe /c "cd /d D:\workspace\tauri\tauri\examples\api\src-tauri\gen\ohos\entry && ohpm install"
```

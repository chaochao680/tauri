---
name: ohos-build
description: 编译 Tauri OpenHarmony 项目（examples/api），生成 HAP 包并签名安装到设备。使用场景：(1) 一键构建部署，(2) HAR 包重建与安装，(3) 模板修改后重新生成，(4) 设备日志与故障诊断。
---

# ohos-build

编译 Tauri OpenHarmony 项目（examples/api），生成 HAP 包并签名安装到设备。

> **路径约定**: 以下脚本中 `${PROJECT_ROOT}` 指 tauri 和 openharmony-ability 共同的父目录。
> 例如本地布局为 `/d/xuqiu/tauri-2.0/{tauri,openharmony-ability}` 时，`PROJECT_ROOT=/d/xuqiu/tauri-2.0`。
> 使用前请先 `export PROJECT_ROOT=...` 或在 env.sh 中设置。

## 一键构建部署

```bash
source ${PROJECT_ROOT}/tauri/.claude/skills/ohos-build/scripts/env.sh
bash ${PROJECT_ROOT}/tauri/.claude/skills/ohos-build/scripts/run-tests.sh "" desktop
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
echo 'DEVECO_HOME="/d/app/DevEco-Studio"' > ${PROJECT_ROOT}/tauri/.claude/skills/ohos-build/scripts/.env.local
```

## 设备类型（OHOS_DEVICE_TYPE）

| 值 | 说明 | 编译特性 |
|---|------|---------|
| `desktop` | PC/桌面设备 | `cfg(desktop)` — Tray/Menu 功能需要此模式 |
| `mobile` | 手机/平板（默认） | `cfg(mobile)` |

## 脚本说明

| 脚本 | 功能 |
|------|------|
| `env.sh` | 环境配置：CC/linker/JAVA_HOME/PATH，必须在其他脚本前 source |
| `run-tests.sh` | 一键全流程（含模板检测、HAR 自动重建、tauriPlugin 自动禁用/恢复） |
| `build-ohos.sh` | 构建全流程（模板检测 → 前端 → Rust → .so → hvigorw 签名打包），自动处理 tauriPlugin |
| `sign-and-install.sh` | 仅安装启动（使用 hvigorw 已签名的 HAP），不构建不签名 |

## 模板修改后的完整生效流程

修改了 `crates/tauri-cli/templates/mobile/open-harmony/` 下的模板文件后，需要：

```bash
# 1. 重装 tauri-cli（模板编译进二进制，必须重装）
cargo install --path crates/tauri-cli --locked

# 2. 删除旧项目并重新生成
cmd.exe /c "rmdir /s /q ${PROJECT_ROOT}\\tauri\\examples\\api\\src-tauri\\gen\\ohos"
(cd examples/api/src-tauri && cargo tauri ohos init --skip-targets-install --ci)

# 3. 执行「init 后补充步骤」（见下方章节）

# 4. 重建 HAR 并 ohpm install
cd ${PROJECT_ROOT}/openharmony-ability
source ${PROJECT_ROOT}/tauri/.claude/skills/ohos-build/scripts/env.sh
ohrs build --arch arm64 --skip-napi-check 2>&1 | tail -5 || true
bash scripts/pack.sh
tar -czf ability.har package
cd ${PROJECT_ROOT}/tauri/examples/api/src-tauri/gen/ohos
ohpm install --all
```

## init 后补充步骤（每次 `tauri ohos init` 后必须执行）

`tauri ohos init` 重新生成 `gen/ohos/` 后，以下内容会丢失或缺失，需要手动补充：

### ① 签名配置（build-profile.json5）

`signingConfigs` 会被重置为 `[]`，必须在 **DevEco Studio** 中重新配置：
- 打开项目 `examples/api/src-tauri/gen/ohos/`
- File → Project Structure → Signing Configs → 配置自动签名或手动签名
- 保存

### ② 页面路由（main_pages.json）

`main_pages.json` 会被模板覆盖为仅包含 `pages/Index`，需要手动添加：

```json5
// gen/ohos/entry/src/main/resources/base/profile/main_pages.json
{
  "src": [
    "pages/Index",
    "pages/TestTrayPage",           // ← QuickOperation 面板页面
    "pages/TransparencyTestPage"    // ← WebView 透明度测试页面
  ]
}
```

- `TestTrayPage` 是 `TestTrayAbility`（QuickOperation 面板）的内容页面，缺少此路由注册会导致面板打开后内容为空。
- `TransparencyTestPage` 是 WebView 容器透明背景测试页面（Float 子窗口穿透测试）。

### ③ TestTrayPage.ets 和 TestTrayAbility.ets 文件

这两个文件不在模板中（属于 examples/api 项目特有），`tauri ohos init` 不会生成它们。如果之前删除了整个 `gen/ohos/` 目录，需要手动恢复：
- `gen/ohos/entry/src/main/ets/pages/TestTrayPage.ets`
- `gen/ohos/entry/src/main/ets/testtrayability/TestTrayAbility.ets`
- `gen/ohos/entry/src/main/ets/pages/TransparencyTestPage.ets` ← WebView 透明度测试页面

**归档位置**: 这些文件已归档在 `.claude/skills/ohos-build/templates/` 目录下：
- `templates/pages/TestTrayPage.ets`
- `templates/testtrayability/TestTrayAbility.ets`
- `templates/pages/TransparencyTestPage.ets`

恢复命令：
```bash
cp ${PROJECT_ROOT}/tauri/.claude/skills/ohos-build/templates/pages/TestTrayPage.ets \
     ${PROJECT_ROOT}/tauri/examples/api/src-tauri/gen/ohos/entry/src/main/ets/pages/
cp ${PROJECT_ROOT}/tauri/.claude/skills/ohos-build/templates/pages/TransparencyTestPage.ets \
     ${PROJECT_ROOT}/tauri/examples/api/src-tauri/gen/ohos/entry/src/main/ets/pages/
# TestTrayAbility 需要复制到 testtrayability 目录
cp ${PROJECT_ROOT}/tauri/.claude/skills/ohos-build/templates/testtrayability/TestTrayAbility.ets \
     ${PROJECT_ROOT}/tauri/examples/api/src-tauri/gen/ohos/entry/src/main/ets/testtrayability/
```

### ④ 扩展能力和权限（module.json5）

`module.json5` 会被模板覆盖，需要手动补充 `TestTrayAbility` 扩展和 `SET_WINDOW_TRANSPARENT` 权限：

```json5
// gen/ohos/entry/src/main/module.json5
{
  "module": {
    // ... 其他配置保持不变 ...
    "extensionAbilities": [
      {
        "name": "EntryBackupAbility",
        "srcEntry": "./ets/entrybackupability/EntryBackupAbility.ets",
        "type": "backup",
        "exported": false,
        "metadata": [
          {
            "name": "ohos.extension.backup",
            "resource": "$profile:backup_config"
          }
        ]
      },
      {
        "name": "TestTrayAbility",
        "srcEntry": "./ets/testtrayability/TestTrayAbility.ets",
        "type": "statusBarView",
        "exported": true
      }
    ],
    "requestPermissions": [
      {
        "name": "ohos.permission.INTERNET"
      },
      {
        "name": "ohos.permission.SET_WINDOW_TRANSPARENT"
      }
    ]
  }
}
```

- `TestTrayAbility` 是系统托盘 QuickOperation 面板的扩展能力，`type` 必须为 `statusBarView`
- `ohos.permission.SET_WINDOW_TRANSPARENT` 是 2in1 设备上 `setWindowContainerColor` API 所需的权限，缺少会导致透明窗口功能失效（error 201）

## 关键注意事项

1. **env.sh 必须先 source** — 设置 OHOS linker/CC/JAVA_HOME，否则 Rust 编译报 `cc not found`
2. **prod feature** — 不加则 app 连接 localhost:1420 而非加载打包前端。脚本已自动包含。
3. **hdc 路径转义** — Git Bash 把 `/data/...` 转为 Windows 路径。设备路径命令用 `hdc shell "cat ..."` 加引号。
4. **签名由 hvigorw 完成** — build-profile.json5 中配置了含 system_basic 权限的证书，hvigorw assembleHap 自动签名。无需手动签名。
5. **HAR 缓存** — 修改了 openharmony-ability 后必须重建 HAR。`run-tests.sh` Step 0 自动检测并处理。
6. **tauriPlugin** — hvigorfile.ts 中的 tauriPlugin 需要 TCP 回调 tauri CLI，独立构建时必须禁用。`build-ohos.sh` 和 `run-tests.sh` 自动处理。
7. **tauri-cli 修改后必须重装** — 如果修改了 `crates/tauri-cli/` 下的 Rust 源码（非模板文件），必须重新安装 tauri-cli，否则 `cargo` 使用缓存的旧二进制，改动不生效：
   ```bash
   cargo install --path crates/tauri-cli --locked
   ```
   仅修改模板文件（`templates/` 下）不需要重装，`build-ohos.sh` 会自动检测模板 mtime 并重建 `gen/ohos/`。

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
cd ${PROJECT_ROOT}/openharmony-ability
source ${PROJECT_ROOT}/tauri/.claude/skills/ohos-build/scripts/env.sh
ohrs build --arch arm64          # 末尾 panic 是已知问题，不影响
bash scripts/pack.sh
tar -czf ability.har package
cmd.exe /c "rmdir /s /q ${PROJECT_ROOT}\\tauri\\examples\\api\\src-tauri\\gen\\ohos\\entry\\oh_modules"
cmd.exe /c "cd /d ${PROJECT_ROOT}\\tauri\\examples\\api\\src-tauri\\gen\\ohos\\entry && ohpm install"
```

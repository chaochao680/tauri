# Tauri OHOS 开发约束与限制

> 本文档记录 Tauri OpenHarmony 适配中所有架构约束、平台限制和开发规则。
> 修改 OHOS 相关代码前必须先阅读并遵守这些规则。

---

## 1. 通用架构约束

### 1.1 cfg 隔离规则

| 规则 | 说明 |
|------|------|
| `cfg(target_env = "ohos")` 是所有设备形态通用 | 不要滥用。OHOS 上的 desktop/mobile 差异用 `cfg(all(target_env = "ohos", desktop))` / `cfg(all(target_env = "ohos", mobile))` |
| `cfg(desktop)` 由 `TAURI_OHOS_DEVICE_TYPE=desktop` 控制 | 包含 tray/menu bar。`cfg(mobile)` 不含 tray/menu |
| 功能在 desktop+mobile 都可用时用 `ohos` cfg | 仅 desktop, 用 `ohos+desktop` |

### 1.2 线程模型：Chrome_IOThread 是 event loop

| 规则 | 说明 |
|------|------|
| **禁止** `run_on_main_thread + rx.recv()` 阻塞模式 | 会死锁： Chrome_IOThread 等 ArkTS 主线程, ArkTS 主线程等 Chrome_IOThread |
| 所有跨线程 NAPI 操作必须用 **TSFN NonBlocking** | `run_on_main_thread` 调度到 Chrome_IOThread (非 ArkTS 主线程), 阻塞 recv 会死锁 |
| Mutex 不得跨越阻塞 I/O 操作持有 | `Arc<Mutex<&'static F>>` 必须 copy 后立即 release: `let cb = *cbs.lock().unwrap(); cb(...);` |
| TrayIcon 是 Sync+Send, 无主线程限制 | tray 操作通过 TSFN 内部处理线程安全, 不限制调用线程 |

### 1.3 Menu 是纯 Rust 数据 + JSON, 不是原生 OS menu

| 规则 | 说明 |
|------|------|
| Menu items 是 Rust `MenuChild` 结构, 通过 JSON 推送到 ArkTS | OHOS 无原生 menubar API (无 HMENU / GTK menu)。整个 menu 系统是自定义实现 |
| 动态更新 (setText/setEnabled) 需要 `refresh_menubar()` | 重新序列化完整 JSON + TSFN 推送。无增量更新机制 |
| Menu 图标通过 **base64 PNG** 编码在 JSON 中 | ArkTS 侧解码为 PixelMap。必须追踪并释放过期 PixelMap (`cleanupStaleIcons`) |
| Menu 文本 `&` (mnemonic) 被静默移除 | `"Save &As"` → `"Save As"`。OHOS 不支持键盘 mnemonic |

### 1.4 Tray 使用 StatusBar API, 不是系统 tray

| 规则 | 说明 |
|------|------|
| Tray icon 通过 `statusBarManager.addToStatusBar()` 实现 | OHOS 无传统系统 tray (Notification Area)。使用桌面扩展 API (`@kit.DeskTopExtensionKit`) |
| Tray 仅在 desktop 模式可用 | mobile 设备无 tray 功能 |
| Tray `rect()` 始终返回 None | StatusBar API 不提供图标位置/尺寸。`AvoidArea.topRect` 返回整个状态栏区域, 不是单个图标 |
| Tray 事件数据有限 | 只有 `iconClickType` ("leftClick"/"rightClick") 和 `menuCode`。无坐标、无双击、无 hover、无中键 |

---

## 2. NAPI / TSFN 规则

### 2.1 napi-derive-ohos 自动 camelCase 转换

- **Rust `#[napi]` 函数名 snake_case → JS camelCase**。如 `emit_menu_event` → `emitMenuEvent`, `on_popup_request` → `onPopupRequest`
- ArkTS 代码必须使用 **camelCase** 名称调用 napi 函数
- 如需保留原名, 必须用 `#[napi(js_name = "original_name")]`
- 使用 snake_case 会导致 `typeof module.on_popup_request !== "function"` 返回 `true` (函数实际名为 `onPopupRequest`), **静默失败不报错**

### 2.2 TSFN 参数传递规则

| 规则 | 说明 |
|------|------|
| TSFN 回调必须返回 **参数元组**, 不是 `Result<()>` | 返回 `()` = 空 JS 参数 (全部 `undefined`)。返回 `FnArgs { data: (arg1, arg2) }` |
| **禁止** 使用 `callee_handled::<true>()` | napi-ohos 在 `CalleeHandled=true` 时自动在首位插入 `null`, 导致参数偏移。必须用 `callee_handled::<false>()` |
| 裸 tuple 类型会序列化为 JS Array | 必须用 `FnArgs<>` 包装 tuple, 否则 JS 函数收到数组而非展开参数 |
| TSFN 数据必须通过泛型参数携带, 不是全局 Mutex | 全局 `Mutex<Option<Data>>` 中转模式在快速连续调用时产生数据竞态, 导致 freeze。每个 TSFN 调用独立 Box 入队, 天然隔离 |

### 2.3 NAPI 上下文限制

| 规则 | 说明 |
|------|------|
| `Function::call()` 在 `render()` 上下文中静默失败 | 不抛错、不执行。必须用 `Object::set()` 设置属性, 让 JS 侧延迟读取 |
| `statusBarManager.on()` 必须在 `addToStatusBar` 之后 200ms 注册 | OHOS 内部 `ScbServerReceiver` 在 `addToStatusBar` 后异步初始化。提前注册的 handler 被静默丢弃 |
| NAPI `Env` 只在获取它的线程有效 | `MAIN_THREAD_ENV` 存储在 `thread_local!` 中, 其他线程调用 `get_main_thread_env()` 返回 `None` |
| `ObjectRef` (napi_ref) 不是 Send/Sync | 必须通过 `Mutex<SendableHelper>` + `ptr::read` 跨线程共享, `unsafe impl Send/Sync` |


## 3. 构建与环境规则

### 3.1 构建环境

| 规则 | 说明 |
|------|------|
| 使用 **Git Bash** 运行构建脚本 | PowerShell/cmd.exe 不兼容 Unix 路径格式 (sed, bash 特性) |
| Rust 交叉编译需要 OHOS clang/sysroot | `CC=clang.exe`, `CFLAGS=--target=aarch64-linux-ohos --sysroot=... -D__MUSL__`, `AR=llvm-ar.exe`, linker=clang.exe |
| 必须使用 `--features prod` 构建标志 | 不加则 app 连接 localhost:1420, 无法加载打包前端 |
| `OHOS_NDK_HOME` 路径不带 `/native` 后缀 | `D:/app/DevEco-Studio/sdk/default/openharmony` (不是 `.../native`) |
| `hdc` 命令中设备路径必须加引号 | Git Bash 会将 `/data/...` 转为 Windows 路径。用 `hdc shell "cat /data/..."` |

### 3.2 HAR 包管理

| 规则 | 说明 |
|------|------|
| 修改 `openharmony-ability` 源码后必须重建 HAR | `ohrs build --arch arm64` + `pack.sh` + `tar -czf ability.har package` + `ohpm install` |
| ohpm install 必须从项目根目录 (`gen/ohos`) 运行 | 不是 entry 子目录 |
| 增量更新时需要 `rmdir /s /q oh_modules` 清理 | 避免 EPERM rename 错误 |
| HAR 重建后 HAP 也必须重建 | ArkTS 代码变更 → HAR → HAP 全链重建 |

### 3.3 签名与部署

| 规则 | 说明 |
|------|------|
| 每次构建生成新的 debug 证书 | 必须先卸载旧版 (`bm uninstall`) 再安装新版 |
| 卸载会清除所有应用数据 | 不适合持久化数据的生产测试 |
| `hvigorw` 需要通过 `cmd.exe /c` 运行 | PowerShell 直接运行可能失败 |
| `hvigorw` 需要 `JAVA_HOME` 和 `DEVECO_SDK_HOME` 环境变量 | 签名步骤需要 Java (`spawn java ENOENT`) |
| `tauriPlugin` 在独立构建时必须禁用 | hvigorfile.ts 中的 tauriPlugin 需要 TCP 回调 tauri CLI |

### 3.5 日志规则

| 规则 | 说明 |
|------|------|
| `log::*!` + `Stdout` target 在 OHOS 上不可见 | stdout 不连接 hilog。必须用 `hilog` crate 或 `TargetKind::Stderr` |
| ArkTS `console.error` 在某些上下文不输出到 hilog | 需要 `hilog` crate 直接写入, 或确认 `console` 在当前上下文可用 |

---
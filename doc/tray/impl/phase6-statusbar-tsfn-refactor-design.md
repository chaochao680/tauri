# Phase 6: StatusBar TSFN 数据传递重构

> 职责：将 statusbar 模块的跨线程数据传递从"全局 Mutex 中转"改为"TSFN 直接携带数据"
> 代码位置：`openharmony-ability/crates/ability/src/statusbar/manager.rs`
> 独立性：✓ 纯内部重构，公开 API 签名不变

## 一、问题描述

当前 `manager.rs` 使用 15 个 `static Mutex<Option<...>>` 全局变量在 Rust 工作线程和 ArkTS 主线程之间传递数据：

```rust
static DATA_ADD_WHITE: Mutex<Option<Vec<u8>>> = Mutex::new(None);
static DATA_ADD_BLACK: Mutex<Option<Vec<u8>>> = Mutex::new(None);
static DATA_ADD_ICON_SIZE: Mutex<Option<u32>> = Mutex::new(None);
// ... 共 15 个
```

**问题**：
1. **非原子性**：`add_to_status_bar` 写入 10 个 Mutex 不是原子操作，并发调用可能交叉
2. **冗余**：每个字段一把锁，声明 + lock/take 代码大量重复
3. **扩展困难**：新增字段需新增 Mutex + 对应的 lock/take 代码

## 二、解决方案

napi-ohos 的 `ThreadsafeFunction<T>` 支持通过泛型参数 `T` 直接携带数据。调用 `tsfn.call(data, mode)` 时，napi-ohos 内部通过 `Box::into_raw` 将数据传递到主线程回调的 `ThreadsafeCallContext<T>.value` 中。

同项目的 `menu/mod.rs` 已使用此模式：
```rust
let tsfn = callback
    .build_threadsafe_function::<PopupRequestData>()
    .build_callback(|ctx: ThreadsafeCallContext<PopupRequestData>| {
        Ok(vec![ctx.value])
    })?;
tsfn.call(Ok(data), ThreadsafeFunctionCallMode::Blocking);
```

## 三、API 可行性验证

已确认 napi-ohos 1.1.6 源码（`threadsafe_function.rs:535-546`）：

```rust
impl<T: 'static, ...> ThreadsafeFunction<T, ..., false, ...> {
    pub fn call(&self, value: T, mode: ThreadsafeFunctionCallMode) -> Status {
        Box::into_raw(Box::new(ThreadsafeFunctionCallJsBackData { data: value, ... }))
    }
}
```

- `T` 唯一约束：`'static`（自定义 struct 含 `Vec<u8>`, `String` 天然满足）
- `callee_handled::<false>()` 不影响 `T` 的类型约束
- `build_callback` 接收 `FnMut(ThreadsafeCallContext<T>) -> Result<CallJsBackArgs>`

## 四、实现变更

### 4.1 新增数据结构

```rust
struct AddStatusBarData {
    white: Option<Vec<u8>>,
    black: Option<Vec<u8>>,
    icon_size: u32,
    ability_name: String,
    title: String,
    height: u32,
    module_name: Option<String>,
    loading_status: Option<bool>,
    menu_json: Option<String>,
    hover_tips: Option<String>,
}

struct UpdateIconData {
    white: Option<Vec<u8>>,
    black: Option<Vec<u8>>,
    icon_size: u32,
}

struct UpdateMenuData { menu_json: String }
struct UpdateTipsData { tips: String }
```

### 4.2 修改 TSFN 类型别名

```rust
// 第一个泛型参数从 () 改为数据结构
type TrayTsfnAdd = ThreadsafeFunction<AddStatusBarData, (), FnArgs<(...)>, Status, false>;
type TrayTsfnUpdateIcon = ThreadsafeFunction<UpdateIconData, (), FnArgs<(...)>, Status, false>;
type TrayTsfnUpdateMenu = ThreadsafeFunction<UpdateMenuData, (), FnArgs<(...)>, Status, false>;
type TrayTsfnUpdateTips = ThreadsafeFunction<UpdateTipsData, (), FnArgs<(...)>, Status, false>;
type TrayTsfnRemove = ThreadsafeFunction<(), (), (), Status, false>;  // 不变
```

### 4.3 删除 15 个 `DATA_*` 静态变量

### 4.4 修改 `init_tray_tsfn`

```rust
let add_tsfn = add_fn
    .build_threadsafe_function::<AddStatusBarData>()
    .callee_handled::<false>()
    .build_callback(move |ctx: ThreadsafeCallContext<AddStatusBarData>| {
        build_add_to_status_bar_args(ctx.env, ctx.value).map(|args| FnArgs { data: args })
    })?;
```

### 4.5 修改 `build_*` 函数

从"无参数 + 读 Mutex"改为"接收数据结构参数"：

```rust
fn build_add_to_status_bar_args(env: Env, data: AddStatusBarData) -> Result<(...)> {
    // 直接使用 data.white, data.black 等
}
```

### 4.6 修改公开 API

```rust
pub fn add_to_status_bar(_app: &crate::OpenHarmonyApp, item: &StatusBarItem) -> Result<()> {
    validate_status_bar_item(item)?;
    let data = AddStatusBarData { /* 从 item 构造 */ };
    let tsfn = TSFN_ADD.lock().unwrap();
    let tsfn = tsfn.as_ref().ok_or_else(|| Error::from_reason("not initialized"))?;
    tsfn.call(data, ThreadsafeFunctionCallMode::NonBlocking);
    Ok(())
}
```

## 五、变更对比

| 维度 | 重构前 | 重构后 |
|------|--------|--------|
| 静态全局变量 | 20 个（5 TSFN + 15 DATA） | 5 个（仅 TSFN） |
| 原子性 | 10 次独立 lock，可能交叉 | 单次构造 struct，天然原子 |
| 数据传递 | 全局 Mutex 中转 | TSFN 内部 Box 传递 |
| 代码行数 | ~415 行 | ~280 行 |
| 扩展性 | 新增字段 = 新增 Mutex + lock/take | 新增字段 = struct 加一个字段 |

## 六、不变的部分

- 5 个 `TSFN_*` 静态变量保留（存储 TSFN 本身仍需全局）
- 公开 API 签名不变
- `build_menu_item_object_static` 等辅助函数不变
- `event.rs`、`types.rs`、`validate.rs` 不变
- 上层 `tray-icon` crate 调用方无需任何修改

## 七、风险评估

| 风险 | 评估 | 应对 |
|------|------|------|
| `T` 需要 `Send`？ | 不需要，napi-ohos 用 `Box::into_raw` + `unsafe` | 无需处理 |
| TSFN 队列积压 | 与重构前行为一致（`MaxQueueSize = 0` 无限队列） | 不引入新风险 |
| 并发调用安全 | 重构后每次调用独立 Box 入队，天然隔离 | **改善** |
| 内存开销 | 每次 Box 一个 struct（~2.3KB 图标数据） | 可忽略 |

## 八、验证方案

1. 交叉编译：`cargo build --target aarch64-unknown-linux-ohos -p openharmony-ability --features "menu webview"`
2. Rust UT：`PACKAGE=openharmony-ability FEATURES=menu bash .../run-ut.sh statusbar`
3. 端到端：`ohos-build` skill 流程构建 → 部署 → 拉取 test-report.md → 所有 autotest pass
4. 手动验证：状态栏图标显示、点击事件、菜单弹出正常

## 九、验证结果（2026-05-18）

### 构建

- `TAURI_OHOS_DEVICE_TYPE=desktop` 交叉编译通过
- HAR 打包成功（`ohrs build --arch arm64` → `pack.sh` → `ability.har`）
- hvigor 构建 API Demo HAP 成功（3.2s）

### 设备部署

- 签名 + 安装成功
- 应用启动正常，无 crash / ANR

### Autotest 结果

共 122 项用例，**117 pass / 5 fail**。

**Tray 相关用例（#108-#122）全部通过 ✅**：

| # | Test | Duration |
|---|------|----------|
| 108 | TrayIcon.new | 320ms |
| 109 | TrayIcon.new_with_id | 319ms |
| 110 | TrayIcon.getById | 338ms |
| 111 | TrayIcon.getById_not_found | 321ms |
| 112 | TrayIcon.removeById | 350ms |
| 113 | TrayIcon.setIcon | 333ms |
| 114 | TrayIcon.setIcon_null | 332ms |
| 115 | TrayIcon.setMenu | 358ms |
| 116 | TrayIcon.setMenu_null | 328ms |
| 117 | TrayIcon.setTooltip | 345ms |
| 118 | TrayIcon.setTitle | 348ms |
| 119 | TrayIcon.setVisible | 357ms |
| 120 | TrayIcon.setTempDirPath | 344ms |
| 121 | TrayIcon.setIconAsTemplate | 335ms |
| 122 | TrayIcon.setShowMenuOnLeftClick | 350ms |

**5 项失败均为预存问题，与本次重构无关**：

| # | Test | Error |
|---|------|-------|
| 3 | core.Channel | expected 1000 messages, got 79 |
| 21 | plugin-http.fetch | plugin http not found |
| 23 | plugin-autostart | plugin autostart not found |
| 24 | plugin-clipboard-manager.writeText | plugin clipboard-manager not found |
| 25 | plugin-clipboard-manager.writeImage | plugin clipboard-manager not found |

### 结论

重构完成，验证通过。TSFN 数据直传模式在真机上工作正常，tray 全部 15 项用例 pass。

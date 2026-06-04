---
name: phase4-clipboard-review-fixes
description: PR检视意见修复进展跟踪
metadata:
  type: project
---

# Phase 4 Clipboard writeImage — PR 检视意见修复进展

> 三条 PR: openharmony-ability#7 / plugins-workspace#3 / tauri#14
> 检视者: MingyuChen1

---

## openharmony-ability#7 (10 条)

| # | 类型 | 问题 | 状态 | 修复方案 |
|---|------|------|------|----------|
| B1 | 🔴 | call_with_return_value Status 未检查 | ✅ | 检查 Status 非 Ok 则返回 Error |
| M1 | 🟡 | unsafe cast PromiseRaw 无类型验证 | ✅ | cast 前验证 get_type() == Object |
| M2 | 🟡 | rx.await 无超时 | ✅ | 加 tokio::time::timeout(10s) |
| M3 | 🟡 | Mutex::lock().unwrap() poisoned panic | ❌不采纳 | lock().unwrap() 是 Rust 常用模式，无实际风险，保持原样 |
| M4 | 🟡 | catch 硬编码 "rejected" 丢弃 BusinessError | ✅ | coerce_to_string 提取错误详情 |
| M5 | 🟡 | rgba 尺寸未校验 | ✅ | checked_mul + 尺寸校验 |
| m1 | 🔵 | PixelMap 未显式 release | ✅ | catch 路径 pm?.release() |
| m2 | 🔵 | _onIconClick/_onMenuClick 范围不当 | ✅ | 从 type.ets 移除，属于 tray/menu PR |
| m3 | 🔵 | 文件末尾缺换行符 | ✅ | 添加换行符 |
| m4 | 🔵 | #[cfg(feature="clipboard")] 冗余 | ✅ | 移除 |

### B1 — call_with_return_value Status 未检查

**文件**: `crates/ability/src/clipboard/mod.rs`
**改动**: 将 `tsfn.call_with_return_value(...)` 返回值赋给 `call_status`，检查 `call_status != Status::Ok` 则返回 Error

```rust
let call_status = tsfn.call_with_return_value(
    data,
    ThreadsafeFunctionCallMode::NonBlocking,
    move |result, _env| { ... },
);
if call_status != Status::Ok {
    hilog_error!("clipboard_write_image: TSFN call failed: {:?}", call_status);
    return Err(Error::from_reason(format!("TSFN call failed: {:?}", call_status)));
}
```

### M1 — unsafe cast PromiseRaw 无类型验证

**文件**: `crates/ability/src/clipboard/mod.rs`
**改动**: cast 前用 `value.get_type()` 验证返回值是 Object（Promise），否则通过 oneshot 发送错误并 `return Ok(())`

```rust
Ok(value) => {
    let value_type = value.get_type()?;
    if value_type != napi_ohos::ValueType::Object {
        let _ = tx.send(Err(Error::from_reason(
            "writeImageToClipboard did not return a Promise"
        )));
        return Ok(());
    }
    let tx_cell = Rc::new(Cell::new(Some(tx)));
    ...
```

### M2 — rx.await 无超时

**文件**: `crates/ability/src/clipboard/mod.rs`
**改动**: 添加 `use tokio::time::{timeout, Duration};`，将 `rx.await` 改为 `timeout(Duration::from_secs(10), rx).await`

```rust
timeout(Duration::from_secs(10), rx)
    .await
    .map_err(|_| Error::from_reason("clipboard write timed out"))?
    .map_err(|_| Error::from_reason("clipboard write cancelled"))?
```

### M3 — Mutex::lock().unwrap() poisoned panic

**决定**: ❌ 不采纳。`lock().unwrap()` 是 Rust 标准模式，mutex poisoning 在此场景下无实际风险。保持原样。

### M4 — catch 硬编码 "rejected" 丢弃 BusinessError

**文件**: `crates/ability/src/clipboard/mod.rs`
**改动**: catch handler 从 `ctx.value.coerce_to_string()` 提取 ArkTS 错误详情，替代硬编码 `"writeImageToClipboard rejected"`

```rust
.catch(move |ctx: CallbackContext<Unknown>| {
    if let Some(sender) = tx_in_catch.replace(None) {
        let reason = ctx.value.coerce_to_string()
            .and_then(|s| s.into_utf8().and_then(|u| u.into_owned()))
            .unwrap_or_else(|_| "unknown".to_string());
        let _ = sender.send(Err(Error::from_reason(format!("rejected: {}", reason))));
    }
    Ok(())
})
```

### M5 — rgba 尺寸未校验

**文件**: `crates/ability/src/clipboard/mod.rs`
**改动**: 在 `clipboard_write_image` 函数开头添加 `checked_mul` 溢出保护 + 尺寸校验

```rust
let expected = (width as usize).checked_mul(height as usize)
    .and_then(|v| v.checked_mul(4))
    .ok_or_else(|| Error::from_reason("dimensions overflow"))?;
if rgba.len() != expected {
    hilog_error!("clipboard_write_image: rgba len {} != expected {} ({}x{}x4)", rgba.len(), expected, width, height);
    return Err(Error::from_reason(format!(
        "rgba len {} != expected {} ({}x{}x4)",
        rgba.len(), expected, width, height)));
}
```

### m1 — PixelMap 未显式 release

**文件**: `native_ability/src/main/ets/helper/ClipboardHelper.ets`
**改动**: 将 `pm` 提升到函数级 `let pm: image.PixelMap | null = null;`，catch 路径添加 `pm?.release()`

```typescript
let pm: image.PixelMap | null = null;
try {
    pm = image.createPixelMapSync(opts);
    ...
} catch (e) {
    pm?.release();
    ...
}
```

### m2 — _onIconClick/_onMenuClick 范围不当

**文件**: `native_ability/src/main/ets/ability/type.ets`
**改动**: 从 ArkHelper interface 中移除 `_onIconClick` 和 `_onMenuClick`，这两个属性属于 tray/menu 功能，不应在本 clipboard PR 中

```typescript
// 移除前
writeImageToClipboard: (rgbaData: Uint8Array, width: number, height: number) => Promise<void>;
_onIconClick?: Callback<emitter.EventData>;
_onMenuClick?: Callback<emitter.EventData>;

// 移除后
writeImageToClipboard: (rgbaData: Uint8Array, width: number, height: number) => Promise<void>;
```

### m3 — 文件末尾缺换行符

**文件**: `crates/ability/src/clipboard/mod.rs`, `native_ability/.../helper/ClipboardHelper.ets`
**改动**: 两个文件末尾各添加一行换行符

### m4 — #[cfg(feature="clipboard")] 冗余

**文件**: `crates/ability/src/clipboard/mod.rs`
**改动**: 移除 `#[cfg(feature = "clipboard")]`，改为无条件 import

```rust
// 改前
#[cfg(feature = "clipboard")]
use ohos_hilog_binding::{hilog_error, LogOptions, set_global_options};
// 改后
use ohos_hilog_binding::{hilog_error, LogOptions, set_global_options};
```

---

## 第二轮检视意见（push 后）

### openharmony-ability#7（2 条 🔵 Minor）

| # | 类型 | 问题 | 状态 | 修复方案 |
|---|------|------|------|----------|
| B-new1 | 🔵 | init_clipboard_tsfn 无幂等保护 | ✅ | AtomicBool + Acquire/Release |
| B-new2 | 🔵 | PixelMap 成功路径未 finally release | ✅ | catch→catch+finally |

#### B-new1 — init_clipboard_tsfn 幂等保护

**文件**: `crates/ability/src/clipboard/mod.rs`
**改动**: 新增 `static TSFN_INITIALIZED: AtomicBool = AtomicBool::new(false);`，在 `init_clipboard_tsfn` 入口用 `Ordering::Acquire` 检查，成功后 `Ordering::Release` 设置。后续热重载等场景不会覆盖旧 TSFN。

```rust
static TSFN_INITIALIZED: AtomicBool = AtomicBool::new(false);

pub fn init_clipboard_tsfn(env: &Env) -> Result<()> {
    if TSFN_INITIALIZED.load(Ordering::Acquire) {
        return Ok(());
    }
    // ... existing init logic ...
    TSFN_INITIALIZED.store(true, Ordering::Release);
    Ok(())
}
```

#### B-new2 — PixelMap finally release

**文件**: `native_ability/src/main/ets/helper/ClipboardHelper.ets`
**改动**: 将 `pm?.release()` 从 catch 移到 `finally` 块，确保成功路径也释放（剪贴板图片可能数 MB）

```typescript
try {
    // ... existing code ...
} catch (e) {
    const err = e as BusinessError;
    console.error(`[Clipboard] writeImageToClipboard failed: ${err.code} ${err.message}`);
    throw new Error(`writeImageToClipboard failed: code=${err.code} msg=${err.message}`);
} finally {
    pm?.release();
}
```

### plugins-workspace#3（2 🟡 Major + 1 🔵 Minor）

| # | 类型 | 问题 | 状态 | 修复方案 |
|---|------|------|------|----------|
| P-new1 | 🟡 | api-iife.js i() 仍残留 instanceof | ✅ | 手动改为 duck-type |
| P-new2 | 🟡 | [patch.crates-io] 相对路径 | ✅ | 加 TODO 注释跟踪 |
| P-new3 | 🔵 | desktop.rs EOF 缺换行符 | ✅ | 添加换行符 |

#### P-new1 — api-iife.js i() instanceof 消除

**文件**: `plugins/clipboard-manager/api-iife.js`
**改动**: 将 `function i(e){return null==e?null:"string"==typeof e?e:e instanceof a?e.rid:e}` 改为 `function i(e){return null==e?null:"string"==typeof e?e:"number"==typeof e.rid?e.rid:e}`，与 writeImage 的 duck-type 一致

#### P-new2 — [patch.crates-io] 路径依赖跟踪

**文件**: `plugins/clipboard-manager/Cargo.toml`
**改动**: 在 `openharmony-ability` 依赖行上方加 `# TODO: Remove path dependency after openharmony-ability is published to crates.io / merged into workspace`

#### P-new3 — desktop.rs EOF newline

**文件**: `plugins/clipboard-manager/src/desktop.rs`
**改动**: 文件末尾添加换行符

### tauri#14（1 ℹ️ Info + 1 🔵 Minor）

| # | 类型 | 问题 | 状态 | 修复方案 |
|---|------|------|------|----------|
| T-new1 | ℹ️ | 设计文档迁移到 openspec | ✅ | 迁移到 openspec/changes/archive/ |
| T-new2 | 🔵 | writeImage(path) 临时文件未清理 | ✅ | 添加 removeFile |

#### T-new1 — 设计文档迁移到 openspec

**原位置**: `doc/ohos_feature/impl/phase4-clipboard-manager-writeImage-ohos-design.md`
**新位置**: `openspec/changes/archive/2026-06-04-clipboard-writeimage/design.md`
**改动**: 将 phase4 的设计文档、进展文档、review-fixes 文档全部从 `doc/ohos_feature/impl/` 迁移到 `openspec/changes/archive/` 下，与 openspec 工作流统一管理

#### T-new2 — writeImage(path) 临时文件清理

**文件**: `examples/api/src/lib/tests/plugins.ts`
**改动**: 在 writeImage(path) 测试中，调用 writeImage 后用 `remove(filePath)` 清理临时 PNG

```typescript
await writeImage(filePath);
// Clean up temp file after test
const { remove } = await import('@tauri-apps/plugin-fs');
await remove(filePath);
```

---

## plugins-workspace#3 (7 条)

| # | 类型 | 问题 | 状态 | 修复方案 |
|---|------|------|------|----------|
| B2 | 🔴 | arboard cfg 缺 not(target_env="ohos") | ✅ | 添加 not(target_env = "ohos") |
| M6 | 🟡 | desktop.rs arboard 类型未隔离 | ✅ | 按4.2.2双版本方案重构 |
| M7 | 🟡 | OHOS stub 返回不透明错误 | ✅ | PlatformNotAvailable + TODO |
| M8 | 🟡 | (image as any).rid 绕过 TS | ✅ | RidHolder interface |
| m5 | 🔵 | JS docblock 缺 OHOS 支持说明 | ✅ | 补充文档 |
| m6 | 🔵 | api-iife.js 构建问题 | ⬜ | 需先让 plugins-workspace 引用本地 tauri api dist |
| m7 | 🔵 | #[allow(unused)] 无注释 | ✅ | 加注释 |

### B2 — arboard cfg 缺 not(target_env = "ohos")

**文件**: `plugins/clipboard-manager/Cargo.toml`
**改动**: arboard 依赖的 cfg 条件从 `cfg(any(...))` 改为 `cfg(all(any(...), not(target_env = "ohos")))`

```toml
# 改前
[target."cfg(any(target_os = \"macos\", windows, ...))".dependencies]
arboard = { version = "3", features = ["wayland-data-control"] }

# 改后
[target."cfg(all(any(target_os = \"macos\", windows, ...), not(target_env = \"ohos\")))".dependencies]
arboard = { version = "3", features = ["wayland-data-control"] }
```

### M6 — desktop.rs arboard 类型未隔离

**文件**: `plugins/clipboard-manager/src/desktop.rs`
**改动**: 整文件重构为两个完全独立的版本：
- `#[cfg(all(desktop, not(target_env = "ohos")))]`: 原有 arboard 版，struct 含 `clipboard: Result<Mutex<Option<arboard::Clipboard>>, arboard::Error>`
- `#[cfg(target_env = "ohos")]`: 无 arboard，struct 只有 `app`，各方法返回 `"not supported on OHOS"` 错误 + TODO 注释

### M7 — OHOS stub 返回不透明错误

**文件**: `plugins/clipboard-manager/src/desktop.rs` (与 M6 同一重构)
**改动**: OHOS 版各方法返回明确的错误消息如 `"write_text not supported on OHOS (only write_image is available)"`，而非之前的 `"ContentNotAvailable"`

### M8 — (image as any).rid 绕过 TS 类型检查

**文件**: `plugins/clipboard-manager/guest-js/index.ts`
**改动**: 定义 `interface RidHolder { rid: number }`，替换所有 `(image as any).rid` 为 `(image as RidHolder).rid`

```typescript
// 改前
typeof (image as any).rid === 'number' ? (image as any).rid : image
// 改后
interface RidHolder { rid: number }
typeof (image as RidHolder).rid === 'number' ? (image as RidHolder).rid : image
```

### m5 — JS docblock 缺 OHOS 支持说明

**文件**: `plugins/clipboard-manager/Cargo.toml` + `guest-js/index.ts`
**改动**:
- Cargo.toml 添加 `ohos = { level = "partial", notes = "write_image only..." }`
- writeImage docblock 补充 `**HarmonyOS (OHOS):** Supported via ArkTS PixelMap bridge.`
- readImage docblock 补充 `**HarmonyOS (OHOS):** Not supported (write only).`

### m6 — api-iife.js 构建问题

**状态**: ⬜ 未完成
**问题**: IIFE 打包时 `transformImage`（函数 `i()`）仍用 `instanceof`，因为 `nodeResolve` 内联了 npm 上的旧版 `@tauri-apps/api@2.11.0`（未含 duck-type 改动）
**需要的操作**: 让 plugins-workspace 的 `@tauri-apps/api` 引用本地 tauri repo 的最新 dist，然后 `pnpm install` + 重新 `pnpm build`

### m7 — #[allow(unused)] 无注释

**文件**: `plugins/clipboard-manager/src/commands.rs`
**改动**: 在 `#[allow(unused)]` 上方添加注释 `// unused on OHOS (TSFN bridge), used on desktop`

---

## tauri#14 (4 条)

| # | 类型 | 问题 | 状态 | 修复方案 |
|---|------|------|------|----------|
| M8 | 🟡 | (image as any).rid → RidHolder | ✅ | RidHolder interface |
| m8 | 🔵 | rid+rgba 同时存在加 debug log | ✅ | log::debug |
| m9 | 🔵 | .plugin() 缩进不对 | ✅ | 修正缩进 |
| C1 | ⚠️ | dist 未 rebuild | ✅ | pnpm build 完成 |

### M8 (tauri) — (image as any).rid → RidHolder

**文件**: `packages/api/src/image.ts`
**改动**: 在 `transformImage` 函数内定义 `interface RidHolder { rid: number }`，替换 `(image as any).rid`

```typescript
// 改前
: typeof (image as any).rid === 'number'
  ? (image as any).rid
  : image

// 改后
interface RidHolder { rid: number }
: typeof (image as RidHolder).rid === 'number'
  ? (image as RidHolder).rid
  : image
```

### m8 — rid+rgba 同时存在加 debug log

**文件**: `crates/tauri/src/image/mod.rs`
**改动**: 在 `visit_map` 中 rid 优先返回前，检查是否有 rgba 同时存在，加 `log::debug`

```rust
if let Some(rid) = rid {
    if rgba.is_some() {
        log::debug!("JsImage::visit_map: both rid and rgba present, using rid={}", rid);
    }
    return Ok(JsImage::Resource(rid));
}
```

### m9 — .plugin() 缩进不对

**文件**: `examples/api/src-tauri/src/lib.rs`
**改动**: OHOS 配置块内 `.plugin(` 从列 0 改为列 4，对齐链式调用

```rust
// 改前
let builder = builder
.plugin(
// 改后
let builder = builder
    .plugin(
```

### C1 — dist 未 rebuild

**文件**: `packages/api/dist/image.js`, `dist/image.cjs`, `dist/image.d.ts`
**改动**: 在 `packages/api` 目录执行 `pnpm build`，重新生成 dist 文件
**验证**: `dist/image.js` 中 `transformImage` 使用 duck-type (`typeof image.rid === 'number'`)，不再有 `instanceof` 或 `as any`

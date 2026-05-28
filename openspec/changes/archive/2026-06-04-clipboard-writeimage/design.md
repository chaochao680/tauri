# Phase 4: clipboard-manager writeImage() OHOS 适配设计

> 版本：v3.7
> 更新时间：2026-06-03
> 状态：真机验证通过 ✅（含全类型 autotest + 手动用例 + fs plugin 替代 save_png_to_cache）

---

## 〇、实现进展

| 文件 | 状态 | 说明 |
|------|------|------|
| `openharmony-ability/crates/ability/src/clipboard/mod.rs` | ✅ TSFN 改造 + 日志清理 | Rust `clipboard_write_image()` + `init_clipboard_tsfn()` TSFN 模式；hilog_info 已移除，保留 hilog_error |
| `openharmony-ability/crates/ability/Cargo.toml` | ✅ 已修改 | 新增 `clipboard = []` feature |
| `openharmony-ability/crates/ability/src/lib.rs` | ✅ 已修改 | 新增 `#[cfg(feature="clipboard")] pub mod clipboard` + re-export `init_clipboard_tsfn` |
| `openharmony-ability/crates/ability/src/render/xcomponent.rs` | ✅ 已修改 | render() 中添加 `#[cfg(feature="clipboard")] init_clipboard_tsfn(env)` |
| `openharmony-ability/native_ability/.../helper/ClipboardHelper.ets` | ✅ 已创建 | ArkTS writeImageToClipboard（PixelMap + pasteboard） |
| `openharmony-ability/native_ability/.../helper/index.ets` | ✅ 已修改 | 新增 ClipboardHelper 导出 |
| `openharmony-ability/native_ability/.../ability/type.ets` | ✅ 已修改 | ArkHelper interface 新增 writeImageToClipboard |
| `openharmony-ability/native_ability/.../ability/ArkHelper.ets` | ✅ 已修改 | 新增 import + 方法实现 |
| `clipboard-manager/Cargo.toml` | ✅ 已修改 | arboard 排除 ohos + 新增 openharmony-ability path dependency |
| `clipboard-manager/src/desktop.rs` | ✅ 已重写 | OHOS 上 arboard init 返回 Error，write_image 整体排除 OHOS（由 commands.rs TSFN 桥接处理） |
| `clipboard-manager/src/error.rs` | ✅ 已修改 | arboard From impl 改为 `cfg(all(desktop, not(target_env = "ohos")))` |
| `clipboard-manager/src/commands.rs` | ✅ 已修改 | OHOS write_image 使用 TSFN 桥接替代 arboard，block scope 解决 MutexGuard Send 问题 |
| `clipboard-manager/guest-js/index.ts` | ✅ 已修改 | writeImage 内联 duck-type 检查替代 transformImage（避免 Vite chunk 分裂） |
| 编译验证 | ✅ 已完成 | cargo tauri ohos build 成功（含 TSFN 改造） |
| 设备测试 | ✅ 已完成 | 真机验证通过：跨应用粘贴图片成功 |
| examples/api autotest | ✅ 已完成 | writeImage 全类型通过（rgba/data-uri/rid/4x4/Uint8Array/path/number[]/ArrayBuffer） |
| examples/api 手动用例 | ✅ 已完成 | TestRunner.svelte 新增 7 个 Clipboard writeImage 手动测试按钮 |
| doc/manual_tests.md | ✅ 已完成 | 三、Clipboard 章节：7 用例（4 T0 + 3 T1） |

---

## 一、背景

clipboard-manager 插件当前将 ohos 路由到 `desktop.rs`（使用 `arboard` crate），但 `arboard` 依赖 X11/Wayland/Win32 剪贴板 API，无法在 HarmonyOS 上工作。本次仅适配 `writeImage()`。

**参考实现**：Tray + Menu 的 PR（muda#1、tray-icon#1、openharmony-ability#2、tauri#8），该实现已合并并验证通过。

---

## 二、架构模式——与 Tray/Menu PR 保持一致

Tray/Menu PR 的做法：

| 仓库 | 改动 |
|------|------|
| **openharmony-ability** | 新增 Rust 模块（`statusbar/`、`menu/`）+ ArkTS 辅助文件 |
| **muda** | `Cargo.toml` 添加 `openharmony-ability` path dependency + 新增 `platform_impl/ohos/mod.rs` 使用 `openharmony_ability::menu::MenuItemData` |
| **tray-icon** | `Cargo.toml` 添加 `openharmony-ability` path dependency + 新增 `platform_impl/ohos/mod.rs` 使用 `openharmony_ability::statusbar::*` |
| **tauri** | `Cargo.toml` patch muda/tray-icon + tauri 层适配 |

clipboard-manager 适配遵循同一模式：

| 仓库 | 改动 |
|------|------|
| **openharmony-ability** | 新增 Rust `clipboard` 模块 + ArkTS `ClipboardHelper.ets` |
| **clipboard-manager** | `Cargo.toml` 添加 `openharmony-ability` path dependency + `desktop.rs` 内部 cfg 条件化调用 `openharmony_ability::clipboard::*` |

---

## 三、数据流

```
JS (writeImage({ rgba, width, height }))
  → guest-js/index.ts transformImage → invoke('write_image', { image: { rgba, width, height } })
    → commands.rs write_image (async, 无 cfg 变化)
      → JsImage::Rgba { rgba, width, height } → into_img() → Image { rgba, width, height }
      → desktop.rs Clipboard::write_image(&self, image)
        → [ohos] openharmony_ability::clipboard::clipboard_write_image(image.rgba(), image.width(), image.height())
          → TSFN: tsfn.call(ClipboardImageData { rgba, width, height }, NonBlocking)
          → TSFN callback: FnArgs((Uint8Array::new(rgba), width, height))
          → ArkTS 主线程: ArkHelper.writeImageToClipboard(rgbaData, width, height)
            → ClipboardHelper.ets: jsArr.set(rgbaData) → createPixelMapSync(editable:true) → writeBufferToPixelsSync → pasteboard.setData
        → [desktop] arboard::Clipboard::set_image (不变)
```

---

## 四、文件改动清单

### 4.1 openharmony-ability

#### 4.1.1 Rust 端

**`openharmony-ability/crates/ability/src/clipboard/mod.rs`** (新建)

```rust
//! OHOS Clipboard module
//!
//! This module provides:
//! - Rust API: clipboard_write_image() for clipboard-manager to write images

use napi_ohos::bindgen_prelude::*;
use crate::{get_helper, get_main_thread_env};

pub fn clipboard_write_image(rgba: &[u8], width: u32, height: u32) -> Result<()> {
    let helper = unsafe { get_helper() };
    let helper_borrow = helper.borrow();
    let helper_ref = helper_borrow.as_ref().ok_or_else(|| {
        Error::from_reason("ArkHelper not initialized")
    })?;

    let env_rc = get_main_thread_env();
    let env_borrow = env_rc.borrow();
    let env_ref = env_borrow.as_ref().ok_or_else(|| {
        Error::from_reason("Main thread env not initialized")
    })?;

    let helper_object = helper_ref.get_value(env_ref)?;
    let write_fn: Function<'_, (Uint8Array, u32, u32), ()> = helper_object
        .get_named_property("writeImageToClipboard")?;

    write_fn.call((Uint8Array::from(rgba.to_vec()), width, height))?;
    Ok(())
}
```

**`openharmony-ability/crates/ability/src/lib.rs`** (新增模块导出)

```rust
mod clipboard;  // 新增

pub use clipboard::*;  // 新增
```

**`openharmony-ability/crates/ability/Cargo.toml`** (新增 clipboard feature)

```toml
[features]
default = []
drag_and_drop = []
webview = ["dep:ohos-web-binding", "dep:http"]
menu = []
clipboard = []  # 新增
```

#### 4.1.2 ArkTS 端

**`openharmony-ability/native_ability/src/main/ets/helper/ClipboardHelper.ets`** (新建)

```typescript
import pasteboard from '@ohos.pasteboard';
import { image } from "@kit.ImageKit";
import { BusinessError } from "@kit.BasicServicesKit";

export async function writeImageToClipboard(
  rgbaData: Uint8Array, width: number, height: number
): Promise<void> {
  try {
    // Copy napi-provided Uint8Array into a standard JS-managed buffer.
    // napi external buffers may not be properly accessible by ArkTS PixelMap APIs.
    const jsArr = new Uint8Array(rgbaData.length);
    jsArr.set(rgbaData);

    const opts: image.InitializationOptions = {
      editable: true,
      pixelFormat: image.PixelMapFormat.RGBA_8888,
      size: { width: width, height: height }
    };
    const pm = image.createPixelMapSync(opts);
    pm.writeBufferToPixelsSync(jsArr.buffer);

    const pasteData = pasteboard.createData(pasteboard.MIMETYPE_PIXELMAP, pm);
    const systemPasteboard = pasteboard.getSystemPasteboard();
    await systemPasteboard.setData(pasteData);
  } catch (e) {
    const err = e as BusinessError;
    console.error(`[Clipboard] writeImageToClipboard failed: ${err.code} ${err.message}`);
    throw new Error(`writeImageToClipboard failed: code=${err.code} msg=${err.message}`);
  }
}
```

**关键设计决策**（真机调试发现的问题）：

1. **`editable: true`**：OHOS `image.createPixelMapSync({ editable: false })` 创建不可修改的 PixelMap，`writeBufferToPixelsSync()` 会报错 "pixelmap data is not editable or modifiable" 并静默失败。必须设为 `editable: true`。

2. **napi buffer 复制**：napi-ohos 创建的 `Uint8Array`（通过 `Uint8Array::new(vec)` 从 Rust 侧传入）的外部 `.buffer` 属性 ArkTS 系统 API 无法正确读取。必须用 `jsArr.set(rgbaData)` 将数据拷贝到 JS 引擎管理的标准 ArrayBuffer 后再传给 `writeBufferToPixelsSync`。

3. **SDK API 版本**：SDK 5.0.0(12) 不存在 `PasteDataType` enum（仅 SDK 14+ 有），必须用 `pasteboard.createData(pasteboard.MIMETYPE_PIXELMAP, pixelMap)` 字符串 MIME 方式创建 PasteData。

4. **`createPixelMapSync` + `writeBufferToPixelsSync`**：使用同步 API 创建空 PixelMap 再写入 buffer，而非 `createPixelMap(buffer, options)` 异步方式（后者在 TSFN 上下文中行为不稳定）。

**`openharmony-ability/native_ability/src/main/ets/helper/index.ets`** (新增导出)

```typescript
export * from "./ClipboardHelper";
```

**`openharmony-ability/native_ability/src/main/ets/ability/type.ets`** (ArkHelper interface)

```typescript
writeImageToClipboard: (rgbaData: Uint8Array, width: number, height: number) => Promise<void>;
```

**`openharmony-ability/native_ability/src/main/ets/ability/ArkHelper.ets`** (createArkHelper)

```typescript
import { writeImageToClipboard } from "../helper/ClipboardHelper";

writeImageToClipboard: async (rgbaData: Uint8Array, width: number, height: number): Promise<void> => {
  return await writeImageToClipboard(rgbaData, width, height);
},
```

### 4.2 clipboard-manager

#### 4.2.1 Cargo.toml

**`plugins-workspace/plugins/clipboard-manager/Cargo.toml`**

```toml
# 修改: arboard 排除 ohos
[target."cfg(all(any(target_os = \"macos\", windows, \
  target_os = \"linux\", target_os = \"dragonfly\", \
  target_os = \"freebsd\", target_os = \"openbsd\", \
  target_os = \"netbsd\"), not(target_env = \"ohos\")))".dependencies]
arboard = { version = "3", features = ["wayland-data-control"] }

# 新增: ohos 依赖（与 muda/tray-icon pattern 一致）
[target.'cfg(target_env = "ohos")'.dependencies]
openharmony-ability = { path = "../../../openharmony-ability/crates/ability", features = ["clipboard"] }
```

#### 4.2.2 desktop.rs —— 内部 cfg 条件化

**`plugins-workspace/plugins/clipboard-manager/src/desktop.rs`**

所有方法添加 `#[cfg(all(desktop, not(target_env = "ohos")))]` 和 `#[cfg(target_env = "ohos")]` 双版本：

| 方法 | desktop 版 | ohos 版 |
|------|-----------|---------|
| `init()` | arboard::Clipboard::new() | 无 arboard 字段 |
| `write_image()` | arboard set_image | `openharmony_ability::clipboard_write_image` |
| `write_text()` | arboard set_text | 返回 Error（未支持） |
| `read_text()` | arboard get_text | 返回 Error（未支持） |
| `read_image()` | arboard get_image | 返回 Error（未支持） |
| `write_html()` | arboard set_html | 返回 Error（未支持） |
| `clear()` | arboard clear | 返回 Error（未支持） |
| `cleanup()` | arboard take | 无操作 |

#### 4.2.3 error.rs

```rust
#[cfg(all(desktop, not(target_env = "ohos")))]
impl From<arboard::Error> for Error {
    fn from(error: arboard::Error) -> Self {
        Self::Clipboard(error.to_string())
    }
}
```

#### 4.2.4 零改动文件

| 文件 | 说明 |
|------|------|
| `lib.rs` | 模块加载、类型导出、init、on_event——全部不变 |
| `commands.rs` | 所有命令 cfg 不变 |
| `mobile.rs` | android/ios——不变 |
| `android/` | Kotlin——不变 |
| `ios/` | Swift——不变 |
| `guest-js/index.ts` | JS 前端——不变 |
| `permissions/` | 权限配置——不变 |
| `build.rs` | 构建脚本——不变 |

---

## 五、与 Tray/Menu PR 的做法对比

| 方面 | muda/tray-icon PR | clipboard-manager 本设计 | 一致性 |
|------|-------------------|--------------------------|--------|
| openharmony-ability Rust 模块 | 新增 `statusbar/`、`menu/` | 新增 `clipboard/` | ✓ 一致 |
| openharmony-ability ArkTS 辅助 | 新增 `menu.ets`、`menu_types.ets` 等 | 新增 `ClipboardHelper.ets` | ✓ 一致 |
| ArkHelper 添加方法 | 添加 `addToStatusBarWithRgba` 等 | 添加 `writeImageToClipboard` | ✓ 一致 |
| Cargo.toml 添加 openharmony-ability | `path = "../openharmony-ability/crates/ability"` | `path = "../../../openharmony-ability/crates/ability"` | ✓ 一致（仅路径因目录层级不同） |
| Cargo.toml 排除 ohos | Linux 依赖加 `not(target_env = "ohos")` | arboard 依赖加 `not(target_env = "ohos")` | ✓ 一致 |
| Rust 调用方式 | `use openharmony_ability::menu::MenuItemData` | `use openharmony_ability::clipboard::clipboard_write_image` | ✓ 一致 |
| napi-ohos 依赖 | 不单独添加，通过 openharmony-ability 获得 | 不单独添加，通过 openharmony-ability 获得 | ✓ 一致 |

---

## 六、待做项（按里程碑组织）

> 🔴 阻断本地 commit/push　🟡 阻断正式发布（crates.io/合入上游）　🟢 后续迭代

| # | 问题 | 里程碑 | 状态 | 说明 |
|---|------|--------|------|------|
| 1 | 线程安全 | — | ✅ | TSFN 模式，clipboard_write_image 通过 TSFN 在主线程执行 |
| 2 | 同步/异步 | — | ✅ | async await + oneshot channel + call_with_return_value，与 Desktop 同步行为一致 |
| 3 | path dependency 跨 git repo | 🟡 | ✅ | clipboard-manager Cargo.toml 使用 path dependency（与 tray-icon/muda/tauri 一致），plugins-workspace Cargo.toml 新增 `[patch.crates-io]` openharmony-ability 行 |
| 4 | clipboard feature flag | — | ✅ | openharmony-ability 新增 `clipboard` feature |
| 5 | PixelMap editable | — | ✅ | `editable: true`，否则 `writeBufferToPixelsSync` 报错 |
| 6 | napi 外部 buffer 不可读 | — | ✅ | `jsArr.set(rgbaData)` 复制到 JS 管理的 buffer |
| 7 | OHOS webview JS `new Image()` 不触发 onload | — | ✅ | demo 中使用 DOM ref 替代（仅影响 demo，不影响插件本身） |

---

## 七、验证流程

```bash
# 1. openharmony-ability 编译验证（clipboard feature）
cargo build -p openharmony-ability --target aarch64-unknown-linux-ohos --features clipboard

# 2. clipboard-manager 编译验证
cargo build -p tauri-plugin-clipboard-manager --target aarch64-unknown-linux-ohos

# 3. tauri 整体编译验证
cargo build -p tauri --target aarch64-unknown-linux-ohos

# 4. 全量构建 + HAP 打包
bash build-ohos.sh

# 5. 安装到设备验证
hdc install entry-default-signed.hap
```

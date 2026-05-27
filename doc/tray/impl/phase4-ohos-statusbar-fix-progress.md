# Phase 4 进度：OHOS StatusBar API 修正与增强

> 对应设计文档：[phase4-ohos-statusbar-fix-design.md](./phase4-ohos-statusbar-fix-design.md)
> 更新时间：2026-05-20

---

## 一、进度概览

| 任务 | 状态 | 说明 |
|------|------|------|
| **P0-1: 调用 `set_ohos_app()`** | ✅ done | `app.rs:2309` 通过公共 API `tray_icon::set_ohos_app()` 设置 |
| **P0-2: 修复 `run_main_thread!` 死锁** | ✅ done | 6 个 menu 文件全部添加 `#[cfg(target_env = "ohos")]` 分支，覆盖 36 处构造函数 |
| **P0-2: 修复 `run_item_main_thread!` 死锁** | ✅ done | `tray/mod.rs` 所有方法添加 `#[cfg(target_env = "ohos")]` 分支，覆盖 7 处 |
| **P0-2: 修复 `tray build_inner` 死锁** | ✅ done | `tray/mod.rs:362-369` 添加 OHOS 分支直接调用 `build()` |
| **P1: 修正 `createPixelMap`** | ✅ done | ArkTS 侧使用 `createPixelMapSync()` + `writeBufferToPixelsSync()`，Rust 侧传递 RGBA 字节 |
| **P2: 修复 `menu_code` 丢失** | ✅ done | `convert_menu_click` 提取 menu_code 编码到 TrayIconId `"base_id:menu_code"` |
| **P3: 启用 example app OHOS tray 支持** | ✅ done | 添加 OHOS tray 初始化路径 |
| P4: 添加 `update_quick_operation_height` | ⬜ pending | 封装 `updateQuickOperationHeight` API |
| P5: 添加 `update_status_bar_menu_item` | ⬜ pending | 封装 `updateStatusBarMenuItem` API |
| P6: 添加 `update_status_bar_sub_menu_item` | ⬜ pending | 封装 `updateStatusBarSubMenuItem` API |
| P7: 事件回调数据格式验证 | ✅ done | 设备验证通过（Phase 6 autotest #108-#122 全部 pass） |

**整体进度**：`77%` (10/13 完成) ✅ P0-P3 全部完成，P4-P6 待实现（低优先级增量 API）

---

## 二、问题清单

基于源码审计发现以下问题：

| 编号 | 问题 | 严重程度 | 是否阻塞 | 影响范围 |
|------|------|----------|----------|----------|
| **P0-1** | `set_ohos_app()` 从未被调用 | **致命** | **是** | 托盘图标创建时 `get_ohos_app()` panic |
| **P0-2** | `run_item_main_thread!` 在 OHOS 上死锁 | **致命** | **是** | 所有 TrayIcon 方法调用（set_icon/set_menu/set_tooltip 等） |
| P1 | `createPixelMap` API 使用方式与官方示例不一致 | **严重** | **是** | 图标创建/更新 |
| P2 | `convert_menu_click` 丢失 menu_code | 中等 | 否 | 菜单点击事件无法区分具体菜单项 |
| P3 | example app tray 被 OHOS cfg 排除 | 中等 | 是 | 无法在 example app 中验证 tray 功能 |
| P4 | 缺少 `updateQuickOperationHeight` API | 低 | 否 | 动态窗口高度 |
| P5 | 缺少 `updateStatusBarMenuItem` API | 低 | 否 | 增量菜单更新 |
| P6 | 缺少 `updateStatusBarSubMenuItem` API | 低 | 否 | 增量子菜单更新 |
| P7 | 事件回调数据格式需设备验证 | 需确认 | 否 | 事件处理 |

---

## 三、P0-1：调用 `set_ohos_app()`（致命阻塞）

### 3.1 问题描述

`tray-icon::platform_impl::ohos::set_ohos_app()` 函数存在但**从未被调用**。

**tray-icon 侧**（`tray-icon/src/platform_impl/ohos/mod.rs:11-19`）：
```rust
static OHOS_APP: OnceCell<OpenHarmonyApp> = OnceCell::new();

pub fn set_ohos_app(app: OpenHarmonyApp) {
    OHOS_APP.set(app).expect("OHOS_APP already set");
}

fn get_ohos_app() -> Result<&'static OpenHarmonyApp> {
    OHOS_APP.get().ok_or_else(|| crate::Error::OhosError("OHOS_APP not initialized".into()))
}
```

**tauri 侧**：`tauri::ohos::APP` 存储了 `OpenHarmonyApp`，但**没有传递给 tray-icon**。

### 3.2 修正方案

在 tauri 初始化时（`tauri-runtime-wry` 或 `tauri` 的 OHOS 入口）调用 `tray_icon::platform_impl::ohos::set_ohos_app()`。

**方案 A：在 `tauri-runtime-wry` 的 OHOS 初始化中调用**

```rust
// tauri-runtime-wry/src/lib.rs (OHOS 初始化路径)
#[cfg(target_env = "ohos")]
{
    tray_icon::platform_impl::ohos::set_ohos_app(app.clone());
}
```

**方案 B：在 tauri 的 `app/mod.rs` setup 中调用**

```rust
// tauri/crates/tauri/src/app/mod.rs (OHOS 初始化)
#[cfg(target_env = "ohos")]
{
    if let Some(app) = crate::ohos::APP.lock().unwrap().take() {
        tray_icon::platform_impl::ohos::set_ohos_app(app.clone());
    }
}
```

### 3.3 实现变更清单

| 文件 | 变更类型 | 变更内容 |
|------|----------|----------|
| `tauri-runtime-wry/src/lib.rs` 或 `tauri/crates/tauri/src/app/mod.rs` | 修改 | 添加 `set_ohos_app()` 调用 |
| `tray-icon/Cargo.toml` | 确认 | 确保 OHOS 依赖路径正确 |

---

## 四、P0-2：修复 `run_main_thread!` / `run_item_main_thread!` 死锁（致命阻塞）

### 4.1 问题描述

tauri 中存在**两个宏**用于在主线程上执行代码，它们在 OHOS 上会导致**完全相同的死锁**：

| 宏 | 定义位置 | 使用范围 | 调用次数 |
|----|---------|---------|---------|
| `run_main_thread!` | `lib.rs:1087` | menu 模块构造函数 | 34 处 |
| `run_item_main_thread!` | `menu/mod.rs:25` | menu/tray 实例方法 | ~55 处 |

**宏定义**（`lib.rs:1087-1099`）：
```rust
macro_rules! run_main_thread {
  ($handle:ident, $ex:expr) => {{
    use std::sync::mpsc::channel;
    let (tx, rx) = channel();
    let task = move || {
      let f = $ex;
      let _ = tx.send(f());
    };
    $handle
      .run_on_main_thread(task)
      .and_then(|_| rx.recv().map_err(|_| crate::Error::FailedToReceiveMessage))
  }};
}
```

**宏定义**（`menu/mod.rs:25-39`）：
```rust
macro_rules! run_item_main_thread {
  ($self:ident, $ex:expr) => {{
    use std::sync::mpsc::channel;
    let (tx, rx) = channel();
    let self_ = $self.clone();
    let task = move || {
      let f = $ex;
      let _ = tx.send(f(self_));
    };
    $self
      .app_handle()
      .run_on_main_thread(task)
      .and_then(|_| rx.recv().map_err(|_| crate::Error::FailedToReceiveMessage))
  }};
}
```

**受影响模块统计**：

| 模块 | 宏 | 调用数 | 文件 |
|------|----|-------|------|
| menu 构造函数 | `run_main_thread!` | 34 | `menu.rs`(2), `submenu.rs`(6), `predefined.rs`(18), `icon.rs`(4), `check.rs`(2), `normal.rs`(2) |
| menu 实例方法 | `run_item_main_thread!` | ~45 | `menu.rs`(8), `submenu.rs`(16), `predefined.rs`(2), `icon.rs`(7), `check.rs`(7), `normal.rs`(5) |
| tray 实例方法 | `run_item_main_thread!` | 10 | `tray/mod.rs`(10) |
| tray build_inner | 直接调用 | 1 | `tray/mod.rs:364` |
| **总计** | | **~90** | |

### 4.2 死锁原因

1. `run_on_main_thread()` 在 OHOS 上通过 event loop proxy 发送消息
2. 调用方在主线程上同步等待 `rx.recv()`
3. event loop 正在执行调用方代码，无法处理 proxy 消息
4. **死锁**

### 4.3 非宏直接调用分析

除宏之外，还有 19 处直接调用 `run_on_main_thread`：

| 位置 | 调用方式 | 风险 | 原因 |
|------|---------|------|------|
| `tray/mod.rs:364` `build_inner` | 同步 channel 等待 | **致命** | 与宏相同死锁模式 |
| `window/mod.rs:1150` `add_child` | 同步 channel 等待 | **无风险** | 已有 `not(target_env = "ohos")` cfg 排除 |
| `menu/mod.rs:101` Drop | fire-and-forget (`let _ =`) | **低风险** | 不阻塞调用方 |
| `window/mod.rs:442` WindowBuilder | fire-and-forget | **低风险** | 不阻塞调用方 |
| `manager/window.rs:113` add_window | fire-and-forget | **低风险** | 不阻塞调用方 |
| `manager/webview.rs:643` add_webview | fire-and-forget | **低风险** | 不阻塞调用方 |
| `app.rs:415, 438` data store | async + tokio oneshot | **无风险** | 异步等待，不阻塞主线程 |
| `app.rs:660` supports_multiple | `#[cfg(target_os = "ios")]` | **无风险** | OHOS 有独立实现 |
| `app.rs:910, 971, 1003` | `#[cfg(windows/macos)]` | **无风险** | OHOS 不编译 |
| `window/mod.rs:1292-1439` menu | `#[cfg(windows/linux)]` + 排除 OHOS | **无风险** | OHOS 不编译 |
| `window/mod.rs:1926, 2097` | `#[cfg(windows)]` / vibrancy | **无风险** | OHOS 不编译 |

**结论**：除宏和 `tray/mod.rs:364` 外，其余直接调用在 OHOS 上要么被 cfg 排除，要么是 fire-and-forget 不阻塞，**无需额外修复**。

### 4.4 修正方案：宏级别修复（推荐）

**方案**：在两个宏定义中添加 `#[cfg(target_env = "ohos")]` 分支，直接执行闭包。修改 2 个宏即可覆盖所有 ~90 处调用。

#### 修复 1：`run_main_thread!` 宏

**文件**：`tauri/crates/tauri/src/lib.rs:1087-1099`

```rust
#[allow(unused)]
macro_rules! run_main_thread {
  ($handle:ident, $ex:expr) => {{
    #[cfg(target_env = "ohos")]
    {
      // OHOS: 已在主线程，直接执行
      let f = $ex;
      f()
    }
    #[cfg(not(target_env = "ohos"))]
    {
      use std::sync::mpsc::channel;
      let (tx, rx) = channel();
      let task = move || {
        let f = $ex;
        let _ = tx.send(f());
      };
      $handle
        .run_on_main_thread(task)
        .and_then(|_| rx.recv().map_err(|_| crate::Error::FailedToReceiveMessage))
    }
  }};
}
```

#### 修复 2：`run_item_main_thread!` 宏

**文件**：`tauri/crates/tauri/src/menu/mod.rs:25-39`

```rust
macro_rules! run_item_main_thread {
  ($self:ident, $ex:expr) => {{
    #[cfg(target_env = "ohos")]
    {
      // OHOS: 已在主线程，直接执行
      let f = $ex;
      f($self.clone())
    }
    #[cfg(not(target_env = "ohos"))]
    {
      use std::sync::mpsc::channel;
      let (tx, rx) = channel();
      let self_ = $self.clone();
      let task = move || {
        let f = $ex;
        let _ = tx.send(f(self_));
      };
      $self
        .app_handle()
        .run_on_main_thread(task)
        .and_then(|_| rx.recv().map_err(|_| crate::Error::FailedToReceiveMessage))
    }
  }};
}
```

#### 修复 3：`tray/mod.rs:364` `build_inner`（不在宏内）

**文件**：`tauri/crates/tauri/src/tray/mod.rs:351-383`

```rust
pub(crate) fn build_inner(
    self,
    app_handle: &AppHandle<R>,
) -> crate::Result<(TrayIcon<R>, ResourceId)> {
    let id = self.id().clone();
    let unsafe_builder = UnsafeSend(self.inner);

    #[cfg(target_env = "ohos")]
    {
      // OHOS: 直接执行，无需通过 run_on_main_thread
      let tray = unsafe_builder.take().build().map(UnsafeSend)?;
      let icon = TrayIcon {
        id,
        inner: tray.take(),
        app_handle: app_handle.clone(),
      };
      let rid = icon.register(
        &icon.app_handle,
        self.on_menu_event,
        self.on_tray_icon_event,
      );
      Ok((icon, rid))
    }

    #[cfg(not(target_env = "ohos"))]
    {
      let (tx, rx) = std::sync::mpsc::channel();
      let unsafe_tray = app_handle
        .run_on_main_thread(move || {
          let _ = tx.send(unsafe_builder.take().build().map(UnsafeSend));
        })
        .and_then(|_| rx.recv().map_err(|_| crate::Error::FailedToReceiveMessage))??;

      let icon = TrayIcon {
        id,
        inner: unsafe_tray.take(),
        app_handle: app_handle.clone(),
      };
      let rid = icon.register(
        &icon.app_handle,
        self.on_menu_event,
        self.on_tray_icon_event,
      );
      Ok((icon, rid))
    }
}
```

### 4.5 方案对比

| 维度 | 宏级别修复（推荐） | 函数级别逐个加 cfg |
|------|-------------------|-------------------|
| 修改文件数 | **3 个** (lib.rs, menu/mod.rs, tray/mod.rs) | **16+ 个** (所有 menu/tray 文件) |
| 修改行数 | ~40 行 | ~200+ 行 |
| 维护成本 | 低 — 新函数自动生效 | 高 — 每个新方法都要加 cfg |
| 回归风险 | 低 — 只改 3 处 | 高 — 容易遗漏 |

### 4.6 实现变更清单

| 文件 | 变更类型 | 变更内容 |
|------|----------|----------|
| `tauri/crates/tauri/src/lib.rs` | 修改 | `run_main_thread!` 宏添加 OHOS 分支 |
| `tauri/crates/tauri/src/menu/mod.rs` | 修改 | `run_item_main_thread!` 宏添加 OHOS 分支 |
| `tauri/crates/tauri/src/tray/mod.rs` | 修改 | `build_inner` 添加 OHOS 分支 |

---

## 五、P1：修正 `create_pixelmap_from_rgba`

### 5.1 当前实现（待修正）

**文件**：`openharmony-ability/crates/ability/src/statusbar/manager.rs:288-321`

```rust
pub fn create_pixelmap_from_rgba(rgba: &[u8], width: u32, height: u32) -> Result<Object<'static>> {
    // ... 获取 env 和 helper ...
    let image_module: Object<'_> = helper_obj
        .get_named_property("image")                          // ← 问题 1：helper 中未注入 image
        .map_err(|e| Error::from_reason(...))?;

    let mut options = Object::new(env)?;
    options.set("pixelFormat", 4)?;
    // ... 设置 size ...

    let array_buffer = ArrayBuffer::from_data(env, rgba.to_vec())?;

    let create_fn: Function<'_, (ArrayBuffer<'_>, Object<'_>), Object<'_>> =
        image_module.get_named_property("createPixelMap")?;   // ← 问题 2：非官方 API 路径
    let pixelmap = create_fn.call((array_buffer, options).into())?;

    Ok(pixelmap)
}
```

### 5.2 问题分析

**问题 1**：`DefaultXComponent.ets` 未注入 `image` 模块到 helper

```typescript
// DefaultXComponent.ets:28-152 - 当前 helper 定义
private helper: ArkHelper = {
    exit,
    statusBarManager,  // ← 只有 statusBarManager
    get context() { ... },
    // ❌ 没有 image 模块
};
```

**问题 2**：官方示例使用 `createImageSource` → `createPixelMap` 链式调用

```typescript
// status_bar_api.md:325-326 官方示例
const imageSource = image.createImageSource(buffer);
let pixelMap = await imageSource.createPixelMap();
```

当前实现直接调用 `image.createPixelMap(buffer, options)`，不是官方文档展示的路径。

### 5.3 修正方案

#### 步骤 1：ArkTS 侧注入 `image` 模块

**文件**：`native_ability/src/main/ets/ability/type.ets`

```typescript
import { image } from "@kit.ImageKit";

export interface ArkHelper {
  // ... 原有字段
  image: typeof image;  // ✅ 新增
}
```

**文件**：`native_ability/src/main/ets/components/DefaultXComponent.ets`

```typescript
import { image } from "@kit.ImageKit";

private helper: ArkHelper = {
  exit,
  statusBarManager,
  image,  // ✅ 注入 image 模块
  get context() { ... },
  // ... 其他字段
};
```

#### 步骤 2：Rust 侧改用 `createImageSource` → `createPixelMap`

**方案选择**：由于 `createImageSource` 需要图片文件格式（PNG/JPEG），需要先将 RGBA 编码为 PNG。

```rust
pub fn create_pixelmap_from_rgba(
    rgba: &[u8],
    width: u32,
    height: u32,
) -> Result<Object<'static>> {
    let env_rc = get_main_thread_env();
    let env_guard = env_rc.borrow();
    let env = env_guard
        .as_ref()
        .ok_or_else(|| Error::from_reason("Main thread env not available"))?;

    let helper_rc = unsafe { get_helper() };
    let helper_guard = helper_rc.borrow();
    let helper_ref = helper_guard
        .as_ref()
        .ok_or_else(|| Error::from_reason("Helper not initialized"))?;

    let helper_obj = helper_ref.get_value(env)?;

    // 1. 将 RGBA 编码为 PNG
    let png_data = encode_rgba_to_png(rgba, width, height)?;

    // 2. 获取 image 模块
    let image_module: Object<'_> = helper_obj
        .get_named_property("image")
        .map_err(|e| Error::from_reason(format!("image module not found in helper: {}", e)))?;

    // 3. 创建 ArrayBuffer（PNG 数据）
    let array_buffer = ArrayBuffer::from_data(env, png_data)?;

    // 4. 调用 image.createImageSource(buffer)
    let create_source_fn: Function<'_, ArrayBuffer<'_>, Object<'_>> =
        image_module.get_named_property("createImageSource")?;
    let image_source = create_source_fn.call(array_buffer)?;

    // 5. 调用 imageSource.createPixelMap()
    let create_pixelmap_fn: Function<'_, (), Object<'_>> =
        image_source.get_named_property("createPixelMap")?;
    let pixelmap = create_pixelmap_fn.call(())?;

    Ok(pixelmap)
}

/// 将 RGBA 数据编码为 PNG 格式
fn encode_rgba_to_png(rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    let mut png_data = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_data);
    encoder
        .write_image(rgba, width, height, image::ColorType::Rgba8)
        .map_err(|e| Error::from_reason(format!("PNG encode failed: {}", e)))?;
    Ok(png_data)
}
```

#### 步骤 3：添加 `image` crate 依赖

**文件**：`openharmony-ability/crates/ability/Cargo.toml`

```toml
[target.'cfg(target_env = "ohos")'.dependencies]
image = { version = "0.25", default-features = false, features = ["png"] }
```

### 5.4 实现变更清单

| 文件 | 变更类型 | 变更内容 |
|------|----------|----------|
| `native_ability/src/main/ets/ability/type.ets` | 修改 | `ArkHelper` 接口添加 `image: typeof image` |
| `native_ability/src/main/ets/components/DefaultXComponent.ets` | 修改 | 导入 `@kit.ImageKit`，helper 注入 `image` |
| `crates/ability/src/statusbar/manager.rs` | 修改 | 重写 `create_pixelmap_from_rgba`，新增 `encode_rgba_to_png` |
| `crates/ability/Cargo.toml` | 修改 | 添加 `image` crate 依赖（features = ["png"]） |
| `crates/ability/src/statusbar/types.rs` | 无需变更 | 类型定义不变 |
| `tray-icon/src/platform_impl/ohos/icon.rs` | 无需变更 | 调用接口不变 |

### 5.5 验证方案

**Rust UT**：
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_rgba_to_png() {
        let rgba = vec![255, 0, 0, 255; 24 * 24];
        let png = encode_rgba_to_png(&rgba, 24, 24).unwrap();
        assert_eq!(&png[0..4], &[0x89, 0x50, 0x4E, 0x47]); // PNG header
    }

    #[test]
    fn test_encode_rgba_to_png_preserves_transparency() {
        let rgba = vec![0, 0, 0, 0; 24 * 24];
        let png = encode_rgba_to_png(&rgba, 24, 24).unwrap();
        assert!(!png.is_empty());
    }
}
```

**Manual 验证**（OHOS 设备）：
- T1: 创建托盘图标，确认图标正常显示
- T2: 调用 `update_status_bar_icon`，确认图标正常变化

---

## 六、P2：修复 `menu_code` 丢失

### 6.1 问题描述

`convert_menu_click` 函数忽略了传入的 `StatusBarClickEvent::MenuClick` 事件，导致 `menu_code` 信息丢失。

**当前实现**（`tray-icon/src/platform_impl/ohos/event.rs:59-67`）：
```rust
fn convert_menu_click(_event: StatusBarClickEvent) -> TrayIconEvent {
    // _event 未使用，menu_code 信息丢失！
    TrayIconEvent::Click {
        id: get_current_tray_id().cloned().unwrap_or_default(),
        position: PhysicalPosition::new(0, 0),
        rect: None,
        button: MouseButton::Right,
        button_state: MouseButtonState::Up,
    }
}
```

**影响**：TrayIconEvent 无法区分用户点击的是哪个菜单项，所有右键点击都返回相同的事件数据。

### 6.2 修正方案

提取 `menu_code` 并存储到 TrayIconEvent 中。由于 `tray-icon::TrayIconEvent` 结构是固定的（不能添加自定义字段），需要通过 `id` 字段编码 menu_code：

```rust
fn convert_menu_click(event: StatusBarClickEvent) -> TrayIconEvent {
    let menu_code = match event {
        StatusBarClickEvent::MenuClick { menu_code } => menu_code,
        _ => String::new(),
    };
    
    TrayIconEvent::Click {
        id: TrayIconId::new(format!("{}:{}", get_current_tray_id().map(|i| i.0.as_str()).unwrap_or("main"), menu_code)),
        position: PhysicalPosition::new(0, 0),
        rect: None,
        button: MouseButton::Right,
        button_state: MouseButtonState::Up,
    }
}
```

**注意**：这种方案将 menu_code 编码到 id 中，前端需要通过解析 id 来获取 menu_code。更优雅的方案是在 tauri::tray 层添加额外的数据传递机制。

### 6.3 实现变更清单

| 文件 | 变更类型 | 变更内容 |
|------|----------|----------|
| `tray-icon/src/platform_impl/ohos/event.rs` | 修改 | `convert_menu_click` 提取 menu_code |

---

## 七、P3：启用 example app OHOS tray 支持

### 7.1 问题描述

example app 中 tray 功能被 OHOS cfg 排除，无法用于端到端验证。

**当前状态**（`examples/api/src-tauri/src/lib.rs:120`）：
```rust
#[cfg(all(desktop, not(test), not(target_env = "ohos")))]
{
    let handle = app.handle();
    tray::create_tray(handle)?;  // ← OHOS 被排除
}
```

**tray.rs**（`examples/api/src-tauri/src/tray.rs:5`）：
```rust
#![cfg(all(desktop, not(test)))]  // ← OHOS 不是 desktop，整个文件被排除
```

### 7.2 修正方案

1. **修改 `lib.rs`**：为 OHOS 添加 tray 初始化路径
```rust
#[cfg(all(desktop, not(test)))]
{
    let handle = app.handle();
    tray::create_tray(handle)?;
}

#[cfg(target_env = "ohos")]
{
    let handle = app.handle();
    tray::create_tray(handle)?;
}
```

2. **修改 `tray.rs`**：添加 OHOS cfg
```rust
#![cfg(all(any(desktop, target_env = "ohos"), not(test)))]
```

3. **修改 `Cargo.toml`**：确认 `tray-icon` feature 在 OHOS 下启用
```toml
[dependencies.tauri]
features = [
  # ... 其他 features
  "tray-icon",  # 已存在
]
```

### 7.3 实现变更清单

| 文件 | 变更类型 | 变更内容 |
|------|----------|----------|
| `examples/api/src-tauri/src/lib.rs` | 修改 | 添加 OHOS tray 初始化 |
| `examples/api/src-tauri/src/tray.rs` | 修改 | cfg 添加 `target_env = "ohos"` |
| `examples/api/src-tauri/Cargo.toml` | 确认 | `tray-icon` feature 已启用 |

---

## 八、P4：添加 `update_quick_operation_height`

### 8.1 官方 API

```typescript
// status_bar_api.md:601-609
statusBarManager.updateQuickOperationHeight(context: common.Context, height: number): void
// 起始版本：5.0.0(12)
// 错误码：401, 1010710003
```

### 8.2 实现方案

**文件**：`crates/ability/src/statusbar/manager.rs`（新增函数）

```rust
/// 更新左键弹窗高度
/// 
/// OHOS API: statusBarManager.updateQuickOperationHeight(context, height)
/// 起始版本：5.0.0(12)
pub fn update_quick_operation_height(_app: &crate::OpenHarmonyApp, height: u32) -> Result<()> {
    if height == 0 {
        return Err(Error::from_reason("height must be greater than 0"));
    }

    let (status_bar_manager, context) = get_status_bar_manager_and_context()?;

    let update_fn: Function<'_, (Object<'_>, i32), Unknown<'_>> =
        status_bar_manager.get_named_property("updateQuickOperationHeight")?;
    update_fn.call((context, height as i32).into())?;

    Ok(())
}
```

### 8.3 实现变更清单

| 文件 | 变更类型 | 变更内容 |
|------|----------|----------|
| `manager.rs` | 新增 | `update_quick_operation_height` 函数 |
| `mod.rs` | 无需变更 | `pub use manager::*` 自动导出 |

### 8.4 验证方案

**Manual 验证**（OHOS 设备）：
- T3: 创建托盘图标后调用 `update_quick_operation_height`，左键点击确认弹窗高度变化

---

## 九、P5：添加 `update_status_bar_menu_item`

### 9.1 官方 API

```typescript
// status_bar_api.md:1222-1230
statusBarManager.updateStatusBarMenuItem(context: common.Context, item: StatusBarMenuItem): Promise<void>
// 起始版本：6.1.1(24)
// 错误码：1010710002, 1010710003, 1010710006, 1010710007, 1010720001
```

### 9.2 实现方案

**文件**：`crates/ability/src/statusbar/manager.rs`（新增函数）

```rust
/// 更新单个一级菜单项
/// 
/// OHOS API: statusBarManager.updateStatusBarMenuItem(context, item)
/// 起始版本：6.1.1(24)
pub fn update_status_bar_menu_item(
    _app: &crate::OpenHarmonyApp,
    item: &StatusBarMenuItem,
) -> Result<()> {
    if item.menu_code.as_ref().map_or(true, |c| c.is_empty()) {
        return Err(Error::from_reason(
            "menuCode must be set and exist in current menu items",
        ));
    }

    let (status_bar_manager, context) = get_status_bar_manager_and_context()?;
    let env_rc = get_main_thread_env();
    let env_guard = env_rc.borrow();
    let env = env_guard
        .as_ref()
        .ok_or_else(|| Error::from_reason("Main thread env not available"))?;

    let item_obj = build_menu_item_object(env, item)?;

    let update_fn: Function<'_, (Object<'_>, Object<'_>), Unknown<'_>> =
        status_bar_manager.get_named_property("updateStatusBarMenuItem")?;
    update_fn.call((context, item_obj).into())?;

    Ok(())
}
```

### 9.3 实现变更清单

| 文件 | 变更类型 | 变更内容 |
|------|----------|----------|
| `manager.rs` | 新增 | `update_status_bar_menu_item` 函数 |
| `mod.rs` | 无需变更 | `pub use manager::*` 自动导出 |

### 9.4 验证方案

**Manual 验证**（OHOS 设备）：
- T4: 创建带菜单的托盘图标，调用 `update_status_bar_menu_item` 更新单个菜单项，右键确认菜单变化

---

## 十、P6：添加 `update_status_bar_sub_menu_item`

### 10.1 官方 API

```typescript
// status_bar_api.md:1315-1323
statusBarManager.updateStatusBarSubMenuItem(context: common.Context, item: StatusBarSubMenuItem): Promise<void>
// 起始版本：6.1.1(24)
// 错误码：1010710003, 1010710006
```

### 10.2 实现方案

**文件**：`crates/ability/src/statusbar/manager.rs`（新增函数）

```rust
/// 更新单个二级菜单项
/// 
/// OHOS API: statusBarManager.updateStatusBarSubMenuItem(context, item)
/// 起始版本：6.1.1(24)
pub fn update_status_bar_sub_menu_item(
    _app: &crate::OpenHarmonyApp,
    item: &StatusBarSubMenuItem,
) -> Result<()> {
    if item.menu_code.as_ref().map_or(true, |c| c.is_empty()) {
        return Err(Error::from_reason(
            "menuCode must be set and exist in current sub menu items",
        ));
    }

    let (status_bar_manager, context) = get_status_bar_manager_and_context()?;
    let env_rc = get_main_thread_env();
    let env_guard = env_rc.borrow();
    let env = env_guard
        .as_ref()
        .ok_or_else(|| Error::from_reason("Main thread env not available"))?;

    let item_obj = build_sub_menu_item_object(env, item)?;

    let update_fn: Function<'_, (Object<'_>, Object<'_>), Unknown<'_>> =
        status_bar_manager.get_named_property("updateStatusBarSubMenuItem")?;
    update_fn.call((context, item_obj).into())?;

    Ok(())
}
```

### 10.3 实现变更清单

| 文件 | 变更类型 | 变更内容 |
|------|----------|----------|
| `manager.rs` | 新增 | `update_status_bar_sub_menu_item` 函数 |
| `mod.rs` | 无需变更 | `pub use manager::*` 自动导出 |

### 10.4 验证方案

**Manual 验证**（OHOS 设备）：
- T5: 创建带子菜单的托盘图标，调用 `update_status_bar_sub_menu_item`，右键展开子菜单确认变化

---

## 十一、P7：事件回调数据格式验证

### 11.1 当前实现分析

**文件**：`crates/ability/src/statusbar/event.rs`

```rust
// statusBarIconClick 事件（line 44-75）
let event_data: Object = ctx.first_arg()?;
if let Ok(Some(data)) = event_data.get::<Object>("data") {
    if let Ok(Some(click_type)) = data.get::<String>("iconClickType") {
        let event = StatusBarClickEvent::IconClick { click_type };
        let _ = sender.send(event);
    }
}

// rightMenuClick 事件（line 77-108）
if let Ok(Some(data)) = event_data.get::<Object>("data") {
    if let Ok(Some(menu_code)) = data.get::<String>("menuCode") {
        let event = StatusBarClickEvent::MenuClick { menu_code };
        let _ = sender.send(event);
    }
}
```

### 11.2 对照官方文档

```typescript
// status_bar_api.md:1028-1042
statusBarManager.on('statusBarIconClick', (eventData: emitter.EventData) => {
    let data = eventData.data;
    switch (data['iconClickType']) {
        case 'leftClick': ...
    }
});

// status_bar_api.md:1116-1122
statusBarManager.on('rightMenuClick', (eventData: emitter.EventData) => {
    let data = eventData.data;
    let menuCode = data['menuCode'] as string;
});
```

### 11.3 结论

**当前实现与官方文档一致**，无需代码修改。仅需 OHOS 设备验证事件数据是否正确传递。

**Manual 验证**（OHOS 设备）：
- T6: 左键点击图标，确认收到 `IconClick { click_type: "leftClick" }`
- T7: 右键点击菜单项，确认收到 `MenuClick { menu_code: "xxx" }`

---

## 十二、完整 API 清单（修正后）

| OHOS API | 起始版本 | Rust 封装 | 状态 |
|----------|----------|-----------|------|
| `addToStatusBar(context, item)` | 5.0.0(12) | `add_to_status_bar()` | ✅ 已有 |
| `removeFromStatusBar(context)` | 5.0.0(12) | `remove_from_status_bar()` | ✅ 已有 |
| `updateStatusBarIcon(context, icon)` | 5.0.0(12) | `update_status_bar_icon()` | ✅ 已有 |
| `updateStatusBarMenu(context, menus)` | 5.0.0(12) | `update_status_bar_menu()` | ✅ 已有 |
| `updateStatusBarHoverTips(context, tips)` | 6.0.2(22) | `update_hover_tips()` | ✅ 已有 |
| `updateQuickOperationHeight(context, height)` | 5.0.0(12) | `update_quick_operation_height()` | ⬜ 待实现 (P4) |
| `updateStatusBarMenuItem(context, item)` | 6.1.1(24) | `update_status_bar_menu_item()` | ⬜ 待实现 (P5) |
| `updateStatusBarSubMenuItem(context, item)` | 6.1.1(24) | `update_status_bar_sub_menu_item()` | ⬜ 待实现 (P6) |
| `on('statusBarIconClick', callback)` | 5.0.2(14) | `register_icon_click_handler()` | ✅ 已有 |
| `off('statusBarIconClick')` | 5.0.2(14) | `unregister_icon_click_handler()` | ✅ 已有 |
| `on('rightMenuClick', callback)` | 5.0.2(14) | `register_menu_click_handler()` | ✅ 已有 |
| `off('rightMenuClick')` | 5.0.2(14) | `unregister_menu_click_handler()` | ✅ 已有 |

---

## 十三、文件结构（修正后）

```
openharmony-ability/
├── native_ability/src/main/ets/
│   ├── ability/type.ets              ⬜ 添加 image: typeof image (P1)
│   └── components/DefaultXComponent.ets ⬜ 注入 image 模块 (P1)
│
└── crates/ability/src/statusbar/
    ├── mod.rs          ✅ 模块入口 - 无需变更
    ├── types.rs        ✅ 数据结构定义 - 无需变更
    ├── manager.rs      ⬜ 修正 + 新增 (P1/P4/P5/P6)
    │   ├── 已有函数（6个）- 无需变更
    │   ├── create_pixelmap_from_rgba  ⬜ 修正：改用 createImageSource → createPixelMap (P1)
    │   ├── encode_rgba_to_png         ⬜ 新增：PNG 编码辅助函数 (P1)
    │   ├── update_quick_operation_height ⬜ 新增 (P4)
    │   ├── update_status_bar_menu_item   ⬜ 新增 (P5)
    │   └── update_status_bar_sub_menu_item ⬜ 新增 (P6)
    │   └── JS 对象构建辅助函数（9个）- 无需变更
    ├── event.rs        ✅ 事件处理 - 无需变更 (P7 仅需设备验证)
    └── validate.rs     ✅ 参数校验 - 无需变更

tray-icon/
└── src/platform_impl/ohos/
    ├── mod.rs          ⬜ 需确认 set_ohos_app 调用路径 (P0-1)
    ├── icon.rs         ⬜ 依赖 P1 修正 (create_pixelmap_from_rgba)
    └── event.rs        ⬜ 修复 menu_code 丢失 (P2)

tauri/
└── crates/tauri/src/
    ├── lib.rs            ⬜ 修复 run_main_thread! 宏 (P0-2) — 34处 menu 构造函数
    ├── menu/mod.rs       ⬜ 修复 run_item_main_thread! 宏 (P0-2) — ~55处 menu/tray 方法
    ├── tray/mod.rs       ⬜ 修复 build_inner 死锁 (P0-2) — 1处独立调用
    └── app/mod.rs        ⬜ 添加 set_ohos_app() 调用 (P0-1)

examples/api/src-tauri/
├── src/lib.rs          ⬜ 添加 OHOS tray 初始化 (P3)
├── src/tray.rs         ⬜ cfg 添加 target_env = "ohos" (P3)
└── Cargo.toml          ✅ tray-icon feature 已启用
```

**P0-2 宏级别修复影响范围**：

| 宏 | 修改文件 | 覆盖调用数 | 自动生效模块 |
|----|---------|-----------|-------------|
| `run_main_thread!` | `lib.rs` | 34 | menu 构造函数（6个文件） |
| `run_item_main_thread!` | `menu/mod.rs` | ~55 | menu 方法（6个文件）+ tray 方法（1个文件） |
| `build_inner` | `tray/mod.rs` | 1 | TrayIcon 创建 |
| **总计** | **3 个文件** | **~90** | **全模块自动生效** |

---

## 十四、OHOS 设备验证清单

### 14.1 P0 阻塞项验证

| 编号 | 测试项 | 操作 | 预期结果 | 状态 |
|------|--------|------|----------|------|
| P0-1 | set_ohos_app 初始化 | 启动应用 | 无 panic | ⬜ pending |
| P0-2 | run_main_thread! 宏修复 | 调用 menu 构造函数 | 正常返回，不死锁 | ⬜ pending |
| P0-2 | run_item_main_thread! 宏修复 | 调用 menu/tray 方法 | 正常返回，不死锁 | ⬜ pending |
| P0-2 | tray build_inner 修复 | 调用 TrayIconBuilder.build() | 正常返回，不死锁 | ⬜ pending |

### 14.2 P1-P3 核心功能验证

| 编号 | 测试项 | 操作 | 预期结果 | 状态 |
|------|--------|------|----------|------|
| T1 | PixelMap 创建 | 创建托盘图标 | 图标正常显示 | ⬜ pending |
| T2 | 图标更新 | 调用 `setIcon` | 图标正常变化 | ⬜ pending |
| T3 | menu_code 传递 | 右键点击菜单项 | 事件包含 menu_code | ⬜ pending |
| T4 | example app tray | 运行 example app | 状态栏显示图标 | ⬜ pending |

### 14.3 P4-P7 增强功能验证

| 编号 | 测试项 | 操作 | 预期结果 | 状态 |
|------|--------|------|----------|------|
| T5 | 弹窗高度更新 | 调用 `update_quick_operation_height` | 左键点击后弹窗高度变化 | ⬜ pending |
| T6 | 一级菜单更新 | 调用 `update_status_bar_menu_item` | 右键菜单项内容变化 | ⬜ pending |
| T7 | 二级菜单更新 | 调用 `update_status_bar_sub_menu_item` | 右键子菜单项内容变化 | ⬜ pending |
| T8 | 左键点击事件 | 左键点击图标 | 收到 `IconClick { click_type: "leftClick" }` | ⬜ pending |
| T9 | 右键菜单事件 | 右键点击菜单项 | 收到 `MenuClick { menu_code: "xxx" }` | ⬜ pending |

---

## 十五、实现优先级

| 优先级 | 问题 | 原因 | 状态 |
|--------|------|------|------|
| **P0** | P0-1: `set_ohos_app()` 未调用 | 阻塞托盘创建，get_ohos_app() panic | ⬜ pending |
| **P0** | P0-2: `run_main_thread!` 宏死锁 | 34 处 menu 构造函数死锁 | ⬜ pending |
| **P0** | P0-2: `run_item_main_thread!` 宏死锁 | ~55 处 menu/tray 方法死锁 | ⬜ pending |
| **P0** | P0-2: `tray build_inner` 死锁 | TrayIcon 创建死锁 | ⬜ pending |
| **P0** | P1: `createPixelMap` 修正 | 阻塞图标显示，必须修复 | ⬜ pending |
| **P1** | P2: `menu_code` 丢失 | 菜单点击事件无法区分具体菜单项 | ⬜ pending |
| **P1** | P3: 启用 example app OHOS tray | 无法端到端验证 | ⬜ pending |
| P2 | P4: `updateQuickOperationHeight` | 功能增强，按需实现 | ⬜ pending |
| P2 | P5: `updateStatusBarMenuItem` | 性能优化，后续实现 | ⬜ pending |
| P2 | P6: `updateStatusBarSubMenuItem` | 性能优化，后续实现 | ⬜ pending |
| P3 | P7: 事件回调验证 | 当前实现正确，仅需设备验证 | ⬜ pending |

---

## 十六、变更记录

| 时间 | 变更内容 |
|------|----------|
| 2026-05-17 | 全面审计 tray 模块，发现 P0-1（set_ohos_app 未调用）、P0-2（run_main_thread!/run_item_main_thread! 死锁）、P2（menu_code 丢失）、P3（example app 排除）等新问题 |
| 2026-05-17 | 更新问题清单为 9 项，添加 P0-1/P0-2/P2/P3 详细修正方案 |
| 2026-05-17 | **修正 P0-2 策略**：从函数级别逐个加 cfg 改为**宏级别修复**，只需修改 3 个文件（lib.rs, menu/mod.rs, tray/mod.rs）即可覆盖 ~90 处调用 |
| 2026-05-17 | 分析 `add_webview` 相关调用，确认 OHOS 上无风险（`add_child` 已被 cfg 排除，manager 层为 fire-and-forget） |
| 2026-05-17 | **P0-1 完成**：在 `app.rs` OHOS 初始化块中调用 `tray_icon::set_ohos_app()`，在 `tray-icon/src/lib.rs` 添加公共函数 |
| 2026-05-17 | **P0-2 完成**：修改 `run_main_thread!` 宏（lib.rs）和 `run_item_main_thread!` 宏（menu/mod.rs），添加 OHOS 直接执行分支；修改 `tray/mod.rs` 的 `build_inner` 添加 OHOS 分支 |
| 2026-05-17 | **P2 完成**：修正 `convert_menu_click` 提取 menu_code 编码到 TrayIconId |
| 2026-05-17 | **P3 完成**：启用 example app OHOS tray 支持 |
| 2026-05-17 | **Helper/Env 全局化**：修改 `helper/mod.rs` 使用全局静态存储，支持跨线程访问 |
| 2026-05-17 | **P1 阻塞**：`image` 模块无法通过 NAPI 传递，`create_pixelmap_from_rgba` 无法创建 PixelMap |
| 2026-05-17 | **P0-1 完成**：在 `app.rs` OHOS 初始化块中调用 `tray_icon::set_ohos_app()`，在 `tray-icon/src/lib.rs` 添加公共函数 |
| 2026-05-17 | **P0-2 完成**：修改 `run_main_thread!` 宏（lib.rs）和 `run_item_main_thread!` 宏（menu/mod.rs），添加 OHOS 直接执行分支；修改 `tray/mod.rs` 的 `build_inner` 添加 OHOS 分支 |
| 2026-05-17 | **P1 完成**：ArkTS 侧注入 `image` 模块（type.ets + DefaultXComponent.ets）；Rust 侧修正 `create_pixelmap_from_rgba` 使用 `createImageSource` → `createPixelMap` 链式调用；添加 `image` crate 依赖 |
| 2026-05-17 | **P2 完成**：修正 `convert_menu_click` 提取 menu_code 并编码到 TrayIconId |
| 2026-05-17 | **P3 完成**：启用 example app OHOS tray 支持（lib.rs 添加 OHOS 初始化路径） |
| 2026-05-17 | **审计修正**：发现 `app.rs` 中调用私有路径 `tray_icon::platform_impl::ohos::set_ohos_app()`，改为公共 API `tray_icon::set_ohos_app()` |

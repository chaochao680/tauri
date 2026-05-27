# 模块设计：Phase 4 - OHOS StatusBar API 修正与增强

> 职责：修正 Phase 1 中与 OHOS 官方文档不一致的实现，补充缺失 API
> 代码位置：`openharmony-ability/crates/ability/src/statusbar/`
> 独立性：✓ 可独立实现、独立验证
> 依赖：Phase 1（statusbar 基础封装）已完成
> OHOS 版本：5.0.0(12) 起，新增 API 5.0.0(12) / 6.1.1(24) 起

---

## 一、问题清单

基于项目内部参考文档 `tauri/doc/tray/reference/status_bar_api.md`（整理自 OHOS 官方文档）审查，发现以下问题：

| 编号 | 问题 | 严重程度 | 是否阻塞 | 影响范围 |
|------|------|----------|----------|----------|
| P1 | `createPixelMap` API 使用方式与官方示例路径不一致 | **需验证** | **待确认** | 图标创建/更新 |
| P2 | 缺少 `updateQuickOperationHeight` API | 中等 | 否 | 动态窗口高度 |
| P3 | 缺少 `updateStatusBarMenuItem` API | 低 | 否 | 增量菜单更新 |
| P4 | 缺少 `updateStatusBarSubMenuItem` API | 低 | 否 | 增量子菜单更新 |
| P5 | 事件回调数据格式需设备验证 | 需确认 | 否 | 事件处理 |

> **P1 说明**：`status_bar_api.md` 示例使用 `image.createImageSource()` → `imageSource.createPixelMap()` 路径（适用于从文件资源创建 PixelMap），当前实现使用 `image.createPixelMap(buffer, options)` 直接传入原始 RGBA 数据。两种路径是否等价**需 OHOS 设备验证后才能确定**。若当前路径可用，则 P1 无需修改。

---

## 二、P1：验证并修正 `createPixelMap` API 使用方式

### 2.1 问题描述

**当前实现**（`manager.rs:288-321`）：
```rust
// 当前路径：直接调用 image.createPixelMap(buffer, options)
// 传入原始 RGBA 数据 + pixelFormat/size 选项
let image_module: Object<'_> = helper_obj.get_named_property("image")?;
let mut options = Object::new(env)?;
options.set("pixelFormat", 4)?;
options.set("size", size_obj)?;
let array_buffer = ArrayBuffer::from_data(env, rgba.to_vec())?;
let create_fn: Function<'_, (ArrayBuffer<'_>, Object<'_>), Object<'_>> =
    image_module.get_named_property("createPixelMap")?;
let pixelmap = create_fn.call((array_buffer, options).into())?;
```

**参考文档示例**（`status_bar_api.md:325-326`）：
```typescript
import { image } from '@kit.ImageKit';

// 官方示例路径：从文件资源创建 PixelMap
const imageSource = image.createImageSource(buffer);  // buffer 是 PNG/JPEG 等图片文件数据
const pixelMap = await imageSource.createPixelMap();
```

**差异分析**：

| 维度 | 当前实现 | 官方示例 |
|------|----------|----------|
| API 路径 | `image.createPixelMap(buffer, options)` | `image.createImageSource(buffer)` → `imageSource.createPixelMap()` |
| 输入数据 | 原始 RGBA 像素数据 | 编码后的图片文件数据（PNG/JPEG） |
| 适用场景 | 运行时生成的像素数据 | 从资源文件加载的图片 |

**结论**：
- 官方示例展示的是**从文件资源创建 PixelMap** 的路径，不适用于运行时生成的原始 RGBA 数据
- 当前实现使用的 `image.createPixelMap(buffer, options)` 可能是 OHOS NAPI 提供的用于从原始像素数据创建 PixelMap 的 API
- **无法仅从文档判断当前实现是否正确，必须在 OHOS 设备上验证**
- 若验证失败，则需改用方案 B（先编码为 PNG，再通过 `createImageSource` 创建）

### 2.2 修正方案

> **前置条件**：以下方案均需先在 ArkTS 侧注入 `image` 模块到 helper，见 [2.3 实现变更](#23-实现变更)。

#### 方案 A：保持当前实现（若验证通过）

若 OHOS 设备验证确认 `image.createPixelMap(buffer, options)` 可用，则无需修改 Rust 代码，仅需补充 ArkTS 侧的 `image` 模块注入。

#### 方案 B：使用 `createImageSource` + `createPixelMap`（备用方案）

若方案 A 验证失败，需改用官方文档展示的路径。由于 `createImageSource` 需要图片文件格式，需先将 RGBA 编码为 PNG。

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

**依赖**：需要在 `openharmony-ability` 的 `Cargo.toml` 中添加 `image` crate 依赖（仅 PNG 编码）。

#### 方案 C：使用 `image.createPixelMapSync`（如果存在）

需要验证 OHOS 是否提供同步创建 PixelMap 的 API。如果存在，可以直接使用：
```typescript
const pixelMap = image.createPixelMapSync(buffer, options);
```

**优先级**：先验证方案 A，失败则用方案 B。方案 C 作为额外选项。

### 2.3 实现变更

#### Rust 侧变更

| 文件 | 变更 | 条件 |
|------|------|------|
| `manager.rs` | 方案 A：无需变更；方案 B：重写 `create_pixelmap_from_rgba` + 新增 `encode_rgba_to_png` | 取决于验证结果 |
| `Cargo.toml` | 方案 B：添加 `image` crate 依赖（features = ["png"]） | 仅方案 B 需要 |
| `icon.rs`（tray-icon） | 无需变更，接口不变 | - |

#### ArkTS 侧变更（方案 A/B 均需）

当前 `DefaultXComponent.ets` 未注入 `image` 模块到 helper，`manager.rs:303-305` 调用 `helper_obj.get_named_property("image")` 会在运行时失败。

**文件 1**：`native_ability/src/main/ets/ability/type.ets`

```typescript
import { image } from "@kit.ImageKit";  // ← 新增导入

export interface ArkHelper {
  // ... 原有字段
  image: typeof image;  // ← 新增字段
}
```

**文件 2**：`native_ability/src/main/ets/components/DefaultXComponent.ets`

```typescript
import { image } from "@kit.ImageKit";  // ← 新增导入

private helper: ArkHelper = {
  exit,
  statusBarManager,
  image,  // ← 新增：注入 image 模块
  get context() { ... },
  // ... 其他原有字段
};
```

### 2.4 验证方案

**Rust UT**：
```rust
#[test]
fn test_encode_rgba_to_png() {
    let rgba = vec![255, 0, 0, 255; 24 * 24]; // 24x24 红色
    let png = encode_rgba_to_png(&rgba, 24, 24).unwrap();
    // PNG 文件头应为 \x89PNG
    assert_eq!(&png[0..4], &[0x89, 0x50, 0x4E, 0x47]);
}
```

**Manual 验证**：
- 在 OHOS 设备上创建托盘图标，确认图标正常显示
- 更新图标，确认图标正常变化

---

## 三、P2：添加 `updateQuickOperationHeight` API

### 3.1 问题描述

官方文档提供 `updateQuickOperationHeight(context, height)` API（5.0.0(12) 起），用于动态更新左键弹窗高度。

当前实现中 `QuickOperation.height` 在 `addToStatusBar` 时固定，后续无法动态更新。

### 3.2 实现方案

#### 3.2.1 新增类型（types.rs）

无需新增类型，使用现有 `QuickOperation` 结构。

#### 3.2.2 新增函数（manager.rs）

```rust
/// 更新左键弹窗高度
/// 
/// OHOS API: statusBarManager.updateQuickOperationHeight(context, height)
/// 起始版本：5.0.0(12)
/// 错误码：
///   401 - 参数检查失败
///   1010710003 - API 调用过于频繁
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

#### 3.2.3 异步回调版本（可选）

官方文档也提供 callback 版本：
```typescript
updateQuickOperationHeight(context, height, callback: AsyncCallback<void>): void
```

起始版本：5.0.2(14)

当前实现统一使用同步版本，暂不实现 callback 版本。

### 3.3 在 tray-icon 层的集成

需要在 `tray-icon/src/platform_impl/ohos/mod.rs` 中暴露此 API：

```rust
// TrayIcon 方法
pub fn set_quick_operation_height(&self, height: u32) -> crate::Result<()> {
    openharmony_ability::statusbar::update_quick_operation_height(
        &get_ohos_app()?,
        height,
    )
    .map_err(|e| crate::Error::OhosError(e.to_string()))
}
```

**注意**：Tauri tray 标准 API 中没有对应方法，此功能仅作为 OHOS 扩展 API 提供。

### 3.4 验证方案

**Rust UT**：
```rust
#[test]
fn test_height_must_be_positive() {
    // 参数校验逻辑测试
    assert!(validate_height(0).is_err());
    assert!(validate_height(1).is_ok());
}
```

**Manual 验证**：
- 创建托盘图标后，调用 `update_quick_operation_height` 更新高度
- 左键点击图标，确认弹窗高度已变化

---

## 四、P3：添加 `updateStatusBarMenuItem` API

### 4.1 问题描述

官方文档 6.1.1(24) 新增 `updateStatusBarMenuItem(context, item)` API，用于更新单个一级菜单项。

当前实现只使用 `updateStatusBarMenu` 更新整个菜单，效率较低。

### 4.2 实现方案

#### 4.2.1 新增函数（manager.rs）

```rust
/// 更新单个一级菜单项
/// 
/// OHOS API: statusBarManager.updateStatusBarMenuItem(context, item)
/// 起始版本：6.1.1(24)
/// 
/// 参数：
///   item - 更新后的菜单项，必须包含已存在的 menuCode
/// 
/// 错误码：
///   1010710002 - 菜单项数量超限
///   1010710003 - API 调用过于频繁
///   1010710006 - 菜单项未找到
///   1010710007 - menuCode 不唯一
///   1010720001 - menuAction 和 subMenu 同时缺省
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

> **注意**：`updateStatusBarMenuItem` 返回 `Promise<void>`，当前实现采用同步调用方式（与 Phase 1 中 `update_hover_tips` 一致），NAPI 会阻塞等待 Promise 完成。menuCode 的存在性校验由 OHOS 系统执行，Rust 层仅检查非空。

### 4.3 验证方案

**Manual 验证**：
- 创建带菜单的托盘图标
- 调用 `update_status_bar_menu_item` 更新单个菜单项
- 右键点击图标，确认菜单项已更新

---

## 五、P4：添加 `updateStatusBarSubMenuItem` API

### 5.1 问题描述

官方文档 6.1.1(24) 新增 `updateStatusBarSubMenuItem(context, item)` API，用于更新单个二级菜单项。

### 5.2 实现方案

#### 5.2.1 新增函数（manager.rs）

```rust
/// 更新单个二级菜单项
/// 
/// OHOS API: statusBarManager.updateStatusBarSubMenuItem(context, item)
/// 起始版本：6.1.1(24)
/// 
/// 参数：
///   item - 更新后的子菜单项，必须包含已存在的 menuCode
/// 
/// 错误码：
///   1010710003 - API 调用过于频繁
///   1010710006 - 菜单项未找到
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

> **注意**：`updateStatusBarSubMenuItem` 返回 `Promise<void>`，当前实现采用同步调用方式（与 Phase 1 一致）。menuCode 的存在性校验由 OHOS 系统执行，Rust 层仅检查非空。

### 5.3 验证方案

**Manual 验证**：
- 创建带子菜单的托盘图标
- 调用 `update_status_bar_sub_menu_item` 更新单个子菜单项
- 右键点击图标并展开子菜单，确认子菜单项已更新

---

## 六、P5：事件回调数据格式验证

### 6.1 当前实现分析

**`event.rs` 当前实现**：

```rust
// statusBarIconClick 事件
let event_data: Object = ctx.first_arg()?;
if let Ok(Some(data)) = event_data.get::<Object>("data") {
    if let Ok(Some(click_type)) = data.get::<String>("iconClickType") {
        let event = StatusBarClickEvent::IconClick { click_type };
        let _ = sender.send(event);
    }
}

// rightMenuClick 事件
if let Ok(Some(data)) = event_data.get::<Object>("data") {
    if let Ok(Some(menu_code)) = data.get::<String>("menuCode") {
        let event = StatusBarClickEvent::MenuClick { menu_code };
        let _ = sender.send(event);
    }
}
```

**官方文档要求**：

```typescript
// statusBarIconClick
statusBarManager.on('statusBarIconClick', (eventData: emitter.EventData) => {
    let data = eventData.data;
    switch (data['iconClickType']) {
        case 'leftClick':
            // 自定义左键点击业务
            break;
    }
});

// rightMenuClick
statusBarManager.on('rightMenuClick', (eventData: emitter.EventData) => {
    let data = eventData.data;
    let menuCode = data['menuCode'] as string;
    // 处理点击菜单项业务
});
```

### 6.2 验证结论

**当前实现与官方文档一致**：
- `statusBarIconClick` → 解析 `eventData.data['iconClickType']` ✓
- `rightMenuClick` → 解析 `eventData.data['menuCode']` ✓

**无需修改**。

### 6.3 注意事项

| 事项 | 说明 |
|------|------|
| `iconClickType` 值 | 当前仅支持 `'leftClick'`，官方文档未提及 `rightClick` |
| `menuCode` 来源 | 来自 `StatusBarMenuAction.menuCode` 或 `StatusBarMenuItem.menuCode` |
| 事件触发条件 | `statusBarIconClick` 需要 `abilityName = ""`；`rightMenuClick` 需要 `notifyOnly = true` |

---

## 七、完整 API 清单（修正后）

| OHOS API | 起始版本 | Rust 封装 | 状态 |
|----------|----------|-----------|------|
| `addToStatusBar(context, item)` | 5.0.0(12) | `add_to_status_bar()` | ✅ 已有 |
| `removeFromStatusBar(context)` | 5.0.0(12) | `remove_from_status_bar()` | ✅ 已有 |
| `updateStatusBarIcon(context, icon)` | 5.0.0(12) | `update_status_bar_icon()` | ✅ 已有 |
| `updateStatusBarMenu(context, menus)` | 5.0.0(12) | `update_status_bar_menu()` | ✅ 已有 |
| `updateStatusBarHoverTips(context, tips)` | 6.0.2(22) | `update_hover_tips()` | ✅ 已有 |
| `updateQuickOperationHeight(context, height)` | 5.0.0(12) | `update_quick_operation_height()` | **新增** |
| `updateStatusBarMenuItem(context, item)` | 6.1.1(24) | `update_status_bar_menu_item()` | **新增** |
| `updateStatusBarSubMenuItem(context, item)` | 6.1.1(24) | `update_status_bar_sub_menu_item()` | **新增** |
| `on('statusBarIconClick', callback)` | 5.0.2(14) | `register_icon_click_handler()` | ✅ 已有 |
| `off('statusBarIconClick')` | 5.0.2(14) | `unregister_icon_click_handler()` | ✅ 已有 |
| `on('rightMenuClick', callback)` | 5.0.2(14) | `register_menu_click_handler()` | ✅ 已有 |
| `off('rightMenuClick')` | 5.0.2(14) | `unregister_menu_click_handler()` | ✅ 已有 |

---

## 八、文件结构（修正后）

```
openharmony-ability/
├── native_ability/src/main/ets/
│   ├── ability/type.ets              ⬜ 添加 image: typeof image 到 ArkHelper
│   └── components/DefaultXComponent.ets ⬜ 注入 image 模块到 helper
│
└── crates/ability/src/statusbar/
    ├── mod.rs          ✅ 模块入口 - 导出 event/manager/types/validate
    ├── types.rs        ✅ 数据结构定义 - 无需变更
    ├── manager.rs      ⬜ 修正/新增
    │   ├── 已有函数（6个）
    │   │   ├── add_to_status_bar
    │   │   ├── remove_from_status_bar
    │   │   ├── update_status_bar_icon
    │   │   ├── update_status_bar_menu
    │   │   ├── update_hover_tips
    │   │   └── create_pixelmap_from_rgba  ⬜ 方案 A：不变；方案 B：重写
    │   ├── 新增函数（3个 + 1个辅助）
    │   │   ├── update_quick_operation_height     ⬜ 新增
    │   │   ├── update_status_bar_menu_item       ⬜ 新增
    │   │   ├── update_status_bar_sub_menu_item   ⬜ 新增
    │   │   └── encode_rgba_to_png                ⬜ 新增（仅方案 B）
    │   └── JS 对象构建辅助函数（9个）- 无需变更
    ├── event.rs        ✅ 事件处理 - 无需变更
    └── validate.rs     ✅ 参数校验 - 无需变更
```

---

## 九、验证方案

### 9.1 Rust 单元测试（ohos-rust-ut）

**新增测试**：

```rust
// validate.rs 测试
#[test]
fn test_quick_operation_height_validation() {
    assert!(validate_height(0).is_err());
    assert!(validate_height(1).is_ok());
    assert!(validate_height(1000).is_ok());
}

// manager.rs 测试（PNG 编码）
#[test]
fn test_encode_rgba_to_png() {
    let rgba = vec![255, 0, 0, 255; 24 * 24];
    let png = encode_rgba_to_png(&rgba, 24, 24).unwrap();
    assert_eq!(&png[0..4], &[0x89, 0x50, 0x4E, 0x47]); // PNG header
}

#[test]
fn test_encode_rgba_to_png_preserves_transparency() {
    let rgba = vec![0, 0, 0, 0; 24 * 24]; // 全透明
    let png = encode_rgba_to_png(&rgba, 24, 24).unwrap();
    assert!(!png.is_empty());
}
```

**运行命令**：
```bash
bash D:/workspace/tauri/tauri/.claude/skills/ohos-rust-ut/scripts/run-ut.sh statusbar
```

### 9.2 端到端测试（frontend-api-testing）

```typescript
// 验证修正后的 PixelMap 创建
{
  name: 'tray.icon.creation',
  category: 'auto',
  async fn() {
    const { TrayIcon } = await import('@tauri-apps/plugin-tray');
    const icon = await loadTestIcon();
    const tray = await TrayIcon.new({ icon });
    assert(tray.id !== undefined, 'tray created successfully');
    tray.destroy();
  },
},
```

### 9.3 手动测试清单

| 编号 | 测试项 | 操作 | 预期结果 | 类型 |
|------|--------|------|----------|------|
| V1 | PixelMap 创建 | 创建托盘图标 | 图标正常显示 | **manual** |
| V2 | 图标更新 | 调用 `setIcon` | 图标正常变化 | **manual** |
| V3 | 弹窗高度更新 | 调用 `update_quick_operation_height` | 左键点击后弹窗高度变化 | **manual** |
| V4 | 一级菜单更新 | 调用 `update_status_bar_menu_item` | 右键菜单项内容变化 | **manual** |
| V5 | 二级菜单更新 | 调用 `update_status_bar_sub_menu_item` | 右键子菜单项内容变化 | **manual** |
| V6 | 左键点击事件 | 左键点击图标 | 收到 Click 事件 | **manual** |
| V7 | 右键菜单事件 | 右键点击菜单项 | 收到 Click { Right } 事件 | **manual** |

---

## 十、风险项

| 风险 | 描述 | 应对 | 状态 |
|------|------|------|------|
| P1 验证结果 | `image.createPixelMap(buffer, options)` 可能不可用 | 若失败则切换到方案 B（PNG 编码） | ⬜ 待 OHOS 验证 |
| `image` 模块未注入 | `DefaultXComponent.ets` 当前未注入 `image`，`manager.rs:303` 会运行时失败 | 按 [2.3](#23-实现变更) 补充 ArkTS 侧变更 | ⬜ 待实现 |
| Promise 返回值忽略 | `updateStatusBarMenuItem`/`updateStatusBarSubMenuItem` 返回 `Promise<void>`，当前同步调用忽略返回值 | 与 Phase 1 的 `update_hover_tips` 保持一致，若出现异步问题再调整 | ⚠️ 需关注 |
| PNG 编码依赖 | 方案 B 需要添加 `image` crate 依赖 | 仅启用 `png` feature，控制体积 | ⬜ 待确认 |
| 新增 API 版本 | `updateStatusBarMenuItem` 需要 6.1.1(24)+ | 低版本调用会返回错误，需在文档中说明 | ⬜ 待处理 |
| PNG 编码性能 | 方案 B 中 24x24 图标 PNG 编码约 10-50μs | 图标创建非高频操作，可接受 | ✅ 可接受 |

---

## 十一、实现优先级

| 优先级 | 问题 | 原因 |
|--------|------|------|
| **P0** | P1: `createPixelMap` 验证 + ArkTS `image` 注入 | 阻塞图标显示，需优先验证当前路径是否可用 |
| P1 | P2: `updateQuickOperationHeight` | 功能增强，按需实现 |
| P2 | P3: `updateStatusBarMenuItem` | 性能优化，后续实现 |
| P2 | P4: `updateStatusBarSubMenuItem` | 性能优化，后续实现 |
| P3 | P5: 事件回调验证 | 当前实现正确，仅需设备验证 |

---

## 十二、完成后通知

本模块完成后，通知集成测试阶段开始。

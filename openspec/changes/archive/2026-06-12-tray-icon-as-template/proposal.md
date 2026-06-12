## Why

`TrayIconBuilder::icon_as_template(true)` 和 `TrayIcon::set_icon_as_template(true)` 在 OHOS 上是 no-op。当前 `icon_to_status_bar_icon()` 将同一张 RGBA 图同时填入 `StatusBarIcon.white` 和 `.black` 两个字段，未利用 OHOS 的深浅色壁纸双图标自动切换机制。当用户更换深色壁纸时，白色图标在白色背景上几乎不可见。

## What Changes

- 修改 `icon_to_status_bar_icon()` 接受 `is_template` 参数，当 `true` 时自动生成白色版（所有非透明像素 RGB→255）和黑色版（RGB→0），分别填入 `white` / `black` 字段
- 在 OHOS platform impl 中实现 `set_icon_as_template()`，更新 `attrs.icon_is_template` 并重新构建图标
- 在 OHOS platform impl 中实现 `set_icon_with_as_template()`，同时更新图标和 template 模式
- 修改 `set_icon()` 在重建图标时尊重 `attrs.icon_is_template` 属性
- 移除 `set_icon_as_template` 和 `set_icon_with_as_template` 的 `#[cfg(target_os = "macos")]` 门控，添加 OHOS 分支
- 添加前端 auto 测试和手动测试用例

## Capabilities

### New Capabilities

- `tray-icon-template`: 托盘图标 template 模式——根据系统壁纸深浅色自动切换白色/黑色图标，确保图标在任何壁纸下可见

### Modified Capabilities

（无现有 spec 需要修改）

## Impact

- **tray-icon crate**: `src/platform_impl/ohos/icon.rs` — 双色图标生成逻辑
- **tray-icon crate**: `src/platform_impl/ohos/mod.rs` — `set_icon_as_template()` + `set_icon_with_as_template()` + `set_icon()` 修改
- **tray-icon crate**: `src/lib.rs` — `set_icon_as_template()` 和 `set_icon_with_as_template()` cfg 门控修改
- **tauri crate**: `crates/tauri/src/tray/mod.rs` — 移除 macOS-only 门控，添加 OHOS 分支
- **前端测试**: `examples/api/src/lib/tests/tray.ts` — auto 测试
- **前端测试**: `examples/api/src/views/TestRunner.svelte` — 手动测试按钮
- **前端 UI**: `examples/api/src/views/Tray.svelte` — 已有 iconAsTemplate checkbox，无需修改

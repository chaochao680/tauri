## 1. tray-icon crate — 双色图标生成

- [x] 1.1 在 `tray-icon/src/platform_impl/ohos/icon.rs` 中添加 `to_monochrome(rgba, color)` 函数：将非透明像素的 RGB 替换为目标色，保留 alpha 通道
- [x] 1.2 修改 `icon_to_status_bar_icon()` 签名，添加 `is_template: bool` 参数；当 `true` 时 white 填白色版、black 填黑色版；当 `false` 时保持当前行为（同一张图）
- [x] 1.3 为 `to_monochrome` 添加 Rust unit test：验证白色版 RGB=255、黑色版 RGB=0、alpha 保留
- [x] 1.4 为 `icon_to_status_bar_icon` 添加 Rust unit test：`is_template=true` 时 white/black 不同；`is_template=false` 时 white/black 相同

## 2. tray-icon crate — OHOS platform impl

- [x] 2.1 在 `tray-icon/src/platform_impl/ohos/mod.rs` 中新增 `set_icon_as_template()` 方法：更新 `attrs.icon_is_template`，调用 remove → `build_item_from_attrs()` → add 重建
- [x] 2.2 修改 `set_icon()` 方法：在调用 `icon_to_status_bar_icon()` 时传入 `self.attrs.borrow().icon_is_template`
- [x] 2.3 修改 `new()` 中 `icon_to_status_bar_icon()` 调用：传入 `attrs.icon_is_template`
- [x] 2.4 添加 Rust unit test：`set_icon_as_template(true)` 后 attrs 更新
- [x] 2.5 在 `tray-icon/src/platform_impl/ohos/mod.rs` 中新增 `set_icon_with_as_template()` 方法：更新 `attrs.icon_is_template` 后调用 `set_icon()`，复用 D4 的 template 感知逻辑

## 3. tray-icon crate — lib.rs 层

- [x] 3.1 修改 `set_icon_as_template()`：移除 `#[cfg(target_os = "macos")]` 门控，添加 `#[cfg(target_env = "ohos")]` 分支直接调用 `self.tray.borrow_mut().set_icon_as_template(is_template)`
- [x] 3.2 移除 `with_icon_as_template()` 文档注释中的 "**macOS only**" 标注，改为说明 OHOS 也支持
- [x] 3.3 修改 `set_icon_with_as_template()`：cfg 门控从 `#[cfg(target_os = "macos")]` 改为 `#[cfg(any(target_os = "macos", target_env = "ohos"))]`，OHOS 分支调用 `self.tray.borrow_mut().set_icon_with_as_template(icon, is_template)`

## 4. tauri crate — tray 层

- [x] 4.1 修改 `tauri/crates/tauri/src/tray/mod.rs` 中 `set_icon_as_template()`：添加 `#[cfg(target_env = "ohos")]` 分支直接调用 `self.inner.set_icon_as_template(is_template)`，移除 `#[allow(unused)]` 标注（OHOS 分支使用后参数不再是 unused）
- [x] 4.2 移除 `icon_as_template()` builder 方法和 `set_icon_as_template()` 文档注释中的 "**macOS only**" 标注

## 5. 前端 auto 测试

- [x] 5.1 在 `examples/api/src/lib/tests/tray.ts` 中添加 `TrayIcon.setIconAsTemplate_true` 测试用例：创建 tray 后调用 `setIconAsTemplate(true)`，不抛异常
- [x] 5.2 添加 `TrayIcon.setIconAsTemplate_false` 测试用例：调用 `setIconAsTemplate(false)`，不抛异常
- [x] 5.3 添加 `TrayIcon.setIconAsTemplate_toggle` 测试用例：先 true 后 false，不抛异常

## 6. 前端手动测试

- [x] 6.1 在 `examples/api/src/views/TestRunner.svelte` 中添加手动测试 handler `manualIconAsTemplate`：调用 `tray.setIconAsTemplate(true)`，提示用户观察状态栏图标在深色/浅色壁纸下的变化
- [x] 6.2 在 Manual Tests 区域添加按钮 `<button>Icon as Template (check wallpaper)</button>`

## 7. 编译验证与设备测试

- [x] 7.1 `cargo check -p tray-icon` Windows 编译通过
- [x] 7.2 `cargo test -p tray-icon --lib` Rust unit test 全部通过
- [x] 7.3 OHOS 交叉编译通过：`cargo check -p tray-icon --target aarch64-unknown-linux-ohos`
- [x] 7.4 使用 ohos-build skill 构建 HAP 并安装到设备
- [x] 7.5 设备验证：auto test 报告中 setIconAsTemplate 用例全部 pass
- [x] 7.6 设备验证：手动测试 — 勾选 Icon as template 创建 tray → 切换壁纸 → 图标颜色自动切换
- [x] 7.7 设备验证：手动测试 — 取消 template → 图标恢复原始颜色
- [x] 7.8 设备验证：Tray.svelte 中 iconAsTemplate checkbox 功能正常

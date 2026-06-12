## Context

当前 OHOS tray icon 实现中，`icon_to_status_bar_icon()` 将用户提供的 RGBA 图标数据**原封不动**地同时填入 `StatusBarIcon.white` 和 `StatusBarIcon.black` 两个字段。OHOS 系统根据壁纸深浅色自动选择 white 或 black 图标显示，但由于两者是同一张图，深色壁纸下白色图标会看不清。

`TrayIconBuilder::icon_as_template(bool)`、`TrayIcon::set_icon_as_template(bool)` 和 `TrayIcon::set_icon_with_as_template(icon, bool)` 在 OHOS 上是 no-op（`#[cfg(target_os = "macos")]` 门控）。前端 `Tray.svelte` 已有 `iconAsTemplate` checkbox，但 OHOS 上无效果。

关键文件：
- `tray-icon/src/platform_impl/ohos/icon.rs` — `icon_to_status_bar_icon()` 和 RGBA 缩放
- `tray-icon/src/platform_impl/ohos/mod.rs` — `TrayIcon::new()`, `set_icon()`
- `tray-icon/src/lib.rs` — `TrayIconAttributes.icon_is_template`, `set_icon_as_template()`, `set_icon_with_as_template()`
- `tauri/crates/tauri/src/tray/mod.rs` — tauri 层 `set_icon_as_template()`

## Goals / Non-Goals

**Goals:**
- OHOS 上 `icon_as_template = true` 时，自动生成白色版和黑色版 PixelMap，系统根据壁纸自动切换
- `set_icon_as_template()` 运行时可切换 template 模式，立即生效
- `set_icon()` 更换图标后保持 template 设置，自动重新生成双色图标
- 添加 auto 测试和手动测试验证

**Non-Goals:**
- 不改变 macOS 行为（NSImage template 机制不变）
- 不改变 Windows 行为（始终 no-op）
- 不做半透明/渐变着色处理（只处理纯 alpha 蒙版）

## Decisions

### D1: 白色/黑色图标生成算法 — Alpha Mask 法

**选择**：将所有非透明像素的 RGB 设为目标色（白=255, 黑=0），保留原始 alpha 通道。

```rust
fn to_monochrome(rgba: &[u8], color: u8) -> Vec<u8> {
    rgba.chunks(4)
        .flat_map(|pixel| [color, color, color, pixel[3]])
        .collect()
}
```

**理由**：
- template 图标的设计初衷就是单色蒙版，与 macOS NSImage template 语义一致
- 实现简单，无外部依赖
- 保留 alpha 通道确保抗锯齿和半透明效果不丢失

**替代方案**：亮度加权灰度 → 丢弃原始颜色信息，不适合 template 语义（template 就是忽略颜色的）

### D2: 修改 `icon_to_status_bar_icon` 签名

**选择**：添加 `is_template: bool` 参数。

```rust
pub fn icon_to_status_bar_icon(
    icon: &PlatformIcon,
    is_template: bool,
) -> crate::Result<openharmony_ability::statusbar::StatusBarIcon>
```

- `is_template = false`：white 和 black 填同一张图（当前行为，不变）
- `is_template = true`：white = 白色版，black = 黑色版

**理由**：最小改动，函数内部完成所有逻辑，调用者无需关心颜色转换。

### D3: `set_icon_as_template` 实现方式 — remove + rebuild + add

**选择**：与 `set_title` 一致，通过 remove → `build_item_from_attrs()` → add 实现。

```rust
pub fn set_icon_as_template(&mut self, is_template: bool) -> crate::Result<()> {
    self.attrs.borrow_mut().icon_is_template = is_template;
    if *self.is_visible.borrow() {
        let app = get_ohos_app();
        openharmony_ability::statusbar::remove_from_status_bar(app).ok();
        let item = build_item_from_attrs(&self.attrs.borrow())?;
        openharmony_ability::statusbar::add_to_status_bar(app, &item)
            .map_err(|e| crate::Error::OhosError(e.to_string()))?;
    }
    Ok(())
}
```

**理由**：OHOS StatusBar API 没有 `updateStatusBarIconTemplate()` 方法，必须 remove + add。与现有 `set_title()` 的内联 remove-rebuild-add 模式一致（不抽取公共方法，保持与 `set_title` 相同的粒度）。

### D4: `set_icon` 需尊重 `icon_is_template`

**选择**：`set_icon()` 在重建 `StatusBarIcon` 时读取 `attrs.icon_is_template`，传入 `icon_to_status_bar_icon()`。

**理由**：用户可能先设 template 模式，后换图标。如果不尊重 template 属性，换图标后 template 效果会丢失。

### D5: tauri 层门控移除

**选择**：移除 `set_icon_as_template` 的 `#[cfg(target_os = "macos")]` 门控，添加 `#[cfg(target_env = "ohos")]` 分支。

```rust
pub fn set_icon_as_template(&self, is_template: bool) -> crate::Result<()> {
    #[cfg(target_os = "macos")]
    run_item_main_thread!(self, |self_: Self| {
        self_.inner.set_icon_as_template(is_template)
    })?;
    #[cfg(target_env = "ohos")]
    {
        self.inner.set_icon_as_template(is_template)?;
    }
    #[cfg(not(any(target_os = "macos", target_env = "ohos")))]
    let _ = is_template;
    Ok(())
}
```

**理由**：OHOS 的 TSFN 调用不需要 `run_on_main_thread`，直接调用即可（与 `set_visible` 等现有 OHOS 分支一致）。

### D6: `set_icon_with_as_template` 组合方法

**选择**：在 OHOS platform impl 中新增 `set_icon_with_as_template()` 方法，同时更新 `attrs.icon_is_template` 和调用 `set_icon()`，复用 D4 的 template 感知逻辑。`lib.rs` 的 cfg 门控从 `#[cfg(target_os = "macos")]` 改为 `#[cfg(any(target_os = "macos", target_env = "ohos"))]`。

```rust
// platform_impl/ohos/mod.rs
pub fn set_icon_with_as_template(&mut self, icon: Option<crate::Icon>, is_template: bool) -> crate::Result<()> {
    self.attrs.borrow_mut().icon_is_template = is_template;
    self.set_icon(icon.map(|i| i.inner))
}
```

**理由**：`lib.rs:526` 已有 `set_icon_with_as_template()` 方法但仅 macOS 可用。OHOS 需要同时支持此方法，避免用户调用时在 OHOS 上静默 no-op。实现上先更新 template 属性再调 `set_icon()`，由 D4 保证新图标自动按 template 模式生成双色版。

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| 非单色图标使用 template 模式时颜色丢失 | 文档标注 template 仅适用于单色图标；这是 macOS 也有的限制 |
| `set_icon_as_template` 的 remove+add 可能导致图标闪烁 | 与现有 `set_title` 行为一致，闪烁时间极短（<100ms） |
| 缩放后再着色可能产生锯齿 | 先缩放后着色，着色只改 RGB 不改 alpha，抗锯齿保留 |
| Windows 上 `icon_as_template` 仍为 no-op | 不变，Windows 任务栏图标无此概念，行为一致 |

## Why

wry OHOS 的 `set_bounds` 对**非子 webview**（主 webview）是 cache-only（仅更新 `bounds_cache`，不调 ArkTS `setBounds`），导致主 webview 的 `set_bounds()` 调用无实际效果——bounds 缓存更新了但 Web 组件未重渲染。R74 透明背景经 archive `p1-webview-transparent` 已实现（ArkHelper `init.transparent` + DefaultWebview `RenderMode.SYNC_RENDER` + 容器防御性透明 + `set_background_color` 动态更新），本 Phase 核实并关闭。

## What Changes

- **R78 非子 set_bounds**：移除 wry `set_bounds` 的 `if !self.is_child { cache-only; return; }` 早返回，使**子与非子 webview 都调** `self.webview.set_bounds(x, y, w, h)` + 更新缓存。ArkTS `setBounds` 已实现（ArkHelper:317 monkey-patch → `applyStyle({x,y,width,height})` → `updateWebviewStyle` → `node.update(newEntry)` → Web 组件按 `data.style.width/height/position` 重渲染），主 webview 的 Web 组件确实由 `data.style` 驱动布局，故 setBounds 对主 webview 生效。
- **R74 透明背景**：核实已闭环（无代码改动）——archive `p1-webview-transparent` 的全部改动已在代码中（ArkHelper `init.transparent=true` + DefaultWebview `RenderMode.SYNC_RENDER` + `set_background_color` 动态更新）。标注 R74 关闭。

## Capabilities

### New Capabilities
- `webview-bounds-nonchild`: 非子（主）webview 的 `set_bounds` 调用 ArkTS `setBounds` 实际生效（经 `updateWebviewStyle` 重渲染 Web 组件）

### Modified Capabilities
- `webview-transparent-bg`（archive p1-webview-transparent）：R74 核实已闭环，无代码变更，仅标注关闭

## Impact

- **wry**（Rust）：`src/ohos/mod.rs` 的 `set_bounds` 移除非子 cache-only 早返回（~4 行删除）
- **openharmony-ability**：无改动（ArkTS `setBounds` 已实现）
- **平台一致性**：与 Windows/macOS 的 `set_bounds` 行为对齐（主 webview 可设 bounds）
- **铁律遵守**：wry 改动限于 `cfg(target_env="ohos")` 路径；ArkTS 调用经 openharmony-ability（已有）

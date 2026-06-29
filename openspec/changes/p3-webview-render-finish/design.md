## Context

wry OHOS `InnerWebView::set_bounds`（`wry/src/ohos/mod.rs:483-497`）：
- 非子（主）webview：`if !self.is_child { cache-only; return; }`（484-488）——仅更新 `bounds_cache`，不调 ArkTS
- 子 webview：调 `self.webview.set_bounds(x,y,w,h)` + 缓存（489-496）

ArkTS `setBounds`（ArkHelper.ets:317-319 monkey-patch）→ `applyStyle({x,y,width,height})` → `updateWebviewStyle(webTag, style)` → `node.update(newEntry)` → Web 组件按 `data.style.width/height/position` 重渲染。主 webview 的 Web 组件 `.width(data.style?.width ?? "100%")` + `.position({...})`（DefaultWebview.ets:119-121）——**确实由 `data.style` 驱动布局**，故 `setBounds` 对主 webview 生效。

R74 透明背景：archive `p1-webview-transparent` 已实现（ArkHelper `init.transparent=true`、DefaultWebview `RenderMode.SYNC_RENDER`、容器防御性透明、`set_background_color` 动态更新）。代码核实闭环。

## Goals / Non-Goals

**Goals:**
- R78：非子 webview `set_bounds` 调 ArkTS `setBounds`（移除 cache-only 早返回），主 webview 的 set_bounds 实际生效
- R74：核实透明背景已闭环，标注关闭

**Non-Goals:**
- 不改动 ArkTS `setBounds`（已实现）
- 不改动 transparent 实现（已闭环）
- 不处理主 webview set_bounds 与窗口 resize 的交互（set_bounds 设具体值后 Web 组件按该值布局，窗口 resize 时需调用方重新 set_bounds——与桌面平台行为一致）

## Decisions

### D1: 非子 cache-only 是正确行为（经尝试移除后回退）
原计划移除非子 cache-only 早返回，让主 webview 也调 ArkTS `setBounds`。经设备验证发现：`setBounds` → `applyStyle({x,y,width,height})` 会把 Web 组件的 `data.style.width/height` 从 `"100%"` 替换为具体像素值，导致全屏/窗口 resize 时 Web 组件不再自动填满窗口 → **左侧和下方黑边**。回退后 cache-only 恢复正确行为：主 webview 始终经 `"100%"` 填满窗口，`set_bounds` 仅更新 `bounds_cache`（`bounds()` 返回缓存值），不调 ArkTS setBounds。子 webview 仍调 ArkTS setBounds（使用绝对定位，不受影响）。

### D2: R74 透明背景关闭
archive `p1-webview-transparent` 全部改动已在代码中。R74 标注关闭，无代码变更。

## Risks / Trade-offs

- **R78 cache-only 是平台限制**：主 webview 的 Web 组件经 `"100%"` 宽高填满窗口，set_bounds 不应对其生效（否则黑边）。cache-only 是正确行为，非实现缺陷。`bounds()` 返回缓存值（调用方设置的值），不反映实际窗口尺寸——与桌面平台的"主 webview bounds = 窗口 bounds"语义略有差异，但 OHOS 主 webview 无独立 bounds 概念。
- **全屏回归防护**：添加 manual_tests.md 7.4 "全屏无黑边"用例（T0），防止后续误移除 cache-only。

## Context

`tauri/crates/tauri/src/webview/plugin.rs` 中，所有 webview 命令（getter/setter）定义在 `#[cfg(desktop)] mod desktop_commands` 模块内：

```rust
#[cfg(desktop)]                                    // line 42
mod desktop_commands {
  // getter!/setter! 宏定义
  // setter!(set_webview_background_color, set_background_color, Option<Color>);  // line 124-128
  // ... 其他所有命令
}

// 注册处 (line 232+)
builder.invoke_handler(crate::generate_handler![
  // ...
  #[cfg(desktop)] desktop_commands::set_webview_background_color,  // line 246
  // ...
])
```

`#[cfg(desktop)]` 在 OHOS mobile 模式下为 `false`（由 `TAURI_OHOS_DEVICE_TYPE` 环境变量控制），导致整个模块不编译，命令不注册。

**但底层已完全支持**：
- wry OHOS 后端 `wry/src/ohos/mod.rs:244` — `set_background_color()` 已实现 RGBA → `0xAARRGGBB` 转换
- openharmony-ability `helper/webview.rs:241` — `set_background_color` NAPI 方法已实现
- ArkTS 端 `ArkHelper.ets` — monkey-patch `setBackgroundColor` 已实现 `applyStyle` → WebBuilder 重渲染

**对比 window 级别命令**（`window/plugin.rs:308`）：
```rust
commands::set_background_color,  // 无 cfg(desktop) 门控，全平台可用 ✅
```

## Goals / Non-Goals

**Goals:**
- OHOS mobile 上 `plugin:webview|set_webview_background_color` 命令可用
- `webviewWindow.setBackgroundColor()` 链式调用在 OHOS 上完整成功
- 不改变 Desktop 平台的任何现有行为

**Non-Goals:**
- 不修改 wry OHOS 后端实现（Phase 1 已完成）
- 不修改 openharmony-ability NAPI 实现（Phase 1 已完成）
- 不修改 ArkTS 端 monkey-patch 实现（Phase 1 已完成）
- 不修改 TypeScript API 层（`webviewWindow.ts` 已正确实现）
- 不将其他 `desktop_commands` 命令提取到全平台（仅提取 `set_webview_background_color`）

## Decisions

### Decision 1: 新增独立 `commands` 模块而非移除整个 `#[cfg(desktop)]`

**选择**：新建一个不受 `#[cfg(desktop)]` 限制的 `mod commands`，仅包含 `set_webview_background_color`。保留 `desktop_commands` 模块及其 `#[cfg(desktop)]` 门控不变。

```rust
// 新增：无平台限制的命令模块
mod commands {
  use super::*;
  use crate::{command, utils::config::Color, Webview, Runtime};

  fn get_webview<R: Runtime>(...) -> crate::Result<Webview<R>> { ... }

  #[command(root = "crate")]
  pub async fn set_webview_background_color<R: Runtime>(...) -> crate::Result<()> { ... }
}

// desktop_commands 保持原样
#[cfg(desktop)]
mod desktop_commands {
  // ... 其他命令不变
  // 删除: setter!(set_webview_background_color, set_background_color, Option<Color>);
}
```

**理由**：
- `desktop_commands` 中的其他命令（`webview_close`、`set_webview_size`、`set_webview_zoom`、`print` 等）可能确实仅适用于 desktop 平台，盲目移除 `#[cfg(desktop)]` 可能引入未验证的行为
- 最小改动原则：仅提取已验证全平台可用的命令
- 与 `window/plugin.rs` 的模式一致（`commands::set_background_color` 在全平台模块，`desktop_commands` 的命令在 `#[cfg(desktop)]` 模块）

**替代方案 1**：移除整个 `#[cfg(desktop)]` 门控 → 风险过大，影响 10+ 命令
**替代方案 2**：为 OHOS 单独添加 `#[cfg(target_env = "ohos")]` 分支 → 不如直接让命令对所有平台可用，且违反了约束文档"不要滥用 cfg"原则

### Decision 2: 不提取其他 desktop_commands 命令

**选择**：仅提取 `set_webview_background_color`，不提取 `set_webview_zoom`、`webview_hide`、`webview_show` 等其他命令。

**理由**：
- `set_background_color` 的底层实现在所有平台（Windows/macOS/Linux/OHOS）都已就绪
- 其他命令的 OHOS 支持状态未经验证（如 `set_webview_zoom` 在 OHOS ArkWeb 上是否可用）
- 每个命令的全平台可用性应单独验证后逐个提取

## cfg 隔离策略

| 修改 | 隔离方式 | 说明 |
|------|---------|------|
| `mod commands` 新增 | 无 cfg 门控（全平台） | 命令对所有平台可用 |
| `commands::set_webview_background_color` 注册 | 无 cfg 门控（全平台） | 替代原来的 `#[cfg(desktop)]` |
| `desktop_commands` 内删除 `setter!` | 保持 `#[cfg(desktop)]` | 避免重复定义 |

**结论**：不添加任何新 cfg gate，仅移除一个 `#[cfg(desktop)]` 限制。

## Skill 校验结果

| 检查项 | 来源 | 结果 |
|--------|------|------|
| cfg 隔离规则 | ohos-constraints §1.1 | ✅ 不新增 OHOS cfg，仅移除 desktop 限制 |
| OHOS `target_os` 是 `"linux"` | ohos-constraints §5.4 | ✅ 不涉及 Linux 依赖 |
| desktop/mobile 区分 | ohos-constraints §1.1 | ✅ 命令在 mobile + desktop 均可用 |
| 不影响其他平台 | review-checklist B1-B3 | ✅ Desktop 行为不变 |
| openharmony-ability 桥接 | review-checklist F1-F2 | ✅ 调用链仍走 wry → openharmony-ability |
| NAPI/TSFN | review-checklist C | ✅ 不涉及 |
| 线程模型 | review-checklist D | ✅ 不涉及 |
| ArkTS 框架 | review-checklist E | ✅ 不涉及 |
| API 版本隔离 | ohos-version-isolation | ✅ 不涉及高版本 API |
| 审计清单 | audit-checklist | ✅ 全部通过 |

## Risks / Trade-offs

- **[其他 desktop_commands 命令未全平台可用]** 仅提取了 `set_webview_background_color`，其他 webview 命令（zoom/hide/show 等）在 OHOS mobile 上仍不可用 → 这是有意为之的保守策略，后续可按需逐个验证和提取
- **[Android/iOS 命令注册但后端未实现]** 命令注册扩展到全平台后，Android/iOS 上如果 wry 后端未实现 `set_background_color`，调用会返回 error 而非 panic → 可接受，与 Desktop 上调用不存在命令的行为一致

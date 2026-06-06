## Why

Phase 1-3 分别实现了 WebView 透明、无标题栏和窗口背景透明三个独立功能。但端到端使用时，三个功能需要协同工作：`decorations: false` + `transparent: true` + WebView `background-color: transparent` 才能实现完整的桌面穿透效果。当前各 Phase 独立设计，存在以下端到端集成风险：
- `create_os_window` 函数签名需要同时支持 decorations + transparent + backgroundColor 参数，但各 Phase 独立扩展可能导致签名不一致
- 主窗口（window_id=0）的 decorations/transparent/background_color 需要在 ArkTS 侧统一处理，但各 Phase 分别设计了独立的处理路径
- tauri-runtime-wry → wry → tao 的完整传递链路尚未验证，可能存在属性丢失
- 多窗口场景下各窗口独立的 decorations/transparent 状态可能相互干扰

## What Changes

- **create_os_window 签名整合**：统一 `create_os_window` 函数签名，确保 decorations、transparent、backgroundColor 参数同时可用且一致
- **主窗口属性统一处理**：在 ArkTS 侧为主窗口（window_id=0）建立统一的属性处理入口，集中处理 decorations + transparent + background_color
- **端到端传递链路验证**：验证 `tauri config → tauri-runtime-wry → wry → tao → NAPI → ArkTS` 全链路属性传递完整性
- **多窗口状态隔离验证**：验证多个窗口各自的 decorations/transparent/background_color 状态独立
- **组合场景测试**：decorations=false + transparent=true、decorations=false + background_color 等组合场景

## Capabilities

### New Capabilities
- `e2e-integration`: 端到端集成验证，覆盖 Phase 1-3 功能的组合场景和全链路传递

### Modified Capabilities
（无现有 capability 的需求变更，但可能需要修复 Phase 1-3 设计中发现的集成问题）

## Impact

- **tao** (Rust)：`src/platform_impl/ohos/mod.rs` 可能需要调整 `create_os_window` 调用签名和主窗口初始化逻辑
- **openharmony-ability** (Rust)：`crates/ability/src/window/mod.rs` 的 `create_os_window` 最终签名需整合 Phase 2+3 参数
- **openharmony-ability** (ArkTS)：`ArkHelper.ets` 和 `WindowManager.ets` 可能需要统一主窗口属性处理入口
- **测试覆盖**：新增端到端集成测试用例

## Why

OHOS 平台上，当系统准备销毁 UIAbility 时（用户从最近任务列表关闭应用、调用 `terminateSelf()` 等），tauri-runtime-wry 收到 `Event::LoopDestroyed` 后直接发送 `RunEvent::Exit`，跳过了 `RunEvent::ExitRequested`。用户无法在 `app.run()` 回调中拦截退出（如保存未保存的数据、显示确认对话框）。在 Windows/macOS 上，`ExitRequested` 正常工作，这是 OHOS 平台的功能缺失。

## What Changes

- 在 openharmony-ability 中实现 OHOS `UIAbility.onPrepareToTerminate` 生命周期回调，使 ArkTS 层能在 Ability 销毁前通知 Rust 层
- 在 openharmony-ability 的 `Event` enum 和 `WindowStageEventCallback` 中新增 `PrepareToTerminate` 变体和回调接口
- 在 tao OHOS 适配层新增 `MainEvent::PrepareToTerminate` 事件，并映射为 tao 事件
- 修改 tauri-runtime-wry 的 `LoopDestroyed` 处理逻辑：先触发 `RunEvent::ExitRequested`，根据用户是否调用 `prevent_exit()` 决定是取消终止还是继续退出
- 保留 `LoopDestroyed` → `Exit` 的 fallback 路径（应对 `onPrepareToTerminate` 未触发的场景）

## Capabilities

### New Capabilities
- `ohos-exit-flow`: OHOS 平台的应用退出拦截机制 — 通过 `onPrepareToTerminate` 生命周期回调实现 `RunEvent::ExitRequested` 的正确触发，支持用户代码拦截退出

### Modified Capabilities
<!-- 无需修改现有 spec -->

## Impact

- **openharmony-ability crate** (`crates/ability/src/event.rs`, `crates/ability/src/lifecycle.rs`): 新增事件变体和 NAPI 回调
- **openharmony-ability ArkTS** (`native_ability/.../ability/NativeAbility.ets`, `type.ets`): 重写 `onPrepareToTerminate` 方法 + 接口扩展
- **tao** (`src/platform_impl/ohos/mod.rs`): 新增 MainEvent 变体 + 事件映射
- **tauri-runtime-wry** (`crates/tauri-runtime-wry/src/lib.rs`): 修改 LoopDestroyed handler + 新增 PrepareToTerminate 处理
- **前置条件**: OHOS 设备需要开启系统参数 `persist.sys.prepare_terminate = true`

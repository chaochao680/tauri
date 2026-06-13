## Why

tauri-plugin-single-instance 在 OHOS 平台上没有 `platform_impl` 模块，导致编译失败。Phase 1 已完成 `want.parameters` 的桥接存储（`take_want_parameters()` getter），现在需要实现 OHOS 平台模块来消费这些数据，在 `RunEvent::Opened` 时触发用户 callback，完成单实例功能的端到端闭环。

## What Changes

- 新增 `platform_impl/ohos.rs`：OHOS 平台实现，hook `RunEvent::Opened` 事件，调用 `take_want_parameters()` 获取参数并触发用户 callback
- 在 `lib.rs` 添加 `cfg(target_env = "ohos")` 条件编译，指向 OHOS 平台模块
- 在 `Cargo.toml` 添加 `openharmony-ability` 作为 OHOS target 依赖

## Capabilities

### New Capabilities
- `ohos-single-instance`: OHOS 平台上的单实例检测与参数转发插件实现

### Modified Capabilities

## Impact

- **single-instance plugin**：新增 OHOS 平台支持（3 个文件）
- **依赖**：新增 `openharmony-ability` OHOS target 依赖
- **其他平台**：无影响，OHOS 代码在 `cfg(target_env = "ohos")` 隔离下

## Why

tauri-plugin-single-instance 在 OHOS 平台上无任何实现。OHOS 原生提供 singleton launchType（且为默认值），再次启动时通过 `onNewWant(want)` 回调通知已有实例。当前事件链已打通 `onNewWant` → `RunEvent::Opened`，但仅传递 `want.uri`，丢弃了 `want.parameters`（启动参数）。single-instance 插件需要这些参数来通知首实例"第二次启动携带了什么数据"，因此必须扩展事件链以携带完整参数。

## What Changes

- 扩展 `NativeAbility.ets` 的 `onNewWant` 处理，提取 `want.parameters` 序列化为 JSON 字符串并通过 Object 传递给 NAPI
- 扩展 NAPI `on_new_want` 桥接函数，从接收单个 `String` 改为接收包含 `uri` + `parametersJson` 的 `Object`
- 在 `openharmony-ability` Rust 侧新增全局存储（`static Mutex<String>`），保存最新的 `want.parameters`
- 暴露 `take_want_parameters() -> String` getter 函数，供其他 crate 读取（take 语义：读取后清空）
- `Event::NewWant` 保持不变，参数通过 Mutex getter 独立读取，不经事件链传递

## Capabilities

### New Capabilities
- `ohos-want-parameters`: 扩展 onNewWant 事件链，使 want.parameters 从 ArkTS 层传递到 Rust 层并可通过 getter 读取

### Modified Capabilities
<!-- 无现有 capability 需要修改 -->

## Impact

- **openharmony-ability**：ArkTS 层（NativeAbility.ets）+ Rust 层（lifecycle.rs, app.rs）修改
- **依赖**：single-instance 插件将新增 `openharmony-ability` 作为 OHOS target 依赖
- **兼容性**：`Event::NewWant` 增加字段为 additive change，不影响现有 `Event::Opened` 事件流
- **其他平台**：无影响，所有修改在 `cfg(target_env = "ohos")` 隔离下

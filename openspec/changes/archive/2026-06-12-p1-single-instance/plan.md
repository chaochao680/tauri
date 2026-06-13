# Single-Instance OHOS 适配计划

**创建时间**：2026-06-09
**功能描述**：适配 tauri-plugin-single-instance 到 OHOS 平台，利用 OHOS 原生 singleton launchType + onNewWant 机制实现单实例检测和参数转发
**判断依据**：涉及 2 个代码层 + 2 个传导修改层，预估 10 个文件，既有底层事件链又有上层插件实现

## Phase 列表

| Phase | 名称 | openspec change | 状态 | 涉及层 | 预估文件 | 验证方式 |
|-------|------|----------------|------|--------|---------|---------|
| 1 | onNewWant 参数转发 | p1_single-instance | ✓ 已归档 | openharmony-ability + tao + tauri | 5 | cargo check + 设备端 Rust UT |
| 2 | single-instance OHOS 实现 | p2_single-instance | ✓ 已归档 | single-instance plugin | 5 | 设备端功能验证（二次启动触发 callback）|

## Phase 详细说明

### Phase 1: onNewWant 参数转发
- **目标**：扩展 onNewWant 事件链，使其携带 want.parameters（启动参数 Map），从 ArkTS 层贯穿到 tauri RunEvent
- **文件列表**：
  - `openharmony-ability/native_ability/src/main/ets/ability/NativeAbility.ets` — 提取 want.parameters 并通过 TSFN 传递
  - `openharmony-ability/crates/ability/src/event.rs` — Event::NewWant 增加 parameters 字段
  - `openharmony-ability/crates/ability/src/lifecycle.rs` — NAPI 桥接传递 parameters HashMap
  - `tao/src/platform_impl/ohos/mod.rs` — 映射新事件字段到 tao Event
  - `tauri/crates/tauri/src/app.rs` — RunEvent 扩展携带 OHOS 参数
- **依赖**：无
- **验证**：cargo check 编译通过 + 设备端二次启动时 Rust 侧能收到 want.parameters

### Phase 2: single-instance OHOS 实现
- **目标**：新增 platform_impl/ohos.rs，利用 Phase 1 扩展的事件链实现单实例回调
- **文件列表**：
  - `plugins-workspace/plugins/single-instance/src/platform_impl/ohos.rs` — 新增 OHOS 平台实现
  - `plugins-workspace/plugins/single-instance/src/lib.rs` — 添加 cfg(target_env = "ohos") gate
  - `plugins-workspace/plugins/single-instance/Cargo.toml` — 添加 OHOS 平台支持元数据和依赖
- **依赖**：Phase 1 完成
- **验证**：设备端功能验证 — 启动 app → 再次点击图标 → 用户 callback 被调用并收到正确参数

## 核心设计思路

OHOS 适配不模仿 Windows（Mutex+WM_COPYDATA）、macOS（Unix Socket）或 Linux（D-Bus）的 IPC 模式。
而是**借力 OHOS 原生单实例机制**：
1. OHOS 默认 `launchType: "singleton"`，OS 层面阻止创建第二个 Ability 实例
2. 再次启动时 OS 调用 `onNewWant(want, launchParam)` 而非创建新实例
3. `want` 对象携带启动参数（URI、parameters 等）
4. 插件只需 hook `onNewWant` 事件，将参数转发给用户 callback

属于 **Direct Call 适配模式**（openharmony-ability 已有 onNewWant 桥接，只需扩展数据字段）。

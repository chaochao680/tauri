## Why

在 Tauri OHOS 适配过程中，不同插件的适配复杂度差异巨大：
- **process/updater 插件**：仅需修改 2-3 个文件，无需修改 tauri 核心仓库
- **dialog 插件**：需要修改 7+ 个文件，且依赖 tauri 核心仓库的 50+ 文件改动（PR #20）

这种差异引发了架构审视：
1. 为什么 process/updater 可以简单适配，而 dialog 不行？
2. tauri 核心 HAR 和 openharmony-ability HAR 的职责边界是什么？
3. 能否让 dialog 也像 process/updater 一样简单适配？
4. 当前的 PluginManager 架构是否合理？

本分析旨在澄清 OHOS 插件系统的架构决策，为后续插件适配提供指导。

## What Changes

这是一份**架构分析文档**，不涉及代码变更。分析内容包括：

### 核心发现

1. **两个 HAR 的职责分离**
   - `openharmony-ability`（`@ohos-rs/ability`）：框架级原生能力桥接（生命周期、窗口、WebView、菜单、updater）
   - tauri 核心 HAR（`@tauri/app`）：插件注册框架（Plugin.ets + PluginManager.ets）

2. **三种适配模式的对比**
   - **直接调用模式**（process）：复用 openharmony-ability 已有的 `appRecovery` 桥接
   - **平行替换模式**（updater）：复用 openharmony-ability 的 `updateManager` 桥接，整体门控桌面端代码
   - **插件注册模式**（dialog）：需要独立的 ArkTS 插件模块，通过 PluginManager 注册

3. **为什么 dialog 不能简单适配**
   - OHOS 被编译器判定为 `cfg(desktop) = true`，但行为必须是 mobile
   - `FileDialogBuilder`/`MessageDialogBuilder` 是同一 struct 跨平台，但部分字段/方法仅桌面有
   - 必须逐行修改 `#[cfg(desktop)]` → `#[cfg(all(desktop, not(target_env = "ohos")))]`
   - 这不是风格选择，是编译正确性问题

4. **架构决策的合理性**
   - PluginManager 模式与 Android/iOS 的插件架构一致（每个插件独立 HAR/AAR/framework）
   - 如果将 dialog 塞进 openharmony-ability，会导致：体积膨胀、发布耦合、职责模糊
   - 当前方案虽然改动量大，但长期可扩展性更好

## Impact

- **文档**：为后续插件适配提供架构指导
- **决策记录**：明确 tauri 核心 HAR 与 openharmony-ability 的职责边界
- **最佳实践**：总结三种适配模式的适用场景

## Stakeholders

- Tauri OHOS 适配开发者
- Tauri 插件维护者
- openharmony-ability 维护者
- 使用 Tauri 插件的 OHOS 应用开发者

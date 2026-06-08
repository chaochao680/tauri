## Why

`@tauri-apps/plugin-autostart` 目前仅支持 Windows、macOS 和 Linux，无法在 OHOS 平台上使用。OHOS 对开机自启动有严格的系统限制——普通应用无法程序化开启/关闭自启动，只能引导用户到系统设置页面手动操作。需要为 OHOS 平台实现适配层，使插件接口在 OHOS 上具备合理的语义映射。

## What Changes

- 在 `openharmony-ability` 仓库新增 `autostart` 模块，提供 Rust API + ArkTS 实现 + TSFN 桥接
- 修改 `plugins-workspace/plugins/autostart` 插件，为 OHOS 平台添加条件编译分支，调用 `openharmony-ability` 提供的接口
- `enable()` / `disable()` 在 OHOS 上引导用户跳转到系统"应用启动管理"设置页面
- `isEnabled()` 在 OHOS 上使用 `autoStartupManager.getAutoStartupStatusForSelf()` 查询状态（API 21+）

## Capabilities

### New Capabilities
- `ohos-autostart`: OHOS 平台开机自启动功能，包括状态查询和设置页引导跳转

### Modified Capabilities

（无）

## Impact

- **仓库**: openharmony-ability（新增模块）、plugins-workspace/plugin-autostart（条件编译）
- **API**: 前端 JS API 不变，后端 Rust 接口保持一致
- **依赖**: plugin-autostart 在 OHOS 上依赖 openharmony-ability 而非 auto-launch crate
- **版本要求**: `autoStartupManager` API 需要 API 21+，`startAbility` 跳转设置页需要系统版本 6.0.0.112+
- **平台限制**: OHOS 上 `enable()`/`disable()` 无法直接生效，只能引导用户手动操作——这是平台设计约束，非实现缺陷

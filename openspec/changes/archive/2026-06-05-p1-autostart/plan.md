# Autostart 适配计划

**创建时间**：2026-06-05
**功能描述**：为 @tauri-apps/plugin-autostart 实现 OHOS 平台的开机自启动功能（enable/disable/isEnabled）
**判断依据**：涉及 2 个代码层，预估 8 个文件（≤10），不拆分 Phase

## Phase 列表

| Phase | 名称 | openspec change | 状态 | 涉及层 | 预估文件 | 验证方式 |
|-------|------|----------------|------|--------|---------|---------|
| 1 | autostart 完整实现 | p1-autostart | ✓ 已归档 | openharmony-ability + plugin-autostart | 8 | 设备端功能测试 |

## Phase 详细说明

### Phase 1: autostart 完整实现

- **目标**：在 OHOS 平台实现 autostart 插件的三个接口
  - `enable()` - 引导用户到系统设置页面开启自启动
  - `disable()` - 引导用户到系统设置页面关闭自启动
  - `isEnabled()` - 查询当前自启动状态

- **文件列表**：
  - openharmony-ability:
    - `crates/ability/src/helper/autostart.rs`（新增）— TSFN 基础设施
    - `crates/ability/src/helper/mod.rs`（修改）— 注册 autostart 模块
    - `crates/ability/src/autostart.rs`（新增）— Rust API（AutostartManager）
    - `crates/ability/src/lib.rs`（修改）— 导出 autostart 模块
    - `native_ability/src/main/ets/helper/autostart.ets`（新增）— ArkTS 实现
    - `native_ability/src/main/ets/helper/index.ets`（修改）— 注册 helper 函数
  - plugins-workspace:
    - `plugins/autostart/src/lib.rs`（修改）— OHOS 条件编译
    - `plugins/autostart/Cargo.toml`（修改）— 添加 openharmony-ability 依赖

- **依赖**：无
- **平台限制**：
  - OHOS 普通应用无法程序化开启/关闭自启动，需引导用户到系统设置
  - `isEnabled()` 使用 `autoStartupManager.getAutoStartupStatusForSelf()`（API 21+）

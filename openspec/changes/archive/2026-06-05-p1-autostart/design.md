## Context

`@tauri-apps/plugin-autostart` 使用 `auto-launch` crate 在 Windows/macOS/Linux 上实现开机自启动。该 crate 不支持 OHOS 平台。

OHOS（HarmonyOS）对开机自启动有严格的系统级限制：
- **普通应用无法程序化开启/关闭自启动**，必须由用户在系统"应用启动管理"页面手动操作
- `autoStartupManager.getAutoStartupStatusForSelf()`（API 21+）可查询当前状态
- `startAbility()` 可跳转到系统设置页面引导用户操作

现有代码在 `lib.rs` 第 225-226 行已有 `cfg(target_env = "ohos")` 分支设置了 `app_path`，但 `auto-launch` crate 本身不支持 OHOS，会导致编译或运行失败。

## Goals / Non-Goals

**Goals:**
- 在 OHOS 平台实现 `enable()` / `disable()` / `isEnabled()` 三个接口的合理语义映射
- `isEnabled()` 使用 `autoStartupManager` 查询真实状态
- `enable()` / `disable()` 引导用户到系统设置页面
- 遵守 OHOS 约束（cfg 隔离、TSFN 规则、版本守卫）
- 不影响 Windows/macOS/Linux 的现有实现

**Non-Goals:**
- 不尝试绕过 OHOS 系统限制实现程序化开启/关闭（这是平台设计约束）
- 不实现企业 MDM 应用的 `addAutoStartApps`（需要特殊权限，不适用于 Tauri 场景）
- 不修改前端 JS API（保持不变）

## Decisions

### 1. 底层桥接架构：复用 openharmony-ability TSFN 模式

**决策**：在 `openharmony-ability` 仓库新增 `autostart` 模块，采用与 `updater` 相同的 TSFN 桥接模式。

**理由**：
- 遵循铁律 #1：openharmony-ability 是唯一 ArkTS 桥接仓
- 复用已有的 helper/TSFN 基础设施（`set_helper` / `get_helper` / `get_main_thread_env`）
- `updater` 模块已验证此模式可行

**替代方案**：直接在 plugin-autostart 中调用 NAPI → 违反铁律 #1

### 2. OHOS 实现不使用 auto-launch crate

**决策**：在 `plugin-autostart` 中，OHOS 平台通过 `cfg(target_env = "ohos")` 条件编译，使用 `openharmony-ability` 提供的 `AutostartManager` 替代 `auto-launch` crate。

**理由**：
- `auto-launch` crate 不支持 OHOS（target_os 是 linux 但实际运行环境是 OHOS）
- OHOS 需要 ArkTS 侧调用系统 API，必须通过 openharmony-ability 桥接

**实现方式**：
```rust
#[cfg(target_env = "ohos")]
use openharmony_ability::autostart::AutostartManager;

#[cfg(not(target_env = "ohos"))]
use auto_launch::AutoLaunch;
```

### 3. enable/disable 的实现：跳转设置页面

**决策**：`enable()` 和 `disable()` 都跳转到系统"应用启动管理"设置页面，由用户手动操作。

**理由**：
- OHOS 平台限制，普通应用无法程序化修改自启动状态
- 这是 HarmonyOS 设计规范的要求，不是实现缺陷

**ArkTS 实现**：
```typescript
const want: Want = {
  bundleName: 'com.huawei.hmos.settings',
  abilityName: 'com.huawei.hmos.settings.MainAbility',
  uri: 'pc_app_setup_settings'
};
context.startAbility(want);
```

### 4. isEnabled 的版本守卫

**决策**：`isEnabled()` 使用 `autoStartupManager.getAutoStartupStatusForSelf()`，需要 API 21+。对于 API < 21 的设备，返回 `false`（静默降级）。

**理由**：
- tauri api demo 默认 API 12，不能直接使用 API 21 的接口
- 遵循约束 6.3 的"强制回退值"降级模式
- 返回 `false` 比抛错更安全，不影响应用主流程

**Rust 侧版本守卫**（遵循 `ohos-version-api` spec，与已有模块一致）：

`autoStartupManager` 是 API 21 新增模块，必须在 Rust 侧使用 `version::sdk_api_version()` 做版本守卫，低版本时直接返回 `false`，不发起 TSFN 调用：

```rust
// crates/ability/src/autostart.rs
pub async fn is_enabled(&self) -> Result<bool> {
    // 版本守卫：autoStartupManager 需要 API 21+
    if crate::version::sdk_api_version() < 21 {
        return Ok(false);  // 静默降级
    }

    // 通过 TSFN 调用 ArkTS autostartIsEnabled()
    let tsfn = get_autostart_is_enabled_tsfn()
        .ok_or_else(|| Error::from_reason("autostart TSFN not initialized"))?;
    // ... TSFN 调用 ...
}
```

ArkTS 侧仍保留 try-catch 作为防御性编程（处理 Error 801 等运行时异常），但不做版本判断。

### 5. TSFN 设计：3 个函数

| TSFN | ArkTS 函数 | 参数 | 返回值 |
|------|-----------|------|--------|
| `AUTOSTART_ENABLE_TSFN` | `autostartEnable()` | 无 | `Promise<void>` |
| `AUTOSTART_DISABLE_TSFN` | `autostartDisable()` | 无 | `Promise<void>` |
| `AUTOSTART_IS_ENABLED_TSFN` | `autostartIsEnabled()` | 无 | `Promise<boolean>` |

遵循 TSFN 规则：
- `callee_handled::<false>()`（不使用 true，避免参数偏移）
- camelCase 命名（`autostartEnable` 而非 `autostart_enable`）
- 通过 `oneshot::channel` 桥接 Promise 到 Rust Future

### 6. plugin-autostart 中的 AutoLaunchManager 替代

**决策**：在 `plugin-autostart` 中，OHOS 平台的 `AutoLaunchManager` 包装 `openharmony_ability::autostart::AutostartManager`。

```rust
#[cfg(target_env = "ohos")]
pub struct AutoLaunchManager(openharmony_ability::autostart::AutostartManager);

#[cfg(not(target_env = "ohos"))]
pub struct AutoLaunchManager(auto_launch::AutoLaunch);
```

OHOS 的 `enable()` / `disable()` / `is_enabled()` 方法委托给 `openharmony-ability`。

### 7. Sync vs Async 桥接

**决策**：OHOS 的 `AutostartManager` 方法返回 `Future`（async），而非 OHOS 的 `AutoLaunch` 方法是同步的。在 `plugin-autostart` 中使用 `cfg` 区分调用方式。

**理由**：
- OHOS TSFN 调用 ArkTS 是异步的（Promise → Future）
- `auto-launch` crate 的方法是同步的
- 现有的 `#[command]` 函数已经是 `async fn`，可以 `.await`

**实现方式**：
```rust
#[command]
async fn enable(manager: State<'_, AutoLaunchManager>) -> Result<()> {
    #[cfg(not(target_env = "ohos"))]
    { manager.enable() }

    #[cfg(target_env = "ohos")]
    { manager.enable().await }
}
```

## Risks / Trade-offs

1. **[平台限制] enable/disable 无法直接生效** → 在 API 文档和日志中明确说明 OHOS 平台行为。用户调用 `enable()` 后需手动在设置页面开启。

2. **[版本限制] isEnabled 需要 API 21+** → 低版本设备返回 `false`，通过版本守卫确保不会崩溃。

3. **[UX 不一致] enable/disable 是"建议性"操作** → 与 Windows/macOS 的"直接生效"不同。考虑在前端 API 文档中标注 OHOS 平台差异。

4. **[设置页 URI 稳定性] `pc_app_setup_settings` 可能在不同版本/设备上不可用** → `startAbility` 失败时返回错误，不影响应用主流程。

5. **[HAR 重建] 修改 openharmony-ability 后需要重建 HAR** → 在 verify 阶段的标准流程中已覆盖。

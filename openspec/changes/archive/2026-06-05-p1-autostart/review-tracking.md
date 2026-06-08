# Autostart PR 检视意见跟踪

**创建时间**: 2026-06-05
**PR 列表**:
- tauri#30: https://github.com/Eulogizethesun/tauri/pull/30
- openharmony-ability#15: https://github.com/Eulogizethesun/openharmony-ability/pull/15
- plugins-workspace#9: https://github.com/Eulogizethesun/plugins-workspace/pull/9

**检视人**: MingyuChen1
**总意见数**: 20 条（去重后 17 项）
**修复状态**: ✅ 17 项全部修复

---

## 一、openharmony-ability#15（10 条意见 → 7 项）

### OHA-1: rx.await 缺少 timeout 保护 🔴
- **来源**: Comment #1, #4（重复）
- **文件**: `crates/ability/src/autostart.rs` L63, L94, L133
- **问题**: `rx.await` 无超时保护，若 ArkTS Promise 永不 resolve，future 永远挂起
- **修复**: 用 `tokio::time::timeout(Duration, rx)` 包裹；`is_enabled` 用 5s，`enable/disable` 用 10s
- **状态**: ✅ 已修复

**修改内容**:

```rust
// 修改前 (L63)
rx.await
    .map_err(|_| Error::from_reason("autostart enable receiver dropped"))?

// 修改后 (L63-66)
timeout(Duration::from_secs(10), rx)
    .await
    .map_err(|_| Error::from_reason("autostart enable timed out"))?
    .map_err(|_| Error::from_reason("autostart enable receiver dropped"))?
```

**依赖添加**: 在文件顶部添加 `use tokio::time::{timeout, Duration};`

---

### OHA-2: unsafe cast 前未验证类型 🔵
- **来源**: Comment #2, #8（重复）
- **文件**: `crates/ability/src/autostart.rs` L147, L153（handle_void_promise, handle_bool_promise）
- **问题**: `unsafe { value.cast::<PromiseRaw>() }` 前未检查 `get_type()`，若 ArkTS 返回非 Promise 会导致 UB
- **修复**: 添加 `value.get_type()? != ValueType::Object` 检查，与 clipboard 模块一致
- **状态**: ✅ 已修复

**修改内容**:

```rust
// handle_void_promise (L151-162)
fn handle_void_promise(
    value: Unknown<'static>,
    tx: Rc<Cell<Option<oneshot::Sender<Result<()>>>>>,
) -> Result<()> {
    // 新增：Validate type before unsafe cast
    if value.get_type()? != ValueType::Object {
        send_once(&tx, Err(Error::from_reason("expected Promise from ArkTS")));
        return Ok(());
    }

    let promise: PromiseRaw<'static, ()> = unsafe { value.cast()? };
    // ...
}

// handle_bool_promise (L178-189) - 相同模式
```

**依赖添加**: 在 napi_ohos 导入中添加 `ValueType`

---

### OHA-3: `let _ =` 静默丢弃 Promise 链错误 🔵
- **来源**: Comment #3, #7（重复）
- **文件**: `crates/ability/src/autostart.rs` L163, L190（handle 函数内部）
- **问题**: `.then()`/`.catch()` 设置失败被 `let _ =` 静默丢弃，调用方得不到错误信息
- **修复**: 改用 `?` 传播错误，函数签名改为返回 `Result<()>`
- **状态**: ✅ 已修复

**修改内容**:

```rust
// 修改前 (L157-168)
let _ = promise
    .then(move |_ctx: CallbackContext<()>| {
        send_once(&tx_then, Ok(()));
        Ok(())
    })
    .and_then(|p| {
        p.catch(move |ctx: CallbackContext<Unknown>| {
            send_once(&tx, Err(ctx.value.into()));
            Ok(())
        })
    });

// 修改后 (L166-175)
promise
    .then(move |_ctx: CallbackContext<()>| {
        send_once(&tx, Ok(()));
        Ok(())
    })?
    .catch(move |ctx: CallbackContext<Unknown>| {
        send_once(&tx_catch, Err(ctx.value.into()));
        Ok(())
    })?;

Ok(())
```

---

### OHA-4: doc comment 应明确 enable/disable 语义 🟡
- **来源**: Comment #5
- **文件**: `crates/ability/src/autostart.rs` L32, L37, L69
- **问题**: enable/disable 语义有歧义——调用方期望"已启用"但实际只打开设置页
- **修复**: 更新 doc comment 说明 OHOS 平台限制
- **状态**: ✅ 已修复

**修改内容**:

```rust
// 修改前 (L32)
/// Autostart manager for OHOS platform.
pub struct AutostartManager;

// 修改后 (L36-40)
/// Autostart manager for OHOS platform.
///
/// Both `enable()` and `disable()` open the **same** system "App launch management"
/// settings page — OHOS does not allow ordinary apps to programmatically toggle
/// autostart. Method names reflect user intent, not guaranteed outcome.
pub struct AutostartManager;

// enable() 方法 (L44-47)
/// Navigate to system autostart settings page.
///
/// Both `enable()` and `disable()` open the **same** page —
/// OHOS does not allow apps to programmatically toggle autostart.
/// Method names reflect user intent, not guaranteed outcome.

// disable() 方法 (L76-79) - 相同模式
```

---

### OHA-5: ArkTS Promise 包装可简化 🔵
- **来源**: Comment #6
- **文件**: `native_ability/src/main/ets/helper/autostart.ets` L23-42
- **问题**: `new Promise<void>(...)` 包装 callback 版 startAbility 可简化
- **修复**: 使用 `await context.startAbility(want)` Promise 版本
- **状态**: ✅ 已修复

**修改内容**:

```typescript
// 修改前 (L23-42)
export const openAutostartSettings = async (context: common.UIAbilityContext): Promise<void> => {
  const want: Want = { /* ... */ };
  return new Promise<void>((resolve, reject) => {
    try {
      context.startAbility(want, (err: BusinessError) => {
        if (err && err.code) {
          hilog.error(/* ... */);
          reject(err);
          return;
        }
        resolve();
      });
    } catch (err) {
      // ...
    }
  });
};

// 修改后 (L23-35)
export const openAutostartSettings = async (context: common.UIAbilityContext): Promise<void> => {
  const want: Want = {
    bundleName: 'com.huawei.hmos.settings',
    abilityName: 'com.huawei.hmos.settings.MainAbility',
    uri: 'pc_app_setup_settings'
  };

  try {
    await context.startAbility(want);
    hilog.info(DOMAIN, 'autostart', 'Navigated to autostart settings page');
  } catch (err) {
    const error = err as BusinessError;
    hilog.error(DOMAIN, 'autostart', 'startAbility failed: code=%{public}d, message=%{public}s',
      error.code, error.message);
    throw error;
  }
};
```

---

### OHA-6: TSFN 创建函数缺少幂等保护 🔵
- **来源**: Comment #9
- **文件**: `crates/ability/src/helper/autostart.rs` L35, L71, L107（三个 create_*_tsfn 函数）
- **问题**: 多次调用 `create_*_tsfn()` 会重复创建 TSFN
- **修复**: 添加 `AtomicBool` 守卫，与 clipboard 模块一致
- **状态**: ✅ 已修复

**修改内容**:

```rust
// 新增导入 (L1)
use std::sync::atomic::{AtomicBool, Ordering};

// 每个 TSFN 添加 static 守卫
static AUTOSTART_ENABLE_INITIALIZED: AtomicBool = AtomicBool::new(false);   // L33
static AUTOSTART_DISABLE_INITIALIZED: AtomicBool = AtomicBool::new(false);  // L69
static AUTOSTART_IS_ENABLED_INITIALIZED: AtomicBool = AtomicBool::new(false); // L105

// create_autostart_enable_tsfn (L37-41)
pub fn create_autostart_enable_tsfn(env: &Env) -> Result<Arc<AutostartEnableTsfn>> {
    if AUTOSTART_ENABLE_INITIALIZED.load(Ordering::Acquire) {
        return get_autostart_enable_tsfn()
            .ok_or_else(|| Error::from_reason("AUTOSTART_ENABLE_TSFN flag set but TSFN is None"));
    }
    // ... 原有逻辑 ...
    AUTOSTART_ENABLE_INITIALIZED.store(true, Ordering::Release);
    Ok(tsfn_arc)
}
```

---

### OHA-7: TSFN 初始化错误被静默丢弃 🔵
- **来源**: Comment #10
- **文件**: `crates/ability/src/render/xcomponent.rs` L47-49
- **问题**: `let _ =` 丢弃 TSFN 创建错误，初始化失败不可追溯
- **修复**: 改用 `if let Err(e)` + `eprintln!` 记录错误
- **状态**: ✅ 已修复

**修改内容**:

```rust
// 修改前 (L47-49)
let _ = create_autostart_enable_tsfn(env);
let _ = create_autostart_disable_tsfn(env);
let _ = create_autostart_is_enabled_tsfn(env);

// 修改后 (L47-55)
if let Err(e) = create_autostart_enable_tsfn(env) {
    eprintln!("create_autostart_enable_tsfn failed: {}", e);
}
if let Err(e) = create_autostart_disable_tsfn(env) {
    eprintln!("create_autostart_disable_tsfn failed: {}", e);
}
if let Err(e) = create_autostart_is_enabled_tsfn(env) {
    eprintln!("create_autostart_is_enabled_tsfn failed: {}", e);
}
```

---

## 二、plugins-workspace#9（3 条意见 → 3 项）

### PW-1: setup() 中 OHOS 分支缺少注释说明 🔵
- **来源**: Comment #1
- **文件**: `plugins/autostart/src/lib.rs` L280
- **问题**: Builder 的 app_name/args 在 OHOS 上被静默忽略，缺少注释
- **修复**: 添加注释说明 OHOS 不需要这些参数
- **状态**: ✅ 已修复

**修改内容**:

```rust
// 修改前 (L278-281)
#[cfg(target_env = "ohos")]
{
    app.manage(AutoLaunchManager(AutostartManager));
}

// 修改后 (L278-284)
#[cfg(target_env = "ohos")]
{
    // OHOS: app_name and args are not needed — autostart is managed by
    // the system settings page via openharmony-ability TSFN bridge
    let _ = self;
    app.manage(AutoLaunchManager(AutostartManager));
}
```

---

### PW-2: OHOS 分支 setup 可能产生 unused field 警告 🟡
- **来源**: Comment #2
- **文件**: `plugins/autostart/src/lib.rs` L280
- **问题**: move 闭包未访问 self.app_name/self.args，可能产生编译警告
- **修复**: 添加 `let _ = self;` 显式消费 Builder（与 PW-1 合并修复）
- **状态**: ✅ 已修复

---

### PW-3: 内层 cfg 中冗余的 not(target_env = "ohos") 🔵
- **来源**: Comment #3
- **文件**: `plugins/autostart/src/lib.rs` L262
- **问题**: 外层已有 `#[cfg(not(target_env = "ohos"))]`，内层 `not(target_env = "ohos")` 冗余
- **修复**: 简化为 `#[cfg(target_os = "linux")]`
- **状态**: ✅ 已修复

**修改内容**:

```rust
// 修改前 (L262)
#[cfg(all(target_os = "linux", not(target_env = "ohos")))]

// 修改后 (L262)
#[cfg(target_os = "linux")]
```

---

## 三、tauri#30（7 条意见 → 7 项）

> 注：Review #1 包含 F1-F6 共 6 个 findings，Review #2 (Round 2) 补充 1 个（等同于 inline comment #1）。
> F1=TAURI-3, F2=TAURI-2, Round 2=TAURI-1, F3=TAURI-4, F4=TAURI-5, F5=TAURI-6, F6=TAURI-7。

### TAURI-1 (F-Round2): tasks.md 缺少跨仓库说明 🟡
- **来源**: Review #2 (Round 2)
- **文件**: `openspec/changes/archive/2026-06-05-p1-autostart/tasks.md`
- **问题**: §1-§5 代码在其他仓库，未在 tasks.md 说明
- **修复**: 在文件头部添加跨仓库说明
- **状态**: ✅ 已修复

**修改内容**:

```markdown
// 新增内容 (文件顶部)
> 注：§1-§5 任务在 openharmony-ability#15 / plugins-workspace#9 仓库各自 PR 中完成，
> 本仓库 PR 仅包含 §6 前端测试任务。

## 1. ArkTS 实现（openharmony-ability）
```

---

### TAURI-2 (F2): design.md 编号重复 🟡
- **来源**: Review #1 F2, inline Comment #2
- **文件**: `openspec/changes/archive/2026-06-05-p1-autostart/design.md` L131
- **问题**: `## 6.` 与 `### 6.` 重复编号，层级不一致
- **修复**: 改为 `### 7. Sync vs Async 桥接`
- **状态**: ✅ 已修复

**修改内容**:

```markdown
// 修改前 (L131)
## 6. Sync vs Async 桥接

// 修改后 (L131)
### 7. Sync vs Async 桥接
```

---

### TAURI-3 (F1): side-effect 测试在 OHOS 上可能打断后续测试 🟡
- **来源**: Review #1 F1, inline Comment #3
- **文件**: `examples/api/src/lib/tests/plugins.ts` L238, L248
- **问题**: enable()/disable() 在 OHOS 上跳转设置页导致 app 进入后台
- **初始方案**: ~~OHOS 平台检测跳过~~ — 会导致 OHOS 上完全不执行测试，无法验证功能
- **修正方案**: 将两个 side-effect 测试移到 side-effect 列表末尾（manual tests 之前）。这样：
  - 测试仍然执行，验证 autostart 功能 ✅
  - app 进入后台时，其他 side-effect 测试已全部跑完 ✅
  - `startAbility` 是非阻塞的（~70ms 即返回），测试本身能通过 ✅
- **状态**: ✅ 已修复

**修改内容**:

```typescript
// 从原位置（L237-270）删除两个 side-effect 测试，移到文件末尾：

// @tauri-apps/plugin-autostart (side-effect tests moved to end — on OHOS,
// enable()/disable() call startAbility which sends app to background;
// placing them last ensures other side-effect tests run first)
// ⚠️ IMPORTANT: Do NOT add new side-effect tests after this section.
// These tests MUST remain at the end of the side-effect list because
// on OHOS they trigger startAbility() which sends the app to background,
// disrupting any subsequent automated test execution.
{
  name: '@tauri-apps/plugin-autostart.enable+disable (no throw)',
  category: 'side-effect',
  async fn() {
    const { enable, disable } = await import('@tauri-apps/plugin-autostart');
    await enable();
    await disable();
  },
},
{
  name: '@tauri-apps/plugin-autostart.enable+isEnabled+disable',
  category: 'side-effect',
  async fn() {
    const { enable, disable, isEnabled } = await import('@tauri-apps/plugin-autostart');
    await enable();
    const afterEnable = await isEnabled();
    assert(typeof afterEnable === 'boolean', 'isEnabled should return boolean after enable()');
    await disable();
    const afterDisable = await isEnabled();
    assert(typeof afterDisable === 'boolean', 'isEnabled should return boolean after disable()');
  },
},
```

---

### TAURI-4 (F3): MacosLauncher::LaunchAgent 在 OHOS 上无意义 🔵
- **来源**: Review #1 F3
- **文件**: `examples/api/src-tauri/src/lib.rs` L95
- **问题**: OHOS 分支调用 `init(MacosLauncher::LaunchAgent, None)` 但 `MacosLauncher` 参数在 OHOS 上无意义
- **修复**: 添加注释说明该参数在 OHOS 上被忽略
- **状态**: ✅ 已修复

**修改内容**:

```rust
// 修改前 (L94-98)
    .plugin(tauri_plugin_clipboard_manager::init())
    .plugin(tauri_plugin_autostart::init(
      tauri_plugin_autostart::MacosLauncher::LaunchAgent,
      None,
    ));

// 修改后 (L94-99)
    .plugin(tauri_plugin_clipboard_manager::init())
    // MacosLauncher::LaunchAgent is ignored on OHOS (macOS-specific parameter)
    .plugin(tauri_plugin_autostart::init(
      tauri_plugin_autostart::MacosLauncher::LaunchAgent,
      None,
    ));
```

---

### TAURI-5 (F4): autostart-plan.md 位于 openspec 根目录 🔵
- **来源**: Review #1 F4
- **文件**: `openspec/autostart-plan.md`
- **问题**: 与 openspec 约定不一致，plan 文件应移入归档目录
- **修复**: `git mv` 到 `openspec/changes/archive/2026-06-05-p1-autostart/plan.md`
- **状态**: ✅ 已修复

**修改内容**:

```bash
git mv openspec/autostart-plan.md openspec/changes/archive/2026-06-05-p1-autostart/plan.md
```

---

### TAURI-6 (F5): ohos-autostart spec.md Purpose 字段为 TBD 🔵
- **来源**: Review #1 F5
- **文件**: `openspec/specs/ohos-autostart/spec.md` L3-4
- **问题**: Purpose 字段为 `TBD - created by archiving change p1-autostart`
- **修复**: 填写实际 purpose 描述
- **状态**: ✅ 已修复

**修改内容**:

```markdown
// 修改前 (L3-4)
## Purpose
TBD - created by archiving change p1-autostart. Update Purpose after archive.

// 修改后 (L3-4)
## Purpose
OHOS platform adaptation for `@tauri-apps/plugin-autostart`. Defines the behavior of
`enable()`, `disable()`, and `isEnabled()` on OpenHarmony where ordinary apps cannot
programmatically toggle autostart — instead, `enable()`/`disable()` navigate to the
system settings page for manual operation, and `isEnabled()` queries `autoStartupManager`
(API 21+) with a forced fallback value of `false` on older versions.
```

---

### TAURI-7 (F6): openspec/config.yaml 仅包含空模板 ℹ️
- **来源**: Review #1 F6
- **文件**: `openspec/config.yaml`
- **问题**: 仅包含空模板，无实际配置
- **分析**: 此文件是 `openspec init` 时自动生成的工具配置文件，其他归档任务都没有此文件，不应提交到 PR
- **修复**: `git rm --cached openspec/config.yaml`，从 PR 中移除，保留为本地未跟踪文件
- **状态**: ✅ 已修复

---

## 修复统计

| 仓库 | 总数 | 🔴 阻塞 | 🔵 建议 | 🟡 质量 | ℹ️ 信息 | 已修复 |
|------|------|---------|---------|---------|---------|--------|
| openharmony-ability | 7 | 1 | 5 | 1 | 0 | ✅ 7 |
| plugins-workspace | 3 | 0 | 2 | 1 | 0 | ✅ 3 |
| tauri | 7 | 0 | 4 | 2 | 1 | ✅ 7 |
| **合计** | **17** | **1** | **11** | **4** | **1** | **✅ 17** |

---

## 提交策略

按用户要求，所有修改通过 `git commit --amend` 合并到原 PR commit，不创建新 commit：

1. **openharmony-ability**: `git commit --amend` 到原 commit `f3bcf4e` (feat: add autostart TSFN bridge)
2. **plugins-workspace**: `git commit --amend` 到原 commit `0287be0` (feat: add OHOS platform support)
3. **tauri**: `git commit --amend` 到原 commit `fd557b4` (feat: OHOS adaptation)

三个仓库 amend 后分别 `git push --force-with-lease` 更新 PR。

等待用户 review 确认后执行 amend + force-push。

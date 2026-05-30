# Tauri RunEvent 架构文档

本文档描述 Tauri 应用中 `RunEvent` 事件的触发流程和架构设计，特别关注 OpenHarmony 平台的实现。

## 1. 事件触发流程

### 1.1 Ready 事件

**触发时机**: 应用启动时

**触发路径**:
```
tao EventLoop 启动
  ↓
Event::NewEvents(StartCause::Init)
  ↓
tauri-runtime-wry 转换
  ↓
RunEvent::Ready
```

**状态**: ✅ 已实现，正常工作

---

### 1.2 Resumed 事件

**触发时机**: 应用从后台恢复到前台

**触发路径**:
```
OHOS MainEvent::Resume
  ↓
tao Event::Resumed
  ↓
RunEvent::Resumed
```

**状态**: ✅ 已实现，正常工作

---

### 1.3 MainEventsCleared 事件

**触发时机**: 每个事件循环周期，所有事件处理完毕后

**触发路径**:
```
tao EventLoop 处理完一轮事件
  ↓
Event::MainEventsCleared
  ↓
RunEvent::MainEventsCleared
```

**状态**: ✅ 已实现，正常工作（频繁触发）

---

### 1.4 WindowEvent::CloseRequested 事件

**触发时机**: 用户点击窗口关闭按钮

**触发路径**:

#### 主窗口 (Main Window):
```
OHOS MainEvent::WindowDestroy
  ↓
tao 转发为 TaoWindowEvent::CloseRequested
  ↓
Event::WindowEvent { CloseRequested }
  ↓
RunEvent::WindowEvent::CloseRequested
```

#### 子窗口 (Sub-window):
```
应用调用 WindowMessage::Destroy
  ↓
on_close_requested()
  ↓
Event::WindowEvent { CloseRequested }
  ↓
RunEvent::WindowEvent::CloseRequested
```

**状态**: ✅ 已实现，正常工作

---

### 1.5 WindowEvent::Destroyed 事件

**触发时机**: 窗口真正被销毁

**触发路径**:

#### 主窗口 (Main Window):
```
OHOS MainEvent::WindowDestroy
  ↓
tao 转发为 TaoWindowEvent::Destroyed
  ↓
Event::WindowEvent { Destroyed }
  ↓
RunEvent::WindowEvent::Destroyed
```

#### 子窗口 (Sub-window):
```
OHOS 系统触发 TaoWindowEvent::Destroyed
  ↓
Event::WindowEvent { Destroyed }
  ↓
RunEvent::WindowEvent::Destroyed
```

**状态**: ⚠️ 部分实现
- 主窗口: ✅ 正常
- 子窗口: ❌ 通过 `WindowMessage::Destroy` 关闭时不会触发（见 TODO #2）

---

### 1.6 ExitRequested 事件

**触发时机**: 应用准备退出时，给用户代码一个拦截退出的机会

**触发路径**:

#### 路径 A - 所有窗口关闭:
```
TaoWindowEvent::Destroyed
  ↓
从 windows store 移除窗口
  ↓
检查 windows.is_empty()
  ↓
触发 RunEvent::ExitRequested
```

#### 路径 B - 事件循环销毁 (当前问题):
```
OHOS MainEvent::Destroy
  ↓
tao Event::LoopDestroyed
  ↓
tauri-runtime-wry 尝试触发 RunEvent::ExitRequested
  ↓
用户代码调用 api.prevent_exit()
  ↓
阻止 RunEvent::Exit
```

**状态**: ⚠️ 部分实现（见 TODO #1）
- 路径 A: ✅ 理论上可以工作（但 OHOS 上由于子窗口问题难以触发）
- 路径 B: ❌ 被 `api.prevent_exit()` 阻止

---

### 1.7 Exit 事件

**触发时机**: 应用真正退出

**触发路径**:
```
tao Event::LoopDestroyed
  ↓
RunEvent::Exit
```

**状态**: ⚠️ 当前依赖 ExitRequested 不被阻止（见 TODO #1）

---

## 2. 未实现问题 (TODO)

### TODO #1: ExitRequested 和 Exit 的触发问题

**问题描述**:
在 OHOS 平台上，`ExitRequested` 事件应该使用 `UIAbility.onPrepareToTerminate` 生命周期回调来实现，而不是当前的 `Event::LoopDestroyed` 机制。

**当前实现的问题**:
1. `ExitRequested` 事件可以触发
2. 但应用的 `api.prevent_exit()` 会阻止退出
3. 导致 `RunEvent::Exit` 永远不会触发

**根因分析**:
```rust
// lib.rs 中的当前实现
RunEvent::ExitRequested { api, code, .. } if code.is_none() => {
  api.prevent_exit();  // ← 这里阻止了退出
}
```

当 `Event::LoopDestroyed` 触发时，`code` 为 `None`，所以会调用 `prevent_exit()`。

**正确的解决方案**:
使用 OpenHarmony 的 `UIAbility.onPrepareToTerminate` 生命周期回调：

```typescript
// NativeAbility.ets
onPrepareToTerminate(): void {
  // 通知 Rust 层触发 ExitRequested
  this.forEachLifecycle((lifecycle) => {
    lifecycle.windowStageEventCallback.onPrepareToTerminate();
  });
}
```

**实现步骤**:
1. 在 `openharmony-ability` 中添加 `onPrepareToTerminate` 回调
2. 在 `tao` 的 OHOS 适配层添加对应的 `MainEvent::PrepareToTerminate`
3. 在 `tauri-runtime-wry` 中处理该事件，触发 `RunEvent::ExitRequested`
4. 需要设计机制区分"可以阻止的退出"和"不可阻止的退出"

**参考资料**:
- [UIAbility 生命周期](https://developer.huawei.com/consumer/cn/doc/harmonyos-guides-V5/uiability-lifecycle-V5)
- `onPrepareToTerminate`: Ability 即将被销毁前的回调，可以在此处执行清理工作

**优先级**: 🔴 高

**当前临时方案**:
在 `Event::LoopDestroyed` 时直接触发 `RunEvent::Exit`，跳过 `ExitRequested`（见代码中的 TODO 注释）

---

### TODO #2: 子窗口 Destroyed 事件缺失

**问题描述**:
通过 `WindowMessage::Destroy` 关闭子窗口时：
1. `WindowEvent::CloseRequested` 会触发
2. 但 `WindowEvent::Destroyed` 不会触发

**根因分析**:
- 子窗口关闭走 `WindowMessage::Destroy` → `on_close_requested` 路径
- 这个路径只触发 `CloseRequested`，不会触发 `Destroyed`
- 主窗口有 OHOS 的 `MainEvent::WindowDestroy` 额外触发 `Destroyed`
- 子窗口没有对应的 OHOS 事件

**影响**:
- 子窗口关闭后，WindowsStore 中的条目只是设置 `inner = None`，没有被移除
- 可能导致内存泄漏或状态不一致

**可能的解决方案**:
1. 在 `on_close_requested` 完成后，手动触发 `Destroyed` 事件
2. 或者在 `WindowMessage::Destroy` 处理流程中添加 `Destroyed` 触发逻辑
3. 需要确保事件触发顺序正确：`CloseRequested` → 用户处理 → `Destroyed`

**优先级**: 🟡 中

---

### TODO #3: 窗口清理逻辑不完整

**问题描述**:
`on_window_close` 函数只设置 `inner = None`，没有从 `WindowsStore` 中移除条目。

**当前实现**:
```rust
fn on_window_close(window_id: WindowId, windows: Arc<WindowsStore>) {
  if let Some(window_wrapper) = windows.0.borrow_mut().get_mut(&window_id) {
    window_wrapper.inner = None;  // 只清空 inner，不移除条目
    #[cfg(windows)]
    window_wrapper.surface.take();
  }
}
```

**问题**:
- 窗口条目仍然存在于 `WindowsStore` 中
- `windows.is_empty()` 永远返回 `false`（除非通过 `TaoWindowEvent::Destroyed` 移除）
- 可能导致 `ExitRequested` 的路径 A 永远无法触发

**可能的解决方案**:
1. 在 `on_window_close` 中直接移除窗口条目
2. 或者在合适的时机（如 `Destroyed` 事件后）清理 `inner = None` 的条目
3. 需要与 TODO #2 一起考虑

**优先级**: 🟡 中

---

### TODO #4: Opened 事件未适配

**问题描述**:
`RunEvent::Opened` 事件用于处理外部 URL/深链接唤起应用（如 `myapp://some/path`），当前在 OHOS 上未真正调通。

**当前实现**:
```rust
// tao OHOS 适配层
MainEvent::NewWant { uri } => {
  if let Some(url) = url::Url::parse(&uri).ok() {
    h(event::Event::Opened { urls: vec![url] });
  }
}
```

当前通过 `MainEvent::NewWant` 转发，但这只处理了 `onNewWant` 回调，没有完整适配 HarmonyOS 的 URL Schema 机制。

**正确的解决方案**:
使用 HarmonyOS 的 URL Schema 进行完整适配：

1. 在 `module.json5` 中配置 `uri` 和 `skills`，声明应用支持的 URL Scheme
2. 在 `NativeAbility.onCreate` 和 `onNewWant` 中解析 `want.uri`，提取 URL
3. 将解析到的 URL 通过 `MainEvent::NewWant` 转发给 Rust 层
4. 需要处理冷启动（`onCreate`）和热启动（`onNewWant`）两种场景

**配置示例**:
```json5
// module.json5
{
  "module": {
    "abilities": [{
      "skills": [{
        "entities": ["entity.system.home"],
        "actions": ["action.system.home"],
        "uris": [{
          "scheme": "myapp",
          "host": "*"
        }]
      }]
    }]
  }
}
```

**参考资料**:
- [HarmonyOS Deep Link](https://developer.huawei.com/consumer/cn/doc/harmonyos-guides-V5/deep-linking-V5)
- [UIAbility onNewWant](https://developer.huawei.com/consumer/cn/doc/harmonyos-references-V5/js-apis-app-ability-uiability-V5#onnewwant)

**优先级**: 🟡 中

---

### TODO #5: WindowEvent::CloseRequested 拦截能力未验证

**问题描述**:
`RunEvent::WindowEvent::CloseRequested` 事件能够正常接收，但调用 `api.prevent_close()` 是否真正阻止窗口关闭还未验证。

**当前实现**:
```rust
// examples/api/src-tauri/src/lib.rs
RunEvent::WindowEvent {
  event: tauri::WindowEvent::CloseRequested { api, .. },
  label,
  ..
} => {
  log::info!("closing window...");
  api.prevent_close();  // ← 是否真正阻止了窗口关闭？
  _app_handle.get_webview_window(label).unwrap().destroy().unwrap();
}
```

**待验证**:
1. `prevent_close()` 在 OHOS 平台上是否真正阻止系统关闭窗口的行为
2. 当前代码中 `prevent_close()` 后紧接着调用了 `destroy()`，实际上窗口还是被关闭了
3. 如果不调用 `destroy()`，仅 `prevent_close()` 是否能保持窗口打开
4. OHOS 的 `MainEvent::WindowDestroy` 是否可以通过某种方式取消

**验证方法**:
```rust
RunEvent::WindowEvent {
  event: tauri::WindowEvent::CloseRequested { api, .. },
  label,
  ..
} => {
  api.prevent_close();
  // 不调用 destroy()，观察窗口是否保持打开
  // 如果窗口仍然关闭，说明 prevent_close() 在 OHOS 上无效
}
```

**优先级**: 🟡 中

---

## 3. 调试方法

### 3.1 查看 RunEvent 日志

```bash
# 实时查看
hdc shell hilog -x 2>/dev/null | grep -E "(RunEvent|wry)"

# 查看所有应用日志
hdc shell "hilog -x | grep tauritest"
```

### 3.2 关键日志标记

- `[RunEvent] Ready` - 应用启动
- `[RunEvent] Resumed` - 应用恢复
- `[RunEvent] MainEventsCleared` - 事件循环周期
- `[RunEvent] WindowEvent::CloseRequested` - 窗口关闭请求
- `[RunEvent] WindowEvent::Destroyed` - 窗口销毁
- `[RunEvent] ExitRequested` - 退出请求
- `[RunEvent] Exit` - 应用退出
- `[wry] Event::LoopDestroyed received` - 事件循环销毁

### 3.3 触发测试场景

1. **启动应用**: 查看 `Ready`、`Resumed`、`MainEventsCleared`
2. **关闭子窗口**: 查看 `CloseRequested`（应该有 `Destroyed` 但缺失）
3. **关闭主窗口**: 查看 `CloseRequested`、`Destroyed`、`ExitRequested`、`Exit`

---

## 4. 架构改进建议

### 4.1 事件触发一致性

目前主窗口和子窗口的事件触发路径不一致：
- 主窗口依赖 OHOS 系统事件
- 子窗口依赖应用层消息

建议统一事件触发机制，确保所有窗口类型的行为一致。

### 4.2 退出流程优化

当前的退出流程存在歧义：
- `ExitRequested` 何时应该被阻止？
- 系统关闭和用户主动关闭应该如何区分？

建议引入更细粒度的退出类型：
```rust
enum ExitReason {
  UserClose,      // 用户主动关闭，可以阻止
  SystemShutdown, // 系统关闭，不可阻止
  AllWindowsClosed, // 所有窗口关闭，可以阻止
}
```

### 4.3 窗口生命周期管理

建议实现更完善的窗口生命周期状态机：
```
Creating → Ready → Closing → Closed → Destroyed
```

每个状态转换都应该有明确的事件触发。

---

## 5. 相关代码位置

- **事件定义**: `crates/tauri-runtime-wry/src/lib.rs`
- **窗口管理**: `crates/tauri-runtime-wry/src/window.rs`
- **OHOS 适配**: `crates/tao/src/platform_impl/ohos/mod.rs`
- **示例应用**: `examples/api/src-tauri/src/lib.rs`

---

**文档版本**: v1.0  
**最后更新**: 2026-05-30  
**作者**: Tauri Team

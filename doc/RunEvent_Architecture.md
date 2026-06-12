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

**状态**: 🚫 跨平台遗留问题（死代码）

**说明**:
- Tauri 强制 `ControlFlow = Wait`，`StartCause::Poll` 永远不触发
- `StartCause::Poll → RunEvent::Resumed` 映射是死代码
- `Event::Resumed` 被 `_ => ()` 丢弃
- 这是 winit 0.28 生命周期事件演进后 Tauri 未适配的历史遗留问题，所有平台（桌面、iOS、Android、OHOS）都受影响
- 修复需要统一评估对桌面平台的影响，不宜在 OHOS 移植中单独处理

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

#### 路径 B - 事件循环销毁 (已修复):
```
OHOS MainEvent::Destroy
  ↓
tao Event::LoopDestroyed
  ↓
tauri-runtime-wry 检查 ExitState 标志
  ↓ (如果未通过路径 A 发送过)
触发 RunEvent::ExitRequested
  ↓
触发 RunEvent::Exit
```

**状态**: ✅ 已实现，正常工作
- 路径 A: ✅ 正常工作
- 路径 B: ✅ 已修复 — `LoopDestroyed` handler 先触发 `ExitRequested` 再触发 `Exit`，使用 `ExitState(AtomicBool)` 防止重复触发
- 注意：OHOS 上 `LoopDestroyed` 时系统已开始销毁，`prevent_exit()` 可能无法真正阻止退出，但用户代码可执行清理逻辑

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

### ✅ 已解决 #1: ExitRequested 和 Exit 的触发问题

**解决方案**: 修改 `LoopDestroyed` handler，在发送 `Exit` 前先发送 `ExitRequested`。使用 `ExitState(AtomicBool)` 防止与窗口关闭路径重复触发。

**实现细节**:
- 新增 `ExitState` 结构体存储 `AtomicBool` 标志
- `TaoWindowEvent::Destroyed` 路径触发 `ExitRequested` 时设置标志为 `true`
- `LoopDestroyed` 路径检查标志，仅在 `false` 时发送 `ExitRequested`
- `prevent_exit()` 在 OHOS 上可能无法真正阻止退出（系统已开始销毁），但用户代码可执行清理

**状态**: ✅ 已实现

**后续增强方向**:
- 验证 OHOS `onPrepareToTerminate` 回调（需 `persist.sys.prepare_terminate = true`）
- 如验证通过，可实现真正可阻止的退出拦截

---

### ✅ 已解决 #2: 子窗口 Destroyed 事件缺失

**解决方案**: 重构 `on_window_close` 函数，使其执行完整清理逻辑（移除条目 + 发送 `Destroyed` 事件 + 检查空 → 触发 `ExitRequested`）。

**实现细节**:
- `on_window_close` 函数签名添加 `callback` 和 `exit_state` 参数
- 函数内部调用 `windows.0.borrow_mut().remove(&window_id)` 移除条目
- 移除成功后发送 `RunEvent::WindowEvent { event: WindowEvent::Destroyed }`
- 检查 `windows.0.borrow().is_empty()`，如果为空则触发 `ExitRequested`
- `WindowMessage::Destroy` 处理器改为调用 `on_close_requested`（先发送 `CloseRequested`，再调用 `on_window_close`）
- `TaoWindowEvent::Destroyed` 处理器改为调用 `on_window_close`（统一清理路径）

**状态**: ✅ 已实现

---

### ✅ 已解决 #3: 窗口清理逻辑不完整

**解决方案**: 与 TODO #2 一起解决。重构后的 `on_window_close` 函数直接从 `WindowsStore` 移除条目，而不是仅设置 `inner = None`。

**实现细节**:
- `on_window_close` 使用 `windows.0.borrow_mut().remove(&window_id)` 移除条目
- 移除操作是幂等的（多次调用不会重复移除）
- `windows.is_empty()` 现在能正确反映剩余窗口数量
- `ExitRequested` 路径 A（所有窗口关闭）能正确触发

**状态**: ✅ 已实现

---

### ✅ 已解决 #4: Opened 事件适配

**解决方案**: 通过 `MainEvent::NewWant` → `Event::Opened` 路径实现深链接支持。

**实现细节**:
- openharmony-ability: `Event::NewWant { uri: String }` 变体 + ArkTS `onNewWant` handler
- tao OHOS: `MainEvent::NewWant { uri }` → `url::Url::parse(&uri)` → `Event::Opened { urls }`
- tauri-runtime: `Opened` cfg 扩展包含 `target_env = "ohos"`
- tauri-runtime-wry: `Event::Opened` handler cfg 扩展包含 `target_env = "ohos"`

**状态**: ✅ 已实现

---

### ⚠️ 待验证 #5: WindowEvent::CloseRequested 拦截能力

**问题描述**:
`RunEvent::WindowEvent::CloseRequested` 事件能够正常接收，但调用 `api.prevent_close()` 是否真正阻止窗口关闭还未在设备上验证。

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

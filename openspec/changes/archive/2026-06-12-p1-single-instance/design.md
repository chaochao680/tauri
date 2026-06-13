## Context

OHOS 默认 `launchType: "singleton"`，再次启动时调用 `onNewWant(want)`。当前事件链已打通 `onNewWant → RunEvent::Opened`，但仅传递 `want.uri`。`want.parameters`（类型 `{[key: string]: Object}`）在 ArkTS 层被丢弃。

single-instance 插件需要这些参数来通知首实例"第二次启动携带了什么数据"。

**当前事件链**：
```
ArkTS: onNewWant(want) → 仅提取 want.uri
  → NAPI: on_new_want(uri) → Event::NewWant { uri }
    → tao: Event::Opened { urls: [Url::parse(uri)] }
      → tauri: RunEvent::Opened { urls }
```

**约束**：
- 线程模型：禁止阻塞 recv，必须用 TSFN NonBlocking
- NAPI：callee_handled::<false>()，snake_case → camelCase
- cfg 隔离：所有修改在 cfg(target_env = "ohos") 下

## Goals / Non-Goals

**Goals:**
- 将 `want.parameters` 从 ArkTS 层传递到 Rust 层
- 提供 Rust 侧可读取的 getter 函数
- 保持与现有 `Event::Opened` 事件链的兼容性

**Non-Goals:**
- 不修改 tao 或 tauri 的事件链（避免跨平台影响）
- 不实现 single-instance 插件本身（Phase 2 负责）
- 不处理 `onCreate` 中的初始 want.parameters（仅 onNewWant）

## Decisions

### D1: 在 openharmony-ability 内存储参数，不扩展事件链

**选择**：在 `openharmony-ability` Rust 侧用 `static Mutex` 存储最新 `want.parameters`，暴露 getter 函数。

**备选方案**：
- A) 扩展 `Event::NewWant` + tao `Event::Opened` + `RunEvent` 全链传递 → 需要修改 3 个仓库的事件枚举，且 `Event::Opened` 是跨平台的，添加 OHOS 字段需要 cfg 污染
- B) 新增 OHOS 专用 `RunEvent::OhosNewWant` 变体 → 仍需修改 tauri 核心，增加复杂度

**理由**：
- 只需改 openharmony-ability + single-instance，最小化影响范围
- getter 模式简洁，插件按需读取
- 符合 Direct Call 适配模式

### D2: NAPI 桥接扩展 — on_new_want 改为接收 Object 参数

**选择**：将 `on_new_want` 从接收单个 `String`（uri）改为接收 `Object`（含 uri + parametersJson 字段）。这与 `avoid_area_change`、`window_size_change` 等现有回调的 Object 传参模式一致。

**Rust NAPI struct 更新**：
```rust
// lifecycle.rs - WindowStageEventCallback
pub on_new_want: Function<'a, Object<'a>, ()>,  // 原: Function<'a, String, ()>
```

**闭包实现**：
```rust
let on_new_want = env.create_function_from_closure("on_new_want", move |ctx| {
    let data = ctx.first_arg::<Object>()?;
    let uri = data.get_named_property::<String>("uri")?;
    let parameters_json = data.get_named_property::<String>("parametersJson")?;
    // 存储参数到全局状态
    store_want_parameters(&parameters_json);
    // 触发事件（保持现有流程，Event 不修改）
    if let Some(ref mut h) = *on_new_want_app.event_loop.borrow_mut() {
        h(Event::NewWant { uri })
    }
    Ok(())
})?;
```

**ArkTS 侧**：
```typescript
onNewWant(want: Want, launchParam: AbilityConstant.LaunchParam): void {
    const uri = want.uri ?? '';
    const parametersJson = JSON.stringify(want.parameters ?? {});
    this.forEachLifecycle((lifecycle) => 
        lifecycle.windowStageEventCallback.onNewWant({ uri, parametersJson })
    );
}
```

### D3: 参数存储 — static Mutex<String>

**选择**：`static Mutex<String>` 存储最新 parameters JSON 字符串。

```rust
static WANT_PARAMETERS: Mutex<String> = Mutex::new(String::new());

pub(crate) fn store_want_parameters(json: &str) {
    if let Ok(mut params) = WANT_PARAMETERS.lock() {
        *params = json.to_string();
    }
}

pub fn take_want_parameters() -> String {
    WANT_PARAMETERS.lock()
        .map(|mut p| std::mem::take(&mut *p))
        .unwrap_or_default()
}
```

**用 `take` 语义**（读取后清空）：防止重复处理同一组参数。

**竞态风险**：极短时间内连续两次 `onNewWant` 可能覆盖参数。但 OHOS singleton 模式下这种情况极其罕见，且 single-instance 场景不需要排队处理。

### D4: TypeScript 接口扩展

**选择**：`WindowStageEventCallback.onNewWant` 从 `(uri: string) => void` 改为 `(data: Record<string, string>) => void`。

```typescript
// type.ets
export interface WindowStageEventCallback {
    onNewWant: (data: Record<string, string>) => void;
    // ... other callbacks unchanged
}
```

传递对象而非多参数，与 `onWindowSizeChange(arg: object)` / `onAvoidAreaChange(arg: object)` 模式一致。

### D5: Event::NewWant 不修改

**选择**：保持 `Event::NewWant { uri: String }` 不变。参数通过 Mutex getter 读取，不经事件链传递。

**理由**：
- 避免修改 event.rs → tao → tauri-runtime-wry 整条事件链
- `Event::Opened { urls }` 是跨平台的，添加 OHOS 字段需要 cfg 污染
- Mutex getter 模式足够满足 single-instance 插件需求

### D6: 参数映射到 single-instance callback

**选择**：
- `args: Vec<String>` = `[uri, parameters_json]`
- `cwd: String` = `""`（OHOS 无传统 cwd 概念）

这与 Windows（cwd + argv）、macOS（cwd + argv）的语义对齐：第一个元素是来源标识，第二个是携带数据。

## Risks / Trade-offs

- **[竞态] 连续 onNewWant 覆盖参数** → 使用 take 语义减少影响；singleton 模式下极端罕见
- **[兼容] onNewWant 回调签名变更** → ArkTS 侧需确保所有 lifecycle handler 更新；旧版 HAR 不兼容新 NAPI 签名，需同步重建
- **[限制] 参数为 JSON 字符串** → want.parameters 中的复杂对象（如 FD 文件描述符）不可序列化，JSON.stringify 会跳过或转为 null。single-instance 场景不需要 FD，可接受

## Context

OHOS 默认 `launchType: "singleton"`，OS 层面阻止创建第二个 Ability 实例。再次启动时 `onNewWant` 触发 → `RunEvent::Opened { urls }` 到达插件。Phase 1 已将 `want.parameters` 存储到 `openharmony_ability::take_want_parameters()`。

macOS 实现通过 Unix socket 监听 → 异步 spawn 读取 → 触发 callback。OHOS 不需要这些：OS 已处理单实例，插件只需 hook 事件。

## Goals / Non-Goals

**Goals:**
- 实现 OHOS `platform_impl/ohos.rs`，提供 `init()` 和 `destroy()` 函数
- Hook `RunEvent::Opened` 事件，调用 callback 并传递参数
- 首次启动时不触发 callback（仅再次启动时触发）

**Non-Goals:**
- 不实现进程互斥（OS 已处理）
- 不实现 deep-link 集成（可选 feature，后续按需添加）

## Decisions

### D1: init() 不创建任何资源，仅注册 on_event handler

**选择**：`setup()` 为空（OHOS 不需要 socket/mutex/D-Bus）。所有逻辑在 `on_event` 中。

**理由**：OHOS singleton 由 OS 强制执行，插件只需响应事件。

### D2: 在 RunEvent::Opened 中触发 callback

**选择**：匹配 `RunEvent::Opened { urls }` 事件，调用 `take_want_parameters()` 获取参数，构建 args 并触发 callback。

```rust
.on_event(|app, event| {
    #[cfg(any(target_os = "macos", target_os = "ios", target_os = "android", target_env = "ohos"))]
    if let RunEvent::Opened { urls } = event {
        let params_json = openharmony_ability::take_want_parameters();
        let uri = urls.first().map(|u| u.to_string()).unwrap_or_default();
        let args = if params_json.is_empty() {
            vec![uri]
        } else {
            vec![uri, params_json]
        };
        cb(app.app_handle(), args, String::new());
    }
})
```

**首次启动**：`onNewWant` 不会被调用（首次走 `onCreate`），所以 `RunEvent::Opened` 不触发，callback 不执行 — 正确行为。

**再次启动**：`onNewWant` → `Event::NewWant` → tao `Event::Opened` → `RunEvent::Opened` → callback 触发 — 正确行为。

### D3: callback 参数映射

**选择**：
- `args: Vec<String>` = `[uri, parameters_json]`（如果 parameters 为空则仅 `[uri]`）
- `cwd: String` = `""`（OHOS 无传统 cwd）

**对齐**：macOS 传 `[cwd, arg1, arg2, ...]`。OHOS 传 `[uri, params_json]`。语义等价：来源标识 + 携带数据。

### D4: destroy() 为空

**选择**：`destroy()` 是 no-op。无需清理 socket/mutex/D-Bus。

### D5: cfg 隔离

**选择**：
- `lib.rs` 中添加 `#[cfg(target_env = "ohos")]` 指向 `ohos.rs`
- OHOS 的 `target_os` 是 `"linux"`，所以不能用 `target_os = "ohos"`
- Cargo.toml 中 `openharmony-ability` 用 `[target.'cfg(target_env = "ohos")'.dependencies]`

### D6: on_event 中 RunEvent::Opened 的 cfg gate

**选择**：`RunEvent::Opened` 在 tauri 中已 cfg gate 为 `#[cfg(any(macos, ios, android, target_env = "ohos"))]`。OHOS platform_impl 中匹配此事件时需要相同的 cfg gate。

## Risks / Trade-offs

- **[首次启动误触发]** 已分析：首次启动走 `onCreate` 不走 `onNewWant`，`RunEvent::Opened` 不触发 — 无风险
- **[callback 线程安全]** `SingleInstanceCallback` 要求 `Send + Sync`。`on_event` 在事件循环线程调用，callback 需要同步安全 — 与 macOS 一致
- **[deep-link feature]** 当前 OHOS 实现不支持 `deep-link` feature。`cfg(feature = "deep-link")` 代码在 OHOS 上仍会编译，但 `handle_cli_arguments` 需要额外适配 — 留作后续

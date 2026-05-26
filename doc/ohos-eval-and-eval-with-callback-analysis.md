# eval 与 eval_with_callback 实现分析文档

## 1. 概述

`eval` 和 `eval_with_callback` 是 Tauri WebView 的两个核心 JavaScript 执行接口：
- **eval**: 从 Rust 端执行 JS 代码，不接收返回值（fire-and-forget）
- **eval_with_callback**: 从 Rust 端执行 JS 代码，并通过回调接收 JS 返回值

## 2. 调用链路分析

### 2.1 eval 调用链

```
Tauri App (Rust)
  → Webview.eval(js)
    → webview.dispatcher.eval_script(js)
      → wry → WebView::evaluate_script(js)
        → openharmony-ability WebView::evaluate_script(js)
          → evaluate_script_with_callback(js, None)  // callback=None
            → napi: JsHelper.runJavaScript(code, nullCallback)
              → ArkTS: controller.runJavaScript(code)
                → OpenHarmony WebviewController.runJavaScript(code)
```

**关键实现** (`openharmony-ability/crates/ability/src/helper/webview.rs:193-195`):
```rust
pub fn evaluate_script(&self, js: &str) -> Result<()> {
    self.evaluate_script_with_callback(js, None)  // 内部复用 eval_with_callback
}
```

`eval` 实际上是 `eval_with_callback` 的简化版本，callback=None 时跳过回调创建。

### 2.2 eval_with_callback 调用链

```
Tauri App (Rust)
  → Webview.eval_with_callback(js, callback)
    → webview.dispatcher.eval_script_with_callback(js, callback)
      → wry → WebView::evaluate_script_with_callback(js, callback)
        → openharmony-ability WebView::evaluate_script_with_callback(js, callback)
          → napi:
            1. env.create_function_from_closure("evaluate_js_callback", callback)
            2. JsHelper.runJavaScript(code, napiCallback)
              → ArkTS: controller.runJavaScript(code).then(napiCallback)
                → OpenHarmony WebviewController.runJavaScript(code)
```

**关键实现** (`openharmony-ability/crates/ability/src/helper/webview.rs:197-227`):
```rust
pub fn evaluate_script_with_callback(
    &self, js: &str,
    callback: Option<Box<dyn Fn(String) + Send + 'static>>,
) -> Result<()> {
    // 获取 ArkTS runJavaScript 函数
    let evaluate_js_js_function = self.inner.get_value(env)?
        .get_named_property::<Function<...>>("runJavaScript")?;

    // 创建 napi 回调包装器
    let cb = env.create_function_from_closure("evaluate_js_callback", move |ctx| {
        let ret = ctx.try_get::<String>(0)?;  // 0-based 索引!
        if let Some(cb) = callback.as_ref() {
            cb(ret);
        }
        Ok(())
    })?;

    // 调用 ArkTS runJavaScript(code, callback)
    evaluate_js_js_function.call((js.to_string(), cb).into())?;
}
```

### 2.3 ArkTS 端实现

**DefaultWebview.ets (`buildJsHelper` 函数)**:
```typescript
const runJavaScript = (code: string, cb: (result?: string) => void) => {
    controller.runJavaScript(code).then((ret) => {
        cb(ret);  // ret 是 JS 表达式的返回值字符串
    });
};
```

OpenHarmony `WebviewController.runJavaScript(code)` 返回 Promise，resolve 值为 JS 表达式结果的字符串表示。

## 3. 测试用例

### 3.1 已有测试 - eval (仅验证副作用)

**位置**: `examples/api/src/lib/tests/core.ts`

```typescript
{
    name: 'app_handle.get_webview_window (test_eval)',
    category: 'auto',
    async fn() {
        const originalTitle = document.title;
        await invoke('test_eval');
        await new Promise((r) => setTimeout(r, 100));
        assert(document.title.includes('Eval Success'));
        document.title = originalTitle;
    },
},
```

**局限**: 仅验证 eval 的副作用（标题变更），不验证返回值，且不测试 `eval_with_callback`。

### 3.2 新增测试 - eval_with_callback (验证返回值)

**Rust 命令** (`examples/api/src-tauri/src/cmd.rs`):
```rust
#[command]
pub fn test_eval_with_callback<R: tauri::Runtime>(app: tauri::AppHandle<R>) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        let app_clone = app.clone();
        window.eval_with_callback(
            r#"(function() {
                return JSON.stringify({
                    arithmetic: 1+2,
                    stringLen: "hello".length,
                    bool: true
                });
            })()"#,
            move |result| {
                log::info!("eval_with_callback result from JS: {}", result);
                let _ = app_clone.emit("eval-with-callback-result", &result);
            },
        )?;
    }
    Ok(())
}
```

**JS 测试用例** (`examples/api/src/lib/tests/core.ts`):
```typescript
{
    name: 'webview.eval_with_callback',
    category: 'auto',
    async fn() {
        const resultPromise = new Promise<string>((resolve) => {
            const unlisten = listen('eval-with-callback-result', (event) => {
                unlisten.then((fn) => fn());
                resolve(event.payload as string);
            });
        });
        await invoke('test_eval_with_callback');
        const result = await resultPromise;
        const parsed = JSON.parse(result);
        assert(parsed.arithmetic === 3);
        assert(parsed.stringLen === 5);
        assert(parsed.bool === true);
    },
},
```

**测试原理**:
1. JS 端先注册事件监听 `eval-with-callback-result`
2. 调用 Rust 命令 `test_eval_with_callback`
3. Rust 使用 `eval_with_callback` 执行 JS 表达式并获取返回值
4. Rust 通过 `app.emit()` 将结果发送回 JS
5. JS 验证返回值内容

## 4. 设端验证结果

### 4.1 localStorage (eval_with_callback 基础验证)

```
A00000/com.tauri.api/tauritest: test_local_storage called
A00000/com.tauri.api/tauritest: localStorage test result from JS: "hello_from_rust"
```

**结论**: localStorage 的 setItem/getItem 通过 eval_with_callback 正确返回。

### 4.2 eval_with_callback (完整验证)

```
A00000/com.tauri.api/tauritest: test_eval_with_callback called
A00000/com.tauri.api/tauritest: eval_with_callback result from JS: "{\"arithmetic\":3,\"stringLen\":5,\"bool\":true}"
```

**结论**: eval_with_callback 在 OpenHarmony 上完全正常工作：
- 算术运算: 1+2=3 ✓
- 字符串属性: "hello".length=5 ✓
- 布尔值: true ✓
- JSON.stringify 返回值正确传递 ✓

### 4.3 eval (副作用验证)

```
A01999/com.tauri.api/DefaultWebview: onTitleReceive → onTitleChange called: ✅ Eval Success! (From Rust)
A01999/com.tauri.api/DefaultWebview: onTitleReceive → onTitleChange called: ✅ INIT SCRIPT WORKED!
```

**结论**: eval (fire-and-forget) 正常工作，标题变更正确生效。

## 5. 支持状态总结

| 功能 | 支持状态 | 验证方式 |
|------|----------|----------|
| `eval` (fire-and-forget) | ✅ 完全支持 | 标题变更验证 |
| `eval_with_callback` (返回值) | ✅ 完全支持 | JSON 返回值验证 |
| `localStorage` (依赖 domStorageAccess) | ✅ 完全支持 | setItem/getItem 验证 |

## 6. 关键注意事项

### 6.1 eval_with_callback 的回调参数索引

napi-ohos 的 `create_function_from_closure` 使用 **0-based** 参数索引：

```rust
let ret = ctx.try_get::<String>(0)?;  // 第一个参数，索引为 0
```

注意：不是 1-based，这是 OpenHarmony napi 的特殊行为。

### 6.2 domStorageAccess 必须开启

OpenHarmony Web 组件默认 `domStorageAccess=false`，必须显式设置 `domStorageAccess(true)` 才能使用 localStorage/sessionStorage。已在 `DefaultWebview.ets` 的 WebBuilder 和 EmbeddedWebBuilder 中添加。

### 6.3 回调中 emit 的线程安全

`eval_with_callback` 的回调在主线程执行，因此回调中可以直接使用 `app.emit()`。但如果回调涉及异步操作，需要注意线程安全。

### 6.4 JS 表达式必须是可序列化的

`runJavaScript` 返回 Promise 的 resolve 值是 JS 表达式的字符串表示。如果 JS 代码返回 `undefined`，napi 端会将其转换为 `"undefined"` 字符串：

```rust
let ret = match ret {
    Either::A(s) => s,
    Either::B(_ret) => String::from("undefined"),
};
```

建议用 IIFE 包裹 JS 代码并确保返回可序列化值：
```javascript
(function() { return JSON.stringify({...}); })()
```

## 7. 注册清单

新增 `test_eval_with_callback` 命令需要同步修改三个文件：

1. **cmd.rs**: 添加 `#[command] pub fn test_eval_with_callback`
2. **lib.rs**: 在 invoke_handler 列表中注册
3. **build.rs**: 在 app_manifest.commands 列表中添加 `"test_eval_with_callback"`
4. **run-app.json**: 在 permissions 中添加 `"allow-test-eval-with-callback"`
5. **core.ts**: 添加自动化测试用例
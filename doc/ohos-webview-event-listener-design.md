# OHOS WebView 事件监听器实现设计

## 概述

本文档记录 Tauri 在 OpenHarmony 平台上 WebView 事件监听器的实现设计，
涵盖 `on_navigation`、`on_document_title_changed`、`on_page_load`、`on_download` 四个事件接口。
重点描述：回调注册模式、napi 函数传递链路、ArkTS 组件事件绑定，以及实践中发现的关键问题与修复。

---

## 1. 架构总览

### 1.1 跨层调用链

```
用户代码 (lib.rs / cmd.rs)
  └─ WebviewWindowBuilder::on_xxx()
      └─ tauri-runtime-wry (构建时注册)
          └─ wry::WebViewBuilder::on_xxx()
              └─ wry/src/ohos/mod.rs (InnerWebView::new, 构建前注册)
                  └─ openharmony-ability::WebViewBuilder::on_xxx()
                      └─ napi create_function_from_closure → Function
                          └─ WebViewInitData 传递给 ArkTS
                              └─ ArkHelper.ets → DefaultWebview.ets
                                  └─ ArkUI Web 组件事件属性
```

### 1.2 两个 WebViewInitData 类型

| 类型 | 来源 | 作用 |
|------|------|------|
| `NativeWebViewInitData` | `type.ets` (手工维护) | napi 传递过来的数据类型声明 |
| `WebViewInitData` | `DefaultWebview.ets` | ArkUI Web 组件使用的数据接口 |

**关键**: napi 的 `#[napi(object)]` struct 字段名会自动转为 camelCase（`on_page_begin` → `onPageBegin`），
但 `NativeWebViewInitData` 是手工维护的接口，新增字段时必须同步更新。

### 1.3 数据传递流程

```
Rust WebViewInitData (napi object)
  │
  ├─ on_navigation_request: Option<Function<String, bool>>
  ├─ on_title_change:       Option<Function<String, ()>>
  ├─ on_page_begin:         Option<Function<String, ()>>
  ├─ on_page_end:           Option<Function<String, ()>>
  ├─ on_download_start:     Option<Function<(String, String), DownloadStartResult>>
  ├─ on_download_end:       Option<Function<(String, Option<String>, bool), ()>>
  │
  └─→ napi 序列化为 JS 对象 (camelCase 字段名)
       │
       ├─ onNavigationRequest: function | undefined
       ├─ onTitleChange:       function | undefined
       ├─ onPageBegin:         function | undefined
       ├─ onPageEnd:           function | undefined
       ├─ onDownloadStart:     function | undefined
       ├─ onDownloadEnd:       function | undefined
       │
       └─→ ArkHelper.ets 提取并传入 WebViewInitData
            └─→ DefaultWebview.ets Web 组件绑定事件
```

---

## 2. 回调注册模式：构建前 vs 构建后

### 2.1 构建前注册（Pre-build）— 正确模式

ArkUI 的 `Web` 组件事件必须在 `@Builder` 函数中作为组件属性绑定，
不能在组件创建后再动态添加。

```rust
// wry/src/ohos/mod.rs — 构建前注册
let mut webview_builder = WebViewBuilder::new()
    .on_navigation_request(move |url| { ... })
    .on_title_change(move |title| { ... })
    .on_page_begin(move |url| { ... })
    .on_page_end(move |url| { ... });

let webview = webview_builder.build()?;
```

回调在 `WebViewBuilder::build()` 时被转换为 napi `Function`，
随 `WebViewInitData` 一起传入 ArkTS，最终在 `@Builder` 函数中绑定到 `Web` 组件。

### 2.2 构建后注册（Post-build）— 仅适用于原生层回调

`Webview`（Rust 侧）的 `on_controller_attach()`、`on_page_begin()`、`on_page_end()` 方法
是注册到 `ohos_web_binding::Web`（原生层）的回调，不经过 ArkTS 组件。

但 **ArkTS 组件事件不能通过这种方式触发**，因为原生层回调不会调用 ArkTS 的 `.onPageBegin()` 等属性。

### 2.3 设计决策

| 事件 | 注册时机 | 原因 |
|------|----------|------|
| on_navigation | 构建前 | 需要 ArkTS `onLoadIntercept` 事件 |
| on_document_title_changed | 构建前 | 需要 ArkTS `onTitleReceive` 事件 |
| on_page_load | 构建前 | 需要 ArkTS `onPageBegin/onPageEnd` 事件 |
| on_download | 构建前 | 需要 ArkTS `WebDownloadDelegate`（在 `onControllerAttached` 中设置） |
| on_controller_attach | 构建后 | 原生层回调，不需要 ArkTS 组件事件 |

---

## 3. 各事件实现细节

### 3.1 on_navigation（导航拦截）

**语义映射**: OpenHarmony `onLoadIntercept` 与 Tauri `on_navigation` 语义**相反**：
- `onLoadIntercept`: 返回 `true` = 拦截（阻止），`false` = 放行
- `on_navigation`: 返回 `true` = 允许，`false` = 拦截

```typescript
// DefaultWebview.ets
.onLoadIntercept((event) => {
  if (typeof data?.onNavigationRequest === "function") {
    const url = event.data.getRequestUrl();
    const ret = data.onNavigationRequest(url);
    return !ret;  // 语义反转：allow → false(放行), block → true(拦截)
  }
  return false;  // 默认放行
})
```

**napi 闭包**:
```rust
// WebViewBuilder::build()
let on_navigation_request = self.on_navigation_request.and_then(|handler| {
    env.create_function_from_closure("on_navigation_request", move |ctx| {
        let url = ctx.try_get::<String>(0)?;  // 0-based 索引
        let url_str = match url { Either::A(s) => s, Either::B(_) => String::new() };
        let ret = handler(url_str);
        Ok(ret)  // 返回 bool
    }).ok()
});
```

### 3.2 on_document_title_changed（标题变更）

ArkTS `onTitleReceive` 事件提供 `e.title`，直接传递给回调。

```typescript
// DefaultWebview.ets
.onTitleReceive((e) => {
  if (typeof data?.onTitleChange === "function") {
    data.onTitleChange(e.title);
  }
})
```

**特殊行为**: 初始化脚本（`initialization_scripts`）在 `DOMContentLoaded` 前执行，
所以标题变更顺序为：
1. HTML `<title>` 解析 → "Hello World"
2. 初始化脚本执行 → 覆盖标题

### 3.3 on_page_load（页面加载）— 修复历程

**初始问题**: Rust 侧 `on_page_load` 回调不被触发。

**三层修复**:

| 层 | 问题 | 修复 |
|----|------|------|
| wry 层 | 回调在 build 之后注册，原生层回调无法触发 ArkTS 组件事件 | 移到构建前，通过 WebViewBuilder 注册 |
| ArkHelper.ets | `createWebview` 未传递 `onPageBegin/onPageEnd` | 添加 `onPageBegin: data?.onPageBegin` 等字段 |
| type.ets | `NativeWebViewInitData` 接口未声明 `onPageBegin/onPageEnd` | 添加属性定义，否则 ArkTS 编译报错 |

**最终实现**:

wry 层拆分 `on_page_load_handler` 为 `on_page_begin` 和 `on_page_end`：
```rust
// wry/src/ohos/mod.rs
if let Some(on_page_load_handler) = on_page_load_handler {
    let handler = Arc::new(on_page_load_handler);
    let handler_begin = handler.clone();
    let handler_end = handler.clone();
    webview_builder = webview_builder.on_page_begin(move |url: String| {
        handler_begin(PageLoadEvent::Started, url);
    });
    webview_builder = webview_builder.on_page_end(move |url: String| {
        handler_end(PageLoadEvent::Finished, url);
    });
}
```

ArkTS 组件绑定：
```typescript
// DefaultWebview.ets — WebBuilder & EmbeddedWebBuilder
.onPageBegin((e) => {
  if (typeof data?.onPageBegin === "function") {
    data.onPageBegin(e.url);
  }
})
.onPageEnd((e) => {
  if (typeof data?.onPageEnd === "function") {
    data.onPageEnd(e.url);
  }
})
```

### 3.4 on_download（下载处理）

下载处理使用 `WebDownloadDelegate`，必须在 `onControllerAttached` 中设置：
```typescript
// DefaultWebview.ets
.onControllerAttached(() => {
  setupDownloadDelegate(data);
})
```

`setupDownloadDelegate` 创建 `WebDownloadDelegate` 并绑定 `onBeforeDownload`、
`onDownloadFinish`、`onDownloadFailed`，然后调用 `data.controller.setDownloadDelegate(download)`。

Rust 侧回调类型较复杂，`on_download_start` 返回 `DownloadStartResult`（包含 `allow` 和 `temp_path`），
`on_download_end` 接收 `(url, path, success)` 三参数。

---

## 4. 新增事件处理器配方

当需要新增一个 WebView 事件监听器时，需修改以下文件：

### Step 1: openharmony-ability Rust 层

**文件**: `crates/ability/src/helper/webview.rs`
- 在 `WebViewInitData` struct 中添加 `Option<Function<...>>` 字段
- 注意生命周期 `'a`

**文件**: `crates/ability/src/webview/mod.rs`
- 在 `WebViewBuilder` struct 中添加 `Option<Box<dyn Fn(...)>>` 字段
- 添加 builder 方法（使用 `unsafe { std::mem::transmute }` 消除生命周期）
- 在 `build()` 方法中创建 napi 闭包（`env.create_function_from_closure`）
  - 使用 **0-based** 参数索引（`ctx.try_get::<T>(0)`）
  - 处理 `Either::A/B` 返回值
- 将闭包传入 `WebViewInitData`

### Step 2: openharmony-ability ArkTS 层

**文件**: `native_ability/src/main/ets/ability/type.ets`
- 在 `NativeWebViewInitData` 接口中添加字段声明

**文件**: `native_ability/src/main/ets/ability/ArkHelper.ets`
- 在 `createWebview` 和 `createEmbeddedWebview` 中传递新字段
  - `onXxx: data?.onXxx`

**文件**: `native_ability/src/main/ets/webview/DefaultWebview.ets`
- 在 `WebviewInitData` 接口中添加回调字段
- 在 `WebBuilder` 的 `Web` 组件中添加事件属性绑定
- 在 `EmbeddedWebBuilder` 的 `Web` 组件中添加同样的事件绑定

### Step 3: wry 层

**文件**: `wry/src/ohos/mod.rs`
- 在 `InnerWebView::new()` 中提取 `WebViewAttributes` 的对应字段
- 通过 `WebViewBuilder` 方法注册回调（**构建前**）

### Step 4: tauri-runtime-wry 层

**文件**: `crates/tauri-runtime-wry/src/lib.rs`
- 在 Webview 构建区域调用 wry 的对应 builder 方法

### Step 5: tauri API 层（如果尚未定义）

**文件**: `crates/tauri/src/webview/mod.rs` 或 `webview_window.rs`
- 添加公共 API 方法和 `WebViewAttributes` 字段

---

## 5. 关键教训

### 5.1 napi 参数索引：0-based

```rust
// ❌ 错误 (1-based, 会导致 "Arguments index out of range" 崩溃)
let url = ctx.try_get::<String>(1)?;

// ✅ 正确 (0-based)
let url = ctx.try_get::<String>(0)?;
```

### 5.2 三种 WebViewInitData 必须同步

添加新的回调字段时，必须同时更新：
1. Rust `WebViewInitData`（napi struct）— 自动生成 camelCase JS 属性
2. ArkTS `NativeWebViewInitData`（type.ets）— 手工维护的类型声明
3. ArkTS `WebViewInitData`（DefaultWebview.ets）— Web 组件使用的数据接口

如果只更新了 Rust 层但忘记更新 `type.ets`，ArkTS 编译会报错：
`Property 'onXxx' does not exist on type 'WebViewInitData'`

如果只更新了 Rust 层和 type.ets 但忘记在 `ArkHelper.ets` 传递字段，
回调到达 DefaultWebview.ets 时会是 `undefined`（`typeof === "undefined"`）。

### 5.3 ArkUI 组件事件必须在 @Builder 中绑定

ArkUI 的 `Web` 组件事件（如 `.onPageBegin()`、`.onTitleReceive()`）
必须在 `@Builder` 函数中声明为组件属性。无法在组件创建后动态添加。

这决定了所有需要 ArkTS 事件的回调都必须通过 **构建前注册** 模式传递。

### 5.4 语义反转：onLoadIntercept vs on_navigation

OpenHarmony 的 `onLoadIntercept` 和 Tauri 的 `on_navigation` 语义相反，
必须在 ArkTS 层做 `!ret` 反转。遗漏此反转会导致所有导航被拦截（页面无法加载）。

### 5.5 WebDownloadDelegate 必须在 onControllerAttached 后设置

`setDownloadDelegate()` 需要在 WebviewController 就绪后调用，
否则 delegate 不会生效。我们在 `onControllerAttached` 回调中设置。

---

## 6. 事件对照表

| Tauri 事件 | ArkTS Web 组件事件 | 注册时机 | 语义映射 |
|------------|-------------------|----------|----------|
| on_navigation | onLoadIntercept | 构建前 | 反转 (true↔false) |
| on_document_title_changed | onTitleReceive | 构建前 | 直传 |
| on_page_load (Started) | onPageBegin | 构建前 | 直传 |
| on_page_load (Finished) | onPageEnd | 构建前 | 直传 |
| on_download_start | WebDownloadDelegate.onBeforeDownload | 构建前(闭包), 运行时(delegate) | 类型转换 |
| on_download_end | WebDownloadDelegate.onFinish/onFailed | 构建前(闭包), 运行时(delegate) | 类型转换 |
| on_controller_attach | Web.on_controller_attach | 构建后 | 原生层回调 |

---

## 7. 验证方法

### 7.1 设备日志

```bash
# 快速查看 WebView 事件日志
hdc shell "hilog -x 2>/dev/null | grep -E 'A01999'"

# 查看完整应用日志
hdc shell "hilog -x 2>/dev/null | grep -E 'com.tauri.api'"
```

### 7.2 前端测试

在 `examples/api/src/lib/tests/core.ts` 中通过 `listen()` 监听事件，
使用 `invoke('create_isolated_window', ...)` 创建子窗口触发事件，
等待 1-2 秒后断言事件是否收到。

```typescript
{
  name: 'on_navigation interceptor',
  async fn() {
    let interceptedUrl: string | null = null;
    const unlisten = await listen('navigation-intercepted', (event) => {
      interceptedUrl = event.payload as string;
    });
    await invoke('create_isolated_window', { ... });
    await new Promise((r) => setTimeout(r, 1500));
    assert(interceptedUrl !== null, 'Expected navigation-intercepted event');
    unlisten();
  },
}
```

### 7.3 构建与安装

```bash
# 1. 打包 openharmony-ability
cd C:\myprogram\code\tauri\openharmony-ability && .\pack.bat

# 2. 构建
cd C:\myprogram\code\tauri\tauri\examples\api && cargo tauri ohos build --device-type desktop

# 3. 安装
cd C:\myprogram\code\tauri\tauri\examples\api && .\hdcinstall.bat
```

---

## 附录：文件修改清单

| 文件 | 修改内容 |
|------|----------|
| `wry/src/ohos/mod.rs` | 提取 navigation/title/page_load/download handler，构建前注册 |
| `openharmony-ability/crates/ability/src/helper/webview.rs` | WebViewInitData 添加 on_page_begin/on_page_end |
| `openharmony-ability/crates/ability/src/webview/mod.rs` | WebViewBuilder 添加 on_page_begin/on_page_end |
| `openharmony-ability/native_ability/.../ability/type.ets` | NativeWebViewInitData 添加 onPageBegin/onPageEnd |
| `openharmony-ability/native_ability/.../ability/ArkHelper.ets` | 传递 onPageBegin/onPageEnd |
| `openharmony-ability/native_ability/.../webview/DefaultWebview.ets` | WebViewInitData 接口 + 组件事件绑定 |
| `examples/api/src-tauri/src/lib.rs` | 主窗口 on_navigation/on_title/on_download 事件发射 |
| `examples/api/src-tauri/src/cmd.rs` | 子窗口 on_page_load/on_navigation/on_title 事件发射 + 编号标题 |
| `examples/api/src/lib/tests/core.ts` | 前端测试：on_navigation, on_document_title_changed |
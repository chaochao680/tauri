# OHOS WebView 事件接口调用链梳理

## 概述

本文档梳理 Tauri 框架在 OpenHarmony 平台上 WebView 相关事件接口的调用链现状，
覆盖从 Tauri 用户 API → tauri-runtime-wry → wry → openharmony-ability → ArkTS/ETS 的完整链路。

---

## 调用链架构

```
用户代码 (lib.rs)
  └─ WebviewBuilder::on_xxx()
      └─ tauri-runtime-wry (lib.rs:4955-5099)
          └─ wry::WebViewBuilder::with_xxx_handler()
              └─ wry/src/ohos/mod.rs (InnerWebView::new)
                  └─ openharmony-ability::WebViewBuilder / Webview
                      └─ ArkTS/ETS Web 组件事件
```

---

## 1. on_navigation（导航拦截）

| 层级 | 位置 | 状态 |
|------|------|------|
| Tauri API | `crates/tauri/src/webview/mod.rs:540` `on_navigation()` | ✅ 已定义 |
| tauri-runtime-wry | `src/lib.rs:4955` `with_navigation_handler()` | ✅ 已调用 |
| wry 通用层 | `wry/src/lib.rs:1281` `with_navigation_handler()` | ✅ 已定义 |
| **wry ohos 层** | **`wry/src/ohos/mod.rs:33-49`** | **❌ 被丢弃（`..` 解构）** |
| openharmony-ability builder | `crates/ability/src/webview/mod.rs:179` `on_navigation_request()` | ✅ 方法已定义 |
| openharmony-ability NAPI | `crates/ability/src/helper/webview.rs:56` `on_navigation_request` | ✅ 字段已定义 |
| ArkTS WebViewInitData | `type.ets:60` `onNavigationRequest?: (url) => boolean` | ✅ 接口已定义 |
| ArkUI Web 组件 | `DefaultWebview.ets:96` `.onLoadIntercept()` → `data.onNavigationRequest(url)` | ✅ 已绑定 |

**断点**: wry ohos 层在 `InnerWebView::new()` 解构 `WebViewAttributes` 时用 `..` 丢弃了 `navigation_handler`，
未调用 `WebViewBuilder::on_navigation_request()` 将处理器传递给 openharmony-ability。

---

## 2. on_document_title_changed（标题变更）

| 层级 | 位置 | 状态 |
|------|------|------|
| Tauri API | `crates/tauri/src/webview/mod.rs:606` `on_document_title_changed()` | ✅ 已定义 |
| tauri-runtime-wry | `src/lib.rs:5024` `with_document_title_changed_handler()` | ✅ 已调用 |
| wry 通用层 | `wry/src/lib.rs:1362` `with_document_title_changed_handler()` | ✅ 已定义 |
| **wry ohos 层** | **`wry/src/ohos/mod.rs:33-49`** | **❌ 被丢弃（`..` 解构）** |
| openharmony-ability builder | `crates/ability/src/webview/mod.rs:194` `on_title_change()` | ✅ 方法已定义 |
| openharmony-ability NAPI | `crates/ability/src/helper/webview.rs:57` `on_title_change` | ✅ 字段已定义 |
| ArkTS WebViewInitData | `type.ets:61` `onTitleChange?: (title) => void` | ✅ 接口已定义 |
| ArkUI Web 组件 | `DefaultWebview.ets:104` `.onTitleReceive(e)` → `data.onTitleChange(e.title)` | ✅ 已绑定 |

**断点**: 同上，wry ohos 层丢弃了 `document_title_changed_handler`，
未调用 `WebViewBuilder::on_title_change()` 将处理器传递给 openharmony-ability。

---

## 3. on_page_load（页面加载）

| 层级 | 位置 | 状态 |
|------|------|------|
| Tauri API | `crates/tauri/src/webview/mod.rs:700` `on_page_load()` | ✅ 已定义 |
| tauri-runtime-wry | `src/lib.rs:5088` `with_on_page_load_handler()` | ✅ 已调用 |
| wry 通用层 | `wry/src/lib.rs:1384` `with_on_page_load_handler()` | ✅ 已定义 |
| **wry ohos 层** | **`wry/src/ohos/mod.rs:33-49`** | **❌ 被丢弃（`..` 解构）** |
| openharmony-ability Webview | `crates/ability/src/helper/webview.rs:301` `on_page_begin()` | ✅ 方法已定义（创建后注册） |
| openharmony-ability Webview | `crates/ability/src/helper/webview.rs:311` `on_page_end()` | ✅ 方法已定义（创建后注册） |
| ArkTS Web 组件 | 无直接 ArkTS 接口 | ⚠️ 原生绑定层回调，不经过 ArkTS |

**断点**: wry ohos 层丢弃了 `on_page_load_handler`。
底层已有 `Webview::on_page_begin()` 和 `Webview::on_page_end()` 方法，
但需要在 `build()` 之后调用。需将 `PageLoadEvent::Started` 映射到 `on_page_begin()`,
`PageLoadEvent::Finished` 映射到 `on_page_end()`。

---

## 4. on_new_window（新窗口请求）

| 层级 | 位置 | 状态 |
|------|------|------|
| Tauri API | `crates/tauri/src/webview/mod.rs:597` `on_new_window()` | ✅ 已定义 |
| tauri-runtime-wry | `src/lib.rs:4964` `with_new_window_req_handler()` | ✅ 已调用 |
| wry 通用层 | `wry/src/lib.rs:1343` `with_new_window_req_handler()` | ✅ 已定义 |
| **wry ohos 层** | **`wry/src/ohos/mod.rs:33-49`** | **❌ 被丢弃（`..` 解构）** |
| openharmony-ability | ❌ 无对应接口 | |
| ArkTS | ❌ 无对应接口 | |

**断点**: 完全缺失。openharmony-ability 和 ArkTS 层都没有对应接口。
当前 ohos 上 Tauri 的 `on_new_window` 返回 `NewWindowResponse::Allow(PhantomData)` 简单放行，
不做拦截。需要在 openharmony-ability 中补充此接口。

---

## 5. on_close_requested（关闭请求）

| 层级 | 位置 | 状态 |
|------|------|------|
| Tauri API | `crates/tauri/src/webview/webview_window.rs` | ✅ 已定义（Window 级别） |
| tauri-runtime-wry | `src/lib.rs:4487` | ✅ 已调用 |
| wry ohos 层 | ❌ 无对应 | 窗口关闭由 ArkUI 窗口生命周期处理 |
| openharmony-ability | `Event::Destroy` | ⚠️ 能力销毁事件，不是可拦截的关闭请求 |

**断点**: OpenHarmony 不支持拦截窗口关闭请求，只能监听 `onAbilityDestroy`。
这是平台限制，无法实现 `on_close_requested` 的拦截语义。

---

## 6. download_handler（下载处理）

| 层级 | 位置 | 状态 |
|------|------|------|
| Tauri API | `WebviewBuilder::download_handler()` | ✅ 已定义 |
| wry ohos 层 | ❌ 未传递 | |
| openharmony-ability builder | `on_download_start()` / `on_download_end()` | ✅ 方法已定义 |
| ArkTS WebViewInitData | `onDownloadStart` / `onDownloadEnd` | ✅ 接口已定义 |
| ArkUI Web 组件 | `DefaultWebview.ets:224` `WebDownloadDelegate` | ✅ 已绑定 |

**断点**: wry ohos 层同样丢弃了 `download_handler`。底层已有完整支持。

---

## 7. 生命周期事件（已打通 ✅）

| 事件 | ArkTS → Rust | 状态 |
|------|-------------|------|
| onWindowStageCreate | → `Event::WindowCreate` | ✅ |
| onWindowStageDestroy | → `Event::WindowDestroy` | ✅ |
| onAbilityCreate | → `Event::Create` | ✅ |
| onAbilityDestroy | → `Event::Destroy` | ✅ |
| onWindowStageEvent | → `Event::Start/GainedFocus/LostFocus/Pause/Stop` | ✅ |
| onWindowSizeChange | → `Event::WindowResize` | ✅ |
| onWindowRectChange | → `Event::ContentRectChange` | ✅ |
| onAvoidAreaChange | → `Event::AvoidAreaChange` | ✅ |
| onConfigurationUpdated | → `Event::ConfigChanged` | ✅ |
| onMemoryLevel | → `Event::LowMemory` | ✅ |
| onKeyboardHeightChange | → `Event::KeyboardEvent` | ✅ |
| onAbilitySaveState | → `Event::SaveState` | ✅ |
| onBackPress | → `get_back_press_interceptor()` | ✅ |

---

## 8. 其他 stub / 缺陷

| 接口 | 位置 | 状态 |
|------|------|------|
| `onAbilityRestoreState` | Rust lifecycle.rs:27 定义，ArkTS type.ets **缺失** | ❌ 接口不对等 |
| `WebViewComponentEventCallback` | type.ets:34-37 定义 `onComponentCreated/onComponentDestroyed` | ❌ 未接线（纯 stub） |
| `onDragAndDrop` | feature-gated `drag_and_drop` | ❌ 仅 stub |
| `registerIconClickHandler/registerMenuClickHandler` | ArkHelper.ets 遗留 stub | ⚠️ 仅 log，实际用 `_onIconClick/_onMenuClick` |

---

## 修复优先级

### P0：必须修复（wry ohos 层断线）— ✅ 已修复

1. **on_navigation**: ✅ wry ohos 已提取 `navigation_handler`，调用 `WebViewBuilder::on_navigation_request()`
2. **on_document_title_changed**: ✅ 同上，已提取 `document_title_changed_handler`，调用 `WebViewBuilder::on_title_change()`
3. **on_page_load**: ✅ 已提取 `on_page_load_handler`，build 后调用 `webview.on_page_begin()` / `webview.on_page_end()`
4. **download_handler**: ✅ 已提取 `download_handler`，调用 `WebViewBuilder::on_download_start()` / `on_download_end()`

**附加修复**: napi_ohos `create_function_from_closure` 使用 0-based 索引（index 0 = 第一个参数），原来所有闭包使用 `ctx.try_get::<T>(1)` (1-based) 导致 "Arguments index out of range" 崩溃。已全部修正为 `ctx.try_get::<T>(0)`。

### P1：需要新增（openharmony-ability 缺失接口）

5. **on_new_window**: 需在 openharmony-ability 新增 `on_new_window_request` 接口，ArkTS 补充对应事件

### P2：平台限制 / 低优先级

6. **on_close_requested**: OpenHarmony 平台不支持拦截，无法实现
7. **onAbilityRestoreState**: Rust/ArkTS 接口不对等，需补充
8. **onDragAndDrop**: 仅 stub，依赖 HarmonyPC 特性

---

## 编译验证命令

```bash
# 1. 打包 openharmony-ability
cd C:\myprogram\code\tauri\openharmony-ability
.\pack.bat

# 2. 构建 ohos 应用
cd C:\myprogram\code\tauri\tauri\examples\api
cargo tauri ohos build --device-type desktop
```
# ohos-web-page-snapshot Specification

## Purpose
TBD - created by archiving change p1-web-page-snapshot. Update Purpose after archive.
## Requirements
### Requirement: ArkTS JsHelper exposes webPageSnapshot method
ArkTS JsHelper 接口 SHALL 提供 `webPageSnapshot` 方法，接受 webview tag 参数，调用对应 Web 组件的 `WebviewController.webPageSnapshot()` API，将返回的 `PixelMap` 转换为 RGBA `Uint8Array`，以 `{ rgba: Uint8Array, width: number, height: number }` 格式通过 Promise 返回。

#### Scenario: Successful full-page snapshot
- **WHEN** Rust 侧通过 NAPI 调用 JsHelper 的 `webPageSnapshot` 方法，传入有效的 webview tag
- **THEN** ArkTS 调用 `WebviewController.webPageSnapshot()`，将 `SnapshotResult.imagePixelMap` 通过 `readPixelsToBufferSync` 转为 RGBA `Uint8Array`，并返回包含 rgba 数据、width、height 的对象

#### Scenario: Web page not fully loaded
- **WHEN** `webPageSnapshot` 被调用但 Web 组件尚未完成初始化或未关联
- **THEN** 返回的 Promise SHALL reject 并包含错误信息（如 "WebviewController not associated" 或 OHOS 错误码）

#### Scenario: PixelMap resource cleanup
- **WHEN** `webPageSnapshot` 成功获取 PixelMap 并完成 RGBA 转换
- **THEN** ArkTS 侧 SHALL 立即调用 `pixelMap.release()` 释放资源，即使转换过程中发生异常也 MUST 通过 try/finally 确保释放

### Requirement: Rust Webview struct provides web_page_snapshot method
openharmony-ability 的 Rust `Webview` struct SHALL 提供 `web_page_snapshot(&self, callback)` 方法，使用回调模式接收 `SnapshotData`（包含 `rgba: Vec<u8>`、`width: u32`、`height: u32`）。回调模式用于避免主线程死锁（`webPageSnapshot` 是异步 API，不能同步等待 Promise）。

#### Scenario: Successful snapshot from Rust
- **WHEN** 调用 `webview.web_page_snapshot(callback)`
- **THEN** 方法 SHALL 通过 NAPI `Function::call` 调用 ArkTS JsHelper 的 `webPageSnapshot` 方法，获取返回的 Promise，通过 `.then()/.catch()` 处理结果，成功时调用 `callback(Ok(SnapshotData))`

#### Scenario: Main thread safety via callback pattern
- **WHEN** `web_page_snapshot()` 被调用
- **THEN** 方法 SHALL 使用回调模式（非同步阻塞），因为 `webPageSnapshot` 是异步 OHOS API，在主线程同步等待 Promise 会导致死锁

#### Scenario: ArkTS side error propagation
- **WHEN** ArkTS 侧 `webPageSnapshot` Promise reject
- **THEN** Rust 侧 SHALL 调用 `callback(Err(error_message))`，包含 ArkTS 错误信息的字符串描述

#### Scenario: Promise then/catch handling
- **WHEN** Promise resolve 时数据提取失败（如缺少 rgba/width/height 字段）
- **THEN** SHALL 调用 `callback(Err(description))` 而非 panic

### Requirement: SnapshotData struct definition
openharmony-ability SHALL 定义 `SnapshotData` struct 作为截图返回类型。

#### Scenario: SnapshotData fields
- **WHEN** `web_page_snapshot()` 成功返回
- **THEN** `SnapshotData` MUST 包含：`rgba: Vec<u8>`（RGBA_8888 格式像素数据，长度 = width × height × 4）、`width: u32`（像素宽度）、`height: u32`（像素高度）


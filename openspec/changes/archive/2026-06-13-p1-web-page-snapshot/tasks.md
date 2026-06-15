## 1. ArkTS 侧 webPageSnapshot 实现

- [x] 1.1 在 `native_ability/src/main/ets/webview/Utils.ets` 的 JsHelper 接口中新增 `webPageSnapshot` 方法声明，接受 tag 参数，返回 `Promise<{ rgba: Uint8Array, width: number, height: number }>`
- [x] 1.2 在 `native_ability/src/main/ets/webview/DefaultWebview.ets` 中实现 `webPageSnapshot`：调用 `controller.webPageSnapshot()`，在 AsyncCallback 中将 `SnapshotResult.imagePixelMap` 通过 `readPixelsToBufferSync` 转为 RGBA `Uint8Array`，使用 try/finally 确保 `pixelMap.release()`，resolve `{ rgba, width, height }`

## 2. Rust 侧 web_page_snapshot 方法

- [x] 2.1 在 `crates/ability/src/helper/webview.rs` 中定义 `SnapshotData` struct：`pub struct SnapshotData { pub rgba: Vec<u8>, pub width: u32, pub height: u32 }`
- [x] 2.2 在 `crates/ability/src/helper/webview.rs` 的 `Webview` impl 中新增 `web_page_snapshot(&self) -> Result<SnapshotData>` 方法：通过 NAPI `Function::call` 调用 JsHelper 的 `webPageSnapshot`，解析返回的 JS 对象提取 rgba/width/height 字段

## 3. 验证与测试

- [ ] 3.1 确认 `cargo check --target ohos` 编译通过（需 OHOS 构建环境）
- [ ] 3.2 在设备端验证：通过 NAPI 调用 `web_page_snapshot()`，确认返回非空 RGBA 数据且 `rgba.len() == width * height * 4`（需 OHOS 设备）

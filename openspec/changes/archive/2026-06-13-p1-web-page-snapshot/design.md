## Context

openharmony-ability 是 Tauri OHOS 的唯一 ArkTS 桥接仓。当前 `Webview` struct 提供 `url()`、`load_url()`、`evaluate_script()`、`cookies()` 等方法，但不支持截图。

OHOS `WebviewController.webPageSnapshot(SnapshotInfo, AsyncCallback<SnapshotResult>)` API（API 12+）可获取网页全量绘制结果为 `PixelMap`。

现有 clipboard 模块（`crates/ability/src/clipboard/mod.rs`）已建立 TSFN + oneshot channel 的异步模式：Rust 通过 TSFN 调用 ArkTS 方法，ArkTS 返回 Promise，Rust 用 oneshot channel 等待结果。

**约束**：
- 三条铁律：openharmony-ability 是唯一桥接仓，不影响其他平台，`cfg(target_env = "ohos")` 隔离
- NAPI `snake_case` → `camelCase` 自动转换
- TSFN 必须 `callee_handled::<false>()`
- 禁止 `run_on_main_thread + rx.recv()` 阻塞模式
- `ObjectRef` 非 Send/Sync，需 `unsafe impl Send`

## Goals / Non-Goals

**Goals:**
- 在 openharmony-ability Rust `Webview` struct 上新增 `web_page_snapshot()` 方法，返回 RGBA 字节数据 + 宽高
- ArkTS 侧调用 `WebviewController.webPageSnapshot()` 并将 PixelMap 转为 RGBA 数据
- 遵循现有 clipboard TSFN 模式实现异步回调
- 正确处理 PixelMap 资源释放

**Non-Goals:**
- 不在 Tauri 上层（wry/tauri-runtime/tauri core）新增任何截图 API
- 不实现 `with_webview` 贯通（Phase 2 的工作）
- 不支持 `getSurfaceId` + `createPixelMapFromSurface` 的 Surface 截图模式
- 不做长截图拼接（webPageSnapshot 自身处理全量绘制）

## Decisions

### D1: 返回数据类型 — `SnapshotData { rgba: Vec<u8>, width: u32, height: u32 }`

**选择**：定义 `SnapshotData` struct 返回 RGBA 字节 + 宽高。

**理由**：
- 与 clipboard 模块的 `ClipboardImageData { rgba, width, height }` 一致
- `Vec<u8>` 在 Rust 侧易于处理（可转为 `Image`、写入文件、传递给上层）
- 宽高信息用于后续创建图像

**替代方案**：返回 `image::PixelMap` 或 `image::DynamicImage`（引入 `image` crate 依赖）→ 增加不必要的依赖，且 openharmony-ability 应保持轻量

### D2: ArkTS 侧 PixelMap → RGBA 转换 — `readPixelsToBufferSync`

**选择**：使用 `pixelMap.readPixelsToBufferSync(buffer)` + `new Uint8Array(buffer)` 同步转换。

**理由**：
- 同步方法简单直接，避免额外的 Promise 链
- `webPageSnapshot` 本身是异步的（AsyncCallback），ArkTS 侧已在异步上下文中
- `readPixelsToBuffer` 读取全图数据，不需要指定区域

**注意**：OHOS `webPageSnapshot` 返回的 PixelMap 格式是 `RGBA_8888`（与 `createPixelMap` 默认格式一致），不需要 BGRA→RGBA 转换。

**SnapshotResult 字段校验**：官方文档（API 12+）定义 `SnapshotResult` 包含 `id?`, `status?`, `size?`, `imagePixelMap?` 四个可选字段。ArkTS 侧必须检查 `result.status === true` 且 `result.imagePixelMap` 非 undefined，否则视为截图失败并 reject Promise。`result.size` 返回实际绘制尺寸（单位 vp），可用于验证。

### D3: 异步通信模式 — 回调模式（PromiseRaw .then/.catch）

**选择**：使用回调模式而非同步返回，避免主线程死锁。

**流程**：
1. Rust `web_page_snapshot(callback)` 通过 `Function::call()` 调用 ArkTS `webPageSnapshot()`
2. 获取返回的 Promise，通过 `PromiseRaw::then()` 和 `catch()` 处理结果
3. `.then()` 中提取 SnapshotData 并调用 `callback(Ok(data))`
4. `.catch()` 中提取错误信息并调用 `callback(Err(reason))`

**理由**：`webPageSnapshot` 是 OHOS 异步 API，在主线程同步等待 Promise 会导致事件循环死锁。回调模式与 `evaluate_script_with_callback` 一致，是 openharmony-ability 中处理异步操作的标准模式。TSFN + oneshot 模式虽然可行（clipboard 模块使用），但需要额外的 TSFN 初始化基础设施，对于 Phase 1 不必要。

### D4: SnapshotInfo 参数 — 固定传全量尺寸

**选择**：不暴露 `SnapshotInfo` 的 `size` 参数给用户，使用默认全量绘制。

**理由**：
- Phase 1 仅做能力桥接，不需要精细控制
- `webPageSnapshot` 默认绘制整个网页内容
- 如需指定尺寸，可在 Phase 2 或更高层封装时扩展

## Risks / Trade-offs

- **[大页面内存]** → `webPageSnapshot` 最大支持 16000×16000px，超大页面可能 OOM。Mitigation: ArkTS 侧捕获异常并返回错误，不 crash。
- **[视频内容空白]** → 官方文档说明视频区域在截图中显示占位图或空白。这是 OHOS API 限制，不在此 Phase 解决。
- **[PixelMap 泄漏]** → 忘记 `pixelMap.release()` 会泄漏 GPU 内存。Mitigation: ArkTS 侧在转换完 RGBA 后立即 `pixelMap.release()`，使用 try/finally 确保释放。
- **[webPageSnapshot 是 API 12+]** → 低于 API 12 的设备不支持。Mitigation: 调用时捕获异常返回错误信息，不做版本守卫（tauri api demo 默认 API 12）。

## Why

examples/api 的 HTTP 自动测试（8 个用例）100% 依赖外部服务 httpbin.org，导致频繁的 timeout 和 503 错误。最新测试报告显示 5/8 测试失败，全部表现为精确的 5000ms 超时。

**根因分析：**
1. httpbin.org 是公共免费服务，存在限流和 503 问题
2. 测试串行执行，前 2-3 个请求消耗连接预算后，后续请求被限流
3. 项目已有 localhost:3003 echo server（`lib.rs` 中的 tiny_http），但**无测试使用**
4. echo server 仅在 `#[cfg(desktop)]` 下启动，OHOS mobile 上不可用
5. HTTP scope 配置未包含 localhost，前端无法调用
6. echo server 固定返回 200，无法测试 error handling (404)
7. 外网请求无 connectTimeout 和 retry 机制

**影响：** HTTP 测试的假失败污染整体测试报告，影响 CI 判定和其他问题的定位。

## What Changes

- **Echo server 增强**：移除 `#[cfg(desktop)]` 门控（全平台可用），添加 `/status/{code}` 路径解析支持自定义状态码
- **HTTP scope 扩展**：在 `run-app.json` 中添加 `http://localhost:3003/*`
- **测试迁移**：6 个不依赖 httpbin.org 特有响应结构的测试迁移到 localhost:3003（GET/POST/PUT/DELETE/custom headers/error handling）
- **外网测试加固**：2 个必须走 httpbin.org 的测试（HTTPS/rustls-tls 和 JSON parse）添加 `retryFetch` 辅助函数（最多 3 次重试，间隔 1s，connectTimeout: 3000ms）

## Capabilities

### New Capabilities
- `http-test-stability`: HTTP 测试稳定性提升，通过本地 echo server 消除外网依赖

### Modified Capabilities
（无现有 capability 的需求变更）

## Impact

- **src-tauri/src/lib.rs** (Rust)：echo server 移除 desktop 限制 + 增加 `/status/{code}` 路径解析
- **src-tauri/capabilities/run-app.json**：scope 添加 localhost:3003
- **src/lib/tests/plugins.ts** (TypeScript)：6 个测试迁移 + 新增 retryFetch + 2 个测试加固
- **Desktop 平台**：行为不变，echo server 仍正常启动
- **OHOS mobile**：echo server 新增可用，6 个测试不再依赖外网
- **预期效果**：6/8 测试从 5s 超时风险降为 <100ms 稳定通过

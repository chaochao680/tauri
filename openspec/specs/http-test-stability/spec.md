# http-test-stability Specification

## Purpose
TBD - created by archiving change p6-http-test-stability. Update Purpose after archive.
## Requirements
### Requirement: HTTP 测试使用本地 echo server

6 个 HTTP 测试用例（GET/POST/PUT/DELETE/custom headers/error handling）SHALL 使用本地 echo server（`http://localhost:3003`）替代外部服务 httpbin.org，消除外网依赖导致的 timeout 和 503 假失败。

#### Scenario: GET 请求本地 echo server
- **WHEN** 执行 `fetch (GET)` 测试
- **THEN** SHALL 请求 `http://localhost:3003/get`
- **THEN** 响应状态 SHALL 为 200
- **测试分类**: `auto`

#### Scenario: POST 请求体验证
- **WHEN** 执行 `fetch (POST)` 测试，发送 body `{"test":"post-data"}`
- **THEN** SHALL 请求 `http://localhost:3003/post`
- **THEN** echo server SHALL 原样返回请求体
- **THEN** 响应文本 SHALL 等于发送的 body
- **测试分类**: `auto`

#### Scenario: PUT 请求体验证
- **WHEN** 执行 `fetch (PUT)` 测试，发送 body `{"update":"put-data"}`
- **THEN** SHALL 请求 `http://localhost:3003/put`
- **THEN** echo server SHALL 原样返回请求体
- **THEN** 响应文本 SHALL 等于发送的 body
- **测试分类**: `auto`

#### Scenario: DELETE 请求
- **WHEN** 执行 `fetch (DELETE)` 测试
- **THEN** SHALL 请求 `http://localhost:3003/delete`
- **THEN** 响应状态 SHALL 为 200
- **测试分类**: `auto`

#### Scenario: 自定义 headers 传递
- **WHEN** 执行 `fetch (custom headers)` 测试，设置 `X-Custom-Header: test-value-123` 和 `X-Another-Header: another-value`
- **THEN** SHALL 请求 `http://localhost:3003/headers`
- **THEN** echo server SHALL 将请求头作为响应头回显
- **THEN** 响应头中 SHALL 包含 `X-Custom-Header: test-value-123`
- **THEN** 响应头中 SHALL 包含 `X-Another-Header: another-value`
- **测试分类**: `auto`

#### Scenario: 错误状态码处理
- **WHEN** 执行 `fetch (error handling)` 测试
- **THEN** SHALL 请求 `http://localhost:3003/status/404`
- **THEN** echo server SHALL 返回 404 状态码
- **THEN** 响应 `status` SHALL 为 404
- **THEN** 响应 `ok` SHALL 为 false
- **测试分类**: `auto`

### Requirement: 外网测试具备 retry 容错

2 个必须使用 httpbin.org 的测试（HTTPS/rustls-tls、JSON parse）SHALL 使用 `retryFetch` 辅助函数，具备连接超时和自动重试能力。

#### Scenario: HTTPS 测试 retry
- **WHEN** 执行 `fetch (HTTPS/rustls-tls)` 测试
- **THEN** SHALL 使用 `retryFetch` 请求 `https://httpbin.org/get`
- **THEN** `retryFetch` SHALL 设置 `connectTimeout: 3000`
- **THEN** 遇到 503 或网络错误时 SHALL 自动重试，最多 3 次，间隔 1s
- **测试分类**: `auto`

#### Scenario: JSON parse 测试 retry
- **WHEN** 执行 `fetch (JSON parse)` 测试
- **THEN** SHALL 使用 `retryFetch` 请求 `https://httpbin.org/json`
- **THEN** 重试行为与 HTTPS 测试一致
- **THEN** 响应 JSON SHALL 包含 `slideshow` 属性
- **测试分类**: `auto`

### Requirement: Echo server 全平台可用

`src-tauri/src/lib.rs` 中的 tiny_http echo server SHALL 在所有平台（含 OHOS mobile）上启动，不受 `#[cfg(desktop)]` 限制。

#### Scenario: OHOS mobile 上 echo server 可用
- **WHEN** 应用在 OHOS mobile 设备上启动
- **THEN** echo server SHALL 在 `localhost:3003` 上监听
- **THEN** HTTP 测试 SHALL 能够成功连接 localhost:3003
- **测试分类**: `manual`（需设备端验证）

### Requirement: Echo server 支持自定义状态码

Echo server SHALL 解析请求 URL 中的 `/status/{code}` 路径，返回指定的 HTTP 状态码。

#### Scenario: /status/404 返回 404
- **WHEN** 请求 `http://localhost:3003/status/404`
- **THEN** echo server SHALL 返回 HTTP 404 状态码
- **测试分类**: `auto`

#### Scenario: /status/200 返回 200
- **WHEN** 请求 `http://localhost:3003/status/200`
- **THEN** echo server SHALL 返回 HTTP 200 状态码
- **测试分类**: `auto`

#### Scenario: 非 /status/ 路径默认 200
- **WHEN** 请求 `http://localhost:3003/any-other-path`
- **THEN** echo server SHALL 返回 HTTP 200 状态码（默认行为不变）
- **测试分类**: `auto`

---


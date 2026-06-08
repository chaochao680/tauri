## Context

examples/api 的 HTTP 自动测试位于 `src/lib/tests/plugins.ts`，共 8 个 `category: 'auto'` 用例：

| # | 测试 | 方法 | 目标 | 依赖 httpbin.org 的原因 |
|---|------|------|------|------------------------|
| 1 | GET | GET | httpbin.org/get | 验证 data.url 回显 |
| 2 | POST | POST | httpbin.org/post | 验证 data.json 回显 |
| 3 | PUT | PUT | httpbin.org/put | 验证 data.json 回显 |
| 4 | DELETE | DELETE | httpbin.org/delete | 仅验证 status=200 |
| 5 | custom headers | GET | httpbin.org/headers | 验证 data.headers 回显 |
| 6 | JSON parse | GET | httpbin.org/json | 需要 slideshow 结构 |
| 7 | HTTPS/rustls-tls | GET | httpbin.org/get | 需要真实 TLS 握手 |
| 8 | error handling | GET | httpbin.org/status/404 | 需要服务端返回 404 |

**已有基础设施：** `lib.rs` 中有一个 tiny_http echo server（`localhost:3003`），回显请求体和请求头：
```rust
#[cfg(desktop)]  // ← 仅 desktop 启动
std::thread::spawn(|| {
    let server = tiny_http::Server::http("localhost:3003").unwrap();
    loop {
        // echo: 原样返回请求体 + 请求头，固定 200
    }
});
```

**问题分析：** 6 个测试（GET/POST/PUT/DELETE/custom headers/error handling）验证的是**客户端行为**（HTTP 方法、请求体发送、自定义 headers 传递、状态码处理），完全可以由本地 echo server 覆盖。只有 2 个测试（HTTPS/rustls-tls、JSON parse）必须使用真实外网端点。

## Goals / Non-Goals

**Goals:**
- 6 个测试迁移到 localhost:3003，消除外网依赖
- Echo server 全平台可用（含 OHOS mobile）
- Echo server 支持自定义状态码（`/status/{code}` 路径）
- 2 个外网测试具备 retry 和 connectTimeout 容错
- 不引入新的 Rust 依赖

**Non-Goals:**
- 不替换 tiny_http 为其他 HTTP server（如 axum/warp）
- 不搭建 HTTPS 本地服务器（TLS 测试保留 httpbin.org）
- 不修改 test-runner.ts 的超时机制（5s 超时足够本地测试）
- 不修改其他非 HTTP 测试

## Decisions

### Decision 1: 迁移 6 个测试到 localhost:3003（而非全部 8 个）

**选择：** 仅迁移不依赖 httpbin.org 特有响应结构的 6 个测试。

| 测试 | 迁移到 localhost | 断言调整 |
|------|----------------|----------|
| GET | ✅ | 仅验证 status=200（echo server 无 data.url 回显） |
| POST | ✅ | 验证 status=200 + body 原样返回（`resp.text()` 对比） |
| PUT | ✅ | 同 POST |
| DELETE | ✅ | 仅验证 status=200 |
| custom headers | ✅ | 验证 echo server 回显的请求头（`resp.headers.get()`） |
| error handling | ✅ | 需要 echo server 支持 `/status/404` 路径 |
| JSON parse | ❌ 保留 httpbin | 需要 slideshow JSON 结构，echo server 无法提供 |
| HTTPS/rustls-tls | ❌ 保留 httpbin | 需要真实 TLS 握手验证 |

**理由：**
- 本地迁移覆盖 75% 测试，效果显著
- JSON parse 和 HTTPS 测试的核心价值在于验证真实网络交互，不适合本地化
- 与全部本地化相比，改动最小且风险最低

### Decision 2: 增强 echo server 支持 path-based status code

**选择：** 在 echo server 的请求处理中解析 URL path，`/status/{code}` 返回指定状态码：

```rust
let path = request.url().to_string();
let status = if let Some(code_str) = path.strip_prefix("/status/") {
    code_str.parse::<u16>().unwrap_or(200)
} else {
    200
};
```

**理由：**
- 改动极小（3 行代码），不影响已有 echo 行为
- 使 error handling 测试可以完全本地化
- tiny_http 的 `StatusCode` 接受任意 u16，无需枚举映射

### Decision 3: 移除 `#[cfg(desktop)]` 使 echo server 全平台启动

**选择：** 移除 echo server 的 `#[cfg(desktop)]` 门控，使其在 OHOS mobile/desktop 上也启动。

**理由：**
- `tiny_http` 已是无条件依赖（Cargo.toml 无 cfg gate），纯 Rust 实现无平台限制
- OHOS mobile 上 localhost:3003 端口绑定无技术障碍
- 移除限制后 echo server 在所有平台一致可用
- 不影响其他平台的已有行为（Desktop 上行为完全不变）

### Decision 4: 外网测试使用 retryFetch 辅助函数

**选择：** 新增 `retryFetch` 函数，为 2 个外网测试提供容错：

```typescript
async function retryFetch(url: string, init, maxRetries = 3): Promise<Response> {
  const { fetch } = await import('@tauri-apps/plugin-http');
  const opts = { ...init, connectTimeout: 3000 };
  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      const resp = await fetch(url, opts);
      if (resp.status !== 503 || attempt === maxRetries) return resp;
    } catch (e) { /* retry */ }
    if (attempt < maxRetries) await new Promise(r => setTimeout(r, 1000));
  }
  throw lastError;
}
```

**理由：**
- `connectTimeout: 3000` 让连接超时快速失败，避免等满 5s
- 503 自动重试 3 次，间隔 1s，应对 httpbin.org 瞬时限流
- 仅在需要外网的 2 个测试中使用，不增加其他测试的复杂度
- 函数签名与标准 fetch 兼容，测试代码改动最小

### Decision 5: 断言适配 — echo server 回显机制

**关键差异：** httpbin.org 返回结构化 JSON（`data.json.test`），echo server 原样返回请求体。

**适配方案：**
- POST/PUT 断言从 `data.json.test === 'post-data'` 改为 `resp.text() === body`（验证请求体原样返回）
- custom headers 断言从 `data.headers['X-Custom-Header']` 改为 `resp.headers.get('X-Custom-Header')`（echo server 将请求头作为响应头返回）
- GET 断言从 `data.url === '...'` 改为仅验证 `status === 200`（echo server 无 URL 回显）

## cfg 隔离策略

| 修改 | 隔离方式 | 说明 |
|------|---------|------|
| echo server 移除 `#[cfg(desktop)]` | 移除限制 → 全平台 | tiny_http 纯 Rust，无平台限制 |
| `/status/{code}` 路径解析 | 无新 cfg | 通用逻辑，不需要平台隔离 |
| 测试迁移到 localhost | 无 cfg | 前端 JS 测试代码，不涉及 cfg |

**结论：** 不添加任何新 cfg gate。

## Skill 校验结果

| 检查项 | 来源 | 结果 |
|--------|------|------|
| cfg 隔离规则 | ohos-constraints §1.1 | ✅ 移除 desktop 限制，不新增 OHOS cfg |
| OHOS target_os 是 "linux" | ohos-constraints §5.4 | ✅ tiny_http 无条件依赖，无 Linux 排除问题 |
| 不影响其他平台 | review-checklist B1-B3 | ✅ Desktop echo server 行为不变 |
| openharmony-ability 桥接 | review-checklist F1-F2 | ✅ 不涉及系统能力调用 |
| NAPI/TSFN | review-checklist C | ✅ 不涉及 |
| 线程模型 | review-checklist D | ✅ echo server 在独立线程，与主线程无交互 |
| ArkTS 框架 | review-checklist E | ✅ 不涉及 |
| API 版本隔离 | ohos-version-isolation | ✅ 不涉及高版本 API |
| 前端测试规范 | frontend-api-testing | ✅ 遵循 auto 分类 + 动态 import + assert |

## Risks / Trade-offs

- **[OHOS mobile 端口绑定]** 假设 localhost:3003 在 OHOS mobile 上可正常绑定 → tiny_http 使用标准 TCP socket，OHOS 基于 Linux 内核，端口绑定无技术障碍
- **[echo server 线程生命周期]** echo server 在独立线程中无限循环，应用退出时线程被 OS 回收 → 与当前 desktop 行为一致，无新增风险
- **[retry 增加最坏情况耗时]** 3 次重试 + 1s 间隔 = 最坏 ~4s → 仍在 test-runner 5s 超时内，且仅影响 2 个外网测试
- **[断言语义变化]** POST/PUT 从验证 JSON 解析改为验证文本回显 → 仍然验证了请求体正确发送的核心功能

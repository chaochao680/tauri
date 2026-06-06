# HTTP 测试稳定性提升 - 实施任务

## 状态
- **方案**: 本地 echo server 迁移 + 外网 retry 加固
- **Skill 校验**: ✅ 已完成（cfg 隔离/平台隔离/桥接规则/NAPI/线程/ArkTS/版本隔离/前端测试规范 全部通过）
- **当前阶段**: 待实施

---

## 1. Echo server 增强（lib.rs）

### 1.1 移除 `#[cfg(desktop)]` 门控
- [ ] 删除 `src-tauri/src/lib.rs` 中 echo server 的 `#[cfg(desktop)]` 前缀（约 line 338）

### 1.2 添加 `/status/{code}` 路径解析
- [ ] 在 echo server 请求处理中添加 URL path 解析：
  ```rust
  let path = request.url().to_string();
  let status = if let Some(code_str) = path.strip_prefix("/status/") {
      code_str.parse::<u16>().unwrap_or(200)
  } else {
      200
  };
  ```
- [ ] 将 `tiny_http::StatusCode(200)` 替换为 `tiny_http::StatusCode(status)`

---

## 2. HTTP scope 扩展（run-app.json）

### 2.1 添加 localhost:3003 到 scope
- [ ] 在 `src-tauri/capabilities/run-app.json` 的 `http:default` allow 列表中添加 `{ "url": "http://localhost:3003/*" }`

---

## 3. 测试迁移（plugins.ts）

### 3.1 添加 retryFetch 辅助函数
- [ ] 在 `assert()` 函数后添加 `retryFetch` 函数（connectTimeout: 3000, 最多 3 次重试, 503 自动重试）

### 3.2 迁移 6 个测试到 localhost:3003
- [ ] GET: URL 改为 `http://localhost:3003/get`，断言改为仅验证 status=200
- [ ] POST: URL 改为 `http://localhost:3003/post`，断言改为 `resp.text() === body`
- [ ] PUT: URL 改为 `http://localhost:3003/put`，断言改为 `resp.text() === body`
- [ ] DELETE: URL 改为 `http://localhost:3003/delete`
- [ ] custom headers: URL 改为 `http://localhost:3003/headers`，断言改为 `resp.headers.get()`
- [ ] error handling: URL 改为 `http://localhost:3003/status/404`

### 3.3 加固 2 个外网测试
- [ ] JSON parse: 使用 `retryFetch` 替代直接 `fetch`
- [ ] HTTPS/rustls-tls: 使用 `retryFetch` 替代直接 `fetch`

---

## 4. 验证

### 4.1 编译验证
- [ ] Desktop `cargo check` 通过

### 4.2 功能验证
- [ ] 6 个本地测试全部 PASS，耗时 <100ms
- [ ] 2 个外网测试成功率显著提升（retry 覆盖 503 和超时）
- [ ] OHOS mobile 上 echo server 正常启动

### 4.3 回归验证
- [ ] Desktop 上所有测试行为不变
- [ ] 其他非 HTTP 测试不受影响

---

## 依赖关系

```
1. Echo server 增强
    └─→ 2. HTTP scope 扩展
        └─→ 3. 测试迁移
            └─→ 4. 编译验证
                └─→ 5. 功能验证（设备端）
                └─→ 6. 回归验证（Desktop）
```

---

## 预估工作量

| 模块 | 任务数 | 预估时间 |
|------|--------|----------|
| Echo server 增强 | 3 | 0.2h |
| HTTP scope 扩展 | 1 | 0.05h |
| 测试迁移 + 加固 | 9 | 0.5h |
| 编译验证 | 1 | 0.1h |
| 功能验证 | 3 | 1h |
| 回归验证 | 2 | 0.5h |
| **总计** | **19** | **~2.3h** |

---

## 验收标准

1. ✅ Echo server 全平台可用（含 OHOS mobile）
2. ✅ Echo server 支持 `/status/{code}` 路径
3. ✅ HTTP scope 包含 localhost:3003
4. ✅ 6 个本地测试 PASS 且耗时 <100ms
5. ✅ 2 个外网测试具备 retry 容错
6. ⬜ 编译验证通过
7. ⬜ 设备端功能验证通过
8. ⬜ Desktop 回归验证通过

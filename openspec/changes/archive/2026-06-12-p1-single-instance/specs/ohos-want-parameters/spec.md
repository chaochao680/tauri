## ADDED Requirements

### Requirement: ArkTS 层提取并传递 want.parameters
`NativeAbility.ets` 的 `onNewWant` 回调 SHALL 提取 `want.parameters`，使用 `JSON.stringify()` 序列化为 JSON 字符串，并通过 lifecycle 回调传递给 NAPI 层。

#### Scenario: 正常 onNewWant 携带 parameters
- **WHEN** OHOS 系统调用 `onNewWant(want)` 且 `want.parameters` 包含 `{key1: "value1", key2: 100}`
- **THEN** ArkTS 层 SHALL 将 `want.parameters` 序列化为 `{"key1":"value1","key2":100}` 并传递给 NAPI `on_new_want` 函数

#### Scenario: onNewWant 无 parameters
- **WHEN** OHOS 系统调用 `onNewWant(want)` 且 `want.parameters` 为 undefined 或空
- **THEN** ArkTS 层 SHALL 传递空 JSON 对象字符串 `"{}"`

#### Scenario: onNewWant 同时携带 uri 和 parameters
- **WHEN** `want.uri` 为 `"tauri://app/page"` 且 `want.parameters` 为 `{source: "widget"}`
- **THEN** ArkTS 层 SHALL 同时传递 uri 和 parameters JSON 字符串给 NAPI

### Requirement: NAPI 桥接函数接收含参数的 Object
`on_new_want` NAPI 闭包 SHALL 接收一个 `Object` 参数，包含 `uri: String` 和 `parametersJson: String` 两个命名属性。`WindowStageEventCallback.on_new_want` 的类型 SHALL 从 `Function<'a, String, ()>` 更新为 `Function<'a, Object<'a>, ()>`。

#### Scenario: NAPI 接收完整参数对象
- **WHEN** ArkTS 调用 `onNewWant({ uri: "tauri://test", parametersJson: "{\"key\":\"val\"}" })`
- **THEN** Rust 侧 NAPI 闭包 SHALL 通过 `get_named_property` 提取 `uri = "tauri://test"` 和 `parametersJson = "{\"key\":\"val\"}"`

### Requirement: Rust 侧存储并可读取 want.parameters
`openharmony-ability` SHALL 提供全局存储机制，保存最新的 `want.parameters` JSON 字符串，并通过公共 getter 函数暴露给其他 crate。

#### Scenario: 存储并读取参数
- **WHEN** `onNewWant` 携带 `parameters_json = "{\"source\":\"notification\"}"`
- **THEN** `take_want_parameters()` SHALL 返回 `"{\"source\":\"notification\"}"`

#### Scenario: take 语义 — 读取后清空
- **WHEN** 调用 `take_want_parameters()` 一次后再次调用
- **THEN** 第二次调用 SHALL 返回空字符串 `""`

#### Scenario: 无参数时读取
- **WHEN** 未发生过 `onNewWant` 事件时调用 `take_want_parameters()`
- **THEN** SHALL 返回空字符串 `""`

### Requirement: TypeScript 接口更新
`WindowStageEventCallback.onNewWant` 的类型签名 SHALL 从 `(uri: string) => void` 扩展为 `(data: Record<string, string>) => void`。传递对象包含 `uri` 和 `parametersJson` 两个字段。

#### Scenario: 新接口兼容性
- **WHEN** lifecycle module 注册 `onNewWant` 回调
- **THEN** 回调 SHALL 接收一个 `Record<string, string>` 对象参数，含 `uri` 和 `parametersJson` 字段

### Requirement: 线程安全存储
参数存储机制 SHALL 保证线程安全，使用 `Mutex<String>` 保护。存储和读取操作 MUST 不阻塞主线程。

#### Scenario: 并发读写安全
- **WHEN** ArkTS 主线程写入参数的同时 Rust 工作线程读取
- **THEN** 不会发生数据竞态或 panic

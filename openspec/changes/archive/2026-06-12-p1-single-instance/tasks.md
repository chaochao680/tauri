## 1. TypeScript 接口扩展

- [x] 1.1 在 `openharmony-ability/native_ability/src/main/ets/ability/type.ets` 中，将 `WindowStageEventCallback.onNewWant` 签名从 `(uri: string) => void` 改为 `(data: Record<string, string>) => void`

## 2. ArkTS 层提取并传递 want.parameters

- [x] 2.1 在 `openharmony-ability/native_ability/src/main/ets/ability/NativeAbility.ets` 的 `onNewWant` 方法中，添加 `const parametersJson = JSON.stringify(want.parameters ?? {})` 提取参数
- [x] 2.2 修改 `forEachLifecycle` 回调，传递对象 `{ uri, parametersJson }` 给 `lifecycle.windowStageEventCallback.onNewWant({ uri, parametersJson })`

## 3. NAPI 桥接层扩展

- [x] 3.1 在 `openharmony-ability/crates/ability/src/lifecycle.rs` 的 `WindowStageEventCallback` struct 中，将 `on_new_want` 类型从 `Function<'a, String, ()>` 改为 `Function<'a, Object<'a>, ()>`
- [x] 3.2 在 `on_new_want` 闭包中，使用 `ctx.first_arg::<Object>()` 获取对象，再通过 `get_named_property::<String>("uri")` 和 `get_named_property::<String>("parametersJson")` 提取字段
- [x] 3.3 调用 `crate::app::store_want_parameters(&parameters_json)` 存储参数

## 4. Rust 侧参数存储与 Getter

- [x] 4.1 在 `openharmony-ability/crates/ability/src/app.rs` 中添加 `static WANT_PARAMETERS: Mutex<String> = Mutex::new(String::new())`
- [x] 4.2 实现 `pub(crate) fn store_want_parameters(json: &str)` 存储函数
- [x] 4.3 实现 `pub fn take_want_parameters() -> String` 公共 getter 函数（take 语义：读取后清空）
- [x] 4.4 确认 `use std::sync::Mutex` 导入正确，无跨线程阻塞风险

## 5. 编译验证

- [x] 5.1 在 host target 下 `cargo check -p openharmony-ability` 通过
- [x] 5.2 OHOS target 编译验证留到 verify 阶段（设备端构建）

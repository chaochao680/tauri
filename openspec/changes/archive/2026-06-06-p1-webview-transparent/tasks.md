# Phase 1: WebView 透明背景 - 实施任务

## 状态
- **方案**: B（完整方案）
- **API 验证**: ✅ 已完成（2026-06-04，含 arkts-helper 官方文档验证）
- **代码审查**: ✅ 已完成（2026-06-04，大部分任务已在源码中实现）
- **Web 引擎 renderMode 验证**: ⚠️ 未完成（实施前必须先验证 Task 5.3）
- **set_background_color 线程安全确认**: ⚠️ 未完成（Task 5.4 子任务待确认）
- **当前阶段**: 仅剩 1 个代码任务 + 验证 + 清理

---

## 1. ArkTS 端透明逻辑与类型修正

### 1.1 修复 ArkHelper.ets 的 transparent 优先级逻辑
- [x] ~~修改 `ArkHelper.ets` 优先级逻辑~~ — ✅ 已在源码中修复（line 230, 335 均已改为 `if (data?.transparent)`）

### 1.2 修正 `setBackgroundColor` 参数类型
- [x] ~~修改 `Utils.ets` 声明为 `(color: number) => void`~~ — ✅ 已完成（4 处全部更新）
- [x] ~~修改 `DefaultWebview.ets` WebviewController 扩展声明~~ — ✅ 已完成

---

## 2. ArkTS 颜色格式转换

### 2.1 实现颜色转换工具函数（含类型守卫）
- [x] ~~在 `Utils.ets` 中实现 `convertRRGGBBAAtoAARRGGBB`~~ — ✅ 已实现（Utils.ets:18），且签名已扩展为 `(color: string | Color | number | undefined)`

### 2.2 验证颜色格式边界情况
- [ ] 验证函数对 `number`（`0xAARRGGBB`）输入的透传正确性
- [ ] 验证 `undefined` 输入的透传正确性
- [ ] 验证 `Color.Transparent` 枚举的透传正确性

> **注意**：Rust 端已在 `wry/src/ohos/mod.rs:66-68` 将 RGBA 转为 `u32`，NAPI 传入的 `backgroundColor` 永远是 `number | Color | undefined`，**字符串分支（`#RRGGBBAA` → `#AARRGGBB`）是死代码**。函数仍应保留作为防御性编程。

---

## 3. ArkTS Web 组件透明实现

### 3.1 WebBuilder 和 EmbeddedWebBuilder 中调用转换函数 ⚠️ 关键剩余任务
- [ ] 修改 `DefaultWebview.ets` 的 `WebBuilder`（约 line 56）：
  ```typescript
  // 修改前：
  .backgroundColor(data?.style?.backgroundColor)
  // 修改后：
  .backgroundColor(convertRRGGBBAAtoAARRGGBB(data?.style?.backgroundColor))
  ```
  **注意**：函数已导入（line 7）但从未调用。此任务仅添加函数调用包装。

- [ ] 修改 `DefaultWebview.ets` 的 `EmbeddedWebBuilder`（约 line 149）：
  - 应用与 `WebBuilder` 完全相同的修改

### 3.2 父容器全层级透明支持（防御性设置）
- [x] ~~修改 `DefaultXComponent.ets` 的 Stack/Row/Column 容器~~ — ✅ 已实现（3 层容器均已设置 `.backgroundColor(Color.Transparent)`）

### 3.3 创建时 vs 运行时背景色职责划分
- [x] 创建时：ArkHelper.ets 消费 transparent + WebBuilder 声明式属性（Task 1.1 + 3.1）
- [x] 运行时：controller.setBackgroundColor() 命令式 API（Task 4 已实现）

---

## 4. Rust 端颜色格式转换

### 4.1 wry 端 `set_background_color` 实现
- [x] ~~修改 `wry/src/ohos/mod.rs` 的 `set_background_color`~~ — ✅ 已实现（RGBA → `0xAARRGGBB` 转换完整）

### 4.2 openharmony-ability 端 NAPI 桥接层修改
- [x] ~~修改 `webview.rs` 参数类型为 `u32`~~ — ✅ 已完成（`Function<'_, u32, ()>`）
- [x] ~~修改 ArkHelper.ets 猴子补丁为 `(color: number)`~~ — ✅ 已完成（line 271, 345）
- [x] ~~扩展 `WebviewStyle.backgroundColor` 类型~~ — ✅ 已完成（`Color | number`，无需 `string`）

---

## 5. WebViewAttributes.transparent 传递验证

### 5.1 验证传递链路
- [ ] 确认 `tauri` → `tauri-runtime-wry` → `wry` → `openharmony-ability` 的 `transparent` 字段完整传递
- [ ] 确认 `transparent` 被正确传入 `WebviewInitData`

### 5.2 优先级逻辑验证
- [ ] 测试 `transparent: true` 时忽略 `background_color`
- [ ] 测试 `transparent: false` 时使用 `background_color`
- [ ] 测试两者都未设置时的默认行为

### 5.3 Web 引擎透明度验证（实施前必须）
- [ ] 在 OHOS 设备上验证：默认异步渲染模式下 `.backgroundColor(Color.Transparent)` 是否透明
- [ ] 如不透，测试 `renderMode: RenderMode.SYNC_RENDER`
- [ ] 如需 SYNC_RENDER，评估性能影响

### 5.4 set_background_color 线程安全确认
- [ ] 确认 `wry` 调用 `set_background_color` 时的线程上下文
- [ ] 确认 `get_main_thread_env()` 在 wry 调用上下文中返回 `Some(env)`
- [ ] 如不是主线程，需通过 TSFN 调度

---

## 6. 代码清理（审查发现的新增任务）

### 6.1 debug 日志降级
- [ ] 将 `openharmony-ability/crates/ability/src/helper/webview.rs` 中 `set_background_color` 方法内的 6 处 `log::error!` 降级为 `log::debug!`（当前用于正常路径追踪）
- [ ] 将 `wry/src/ohos/mod.rs` 中 `set_background_color` 方法的 `eprintln!` 替换为 `log::debug!`

### 6.2 demo_native 类型修复
- [ ] 修改 `openharmony-ability/rust_example/demo_native/src/lib.rs:119` 的 `set_background_color`：
  - 参数从 `color: String` 改为 `color: u32`
  - NAPI Function 泛型从 `Function<'_, String, ()>` 改为 `Function<'_, u32, ()>`
  - 与 ArkTS 端 `number` 类型对齐，消除运行时类型不匹配

---

## 7. 集成测试

### 7.1 基础透明功能测试
- [ ] 测试 `transparent: true` 创建 WebView，确认背景透明
- [ ] 测试 HTML `body { background-color: transparent; }` 能穿透到下层
- [ ] 测试 `transparent: false`（默认）时背景不透明

### 7.2 背景色设置测试
- [ ] 测试 `background_color` 创建 WebView，确认颜色正确
- [ ] 测试 `background_color: null` 时使用默认背景

### 7.3 运行时动态更新测试
- [ ] 测试 `set_background_color(0x80000000)` 运行时切换为半透明黑
- [ ] 测试 `set_background_color(0x00000000)` 运行时切换为完全透明
- [ ] 验证动态更新后立即生效（无需刷新）

---

## 依赖关系

```
⚠️ 5.3 Web 引擎透明度验证（实施前必须先确认）
⚠️ 5.4 set_background_color 线程安全确认
    └─→ 3.1 WebBuilder 调用转换函数（唯一代码任务）
    └─→ 6. 代码清理
    └─→ 7. 集成测试
```

---

## 预估工作量

| 模块 | 任务数 | 预估时间 |
|------|--------|----------|
| 代码任务（3.1 WebBuilder 调用） | 2 | 0.2h |
| 代码清理（6.1 日志 + 6.2 demo） | 3 | 0.3h |
| 验证（5.1-5.4） | 10 | 2h |
| 集成测试（7） | 7 | 1.5h |
| **总计** | **22** | **~4h** |

> **注**：原 40 个任务中大部分已在源码中实现，实际剩余工作量大幅减少。

---

## 验收标准

1. ✅ `transparent: true` 优先于 `background_color`（已修复）
2. ✅ WebView 支持 `transparent: true` 创建透明背景（待 Task 3.1 接入转换函数）
3. ✅ 运行时 `set_background_color` 可动态更新背景色（已实现）
4. ✅ 颜色格式转换函数被 WebBuilder/EmbeddedWebBuilder 正确调用（Task 3.1）
5. ✅ 父容器透明不遮挡（已实现）
6. ✅ debug 日志已清理、demo_native 已修复
7. ✅ 所有集成测试通过

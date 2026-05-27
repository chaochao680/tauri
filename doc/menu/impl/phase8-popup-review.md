# Phase 8: popup() 设计方案审计

> 审计对象：`phase8-popup-root-cause-and-fix.md`
> 审计依据：muda OHOS 序列化源码、openharmony-ability menu 实现、tray statusbar 调试经验
> 日期：2026-05-20

## 结论

核心修复方案（string id 锚点 + anchorPosition 定位）正确可行。但有 6 个问题需要在实施前解决或确认。

---

## 问题 1：`close` vs `destroyWindow` 类型不一致

**严重程度**：⚠️ 中 — predefined CloseWindow 点击无效果

**现状**：
- muda 序列化 `PredefinedMenuItemType::CloseWindow` 为 `"close"`（`muda/src/platform_impl/ohos/mod.rs:275`）
- `PredefinedActionExecutor.execute()` 的 switch 中只有 `'destroyWindow'` 分支，没有 `'close'`
- 点击 CloseWindow 菜单项时，executor 走到 default 分支，静默忽略

**修复建议**：在 `PredefinedActionExecutor` 中添加：
```typescript
case 'close':
  await this.win?.destroyWindow();
  break;
```

同时在 `menu_types.ets` 的 `PredefinedType` 联合类型中补充 `'close'`。

---

## 问题 2：`&` 助记符未去除

**严重程度**：⚠️ 中 — 菜单文本显示 `&Minimize` 而非 `Minimize`

**现状**：
- `PredefinedMenuItemType::text()` 返回带 `&` 的文本（如 `"&Minimize"`、`"Ma&ximize"`）
- muda `to_menu_item_data()` 直接用 `self.text.clone()` 作为 text 字段
- Builder 中 `item.text ?? item.predefinedType ?? ''` 直接渲染，未 strip

**修复建议**（二选一）：

方案 A（推荐）：在 muda 序列化时去除
```rust
text: Some(self.text.replace("&", "")),
```

方案 B：在 ArkTS Builder 中去除
```typescript
const displayText = (item.text ?? '').replace(/&/g, '');
```

方案 A 更好 — 一次性解决，所有消费者（popup menu、tray menu）都受益。注意 tray phase7 已经在 `tray-icon` 侧做了 `strip_mnemonics`，但 muda 源头没改，menu 侧需要独立处理。

---

## 问题 3：`anchorPosition` API 版本风险

**严重程度**：⚠️ 中 — 低版本设备上菜单位置错误

**现状**：
- 设计文档声明 `anchorPosition` 需要 API 20+
- 项目 `compatibleSdkVersion` 为 `"5.0.0(12)"`（API 12）
- 如果设备运行 API 18-19，`anchorPosition` 被忽略，菜单位置由 `placement` 决定

**需要确认**：
1. 实际目标设备的 API 版本是否 ≥ 20？
2. 编译 SDK 版本是否 ≥ 20？（决定 `anchorPosition` 是否能通过编译）

**如果需要兼容 API 18-19**：
```typescript
const options: MenuOptions = {};
if (canUseAnchorPosition()) {
  options.anchorPosition = { x: x ?? 0, y: y ?? 0 };
} else {
  // offset 方案：相对于锚点左上角的偏移
  // 但 offset 语义依赖 placement 方向，不如 anchorPosition 直观
  options.offset = { x: x ?? 0, y: y ?? 0 };
  options.placement = Placement.TopLeft;
}
```

**如果确认只跑 API 20+**：无需修改，在文档中明确标注即可。

---

## 问题 4：菜单事件回传未接通

**严重程度**：⚠️ 中 — popup 能弹出但点击无反馈

**现状**：
- `TauriMenuManager.handleItemClick()` 中 `this.onMenuClick` 始终为 `null`
- 没有调用 NAPI 的 `emit_menu_event(item_id)` 通知 Rust
- 用户点击 Normal/Check/Icon item 后，Rust 侧 `MenuEvent` listener 收不到事件

**设计文档标记为"前置 bug，不在本阶段范围"**。

**建议**：本阶段一并修复。理由：
1. popup 弹出但无事件 = 功能不完整，用户无法验证 popup 是否真正工作
2. 修复量小 — 只需在 `handleItemClick` 中调用已有的 NAPI 函数
3. 不修复的话，后续验证 popup 时会产生困惑（"菜单弹出了但点击没反应"）

**修复方向**：
```typescript
import { emit_menu_event } from 'libnative_ability.so';

handleItemClick(item: TauriMenuItemData): void {
  if (item.type === 'predefined' && item.predefinedType) {
    this.executor.execute(item.predefinedType);
  } else {
    emit_menu_event(item.id);
  }
}
```

需要确认 `emit_menu_event` 是否已在 NAPI 层导出。如果没有，需要在 `openharmony-ability/crates/ability/src/menu/mod.rs` 中添加。

---

## 问题 5：`eval_with_callback` 可行性待验证

**严重程度**：⚠️ 低 — 仅影响无坐标 popup 场景

**设计文档方案**：当 `popup()` 不传坐标时，通过 `webview.eval_with_callback` 从 JS 读取 `__TAURI_MENU_LAST_POINTER__`。

**潜在风险**：
1. 从 tray 经验看，某些 JS bridge 调用在特定上下文中会静默失败
2. `eval_with_callback` 是异步的，需要 oneshot channel 等待 — 如果 WebView 线程被阻塞，可能死锁
3. 需要确认 OHOS ArkWeb 的 `runJavaScript` 在 popup command 的 async 上下文中能正常返回

**降级方案**：如果 `eval_with_callback` 不可靠，可以改为：
- Rust 侧维护一个 `AtomicF64` 对的 last_pointer 坐标
- 通过 IPC command（`__internal_update_pointer`）从 JS 侧主动推送
- popup 时直接读取 atomic 值，无需异步等待

**建议**：先按设计文档方案实施，在设备上验证。如果失败再切换到 atomic 方案。

---

## 问题 6：id 冲突风险

**严重程度**：⚠️ 低

**现状**：设计文档使用 `"__tauri_menu_anchor__"` 作为锚点 id。

**风险**：如果用户代码中恰好使用了相同 id，`openMenu` 可能定位到错误组件。

**建议**：使用更长的前缀降低冲突概率：`"__tauri_internal_menu_popup_anchor__"`。或者在文档中明确标注这是保留 id。

实际风险极低 — 用户代码在 WebView 内部，不会设置 ArkUI 组件 id。

---

## 已确认正确的部分

| 设计点 | 验证结果 |
|--------|---------|
| TargetInfo number → string id 修复 | ✅ 根因分析正确，修复方案合理 |
| MainPage 根 Row 加 `.id()` 作为锚点 | ✅ 不影响布局和事件，位置正确 |
| Separator 判断逻辑错误 | ✅ muda 确实发送 `type: "predefined"` + `predefinedType: "separator"` |
| 不需要修改 muda/tauri Rust 层 | ✅ popup 接口签名不变 |
| ComponentContent + nestingBuilderSupported | ✅ 子菜单递归渲染需要此选项 |
| `js_init_script` 注入 pointerdown 监听 | ✅ 每个 WebView 独立，popup command 的 webview 参数是正确的 |
| 坐标系一致性分析 | ✅ 全屏根组件的 anchorPosition = 窗口绝对坐标 |

---

## 实施建议

按以下顺序修复：

1. **核心修复**：string id 锚点 + separator 判断（设计文档 Step 1-3）
2. **必要补充**：`close` 分支 + `&` strip + 事件回传
3. **验证**：设备上确认 popup 弹出 + 位置正确 + 点击有事件
4. **可选**：无坐标 popup 的 `eval_with_callback` 方案（可后续单独验证）

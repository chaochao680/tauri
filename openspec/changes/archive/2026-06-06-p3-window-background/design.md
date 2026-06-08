## Context

当前 Tauri OHOS 适配中，窗口背景色/透明控制完全缺失：

**已有管道（部分存在）：**
```
Tauri config → tauri-runtime-wry → tao WindowAttributes.transparent → platform_impl/ohos/mod.rs → ❌ 丢弃
Tauri config → tauri-runtime-wry → tao WindowAttributes.background_color → platform_impl/ohos/mod.rs → ❌ 丢弃
```

**断点位置：**
1. `tao/src/platform_impl/ohos/mod.rs:858`：`set_background_color()` 是空操作 `pub fn set_background_color(&self, _color: Option<crate::window::RGBA>) {}`
2. `Window::new()` 接收 `window_attrs.transparent` 和 `window_attrs.background_color` 但未读取
3. `create_os_window` config 对象无 `transparent` 或 `background_color` 字段
4. `type.ets` 的 `WindowConfig` 接口无相关字段
5. `WindowManager.createSubWindow` 不调用 `win.setWindowBackgroundColor()`

**OHOS Window API 背景色原理：**
- `window.Window.setWindowBackgroundColor(color: string)` (API 9+) 设置窗口背景色，支持 `#AARRGGBB` 格式
- 传入 `'#00000000'` 实现完全透明
- 主窗口和子窗口均支持
- PC 设备自由多窗模式下，还需 `setWindowContainerColor` 设置标题栏透明（当前不涉及）

**其他平台实现参考：**
- **Windows**：`WindowFlags::TRANSPARENT` + DWM blur behind + `WM_ERASEBKGND` 填充 `background_color`，alpha 被忽略
- **macOS**：`setOpaque(false)` + `NSColor.backgroundColor`，支持透明和自定义颜色，运行时 `setBackgroundColor` 动态更新

**Phase 1 关系**：Phase 1 实现 WebView 容器透明（Web 组件级别），Phase 3 实现窗口级别透明。两者配合才能实现完整的桌面穿透效果：WebView 透明 + 窗口透明 = 内容穿透到桌面。

## Goals / Non-Goals

**Goals:**
- 窗口创建时 `transparent: true` 生效：通过 `setWindowBackgroundColor('#00000000')` 设置窗口背景透明
- 窗口创建时 `background_color` 生效：颜色从 `Option<RGBA>` 转换为 `#AARRGGBB` 字符串传递给 OHOS
- 运行时 `set_background_color()` 生效：动态更新窗口背景色
- `transparent: true` 优先级高于 `background_color`（与 Phase 1 WebView transparent 优先级一致）
- 与 Windows/macOS 行为意图一致

**Non-Goals:**
- 不涉及 WebView 容器透明 — 属于 Phase 1
- 不涉及窗口 decorations — 属于 Phase 2
- 不涉及 PC 自由多窗模式下的标题栏透明（`setWindowContainerColor`）— 当前 Tauri OHOS 不涉及此场景
- 不涉及 `ohos.permission.SET_WINDOW_TRANSPARENT` 权限申请 — Tauri 应用使用普通子窗口，不需要此权限

## Decisions

### Decision 1: transparent 和 background_color 通过 WindowConfig 传递

**选择**：在 `create_os_window` 的 config 对象和 ArkTS `WindowConfig` 接口中添加 `transparent: boolean` 和 `backgroundColor: number`（`0xAARRGGBB` 格式）两个字段。

**理由**：
- 复用 `createOSWindow` 已有的 config 对象传递模式（Phase 2 也扩展了此 config）
- `transparent` 使用 boolean 类型，ArkTS 侧根据此值决定是设置 `'#00000000'` 还是使用 `backgroundColor`
- `backgroundColor` 使用 `number`（`0xAARRGGBB`）类型，与 Phase 1 的 WebView `setBackgroundColor` 参数类型一致
- 颜色格式统一为 `0xAARRGGBB`，Rust 端负责 RGBA → `0xAARRGGBB` 转换

**替代方案**：使用字符串格式 `#AARRGGBB` → 需要额外的字符串构造和解析开销

### Decision 2: transparent 优先于 background_color

**选择**：当 `transparent: true` 时，忽略 `background_color`，直接设置窗口背景为 `'#00000000'`（完全透明）。优先级逻辑在 Rust 端处理：
```rust
let bg_color = if window_attrs.transparent {
    Some(0x00000000u32) // 完全透明
} else {
    window_attrs.background_color.map(|(r, g, b, a)| {
        ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
    })
};
```

**理由**：
- 与 Phase 1 的 WebView transparent 优先级逻辑一致
- 与 macOS 行为一致（`transparent: true` 时使用 `NSColor.clearColor()`，忽略 `background_color`）
- 优先级在 Rust 端处理后，ArkTS 侧只需处理最终的 `backgroundColor` 值

### Decision 3: WindowManager 创建子窗口后调用 setWindowBackgroundColor

**选择**：在 `WindowManager.createSubWindow` 中，`loadContentByName` 之后调用 `win.setWindowBackgroundColor(colorString)` 设置窗口背景。

**颜色转换**：
- ArkTS 端收到 `backgroundColor: number`（`0xAARRGGBB`），转换为 `#AARRGGBB` 字符串：
  ```typescript
  function numberToColorString(color: number): string {
    return '#' + color.toString(16).padStart(8, '0');
  }
  ```
- 如果 `transparent: true`，直接使用 `'#00000000'`

**调用时序**：
- `setWindowBackgroundColor` 在 `loadContentByName` 之后、`showWindow` 之前调用
- 这与 OHOS 官方推荐的调用时序一致（先加载内容，再设置属性，最后显示窗口）

### Decision 4: 运行时 set_background_color 通过独立 NAPI 函数

**选择**：新增 `setWindowBackgroundColor(windowId: i64, color: u32)` NAPI 函数。tao 的 `set_background_color` 方法调用此函数。

**Rust 端颜色转换**：
```rust
pub fn set_background_color(&self, color: Option<RGBA>) {
    let color_u32 = if self.transparent {
        0x00000000u32  // transparent=true 时强制透明（见 Decision 6）
    } else {
        match color {
            Some((r, g, b, a)) => ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32),
            None => 0xFFFFFFFF, // 不透明白色作为默认值
        }
    };
    if let Some(window_id) = self.window_id {
        let _ = openharmony_ability::window::set_window_background_color(window_id, color_u32);
    }
}
```

**线程安全**：与 `create_os_window` 和 `set_window_decorations` 使用相同的 `get_main_thread_env()` 模式。

### Decision 5: 主窗口背景色通过独立 NAPI 调用传递

**选择**：主窗口（window_id=0）的背景色通过 `set_background_color` → NAPI `setWindowBackgroundColor(0, color)` 传递。WindowManager 收到 windowId=0 时，操作主窗口实例。

**获取主窗口实例**：`WindowManager` **没有** `mainWindow` 属性。与 Phase 2 的 `hideSystemBar` 相同，通过 `this.windowStage.getMainWindowSync()` 获取主窗口：
```typescript
setWindowBackground(windowId: number, color: number): void {
    let colorStr = '#' + color.toString(16).padStart(8, '0');
    if (windowId === 0 && this.windowStage) {
        let mainWindow = this.windowStage.getMainWindowSync();
        mainWindow.setWindowBackgroundColor(colorStr);
    } else {
        let entry = this.windows.get(windowId);
        if (entry) {
            entry.window.setWindowBackgroundColor(colorStr);
        }
    }
}
```

**理由**：
- 主窗口不走 `createOSWindow` 路径，无法在创建时设置
- 主窗口背景色通常在应用启动后通过 `set_background_color` 设置
- 复用 Phase 2 的 `windowStage.getMainWindowSync()` 模式，无需额外存储

### Decision 6: transparent 字段创建后不可变

**选择**：`Window` 结构体的 `transparent` 字段使用普通 `bool`（而非 `AtomicBool`），因为它是**创建时设定、之后不可变**的属性。

**运行时行为**：当 `transparent: true` 时，`set_background_color()` 被调用会**静默忽略**传入的颜色值，强制使用 `0x00000000`（完全透明）。这与创建时的优先级规则一致：transparent 永远优先于 background_color。

**理由**：
- tao 公共 API 中没有 `set_transparent()` 方法，transparent 只能通过 `with_transparent()` 在创建时设置
- 与 macOS 行为一致（macOS 的 `transparent` 也是存储为普通 bool，运行时通过 `self.transparent` 检查）
- `set_background_color` 对 transparent 窗口的静默忽略行为与 Windows 一致（Windows 上 transparent 窗口使用 DWM blur，background_color 不影响透明度）

### Decision 7: ArkHelper 接口、调用链转发和双文件同步

与 Phase 2 Decision 8/9/10 相同，本 Phase 也需要：
1. 在 `type.ets` 的 `ArkHelper` 接口中添加 `setWindowBackgroundColor: (windowId: number, color: number) => void`
2. 在 `ArkHelper.ets` 的 `createOSWindow` 调用处转发 `config.transparent` 和 `config.backgroundColor` 给 `createSubWindow`
3. 所有 ArkTS 修改同时应用到 `native_ability/` 和 `package/` 两个目录

## cfg 隔离策略

本 Phase 所有修改均在 OHOS 特有代码路径中：

| 文件 | 隔离方式 | 说明 |
|------|---------|------|
| `tao/src/platform_impl/ohos/mod.rs` | 整个文件在 `ohos` 模块下 | 仅 OHOS 编译时包含 |
| `openharmony-ability/*` | 整个 crate 是 OHOS 专用 | 仅用于 OHOS 平台 |
| `WindowManager.ets` | ArkTS 文件，仅 OHOS | 不参与其他平台编译 |
| `type.ets` | ArkTS 文件，仅 OHOS | 不参与其他平台编译 |

**结论**：无需添加额外的 `cfg` gate，所有修改天然隔离。

## Risks / Trade-offs

- **[setWindowBackgroundColor 调用时序]** `setWindowBackgroundColor` 需要在 `loadContentByName` 之后调用。如果在 `showWindow` 之后调用，可能出现短暂的白色闪烁 → **缓解**：在 `createSubWindow` 中严格按 loadContent → setBackground → showWindow 顺序执行
- **[子窗口透明限制]** OHOS 文档提到"PC 和平板的自由多窗模式下，应用子窗口可以实现背景完全透明；其他设备或主窗口可能无法实现完全透明" → **降级行为**：如果设备不支持完全透明，`setWindowBackgroundColor` 不报错但效果可能有限。记录为已知限制
- **[颜色格式一致性]** Phase 1 的 WebView backgroundColor 使用 `0xAARRGGBB` number，Phase 3 的 window backgroundColor 也使用相同格式。需确保转换逻辑一致 → 统一在 Rust 端做 RGBA → `0xAARRGGBB` 转换
- **[主窗口背景色时序]** 主窗口背景色通过运行时 `set_background_color` 设置，无法在窗口创建时立即生效 → 可能出现短暂的非透明背景闪烁。与 Windows/macOS 行为一致（也是在窗口创建后设置）
- **[setWindowContainerColor 不涉及]** PC 自由多窗模式下需要 `setWindowContainerColor` 设置标题栏透明 → 当前 Tauri OHOS 不涉及 PC 自由多窗场景，不实现此功能

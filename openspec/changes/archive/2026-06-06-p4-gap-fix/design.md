## Context

Phase 1-3 独立设计了三个功能：
- **Phase 1**：WebView 容器透明（wry 层 + openharmony-ability ArkTS）
- **Phase 2**：无标题栏窗口（tao 层 + openharmony-ability NAPI + ArkTS FloatPage）
- **Phase 3**：窗口背景透明（tao 层 + openharmony-ability NAPI + ArkTS WindowManager）

三个功能涉及相同的代码层（tao + openharmony-ability），但各 Phase 独立扩展了接口参数。Phase 4 需要确保它们在实现时不会产生冲突或遗漏。

**潜在集成问题：**

1. **create_os_window 签名冲突**：Phase 2 添加了 `decorations: bool` 参数，Phase 3 添加了 `transparent: bool` 和 `background_color: Option<u32>` 参数。如果各 Phase 独立修改函数签名，可能导致编译冲突或参数遗漏
2. **主窗口属性处理碎片化**：Phase 2 的主窗口 decorations 通过 `setWindowSystemBarEnable` 处理，Phase 3 的主窗口 transparent 通过 `setWindowBackgroundColor` 处理。两个处理路径可能在时序上冲突
3. **NAPI handler 命名冲突**：Phase 2 新增 `setWindowDecorations`，Phase 3 新增 `setWindowBackgroundColor`。ArkHelper.ets 需要同时注册两个 handler
4. **LocalStorage 字段扩展**：Phase 2 在 LocalStorage 中添加 `decorations`，Phase 3 可能也需要存储 `transparent` 状态。需要确保字段名不冲突
5. **WindowManager WindowEntry 扩展**：Phase 2 将 `windows` Map 从 `Map<number, window.Window>` 扩展为 `Map<number, WindowEntry>`（包含 storage 引用）。Phase 3 的 `setWindowBackground` 也需要通过此 Map 访问窗口实例

## Goals / Non-Goals

**Goals:**
- 统一 `create_os_window` 函数签名，整合 Phase 2+3 参数
- 统一主窗口属性处理入口（decorations + transparent + background_color 集中处理）
- 端到端验证 tauri config → wry → tao → NAPI → ArkTS 全链路
- 验证多窗口场景下各窗口状态独立
- 验证 decorations + transparent + background_color 组合场景

**Non-Goals:**
- 不引入新功能
- 不修改 Phase 1 的 WebView 透明逻辑（Phase 1 在 wry 层，与 Phase 2+3 在 tao 层不冲突）

## Decisions

### Decision 1: create_os_window 使用 WindowConfig 结构体传递参数

**选择**：将 `create_os_window` 的参数从散列参数改为统一的结构体，避免签名膨胀：
```rust
pub struct WindowCreateParams {
    pub name: String,
    pub window_type: i32,
    pub width: i32,      // 默认 800
    pub height: i32,     // 默认 600
    pub x: i32,          // 默认 100
    pub y: i32,          // 默认 100
    pub decorations: bool,
    pub transparent: bool,
    pub background_color: Option<u32>,
}

pub fn create_os_window(params: WindowCreateParams) -> napi_ohos::Result<i64> { ... }
```

> ⚠️ **实施顺序重要变更**：经审计发现，Phase 2 和 Phase 3 分别独立修改 `create_os_window` 签名会导致合并冲突。**推荐在实施 Phase 2 之前先完成此结构体重构**（作为 Phase 4 的 Task 1 提前执行），然后 Phase 2 和 Phase 3 只需在 `Window::new()` 中填充对应字段即可。

**理由**：
- 当前已有 7 个参数（name, type, windowId, width, height, x, y），加上 Phase 2+3 的参数将达到 10+
- 使用结构体更易于维护和扩展
- ArkTS 侧的 WindowConfig 接口已是对象形式，Rust 侧对齐
- `windowId` 不在此结构体中（由 `create_os_window` 内部通过 `NEXT_WINDOW_ID` 自动生成）

**替代方案**：保持散列参数 → 签名膨胀，容易遗漏参数

### Decision 2: 主窗口属性通过统一初始化函数处理

**选择**：在 ArkTS 的 `WindowManager` 中添加 `initMainWindow(config)` 方法，集中处理主窗口的 decorations + transparent + background_color：
```typescript
initMainWindow(config: MainWindowConfig): void {
    if (!this.windowStage) return;
    try {
        let mainWindow = this.windowStage.getMainWindowSync();
        
        // 1. 设置 decorations（隐藏状态栏）
        if (config.decorations === false) {
            this.hideSystemBar();  // 内部也使用 getMainWindowSync()
        }
        
        // 2. 设置 transparent/background_color
        if (config.transparent) {
            mainWindow.setWindowBackgroundColor('#00000000');
        } else if (config.backgroundColor !== undefined) {
            let colorStr = '#' + config.backgroundColor.toString(16).padStart(8, '0');
            mainWindow.setWindowBackgroundColor(colorStr);
        }
    } catch (e) {
        console.error(`Failed to init main window: ${e}`);
    }
}
```

**理由**：
- 避免 Phase 2 和 Phase 3 分别在不同位置处理主窗口属性导致的时序冲突
- 统一入口便于维护和调试
- 与 FloatPage 子窗口的 `createSubWindow` 处理模式对称

### Decision 3: tao Window::new() 中主窗口属性通过延迟调用传递

**选择**：主窗口创建后，tao 通过 NAPI 调用一个统一的初始化函数 `initMainWindow(config)` 传递 decorations + transparent + background_color。

**理由**：
- 主窗口在 tao `Window::new()` 之前已由 ArkTS UIAbility 创建
- `Window::new()` 中主窗口路径（`is_main_window = true`）不调用 `create_os_window`
- 需要一个独立的 NAPI 调用将属性从 tao 传递到 ArkTS
- 此调用在 `Window::new()` 返回前完成（同步 NAPI 调用）

## cfg 隔离策略

本 Phase 不引入新的代码路径，仅整合 Phase 1-3 的现有修改。所有修改天然在 OHOS 特有路径中。

## Risks / Trade-offs

- **[实施顺序依赖]** Phase 4 依赖 Phase 1-3 全部实施完成。如果 Phase 1-3 实施中发现设计变更，Phase 4 需要相应调整 → **缓解**：Phase 4 设计保持灵活，以"审计+修复"模式应对
- **[create_os_window 重构影响]** 将散列参数改为结构体可能影响所有调用方 → **缓解**：当前仅 `tao/src/platform_impl/ohos/mod.rs` 一个调用方
- **[端到端测试覆盖不全]** 可能存在未预见的组合场景 → **缓解**：在 Phase 4 实施阶段增加探索性测试

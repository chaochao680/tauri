# Phase 4: 差距修复 - 实施任务

## 状态
- **方案**: create_os_window 结构体化 + 主窗口统一初始化 + 端到端集成测试
- **当前阶段**: Task 1 应在 Phase 2/3 实施之前提前执行（避免签名合并冲突）

> ⚠️ **实施顺序**：Task 1（WindowCreateParams 结构体重构）应在 Phase 2 和 Phase 3 实施**之前**完成。
> Phase 2/3 实施时只需在 `Window::new()` 中填充对应字段，无需再修改 `create_os_window` 签名。

---

## 1. create_os_window 参数整合（⚠️ 提前执行）

### 1.1 定义 WindowCreateParams 结构体
- [ ] 在 `openharmony-ability/crates/ability/src/window/mod.rs` 中定义（包含全部参数）：
  ```rust
  pub struct WindowCreateParams {
      pub name: String,
      pub window_type: i32,
      pub width: i32,           // 默认 800
      pub height: i32,          // 默认 600
      pub x: i32,               // 默认 100
      pub y: i32,               // 默认 100
      pub decorations: bool,    // Phase 2
      pub transparent: bool,    // Phase 3
      pub background_color: Option<u32>,  // Phase 3
  }
  ```
  > `windowId` 不在结构体中（由 `create_os_window` 内部通过 `NEXT_WINDOW_ID` 自动生成）。

### 1.2 重构 create_os_window 签名
- [ ] 修改 `create_os_window` 接受 `WindowCreateParams` 参数
- [ ] 在 config 对象中设置所有字段：`name, type, windowId, width, height, x, y, decorations, transparent, backgroundColor`

### 1.3 更新 tao 调用方
- [ ] 修改 `tao/src/platform_impl/ohos/mod.rs` 的 `Window::new()` 中 `create_os_window` 调用：
  ```rust
  let params = WindowCreateParams {
      name: label,
      window_type: window_type as i32,
      width: 800,
      height: 600,
      x: 100,
      y: 100,
      decorations: window_attrs.decorations,
      transparent: window_attrs.transparent,
      background_color: if window_attrs.transparent {
          Some(0x00000000)
      } else {
          window_attrs.background_color.map(|(r,g,b,a)|
              ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32))
      },
  };
  create_os_window(params)
  ```

### 1.4 提取颜色转换辅助函数
- [ ] 在 `tao/src/platform_impl/ohos/mod.rs` 中提取 RGBA → `0xAARRGGBB` 转换为辅助函数，避免 Phase 3 和 Phase 4 重复逻辑：
  ```rust
  fn rgba_to_ohos_color(transparent: bool, bg: Option<RGBA>) -> Option<u32> {
      if transparent {
          Some(0x00000000)
      } else {
          bg.map(|(r, g, b, a)|
              ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32))
      }
  }
  ```
  此函数在 `Window::new()`（创建路径）和 `set_background_color()`（运行时路径）中复用。

---

## 2. 主窗口统一初始化

### 2.1 新增 initMainWindow NAPI 函数
- [ ] 在 `openharmony-ability/crates/ability/src/window/mod.rs` 中添加：
  ```rust
  pub fn init_main_window(config: MainWindowConfig) -> napi_ohos::Result<()> {
      // 调用 ArkTS "initMainWindow" handler
  }
  ```

### 2.2 ArkTS 侧 initMainWindow handler
- [ ] 在 `WindowManager.ets` 中添加 `initMainWindow(config)` 方法
- [ ] 在 `ArkHelper.ets` 中注册 `initMainWindow` handler
- [ ] 集中处理：decorations → hideSystemBar() + transparent → setWindowBackgroundColor()

### 2.3 tao 主窗口初始化调用
- [ ] 在 `tao/src/platform_impl/ohos/mod.rs` 的 `Window::new()` 主窗口路径中：
  ```rust
  if is_main_window {
      // ... existing UIAbility singleton check ...
      let config = MainWindowConfig {
          decorations: window_attrs.decorations,
          transparent: window_attrs.transparent,
          background_color: /* ... */,
      };
      let _ = openharmony_ability::window::init_main_window(config);
  }
  ```

---

## 3. WindowManager 属性处理整合

### 3.1 统一 createSubWindow 属性处理
- [ ] 确保 `createSubWindow` 中 decorations、transparent、backgroundColor 的处理顺序一致：
  1. `loadContentByName`（加载 FloatPage）
  2. `setWindowBackgroundColor`（设置背景色/透明）
  3. `resize` + `moveWindowTo`（设置位置和大小）
  4. `showWindow`（显示窗口）

### 3.2 统一 WindowEntry 结构
- [ ] 确认 `WindowEntry` 包含 window + storage 引用：
  ```typescript
  interface WindowEntry {
    window: window.Window;
    storage: LocalStorage;
  }
  ```
- [ ] 确认 `setDecorations` 和 `setWindowBackground` 均通过 WindowEntry 访问

---

## 4. 端到端传递链路验证

### 4.1 tauri-runtime-wry → wry 传递验证
- [ ] 验证 `decorations` 属性从 tauri-runtime-wry 正确传递到 wry
- [ ] 验证 `transparent` 属性从 tauri-runtime-wry 正确传递到 wry
- [ ] 验证 `background_color` 属性从 tauri-runtime-wry 正确传递到 wry

### 4.2 wry → tao 传递验证
- [ ] 验证 wry 的 `WindowBuilder` 正确调用 tao 的 `with_decorations`、`with_transparent`、`with_background_color`
- [ ] 确认 wry 的 `Window::set_background_color` 正确调用 tao 的 `set_background_color`

### 4.3 tao → NAPI → ArkTS 传递验证
- [ ] 验证 tao 的 `Window::new()` 正确传递所有属性到 `create_os_window`
- [ ] 验证 ArkTS 侧 WindowConfig 接收到所有属性
- [ ] 验证 WindowManager 正确应用所有属性

---

## 5. 组合场景测试

### 5.1 decorations=false + transparent=true
- [ ] 创建无边框透明 Float 窗口，确认标题栏隐藏 + 背景透明
- [ ] HTML `body { background: transparent }` 确认穿透到桌面

### 5.2 decorations=false + background_color
- [ ] 创建无边框 + 半透明红色背景窗口，确认标题栏隐藏 + 红色背景

### 5.3 decorations=true + transparent=true
- [ ] 创建有标题栏 + 透明背景窗口，确认标题栏可见 + 背景透明

### 5.4 全部默认
- [ ] 创建默认窗口，确认标题栏可见 + 不透明背景

### 5.5 运行时切换组合
- [ ] 运行时 `set_decorations(false)` + `set_background_color(transparent)`，确认动态切换正确
- [ ] 运行时 `set_decorations(true)` + `set_background_color(red)`，确认恢复

---

## 6. 多窗口隔离测试

### 6.1 两个窗口不同 decorations
- [ ] 窗口 A (decorations=true) + 窗口 B (decorations=false)，确认互不干扰

### 6.2 两个窗口不同 transparent
- [ ] 窗口 A (transparent=true) + 窗口 B (transparent=false)，确认互不干扰

### 6.3 窗口创建/销毁不影响其他窗口
- [ ] 创建窗口 A → 创建窗口 B → 销毁窗口 A → 确认窗口 B 不受影响

---

## 依赖关系

```
1. create_os_window 参数整合
    └─→ 2. 主窗口统一初始化
        └─→ 3. WindowManager 属性处理整合
            └─→ 4. 端到端传递链路验证
                └─→ 5. 组合场景测试
                └─→ 6. 多窗口隔离测试
```

---

## 预估工作量

| 模块 | 任务数 | 预估时间 |
|------|--------|----------|
| create_os_window 参数整合 | 3 | 1h |
| 主窗口统一初始化 | 3 | 1h |
| WindowManager 属性处理整合 | 2 | 0.5h |
| 端到端传递验证 | 9 | 2h |
| 组合场景测试 | 7 | 2h |
| 多窗口隔离测试 | 3 | 1h |
| **总计** | **27** | **~7.5h** |

---

## 验收标准

1. ✅ `create_os_window` 使用统一结构体参数，整合 decorations + transparent + backgroundColor
2. ✅ 主窗口属性通过统一 `initMainWindow` 处理
3. ✅ decorations + transparent 组合场景正确工作
4. ✅ 多窗口状态相互独立
5. ✅ 端到端传递链路完整无丢失
6. ✅ 所有组合场景测试通过
7. ✅ 所有多窗口隔离测试通过

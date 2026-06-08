# Phase 3: 窗口背景透明 - 实施任务

## 状态
- **方案**: WindowConfig 扩展 + setWindowBackgroundColor API + NAPI 桥接
- **API 验证**: ✅ 已完成（setWindowBackgroundColor API 9+，主窗口和子窗口均支持）
- **当前阶段**: 等待实施

---

## 1. Rust 端 tao Window 结构扩展

### 1.1 Window 结构体添加 transparent 字段
- [ ] 修改 `tao/src/platform_impl/ohos/mod.rs` 的 `Window` 结构体：
  ```rust
  pub(crate) struct Window {
    app: OpenHarmonyApp,
    window_id: Option<i64>,
    theme: AtomicU8,
    decorations: AtomicBool,
    transparent: bool,  // 新增
  }
  ```

### 1.2 Window::new() 读取 transparent 和 background_color
- [ ] 修改 `Window::new()` 方法：
  - 读取 `window_attrs.transparent` 并存储
  - 将 `background_color` 转换为 `0xAARRGGBB` u32
  - 在创建 Float 窗口时将 transparent 和 backgroundColor 传入 `create_os_window`
  ```rust
  let bg_color = if window_attrs.transparent {
      Some(0x00000000u32)
  } else {
      window_attrs.background_color.map(|(r, g, b, a)| {
          ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
      })
  };
  // 传入 create_os_window
  ```

### 1.3 实现 set_background_color 方法
- [ ] 修改 `set_background_color` 方法，从 no-op 改为实际调用：
  ```rust
  pub fn set_background_color(&self, color: Option<RGBA>) {
      let color_u32 = if self.transparent {
          0x00000000u32
      } else {
          match color {
              Some((r, g, b, a)) => ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32),
              None => 0xFFFFFFFF,
          }
      };
      if let Some(window_id) = self.window_id {
          let _ = openharmony_ability::window::set_window_background_color(window_id, color_u32);
      }
  }
  ```

---

## 2. NAPI 桥接层扩展

### 2.1 create_os_window config 添加 transparent 和 backgroundColor 字段
- [ ] 修改 `openharmony-ability/crates/ability/src/window/mod.rs` 的 `create_os_window` 函数：
  - 添加 `transparent: bool` 和 `background_color: Option<u32>` 参数
  - 在 config 对象中设置相应字段
  ```rust
  pub fn create_os_window(
      name: String,
      window_type: i32,
      decorations: bool,
      transparent: bool,
      background_color: Option<u32>,
  ) -> napi_ohos::Result<i64> {
      // ...
      config.set("transparent", transparent)?;
      if let Some(color) = background_color {
          config.set("backgroundColor", color)?;
      }
      // ...
  }
  ```

### 2.2 新增 set_window_background_color NAPI 函数
- [ ] 在 `openharmony-ability/crates/ability/src/window/mod.rs` 中**新增**运行时背景色更新函数（此函数不存在，需从零编写）：
  ```rust
  pub fn set_window_background_color(window_id: i64, color: u32) -> napi_ohos::Result<()> {
      let ret = unsafe { get_helper() };
      if let Some(h) = ret.borrow().as_ref() {
          if let Some(env) = get_main_thread_env().borrow().as_ref() {
              let obj = h.get_value(env)?;
              let func = obj.get_named_property::<Function<'_, (i64, u32), ()>>("setWindowBackgroundColor")?;
              func.call((window_id, color))?;
              return Ok(());
          }
      }
      Err(Error::from_reason("Helper or Env not initialized"))
  }
  ```

### 2.3 tao 调用方适配
- [ ] 修改 `tao/src/platform_impl/ohos/mod.rs` 的 `Window::new()` 中 `create_os_window` 调用，传入 transparent 和 background_color 参数

---

## 3. ArkTS 端扩展

### 3.1 WindowConfig 接口添加 transparent 和 backgroundColor 字段
- [ ] 修改 `type.ets` 的 `WindowConfig` 接口（**两个位置**都要改）：
  - `native_ability/src/main/ets/ability/type.ets`
  - `package/src/main/ets/ability/type.ets`
  ```typescript
  export interface WindowConfig {
    name: string;
    type: number;
    windowId: number;
    width?: number;
    height?: number;
    x?: number;
    y?: number;
    decorations?: boolean;
    transparent?: boolean;      // 新增
    backgroundColor?: number;   // 新增，0xAARRGGBB 格式
  }
  ```
- [ ] 在 `type.ets` 的 `ArkHelper` 接口中**添加** `setWindowBackgroundColor` 声明：
  ```typescript
  setWindowBackgroundColor: (windowId: number, color: number) => void;
  ```
  > ⚠️ **必须同步更新接口声明和实现**。Rust NAPI 通过 `get_named_property` 查找函数名。

### 3.1b ArkHelper 调用链转发
- [ ] 修改 `ArkHelper.ets`（两个位置）的 `createOSWindow` handler，在调用 `wm.createSubWindow` 时追加 `config.transparent` 和 `config.backgroundColor`：
  ```typescript
  // 修改后（Phase 2 已添加 decorations 参数）：
  return await wm.createSubWindow(
      config.name, config.type, config.windowId,
      config.width, config.height, config.x, config.y,
      config.decorations,
      config.transparent,        // 新增
      config.backgroundColor     // 新增
  );
  ```
- [ ] 在 `ArkHelper.ets`（两个位置）`createArkHelper()` 返回的对象字面量中添加：
  ```typescript
  setWindowBackgroundColor: async (windowId: number, color: number): Promise<void> => {
      await wm.setWindowBackground(windowId, color);
  }
  ```

### 3.2 WindowManager 创建子窗口时设置背景色
- [ ] 修改 `openharmony-ability/native_ability/src/main/ets/window/WindowManager.ets` 的 `createSubWindow` 方法：
  - 添加 `transparent: boolean = false` 和 `backgroundColor?: number` 参数
  - 在 `loadContentByName` 之后、`showWindow` 之前调用 `setWindowBackgroundColor`
  ```typescript
  // 设置窗口背景色
  if (transparent) {
    await win.setWindowBackgroundColor('#00000000');
  } else if (backgroundColor !== undefined) {
    let colorStr = '#' + backgroundColor.toString(16).padStart(8, '0');
    await win.setWindowBackgroundColor(colorStr);
  }
  ```

### 3.3 WindowManager 添加 setWindowBackground 方法
- [ ] 在 `WindowManager.ets` 中添加运行时更新方法。
  > ⚠️ 注意：`WindowManager` **没有** `mainWindow` 属性。主窗口通过 `this.windowStage.getMainWindowSync()` 获取（与 Phase 2 相同模式）。
  ```typescript
  setWindowBackground(windowId: number, color: number): void {
    let colorStr = '#' + color.toString(16).padStart(8, '0');
    if (windowId === 0 && this.windowStage) {
      try {
        let mainWindow = this.windowStage.getMainWindowSync();
        mainWindow.setWindowBackgroundColor(colorStr);
      } catch (e) {
        console.error(`Failed to set main window background: ${e}`);
      }
    } else {
      let entry = this.windows.get(windowId);
      if (entry) {
        entry.window.setWindowBackgroundColor(colorStr);
      }
    }
  }
  ```

---

---

## 4. 颜色格式转换验证

### 4.1 Rust 端 RGBA → 0xAARRGGBB 转换测试
- [ ] 验证 `RGBA(255, 128, 0, 200)` → `0xC8FF8000`
- [ ] 验证 `RGBA(0, 0, 0, 0)` → `0x00000000`（完全透明）
- [ ] 验证 `RGBA(255, 255, 255, 255)` → `0xFFFFFFFF`（不透明白色）

### 4.2 ArkTS 端 u32 → #AARRGGBB 转换测试
- [ ] 验证 `0xC8FF8000` → `'#C8FF8000'`
- [ ] 验证 `0x00000000` → `'#00000000'`
- [ ] 验证 `0x80FF0000` → `'#80FF0000'`（半透明红）

---

## 5. 集成测试

### 5.1 创建时 transparent=true 测试
- [ ] 创建 Float 窗口 `transparent: true`，确认窗口背景透明
- [ ] 创建 Float 窗口 `transparent: false`（默认），确认背景不透明

### 5.2 创建时 background_color 测试
- [ ] 创建 Float 窗口 `background_color: (255, 0, 0, 128)`，确认半透明红色背景
- [ ] 创建 Float 窗口 `background_color: (0, 0, 0, 0)`，确认背景透明

### 5.3 运行时 set_background_color 测试
- [ ] 调用 `set_background_color(Some((255, 0, 0, 255)))`，确认背景变红
- [ ] 调用 `set_background_color(None)`，确认背景恢复默认

### 5.4 transparent 优先级测试
- [ ] 同时设置 `transparent: true` 和 `background_color`，确认透明优先

### 5.5 与 Phase 1 WebView 透明配合测试
- [ ] WebView transparent=true + Window transparent=true，确认完整穿透效果（需设备端验证）

---

## 依赖关系

```
1. Rust 端 Window 结构扩展
    └─→ 2. NAPI 桥接层扩展
        └─→ 3. ArkTS 端扩展
            └─→ 4. 颜色格式验证
                └─→ 5. 集成测试
```

---

## 预估工作量

| 模块 | 任务数 | 预估时间 |
|------|--------|----------|
| Rust 端 Window 结构扩展 | 3 | 0.5h |
| NAPI 桥接层 | 3 | 1h |
| ArkTS 端扩展 | 4 | 1h |
| 颜色格式验证 | 6 | 0.5h |
| 集成测试 | 7 | 1.5h |
| **总计** | **23** | **~4.5h** |

---

## 验收标准

1. ✅ `transparent: true` 创建窗口背景透明
2. ✅ `background_color` 创建窗口使用指定背景色
3. ✅ `transparent` 优先于 `background_color`
4. ✅ 运行时 `set_background_color` 可动态更新背景色
5. ✅ RGBA → `0xAARRGGBB` → `#AARRGGBB` 颜色格式转换正确
6. ✅ 与 Phase 1 WebView 透明配合实现完整穿透效果
7. ✅ 所有集成测试通过

# Phase 2: 无标题栏窗口 - 实施任务

## 状态
- **方案**: decorations 条件渲染 + LocalStorage 状态传递 + NAPI 桥接
- **API 验证**: ✅ 已完成（setWindowSystemBarEnable API 9+、setWindowDecorHeight 不适用）
- **当前阶段**: 等待实施

---

## 1. Rust 端 tao Window 结构扩展

### 1.1 Window 结构体添加 decorations 字段
- [ ] 修改 `tao/src/platform_impl/ohos/mod.rs` 的 `Window` 结构体：
  ```rust
  pub(crate) struct Window {
    app: OpenHarmonyApp,
    window_id: Option<i64>,
    theme: AtomicU8,
    decorations: AtomicBool,  // 新增
  }
  ```

### 1.2 Window::new() 读取 decorations 属性
- [ ] 修改 `Window::new()` 方法，从 `window_attrs.decorations` 读取初始值：
  ```rust
  let decorations = window_attrs.decorations;
  Ok(Self {
    app: el.app.clone(),
    window_id,
    theme: AtomicU8::new(0),
    decorations: AtomicBool::new(decorations),
  })
  ```

### 1.3 实现 set_decorations 方法
- [ ] 修改 `set_decorations` 方法，从 no-op 改为实际调用：
  ```rust
  pub fn set_decorations(&self, decorations: bool) {
    self.decorations.store(decorations, Ordering::Release);
    if let Some(window_id) = self.window_id {
      // 通过 NAPI 调用 ArkTS 更新 decorations 状态
      // 调用 openharmony_ability::window::set_window_decorations(window_id, decorations)
      let _ = openharmony_ability::window::set_window_decorations(window_id, decorations);
    }
  }
  ```

### 1.4 实现 is_decorated 方法
- [ ] 修改 `is_decorated` 方法：
  ```rust
  pub fn is_decorated(&self) -> bool {
    self.decorations.load(Ordering::Acquire)
  }
  ```

---

## 2. NAPI 桥接层 decorations 传递

### 2.1 create_os_window config 添加 decorations 字段
- [ ] 修改 `openharmony-ability/crates/ability/src/window/mod.rs` 的 `create_os_window` 函数：
  - 添加 `decorations: bool` 参数
  - 在 config 对象中设置 `config.set("decorations", decorations)?`
  ```rust
  pub fn create_os_window(name: String, window_type: i32, decorations: bool) -> napi_ohos::Result<i64> {
    // ... 现有逻辑 ...
    config.set("decorations", decorations)?;
    // ...
  }
  ```

### 2.2 新增 set_window_decorations NAPI 函数
- [ ] 在 `openharmony-ability/crates/ability/src/window/mod.rs` 中**新增**运行时 decorations 切换函数（此函数不存在，需从零编写）：
  ```rust
  pub fn set_window_decorations(window_id: i64, decorations: bool) -> napi_ohos::Result<()> {
      let ret = unsafe { get_helper() };
      if let Some(h) = ret.borrow().as_ref() {
          if let Some(env) = get_main_thread_env().borrow().as_ref() {
              let obj = h.get_value(env)?;
              let func = obj.get_named_property::<Function<'_, (i64, bool), ()>>("setWindowDecorations")?;
              func.call((window_id, decorations))?;
              return Ok(());
          }
      }
      Err(Error::from_reason("Helper or Env not initialized"))
  }
  ```
  此函数与 `create_os_window` 使用完全相同的 `get_helper()` + `get_main_thread_env()` 模式。

### 2.3 tao 调用方适配
- [ ] 修改 `tao/src/platform_impl/ohos/mod.rs` 的 `Window::new()` 中 `create_os_window` 调用，传入 `decorations` 参数

---

## 3. ArkTS 端 WindowConfig 和 ArkHelper 扩展

### 3.1 WindowConfig 接口添加 decorations 字段
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
    decorations?: boolean;  // 新增，默认 true
  }
  ```
- [ ] 在 `type.ets` 的 `ArkHelper` 接口中**添加** `setWindowDecorations` 声明：
  ```typescript
  setWindowDecorations: (windowId: number, decorations: boolean) => void;
  ```
  > ⚠️ **必须同步更新接口声明和实现**。Rust NAPI 通过 `get_named_property` 查找函数名，接口中未声明不会报错但运行时会崩溃。

### 3.2 ArkHelper 添加 decorations 处理（含调用链转发）
- [ ] 修改 `ArkHelper.ets`（两个位置）的 `createOSWindow` handler：
  - 从 config 中读取 `decorations` 属性
  - **在 `wm.createSubWindow` 调用处追加** `config.decorations` 参数：
  ```typescript
  // 修改前：
  return await wm.createSubWindow(
      config.name, config.type, config.windowId,
      config.width, config.height, config.x, config.y
  );
  // 修改后：
  return await wm.createSubWindow(
      config.name, config.type, config.windowId,
      config.width, config.height, config.x, config.y,
      config.decorations  // 新增
  );
  ```

### 3.3 ArkHelper 添加 setWindowDecorations 实现
- [ ] 在 `ArkHelper.ets`（两个位置）`createArkHelper()` 返回的对象字面量中添加：
  ```typescript
  setWindowDecorations: async (windowId: number, decorations: boolean): Promise<void> => {
      await wm.setDecorations(windowId, decorations);
  }
  ```

---

## 4. WindowManager decorations 状态管理

### 4.1 createSubWindow 传递 decorations
- [ ] 修改 `openharmony-ability/native_ability/src/main/ets/window/WindowManager.ets` 的 `createSubWindow` 方法：
  - 添加 `decorations: boolean = true` 参数
  - 在创建 LocalStorage 时写入 decorations 状态：
    ```typescript
    let storage = new LocalStorage();
    storage.setOrCreate('windowId', windowId);
    storage.setOrCreate('decorations', decorations);  // 新增
    ```

### 4.2 添加 setDecorations 方法
- [ ] 在 `WindowManager.ets` 中添加运行时更新方法。
  > ⚠️ 注意：`windows` Map 已重构为 `Map<number, WindowEntry>`（Decision 6），其中 `WindowEntry = { window, storage }`。通过 `entry.storage` 访问 LocalStorage。
  ```typescript
  setDecorations(windowId: number, decorations: boolean): void {
    let entry = this.windows.get(windowId);
    if (entry && entry.storage) {
      entry.storage.setOrCreate('decorations', decorations);
    }
    // 如果是主窗口且 decorations=false，隐藏状态栏
    if (windowId === 0 && !decorations) {
      this.hideSystemBar();
    }
  }
  ```

### 4.3 添加 hideSystemBar 辅助方法
- [ ] 在 `WindowManager.ets` 中添加主窗口状态栏隐藏方法。
  > ⚠️ 注意：`WindowManager` **没有** `mainWindow` 属性。通过已有的 `this.windowStage` 调用 `getMainWindowSync()` 获取主窗口实例。
  ```typescript
  private hideSystemBar(): void {
    if (!this.windowStage) return;
    try {
      let mainWindow = this.windowStage.getMainWindowSync();
      let names: Array<'status' | 'navigation'> = [];
      mainWindow.setWindowSystemBarEnable(names).catch((err: BusinessError) => {
        console.error(`Failed to hide system bar: ${err.message}`);
      });
    } catch (e) {
      console.error(`Failed to get main window: ${e}`);
    }
  }
  ```

---

## 5. FloatPage 条件渲染

### 5.1 添加 decorations 状态属性
- [ ] 修改 `openharmony-ability/native_ability/src/main/ets/components/FloatPage.ets`：
  - 添加 `@LocalStorageProp('decorations')` 属性：
    ```typescript
    @LocalStorageProp('decorations') decorations: boolean = true;
    ```

### 5.2 MenuBarComponent 条件渲染
- [ ] 修改 MenuBarComponent 的渲染条件：
  - 从 `if (this.isDesktop)` 改为 `if (this.isDesktop && this.decorations)`

### 5.3 拖拽区 + 关闭按钮条件渲染
- [ ] 修改拖拽区和关闭按钮 Row 的渲染条件：
  - 从 `if (this.windowClass)` 改为 `if (this.windowClass && this.decorations)`

### 5.4 resize handle 保持不变
- [ ] 确认 resize handle（右边缘、下边缘、右下角）不受 decorations 影响：
  - 保持 `if (this.isDesktop && this.windowClass)` 条件不变
  - decorations=false 时仍可调整窗口大小

---

## 6. 集成测试

### 6.1 创建时 decorations=false 测试
- [ ] 创建 Float 窗口 `decorations: false`，确认无标题栏（无 MenuBarComponent、无拖拽区、无关闭按钮）
- [ ] 创建 Float 窗口 `decorations: true`（默认），确认有完整标题栏
- [ ] 创建 decorations=false 窗口，确认 resize handle 仍可用

### 6.2 运行时 set_decorations 测试
- [ ] 对 decorations=true 窗口调用 `set_decorations(false)`，确认标题栏消失
- [ ] 对 decorations=false 窗口调用 `set_decorations(true)`，确认标题栏恢复
- [ ] 快速连续调用 `set_decorations` 多次，确认无崩溃

### 6.3 is_decorated 状态测试
- [ ] decorations=true 窗口查询 `is_decorated()`，确认返回 `true`
- [ ] decorations=false 窗口查询 `is_decorated()`，确认返回 `false`
- [ ] 运行时切换后查询 `is_decorated()`，确认状态正确

### 6.4 主窗口 decorations 测试
- [ ] 主窗口 decorations=false + 全屏模式，确认状态栏隐藏
- [ ] 主窗口 decorations=false + 非全屏模式，确认行为（可能不生效）

### 6.5 多窗口 decorations 独立测试
- [ ] 创建多个窗口，分别设置不同 decorations 状态，确认互不干扰

---

## 7. 文档更新

### 7.1 代码注释
- [ ] 在 `tao/src/platform_impl/ohos/mod.rs` 中添加 decorations 字段和 set_decorations 方法的注释
- [ ] 在 `FloatPage.ets` 中添加 decorations 条件渲染的注释
- [ ] 在 `WindowManager.ets` 中添加 decorations 状态管理的注释

---

## 依赖关系

```
1. Rust 端 Window 结构扩展
    ├── 1.1-1.4 decorations 字段和方法
    └─→ 2. NAPI 桥接层

2. NAPI 桥接层
    ├── 2.1 create_os_window config 扩展
    ├── 2.2 set_window_decorations 新函数
    └── 2.3 tao 调用方适配
    └─→ 3. ArkTS 端扩展

3. ArkTS 端扩展
    ├── 3.1 WindowConfig 接口
    ├── 3.2-3.3 ArkHelper handler
    └─→ 4. WindowManager

4. WindowManager
    ├── 4.1-4.3 decorations 状态管理
    └─→ 5. FloatPage 条件渲染

5. FloatPage 条件渲染
    └─→ 6. 集成测试

7. 文档更新（可并行）
```

---

## 预估工作量

| 模块 | 任务数 | 预估时间 |
|------|--------|----------|
| Rust 端 Window 结构扩展 | 4 | 0.5h |
| NAPI 桥接层 | 3 | 1h |
| ArkTS 端扩展 | 3 | 0.5h |
| WindowManager | 3 | 1h |
| FloatPage 条件渲染 | 4 | 0.5h |
| 集成测试 | 11 | 2h |
| 文档更新 | 3 | 0.5h |
| **总计** | **31** | **~6h** |

---

## 验收标准

1. ✅ `decorations: false` 创建 Float 窗口无标题栏
2. ✅ `decorations: true`（默认）保持现有行为
3. ✅ 运行时 `set_decorations(bool)` 动态切换标题栏
4. ✅ `is_decorated()` 返回正确状态
5. ✅ decorations=false 时 resize handle 保持可用
6. ✅ 主窗口 decorations=false 时系统状态栏隐藏
7. ✅ 多窗口 decorations 状态独立
8. ✅ 所有集成测试通过

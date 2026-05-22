# Phase 5: 集成与测试

## 目标
完成 Tauri Menu API 的 OHOS 集成，编写完整的测试用例，验证所有功能正常工作。

## 依赖
- Phase 0: muda OHOS 后端完成
- Phase 1-4: 所有模块完成
- tauri/Cargo.toml: 已修改为依赖 muda（OHOS）

## 架构说明

> **重要**：Phase 5 是集成阶段，需要：
>
> 1. **修改 tauri/Cargo.toml**（已在 Phase 0 设计）
>    - 让 OHOS 也依赖 muda 和 tray-icon
>
> 2. **修改 tauri::menu 源码**
>    - `menu.rs::popup_inner()`: 添加 #[cfg(target_env = "ohos")] 分支
>    - `submenu.rs::popup_inner()`: 添加 #[cfg(target_env = "ohos")] 分支
>
> 3. **验证编译**
>    - tauri::menu 在 OHOS 上成功编译
>    - 所有 muda::* 类型正确导入
>
> 4. **运行测试**
>    - Rust UT: muda::platform_impl::ohos
>    - 前端 API 测试: tauri::menu 完整功能

## 工作内容

### 5.1 tauri/Cargo.toml 修改

**文件**: `tauri/crates/tauri/Cargo.toml`

```toml
# 移除 not(target_env = "ohos")，让桌面平台正常依赖 muda
[target.'cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "openbsd",
  target_os = "netbsd",
  target_os = "windows",
  target_os = "macos"
))'.dependencies]
muda = { version = "0.17", default-features = false, features = [
  "serde",
  "gtk",
] }
tray-icon = { version = "0.22", default-features = false, features = [
  "serde",
], optional = true }

# 新增 OHOS 依赖本地 muda/tray-icon
[target.'cfg(target_env = "ohos")'.dependencies]
muda = { path = "../../muda" }
tray-icon = { path = "../../tray-icon", optional = true }
```

### 5.2 tauri::menu popup 分支修改

**文件**: `tauri/crates/tauri/src/menu/menu.rs`

在 `popup_inner` 方法中添加 OHOS 分支：

```rust
impl<R: Runtime> ContextMenuBase for Menu<R> {
  fn popup_inner<T: Runtime, P: Into<crate::Position>>(
    &self,
    window: crate::Window<T>,
    position: Option<P>,
  ) -> crate::Result<()> {
    let position = position.map(Into::into);
    run_item_main_thread!(self, move |self_: Self| {
      #[cfg(target_os = "macos")]
      if let Ok(view) = window.ns_view() {
        unsafe {
          self_
            .inner()
            .show_context_menu_for_nsview(view as _, position);
        }
      }

      #[cfg(all(
        any(
          target_os = "linux",
          target_os = "dragonfly",
          target_os = "freebsd",
          target_os = "netbsd",
          target_os = "openbsd"
        ),
        not(target_env = "ohos")
      ))]
      if let Ok(w) = window.gtk_window() {
        self_
          .inner()
          .show_context_menu_for_gtk_window(w.as_ref(), position);
      }

      #[cfg(windows)]
      if let Ok(hwnd) = window.hwnd() {
        unsafe {
          self_
            .inner()
            .show_context_menu_for_hwnd(hwnd.0 as _, position);
        }
      }

      // 新增 OHOS 分支
      #[cfg(target_env = "ohos")]
      {
        // muda::platform_impl::ohos::Menu 通过 build_arkts_menu() 构建菜单数据
        // 调用 openharmony-ability 显示菜单
        self_.inner().show_context_menu_for_ohos_window(position);
      }
    })
  }
}
```

**文件**: `tauri/crates/tauri/src/menu/submenu.rs`

同样在 `popup_inner` 方法中添加 OHOS 分支：

```rust
impl<R: Runtime> ContextMenuBase for Submenu<R> {
  fn popup_inner<T: Runtime, P: Into<crate::Position>>(
    &self,
    window: crate::Window<T>,
    position: Option<P>,
  ) -> crate::Result<()> {
    let position = position.map(Into::into);
    run_item_main_thread!(self, move |self_: Self| {
      // ... macOS, gtk, windows 分支保持不变 ...

      // 新增 OHOS 分支
      #[cfg(target_env = "ohos")]
      {
        self_.inner().show_context_menu_for_ohos_window(position);
      }
    })
  }
}
```

### 5.3 muda OHOS 方法实现

**文件**: `muda/src/platform_impl/ohos/menu.rs`

```rust
impl Menu {
    /// OHOS 专用：显示 context menu
    pub fn show_context_menu_for_ohos_window(&self, position: Option<Position>) -> bool {
        let menu_data = self.build_arkts_menu();
        
        // 调用 openharmony-ability NAPI
        openharmony_ability::menu::popup_context_menu(menu_data, position);
        
        true
    }
}
```

**文件**: `muda/src/platform_impl/ohos/menu_child.rs`

```rust
impl MenuChild {
    /// Submenu 专用：显示 context menu
    pub fn show_context_menu_for_ohos_window(&self, position: Option<Position>) -> bool {
        let menu_data = self.build_arkts_menu();
        
        openharmony_ability::menu::popup_context_menu(menu_data, position);
        
        true
    }
}
```

### 5.4 openharmony-ability NAPI 接口

**文件**: `openharmony-ability/crates/ability/src/menu.rs`

```rust
use napi::bindgen_prelude::*;
use serde::{Deserialize, Serialize};

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArkTsMenuItem {
    pub id: String,
    pub text: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accelerator: Option<String>,
    pub item_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<ArkTsMenuItem>>,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ArkTsMenuData {
    pub items: Vec<ArkTsMenuItem>,
}

#[napi(object)]
pub struct PopupPosition {
    pub x: Option<f64>,
    pub y: Option<f64>,
}

#[napi]
pub fn popup_context_menu(menu_data: ArkTsMenuData, position: Option<PopupPosition>) -> Result<()> {
    // 调用 ArkTS 显示菜单
    // 通过 ArkHelper.popupMenu 调用
    Ok(())
}

#[napi]
pub fn emit_menu_event(menu_id: String) {
    // 发送菜单点击事件到 muda
    muda::platform_impl::ohos::emit_menu_event(menu_id);
}
```

### 5.5 ArkTS 菜单组件

**文件**: `openharmony-ability/native_ability/src/main/ets/components/TauriMenu.ets`

（详细实现见 Phase 4）

### 5.6 验证编译

```bash
# 验证 tauri::menu 在 OHOS 上编译
cd tauri/crates/tauri
cargo build --target aarch64-unknown-linux-ohos

# 应该无错误，确认 muda::* 类型正确导入
```

### 5.7 前端 API 测试用例

> **遵循 frontend-api-testing SKILL 规范**

**文件位置**: `examples/api/src/lib/tests/core.ts`（添加到现有文件）

```typescript
// === Menu 模块测试 ===
// 在 core.ts 末尾添加以下测试用例

import { Menu, MenuItem, Submenu, PredefinedMenuItem } from '@tauri-apps/api/menu';

// P0: 基础 API（category: 'auto'）
{
  name: '@tauri-apps/api/menu.Menu.new',
  category: 'auto',
  async fn() {
    const menu = await Menu.new();
    assert(menu !== undefined, 'Menu.new returned undefined');
  },
},
{
  name: '@tauri-apps/api/menu.MenuItem.new',
  category: 'auto',
  async fn() {
    const item = await MenuItem.new({ text: 'Test Item' });
    const text = await item.text();
    assert(text === 'Test Item', `text mismatch: ${text}`);
  },
},
{
  name: '@tauri-apps/api/menu.Submenu.new',
  category: 'auto',
  async fn() {
    const submenu = await Submenu.new({ text: 'Submenu' });
    assert(submenu !== undefined, 'Submenu.new returned undefined');
  },
},
{
  name: '@tauri-apps/api/menu.Menu.append',
  category: 'auto',
  async fn() {
    const menu = await Menu.new();
    const item = await MenuItem.new({ text: 'Item' });
    await menu.append(item);
    const items = await menu.items();
    assert(items.length === 1, `items.length should be 1, got ${items.length}`);
  },
},

// P1: PredefinedMenuItem（category: 'auto'）
{
  name: '@tauri-apps/api/menu.PredefinedMenuItem.separator',
  category: 'auto',
  async fn() {
    const sep = await PredefinedMenuItem.separator();
    assert(sep !== undefined, 'separator returned undefined');
  },
},
{
  name: '@tauri-apps/api/menu.PredefinedMenuItem.copy',
  category: 'auto',
  async fn() {
    const copy = await PredefinedMenuItem.copy();
    assert(copy !== undefined, 'copy returned undefined');
  },
},
{
  name: '@tauri-apps/api/menu.PredefinedMenuItem.cut',
  category: 'auto',
  async fn() {
    const cut = await PredefinedMenuItem.cut();
    assert(cut !== undefined);
  },
},
{
  name: '@tauri-apps/api/menu.PredefinedMenuItem.paste',
  category: 'auto',
  async fn() {
    const paste = await PredefinedMenuItem.paste();
    assert(paste !== undefined);
  },
},
{
  name: '@tauri-apps/api/menu.PredefinedMenuItem.selectAll',
  category: 'auto',
  async fn() {
    const selectAll = await PredefinedMenuItem.selectAll();
    assert(selectAll !== undefined);
  },
},
{
  name: '@tauri-apps/api/menu.PredefinedMenuItem.undo',
  category: 'auto',
  async fn() {
    const undo = await PredefinedMenuItem.undo();
    assert(undo !== undefined);
  },
},
{
  name: '@tauri-apps/api/menu.PredefinedMenuItem.redo',
  category: 'auto',
  async fn() {
    const redo = await PredefinedMenuItem.redo();
    assert(redo !== undefined);
  },
},

// P2: 窗口操作预定义项（category: 'side-effect'）
{
  name: '@tauri-apps/api/menu.PredefinedMenuItem.minimize',
  category: 'side-effect',
  async fn() {
    const minimize = await PredefinedMenuItem.minimize();
    assert(minimize !== undefined);
    // 窗口最小化可通过窗口状态验证
  },
},
{
  name: '@tauri-apps/api/menu.PredefinedMenuItem.maximize',
  category: 'side-effect',
  async fn() {
    const maximize = await PredefinedMenuItem.maximize();
    assert(maximize !== undefined);
  },
},
{
  name: '@tauri-apps/api/menu.PredefinedMenuItem.closeWindow',
  category: 'side-effect',
  async fn() {
    const close = await PredefinedMenuItem.closeWindow();
    assert(close !== undefined);
  },
},
{
  name: '@tauri-apps/api/menu.PredefinedMenuItem.quit',
  category: 'side-effect',
  async fn() {
    const quit = await PredefinedMenuItem.quit();
    assert(quit !== undefined);
  },
},

// P3: 菜单组合测试
{
  name: '@tauri-apps/api/menu.Menu with submenu',
  category: 'auto',
  async fn() {
    const menu = await Menu.new();
    const submenu = await Submenu.new({ text: 'File' });
    const item = await MenuItem.new({ text: 'Open' });
    await submenu.append(item);
    await menu.append(submenu);
    
    const items = await menu.items();
    assert(items.length === 1, `items.length should be 1`);
  },
},
{
  name: '@tauri-apps/api/menu.Menu with mixed items',
  category: 'auto',
  async fn() {
    const menu = await Menu.new();
    await menu.append(await PredefinedMenuItem.copy());
    await menu.append(await PredefinedMenuItem.separator());
    await menu.append(await MenuItem.new({ text: 'Custom' }));
    
    const items = await menu.items();
    assert(items.length === 3, `items.length should be 3`);
  },
},

// 手动测试（category: 'manual'）- autotest 自动跳过
{
  name: '@tauri-apps/api/menu.Menu.popup',
  category: 'manual',
  async fn() {
    // 手动测试在 TestRunner.svelte 中实现
  },
},
```

### 5.5 手动测试实现

> **遵循 frontend-api-testing SKILL 规范**
> 手动测试使用 `wrapManual()` 包装，结果自动写入 `console-log.txt`

**文件位置**: `examples/api/src/views/TestRunner.svelte`

```typescript
// 在 TestRunner.svelte 中添加以下 handler

import { Menu, MenuItem, PredefinedMenuItem } from '@tauri-apps/api/menu';

async function manualMenuPopup() {
  await wrapManual('menu.popup', async () => {
    const menu = await Menu.new({
      items: [
        await PredefinedMenuItem.copy({ text: '复制' }),
        await PredefinedMenuItem.paste({ text: '粘贴' }),
        await PredefinedMenuItem.separator(),
        await MenuItem.new({ text: '自定义操作' }),
        await PredefinedMenuItem.minimize({ text: '最小化' }),
      ]
    });
    await menu.popup();
    manualResult = 'Menu.popup() - should show menu at cursor';
    onMessage(manualResult);
  });
}

async function manualPredefinedCopy() {
  await wrapManual('menu.copy', async () => {
    // 前置条件：在 WebView 中选中文字
    const copy = await PredefinedMenuItem.copy();
    const menu = await Menu.new({ items: [copy] });
    await menu.popup();
    manualResult = 'Click "Copy" - verify clipboard has selected text';
    onMessage(manualResult);
  });
}

async function manualPredefinedMinimize() {
  await wrapManual('menu.minimize', async () => {
    const minimize = await PredefinedMenuItem.minimize();
    const menu = await Menu.new({ items: [minimize] });
    await menu.popup();
    manualResult = 'Click "Minimize" - window should minimize';
    onMessage(manualResult);
  });
}
```

```svelte
<!-- Manual Tests 区域添加按钮 -->
<div class="manual-tests">
  <h3>Menu Manual Tests</h3>
  <button class="btn" onclick={manualMenuPopup}>
    Menu.popup (should show menu)
  </button>
  <button class="btn" onclick={manualPredefinedCopy}>
    Copy (should copy selected text)
  </button>
  <button class="btn" onclick={manualPredefinedMinimize}>
    Minimize (should minimize window)
  </button>
</div>
```

### 5.6 Rust UT 实现

> **遵循 ohos-rust-ut SKILL 规范**

**测试位置**: `muda/src/platform_impl/ohos/` 各模块的 #[cfg(test)] 部分

**muda/src/platform_impl/ohos/menu.rs**:
```rust
#[cfg(all(test, target_env = "ohos"))]
mod tests {
    use super::*;
    
    #[test]
    fn test_menu_new() {
        let menu = Menu::new(None);
        assert!(menu.id().as_ref().len() > 0);
    }
    
    #[test]
    fn test_menu_build_arkts() {
        let menu = Menu::new(None);
        let data = menu.build_arkts_menu();
        assert_eq!(data.items.len(), 0);
    }
}
```

**muda/src/platform_impl/ohos/menu_child.rs**:
```rust
#[cfg(all(test, target_env = "ohos"))]
mod tests {
    use super::*;
    
    #[test]
    fn test_menu_child_new() {
        let item = MenuChild::new("Test", true, None, None);
        assert_eq!(item.text(), "Test");
        assert!(item.is_enabled());
    }
    
    #[test]
    fn test_menu_child_to_arkts() {
        let item = MenuChild::new("Test", true, None, None);
        let arkts = item.to_arkts_menu_item();
        assert_eq!(arkts.text, "Test");
        assert!(arkts.enabled);
        assert_eq!(arkts.item_type, "item");
    }
    
    #[test]
    fn test_submenu_nested() {
        let mut submenu = MenuChild::new_submenu("File", true, None);
        // 测试嵌套结构
        assert!(submenu.children.is_some());
    }
}
```

**运行命令**:
```bash
# 运行 muda OHOS 后端 UT
cargo test --target aarch64-unknown-linux-ohos -p muda

# 输出示例
# running 6 tests
# test platform_impl::ohos::menu::tests::test_menu_new ... ok
# test platform_impl::ohos::menu::tests::test_menu_build_arkts ... ok
# test platform_impl::ohos::menu_child::tests::test_menu_child_new ... ok
# test platform_impl::ohos::menu_child::tests::test_menu_child_to_arkts ... ok
# test result: ok. 6 passed; 0 failed; 0 ignored
```

### 5.7 测试执行流程

> **遵循 ohos-build SKILL 规范**

**一键测试**（推荐）：
```bash
bash D:/workspace/tauri/tauri/.claude/skills/ohos-build/scripts/run-tests.sh
```

**分步流程**：
```bash
# 1. 禁用 hvigorfile.ts 中的 tauriPlugin
#    编辑 examples/api/src-tauri/gen/ohos/entry/hvigorfile.ts
#    将 plugins:[tauriPlugin()] 改为 plugins:[]

# 2. 构建（启用 AUTOTEST）
export VITE_AUTOTEST=true
bash D:/workspace/tauri/tauri/.claude/skills/ohos-build/scripts/build-ohos.sh

# 3. 签名安装
bash D:/workspace/tauri/tauri/.claude/skills/ohos-build/scripts/sign-and-install.sh

# 4. 等待测试完成（约 10-15 秒）

# 5. 拉取测试报告（使用 cmd.exe 避免 Git Bash 路径转义）
cmd.exe /c "hdc file recv /data/app/el2/100/base/com.tauri.api/cache/test-report.json D:\workspace\tauri\tauri\examples\api\test-report.json"

# 6. 拉取手动测试日志
cmd.exe /c "hdc file recv /data/app/el2/100/base/com.tauri.api/cache/console-log.txt D:\workspace\tauri\tauri\examples\api\console-log.txt"

# 7. 运行 Rust UT（可选）
bash D:/workspace/tauri/tauri/.claude/skills/ohos-rust-ut/scripts/run-ut.sh menu::ohos

# 8. 恢复 hvigorfile.ts
#    将 plugins:[] 改回 plugins:[tauriPlugin()]
```

### 5.8 测试报告格式

**自动测试报告** (`test-report.json`)：
```json
{
  "timestamp": "2026-05-14T05:20:47.253Z",
  "total": 18,
  "passed": 15,
  "failed": 2,
  "skipped": 1,
  "results": [
    { "name": "@tauri-apps/api/menu.Menu.new", "category": "auto", "status": "pass", "duration": 10 },
    { "name": "@tauri-apps/api/menu.Menu.popup", "category": "manual", "status": "skip" },
    { "name": "@tauri-apps/api/menu.PredefinedMenuItem.copy", "category": "auto", "status": "fail", "error": "WebView not ready" }
  ]
}
```

**手动测试日志** (`console-log.txt`)：
```
[menu.popup] Menu.popup() - should show menu at cursor
[menu.copy] Click "Copy" - verify clipboard has selected text
[menu.minimize] Click "Minimize" - window should minimize
```

**Rust UT 输出**：
```
running 4 tests
test menu::ohos::tests::test_menu_id_format ... ok
test menu::ohos::tests::test_menu_item_data_to_json ... ok
test menu::ohos::tests::test_predefined_type_serialization ... ok
test menu::ohos::tests::test_submenu_nested_items ... ok
test result: ok. 4 passed; 0 failed; 0 ignored
```

### 5.9 相关 SKILL 文档

- [ohos-rust-ut](../../.claude/skills/ohos-rust-ut/SKILL.md) - Rust UT 执行
- [frontend-api-testing](../../.claude/skills/frontend-api-testing/SKILL.md) - 前端测试编写规范
- [ohos-build](../../.claude/skills/ohos-build/SKILL.md) - OHOS 构建和安装流程

## 验证清单

### 5.10 API 功能验证

| API | 验证项 | 预期结果 |
|-----|--------|---------|
| `Menu.new()` | 创建菜单 | 返回 Menu 实例 |
| `Menu.append()` | 添加菜单项 | items() 返回正确数量 |
| `Menu.popup()` | 弹出菜单 | 菜单在指定位置显示 |
| `MenuItem.new()` | 创建菜单项 | 返回 MenuItem 实例 |
| `Submenu.new()` | 创建子菜单 | 返回 Submenu 实例 |
| `PredefinedMenuItem.copy()` | 创建复制项 | 点击执行复制操作 |
| `PredefinedMenuItem.minimize()` | 创建最小化项 | 点击最小化窗口 |
| `PredefinedMenuItem.quit()` | 创建退出项 | 点击退出应用 |

### 5.9 端到端场景验证

1. **右键菜单场景**：
   - WebView 中选中文字
   - 右键弹出菜单
   - 点击复制，验证文字已复制

2. **窗口操作场景**：
   - 创建包含 minimize/maximize/close 的菜单
   - 点击各项，验证窗口状态变化

3. **自定义菜单场景**：
   - 创建完全自定义的菜单
   - 点击自定义项，验证回调触发

## 工期
- 2-3 天

## 输出物
- `menu/mod.rs` - OHOS cfg 配置
- `menu/ohos.rs` - Tauri API OHOS 实现
- `Cargo.toml` - 条件编译配置
- `tests/menu.ts` - API 测试用例
- `test-reporter.ts` - 测试报告生成
- 测试报告（JSON）
- 所有测试通过

## 参考文档
- [Menu 组件](../reference/menu.md) - 完整 Menu 示例代码
- [MenuItem 组件](../reference/menu_item.md) - MenuItem 事件处理
- [bindContextMenu](../reference/menu_control.md) - 示例 7 状态变量弹出菜单
- [promptAction](../reference/prompt_action.md) - 可选的 ActionMenu 方案（备选）
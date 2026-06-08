# OHOS 主窗口 WebView 透明穿透分析

**日期**: 2026-06-06
**设备**: HAD-W32 (HUAWEI MateBook Pro, 2in1, API 24, HarmonyOS 6.1.0.130)
**结论**: OHOS 主窗口的 Web 引擎渲染表面不支持透明，仅 Float 子窗口可完整穿透到桌面。

---

## 1. 问题描述

在 Tauri OHOS 应用中，尝试通过 `setBackgroundColor()` 让主窗口的 WebView 内容区透明穿透到桌面，但内容区始终显示为黑色，无法看到桌面。而相同操作在 Float 子窗口中可以正常工作。

## 2. 透明穿透的三层链路

实现窗口内容穿透到桌面，需要三层全部透明：

```
┌─ 层3: HTML body { background: transparent }         ← JS 可控制 ✅
│
├─ 层2: Web 组件 .backgroundColor(Color.Transparent)   ← 仅创建时生效 ⚠️
│         + renderMode: SYNC_RENDER
│
└─ 层1: OHOS 窗口壳层 setWindowBackgroundColor('#00000000')  ← 运行时可控 ✅
          + setWindowContainerColor (API 20+, 2in1/PC 设备)
```

## 3. 主窗口 vs Float 子窗口对比

| 因素 | Float 子窗口 | 主窗口 (UIAbility) |
|------|-------------|-------------------|
| 窗口类型 | `TypeFloat` (8) | `TypeApp` (0) |
| 系统合成 | 特殊透明处理 | 常规不透明合成 |
| `setWindowBackgroundColor` | ✅ 生效 | ✅ 生效（壳层透明） |
| `setWindowContainerColor` | ✅ 生效 | ⚠️ inactive alpha 被系统强制为 FF |
| Web 组件 `.backgroundColor(Color.Transparent)` | ✅ 透明 | ❌ 渲染面仍不透明 |
| `renderMode: SYNC_RENDER` | ✅ 支持透明渲染 | ❌ 主窗口中不生效 |
| 最终效果 | 三层全透，看到桌面 ✅ | 壳层透明但 Web 内容区黑色 ❌ |

## 4. 根因分析

### 4.1 OHOS Web 引擎渲染面限制

OHOS 的 Web 组件基于 Chromium，其渲染表面（Surface）在主窗口（UIAbility）中**始终为不透明**。即使：
- ArkUI 层面设置了 `.backgroundColor(Color.Transparent)`
- 设置了 `renderMode: RenderMode.SYNC_RENDER`
- 在 `loadContent()` 之后调用了 `setWindowBackgroundColor('#00000000')`

Web 引擎内部的渲染面仍然是纯色背景，无法穿透到下层窗口。

### 4.2 系统层面的差异

Float 子窗口（`window.WindowType.TYPE_FLOAT`）在 OHOS 系统合成时有**特殊的透明处理通道**，使得 Web 引擎的透明设置能够传递到系统合成层。而主窗口（`TYPE_APP`）走的是常规合成路径，不传递 Web 组件的透明属性。

### 4.3 containerColor 的限制

在 2in1/PC 设备上，`setWindowContainerColor` 用于设置窗口容器背景色（覆盖标题栏+内容区），但：
- **inactive 颜色的 alpha 被系统强制为 `#FF`**（不透明）
- 传入 `('#80000000', '#80000000')` 会报错：`inactive alpha value error` (code=401)
- 传入 `('#00000000', '#FF000000')` 可以通过，但窗口失焦时仍变为不透明

### 4.4 官方文档确认

OHOS 官方文档描述 `setWindowBackgroundColor` 的作用范围是**窗口壳层**（window shell），不涉及 Web 组件内部的渲染面。Web 组件的透明渲染依赖于 ArkUI 声明式属性 `.backgroundColor()`，该属性：
- **仅在创建时生效**
- **运行时修改不更新 Web 引擎背景**（`BuilderNode.update()` 不会让 Web 组件重新应用 `.backgroundColor()`）
- 在主窗口中的 `SYNC_RENDER` 模式无法实现透明效果

## 5. 尝试过的方案及结果

| 方案 | 结果 | 原因 |
|------|------|------|
| `.transparent(true)` + `SYNC_RENDER` | ❌ Web 内容区黑色 | Web 引擎渲染面在主窗口中不透明 |
| `initMainWindow` NAPI 在 `Window::new()` 中调用 | ❌ 无效 | Rust 侧延迟调用，Web 引擎已按不透明初始化 |
| `NativeAbility.onWindowStageCreate` 中设透明 | ❌ 无效 | 在 `loadContent()` 之前调用，被覆盖 |
| `MainPage.build()` 4 层容器加 `Color.Transparent` | ❌ 无效 | 容器透明不影响 Web 引擎渲染面 |
| FrameNode `commonAttribute.backgroundColor()` | ⚠️ 可能无效 | Web 组件的 FrameNode 可能是代理节点 |
| JS 注入移除所有不透明 CSS | ❌ 无效 | HTML 透明了但 Web 引擎渲染面仍不透明 |
| 始终 `init.transparent = true` (ArkHelper.ets) | ❌ 无效 | WebView 创建时透明了但引擎渲染面仍不透明 |

## 6. 最终方案

### 接受平台限制，使用 Float 子窗口验证透明穿透

- **Float 子窗口**：`transparent: true` + `decorations: false`，创建时设置窗口壳层透明 + Web 组件 `Color.Transparent` + `SYNC_RENDER`，可实现完整穿透
- **主窗口**：保留 `set_background_color` 运行时修改窗口壳层背景色的能力（用于 Float 子窗口），不再尝试主窗口透明

### 代码修改

| 文件 | 修改 |
|------|------|
| `tao/.../ohos/mod.rs` | `set_background_color` 运行时不被 `transparent` 标志锁死颜色 |
| `openharmony-ability/.../ArkHelper.ets` | WebView 创建时始终透明（防御性编程） |
| `lib.rs` | 主窗口 `.transparent(true)`（Web 组件创建时透明，但引擎限制导致不穿透） |
| `TransparencyTest.svelte` | 仅保留 Float 子窗口测试卡片 |

## 7. 如果未来 OHOS 支持主窗口 Web 透明

需要同时满足以下条件：
1. `loadContent()` 之后调用 `setWindowBackgroundColor('#00000000')`
2. `setWindowContainerColor('#00000000', '#FF000000')`（inactive 必须 FF）
3. Web 组件 `.backgroundColor(Color.Transparent)` + `RenderMode.SYNC_RENDER`
4. HTML `body { background: transparent }`
5. 系统合成层支持主窗口 Web 引擎透明渲染（当前不支持）

# Menu/Tray 模块 OHOS 空实现方案

> 版本：v1.0
> 更新时间：2026-05-14
> 目标：让 tauri 在 ohos 上编译通过，menu/tray API 返回 Unsupported 错误

---

## 背景

当前 tauri 的 menu/tray 模块依赖 `muda` 和 `tray-icon` crate，这两个 crate 在 Cargo.toml 中排除了 ohos 平台，导致 ohos 编译失败。

**方案 A**（旧方案）：在 tauri 层做大量 cfg 隔离 + 空实现
**方案 B**（本方案）：在 muda/tray-icon 内部提供 ohos 空实现，tauri 层无需修改

---

## 进度追踪

| Phase | 设计文档 | 进度文档 | 状态 |
|-------|----------|----------|------|
| Overall | [overall-design.md](./overall-design.md) | [overall-progress.md](./overall-progress.md) | 🔄 In Progress |
| Phase 0 | [impl/phase0-muda-stub-design.md](./impl/phase0-muda-stub-design.md) | [impl/phase0-muda-stub-progress.md](./impl/phase0-muda-stub-progress.md) | ⬜ Not Started |
| Phase 1 | [impl/phase1-tray-icon-stub-design.md](./impl/phase1-tray-icon-stub-design.md) | [impl/phase1-tray-icon-stub-progress.md](./impl/phase1-tray-icon-stub-progress.md) | ⬜ Not Started |
| Phase 2 | [impl/phase2-tauri-adaptation-design.md](./impl/phase2-tauri-adaptation-design.md) | [impl/phase2-tauri-adaptation-progress.md](./impl/phase2-tauri-adaptation-progress.md) | ⬜ Not Started |

---

## 目录结构

```
ohos_feature/
├── README.md                           # 本文档（索引）
├── overall-design.md                   # 总体设计文档
├── overall-progress.md                 # 总体进度追踪
├── impl/
│   ├── phase0-muda-stub-design.md      # Phase 0：muda 空实现设计
│   ├── phase0-muda-stub-progress.md    # Phase 0：进度追踪
│   ├── phase1-tray-icon-stub-design.md # Phase 1：tray-icon 空实现设计
│   ├── phase1-tray-icon-stub-progress.md # Phase 1：进度追踪
│   ├── phase2-tauri-adaptation-design.md   # Phase 2：tauri 适配设计
│   └── phase2-tauri-adaptation-progress.md # Phase 2：进度追踪
└── menu_tray_stub_impl_plan.md         # 旧文档（待删除）
```

---

## Phase 0：muda 空实现

在 muda crate 内部提供 ohos 平台的空实现。

| 模块 | 设计文档 | 职责 | 独立验证 |
|------|----------|------|----------|
| muda::platform_impl::ohos | [impl/phase0-muda-stub-design.md](./impl/phase0-muda-stub-design.md) | Menu/MenuItem/Submenu 等空实现 | ✓ cargo build |

**完成后**：通知 Phase 1 开始

---

## Phase 1：tray-icon 空实现

在 tray-icon crate 内部提供 ohos 平台的空实现。

| 模块 | 设计文档 | 职责 | 独立验证 |
|------|----------|------|----------|
| tray-icon::platform_impl::ohos | [impl/phase1-tray-icon-stub-design.md](./impl/phase1-tray-icon-stub-design.md) | TrayIconBuilder/TrayIcon 空实现 | ✓ cargo build |

**完成后**：通知 Phase 2 开始

---

## Phase 2：tauri 适配

移除 tauri Cargo.toml 中对 muda/tray-icon 的 ohos 排除条件。

| 模块 | 设计文档 | 职责 | 独立验证 |
|------|----------|------|----------|
| tauri Cargo.toml | [impl/phase2-tauri-adaptation-design.md](./impl/phase2-tauri-adaptation-design.md) | 移除 `not(target_env = "ohos")` 条件 | ✓ cargo build |

**完成后**：运行 autotest 验收

---

## 依赖关系

```
┌─────────────────────────────────────────────────────┐
│  tauri                                              │
│    ├── tauri::menu (无需修改)                        │
│    │     └── 直接使用 muda API                       │
│    └── tauri::tray (无需修改)                        │
│          └── 直接使用 tray-icon API                  │
└─────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────┐
│  muda crate (Phase 0)                               │
│    └── platform_impl::ohos                          │
│          ├── Menu::new() → Error::Unsupported       │
│          ├── MenuItem::new() → Error::Unsupported   │
│          ├── MenuId (正常定义)                       │
│          └── MenuEvent (正常定义)                    │
└─────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────┐
│  tray-icon crate (Phase 1)                          │
│    └── platform_impl::ohos                          │
│          ├── TrayIconBuilder::build() → Error       │
│          ├── TrayIcon (正常定义)                     │
│          └── TrayIconId (正常定义)                   │
└─────────────────────────────────────────────────────┘
```

---

## 实现顺序

| Phase | Step | 模块 | 独立验证 |
|-------|------|------|----------|
| 0 | 1 | muda 空实现 | ✓ cargo build muda |
| 1 | 2 | tray-icon 空实现 | ✓ cargo build tray-icon |
| 2 | 3 | tauri Cargo.toml | ✓ cargo build tauri |

---

## 验收标准

| 条件 | 要求 |
|------|------|
| 编译成功 | mobile 和 desktop 模式均无 Rust 编译错误 |
| HAP 安装 | 签名安装成功，应用能启动 |
| autotest | `failed = 0`（menu/tray API 返回 Unsupported 为预期） |

---

## 与真实实现的关系

本方案为**空实现**，仅解决编译问题。后续真实实现见：

- `doc/menu/` - menu 模块真实实现
- `doc/tray/` - tray 模块真实实现

空实现完成后，真实实现可渐进替换。

---

## 关键决策

| 决策 | 原因 |
|------|------|
| 空实现在底层 crate | tauri 层无需修改，架构更清晰 |
| 类型定义正常提供 | API 可调用，仅操作返回错误 |
| 错误使用 Unsupported | 明确告知用户该平台不支持 |
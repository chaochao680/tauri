# Tray 模块 OHOS 适配 - API 维度设计文档

> 版本：v2.0
> 更新时间：2026-05-20

---

## 总体设计

完整设计见 [overall-design.md](./overall-design.md)，包含：
- 架构分层设计
- 数据流设计
- 接口映射对照
- 关键设计决策
- 风险与限制
- 实现路线图

---

## 进度追踪

| Phase | 设计文档 | 进度文档 | 状态 |
|-------|----------|----------|------|
| Overall | [overall-design.md](./overall-design.md) | [overall-progress.md](./overall-progress.md) | ✅ 98% |
| Phase 0 | [impl/phase0-muda-ohos-design.md](./impl/phase0-muda-ohos-design.md) | [impl/phase0-muda-ohos-progress.md](./impl/phase0-muda-ohos-progress.md) | ✅ Done (100%) |
| Phase 1 | [impl/phase1-ohos-statusbar-design.md](./impl/phase1-ohos-statusbar-design.md) | [impl/phase1-ohos-statusbar-progress.md](./impl/phase1-ohos-statusbar-progress.md) | ✅ Done (100%) |
| Phase 2 | [impl/phase2-TrayIconBuilder-design.md](./impl/phase2-TrayIconBuilder-design.md) | [impl/phase2-TrayIconBuilder-progress.md](./impl/phase2-TrayIconBuilder-progress.md) | ✅ Done (100%) |
| Phase 3 | [impl/phase3-TrayIconEvent-design.md](./impl/phase3-TrayIconEvent-design.md) | [impl/phase3-TrayIconEvent-progress.md](./impl/phase3-TrayIconEvent-progress.md) | ✅ Done (100%) |
| Phase 4 | [impl/phase4-ohos-statusbar-fix-design.md](./impl/phase4-ohos-statusbar-fix-design.md) | [impl/phase4-ohos-statusbar-fix-progress.md](./impl/phase4-ohos-statusbar-fix-progress.md) | ✅ P0-P3 Done (69%) |
| Phase 5 | [impl/phase5-tray-testing-design.md](./impl/phase5-tray-testing-design.md) | [impl/phase5-tray-testing-progress.md](./impl/phase5-tray-testing-progress.md) | ✅ 设计+代码 Done (85%) |
| Phase 6 | [impl/phase6-statusbar-tsfn-refactor-design.md](./impl/phase6-statusbar-tsfn-refactor-design.md) | [impl/phase6-statusbar-tsfn-refactor-progress.md](./impl/phase6-statusbar-tsfn-refactor-progress.md) | ✅ Done (100%) |
| Phase 7 | [impl/phase7-predefined-check-icon-design.md](./impl/phase7-predefined-check-icon-design.md) | [impl/phase7-predefined-check-icon-progress.md](./impl/phase7-predefined-check-icon-progress.md) | ✅ Done (100%) |
| Integration | — | [overall-progress.md](./overall-progress.md) §六 | ⬜ Not Started |

**整体进度**：`98%` ✅ 全部功能实现完成，设备验证通过。剩余：集成测试自动化。

## 架构依赖

| 平台 | muda 依赖 | tray-icon 依赖 | 系统接口库 |
|------|-----------|----------------|-----------|
| Windows | windows-sys | windows-sys | Win32 API |
| macOS | objc2-app-kit | objc2-app-kit | Objective-C API |
| Linux | gtk | libappindicator | GTK |
| **OHOS** | openharmony-ability (menu) | openharmony-ability (statusbar) | NAPI 封装 |

---

## 目录结构

```
tray/
├── README.md                    # 本文档（索引）
├── overall-design.md            # 总体设计文档
├── overall-progress.md          # 总体进度追踪
├── api-to-support.md            # 接口需求定义
├── DEBUG.md                     # 问题修复记录（Fix 1-24）
├── impl/                        # 实现设计和进度文档
│   ├── phase0-muda-ohos-design.md       # Phase 0：muda OHOS 后端设计
│   ├── phase0-muda-ohos-progress.md     # Phase 0：进度追踪
│   ├── phase1-ohos-statusbar-design.md  # Phase 1：底层 API 封装设计
│   ├── phase1-ohos-statusbar-progress.md # Phase 1：进度追踪
│   ├── phase2-TrayIconBuilder-design.md  # Phase 2：托盘图标构建器设计
│   ├── phase2-TrayIconBuilder-progress.md # Phase 2：进度追踪
│   ├── phase3-TrayIconEvent-design.md    # Phase 3：托盘事件系统设计
│   ├── phase3-TrayIconEvent-progress.md  # Phase 3：进度追踪
│   ├── phase4-ohos-statusbar-fix-design.md   # Phase 4：StatusBar API 修正
│   ├── phase4-ohos-statusbar-fix-progress.md # Phase 4：进度追踪
│   ├── phase5-tray-testing-design.md     # Phase 5：端到端测试设计
│   ├── phase5-tray-testing-progress.md   # Phase 5：进度追踪
│   ├── phase6-statusbar-tsfn-refactor-design.md   # Phase 6：TSFN 数据传递重构
│   ├── phase6-statusbar-tsfn-refactor-progress.md # Phase 6：进度追踪
│   ├── phase7-predefined-check-icon-design.md   # Phase 7：Predefined/Check/Icon
│   └── phase7-predefined-check-icon-progress.md # Phase 7：进度追踪
└── reference/                   # 参考资料
    ├── tray-reference.md
    ├── status_bar.md
    ├── status_bar_api.md
    └── status_bar_view_extension_ability.md
```

---

## Phase 0：muda OHOS 后端

| 模块 | 设计文档 | 职责 | 独立验证 |
|------|----------|------|----------|
| muda OHOS 后端 | [impl/phase0-muda-ohos-design.md](./impl/phase0-muda-ohos-design.md) | 为 muda crate 实现 OHOS 平台支持 | ✓ auto/manual 测试 |

**完成后**：通知 Phase 1 和 Phase 2 开始

---

## Phase 1：底层 API 封装

| 模块 | 设计文档 | 职责 | 独立验证 |
|------|----------|------|----------|
| OHOS StatusBar API 封装 | [impl/phase1-ohos-statusbar-design.md](./impl/phase1-ohos-statusbar-design.md) | 封装 statusBarManager API | ✓ 独立测试程序 |

**完成后**：通知 Phase 2 开始

---

## Phase 2：API 实现 - TrayIconBuilder

| 接口 | 设计文档 | 说明 | 依赖 |
|------|----------|------|------|
| TrayIconBuilder | [impl/phase2-TrayIconBuilder-design.md](./impl/phase2-TrayIconBuilder-design.md) | 托盘图标构建器 | phase1 |

**完成后**：通知 Phase 3 开始

---

## Phase 3：API 实现 - TrayIconEvent

| 接口 | 设计文档 | 说明 | 依赖 |
|------|----------|------|------|
| TrayIconEvent | [impl/phase3-TrayIconEvent-design.md](./impl/phase3-TrayIconEvent-design.md) | 托盘事件系统 | phase2 |
| MouseButton | 见 TrayIconEvent 文档附录 | 鼠标按钮枚举 | - |
| MouseButtonState | 见 TrayIconEvent 文档附录 | 鼠标按钮状态 | - |

---

## Phase 4：StatusBar API 修正与增强

| 模块 | 设计文档 | 说明 | 依赖 |
|------|----------|------|------|
| P0: set_ohos_app / 宏死锁修复 | [impl/phase4-ohos-statusbar-fix-design.md](./impl/phase4-ohos-statusbar-fix-design.md) | 修复初始化和死锁问题 | phase1 |
| P1: createPixelMap 修正 | 同上 | 改用 createImageSource 链式调用 | phase1 |
| P2-P3: menu_code / example app | 同上 | 事件增强和示例启用 | phase2 |
| P4-P7: 增量 API | 同上 | 待设备验证 | phase1 |

---

## Phase 5：端到端测试

| 模块 | 设计文档 | 说明 | 依赖 |
|------|----------|------|------|
| Auto 测试 (16 个) | [impl/phase5-tray-testing-design.md](./impl/phase5-tray-testing-design.md) | tray.ts 测试用例 | phase2 |
| Manual 测试 (18 个) | 同上 | 人工验证用例 | phase2 |

---

## Phase 6：StatusBar TSFN 数据传递重构

| 模块 | 设计文档 | 说明 | 依赖 |
|------|----------|------|------|
| TSFN 泛型数据传递 | [impl/phase6-statusbar-tsfn-refactor-design.md](./impl/phase6-statusbar-tsfn-refactor-design.md) | 删除 15 个 DATA_* Mutex | phase5 |

---

## Phase 7：Predefined/Check/Icon 支持

| 模块 | 设计文档 | 说明 | 依赖 |
|------|----------|------|------|
| Predefined 动作 | [impl/phase7-predefined-check-icon-design.md](./impl/phase7-predefined-check-icon-design.md) | minimize/maximize/close/quit | phase6 |
| Check toggle | 同上 | 菜单项选中状态切换 | phase6 |
| Icon 项 | 同上 | PNG decode + PixelMap | phase6 |

---

## 依赖关系

```
┌─────────────────────────────────────────────────────┐
│  tauri::tray                                        │
│    ├── TrayIconBuilder → build() 创建图标           │
│    ├── TrayIconEvent   → receiver() 接收事件        │
│    ├── MouseButton     → Left/Right/Middle          │
│    └── MouseButtonState → Up/Down                   │
└─────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────┐
│  tray-icon crate                                    │
│    └── platform_impl::ohos                          │
│          ├── TrayIcon::new()                        │
│          ├── TrayIcon::set_icon/menu/visible        │
│          └── TrayIconEvent::send()                  │
└─────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────┐
│  muda::platform_impl::ohos (Phase 0)                │
│    ├── Menu / MenuItem / Submenu                    │
│    ├── ContextMenu trait                            │
│    └── MenuEvent                                    │
└─────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────┐
│  openharmony-ability::statusbar (Phase 1)           │
│    ├── add_to_status_bar()                          │
│    ├── remove_from_status_bar()                     │
│    ├── update_status_bar_icon/menu                  │
│    ├── register_icon_click_handler()                │
│    ├── register_menu_click_handler()                │
│    ├── icon_click_receiver()                        │
│    └── menu_click_receiver()                        │
└─────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────┐
│  OHOS @kit.DeskTopExtensionKit::statusBarManager    │
│    ├── statusBarIconClick (左键点击图标)             │
│    └── rightMenuClick (右键点击菜单项)               │
└─────────────────────────────────────────────────────┘
```

---

## 实现顺序

| Phase | Step | 模块 | 独立验证 |
|-------|------|------|----------|
| 0 | 1 | muda OHOS 后端 | ✓ Menu/MenuItem API |
| 1 | 2 | OHOS StatusBar API 封装 | ✓ 独立测试程序 |
| 2 | 3 | TrayIconBuilder | ✓ 创建/更新/移除图标 |
| 3 | 4 | TrayIconEvent | ✓ 点击事件触发和接收 |

---

## 参考资料

- [reference/tray-reference.md](./reference/tray-reference.md) - OHOS 系统托盘官方示例
- [reference/status_bar_api.md](./reference/status_bar_api.md) - OHOS statusBarManager API 完整文档
- [reference/status_bar_view_extension_ability.md](./reference/status_bar_view_extension_ability.md) - StatusBarViewExtensionAbility
- [api-to-support.md](./api-to-support.md) - Tauri tray 模块接口需求
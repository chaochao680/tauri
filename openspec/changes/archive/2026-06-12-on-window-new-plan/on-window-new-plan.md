# on-window-new 适配计划

**创建时间**：2026-06-10
**功能描述**：OHOS 平台新窗口请求拦截——打通 `on_new_window` 从 ArkTS → openharmony-ability → wry → tauri-runtime-wry → tauri 用户 API 的完整链路
**判断依据**：涉及 4 个代码层，预估 11 个文件

## Phase 列表

| Phase | 名称 | openspec change | 状态 | 涉及层 | 预估文件 | 验证方式 |
|-------|------|----------------|------|--------|---------|---------|
| 1 | openharmony-ability NAPI + ArkTS + wry + tauri 全链路 | p1-on-window-new | ✓ 设计完成 | openharmony-ability + wry + tauri-runtime-wry + tauri | 11 | cargo check + 设备端测试 |
| 2 | Create 变体 + OS 级窗口 | p2-on-window-new | ○ 待开始 | tauri + tao + openharmony-ability | ~15 | OS 窗口创建 + Tauri 管理 |
| 3 | onWindowNewExt 增强 | p3-on-window-new | ○ 待开始 | openharmony-ability + wry | ~5 | WindowFeatures 信息传递 |

## Phase 详细说明

### Phase 1: 全链路拦截 (Allow/Deny)
- **目标**: 打通 onWindowNew 事件从 ArkTS 到 Tauri 用户 API 的完整链路，支持 Deny 和 Allow
- **openspec change**: `p1-on-window-new`
- **文件列表**: openharmony-ability (6), wry (2), tauri-runtime-wry (1), tauri (1), examples (1)
- **依赖**: 无
- **状态**: ✓ 设计完成 — 所有 artifact 已生成，可使用 tauri-ohos-apply Skill 开始实现

### Phase 2: Create 变体 + OS 级窗口
- **目标**: 支持 `NewWindowResponse::Create { window }`，使用 OS 级窗口创建（`@ohos.window.createWindow`）
- **依赖**: Phase 1 完成 + `ohos-os-level-window-design.md` 中的 WindowManager 实现
- **状态**: ○ 待开始 — 依赖 OS 级窗口基础设施

### Phase 3: onWindowNewExt 增强
- **目标**: 使用 `onWindowNewExt` (API 12+) 获取 `NavigationPolicy` / `WindowFeatures`，映射到 Tauri `NewWindowFeatures.size/position`
- **依赖**: Phase 1 完成
- **状态**: ○ 待开始

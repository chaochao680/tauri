# Phase 7: StatusBar Menu Predefined/Check/Icon — Progress

## Status: ✅ 设备验证通过

## Checklist

- [x] Step 1: 扩展 MenuJsonItem + strip `&` + 传递 checked/icon
- [x] Step 2: 添加 MENU_METADATA static
- [x] Step 3: 修改 event loop 区分 predefined/check/regular
- [x] Step 4: 新增 executePredefinedAction TSFN (openharmony-ability)
- [x] Step 5: ArkTS executePredefinedAction helper
- [x] Step 6: Check toggle 实现
- [x] Step 7: Icon 项支持 (PNG decode + PixelMap)
- [x] Step 8: 依赖变更 (base64)
- [x] 构建验证 (OHOS + Windows)
- [x] OHOS 设备 UT (14 passed)
- [x] 设备手动验证

## 遗留问题

（无 — 全部已解决）

## Log

### 2026-05-20 (下午)

- 修复 predefined minimize/hide/close 不生效问题（Fix 24）：
  - 根因：系统 abilityName 窗口激活与 minimize 异步竞争，minimize 被激活覆盖
  - 修复：minimize/hide/close 使用 setTimeout(300ms) 延迟执行，等系统激活完成后再操作
  - close 改为 minimize（OHOS 无法 destroyWindow 后恢复窗口，minimize 是"close to tray"的等价行为）
- 确认 notifyOnly: true 不能阻止系统窗口激活（OHOS 平台行为）
- 确认功能等价组：maximize=fullscreen, minimize=hide=close
- 更新设计文档：添加功能等价说明、abilityName 机制、跨平台对比
- 设备验证通过：所有 predefined 动作正常工作

- 设备手动验证通过：Quit 退出、Minimize 最小化、菜单文本无 `&` 符号
- 修复 menu click 事件不触发的核心问题：
  - 根因：`Function::call()` 在 render() 同步上下文中静默失败
  - 修复：`helper_obj.set("_onMenuClick", closure)` 绕过 Function::call，ArkTS 侧 setTimeout(200ms) 延迟注册
- 修复文件：
  - `openharmony-ability/crates/ability/src/statusbar/manager.rs`: 直接设置回调属性
  - `openharmony-ability/crates/ability/src/statusbar/event.rs`: 公开 sender 访问器
  - `openharmony-ability/native_ability/src/main/ets/components/DefaultXComponent.ets`: helperRef + setTimeout 注册
- 记录遗留问题：fillMenuItemAbilityName 与 notifyOnly 交互（详见 DEBUG.md）

- 完成根因分析和方案设计
- 确认 Windows/macOS 参考实现位置
- 确认已有依赖：`png = "0.18"` (tray-icon OHOS)，需添加 `base64 = "0.22"`
- 确认 ArkTS 侧已有 `createPixelMapFromRgba` 可复用
- 确认 `openharmony_ability::OpenHarmonyApp::exit()` 可用于 quit 动作
- 完成全部实现：
  - `tray-icon/src/platform_impl/ohos/mod.rs`: MenuJsonItem 扩展、strip_mnemonics、decode_png_to_rgba、MENU_METADATA、build_item_options
  - `tray-icon/src/platform_impl/ohos/event.rs`: predefined/check/regular 分发、toggle_check_item、rebuild_and_update_menu
  - `tray-icon/Cargo.toml`: 添加 base64 = "0.22"
  - `openharmony-ability/crates/ability/src/statusbar/types.rs`: StatusBarMenuItemOptions 添加 icon_rgba/icon_size
  - `openharmony-ability/crates/ability/src/statusbar/manager.rs`: executePredefinedAction TSFN + icon_rgba 传递
  - `openharmony-ability/native_ability/src/main/ets/ability/type.ets`: ArkHelper 接口添加 executePredefinedAction
  - `openharmony-ability/native_ability/src/main/ets/components/DefaultXComponent.ets`: executePredefinedAction 实现 + processMenuItemIcons
- OHOS 交叉编译通过，Windows 编译通过
- OHOS 设备 UT 14 tests passed

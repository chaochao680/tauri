## 1. wry OHOS 层

- [x] 1.1 `wry/src/ohos/mod.rs`：`set_bounds` 非子分支**维持 cache-only**（经尝试移除后全屏黑边，已回退）。cache-only 是正确行为：主 webview 经 `"100%"` 宽高填满窗口，set_bounds 仅更新 `bounds_cache`
- [x] 1.2 注释说明：cache-only 原因（setBounds 会把 `"100%"` 替换为具体像素值 → 全屏黑边）；子 webview 仍调 ArkTS setBounds（绝对定位，不受影响）

## 2. R74 透明背景核实

- [x] 2.1 核实 archive `p1-webview-transparent` 全部改动在代码中 → 确认 R74 已闭环，标注关闭

## 3. 编译验证

- [x] 3.1 `cargo check --target aarch64-unknown-linux-ohos --features openharmony-ability/webview`（wry）通过
- [x] 3.2 `cargo check`（host 非 ohos）通过
- [x] 3.3 三环境编译（Windows / OHOS desktop / OHOS mobile）通过

## 4. 设备端验证

- [x] 4.1 全屏无黑边：p3 构建（cache-only 回退后）全屏应用四个方向无黑边（manual，用户确认修复）
- [x] 4.2 cookie 等既有用例无回归（auto，210/212，2 既有失败）

## 5. 文档

- [x] 5.1 manual_tests.md 新增 7.4 "全屏无黑边"回归防护用例（T0）+ 统计表更新（53/54/107）
- [x] 5.2 design/spec 更新：R78 cache-only 为正确行为（非缺陷），全屏回归防护
- [x] 5.3 确认改动仅位于 `cfg(target_env="ohos")` 路径

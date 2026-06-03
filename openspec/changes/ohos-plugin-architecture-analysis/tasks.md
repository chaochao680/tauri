## Action Items

基于架构分析，以下是后续需要执行的任务：

### 高优先级

- [ ] **1. 编写 OHOS 插件适配指南**
  - **位置**：`doc/ohos-plugin-adaptation-guide.md` 或独立 openspec
  - **内容**：
    1. 三种适配模式的判断标准（直接调用 / 平行替换 / 插件注册）
    2. 每种模式的具体步骤和代码模板
    3. 常见陷阱（如 `cfg(desktop)` 在 OHOS 上的行为）
    4. 如何判断插件功能是否已被 openharmony-ability 覆盖
  - **负责人**：OHOS 适配团队
  - **预计工时**：2-3 天

### 中优先级

- [ ] **2. 明确 openharmony-ability 的职责边界**
  - **位置**：`openharmony-ability/README.md`
  - **内容**：添加一节"Scope and Boundaries"，明确声明：
    - 只包含框架级能力（每个 Tauri app 都需要的）
    - 不包含插件级能力（只有用了某个插件才需要的）
    - 判断标准和示例
  - **负责人**：openharmony-ability 维护者
  - **预计工时**：0.5 天

- [ ] **3. 拆分 dialog 插件的 PR**
  - **当前状态**：plugins-workspace PR #6 包含所有修改
  - **改进方案**：拆分为 3 个 PR：
    1. `Cargo.toml` 平台配置 + `build.rs` + `schema.json`
    2. `commands.rs` + `lib.rs` cfg 修改
    3. `mobile.rs` OHOS 注册（依赖 tauri 核心已合入）
  - **负责人**：dialog 插件适配者
  - **预计工时**：1 天

- [ ] **4. 发布包含 OHOS API 的 tauri-plugin 版本**
  - **当前问题**：dialog PR 中 workspace 依赖改为本地路径 `path = "../tauri/crates/tauri-plugin"`
  - **改进方案**：发布 tauri-plugin 新版本，包含 `ohos_path()` 和 `register_ohos_plugin()` API
  - **负责人**：tauri 核心维护者
  - **预计工时**：0.5 天

### 低优先级

- [ ] **5. 优化 tauri PR 的组织方式**
  - **当前问题**：tauri PR #20 混合了插件基础设施、dialog 模板、构建脚本、杂项修复
  - **改进方案**：未来类似 PR 应拆分为：
    1. OHOS 插件基础设施（ohos.rs, ohos_plugin.rs, plugins.rs）← 先合
    2. 构建脚本改进（template detection, plugin injection）← 先合
    3. 具体插件的 ArkTS 模板 ← 后合
    4. 杂项修复 ← 分开
  - **负责人**：OHOS 适配团队
  - **备注**：这是流程改进，适用于未来的大型 PR

- [ ] **6. 评估是否需要 PluginManager 性能监控**
  - **当前状态**：dialog 是低频操作，性能不是瓶颈
  - **未来考虑**：如果出现高频插件（如 sensor、gyroscope），可能需要添加性能追踪
  - **负责人**：OHOS 适配团队
  - **备注**：暂不执行，等问题出现再处理

### 已完成

- [x] **架构分析文档**（本 openspec）
  - 澄清了两个 HAR 的职责边界
  - 总结了三种适配模式
  - 评估了当前架构的合理性

## Dependencies

```
发布 tauri-plugin 新版本 (#4)
         ↓
合并 tauri 核心插件基础设施 PR
         ↓
合并 dialog 插件 PR (#3)
         ↓
编写适配指南 (#1) + 明确 openharmony-ability 边界 (#2)
```

## Success Criteria

- [ ] 后续插件适配者能够根据指南快速选择适配模式
- [ ] dialog PR 拆分后更易 review 和合并
- [ ] openharmony-ability 不再收到"添加 xxx 插件功能"的无关 PR
- [ ] workspace 依赖恢复为 crates.io 版本（非本地路径）

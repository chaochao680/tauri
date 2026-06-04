---
name: tauri-ohos-code-review
description: Tauri OHOS 代码检视。使用场景：(1) committer 需要审查 PR，(2) 对照 OHOS 约束 checklist 审计代码，(3) 编译部署验证功能回归，(4) 提交 review comments 到 GitHub。
---

# Tauri OHOS 代码检视

本技能引导完成代码检视流程：解析 PR → checkout 分支 → 代码审计 + 提交 GitHub → 编译部署 → 清理。

> **适用场景**：committer 收到一个或多个 PR 链接，需要系统性审查 OHOS 适配代码质量。

> **前提条件**：
> - 本地已有所有仓库代码（tauri/tao/wry/muda/tray-icon/openharmony-ability/plugins-workspace）
> - 默认连着鸿蒙 PC（用于编译部署）
> - 已安装并认证 `gh` CLI

## 状态追踪

使用 Claude TaskList 追踪每个 Step 的执行状态。

### Guard: 启动时初始化

**每次 skill 被调用时，首先检查 TaskList**：
- 如果 TaskList 非空 → 找到当前 `in_progress` 的 task，从该 step 继续
- 如果 TaskList 为空 → 立即创建以下 task（不可跳过）：

```
TaskCreate: "Step 1: 解析 PR + 检查环境"
TaskCreate: "Step 2: Checkout 各仓 PR 分支"
TaskCreate: "Step 3: 代码检视 + 提交 GitHub"
TaskCreate: "Step 4: 编译部署"
TaskCreate: "Step 5: 清理本地分支"
```

创建后 TaskUpdate 第一个为 `in_progress`，开始执行。

## 步骤

### Step 1: 解析 PR + 检查环境

#### 1a. 解析 PR 链接

从用户输入中提取 PR 信息。支持格式：
- 完整 URL: `https://github.com/Eulogizethesun/tauri/pull/25`
- 简写: `tauri#25` 或 `#25`（默认 tauri 仓库）
- 多个 PR: `tauri#25, tao#8, wry#12, plugins-workspace#6`

对每个 PR，使用 `gh pr view` 获取元信息：

```bash
gh pr view <N> --repo Eulogizethesun/<repo> --json title,body,headRefName,files
```

**解析 PR body 中的关联 PR**：
- 检查 body 是否包含其他仓的 PR 链接（如 `Related: tao#8`）
- 如果发现关联 PR 未在输入中，提示用户补充

#### 1b. 检查 gh CLI

```bash
gh auth status
```

如果未认证，提示用户：
```
请先运行 `gh auth login` 完成认证，然后重新调用 skill。
```

#### 1c. 汇总 PR 列表

输出解析结果：
```
## PR List
1. tauri#25: feat: add ohos_base_path() (review/pr-25)
2. tao#8: fix: tray icon positioning (review/pr-8)
3. wry#12: refactor: event loop (review/pr-12)

涉及仓库: tauri, tao, wry, plugins-workspace
```

### Step 2: Checkout 各仓 PR 分支

对每个涉及的仓库执行以下操作：

#### 2a. 保存当前工作区

```bash
cd D:\workspace\tauri\<repo>
git stash -u  # 保存 uncommitted 改动（包括 untracked）
```

如果 stash 失败（极少见），提示用户手动处理：
```
<repo> 仓库有未提交的改动，请手动 commit 或清理后重新调用 skill。
```

#### 2b. Fetch PR 分支

```bash
git fetch origin pull/<N>/head:review/pr-<N>
```

如果 fetch 失败（PR 不存在或已删除），标记该 PR 为 "❌ Fetch Failed"，跳过该仓库。

#### 2c. Checkout

```bash
git checkout review/pr-<N>
```

如果 checkout 失败（冲突），提示用户：
```
<repo> 仓库 checkout 冲突，请手动解决后重新调用 skill。
```

#### 2d. 记录原始分支

保存每个仓库的原始分支名（通常是 `ohdev`），用于 Step 5 清理：

```
原始分支记录:
  tauri → ohdev
  tao → ohdev
  wry → ohdev
```

#### 2e. 验证 checkout 成功

对每个仓库执行 `git branch --show-current`，确认在 `review/pr-<N>` 分支。

输出：
```
## Checkout Complete
✅ tauri → review/pr-25
✅ tao → review/pr-8
✅ wry → review/pr-12
```

### Step 3: 代码检视 + 提交 GitHub（多轮迭代）

检视分为多轮，每轮侧重不同，**直到连续 2 轮无新发现为止**（loop-until-dry）。

#### 3a. Round 1: Diff 扫描 + Checklist 快速检查

**目标**：快速扫描 PR diff，发现明显违规。

1. 获取每个 PR 的 diff：
   ```bash
   gh pr diff <N> --repo Eulogizethesun/<repo> > review-diff-<repo>.diff
   ```

2. 按文件分组扫描：
   - **代码文件**：`.rs` / `.ets` / `.ts` / `Cargo.toml`（A-G 类检查）
   - **仓库配置文件**：`.gitattributes` / `.gitignore` / `.env.local` / `.env`（H 类检查）
   - **文档/openspec**：`openspec/` / `doc/` 下的文件（H3/H5/H6 检查）

3. 对照 `references/review-checklist.md` 逐项快速检查：
   - A: cfg 隔离 — OHOS 代码是否有正确的 cfg gate
   - B: 平台隔离 — 其他平台代码是否受影响
   - C: NAPI/TSFN — callee_handled、FnArgs、camelCase
   - D: 线程模型 — 阻塞模式、Mutex 跨越
   - E: ArkTS 框架 — @Builder、onLoadIntercept
   - F: openharmony-ability 桥接 — 是否唯一桥接
   - G: 代码质量 — unwrap、硬编码、注释
   - H: 仓库级规范 — 本地配置、gitattributes、openspec 归档、手动用例归档

4. 记录 Round 1 findings。

#### 3b. Round 2: 源码深读 + Openspec 对照（使用 Subagent 并行）

**目标**：阅读变更文件的完整源码（不是 diff），对照 openspec 设计文档验证实现完整性。

**执行方式**：使用 `Agent` 工具并行派发 subagent，每个 subagent 负责一个文件的深度审查。这样可以同时阅读多个完整文件 + openspec 文档，提升效率。

1. **先读取 openspec 文档**（如果 PR 涉及 tauri 仓）：
   ```bash
   # 列出 openspec/changes/ 下的所有变更
   ls openspec/changes/
   # 读取与 PR 功能相关的 openspec
   cat openspec/changes/<change-name>/proposal.md
   cat openspec/changes/<change-name>/design.md
   cat openspec/changes/<change-name>/tasks.md
   # 如有 spec 定义
   cat openspec/changes/<change-name>/specs/<capability>/spec.md
   ```

2. **派发 Subagent 并行深读源码**：

   对 diff 中修改的每个关键文件，派发一个 subagent 做深度审查：

   ```
   Agent("深度审查 <file_path>"):
     - Read 完整文件源码
     - 理解上下文：函数调用链、模块边界、cfg gate 组合
     - 检查 diff 未修改但相关的代码（是否需要同步更新）
     - 对照 openspec design.md 检查该文件的实现是否完整
     - 输出该文件的 findings 列表
   ```

   **Subagent prompt 模板**：
   ```
   深度审查文件 `<file_path>`，完整阅读源码后检查：
   1. 该文件的 OHOS 相关代码是否有完整的 cfg gate
   2. 函数/方法的错误处理是否完整（无 unwrap、无 callback 丢失）
   3. 是否有 diff 未修改但需要同步更新的关联代码
   4. 对照 openspec 中 <feature_name> 的设计，该文件的实现是否完整
   5. 线程安全：Mutex/Arc 使用是否合理

   输出格式：每个 finding 列出 file:line, severity, category, description, suggestion。
   如果没有发现新问题，输出 "No new findings"。
   ```

3. **Openspec 合规性审计**（主 agent 执行）：
   - 逐条核对 design.md 中定义的每个功能点是否在代码中实现
   - 逐条核对 spec.md 中定义的每个 requirement 是否被满足
   - 检查 tasks.md 中 `[x]` 标记的任务是否真正完成
   - 未实现的需求 → 🟡 Major [Spec合规] Requirement X not implemented
   - 设计与实现不一致 → 🟡 Major [Spec合规] Design-implementation mismatch

4. **跨仓一致性检查**（多 PR 场景）：
   - wry 层 API 与 tauri 层调用方是否匹配（参数类型、错误处理）
   - openharmony-ability 的 NAPI 接口与 Rust 侧调用是否一致
   - 新增的公共 API 在所有仓中签名是否对齐

5. **仓库级检查（仅 tauri 仓）**：
   - `git diff <base-branch> -- doc/manual_tests.md`：是否新增了与 PR 功能对应的手动用例（H5）
   - `git diff <base-branch> --name-only -- openspec/changes/`：是否归档了对应的设计文档（H6）

6. 记录 Round 2 findings（排除与 Round 1 重复的）。

#### 3c. Round 3+: 专项深挖

**目标**：针对前两轮发现的模式进行定向深挖。

根据前两轮 findings 的模式，选择以下专项检查：

- **错误路径分析**：如果发现 callback 丢失问题，全面扫描所有异步回调路径
- **线程安全分析**：如果发现锁竞争问题，全面检查所有 Mutex/Arc 使用
- **API 兼容性分析**：如果发现 API 签名不一致，全面比对所有跨仓接口
- **cfg 覆盖分析**：如果发现遗漏的 cfg gate，用 grep 扫描所有 OHOS 代码路径

每轮仅保留与 Round 1/2 不重复的新 findings。

#### 3d. Loop-until-dry 退出条件

```
Round N findings 与 Round N-1 findings 去重比较：
  - 如果 Round N 有 0 个新 finding → dry_count++
  - 如果 Round N 有 ≥1 个新 finding → dry_count = 0

退出条件：dry_count >= 2（连续 2 轮无新发现）
最大轮次：5（防止无限循环）
```

每轮结束输出进度：
```
## Review Progress
Round 1 (Diff 扫描): 5 findings (1 🔴, 2 🟡, 2 🔵)
Round 2 (源码深读): 3 new findings (0 🔴, 2 🟡, 1 🔵)
Round 3 (专项深挖): 1 new finding (0 🔴, 0 🟡, 1 🔵)
Round 4 (专项深挖): 0 new findings → dry_count = 1
Round 5 (专项深挖): 0 new findings → dry_count = 2 → EXIT
Total: 9 unique findings
```

#### 3e. 生成最终 Findings

合并所有轮次的 findings，去重后按仓库分组。每个 finding 包含：

```
Finding 结构:
  repo: <仓库名>
  file: <文件路径>
  line: <行号>
  severity: 🔴 Blocker / 🟡 Major / 🔵 Minor / ℹ️ Suggestion
  category: OHOS约束 / Spec合规 / 平台隔离 / 代码质量 / 测试回归 / 仓库规范
  description: <问题描述>
  suggestion: <修复建议>
  round: <发现轮次，标注来源>
```

输出最终审计进度：
```
## Audit Complete (4 rounds)
✅ tauri#25: 5 findings (1 Blocker, 2 Major, 2 Minor)
✅ tao#8: 2 findings (0 Blocker, 1 Major, 1 Minor)
✅ wry#12: 2 findings (0 Blocker, 1 Major, 1 Suggestion)
```

#### 3f. 提交 Review 到 GitHub（Summary + Inline Comments）

**使用 `gh api` 而非 `gh pr review`**，因为 `gh pr review --body` 只能提交总结评论，无法标注到具体代码行。

1. **获取 Head Commit SHA**：
   ```bash
   gh pr view <N> --repo Eulogizethesun/<repo> --json headRefOid --jq '.headRefOid'
   ```

2. **判断 review 类型**：
   - 有 🔴 Blocker → `event: "REQUEST_CHANGES"`
   - 无 Blocker → `event: "COMMENT"`

3. **提交 API 调用**（每个 PR 独立一个调用）：
   ```bash
   gh api repos/Eulogizethesun/<repo>/pulls/<N>/reviews \
     -X POST \
     --input - <<'ENDJSON'
   {
     "commit_id": "<head_commit_sha>",
     "event": "COMMENT",
     "body": "## OHOS Code Review — <repo>#<N>\n\n| 🔴 | 🟡 | 🔵 | ℹ️ |\n|---|---|---|---|\n| 0 | 2 | 1 | 1 |\n\n详细 inline comments 见下方各文件标注。",
     "comments": [
       {
         "path": "crates/tauri/src/ohos_plugin.rs",
         "line": 79,
         "side": "RIGHT",
         "body": "🟡 **[NAPI]** `unwrap()` 在序列化失败时会 panic。Fix: ..."
       }
     ]
   }
   ENDJSON
   ```

4. **行号定位**：`comments[].line` 是文件中的实际行号，不是 diff 行号。定位方法：
   - `gh pr diff <N> --repo ... > review-diff.diff`
   - 用 `grep -n` 找到目标代码在 diff 中的行号
   - 对照源文件确认行号正确

5. **输出提交结果**：
   ```
   ## Review Submitted
   ✅ tauri#25: https://github.com/.../pull/25#pullrequestreview-xxx (4 inline comments)
   ✅ tao#8: https://github.com/.../pull/8#pullrequestreview-xxx (1 inline comment)
   ✅ wry#12: https://github.com/.../pull/12#pullrequestreview-xxx (no inline comments)
   ```

> 报告格式详见 `references/review-report-template.md`
> API 用法详见 `references/github-review-api.md`

### Step 4: 编译部署

调用 `ohos-build` skill 执行完整编译+部署+autotest。

#### 4a. Source 环境

```bash
source D:/workspace/tauri/tauri/.claude/skills/ohos-build/scripts/env.sh
```

#### 4b. 运行构建+测试

```bash
bash D:/workspace/tauri/tauri/.claude/skills/ohos-build/scripts/run-tests.sh "" desktop
```

脚本自动完成：
1. 检测 `openharmony-ability/` 源码变更，自动重建 HAR 包
2. 前端构建（pnpm + vite，VITE_AUTOTEST=true）
3. Rust 交叉编译（aarch64-unknown-linux-ohos，release，--features prod）
4. 拷贝 .so → hvigorw assembleHap（自动签名）
5. 卸载旧版 → 安装 HAP → 启动
6. 等待 30s → 拉取 test-report.md → 分析结果

#### 4c. 解析测试结果

读取 `examples/api/test-report.md`，提取：
- 总测试数 / 通过 / 失败 / 跳过
- 失败的测试名称和错误信息

如果有测试失败，生成 findings：
- 新增失败 → 🟡 Major [测试回归] New test failure: <test_name>
- 回归（之前通过现在失败）→ 🔴 Blocker [测试回归] Regression: <test_name>

#### 4d. 处理构建失败

如果 ohos-build 脚本报错（编译失败、签名失败等）：
- 标记为 🔴 Blocker [代码质量] Build failed: <错误摘要>
- 继续执行 Step 5（findings 仍有效）

输出：
```
## Build & Test Complete
✅ Build: success
✅ Autotest: 42/42 passed, 0 failed
✅ No regressions
```

### Step 5: 清理本地分支

#### 5a. 切回原始分支并删除 review 分支

对每个涉及的仓库：

```bash
cd D:\workspace\tauri\<repo>
git checkout ohdev           # 切回原始分支
git stash pop                # 恢复 uncommitted 改动（如有）
git branch -D review/pr-<N>  # 删除临时 review 分支
```

#### 5b. 清理本地改动

恢复因 review 产生的本地修改（如 build 产物、test-report 等）：

```bash
git checkout -- <被修改的文件>  # 恢复被修改的文件
git clean -f <untracked 文件>   # 清理 untracked 文件
```

#### 5c. 验证清理结果

对每个仓库执行 `git status --short`，确认无 dirty files。

输出：
```
## Cleanup Complete
✅ tauri → ohdev (clean)
✅ tao → ohdev (stash restored)
✅ wry → ohdev (clean)

## Code Review Complete

Reviewed PRs:
  - tauri#25: 1 Blocker, 2 Major (Request Changes)
  - tao#8: 0 Blocker, 1 Major (Comment)
  - wry#12: Clean (Comment)

Build: ✅ Success
Autotest: ✅ 42/42 passed

All reviews submitted to GitHub. Local branches cleaned up.
```

## 参考文档

- [检视清单](references/review-checklist.md) — OHOS 约束 22 项检查清单
- [报告模板](references/review-report-template.md) — 检视报告格式模板
- [GitHub Review API](references/github-review-api.md) — gh api + inline comments 用法

## 错误处理

| 错误场景 | 处理方式 |
|---------|---------|
| gh CLI 未安装 | Step 1 检测，提示用户安装 |
| gh 未认证 | Step 1 检测，提示用户 `gh auth login` |
| PR 不存在或已关闭 | 标记为 "❌ PR Not Found"，跳过该仓库 |
| git stash 失败 | 提示用户手动 commit 或清理 |
| git checkout 冲突 | 提示用户手动解决 |
| 编译失败 | 标记为 🔴 Blocker，Review 已在 Step 3 提交，继续到 Step 5 清理 |
| 测试超时 | 标记为 🟡 Major，继续到 Step 5 清理 |
| gh api review 失败 | 提示用户手动提交，保留报告内容；常见原因：pending review 冲突、PR 已合并 |

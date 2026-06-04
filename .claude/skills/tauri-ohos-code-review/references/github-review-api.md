# GitHub Review API 参考

> 使用 `gh api` 提交带 inline comments 的 review（而非 `gh pr review --body`）。

## 为什么不用 `gh pr review`

| 方式 | 总结评论 | 代码行标注 | 推荐 |
|------|---------|-----------|------|
| `gh pr review --body` | ✅ | ❌ 只能写总结 | 不推荐 |
| `gh api .../reviews` | ✅ | ✅ 每个 finding 标注到代码行 | **推荐** |

`gh pr review --comment --body "..."` 只能在 PR 上留下一条总结评论，无法将 finding 标注到具体的代码行。Review 时需要用 `gh api` 创建带 `comments` 数组的 review。

## 前置条件

### 安装与认证 gh CLI

```bash
# Windows (winget)
winget install --id GitHub.cli

# 认证
gh auth login

# 验证
gh auth status
# ✓ Logged in to github.com as <username>
```

---

## 获取 PR 信息

### 查看 PR 元信息

```bash
gh pr view <N> --repo Eulogizethesun/<repo> --json title,body,headRefName,headRefOid,files
```

关键字段：
- `headRefOid` — head commit SHA，提交 review 时需要（确保 inline comments 定位到正确的行）
- `headRefName` — PR 分支名
- `files` — 变更的文件列表

### 获取 PR diff

```bash
gh pr diff <N> --repo Eulogizethesun/<repo> > review-diff.diff
```

diff 用于定位 inline comment 的行号（见下方"行号定位"章节）。

---

## 提交 Review（Summary + Inline Comments）

### API 调用

```bash
gh api repos/Eulogizethesun/<repo>/pulls/<N>/reviews \
  -X POST \
  --input - <<'ENDJSON'
{
  "commit_id": "<head_commit_sha>",
  "event": "COMMENT",
  "body": "## 总结\n\n| 🔴 Blocker | 🟡 Major | 🔵 Minor | ℹ️ Suggestion |\n|-----------|---------|---------|---------------|\n| 0 | 2 | 1 | 1 |",
  "comments": [
    {
      "path": "crates/tauri/src/ohos.rs",
      "line": 42,
      "side": "RIGHT",
      "body": "🟡 **[线程安全]** 描述..."
    }
  ]
}
ENDJSON
```

### 请求字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `commit_id` | string | PR 的 head commit SHA（从 `gh pr view --json headRefOid` 获取） |
| `event` | string | `"COMMENT"` 或 `"REQUEST_CHANGES"`（有 Blocker 时用后者） |
| `body` | string | 总体报告摘要（summary），包含 findings 计数表 |
| `comments` | array | inline comments 数组，每个元素标注一个 finding |
| `comments[].path` | string | 文件路径（相对于仓库根目录，不含前导 `/`） |
| `comments[].line` | integer | **diff 中的行号**（不是文件绝对行号） |
| `comments[].side` | string | `"RIGHT"`（标注在 PR 新增/修改后的代码侧） |
| `comments[].body` | string | 该行的 comment 内容 |

### event 类型选择

| 条件 | event | 说明 |
|------|-------|------|
| 有 🔴 Blocker | `"REQUEST_CHANGES"` | 要求作者修改后才能合并 |
| 无 Blocker | `"COMMENT"` | 仅评论，不阻止合并 |

### 响应字段

成功提交后返回 JSON，关键提取：
```json
{
  "id": 4415270708,
  "state": "COMMENTED",
  "html_url": "https://github.com/Eulogizethesun/tauri/pull/20#pullrequestreview-4415270708"
}
```

用 `--jq` 提取：
```bash
gh api ... --jq '"Review URL: \(.html_url), ID: \(.id)"'
```

---

## 行号定位（关键步骤）

`comments[].line` 是 **PR diff 中的行号**，不是文件的绝对行号。这是最容易出错的部分。

### 方法：保存 diff 后用 grep 定位

```bash
# 1. 保存 diff
gh pr diff <N> --repo Eulogizethesun/<repo> > review-diff.diff

# 2. 找到目标文件在 diff 中的位置
grep -n "diff --git.*<filename>" review-diff.diff

# 3. 读取该文件的 diff section
# 从步骤 2 的行号开始读取，到下一个 "diff --git" 为止

# 4. 在 diff section 中找到目标代码的行号
grep -n "<目标代码片段>" review-diff.diff
```

### 示例

假设要标注 `ohos_plugin.rs` 中 `serde_json::to_string(...).unwrap()` 这一行：

```bash
# 找到 ohos_plugin.rs 在 diff 中的位置
grep -n "ohos_plugin.rs" review-diff.diff
# 输出: 3013:diff --git a/crates/tauri/src/ohos_plugin.rs ...

# 从 3013 行开始读取 diff section
sed -n '3013,3120p' review-diff.diff
# 找到 unwrap() 在 diff 第 3097 行

# 所以 comments[].line = 3097（diff 中的行号）
```

### 新增文件的行号

如果文件是新增的（diff 显示 `new file mode`），diff 中的所有行都是 `+` 开头，行号就是 diff 文件中的行号。

```
diff --git a/crates/tauri/src/ohos_plugin.rs b/crates/tauri/src/ohos_plugin.rs
new file mode 100644
--- /dev/null
+++ b/crates/tauri/src/ohos_plugin.rs
@@ -0,0 +1,98 @@           ← diff 中的 @@ 行
+use crate::ohos::...;       ← diff 行号 = grep -n 的结果
```

### 已有文件的行号

对于修改的文件，diff 包含 `-`（删除行）和 `+`（新增行）。comment 只能标注在 `+` 行上（`side: "RIGHT"`）。

```
@@ -1,3 +1,4 @@               ← hunk header
+use std::collections::VecDeque;   ← 新增行（可以标注）
 use std::sync::{Mutex, OnceLock}; ← 未变行（不能标注）
```

---

## 完整示例

### 单 PR Review（COMMENT，2 个 inline comments）

```bash
gh api repos/Eulogizethesun/tauri/pulls/20/reviews \
  -X POST \
  --input - <<'ENDJSON'
{
  "commit_id": "ebe29f4467ea39b28d7a962bc72336ea3883e58d",
  "event": "COMMENT",
  "body": "## OHOS Code Review — tauri#20\n\n| 🔴 Blocker | 🟡 Major | 🔵 Minor | ℹ️ Suggestion |\n|-----------|---------|---------|---------------|\n| 0 | 2 | 1 | 1 |\n\n详细 inline comments 见下方各文件标注。\n\n**亮点 👍**: 插件架构设计合理，cfg 隔离干净。",
  "comments": [
    {
      "path": "crates/tauri/src/ohos_plugin.rs",
      "line": 79,
      "side": "RIGHT",
      "body": "🟡 **[NAPI/错误处理]** `serde_json::to_string(...).unwrap()` 在序列化失败时会 panic。\n\nFix: 替换为 `unwrap_or_else` 或返回错误。"
    },
    {
      "path": "crates/tauri/src/ohos.rs",
      "line": 40,
      "side": "RIGHT",
      "body": "🟡 **[线程安全]** `dispatch_run_command` 双锁竞争。\n\n建议将 `RUN_COMMAND_TSFN` 改用 `OnceLock<RunCommandTsfn>`（去掉外层 Mutex）。"
    }
  ]
}
ENDJSON
```

### 单 PR Review（REQUEST_CHANGES，1 个 Blocker）

```bash
gh api repos/Eulogizethesun/tauri/pulls/25/reviews \
  -X POST \
  --input - <<'ENDJSON'
{
  "commit_id": "abc123def456",
  "event": "REQUEST_CHANGES",
  "body": "## OHOS Code Review — tauri#25\n\n| 🔴 Blocker | 🟡 Major | 🔵 Minor | ℹ️ Suggestion |\n|-----------|---------|---------|---------------|\n| 1 | 0 | 0 | 0 |\n\n发现 1 个 Blocker，请修复后重新提交。",
  "comments": [
    {
      "path": "crates/tauri/src/path.rs",
      "line": 42,
      "side": "RIGHT",
      "body": "🔴 **[cfg隔离]** OHOS 特有代码缺少 `cfg(target_env = \"ohos\")` 包裹，会导致 Windows/macOS/Linux 编译错误。\n\nFix:\n```rust\n#[cfg(target_env = \"ohos\")]\npub fn ohos_base_path() -> PathBuf {\n    PathBuf::from(\"/data/storage/el2/base/haps/entry/files\")\n}\n```"
    }
  ]
}
ENDJSON
```

### 多仓批量提交

对每个 PR 独立调用一次 API：

```bash
# tauri#20
gh api repos/Eulogizethesun/tauri/pulls/20/reviews -X POST --input tauri-20-review.json

# plugins-workspace#6
gh api repos/Eulogizethesun/plugins-workspace/pulls/6/reviews -X POST --input pw-6-review.json

# tao#8
gh api repos/Eulogizethesun/tao/pulls/8/reviews -X POST --input tao-8-review.json
```

---

## 检查已有 Review

### 列出 PR 上的 reviews

```bash
gh api repos/Eulogizethesun/<repo>/pulls/<N>/reviews --jq '.[] | "ID: \(.id), State: \(.state), User: \(.user.login)"'
```

### 检查 Pending reviews（可能阻塞新 review）

```bash
gh api graphql -f query='{
  repository(owner: "Eulogizethesun", name: "<repo>") {
    pullRequest(number: <N>) {
      reviews(first: 10, states: [PENDING]) {
        nodes { id author { login } }
      }
    }
  }
}'
```

如果有 pending review，必须先提交或删除：

```bash
# 删除 pending review
gh api graphql -f query='mutation {
  deletePullRequestReview(input: {pullRequestReviewId: "<review_node_id>"}) {
    pullRequestReview { id }
  }
}'
```

---

## 常见错误

### 错误 1: "User can only have one pending review per pull request"

**原因**：之前有一个 PENDING 状态的 review 未提交。

**解决**：
```bash
# 1. 找到 pending review ID
gh api graphql -f query='{
  repository(owner: "Eulogizethesun", name: "<repo>") {
    pullRequest(number: <N>) {
      reviews(first: 5, states: [PENDING]) {
        nodes { id }
      }
    }
  }
}'

# 2. 删除它
gh api graphql -f query='mutation {
  deletePullRequestReview(input: {pullRequestReviewId: "<node_id>"}) {
    pullRequestReview { id }
  }
}'
```

### 错误 2: "Can not delete a non-pending pull request review"

**原因**：已提交的 review（COMMENTED/APPROVED/CHANGES_REQUESTED）无法删除。

**解决**：无需删除，直接提交新 review。GitHub 允许多个 review 并存。

### 错误 3: "Pull Request is not open" (HTTP 422)

**原因**：PR 已合并或关闭，无法提交 review。

**解决**：仅在终端输出报告，不提交到 GitHub。

### 错误 4: "Not Found" (HTTP 404)

**原因**：仓库名错误、PR 编号错误、或没有仓库权限。

**解决**：
```bash
# 验证 PR 存在
gh pr view <N> --repo Eulogizethesun/<repo>
```

### 错误 5: inline comment 没有出现在代码行上

**原因**：`line` 字段值不正确（不是 diff 中的行号，而是文件绝对行号）。

**解决**：重新保存 diff 并用 `grep -n` 定位正确的行号。

---

## 参考链接

- [gh CLI 官方文档](https://cli.github.com/manual/)
- [GitHub REST API: Create a review](https://docs.github.com/en/rest/pulls/reviews#create-a-review-for-a-pull-request)
- [GitHub REST API: Review comments](https://docs.github.com/en/rest/pulls/comments)

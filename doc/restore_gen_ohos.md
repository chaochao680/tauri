# 恢复 gen/ohos 目录

`examples/api/src-tauri/gen/ohos/` 是 OHOS 项目脚手架，在 `.gitignore` 中不被 git 追踪。
如果误删，按以下步骤从模板恢复。

## 前置条件

- DevEco Studio 已安装（含 ohpm）
- 模板目录存在：`crates/tauri-cli/templates/mobile/open-harmony/`

## 步骤 1：从模板复制文件

```bash
TMPL="D:/workspace/tauri/tauri/crates/tauri-cli/templates/mobile/open-harmony"
OHOS="D:/workspace/tauri/tauri/examples/api/src-tauri/gen/ohos"

mkdir -p "$OHOS"

# 复制根级文件
cp -r "$TMPL/AppScope" "$OHOS/"
cp "$TMPL/build-profile.json5" "$OHOS/"
cp "$TMPL/code-linter.json5" "$OHOS/"
cp "$TMPL/hvigorfile.ts" "$OHOS/"
cp "$TMPL/oh-package.json5" "$OHOS/"
cp "$TMPL/oh-package-lock.json5" "$OHOS/"
cp "$TMPL/.gitignore" "$OHOS/"

# 复制 hvigor 配置
mkdir -p "$OHOS/hvigor"
cp "$TMPL/hvigor/hvigor-config.json5" "$OHOS/hvigor/"

# 复制 entry 目录
mkdir -p "$OHOS/entry"
cp "$TMPL/entry/build-profile.json5" "$OHOS/entry/"
cp "$TMPL/entry/hvigorfile.ts" "$OHOS/entry/"
cp "$TMPL/entry/obfuscation-rules.txt" "$OHOS/entry/"
cp "$TMPL/entry/oh-package.json5" "$OHOS/entry/"
cp "$TMPL/entry/oh-package-lock.json5" "$OHOS/entry/"
cp "$TMPL/entry/.gitignore" "$OHOS/entry/"
cp -r "$TMPL/entry/src" "$OHOS/entry/"
```

## 步骤 2：替换模板变量

模板文件中包含 Handlebars 占位符，需要替换为 api 示例的实际值：

| 占位符 | 实际值 | 所在文件 |
|--------|--------|----------|
| `{{app.identifier}}` | `com.tauri.api` | `AppScope/app.json5` |
| `{{app.publisher}}` | `tauri` | `AppScope/app.json5` |
| `{{app.stylized-name}}` | `Tauri API` | `AppScope/resources/base/element/string.json` |
| `{{app.lib-name}}` | `api_lib` | `entry/src/main/ets/entryability/EntryAbility.ets` |
| `{{tauri-binary}}` | `tauri` | `entry/hvigorfile.ts` |
| `{{quote-and-join tauri-binary-args}}` | `"ohos", "dev"` | `entry/hvigorfile.ts` |
| `{{root-dir-rel}}` | `../../../../` | `entry/hvigorfile.ts` |

值的来源：
- `app.identifier` → `tauri.conf.json` 中的 `identifier` 字段
- `app.lib-name` → `Cargo.toml` 中 `[lib] name` 字段
- `app.stylized-name` → `tauri.conf.json` 中的 `productName` 字段

## 步骤 3：安装 ohpm 依赖

```bash
cd "$OHOS"
ohpm install
```

这会安装 `@ohos-rs/ability` 等依赖到 `oh_modules/`。

## 步骤 4：验证

运行 hvigorw 打包确认脚手架完整：

```bash
# 先禁用 tauriPlugin（Windows 上无法连接 tauri CLI）
# 编辑 entry/hvigorfile.ts，将 plugins:[tauriPlugin()] 改为 plugins:[]

# 然后运行构建
bash D:/workspace/tauri/tauri/.claude/skills/ohos-build/scripts/build-ohos.sh
```

## 注意事项

- `gen/ohos` 在 `src-tauri/.gitignore` 中被 `/gen` 规则排除，不会被 git 追踪
- 正常情况下由 `tauri ohos init` 命令生成，但本地开发环境缺少 `@tauri-apps/cli-win32-x64-msvc` 无法执行
- 恢复后不要删除此目录，后续 build 只是往里面拷贝 .so 并运行 hvigorw
- `entry/libs/arm64-v8a/libapi_lib.so` 由 Rust 编译产生，build 脚本会自动拷贝，无需手动处理

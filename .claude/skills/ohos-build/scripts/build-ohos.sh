#!/bin/bash
# Tauri OpenHarmony Build Script
# 编译 Rust + 前端，生成已签名 HAP（hvigorw 使用 build-profile.json5 中的证书自动签名）

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/env.sh"

API_DIR="$PROJECT_ROOT/examples/api"
SRC_TAURI="$API_DIR/src-tauri"
OHOS_PROJECT="$SRC_TAURI/gen/ohos"
SIGNED_HAP="$OHOS_PROJECT/entry/build/default/outputs/default/entry-default-signed.hap"
SO_FILE="$PROJECT_ROOT/target/aarch64-unknown-linux-ohos/release/libapi_lib.so"
HVIGORFILE="$OHOS_PROJECT/entry/hvigorfile.ts"

echo "=== Tauri OpenHarmony Build ==="
echo "DEVECO_HOME=$DEVECO_HOME"
echo "PROJECT_ROOT=$PROJECT_ROOT"
echo "TAURI_OHOS_DEVICE_TYPE=$TAURI_OHOS_DEVICE_TYPE"
echo ""

# ─── Step 1: 安装前端依赖 ───
if [ ! -d "$API_DIR/node_modules" ]; then
    echo ""
    echo ">>> Step 1: Installing frontend dependencies..."
    (cd "$API_DIR" && pnpm install)
fi

# ─── Step 2: 构建 @tauri-apps/api ───
if [ ! -d "$PROJECT_ROOT/packages/api/dist" ]; then
    echo ""
    echo ">>> Step 2: Building @tauri-apps/api..."
    (cd "$PROJECT_ROOT" && pnpm build:api)
fi

# ─── Step 3: 前端构建 ───
echo ""
echo ">>> Step 3: Building frontend (VITE_AUTOTEST=${VITE_AUTOTEST:-false})..."
export VITE_AUTOTEST="${VITE_AUTOTEST:-false}"
(cd "$API_DIR" && pnpm build)

# ─── Step 4: Rust 编译 ───
echo ""
echo ">>> Step 4: Compiling Rust (aarch64-unknown-linux-ohos release, device_type=$TAURI_OHOS_DEVICE_TYPE)..."
rm -f "$SO_FILE"
(cd "$SRC_TAURI" && TAURI_OHOS_DEVICE_TYPE="$TAURI_OHOS_DEVICE_TYPE" cargo build --target aarch64-unknown-linux-ohos --release --features prod)

if [ ! -f "$SO_FILE" ]; then
    echo "ERROR: Rust compilation failed - .so not found"
    exit 1
fi
echo "    Generated: $SO_FILE"

# ─── Step 5: 拷贝 .so 到 ohos 项目 ───
echo ""
echo ">>> Step 5: Copying .so to ohos project..."
mkdir -p "$OHOS_PROJECT/entry/libs/arm64-v8a"
cp "$SO_FILE" "$OHOS_PROJECT/entry/libs/arm64-v8a/libapi_lib.so"

# ─── Step 6: hvigorw 打包（自动禁用/恢复 tauriPlugin）───
echo ""
echo ">>> Step 6: Running hvigorw assembleHap..."

# 禁用 tauriPlugin（独立构建时不需要 TCP 回调 tauri CLI）
if grep -q 'plugins:\[tauriPlugin()\]' "$HVIGORFILE"; then
    sed -i 's/plugins:\[tauriPlugin()\]/plugins:[]/' "$HVIGORFILE"
    RESTORE_PLUGIN=true
else
    RESTORE_PLUGIN=false
fi

rm -f "$SIGNED_HAP"
(cd "$OHOS_PROJECT" && hvigorw --no-daemon -p product=default -p module=entry@default assembleHap --analyze=normal --parallel --incremental) || HVIGOR_EXIT=$?
HVIGOR_EXIT=${HVIGOR_EXIT:-0}

# 恢复 tauriPlugin
if [ "$RESTORE_PLUGIN" = true ]; then
    sed -i 's/plugins:\[\]/plugins:[tauriPlugin()]/' "$HVIGORFILE"
fi

if [ $HVIGOR_EXIT -ne 0 ]; then
    echo "ERROR: hvigorw assembleHap failed"
    exit 1
fi

# ─── 验证产物 ───
if [ ! -f "$SIGNED_HAP" ]; then
    echo "ERROR: Build failed - signed HAP not found at:"
    echo "  $SIGNED_HAP"
    exit 1
fi

echo ""
echo "=== Build Complete ==="
echo "HAP: $SIGNED_HAP"

#!/bin/bash
#
# 客話企盤 (HakSpring-Kipan) 客話辭典建置入口
# 產線流程：
#   1. extract_and_build.js - 讀取客源翠四縣認證與教典資料，調用 variant-parser 展開音字變體
#   2. 生成 dictionary.bin (TKDB v3)、dictionary.fst (前綴自動機)、syllables.fst (音節庫)
#   3. 若有 --deploy 參數，備份並替換 assets/dictionaries/ 供全平台打包使用
#

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="$REPO_ROOT/output/hakfa"
ASSETS_DIR="$REPO_ROOT/assets/dictionaries"

GREEN='\033[0;32m'
NC='\033[0m'

echo ""
echo -e "${GREEN}======================================================${NC}"
echo -e "${GREEN}  HakSpring-Kipan 客話辭典管線建置                    ${NC}"
echo -e "${GREEN}======================================================${NC}"
echo ""

# 1. 確保 fst-builder 存在
if [ ! -f "$REPO_ROOT/engine/target/debug/fst-builder" ]; then
    echo "▶ 編譯 fst-builder..."
    cargo build -p fst-builder --manifest-path "$REPO_ROOT/engine/Cargo.toml"
fi

# 2. 執行客源翠詞庫提取與三神件建置
echo "▶ 執行客話詞庫建置管線..."
node "$SCRIPT_DIR/sources/official/hakspring/extract_and_build.js"

# 3. 部署選項
if [ "$1" == "--deploy" ]; then
    echo ""
    echo "▶ 部署至 assets/dictionaries/..."
    mkdir -p "$ASSETS_DIR/backup_taigi"
    # 若尚未備份原版台語詞庫，先備份一次
    if [ -f "$ASSETS_DIR/dictionary.bin" ] && [ ! -f "$ASSETS_DIR/backup_taigi/dictionary.bin" ]; then
        echo "  備份原版台語辭典至 $ASSETS_DIR/backup_taigi/"
        cp "$ASSETS_DIR"/*.bin "$ASSETS_DIR"/*.fst "$ASSETS_DIR/backup_taigi/" 2>/dev/null || true
    fi
    cp "$OUTPUT_DIR/dictionary.bin" "$ASSETS_DIR/dictionary.bin"
    cp "$OUTPUT_DIR/dictionary.fst" "$ASSETS_DIR/dictionary.fst"
    cp "$OUTPUT_DIR/syllables.fst" "$ASSETS_DIR/syllables.fst"
    echo "  ✓ 成功部署客話辭典至 $ASSETS_DIR/"
fi

echo ""
echo -e "${GREEN}✓ 客話辭典建置完成！${NC}"
echo "  產物位置: $OUTPUT_DIR"
echo ""

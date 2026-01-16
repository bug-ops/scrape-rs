#!/bin/bash
# Bundle size analysis script for scrape-wasm
# Verifies the WASM bundle stays under 500KB

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WASM_DIR="$SCRIPT_DIR/.."

cd "$WASM_DIR"

echo "Building WASM bundle..."
wasm-pack build --release --target web

echo ""
echo "=== Bundle Size Analysis ==="
echo ""

WASM_FILE="pkg/scrape_wasm_bg.wasm"

if [ ! -f "$WASM_FILE" ]; then
    echo "Error: WASM file not found at $WASM_FILE"
    exit 1
fi

# Raw size (cross-platform compatible)
if [[ "$OSTYPE" == "darwin"* ]]; then
    RAW_SIZE=$(stat -f%z "$WASM_FILE")
else
    RAW_SIZE=$(stat -c%s "$WASM_FILE")
fi

echo "Raw WASM size:     $(printf "%'d" $RAW_SIZE) bytes ($(echo "scale=2; $RAW_SIZE/1024" | bc) KB)"

# Gzipped size
GZIP_SIZE=$(gzip -c "$WASM_FILE" | wc -c | tr -d ' ')
echo "Gzipped size:      $(printf "%'d" $GZIP_SIZE) bytes ($(echo "scale=2; $GZIP_SIZE/1024" | bc) KB)"

# Brotli size (if available)
if command -v brotli &> /dev/null; then
    BROTLI_SIZE=$(brotli -c "$WASM_FILE" | wc -c | tr -d ' ')
    echo "Brotli size:       $(printf "%'d" $BROTLI_SIZE) bytes ($(echo "scale=2; $BROTLI_SIZE/1024" | bc) KB)"
fi

echo ""

# Check against limit (512KB = 524288 bytes)
LIMIT=524288
if [ $RAW_SIZE -gt $LIMIT ]; then
    echo "FAIL: Bundle exceeds 512KB limit!"
    echo ""
    echo "Optimization suggestions:"
    echo "  1. Check for unused features in Cargo.toml"
    echo "  2. Run: cargo bloat --release --crates -p scrape-wasm"
    echo "  3. Analyze with: twiggy top $WASM_FILE -n 20"
    echo "  4. Consider feature-gating heavy dependencies"
    echo "  5. Review imports from scrape-core"
    exit 1
else
    REMAINING=$((LIMIT - RAW_SIZE))
    PERCENTAGE=$(echo "scale=1; $RAW_SIZE*100/$LIMIT" | bc)
    echo "PASS: Bundle size OK (${PERCENTAGE}% of 512KB limit)"
    echo "      Remaining budget: $(printf "%'d" $REMAINING) bytes ($(echo "scale=2; $REMAINING/1024" | bc) KB)"
fi

echo ""

# Detailed analysis with twiggy if available
if command -v twiggy &> /dev/null; then
    echo "=== Top 15 Size Contributors (twiggy) ==="
    twiggy top "$WASM_FILE" -n 15 2>/dev/null || echo "Note: twiggy analysis skipped (may require different WASM format)"
    echo ""
fi

# Package summary
echo "=== Package Contents ==="
ls -lh pkg/ | tail -n +2 | awk '{print $9 ": " $5}'
echo ""
echo "Total pkg/ size: $(du -sh pkg/ | cut -f1)"

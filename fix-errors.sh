#!/bin/bash

set -e  # Exit on error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR"
OUTPUT_FILE="/tmp/prompt.out"

# Check required commands
command -v cargo >/dev/null 2>&1 || { echo "cargo is required but not installed" >&2; exit 1; }
command -v xclip >/dev/null 2>&1 || { echo "xclip is required but not installed" >&2; exit 1; }

echo "Please, fix this consolidated LLM Context" > "$OUTPUT_FILE"

prompts=(
    "./prompts/dev/platform/fix-errors.md"
    "./prompts/dev/platform/shared.md"
    "./Cargo.toml"
)

# Validate files exist
for file in "${prompts[@]}"; do
    if [ ! -f "$file" ]; then
        echo "Required file not found: $file" >&2
        exit 1
    fi
done

for file in "${prompts[@]}"; do
    cat "$file" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
done

dirs=(
    # "auth"
    # "automation"
     #"basic"
    # "bot"
     "bootstrap"
    # "package_manager"
    # "channels"
    # "config"
    # "context"
    # "email"
    # "file"
    # "llm"
    "drive_monitor"
    # "llm_legacy"
    # "org"
    # "session"
    "file"
    "kb"
    "shared"
    #"tests"
    # "tools"
    # "web_automation"
    # "whatsapp"
)
for dir in "${dirs[@]}"; do
    if [ -d "$PROJECT_ROOT/src/$dir" ]; then
        find "$PROJECT_ROOT/src/$dir" -name "*.rs" | while read -r file; do
            if [ -f "$file" ]; then
                echo "$file" >> "$OUTPUT_FILE"
                cat "$file" >> "$OUTPUT_FILE"
                echo "" >> "$OUTPUT_FILE"
            fi
        done
    fi
done

# Also append the specific files you mentioned
echo "$PROJECT_ROOT/src/main.rs" >> "$OUTPUT_FILE"
cat "$PROJECT_ROOT/src/main.rs" >> "$OUTPUT_FILE"

echo "$PROJECT_ROOT/src/basic/keywords/get.rs" >> "$OUTPUT_FILE"
cat "$PROJECT_ROOT/src/basic/keywords/get.rs" >> "$OUTPUT_FILE"

echo "" >> "$OUTPUT_FILE"
echo "Compiling..."
cargo build --message-format=short 2>&1 | grep -E 'error' >> "$OUTPUT_FILE"


# Calculate and display token count (approximation: words * 1.3)
WORD_COUNT=$(wc -w < "$OUTPUT_FILE") || { echo "Error counting words" >&2; exit 1; }
TOKEN_COUNT=$(echo "$WORD_COUNT * 1.3 / 1" | bc) || { echo "Error calculating tokens" >&2; exit 1; }
FILE_SIZE=$(wc -c < "$OUTPUT_FILE") || { echo "Error getting file size" >&2; exit 1; }

echo "" >> "$OUTPUT_FILE"
echo "Approximate token count: $TOKEN_COUNT"
echo "Context size: $FILE_SIZE bytes"

if ! cat "$OUTPUT_FILE" | xclip -selection clipboard; then
    echo "Error copying to clipboard" >&2
    exit 1
fi

echo "Content copied to clipboard (xclip)"
rm -f "$OUTPUT_FILE"

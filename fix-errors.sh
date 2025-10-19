#!/bin/bash

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR"
OUTPUT_FILE="/tmp/prompt.out"
rm $OUTPUT_FILE
echo "Please, fix this consolidated LLM Context" > "$OUTPUT_FILE"

prompts=(
    "./prompts/dev/platform/shared.md"
    "./Cargo.toml"
    "./prompts/dev/platform/fix-errors.md"
)

for file in "${prompts[@]}"; do
    cat "$file" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
done

dirs=(
    #"auth"
    #"automation"
    #"basic"
    #"bot"
    "bootstrap"
    #"channels"
    #"config"
    #"context"
    #"email"
    #"file"
    #"llm"
    #"llm_legacy"
    #"org"
    "session"
    "shared"
    #"tests"
    #"tools"
    #"web_automation"
    #"whatsapp"
)
for dir in "${dirs[@]}"; do
    find "$PROJECT_ROOT/src/$dir" -name "*.rs" | while read file; do
        echo $file >> "$OUTPUT_FILE"
        cat "$file" >> "$OUTPUT_FILE"
        echo "" >> "$OUTPUT_FILE"
    done
done

# Also append the specific files you mentioned
echo "$PROJECT_ROOT/src/main.rs" >> "$OUTPUT_FILE"
cat "$PROJECT_ROOT/src/main.rs" >> "$OUTPUT_FILE"


echo "" >> "$OUTPUT_FILE"

cargo build --message-format=short 2>&1 | grep -E 'error' >> "$OUTPUT_FILE"


# Calculate and display token count (approximation: words * 1.3)
WORD_COUNT=$(wc -w < "$OUTPUT_FILE")
TOKEN_COUNT=$(echo "$WORD_COUNT * 1.3 / 1" | bc)
FILE_SIZE=$(wc -c < "$OUTPUT_FILE")

echo "" >> "$OUTPUT_FILE"

echo "Approximate token count: $TOKEN_COUNT"
echo "Context size: $FILE_SIZE bytes"

cat "$OUTPUT_FILE" | xclip -selection clipboard
echo "Content copied to clipboard (xclip)"
rm -f "$OUTPUT_FILE"

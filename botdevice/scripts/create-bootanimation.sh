#!/bin/bash
# Create Android boot animation from gb-bot.svg
# 
# Boot animation format:
# - bootanimation.zip containing:
#   - desc.txt (animation descriptor)
#   - part0/, part1/ (frame folders with PNG images)
#
# To install (requires root):
#   adb root
#   adb push bootanimation.zip /system/media/bootanimation.zip
#   adb reboot

set -e
cd "$(dirname "$0")/.."

SVG_FILE="icons/gb-bot.svg"
OUTPUT_DIR="bootanimation"
BOOT_ZIP="bootanimation.zip"

# Animation settings
WIDTH=1080
HEIGHT=1920
FPS=30
LOOP_COUNT=0  # 0 = infinite

echo "Creating boot animation from $SVG_FILE..."

# Check for required tools
if ! command -v rsvg-convert &> /dev/null; then
    echo "ERROR: rsvg-convert not found!"
    echo "Install with: sudo apt install librsvg2-bin"
    exit 1
fi

if ! command -v convert &> /dev/null; then
    echo "ERROR: ImageMagick convert not found!"
    echo "Install with: sudo apt install imagemagick"
    exit 1
fi

# Clean and create directories
rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR/part0"
mkdir -p "$OUTPUT_DIR/part1"

# Generate base icon (centered, large)
ICON_SIZE=400
rsvg-convert -w $ICON_SIZE -h $ICON_SIZE "$SVG_FILE" -o "$OUTPUT_DIR/icon_base.png"

# Create background (dark theme matching GB branding)
convert -size ${WIDTH}x${HEIGHT} xc:'#1a1a2e' "$OUTPUT_DIR/background.png"

# Part 0: Static logo appear (fade in effect via multiple frames)
echo "Generating part0 (fade in)..."
for i in $(seq 0 9); do
    opacity=$((i * 10 + 10))
    frame=$(printf "%05d" $i)
    
    # Composite icon on background with opacity
    convert "$OUTPUT_DIR/background.png" \
        \( "$OUTPUT_DIR/icon_base.png" -channel A -evaluate multiply $(echo "scale=2; $opacity/100" | bc) +channel \) \
        -gravity center -composite \
        "$OUTPUT_DIR/part0/${frame}.png"
done

# Part 1: Pulsing animation (loop)
echo "Generating part1 (pulse loop)..."
for i in $(seq 0 29); do
    frame=$(printf "%05d" $i)
    
    # Calculate scale for pulse effect (subtle)
    scale=$(echo "scale=4; 1 + 0.05 * s($i * 3.14159 / 15)" | bc -l)
    new_size=$(echo "scale=0; $ICON_SIZE * $scale / 1" | bc)
    
    # Resize icon for pulse effect
    convert "$OUTPUT_DIR/icon_base.png" \
        -resize ${new_size}x${new_size} \
        "$OUTPUT_DIR/icon_pulse.png"
    
    # Composite on background
    convert "$OUTPUT_DIR/background.png" \
        "$OUTPUT_DIR/icon_pulse.png" \
        -gravity center -composite \
        "$OUTPUT_DIR/part1/${frame}.png"
done

# Create desc.txt
cat > "$OUTPUT_DIR/desc.txt" << EOF
$WIDTH $HEIGHT $FPS
p 1 0 part0
p $LOOP_COUNT 0 part1
EOF

# Create bootanimation.zip
echo "Creating $BOOT_ZIP..."
cd "$OUTPUT_DIR"
zip -r0 "../$BOOT_ZIP" desc.txt part0 part1
cd ..

# Cleanup
rm -f "$OUTPUT_DIR/icon_base.png" "$OUTPUT_DIR/icon_pulse.png" "$OUTPUT_DIR/background.png"

echo ""
echo "========================================="
echo "Boot animation created: $BOOT_ZIP"
echo "========================================="
echo ""
echo "To install (requires rooted device):"
echo "  adb root"
echo "  adb remount"
echo "  adb push $BOOT_ZIP /system/media/bootanimation.zip"
echo "  adb reboot"
echo ""
echo "Or for testing:"
echo "  adb shell bootanimation"

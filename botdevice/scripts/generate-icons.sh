#!/bin/bash
# Generate Android icons from SVG
# Requires: inkscape or rsvg-convert

set -e
cd "$(dirname "$0")/.."

SVG_FILE="icons/gb-bot.svg"
ICON_DIR="icons"

# Android icon sizes
declare -A SIZES=(
    ["mdpi"]=48
    ["hdpi"]=72
    ["xhdpi"]=96
    ["xxhdpi"]=144
    ["xxxhdpi"]=192
)

echo "Generating Android icons from $SVG_FILE..."

# Main icon (512x512 for store)
if command -v rsvg-convert &> /dev/null; then
    rsvg-convert -w 512 -h 512 "$SVG_FILE" -o "$ICON_DIR/icon.png"
    echo "Created icon.png (512x512)"
    
    # Generate density-specific icons
    for density in "${!SIZES[@]}"; do
        size=${SIZES[$density]}
        mkdir -p "$ICON_DIR/$density"
        rsvg-convert -w $size -h $size "$SVG_FILE" -o "$ICON_DIR/$density/ic_launcher.png"
        echo "Created $density/ic_launcher.png (${size}x${size})"
    done
elif command -v inkscape &> /dev/null; then
    inkscape -w 512 -h 512 "$SVG_FILE" -o "$ICON_DIR/icon.png"
    echo "Created icon.png (512x512)"
    
    for density in "${!SIZES[@]}"; do
        size=${SIZES[$density]}
        mkdir -p "$ICON_DIR/$density"
        inkscape -w $size -h $size "$SVG_FILE" -o "$ICON_DIR/$density/ic_launcher.png"
        echo "Created $density/ic_launcher.png (${size}x${size})"
    done
else
    echo "ERROR: Neither rsvg-convert nor inkscape found!"
    echo "Install with: sudo apt install librsvg2-bin"
    exit 1
fi

echo "Done! Icons generated in $ICON_DIR/"

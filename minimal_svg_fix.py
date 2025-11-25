#!/usr/bin/env python3
"""
Minimal SVG fix - ONLY fixes text size and contrast
No structural changes, no breaking modifications
"""

import re
from pathlib import Path


def minimal_fix_svg(filepath):
    """Apply minimal fixes to make SVG text readable"""

    with open(filepath, "r", encoding="utf-8") as f:
        content = f.read()

    # Skip font files
    if "fontawesome" in str(filepath).lower() or "favicon" in str(filepath).lower():
        return False

    # 1. Increase font sizes (minimum 16px for readability)
    def fix_font_size(match):
        size = int(match.group(1))
        if size <= 11:
            return f'font-size="16"'
        elif size == 12:
            return f'font-size="18"'
        elif size <= 14:
            return f'font-size="20"'
        else:
            return f'font-size="{size + 4}"'

    content = re.sub(r'font-size="(\d+)"', fix_font_size, content)

    # 2. Fix text colors for contrast
    # Light grays to dark
    content = content.replace('fill="#CBD5E0"', 'fill="#1F2937"')
    content = content.replace('fill="#A0AEC0"', 'fill="#374151"')
    content = content.replace('fill="#718096"', 'fill="#374151"')
    content = content.replace('fill="#E2E8F0"', 'fill="#1F2937"')

    # Light blues to darker blues
    content = content.replace('fill="#90CDF4"', 'fill="#1E40AF"')
    content = content.replace('fill="#63B3ED"', 'fill="#2563EB"')

    # Light purples to darker
    content = content.replace('fill="#E9D8FD"', 'fill="#7C3AED"')
    content = content.replace('fill="#D6BCFA"', 'fill="#9333EA"')
    content = content.replace('fill="#B794F4"', 'fill="#9333EA"')

    # Light oranges to darker
    content = content.replace('fill="#FBD38D"', 'fill="#EA580C"')
    content = content.replace('fill="#F6AD55"', 'fill="#D97706"')

    # Light reds to darker
    content = content.replace('fill="#FEB2B2"', 'fill="#DC2626"')
    content = content.replace('fill="#FC8181"', 'fill="#EF4444"')

    # Light greens stay green (they're usually OK)
    # But make them slightly darker
    content = content.replace('fill="#9AE6B4"', 'fill="#10B981"')
    content = content.replace('fill="#68D391"', 'fill="#059669"')
    content = content.replace('fill="#48BB78"', 'fill="#047857"')

    # Light teals
    content = content.replace('fill="#81E6D9"', 'fill="#0891B2"')
    content = content.replace('fill="#4FD1C5"', 'fill="#0891B2"')
    content = content.replace('fill="#B2F5EA"', 'fill="#0E7490"')

    # Gray arrows
    content = content.replace('fill="#888"', 'fill="#4B5563"')

    # 3. Make SVG responsive (add style attribute if missing)
    if "<svg" in content and "style=" not in content.split(">")[0]:
        content = re.sub(
            r"(<svg[^>]*)(>)",
            r'\1 style="max-width: 100%; height: auto;"\2',
            content,
            count=1,
        )

    # Write the fixed content
    with open(filepath, "w", encoding="utf-8") as f:
        f.write(content)

    return True


def main():
    """Fix all SVG files in docs/src"""

    docs_src = Path("docs/src")
    svg_files = list(docs_src.rglob("*.svg"))

    print(f"Fixing {len(svg_files)} SVG files...")

    fixed = 0
    for svg_file in svg_files:
        try:
            if minimal_fix_svg(svg_file):
                print(f"✓ {svg_file.name}")
                fixed += 1
        except Exception as e:
            print(f"✗ {svg_file.name}: {e}")

    print(f"\nFixed {fixed} files")
    print("Changes made:")
    print("• Font sizes increased (16px minimum)")
    print("• Text colors darkened for contrast")
    print("• SVGs made responsive")


if __name__ == "__main__":
    main()

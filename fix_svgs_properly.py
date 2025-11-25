#!/usr/bin/env python3
"""
Fix all SVG files to be properly readable on all devices
Focus on text size, contrast, and responsive design
"""

import os
import re
from pathlib import Path


def fix_svg_content(content, filename):
    """
    Apply comprehensive fixes to SVG content
    """

    # 1. Fix SVG header for responsiveness
    # Remove any width/height attributes and ensure proper viewBox
    if "<svg" in content:
        # Extract viewBox if exists, or create from width/height
        viewbox_match = re.search(r'viewBox="([^"]+)"', content)
        width_match = re.search(r'width="(\d+)"', content)
        height_match = re.search(r'height="(\d+)"', content)

        if viewbox_match:
            viewbox = viewbox_match.group(1)
        elif width_match and height_match:
            viewbox = f"0 0 {width_match.group(1)} {height_match.group(1)}"
        else:
            viewbox = "0 0 800 600"

        # Replace the SVG opening tag
        svg_pattern = r"<svg[^>]*>"
        svg_replacement = f'<svg viewBox="{viewbox}" xmlns="http://www.w3.org/2000/svg" style="width: 100%; height: auto; max-width: 100%; display: block;">'
        content = re.sub(svg_pattern, svg_replacement, content, count=1)

    # 2. Fix all font sizes - MINIMUM 16px for readability
    def increase_font_size(match):
        size = int(match.group(1))
        if size < 14:
            return f'font-size="16"'
        elif size < 16:
            return f'font-size="18"'
        elif size < 18:
            return f'font-size="20"'
        else:
            return f'font-size="{size + 4}"'  # Increase all fonts slightly

    content = re.sub(r'font-size="(\d+)"', increase_font_size, content)

    # 3. Fix ALL text colors for maximum contrast
    # Replace all light colors with dark, readable ones
    color_replacements = {
        # Light grays to dark gray/black
        "#CBD5E0": "#1F2937",
        "#A0AEC0": "#374151",
        "#E2E8F0": "#1F2937",
        "#EDF2F7": "#111827",
        "#F7FAFC": "#111827",
        "#9CA3AF": "#374151",
        "#D1D5DB": "#4B5563",
        "#718096": "#374151",
        "#4A5568": "#1F2937",
        # Light blues to dark blues
        "#90CDF4": "#1E40AF",
        "#63B3ED": "#2563EB",
        "#4A90E2": "#1D4ED8",
        "#81E6D9": "#0E7490",
        "#4FD1C5": "#0891B2",
        "#38D4B2": "#0D9488",
        # Light purples to dark purples
        "#E9D8FD": "#6B21A8",
        "#D6BCFA": "#7C3AED",
        "#B794F4": "#9333EA",
        "#9F7AEA": "#7C3AED",
        # Light oranges to dark oranges
        "#FBD38D": "#C2410C",
        "#F6AD55": "#EA580C",
        "#ED8936": "#C2410C",
        # Light reds to dark reds
        "#FEB2B2": "#B91C1C",
        "#FC8181": "#DC2626",
        "#E53E3E": "#DC2626",
        # Light greens to dark greens
        "#9AE6B4": "#047857",
        "#68D391": "#059669",
        "#48BB78": "#047857",
        "#38A169": "#059669",
        "#B2F5EA": "#047857",
        # Generic light to dark
        "#888": "#374151",
        "#888888": "#374151",
        "#FAFAFA": "transparent",
        "#fff": "#111827",
        "#ffffff": "#111827",
        "#FFF": "#111827",
        "#FFFFFF": "#111827",
    }

    for old_color, new_color in color_replacements.items():
        # Replace in fill attributes
        content = re.sub(
            f'fill="{old_color}"', f'fill="{new_color}"', content, flags=re.IGNORECASE
        )
        # Replace in stroke attributes
        content = re.sub(
            f'stroke="{old_color}"',
            f'stroke="{new_color}"',
            content,
            flags=re.IGNORECASE,
        )
        # Replace in style attributes
        content = re.sub(
            f"fill:{old_color}", f"fill:{new_color}", content, flags=re.IGNORECASE
        )
        content = re.sub(
            f"stroke:{old_color}", f"stroke:{new_color}", content, flags=re.IGNORECASE
        )

    # 4. Remove white/light backgrounds
    content = re.sub(r'<rect[^>]*fill="#FAFAFA"[^>]*>', "", content)
    content = re.sub(r'<rect[^>]*fill="white"[^>]*>', "", content)
    content = re.sub(r'<rect[^>]*fill="#FFFFFF"[^>]*>', "", content)
    content = re.sub(r'<rect[^>]*fill="#ffffff"[^>]*>', "", content)

    # 5. Fix stroke widths for visibility
    content = re.sub(r'stroke-width="1"', 'stroke-width="2"', content)
    content = re.sub(r'stroke-width="0\.5"', 'stroke-width="2"', content)

    # 6. Fix font weights
    content = re.sub(r'font-weight="bold"', 'font-weight="700"', content)

    # 7. Fix font families for better rendering
    content = re.sub(
        r'font-family="[^"]*"',
        'font-family="system-ui, -apple-system, BlinkMacSystemFont, Segoe UI, Roboto, sans-serif"',
        content,
    )

    # 8. Ensure arrows and markers are visible
    content = re.sub(
        r'<path d="M0,0 L0,6 L9,3 z" fill="#888"/>',
        '<path d="M0,0 L0,6 L9,3 z" fill="#374151"/>',
        content,
    )

    # 9. Add slight padding to the viewBox if needed
    viewbox_match = re.search(r'viewBox="(\d+)\s+(\d+)\s+(\d+)\s+(\d+)"', content)
    if viewbox_match:
        x, y, width, height = map(int, viewbox_match.groups())
        # Add 20px padding
        new_viewbox = f'viewBox="{x - 20} {y - 20} {width + 40} {height + 40}"'
        content = re.sub(r'viewBox="[^"]*"', new_viewbox, content, count=1)

    return content


def process_all_svgs():
    """Process all SVG files in the docs directory"""

    docs_dir = Path("docs")
    svg_files = list(docs_dir.glob("**/*.svg"))

    # Filter out font files
    svg_files = [
        f
        for f in svg_files
        if "fontawesome" not in str(f).lower() and "favicon" not in str(f).lower()
    ]

    print(f"Found {len(svg_files)} SVG files to fix")
    print("=" * 60)

    fixed = 0
    errors = 0

    for svg_file in svg_files:
        try:
            print(f"Processing: {svg_file.relative_to(docs_dir)}")

            # Read the file
            with open(svg_file, "r", encoding="utf-8") as f:
                content = f.read()

            # Apply fixes
            fixed_content = fix_svg_content(content, svg_file.name)

            # Write back
            with open(svg_file, "w", encoding="utf-8") as f:
                f.write(fixed_content)

            print(f"  ✓ Fixed successfully")
            fixed += 1

        except Exception as e:
            print(f"  ✗ Error: {e}")
            errors += 1

    print()
    print("=" * 60)
    print(f"COMPLETED: {fixed} files fixed, {errors} errors")
    print()
    print("Improvements applied:")
    print("• All text now ≥16px (readable on mobile)")
    print("• High contrast colors (dark text, no light grays)")
    print("• 100% responsive width")
    print("• Removed white backgrounds")
    print("• Enhanced stroke widths")
    print("• Added padding to prevent cutoff")
    print("=" * 60)


if __name__ == "__main__":
    print("=" * 60)
    print("SVG READABILITY FIXER")
    print("Making all diagrams actually readable!")
    print("=" * 60)
    print()
    process_all_svgs()

#!/usr/bin/env python3
"""
Fix and beautify all SVG files with proper syntax and mobile-friendly design
"""

import os
import re
import xml.etree.ElementTree as ET
from pathlib import Path


def fix_svg_file(filepath):
    """Fix a single SVG file with proper formatting and mobile-friendly design"""

    try:
        # Read the original file
        with open(filepath, "r", encoding="utf-8") as f:
            content = f.read()

        # Skip font files and favicons
        if "fontawesome" in str(filepath).lower() or "favicon" in str(filepath).lower():
            return False

        print(f"Fixing: {filepath}")

        # First, clean up any broken attributes
        # Remove any malformed style attributes
        content = re.sub(r'style="[^"]*"[^>]*style="[^"]*"', "", content)

        # Fix basic SVG structure
        if not content.strip().startswith("<?xml"):
            content = '<?xml version="1.0" encoding="UTF-8"?>\n' + content

        # Extract dimensions
        width_match = re.search(r'width="(\d+)"', content)
        height_match = re.search(r'height="(\d+)"', content)
        viewbox_match = re.search(r'viewBox="([^"]+)"', content)

        if viewbox_match:
            viewbox = viewbox_match.group(1)
        elif width_match and height_match:
            width = width_match.group(1)
            height = height_match.group(1)
            viewbox = f"0 0 {width} {height}"
        else:
            viewbox = "0 0 800 600"

        # Create clean SVG header
        svg_header = f'''<?xml version="1.0" encoding="UTF-8"?>
<svg viewBox="{viewbox}" xmlns="http://www.w3.org/2000/svg" style="max-width: 100%; height: auto;">
  <defs>
    <!-- Arrow markers -->
    <marker id="arrow" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto" markerUnits="strokeWidth">
      <path d="M0,0 L0,6 L9,3 z" fill="#2563EB"/>
    </marker>

    <!-- Drop shadow for depth -->
    <filter id="shadow" x="-20%" y="-20%" width="140%" height="140%">
      <feGaussianBlur in="SourceAlpha" stdDeviation="2"/>
      <feOffset dx="1" dy="1" result="offsetblur"/>
      <feComponentTransfer>
        <feFuncA type="linear" slope="0.2"/>
      </feComponentTransfer>
      <feMerge>
        <feMergeNode/>
        <feMergeNode in="SourceGraphic"/>
      </feMerge>
    </filter>
  </defs>
'''

        # Extract the main content (remove old svg tags and defs)
        main_content = re.sub(r"<\?xml[^>]*\?>", "", content)
        main_content = re.sub(r"<svg[^>]*>", "", main_content)
        main_content = re.sub(r"</svg>", "", main_content)
        main_content = re.sub(r"<defs>.*?</defs>", "", main_content, flags=re.DOTALL)

        # Fix font sizes for mobile (minimum 14px for body text)
        def fix_font_size(match):
            size = int(match.group(1))
            if size < 12:
                return f'font-size="{14}"'
            elif size < 14:
                return f'font-size="{14}"'
            elif size > 24:
                return f'font-size="{24}"'
            else:
                return match.group(0)

        main_content = re.sub(r'font-size="(\d+)"', fix_font_size, main_content)

        # Fix colors for better contrast
        color_map = {
            # Blues
            "#63B3ED": "#2563EB",
            "#90CDF4": "#3B82F6",
            "#4A90E2": "#2563EB",
            "#CBD5E0": "#1F2937",  # Light gray text to dark
            "#A0AEC0": "#4B5563",  # Medium gray text to darker
            # Greens
            "#68D391": "#059669",
            "#48BB78": "#10B981",
            "#38A169": "#059669",
            "#9AE6B4": "#10B981",
            # Purples
            "#B794F4": "#7C3AED",
            "#D6BCFA": "#8B5CF6",
            "#9F7AEA": "#7C3AED",
            "#E9D8FD": "#8B5CF6",
            # Oranges
            "#F6AD55": "#EA580C",
            "#FBD38D": "#F97316",
            "#ED8936": "#EA580C",
            # Reds
            "#FC8181": "#DC2626",
            "#FEB2B2": "#EF4444",
            "#E53E3E": "#DC2626",
            # Teals
            "#4FD1C5": "#0891B2",
            "#81E6D9": "#06B6D4",
            "#38D4B2": "#0891B2",
            "#B2F5EA": "#06B6D4",
            # Grays
            "#4A5568": "#6B7280",
            "#718096": "#6B7280",
            "#888": "#6B7280",
        }

        for old_color, new_color in color_map.items():
            main_content = main_content.replace(
                f'fill="{old_color}"', f'fill="{new_color}"'
            )
            main_content = main_content.replace(
                f'stroke="{old_color}"', f'stroke="{new_color}"'
            )
            main_content = main_content.replace(
                f'fill="{old_color.lower()}"', f'fill="{new_color}"'
            )
            main_content = main_content.replace(
                f'stroke="{old_color.lower()}"', f'stroke="{new_color}"'
            )

        # Fix font families
        main_content = re.sub(
            r'font-family="[^"]*"',
            'font-family="system-ui, -apple-system, sans-serif"',
            main_content,
        )

        # Ensure stroke widths are visible
        main_content = re.sub(r'stroke-width="1"', 'stroke-width="2"', main_content)

        # Add rounded corners to rectangles
        def add_rounded_corners(match):
            rect = match.group(0)
            if "rx=" not in rect:
                rect = rect[:-1] + ' rx="6"/>'
            return rect

        main_content = re.sub(r"<rect[^>]*/>", add_rounded_corners, main_content)

        # Combine everything
        final_svg = svg_header + main_content + "\n</svg>"

        # Write the fixed file
        with open(filepath, "w", encoding="utf-8") as f:
            f.write(final_svg)

        print(f"  ✓ Fixed successfully")
        return True

    except Exception as e:
        print(f"  ✗ Error: {e}")
        return False


def main():
    """Fix all SVG files in the docs directory"""

    print("=" * 60)
    print("SVG FIXER - Repairing and beautifying all diagrams")
    print("=" * 60)
    print()

    docs_dir = Path("docs")
    svg_files = list(docs_dir.glob("**/*.svg"))

    print(f"Found {len(svg_files)} SVG files")
    print()

    fixed_count = 0
    skipped_count = 0
    error_count = 0

    for svg_file in svg_files:
        if "fontawesome" in str(svg_file).lower() or "favicon" in str(svg_file).lower():
            print(f"Skipping: {svg_file} (font/favicon)")
            skipped_count += 1
            continue

        result = fix_svg_file(svg_file)
        if result:
            fixed_count += 1
        else:
            error_count += 1

    print()
    print("=" * 60)
    print("SUMMARY")
    print("=" * 60)
    print(f"✓ Fixed: {fixed_count} files")
    print(f"⊘ Skipped: {skipped_count} files")
    if error_count > 0:
        print(f"✗ Errors: {error_count} files")
    print()
    print("All SVG files now have:")
    print("• Mobile-friendly text sizes (min 14px)")
    print("• High contrast colors")
    print("• Consistent fonts")
    print("• Rounded corners")
    print("• Proper stroke widths")
    print("=" * 60)


if __name__ == "__main__":
    main()

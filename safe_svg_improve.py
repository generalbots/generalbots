#!/usr/bin/env python3
"""
Safe SVG Improvement Script
Enhances SVG readability for mobile devices without breaking structure
"""

import os
import re
from pathlib import Path


def safe_improve_svg(filepath):
    """
    Safely improve SVG file for better mobile readability
    Only makes minimal, safe changes to preserve structure
    """

    try:
        # Read the original file
        with open(filepath, "r", encoding="utf-8") as f:
            content = f.read()

        # Skip font files and favicons
        if "fontawesome" in str(filepath).lower() or "favicon" in str(filepath).lower():
            return False, "Skipped (font/favicon)"

        original_content = content

        # 1. Make SVG responsive by adding style attribute to svg tag if not present
        if "style=" not in content.split(">")[0]:  # Check only in the opening SVG tag
            content = re.sub(
                r"(<svg[^>]*)(>)",
                r'\1 style="max-width: 100%; height: auto;"\2',
                content,
                count=1,
            )

        # 2. Increase small font sizes for mobile readability (minimum 14px)
        def increase_font_size(match):
            size = int(match.group(1))
            if size < 12:
                return f'font-size="{14}"'
            elif size == 12 or size == 13:
                return f'font-size="{14}"'
            else:
                return match.group(0)

        content = re.sub(r'font-size="(\d+)"', increase_font_size, content)

        # 3. Improve text color contrast for better readability
        # Only change very light grays to darker ones for text
        text_color_improvements = {
            "#CBD5E0": "#374151",  # Light gray to dark gray
            "#A0AEC0": "#4B5563",  # Medium light gray to medium dark
            "#718096": "#374151",  # Another light gray to dark
            "#E9D8FD": "#6B21A8",  # Very light purple to dark purple
            "#FBD38D": "#92400E",  # Light orange to dark orange
            "#90CDF4": "#1E40AF",  # Light blue to dark blue
            "#B2F5EA": "#047857",  # Light teal to dark teal
            "#9AE6B4": "#047857",  # Light green to dark green
        }

        for old_color, new_color in text_color_improvements.items():
            # Only replace in text elements
            content = re.sub(
                f'(<text[^>]*fill="){old_color}(")',
                f"\\1{new_color}\\2",
                content,
                flags=re.IGNORECASE,
            )

        # 4. Ensure stroke widths are visible (minimum 2)
        content = re.sub(r'stroke-width="1"', 'stroke-width="2"', content)
        content = re.sub(r'stroke-width="0\.5"', 'stroke-width="2"', content)

        # 5. Add rounded corners to rectangles if missing (but small radius)
        def add_rounded_corners(match):
            rect = match.group(0)
            if "rx=" not in rect and 'fill="none"' in rect:
                # Add small rounded corners for better aesthetics
                rect = rect[:-2] + ' rx="4"/>'
            return rect

        content = re.sub(r"<rect[^>]*/>", add_rounded_corners, content)

        # 6. Make arrow markers more visible
        content = re.sub(r'fill="#888"', 'fill="#374151"', content)

        # 7. Improve font families for better cross-platform rendering
        content = re.sub(
            r'font-family="Arial, sans-serif"',
            "font-family=\"-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Arial, sans-serif\"",
            content,
        )

        # 8. Fix font weight declarations
        content = re.sub(r'font-weight="bold"', 'font-weight="600"', content)

        # Only write if changes were made
        if content != original_content:
            # Backup original
            backup_path = str(filepath) + ".backup"
            if not os.path.exists(backup_path):
                with open(backup_path, "w", encoding="utf-8") as f:
                    f.write(original_content)

            # Write improved version
            with open(filepath, "w", encoding="utf-8") as f:
                f.write(content)

            return True, "Improved successfully"
        else:
            return False, "No changes needed"

    except Exception as e:
        return False, f"Error: {str(e)}"


def main():
    """Process all SVG files in docs directory"""

    print("=" * 60)
    print("SAFE SVG IMPROVEMENT SCRIPT")
    print("Enhancing readability without breaking structure")
    print("=" * 60)
    print()

    docs_dir = Path("docs")
    svg_files = list(docs_dir.glob("**/*.svg"))

    print(f"Found {len(svg_files)} SVG files")
    print()

    improved = 0
    skipped = 0
    unchanged = 0
    errors = 0

    for svg_file in svg_files:
        print(f"Processing: {svg_file}")

        success, message = safe_improve_svg(svg_file)

        if success:
            print(f"  ✓ {message}")
            improved += 1
        elif "Skipped" in message:
            print(f"  ⊘ {message}")
            skipped += 1
        elif "No changes" in message:
            print(f"  - {message}")
            unchanged += 1
        else:
            print(f"  ✗ {message}")
            errors += 1

    print()
    print("=" * 60)
    print("SUMMARY")
    print("=" * 60)
    print(f"✓ Improved: {improved} files")
    print(f"- Unchanged: {unchanged} files")
    print(f"⊘ Skipped: {skipped} files")
    if errors > 0:
        print(f"✗ Errors: {errors} files")

    print()
    print("Safe improvements applied:")
    print("• Increased minimum font size to 14px")
    print("• Improved text color contrast")
    print("• Made SVGs responsive (100% width)")
    print("• Enhanced stroke visibility")
    print("• Added subtle rounded corners")
    print("• Improved font families for all devices")
    print()
    print("Original files backed up with .backup extension")
    print("=" * 60)


if __name__ == "__main__":
    main()

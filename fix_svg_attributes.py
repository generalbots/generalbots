#!/usr/bin/env python3
"""
Fix malformed SVG attributes in all SVG files
Specifically fixes the 'rx="5"/' issue and other attribute errors
"""

import os
import re
from pathlib import Path


def fix_malformed_attributes(content):
    """Fix various types of malformed attributes in SVG content"""

    # Fix malformed rx attributes (rx="5"/ should be rx="5")
    content = re.sub(r'rx="([^"]+)"\s*/', r'rx="\1"', content)

    # Fix malformed ry attributes
    content = re.sub(r'ry="([^"]+)"\s*/', r'ry="\1"', content)

    # Fix cases where filter appears after malformed rx
    content = re.sub(
        r'rx="([^"]+)"/\s*filter="([^"]+)"', r'rx="\1" filter="\2"', content
    )

    # Fix double closing brackets
    content = re.sub(r"/>>", r"/>", content)

    # Fix attributes that got split incorrectly
    content = re.sub(r'"\s+([a-z-]+)="', r'" \1="', content)

    # Fix rect tags with malformed endings
    content = re.sub(
        r'<rect([^>]+)"\s*/\s+([a-z-]+)="([^"]+)">', r'<rect\1" \2="\3">', content
    )

    # Fix specific pattern: stroke-width="2" rx="5"/ filter="url(#shadow)">
    content = re.sub(
        r'stroke-width="(\d+)"\s+rx="(\d+)"/\s*filter="([^"]+)">',
        r'stroke-width="\1" rx="\2" filter="\3">',
        content,
    )

    # Fix any remaining "/ patterns at the end of attributes
    content = re.sub(r'="([^"]*)"\s*/', r'="\1"', content)

    # Fix rectangles that should be self-closing
    lines = content.split("\n")
    fixed_lines = []

    for line in lines:
        # If it's a rect element that ends with > but has no content, make it self-closing
        if (
            "<rect" in line
            and line.strip().endswith(">")
            and not line.strip().endswith("/>")
        ):
            # Check if this rect has content after it or should be self-closing
            if (
                'fill="none"' in line
                or 'fill="transparent"' in line
                or 'fill="white"' in line
            ):
                line = line.rstrip(">") + "/>"
        fixed_lines.append(line)

    content = "\n".join(fixed_lines)

    return content


def validate_svg_structure(content):
    """Basic validation to ensure SVG structure is correct"""

    # Check for basic SVG structure
    if "<svg" not in content:
        return False, "Missing SVG tag"

    if "</svg>" not in content:
        return False, "Missing closing SVG tag"

    # Count opening and closing tags for basic elements
    rect_open = content.count("<rect")
    rect_close = content.count("</rect>")
    rect_self = content.count("/>")

    # Basic tag balance check (not perfect but catches major issues)
    text_open = content.count("<text")
    text_close = content.count("</text>")

    if text_open != text_close:
        return False, f"Text tag mismatch: {text_open} opening vs {text_close} closing"

    # Check for common malformed patterns
    if "/ " in content and "filter=" in content:
        malformed = re.findall(r'rx="[^"]+"/\s*filter=', content)
        if malformed:
            return False, f"Found malformed attribute pattern"

    return True, "OK"


def fix_svg_file(filepath):
    """Fix a single SVG file"""

    try:
        # Read the file
        with open(filepath, "r", encoding="utf-8") as f:
            content = f.read()

        # Skip font files and favicons
        if "fontawesome" in str(filepath).lower() or "favicon" in str(filepath).lower():
            return "skipped", None

        # Apply fixes
        fixed_content = fix_malformed_attributes(content)

        # Validate the result
        is_valid, message = validate_svg_structure(fixed_content)

        if not is_valid:
            print(f"  ⚠ Validation warning: {message}")

        # Write back only if content changed
        if fixed_content != content:
            with open(filepath, "w", encoding="utf-8") as f:
                f.write(fixed_content)
            return "fixed", None
        else:
            return "unchanged", None

    except Exception as e:
        return "error", str(e)


def main():
    """Fix all SVG files in the docs directory"""

    print("=" * 60)
    print("SVG ATTRIBUTE FIXER")
    print("Fixing malformed attributes in all SVG files")
    print("=" * 60)
    print()

    docs_dir = Path("docs")
    svg_files = list(docs_dir.glob("**/*.svg"))

    print(f"Found {len(svg_files)} SVG files")
    print()

    stats = {"fixed": 0, "unchanged": 0, "skipped": 0, "error": 0}

    for svg_file in svg_files:
        print(f"Processing: {svg_file}")

        status, error = fix_svg_file(svg_file)
        stats[status] += 1

        if status == "fixed":
            print(f"  ✓ Fixed malformed attributes")
        elif status == "unchanged":
            print(f"  - No changes needed")
        elif status == "skipped":
            print(f"  ⊘ Skipped (font/favicon)")
        elif status == "error":
            print(f"  ✗ Error: {error}")

    print()
    print("=" * 60)
    print("SUMMARY")
    print("=" * 60)
    print(f"✓ Fixed: {stats['fixed']} files")
    print(f"- Unchanged: {stats['unchanged']} files")
    print(f"⊘ Skipped: {stats['skipped']} files")
    if stats["error"] > 0:
        print(f"✗ Errors: {stats['error']} files")

    print()
    print("Common fixes applied:")
    print('• Fixed malformed rx="5"/ attributes')
    print("• Corrected filter attribute placement")
    print("• Fixed self-closing tags")
    print("• Cleaned up attribute spacing")
    print("=" * 60)


if __name__ == "__main__":
    main()

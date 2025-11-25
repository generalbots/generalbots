#!/usr/bin/env python3
"""
SVG Validation and Documentation Mapping Script
Checks all SVG files for readability issues and shows where they're used in the documentation
"""

import os
import re
from collections import defaultdict
from pathlib import Path


def analyze_svg(filepath):
    """Analyze an SVG file for potential readability issues"""
    issues = []
    info = {}

    try:
        with open(filepath, "r", encoding="utf-8") as f:
            content = f.read()

        # Check file size
        file_size = os.path.getsize(filepath)
        info["size"] = f"{file_size:,} bytes"

        # Extract viewBox/dimensions
        viewbox_match = re.search(r'viewBox="([^"]+)"', content)
        width_match = re.search(r'width="(\d+)"', content)
        height_match = re.search(r'height="(\d+)"', content)

        if viewbox_match:
            info["viewBox"] = viewbox_match.group(1)
        elif width_match and height_match:
            info["dimensions"] = f"{width_match.group(1)}x{height_match.group(1)}"

        # Check if responsive
        if 'style="max-width: 100%' in content:
            info["responsive"] = "✓"
        else:
            info["responsive"] = "✗"
            issues.append("Not responsive (missing max-width: 100%)")

        # Find all font sizes
        font_sizes = re.findall(r'font-size="(\d+)"', content)
        if font_sizes:
            sizes = [int(s) for s in font_sizes]
            info["font_sizes"] = f"min:{min(sizes)}px, max:{max(sizes)}px"

            # Check for too small fonts
            small_fonts = [s for s in sizes if s < 12]
            if small_fonts:
                issues.append(
                    f"Small fonts found: {small_fonts}px (mobile needs ≥14px)"
                )

        # Check text colors for contrast
        text_colors = re.findall(r'<text[^>]*fill="([^"]+)"', content)
        light_colors = []
        for color in text_colors:
            if any(
                light in color.upper()
                for light in [
                    "#CBD5E0",
                    "#A0AEC0",
                    "#E2E8F0",
                    "#EDF2F7",
                    "#F7FAFC",
                    "#9CA3AF",
                    "#D1D5DB",
                ]
            ):
                light_colors.append(color)

        if light_colors:
            unique_colors = list(set(light_colors))
            issues.append(f"Low contrast text colors: {', '.join(unique_colors[:3])}")

        # Check for background
        if (
            'fill="#FAFAFA"' in content
            or 'fill="white"' in content
            or 'fill="#FFFFFF"' in content
        ):
            if re.search(
                r'<rect[^>]*width="[^"]*"[^>]*height="[^"]*"[^>]*fill="(white|#FAFAFA|#FFFFFF)"',
                content,
            ):
                issues.append("Has white/light background")

        # Count elements
        info["texts"] = content.count("<text")
        info["rects"] = content.count("<rect")
        info["paths"] = content.count("<path")

        return info, issues

    except Exception as e:
        return {"error": str(e)}, [f"Error reading file: {e}"]


def find_svg_references(docs_dir):
    """Find where SVG files are referenced in documentation"""
    references = defaultdict(list)

    # Search in markdown and HTML files
    for ext in ["*.md", "*.html"]:
        for filepath in Path(docs_dir).rglob(ext):
            if "book" in str(filepath):
                continue  # Skip generated book files

            try:
                with open(filepath, "r", encoding="utf-8") as f:
                    content = f.read()

                # Find SVG references
                svg_refs = re.findall(
                    r'(?:src="|href="|!\[.*?\]\(|url\()([^")\s]+\.svg)', content
                )
                for svg_ref in svg_refs:
                    svg_name = os.path.basename(svg_ref)
                    references[svg_name].append(str(filepath.relative_to(docs_dir)))

            except Exception:
                pass

    return references


def main():
    print("=" * 80)
    print("SVG VALIDATION AND DOCUMENTATION MAPPING")
    print("=" * 80)
    print()

    docs_dir = Path("docs")
    src_dir = docs_dir / "src"

    # Find all SVG files
    svg_files = list(src_dir.rglob("*.svg"))

    # Find references
    print("Scanning documentation for SVG references...")
    references = find_svg_references(docs_dir)
    print()

    # Group SVGs by chapter
    chapters = defaultdict(list)
    for svg_file in svg_files:
        parts = svg_file.parts
        if "chapter-" in str(svg_file):
            chapter = next((p for p in parts if "chapter-" in p), "other")
        elif "appendix" in str(svg_file):
            chapter = "appendix-i"
        else:
            chapter = "root-assets"
        chapters[chapter].append(svg_file)

    # Process each chapter
    total_issues = 0
    for chapter in sorted(chapters.keys()):
        print(f"\n{'=' * 60}")
        print(f"CHAPTER: {chapter.upper()}")
        print(f"{'=' * 60}")

        for svg_file in sorted(chapters[chapter]):
            relative_path = svg_file.relative_to(docs_dir)
            svg_name = svg_file.name

            print(f"\n📊 {relative_path}")
            print("-" * 40)

            info, issues = analyze_svg(svg_file)

            # Display info
            if "error" not in info:
                print(f"   Size: {info.get('size', 'Unknown')}")
                if "viewBox" in info:
                    print(f"   ViewBox: {info['viewBox']}")
                elif "dimensions" in info:
                    print(f"   Dimensions: {info['dimensions']}")

                print(f"   Responsive: {info.get('responsive', '?')}")

                if "font_sizes" in info:
                    print(f"   Font sizes: {info['font_sizes']}")

                print(
                    f"   Elements: {info.get('texts', 0)} texts, {info.get('rects', 0)} rects, {info.get('paths', 0)} paths"
                )

            # Display issues
            if issues:
                total_issues += len(issues)
                print(f"\n   ⚠️  ISSUES ({len(issues)}):")
                for issue in issues:
                    print(f"      • {issue}")
            else:
                print("\n   ✅ No issues found")

            # Display references
            if svg_name in references:
                print(f"\n   📄 Used in:")
                for ref in references[svg_name][:5]:  # Show first 5 references
                    print(f"      • {ref}")
                if len(references[svg_name]) > 5:
                    print(f"      ... and {len(references[svg_name]) - 5} more")
            else:
                print(f"\n   ❓ No references found in documentation")

    # Summary
    print("\n" + "=" * 80)
    print("SUMMARY")
    print("=" * 80)
    print(f"Total SVG files analyzed: {len(svg_files)}")
    print(f"Total issues found: {total_issues}")

    if total_issues > 0:
        print("\n🔧 RECOMMENDED FIXES:")
        print("1. Increase all font sizes to minimum 14px for mobile readability")
        print("2. Replace light gray text colors with darker ones for better contrast")
        print("3. Remove white backgrounds or make them transparent")
        print("4. Add responsive styling (max-width: 100%; height: auto)")
        print("5. Consider using system fonts for better cross-platform support")


if __name__ == "__main__":
    main()

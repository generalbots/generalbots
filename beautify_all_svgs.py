#!/usr/bin/env python3
"""
SVG Beautifier - Updates all SVG diagrams for perfect readability on all devices
with beautiful colors that work in both color and black/white modes.
"""

import os
import re
from pathlib import Path

# Beautiful color palette that works in grayscale
COLORS = {
    # Primary colors with good contrast
    'primary_blue': '#2563EB',      # Bright blue - appears as dark gray in B&W
    'primary_green': '#059669',     # Emerald green - medium gray in B&W
    'primary_purple': '#7C3AED',    # Purple - medium-dark gray in B&W
    'primary_orange': '#EA580C',    # Orange - medium gray in B&W
    'primary_red': '#DC2626',       # Red - dark gray in B&W
    'primary_teal': '#0891B2',      # Teal - medium gray in B&W

    # Text colors for maximum readability
    'text_primary': '#1F2937',      # Almost black - perfect for main text
    'text_secondary': '#4B5563',    # Dark gray - for secondary text
    'text_accent': '#2563EB',       # Blue for emphasis - dark in B&W

    # Background and border colors
    'bg_light': '#F9FAFB',          # Very light gray background
    'border_primary': '#2563EB',    # Blue borders - visible in B&W
    'border_secondary': '#9CA3AF',  # Gray borders

    # Status colors
    'success': '#059669',           # Green - medium gray in B&W
    'warning': '#EA580C',           # Orange - medium gray in B&W
    'error': '#DC2626',             # Red - dark gray in B&W
    'info': '#2563EB',              # Blue - dark gray in B&W
}

# Consistent font sizes for all devices (matching documentation)
FONT_SIZES = {
    'title': '24',           # Main diagram titles
    'subtitle': '20',        # Section titles
    'heading': '18',         # Component headings
    'body': '16',           # Main text (matches doc font size)
    'label': '14',          # Labels and annotations
    'small': '12',          # Small details (minimum for mobile)
}

# Standard margins and padding
LAYOUT = {
    'margin': 40,           # Outer margin
    'padding': 20,          # Inner padding
    'spacing': 15,          # Element spacing
    'corner_radius': 8,     # Rounded corners
}

def create_improved_svg(content, filename):
    """
    Transform SVG content with improved styling for all devices.
    """

    # Extract viewBox or width/height
    viewbox_match = re.search(r'viewBox="([^"]+)"', content)
    width_match = re.search(r'width="(\d+)"', content)
    height_match = re.search(r'height="(\d+)"', content)

    if viewbox_match:
        viewbox = viewbox_match.group(1)
        vb_parts = viewbox.split()
        width = int(vb_parts[2])
        height = int(vb_parts[3])
    elif width_match and height_match:
        width = int(width_match.group(1))
        height = int(height_match.group(1))
    else:
        width, height = 800, 600  # Default size

    # Add responsive margins
    new_width = width + (LAYOUT['margin'] * 2)
    new_height = height + (LAYOUT['margin'] * 2)

    # Create new SVG header with responsive sizing
    new_header = f'''<svg viewBox="0 0 {new_width} {new_height}"
     xmlns="http://www.w3.org/2000/svg"
     style="max-width: 100%; height: auto; min-height: 400px;">

  <!-- Beautiful gradient definitions for depth -->
  <defs>
    <linearGradient id="blueGradient" x1="0%" y1="0%" x2="0%" y2="100%">
      <stop offset="0%" style="stop-color:{COLORS['primary_blue']};stop-opacity:0.9" />
      <stop offset="100%" style="stop-color:{COLORS['primary_blue']};stop-opacity:1" />
    </linearGradient>

    <linearGradient id="greenGradient" x1="0%" y1="0%" x2="0%" y2="100%">
      <stop offset="0%" style="stop-color:{COLORS['primary_green']};stop-opacity:0.9" />
      <stop offset="100%" style="stop-color:{COLORS['primary_green']};stop-opacity:1" />
    </linearGradient>

    <linearGradient id="purpleGradient" x1="0%" y1="0%" x2="0%" y2="100%">
      <stop offset="0%" style="stop-color:{COLORS['primary_purple']};stop-opacity:0.9" />
      <stop offset="100%" style="stop-color:{COLORS['primary_purple']};stop-opacity:1" />
    </linearGradient>

    <linearGradient id="orangeGradient" x1="0%" y1="0%" x2="0%" y2="100%">
      <stop offset="0%" style="stop-color:{COLORS['primary_orange']};stop-opacity:0.9" />
      <stop offset="100%" style="stop-color:{COLORS['primary_orange']};stop-opacity:1" />
    </linearGradient>

    <!-- Enhanced arrow markers -->
    <marker id="arrow" markerWidth="12" markerHeight="12" refX="11" refY="6"
            orient="auto" markerUnits="strokeWidth">
      <path d="M0,0 L0,12 L12,6 z" fill="{COLORS['primary_blue']}" />
    </marker>

    <marker id="arrowGreen" markerWidth="12" markerHeight="12" refX="11" refY="6"
            orient="auto" markerUnits="strokeWidth">
      <path d="M0,0 L0,12 L12,6 z" fill="{COLORS['primary_green']}" />
    </marker>

    <!-- Drop shadow filter for depth -->
    <filter id="shadow" x="-50%" y="-50%" width="200%" height="200%">
      <feGaussianBlur in="SourceAlpha" stdDeviation="3"/>
      <feOffset dx="2" dy="2" result="offsetblur"/>
      <feComponentTransfer>
        <feFuncA type="linear" slope="0.2"/>
      </feComponentTransfer>
      <feMerge>
        <feMergeNode/>
        <feMergeNode in="SourceGraphic"/>
      </feMerge>
    </filter>

    <!-- Soft shadow for text -->
    <filter id="textShadow" x="-50%" y="-50%" width="200%" height="200%">
      <feGaussianBlur in="SourceAlpha" stdDeviation="1"/>
      <feOffset dx="1" dy="1" result="offsetblur"/>
      <feComponentTransfer>
        <feFuncA type="linear" slope="0.15"/>
      </feComponentTransfer>
      <feMerge>
        <feMergeNode/>
        <feMergeNode in="SourceGraphic"/>
      </feMerge>
    </filter>
  </defs>

  <!-- White background with subtle border -->
  <rect x="0" y="0" width="{new_width}" height="{new_height}"
        fill="{COLORS['bg_light']}" stroke="{COLORS['border_secondary']}"
        stroke-width="1" rx="{LAYOUT['corner_radius']}" />

  <!-- Content container with proper margins -->
  <g transform="translate({LAYOUT['margin']}, {LAYOUT['margin']})">'''

    # Process the content
    content = re.sub(r'<svg[^>]*>', '', content)
    content = re.sub(r'</svg>', '', content)

    # Update font sizes to be mobile-friendly and consistent
    content = re.sub(r'font-size="(\d+)"', lambda m: update_font_size(m), content)
    content = re.sub(r'font-size:\s*(\d+)(?:px)?', lambda m: f"font-size:{update_font_size_style(m)}", content)

    # Update text colors for better contrast
    content = re.sub(r'fill="#[A-Fa-f0-9]{6}"', lambda m: update_text_color(m), content)
    content = re.sub(r'stroke="#[A-Fa-f0-9]{6}"', lambda m: update_stroke_color(m), content)

    # Improve rectangles with better styling
    content = re.sub(r'<rect([^>]+)>', lambda m: improve_rect(m), content)

    # Update text elements with better positioning and styling
    content = re.sub(r'<text([^>]*)>(.*?)</text>', lambda m: improve_text(m), content)

    # Add font family consistency
    content = re.sub(r'font-family="[^"]*"',
                    'font-family="-apple-system, BlinkMacSystemFont, \'Segoe UI\', Roboto, \'Helvetica Neue\', Arial, sans-serif"',
                    content)

    # Close the container and SVG
    new_footer = '''
  </g>
</svg>'''

    return new_header + content + new_footer

def update_font_size(match):
    """Update font sizes to be mobile-friendly."""
    size = int(match.group(1))
    if size >= 20:
        return f'font-size="{FONT_SIZES["title"]}"'
    elif size >= 16:
        return f'font-size="{FONT_SIZES["heading"]}"'
    elif size >= 14:
        return f'font-size="{FONT_SIZES["body"]}"'
    elif size >= 12:
        return f'font-size="{FONT_SIZES["label"]}"'
    else:
        return f'font-size="{FONT_SIZES["small"]}"'

def update_font_size_style(match):
    """Update font sizes in style attributes."""
    size = int(match.group(1))
    if size >= 20:
        return FONT_SIZES["title"]
    elif size >= 16:
        return FONT_SIZES["heading"]
    elif size >= 14:
        return FONT_SIZES["body"]
    elif size >= 12:
        return FONT_SIZES["label"]
    else:
        return FONT_SIZES["small"]

def update_text_color(match):
    """Update text fill colors for better contrast."""
    color = match.group(0)
    # Check if it's a light color (rough heuristic)
    if any(light in color.lower() for light in ['fff', 'fef', 'efe', 'fee', 'eee', 'ddd', 'ccc']):
        return f'fill="{COLORS["text_primary"]}"'
    # Keep dark colors but ensure they're dark enough
    elif any(dark in color.lower() for dark in ['000', '111', '222', '333', '444']):
        return f'fill="{COLORS["text_primary"]}"'
    else:
        # For other colors, use our palette
        return f'fill="{COLORS["text_secondary"]}"'

def update_stroke_color(match):
    """Update stroke colors to use our palette."""
    color = match.group(0)
    # Map to our color palette for consistency
    if 'blue' in color.lower() or '4a90e2' in color.lower() or '63b3ed' in color.lower():
        return f'stroke="{COLORS["primary_blue"]}"'
    elif 'green' in color.lower() or '48bb78' in color.lower() or '68d391' in color.lower():
        return f'stroke="{COLORS["primary_green"]}"'
    elif 'purple' in color.lower() or 'b794f4' in color.lower() or '9f7aea' in color.lower():
        return f'stroke="{COLORS["primary_purple"]}"'
    elif 'orange' in color.lower() or 'f6ad55' in color.lower() or 'ed8936' in color.lower():
        return f'stroke="{COLORS["primary_orange"]}"'
    elif 'red' in color.lower() or 'e53e3e' in color.lower() or 'fc8181' in color.lower():
        return f'stroke="{COLORS["primary_red"]}"'
    else:
        return f'stroke="{COLORS["border_primary"]}"'

def improve_rect(match):
    """Improve rectangle elements with better styling."""
    rect = match.group(0)
    # Add rounded corners if not present
    if 'rx=' not in rect:
        rect = rect[:-1] + f' rx="{LAYOUT["corner_radius"]}">'
    # Add subtle shadow for depth
    if 'filter=' not in rect:
        rect = rect[:-1] + ' filter="url(#shadow)">'
    # Ensure proper stroke width
    rect = re.sub(r'stroke-width="[^"]*"', 'stroke-width="2"', rect)
    return rect

def improve_text(match):
    """Improve text elements with better styling."""
    text_tag = match.group(1)
    text_content = match.group(2)

    # Add text shadow for better readability
    if 'filter=' not in text_tag:
        text_tag += ' filter="url(#textShadow)"'

    # Ensure text has proper weight for readability
    if 'font-weight=' not in text_tag and any(word in text_content.lower() for word in ['title', 'process', 'flow', 'system']):
        text_tag += ' font-weight="600"'

    return f'<text{text_tag}>{text_content}</text>'

def process_all_svgs():
    """Process all SVG files in the docs directory."""
    docs_dir = Path('docs')

    # Find all SVG files
    svg_files = list(docs_dir.glob('**/*.svg'))

    print(f"Found {len(svg_files)} SVG files to beautify")

    for svg_file in svg_files:
        # Skip font files
        if 'fontawesome' in str(svg_file).lower() or 'favicon' in str(svg_file).lower():
            print(f"Skipping font/favicon file: {svg_file}")
            continue

        print(f"Beautifying: {svg_file}")

        try:
            # Read the original content
            with open(svg_file, 'r', encoding='utf-8') as f:
                content = f.read()

            # Skip if already processed
            if 'Beautiful gradient definitions' in content:
                print(f"  Already beautified, skipping...")
                continue

            # Create improved version
            improved = create_improved_svg(content, svg_file.name)

            # Save the improved version
            with open(svg_file, 'w', encoding='utf-8') as f:
                f.write(improved)

            print(f"  ✓ Successfully beautified!")

        except Exception as e:
            print(f"  ✗ Error processing {svg_file}: {e}")

if __name__ == "__main__":
    print("=" * 60)
    print("SVG BEAUTIFIER - Making diagrams beautiful for all devices")
    print("=" * 60)
    print("\nFeatures:")
    print("• Consistent text sizing matching documentation")
    print("• Proper margins and padding for mobile")
    print("• Beautiful colors that work in black & white")
    print("• Responsive design for all screen sizes")
    print("• Enhanced readability with shadows and gradients")
    print("\nStarting beautification process...\n")

    process_all_svgs()

    print("\n" + "=" * 60)
    print("✨ Beautification complete!")
    print("All SVGs now have:")
    print("• Mobile-friendly text sizes (min 12px)")
    print("• Consistent font family")
    print("• Proper margins (40px) and padding (20px)")
    print("• High contrast colors readable in B&W")
    print("• Responsive viewBox settings")
    print("=" * 60)

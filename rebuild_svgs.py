#!/usr/bin/env python3
"""
SVG Rebuilder - Converts all SVG files to match the style guide standards
Following the guidelines from botserver/prompts/dev/svg-diagram-style-guide.md
"""

import os
import re
from pathlib import Path
from typing import Dict, List, Tuple

# Style guide constants
COLORS = {
    "blue": "#4A90E2",  # Input/User elements, External/API
    "orange": "#F5A623",  # Processing/Scripts, Storage/Data
    "purple": "#BD10E0",  # AI/ML/Decision
    "green": "#7ED321",  # Execution/Action
    "cyan": "#50E3C2",  # Output/Response
    "gray": "#666",  # Arrows/text
    "dark": "#333",  # Labels
}

SVG_TEMPLATE = """<svg width="800" height="{height}" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <marker id="arrow" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto" markerUnits="strokeWidth">
      <path d="M0,0 L0,6 L9,3 z" fill="#666"/>
    </marker>
  </defs>

  <!-- Title -->
  <text x="400" y="25" text-anchor="middle" font-family="Arial, sans-serif" font-size="16" font-weight="600" fill="#333">{title}</text>

  {content}

  <!-- Description -->
  <text x="400" y="{desc_y}" text-anchor="middle" font-family="Arial, sans-serif" font-size="12" fill="#666">
    {description}
  </text>
</svg>"""


def create_box(x: int, y: int, width: int, height: int, color: str, label: str) -> str:
    """Create a standard box component"""
    center_x = x + width // 2
    center_y = y + height // 2 + 5
    return f'''<rect x="{x}" y="{y}" width="{width}" height="{height}" fill="none" stroke="{color}" stroke-width="2" rx="5"/>
  <text x="{center_x}" y="{center_y}" text-anchor="middle" font-family="Arial, sans-serif" font-size="14" fill="#333">{label}</text>'''


def create_arrow(
    x1: int, y1: int, x2: int, y2: int, dashed: bool = False, opacity: float = 1.0
) -> str:
    """Create an arrow between two points"""
    dash_attr = ' stroke-dasharray="3,3"' if dashed else ""
    opacity_attr = f' opacity="{opacity}"' if opacity < 1.0 else ""
    return f'<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" marker-end="url(#arrow)"{dash_attr}{opacity_attr}/>'


def create_curved_arrow(
    points: List[Tuple[int, int]], dashed: bool = False, opacity: float = 1.0
) -> str:
    """Create a curved arrow path"""
    dash_attr = ' stroke-dasharray="3,3"' if dashed else ""
    opacity_attr = f' opacity="{opacity}"' if opacity < 1.0 else ""

    if len(points) < 3:
        return ""

    path = f"M{points[0][0]},{points[0][1]}"
    if len(points) == 3:
        path += f" Q{points[1][0]},{points[1][1]} {points[2][0]},{points[2][1]}"
    else:
        for i in range(1, len(points)):
            path += f" L{points[i][0]},{points[i][1]}"

    return f'<path d="{path}" marker-end="url(#arrow)"{dash_attr}{opacity_attr}/>'


def rebuild_conversation_flow() -> str:
    """Rebuild conversation flow diagram"""
    boxes = []
    arrows = []

    # Main flow boxes
    boxes.append(create_box(20, 60, 100, 40, COLORS["blue"], "User Input"))
    boxes.append(create_box(160, 60, 100, 40, COLORS["orange"], "ASIC Script"))
    boxes.append(create_box(300, 60, 100, 40, COLORS["purple"], "LM Decision"))
    boxes.append(create_box(440, 60, 100, 40, COLORS["green"], "Bot Executor"))
    boxes.append(create_box(580, 60, 100, 40, COLORS["cyan"], "Bot Response"))

    # Parallel processes
    boxes.append(create_box(360, 160, 120, 40, COLORS["blue"], "Search Knowledge"))
    boxes.append(create_box(500, 160, 100, 40, COLORS["orange"], "Call API"))

    # Main flow arrows
    arrows.append(create_arrow(120, 80, 160, 80))
    arrows.append(create_arrow(260, 80, 300, 80))
    arrows.append(create_arrow(400, 80, 440, 80))
    arrows.append(create_arrow(540, 80, 580, 80))

    # Branch arrows
    arrows.append(create_arrow(490, 100, 420, 160, dashed=True, opacity=0.6))
    arrows.append(create_arrow(490, 100, 550, 160, dashed=True, opacity=0.6))

    # Feedback loops
    arrows.append(
        create_curved_arrow(
            [(420, 200), (420, 240), (630, 240), (630, 100)], dashed=True, opacity=0.4
        )
    )
    arrows.append(
        create_curved_arrow(
            [(550, 200), (550, 230), (620, 230), (620, 100)], dashed=True, opacity=0.4
        )
    )

    content = (
        "\n  ".join(boxes)
        + '\n\n  <g stroke="#666" stroke-width="2" fill="none">\n    '
        + "\n    ".join(arrows)
        + "\n  </g>"
    )

    return SVG_TEMPLATE.format(
        height=320,
        title="The Flow",
        content=content,
        desc_y=300,
        description="The AI handles everything else - understanding intent, collecting information, executing tools, answering from documents. Zero configuration.",
    )


def rebuild_architecture() -> str:
    """Rebuild architecture diagram"""
    boxes = []
    arrows = []

    # Top layer
    boxes.append(create_box(20, 60, 100, 40, COLORS["blue"], "Web Server"))
    boxes.append(create_box(160, 60, 120, 40, COLORS["orange"], "BASIC Interpreter"))
    boxes.append(create_box(320, 60, 100, 40, COLORS["purple"], "LLM Integration"))
    boxes.append(create_box(460, 60, 120, 40, COLORS["green"], "Package Manager"))
    boxes.append(create_box(620, 60, 100, 40, COLORS["cyan"], "Console UI"))

    # Middle layer
    boxes.append(
        create_box(
            250, 160, 300, 40, COLORS["blue"], "Session Manager (Tokio Async Runtime)"
        )
    )

    # Data layer
    boxes.append(create_box(20, 260, 100, 40, COLORS["orange"], "PostgreSQL"))
    boxes.append(create_box(160, 260, 100, 40, COLORS["purple"], "Valkey Cache"))
    boxes.append(create_box(300, 260, 100, 40, COLORS["green"], "Qdrant Vectors"))
    boxes.append(create_box(440, 260, 100, 40, COLORS["cyan"], "Object Storage"))
    boxes.append(create_box(580, 260, 100, 40, COLORS["blue"], "Channels"))
    boxes.append(create_box(700, 260, 80, 40, COLORS["orange"], "External API"))

    # Connection arrows (simplified)
    for x in [70, 220, 370, 520, 670]:
        arrows.append(
            create_curved_arrow(
                [(x, 100), (x, 130), (400, 130), (400, 160)], opacity=0.6
            )
        )

    for x in [70, 210, 350, 490, 630]:
        arrows.append(create_arrow(400, 200, x, 260, opacity=0.6))

    # External API connection
    arrows.append(
        create_curved_arrow(
            [(740, 260), (740, 220), (550, 180)], dashed=True, opacity=0.4
        )
    )

    content = (
        "\n  ".join(boxes)
        + '\n\n  <g stroke="#666" stroke-width="2" fill="none">\n    '
        + "\n    ".join(arrows)
        + "\n  </g>"
    )

    # Add storage detail box
    detail_box = """
  <g transform="translate(20, 330)">
    <rect width="760" height="50" fill="none" stroke="#666" stroke-width="1" rx="5" opacity="0.3"/>
    <text x="10" y="25" font-family="Arial, sans-serif" font-size="12" fill="#666">Storage Contents:</text>
    <text x="130" y="25" font-family="Arial, sans-serif" font-size="12" fill="#666">.gbkb (Documents)</text>
    <text x="280" y="25" font-family="Arial, sans-serif" font-size="12" fill="#666">.gbdialog (Scripts)</text>
    <text x="430" y="25" font-family="Arial, sans-serif" font-size="12" fill="#666">.gbot (Configs)</text>
    <text x="560" y="25" font-family="Arial, sans-serif" font-size="12" fill="#666">Templates</text>
    <text x="660" y="25" font-family="Arial, sans-serif" font-size="12" fill="#666">User Assets</text>
  </g>"""

    content += detail_box

    return SVG_TEMPLATE.format(
        height=400,
        title="General Bots Architecture",
        content=content,
        desc_y=45,
        description="Single binary with everything included - no external dependencies",
    )


def rebuild_package_system_flow() -> str:
    """Rebuild package system flow diagram"""
    boxes = []
    arrows = []

    # Main flow
    boxes.append(create_box(20, 60, 100, 40, COLORS["blue"], "User Request"))
    boxes.append(create_box(160, 60, 100, 40, COLORS["orange"], "start.bas"))
    boxes.append(create_box(300, 60, 100, 40, COLORS["purple"], "LLM Engine"))
    boxes.append(create_box(440, 60, 100, 40, COLORS["cyan"], "Bot Response"))

    # Supporting components
    boxes.append(create_box(240, 160, 120, 40, COLORS["blue"], "Vector Search"))
    boxes.append(create_box(240, 240, 120, 40, COLORS["orange"], ".gbkb docs"))

    # Main flow arrows
    arrows.append(create_arrow(120, 80, 160, 80))
    arrows.append(create_arrow(260, 80, 300, 80))
    arrows.append(create_arrow(400, 80, 440, 80))

    # Bidirectional between start.bas and LLM
    arrows.append(
        create_curved_arrow(
            [(210, 100), (210, 120), (300, 120), (350, 120), (350, 100)],
            dashed=True,
            opacity=0.6,
        )
    )
    arrows.append(
        create_curved_arrow(
            [(350, 60), (350, 40), (260, 40), (210, 40), (210, 60)],
            dashed=True,
            opacity=0.6,
        )
    )

    # LLM to Vector Search
    arrows.append(create_arrow(350, 100, 300, 160, opacity=0.6))

    # Vector Search to .gbkb docs
    arrows.append(create_arrow(300, 200, 300, 240, opacity=0.6))

    # Feedback from Vector Search to LLM
    arrows.append(
        create_curved_arrow(
            [(240, 180), (200, 140), (300, 100)], dashed=True, opacity=0.4
        )
    )

    content = (
        "\n  ".join(boxes)
        + '\n\n  <g stroke="#666" stroke-width="2" fill="none">\n    '
        + "\n    ".join(arrows)
        + "\n  </g>"
    )

    # Add BASIC commands and package structure boxes
    detail_boxes = """
  <g transform="translate(580, 60)">
    <rect width="200" height="120" fill="none" stroke="#7ED321" stroke-width="2" rx="5"/>
    <text x="100" y="25" text-anchor="middle" font-family="Arial, sans-serif" font-size="14" fill="#333">BASIC Commands</text>
    <text x="10" y="50" font-family="monospace" font-size="12" fill="#666">USE KB "docs"</text>
    <text x="10" y="70" font-family="monospace" font-size="12" fill="#666">answer = HEAR</text>
    <text x="10" y="90" font-family="monospace" font-size="12" fill="#666">result = LLM()</text>
    <text x="10" y="110" font-family="monospace" font-size="12" fill="#666">TALK result</text>
  </g>

  <g transform="translate(580, 210)">
    <rect width="200" height="140" fill="none" stroke="#4A90E2" stroke-width="2" rx="5"/>
    <text x="100" y="25" text-anchor="middle" font-family="Arial, sans-serif" font-size="14" fill="#333">Package Structure</text>
    <text x="10" y="50" font-family="monospace" font-size="12" fill="#666">my-bot.gbai/</text>
    <text x="20" y="70" font-family="monospace" font-size="12" fill="#666">├─ .gbdialog/</text>
    <text x="20" y="90" font-family="monospace" font-size="12" fill="#666">├─ .gbkb/</text>
    <text x="20" y="110" font-family="monospace" font-size="12" fill="#666">└─ .gbot/</text>
  </g>"""

    content += detail_boxes

    # Add connection lines to detail boxes
    content += """
  <g stroke="#666" stroke-width="2" fill="none">
    <path d="M210,60 Q395,20 580,80" stroke-dasharray="2,2" opacity="0.3"/>
    <path d="M300,280 Q440,330 580,310" stroke-dasharray="2,2" opacity="0.3"/>
  </g>"""

    # Add labels
    labels = """
  <text x="180" y="35" font-family="Arial, sans-serif" font-size="11" fill="#666">Commands</text>
  <text x="180" y="125" font-family="Arial, sans-serif" font-size="11" fill="#666">Results</text>
  <text x="325" y="135" font-family="Arial, sans-serif" font-size="11" fill="#666">Query</text>
  <text x="250" y="135" font-family="Arial, sans-serif" font-size="11" fill="#666">Context</text>"""

    content += labels

    return SVG_TEMPLATE.format(
        height=400,
        title="Package System Flow",
        content=content,
        desc_y=380,
        description="BASIC scripts orchestrate LLM decisions, vector search, and responses with zero configuration",
    )


def main():
    """Main function to rebuild all SVGs"""
    svgs_to_rebuild = {
        "docs/src/assets/conversation-flow.svg": rebuild_conversation_flow(),
        "docs/src/assets/architecture.svg": rebuild_architecture(),
        "docs/src/assets/package-system-flow.svg": rebuild_package_system_flow(),
    }

    for filepath, content in svgs_to_rebuild.items():
        full_path = Path(filepath)
        if full_path.parent.exists():
            with open(full_path, "w") as f:
                f.write(content)
            print(f"Rebuilt: {filepath}")
        else:
            print(f"Skipping (directory not found): {filepath}")

    print(f"\nRebuilt {len(svgs_to_rebuild)} SVG files according to style guide")
    print(
        "Note: This is a demonstration script. Extend it to rebuild all 28 SVG files."
    )


if __name__ == "__main__":
    main()

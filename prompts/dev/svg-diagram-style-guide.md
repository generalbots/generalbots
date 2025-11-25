# SVG Diagram Style Guide & Prompt Template

## Quick Prompt Template

When creating technical diagrams or flow charts, use this prompt:

```
Create a clean SVG diagram with these specifications:
- Transparent background (no fill)
- Large dimensions: width="1040-1400" height="[appropriate height]" (1.3x standard size)
  - For vertical flows: width="1040" height="[600-1200]"
  - For horizontal flows: width="1400" height="900" (recommended)
- Simple colored borders for components (no fill, stroke-width="2.6")
- Standard Arial font (font-family="Arial, sans-serif")
- Dual-theme support with CSS classes
- Base color palette:
  - Blue: #4A90E2
  - Orange: #F5A623  
  - Purple: #BD10E0
  - Green: #7ED321
  - Cyan: #50E3C2
  - Gray for arrows/text: #666
- Rounded rectangles (rx="6.5") for boxes
- Large arrow markers (13x13) with triangular heads
- Dashed lines for optional/feedback flows (stroke-dasharray="3.9,3.9")
- Subtle neon glow effects for dark themes
- Text should be centered in boxes (text-anchor="middle")
- Font sizes: 29-32px for titles, 22-24px for component labels, 18-21px for descriptions
- DUAL DIAGRAM COMPOSITION when possible (main flow + progress/legend)
- Title positioned well above content (y="45" minimum)
- Text wrapping for long labels (review box width constraints)
```

## Beautiful Composition Rules - THE STANDARD!

### Dual-Diagram Approach (RECOMMENDED)
When creating process flows or pipelines, compose TWO complementary visualizations:

1. **Main Flow Diagram** (Top Section)
   - Primary process visualization with components and connections
   - Positioned in upper 60-70% of canvas
   - Clear phase groupings with section labels
   - Components sized appropriately for their text content

2. **Progress Indicator/Legend** (Bottom Section)
   - Visual timeline or progress bar showing stages
   - Positioned in lower 30-40% of canvas
   - Stage markers with labels below
   - Connected with subtle lines or gradient background
   - Creates visual rhythm and helps navigation

### Text Handling Rules
- **Long Text**: MUST be reviewed against box width
  - If text exceeds box width, either:
    - Increase box width to accommodate
    - Use text wrapping with multiple <tspan> elements
    - Abbreviate with full text in tooltip/description
- **Component Labels**: Keep concise, max 2-3 words when possible
- **Descriptions**: Use separate text elements below main diagram

### Spacing & Visual Hierarchy
- **Title Separation**: Position title FAR from content (y="45" minimum)
- **Phase Grouping**: Clear visual separation between logical phases
- **Vertical Rhythm**: Consistent spacing creates professional look
- **Legend Positioning**: Always at bottom with ample spacing from main diagram

## Enhanced SVG Structure Template with Dual Composition

```svg
<svg width="1400" height="900" xmlns="http://www.w3.org/2000/svg">
  <style>
    /* Light theme defaults */
    .neon-blue { stroke: #4A90E2; stroke-width: 2.6; }
    .neon-orange { stroke: #F5A623; stroke-width: 2.6; }
    .neon-purple { stroke: #BD10E0; stroke-width: 2.6; }
    .neon-green { stroke: #7ED321; stroke-width: 2.6; }
    .neon-cyan { stroke: #50E3C2; stroke-width: 2.6; }
    .main-text { fill: #1a1a1a; }
    .secondary-text { fill: #666; }
    .arrow-color { stroke: #666; fill: #666; }

    /* Dark theme with subtle neon effects */
    @media (prefers-color-scheme: dark) {
      .neon-blue {
        stroke: #00D4FF;
        stroke-width: 2.8;
        filter: drop-shadow(0 0 4px #00D4FF) drop-shadow(0 0 8px #00A0FF);
      }
      .neon-orange {
        stroke: #FF9500;
        stroke-width: 2.8;
        filter: drop-shadow(0 0 4px #FF9500) drop-shadow(0 0 8px #FF7700);
      }
      .neon-purple {
        stroke: #E040FB;
        stroke-width: 2.8;
        filter: drop-shadow(0 0 4px #E040FB) drop-shadow(0 0 8px #D500F9);
      }
      .neon-green {
        stroke: #00FF88;
        stroke-width: 2.8;
        filter: drop-shadow(0 0 4px #00FF88) drop-shadow(0 0 8px #00E676);
      }
      .neon-cyan {
        stroke: #00E5EA;
        stroke-width: 2.8;
        filter: drop-shadow(0 0 4px #00E5EA) drop-shadow(0 0 8px #00BCD4);
      }
      .main-text { fill: #FFFFFF; }
      .secondary-text { fill: #B0B0B0; }
      .arrow-color { stroke: #B0B0B0; fill: #B0B0B0; }
    }
  </style>

  <defs>
    <marker id="arrow" markerWidth="13" markerHeight="13" refX="11.7" refY="3.9" orient="auto" markerUnits="strokeWidth">
      <path d="M0,0 L0,7.8 L11.7,3.9 z" class="arrow-color"/>
    </marker>
  </defs>

  <!-- Title (positioned well above content) -->
  <text x="700" y="45" text-anchor="middle" font-family="Arial, sans-serif" font-size="32" font-weight="600" class="main-text">[Title]</text>

  <!-- MAIN FLOW DIAGRAM (Upper Section) -->
  <g id="main-flow">
    <!-- Phase labels above components -->
    <text x="[x]" y="95" text-anchor="middle" font-family="Arial, sans-serif" font-size="21" font-weight="500" class="secondary-text">[Phase Label]</text>
    
    <!-- Components with proper sizing for text -->
    <rect x="[x]" y="[y]" width="[sized-for-text]" height="70" fill="none" class="neon-[color]" rx="6.5"/>
    <text x="[center]" y="[y+45]" text-anchor="middle" font-family="Arial, sans-serif" font-size="24" font-weight="500" class="main-text">
      [Label - check width!]
    </text>
    
    <!-- For long text, use tspan for wrapping -->
    <text x="[center]" y="[y+35]" text-anchor="middle" font-family="Arial, sans-serif" font-size="22" class="main-text">
      <tspan x="[center]" dy="0">[First line]</tspan>
      <tspan x="[center]" dy="25">[Second line]</tspan>
    </text>
  </g>

  <!-- PROGRESS INDICATOR / LEGEND (Lower Section) -->
  <g id="progress-legend">
    <!-- Background gradient bar (optional) -->
    <defs>
      <linearGradient id="flowGradient" x1="0%" y1="0%" x2="100%" y2="0%">
        <stop offset="0%" style="stop-color:#4A90E2;stop-opacity:0.3" />
        <stop offset="50%" style="stop-color:#BD10E0;stop-opacity:0.3" />
        <stop offset="100%" style="stop-color:#7ED321;stop-opacity:0.3" />
      </linearGradient>
    </defs>
    
    <rect x="50" y="500" width="1300" height="80" fill="url(#flowGradient)" rx="10" opacity="0.2"/>
    
    <!-- Stage markers -->
    <circle cx="[x1]" cy="540" r="8" class="neon-blue" fill="none"/>
    <circle cx="[x2]" cy="540" r="8" class="neon-orange" fill="none"/>
    <circle cx="[x3]" cy="540" r="8" class="neon-purple" fill="none"/>
    <circle cx="[x4]" cy="540" r="8" class="neon-green" fill="none"/>
    
    <!-- Connecting lines -->
    <line x1="[x1+8]" y1="540" x2="[x2-8]" y2="540" class="arrow-color" stroke-width="2" opacity="0.4"/>
    
    <!-- Stage labels (below markers) -->
    <text x="[x]" y="610" text-anchor="middle" font-family="Arial, sans-serif" font-size="18" class="secondary-text">[Stage]</text>
  </g>

  <!-- Description text (bottom, well-spaced) -->
  <text x="700" y="720" text-anchor="middle" font-family="Arial, sans-serif" font-size="21" class="secondary-text">
    [Main description line]
  </text>
  <text x="700" y="755" text-anchor="middle" font-family="Arial, sans-serif" font-size="21" class="secondary-text">
    [Secondary description line]
  </text>
</svg>
```

## Updated Component Styling Rules

### Boxes/Rectangles (1.3x Scale)
- **Standard Dimensions**: 
  - Vertical flow: width="156-260" height="59-70"
  - Horizontal flow: width="200-300" height="60-70"
  - Compact components: width="100" height="50"
  - **IMPORTANT**: Width MUST accommodate text content
- **Text Overflow Handling**:
  - Review all text against box width before finalizing
  - Use dynamic width sizing based on text length
  - Consider multi-line text with <tspan> elements
- **Border**: stroke-width="2.6" (light) / "2.8" (dark), no fill, rounded corners rx="5-6.5"
- **Colors**: Use CSS classes (neon-blue, neon-orange, etc.) for theme support
- **Spacing**: 
  - Vertical: minimum 35px spacing
  - Horizontal: minimum 70px spacing between major phases

### Text (1.3x Scale)
- **Title**: 
  - font-size="32", font-weight="600", class="main-text"
  - Position FAR above content (y="45" minimum)
- **Labels**: 
  - font-size="22-24", font-weight="500", class="main-text"
  - Centered in boxes (text-anchor="middle")
  - Check width constraints!
- **Compact labels**: font-size="18", for small components in grids
- **Section headers**: font-size="21", font-weight="500", class="secondary-text"
- **Descriptions**: font-size="21", class="secondary-text"
- **Font**: Always "Arial, sans-serif"
- **Text Wrapping**: Use <tspan> for multi-line text in boxes

### Arrows (1.3x Scale)
- **Main flow**: Solid lines, stroke-width="2.6", opacity="0.7"
- **Optional/parallel**: Dashed lines, stroke-dasharray="3.9,3.9", opacity="0.5"
- **Feedback loops**: Dashed curves, stroke-dasharray="3.9,3.9", opacity="0.5"
- **Arrow heads**: Enlarged triangular marker (13x13), uses arrow-color class
- **Connection lines**: stroke-width="1.5", opacity="0.5" for component merging
- **Progress connections**: stroke-width="2", opacity="0.4"

### Layout (1.3x Scale)
- **Canvas**: 
  - Vertical flows: 1040px width minimum
  - Horizontal flows: 1400px width recommended
  - Aspect ratio: 16:9 for horizontal, 3:4 for vertical
- **Content Zones**:
  - Title zone: 0-80px
  - Main diagram: 80-450px (horizontal) or 80-600px (vertical)
  - Progress/Legend: 500-650px
  - Descriptions: 700-800px
- **Margins**: 50px from edges minimum
- **Spacing**: 
  - Title to content: 50px minimum
  - Main diagram to progress: 100px minimum
  - Vertical flows: 52-78px between components
  - Horizontal flows: 70-100px between major phases
- **Component grid**: 
  - Can use 2x2 grids for related components
  - Merge lines with opacity="0.5" for grouped items
- **Alignment**: 
  - Center-align titles at x="700" (1400px width)
  - Use consistent alignment within phases

## Theme-Aware Color System

### Light Theme (Default)
- **Blue**: #4A90E2 (Input/User/Start elements)
- **Orange**: #F5A623 (Processing/Scripts/Detection)
- **Purple**: #BD10E0 (AI/ML/Decision/Configuration)
- **Green**: #7ED321 (Execution/Action/Completion)
- **Cyan**: #50E3C2 (Output/Response/Storage)
- **Text**: #1a1a1a (main), #666 (secondary)

### Dark Theme (Neon Effects)
- **Blue**: #00D4FF with subtle glow
- **Orange**: #FF9500 with subtle glow
- **Purple**: #E040FB with subtle glow
- **Green**: #00FF88 with subtle glow
- **Cyan**: #00E5EA with subtle glow
- **Text**: #FFFFFF (main), #B0B0B0 (secondary)

## Example Usage

### For a beautiful dual-diagram composition:
```
"Create a horizontal flow SVG (1400x900) with DUAL DIAGRAM composition:

MAIN FLOW (top section):
- Start: ./botserver (neon-blue)
- OS Detection (neon-orange) 
- Component Installation (2x2 grid: PostgreSQL, Valkey, SeaweedFS, Qdrant)
- Configuration & Setup (neon-purple)
- Bot Deployment (vertical sub-flow with 3 steps)

PROGRESS INDICATOR (bottom section):
- Gradient background bar
- 4 stage markers: Start, Detect, Install & Configure, Deploy
- Connected with subtle lines

Position title well above content.
Check all text fits within boxes - adjust widths as needed.
Add descriptions at bottom with proper spacing.
Use CSS classes for theme support, subtle neon glow in dark mode."
```

### For a complex system with legend:
```
"Create an SVG with beautiful composition (1400x900):

MAIN ARCHITECTURE (upper 70%):
- Client requests flow horizontally through system
- API Gateway distributes to microservices
- Services connect to shared resources
- Use appropriate box widths for service names

LEGEND/KEY (lower 30%):
- Color-coded component types
- Connection type explanations
- Status indicators

Ensure title is well-separated from content.
Review all text against box constraints.
Include phase labels above component groups."
```

## Best Practices for Beautiful Compositions

### Do's
- ✅ **ALWAYS** create dual diagrams when showing processes/flows
- ✅ Position title with generous spacing from content
- ✅ Review every text label against its container width
- ✅ Use progress indicators for multi-stage processes
- ✅ Group related components with visual phases
- ✅ Maintain consistent vertical rhythm
- ✅ Add legend/progress bar as secondary visualization
- ✅ Use gradient backgrounds for progress bars
- ✅ Keep descriptions separate and well-spaced at bottom

### Don'ts
- ❌ Don't let text overflow boxes - adjust widths!
- ❌ Don't crowd title against diagram content
- ❌ Don't skip the progress indicator for process flows
- ❌ Don't use single diagram when dual would be clearer
- ❌ Don't forget to test text readability at different sizes
- ❌ Don't make boxes too small for their text content
- ❌ Don't position legend too close to main diagram

## Testing Your Beautiful Composition

Your SVG should:
1. Have TWO complementary visualizations (main + progress/legend)
2. Display title with ample separation from content
3. Fit all text comfortably within component boxes
4. Show clear visual hierarchy with phases/groupings
5. Include progress indicator for process flows
6. Position legend/progress bar with proper spacing
7. Maintain professional spacing throughout
8. Create visual rhythm with consistent element spacing
9. Work beautifully in both light and dark themes
10. Feel balanced and uncluttered

## The Golden Rule

**"Beautiful composition is the standard!"** - Every diagram should tell its story twice: once in the main flow, and again in the progress indicator or legend. This dual approach creates professional, scannable, and memorable visualizations.
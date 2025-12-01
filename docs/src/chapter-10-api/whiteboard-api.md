# Whiteboard API

The Whiteboard API provides endpoints for collaborative drawing, diagramming, and visual collaboration within BotServer.

## Status

**⚠️ NOT IMPLEMENTED**

This API is planned for future development but is not currently available in BotServer.

## Planned Features

The Whiteboard API will enable collaborative real-time drawing, shape and diagram creation, text annotations, image uploads, multi-user cursors, version history, and export capabilities. These features will provide teams with a complete visual collaboration environment integrated directly into the BotServer platform.

## Planned Endpoints

### Whiteboard Management

The whiteboard management endpoints handle the lifecycle of whiteboard instances. Creating a whiteboard uses `POST /api/v1/whiteboards`, while retrieving whiteboard details uses `GET /api/v1/whiteboards/{board_id}`. Updates are handled through `PATCH /api/v1/whiteboards/{board_id}`, deletion through `DELETE /api/v1/whiteboards/{board_id}`, and listing all whiteboards through `GET /api/v1/whiteboards`.

### Collaboration

Real-time collaboration is managed through several endpoints. Users join sessions via `POST /api/v1/whiteboards/{board_id}/join` and leave via `POST /api/v1/whiteboards/{board_id}/leave`. The current participant list is available at `GET /api/v1/whiteboards/{board_id}/participants`. For real-time updates, a WebSocket connection is established at `WebSocket /api/v1/whiteboards/{board_id}/ws`.

### Content Operations

Content manipulation endpoints allow adding elements with `POST /api/v1/whiteboards/{board_id}/elements`, updating them with `PATCH /api/v1/whiteboards/{board_id}/elements/{element_id}`, and removing them with `DELETE /api/v1/whiteboards/{board_id}/elements/{element_id}`. The entire board can be cleared using `POST /api/v1/whiteboards/{board_id}/clear`.

### Export

Export functionality supports multiple formats. PNG export is available at `GET /api/v1/whiteboards/{board_id}/export/png`, SVG at `GET /api/v1/whiteboards/{board_id}/export/svg`, and PDF at `GET /api/v1/whiteboards/{board_id}/export/pdf`.

## Planned Integration with BASIC

When implemented, whiteboard features will be accessible via BASIC keywords:

```basic
' Create whiteboard (not yet available)
board_id = CREATE WHITEBOARD "Architecture Diagram"
SHARE WHITEBOARD board_id, ["user123", "user456"]

' Add content (not yet available)
ADD TO WHITEBOARD board_id, "rectangle", {x: 100, y: 100, width: 200, height: 100}
ADD TO WHITEBOARD board_id, "text", {x: 150, y: 150, text: "Component A"}

' Export whiteboard (not yet available)
image_url = EXPORT WHITEBOARD board_id, "png"
SEND FILE image_url
```

## Planned Data Models

### Whiteboard

```json
{
  "board_id": "wb_123",
  "name": "Architecture Diagram",
  "owner": "user123",
  "created_at": "2024-01-15T10:00:00Z",
  "updated_at": "2024-01-15T14:30:00Z",
  "settings": {
    "background": "grid",
    "canvas_width": 1920,
    "canvas_height": 1080,
    "allow_anonymous": false,
    "max_participants": 50
  },
  "participants": [
    {
      "user_id": "user123",
      "role": "owner",
      "cursor_position": {"x": 500, "y": 300}
    }
  ],
  "element_count": 42
}
```

### Drawing Element

```json
{
  "element_id": "elem_456",
  "board_id": "wb_123",
  "type": "rectangle",
  "properties": {
    "x": 100,
    "y": 100,
    "width": 200,
    "height": 100,
    "fill": "#ffffff",
    "stroke": "#000000",
    "stroke_width": 2
  },
  "created_by": "user123",
  "created_at": "2024-01-15T10:05:00Z",
  "z_index": 1
}
```

## Planned Features Detail

### Drawing Tools

The drawing tools will include basic shapes such as rectangles, circles, triangles, lines, and arrows. Freehand drawing will support pen, pencil, and highlighter modes. Text tools will provide labels, sticky notes, and comments. Smart connectors will automatically route between shapes, and templates will offer pre-built layouts for flowcharts, mind maps, and wireframes.

### Collaboration Features

Real-time collaboration will include cursor tracking so users can see where others are working, presence indicators showing who is currently viewing the board, and change notifications for updates made by collaborators. A commenting system will enable discussions on specific elements. Version control will track the history of changes, and conflict resolution will handle simultaneous edits gracefully.

### Advanced Features

Advanced functionality will support layers for organizing complex diagrams, grouping to manipulate multiple elements together, and alignment and distribution tools for precise positioning. Copy and paste will work between boards, undo and redo history will allow reverting changes, and keyboard shortcuts will speed up common operations.

## Implementation Considerations

When implemented, the Whiteboard API will use WebSocket for real-time collaboration and implement CRDT (Conflict-free Replicated Data Types) for conflict-free editing. Data will be stored in PostgreSQL with JSON columns for flexibility. The cache component will improve performance for frequently accessed boards. SVG will serve as the primary format for rendering, and the system will support touch devices and stylus input. Access controls and permissions will ensure proper security.

## Alternative Solutions

Until the Whiteboard API is implemented, several alternatives are available.

External whiteboard services can be integrated, including Miro API, embedded Excalidraw, draw.io (diagrams.net), or Microsoft Whiteboard.

For simple drawing storage, you can store drawing data as JSON in bot memory:

```basic
' Store drawing as JSON
drawing = {
    "shapes": [
        {"type": "rect", "x": 10, "y": 10, "w": 100, "h": 50}
    ]
}
SET BOT MEMORY "drawing_001", JSON_STRINGIFY(drawing)
```

Image-based collaboration offers another approach, allowing you to upload and annotate images, use existing image editing APIs, or share screenshots with markup.

## Future Technology Stack

The planned implementation will use the Canvas API or SVG for rendering, WebSocket for real-time synchronization, Y.js or OT.js for collaborative editing, Fabric.js for canvas manipulation, PostgreSQL for data persistence, cache for real-time state management, and Sharp for image processing.

## Workaround Example

Until the Whiteboard API is available, you can implement basic diagram storage:

```basic
' Simple diagram system using text
FUNCTION CreateDiagram(name)
    diagram = {
        "name": name,
        "elements": [],
        "connections": []
    }
    SET BOT MEMORY "diagram_" + name, JSON_STRINGIFY(diagram)
    RETURN name
END FUNCTION

FUNCTION AddElement(diagram_name, element_type, label)
    diagram_key = "diagram_" + diagram_name
    diagram_json = GET BOT MEMORY diagram_key
    diagram = JSON_PARSE(diagram_json)
    
    element = {
        "id": GENERATE_ID(),
        "type": element_type,
        "label": label
    }
    
    diagram.elements = APPEND(diagram.elements, element)
    SET BOT MEMORY diagram_key, JSON_STRINGIFY(diagram)
    RETURN element.id
END FUNCTION

FUNCTION GenerateAsciiDiagram(diagram_name)
    diagram_json = GET BOT MEMORY "diagram_" + diagram_name
    diagram = JSON_PARSE(diagram_json)
    
    output = "Diagram: " + diagram.name + "\n\n"
    
    FOR EACH element IN diagram.elements
        IF element.type = "box" THEN
            output = output + "[" + element.label + "]\n"
        ELSE IF element.type = "circle" THEN
            output = output + "(" + element.label + ")\n"
        END IF
    NEXT
    
    RETURN output
END FUNCTION
```

## Use Cases

### Technical Planning

Technical teams can use the Whiteboard API for architecture diagrams, database schemas, network topology visualization, UML diagrams, and flowcharts that document system design and processes.

### Business Collaboration

Business users will benefit from mind mapping for brainstorming, process flow documentation, organizational charts, collaborative brainstorming sessions, and project planning visualizations.

### Education

Educational applications include teaching illustrations, student collaboration on group projects, visual problem solving, and graphical explanations of complex concepts.

## Integration Points

When available, the Whiteboard API will integrate with the [Storage API](./storage-api.md) for saving whiteboard data, the [Calls API](./calls-api.md) for sharing during video calls, [Document Processing](./document-processing.md) for import and export capabilities, and the [Notifications API](./notifications-api.md) for collaboration alerts.

## Status Updates

Check the [GitHub repository](https://github.com/generalbots/botserver) for updates on Whiteboard API implementation status.

For immediate visual collaboration needs, consider embedding existing solutions like Excalidraw or Miro rather than waiting for the native implementation.
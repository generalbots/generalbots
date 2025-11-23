# Whiteboard API

The Whiteboard API provides endpoints for collaborative drawing, diagramming, and visual collaboration within BotServer.

## Status

**⚠️ NOT IMPLEMENTED**

This API is planned for future development but is not currently available in BotServer.

## Planned Features

The Whiteboard API will enable:
- Collaborative real-time drawing
- Shape and diagram creation
- Text annotations
- Image uploads
- Multi-user cursors
- Version history
- Export capabilities

## Planned Endpoints

### Whiteboard Management
- `POST /api/v1/whiteboards` - Create whiteboard
- `GET /api/v1/whiteboards/{board_id}` - Get whiteboard
- `PATCH /api/v1/whiteboards/{board_id}` - Update whiteboard
- `DELETE /api/v1/whiteboards/{board_id}` - Delete whiteboard
- `GET /api/v1/whiteboards` - List whiteboards

### Collaboration
- `POST /api/v1/whiteboards/{board_id}/join` - Join session
- `POST /api/v1/whiteboards/{board_id}/leave` - Leave session
- `GET /api/v1/whiteboards/{board_id}/participants` - List participants
- `WebSocket /api/v1/whiteboards/{board_id}/ws` - Real-time updates

### Content Operations
- `POST /api/v1/whiteboards/{board_id}/elements` - Add element
- `PATCH /api/v1/whiteboards/{board_id}/elements/{element_id}` - Update element
- `DELETE /api/v1/whiteboards/{board_id}/elements/{element_id}` - Delete element
- `POST /api/v1/whiteboards/{board_id}/clear` - Clear board

### Export
- `GET /api/v1/whiteboards/{board_id}/export/png` - Export as PNG
- `GET /api/v1/whiteboards/{board_id}/export/svg` - Export as SVG
- `GET /api/v1/whiteboards/{board_id}/export/pdf` - Export as PDF

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
- **Basic Shapes**: Rectangle, circle, triangle, line, arrow
- **Freehand Drawing**: Pen, pencil, highlighter
- **Text Tools**: Labels, sticky notes, comments
- **Connectors**: Smart connectors between shapes
- **Templates**: Flowcharts, mind maps, wireframes

### Collaboration Features
- Real-time cursor tracking
- User presence indicators
- Change notifications
- Commenting system
- Version control
- Conflict resolution

### Advanced Features
- Layers support
- Grouping elements
- Alignment and distribution
- Copy/paste between boards
- Undo/redo history
- Keyboard shortcuts

## Implementation Considerations

When implemented, the Whiteboard API will:

1. **Use WebSocket** for real-time collaboration
2. **Implement CRDT** for conflict-free editing
3. **Store in PostgreSQL** with JSON columns
4. **Cache in Redis** for performance
5. **Use SVG** as primary format
6. **Support touch devices** and stylus input
7. **Include access controls** and permissions

## Alternative Solutions

Until the Whiteboard API is implemented, consider:

1. **External Whiteboard Services**
   - Integrate with Miro API
   - Embed Excalidraw
   - Use draw.io (diagrams.net)
   - Connect to Microsoft Whiteboard

2. **Simple Drawing Storage**
   ```basic
   ' Store drawing as JSON
   drawing = {
       "shapes": [
           {"type": "rect", "x": 10, "y": 10, "w": 100, "h": 50}
       ]
   }
   SET BOT MEMORY "drawing_001", JSON_STRINGIFY(drawing)
   ```

3. **Image-Based Collaboration**
   - Upload and annotate images
   - Use existing image editing APIs
   - Share screenshots with markup

## Future Technology Stack

The planned implementation will use:
- **Canvas API** or **SVG** - Rendering
- **WebSocket** - Real-time sync
- **Y.js** or **OT.js** - Collaborative editing
- **Fabric.js** - Canvas manipulation
- **PostgreSQL** - Data persistence
- **Redis** - Real-time state
- **Sharp** - Image processing

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
- Architecture diagrams
- Database schemas
- Network topology
- UML diagrams
- Flowcharts

### Business Collaboration
- Mind mapping
- Process flows
- Organizational charts
- Brainstorming sessions
- Project planning

### Education
- Teaching illustrations
- Student collaboration
- Problem solving
- Visual explanations

## Integration Points

When available, the Whiteboard API will integrate with:
- [Storage API](./storage-api.md) - Save whiteboard data
- [Calls API](./calls-api.md) - Share during calls
- [Document Processing](./document-processing.md) - Import/export
- [Notifications API](./notifications-api.md) - Collaboration alerts

## Status Updates

Check the [GitHub repository](https://github.com/generalbots/botserver) for updates on Whiteboard API implementation status.

For immediate visual collaboration needs, consider embedding existing solutions like Excalidraw or Miro rather than waiting for the native implementation.
# Video API

> **API for video project management, editing, scene composition, AI-powered enhancement, and export.**

---

## Base URL

```
/api/video
```

## Authentication

All endpoints require a valid session token via `Authorization: Bearer <token>` header.

---

## Endpoints

### List Projects

**`GET /api/video/projects`**

Returns a list of all video projects.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `page` | integer | No | Page number (default: 1) |
| `limit` | integer | No | Items per page (default: 20) |
| `status` | string | No | Filter: `draft`, `processing`, `completed`, `exported` |

**Response:**
```json
{
  "projects": [
    {
      "id": "uuid-string",
      "name": "Product Demo v2",
      "duration_seconds": 120,
      "resolution": "1920x1080",
      "clip_count": 8,
      "status": "draft",
      "created_at": "2026-01-15T10:30:00Z",
      "updated_at": "2026-01-20T14:45:00Z",
      "thumbnail_url": "/api/video/projects/uuid/thumbnail"
    }
  ],
  "total": 12
}
```

---

### Create Project

**`POST /api/video/projects`**

Creates a new video project.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Project name |
| `resolution` | string | No | Output resolution (default: `1920x1080`) |
| `fps` | integer | No | Frames per second (default: 30) |
| `template_id` | string | No | Template to base project on |

**Request Body:**
```json
{
  "name": "Product Demo v2",
  "resolution": "1920x1080",
  "fps": 30
}
```

**Response:**
```json
{
  "id": "uuid-string",
  "name": "Product Demo v2",
  "resolution": "1920x1080",
  "fps": 30,
  "created_at": "2026-01-20T15:00:00Z"
}
```

---

### Update Clip

**`PUT /api/video/clips/:id`**

Updates a clip's properties within a project.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string (path) | Yes | Clip ID |
| `start_time` | number | No | Start time in seconds |
| `end_time` | number | No | End time in seconds |
| `trim_start` | number | No | Trim from beginning in seconds |
| `trim_end` | number | No | Trim from end in seconds |
| `volume` | number | No | Volume level (0.0 - 1.0) |
| `opacity` | number | No | Opacity (0.0 - 1.0) |
| `position` | object | No | `{x, y, z}` coordinates |

**Request Body:**
```json
{
  "start_time": 10.5,
  "end_time": 25.0,
  "volume": 0.8,
  "opacity": 1.0
}
```

**Response:**
```json
{
  "id": "clip-uuid",
  "updated": true,
  "duration_seconds": 14.5
}
```

---

### Delete Clip

**`DELETE /api/video/clips/:id`**

Removes a clip from the project.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string (path) | Yes | Clip ID |

**Response:**
```json
{
  "deleted": true,
  "id": "clip-uuid"
}
```

---

### Split Clip

**`POST /api/video/clips/:id/split`**

Splits a clip at a specified timestamp into two separate clips.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string (path) | Yes | Clip ID to split |
| `at_time` | number | Yes | Timestamp in seconds to split at |

**Request Body:**
```json
{
  "at_time": 15.3
}
```

**Response:**
```json
{
  "original_clip_id": "clip-uuid-1",
  "new_clip_id": "clip-uuid-2",
  "split_at": 15.3,
  "first_duration": 15.3,
  "second_duration": 9.7
}
```

---

### Delete Audio Track

**`DELETE /api/video/audio/:id`**

Removes an audio track from a project.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string (path) | Yes | Audio track ID |

**Response:**
```json
{
  "deleted": true,
  "id": "audio-track-uuid"
}
```

---

### Upload to Project

**`POST /api/video/projects/:id/upload`**

Uploads a video file to an existing project.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string (path) | Yes | Project ID |
| `file` | binary | Yes | Video file (mp4, mov, avi, webm) |
| `name` | string | No | Clip name (default: filename) |

**Response:**
```json
{
  "clip_id": "clip-uuid",
  "filename": "intro-sequence.mp4",
  "duration_seconds": 30.5,
  "resolution": "1920x1080",
  "format": "mp4",
  "size_bytes": 15728640,
  "processing": true
}
```

---

### Generate Preview

**`GET /api/video/projects/:id/preview`**

Generates or retrieves a preview of the project timeline.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string (path) | Yes | Project ID |
| `quality` | string | No | Preview quality: `low`, `medium`, `high` (default: `medium`) |

**Response:**
```json
{
  "preview_url": "/api/video/preview/uuid-string.mp4",
  "duration_seconds": 120,
  "quality": "medium",
  "generated_at": "2026-01-20T15:10:00Z",
  "size_bytes": 5242880
}
```

---

### Text-to-Speech

**`POST /api/video/projects/:id/tts`**

Generates a voiceover using text-to-speech for the project.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string (path) | Yes | Project ID |
| `text` | string | Yes | Text to convert to speech |
| `voice` | string | No | Voice ID or name (default: system default) |
| `speed` | number | No | Speech speed multiplier (0.5 - 2.0, default: 1.0) |
| `language` | string | No | Language code (default: `pt-BR`) |

**Request Body:**
```json
{
  "text": "Bem-vindos ao nosso produto. Vamos mostrar os principais recursos.",
  "voice": "pt-br-female-1",
  "speed": 1.0,
  "language": "pt-BR"
}
```

**Response:**
```json
{
  "audio_id": "tts-uuid",
  "duration_seconds": 8.5,
  "audio_url": "/api/video/audio/tts-uuid",
  "processing": true
}
```

---

### Generate Scenes

**`POST /api/video/projects/:id/scenes`**

Uses AI to generate scene compositions from a script or description.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string (path) | Yes | Project ID |
| `script` | string | Yes | Scene description or script |
| `scene_count` | integer | No | Number of scenes (default: auto) |
| `style` | string | No | Visual style: `cinematic`, `minimal`, `dynamic` |

**Request Body:**
```json
{
  "script": "Opening with product close-up, transition to team working, end with logo reveal",
  "scene_count": 3,
  "style": "cinematic"
}
```

**Response:**
```json
{
  "scenes": [
    {
      "id": "scene-uuid-1",
      "name": "Product Close-up",
      "duration_seconds": 10,
      "clips": ["clip-uuid-1"],
      "transitions": ["fade-in"]
    },
    {
      "id": "scene-uuid-2",
      "name": "Team Working",
      "duration_seconds": 15,
      "clips": ["clip-uuid-2", "clip-uuid-3"],
      "transitions": ["crossfade"]
    }
  ],
  "total_duration": 35
}
```

---

### Reframe Video

**`POST /api/video/projects/:id/reframe`**

AI-powered reframing to adapt video for different aspect ratios (e.g., 16:9 to 9:16 for mobile).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string (path) | Yes | Project ID |
| `target_ratio` | string | Yes | Target ratio: `9:16`, `1:1`, `4:5`, `16:9` |
| `focus_mode` | string | No | Focus: `center`, `smart`, `manual` |
| `focus_point` | object | No | `{x, y}` for manual focus (0.0-1.0) |

**Request Body:**
```json
{
  "target_ratio": "9:16",
  "focus_mode": "smart"
}
```

**Response:**
```json
{
  "reframed_project_id": "new-project-uuid",
  "target_ratio": "9:16",
  "clips_reframed": 8,
  "processing": true
}
```

---

### Enhance Video

**`POST /api/video/projects/:id/enhance`**

Applies AI-powered video enhancement (color correction, stabilization, noise reduction).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string (path) | Yes | Project ID |
| `enhancements` | array | Yes | Enhancements to apply |
| `intensity` | number | No | Enhancement intensity (0.0 - 1.0, default: 0.5) |

**Enhancement types:** `color_correction`, `stabilization`, `noise_reduction`, `sharpness`, `brightness`, `contrast`

**Request Body:**
```json
{
  "enhancements": ["color_correction", "stabilization", "noise_reduction"],
  "intensity": 0.7
}
```

**Response:**
```json
{
  "enhanced_project_id": "new-project-uuid",
  "enhancements_applied": ["color_correction", "stabilization", "noise_reduction"],
  "processing": true,
  "estimated_time_seconds": 45
}
```

---

### Delete Keyframe

**`DELETE /api/video/keyframes/:id`**

Removes a keyframe animation from a clip.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string (path) | Yes | Keyframe ID |

**Response:**
```json
{
  "deleted": true,
  "id": "keyframe-uuid"
}
```

---

### Get Templates

**`GET /api/video/templates`**

Returns available video project templates.

**Response:**
```json
{
  "templates": [
    {
      "id": "template-uuid",
      "name": "Product Demo",
      "description": "Template for product demonstration videos",
      "duration_seconds": 60,
      "resolution": "1920x1080",
      "category": "marketing",
      "thumbnail_url": "/api/video/templates/template-uuid/thumbnail"
    },
    {
      "id": "template-uuid-2",
      "name": "Social Media Story",
      "description": "Vertical format for Instagram/TikTok stories",
      "duration_seconds": 15,
      "resolution": "1080x1920",
      "category": "social",
      "thumbnail_url": "/api/video/templates/template-uuid-2/thumbnail"
    }
  ]
}
```

---

### Chat with Project

**`POST /api/video/projects/:id/chat`**

AI assistant for project — ask questions about the video, get suggestions, or request edits via natural language.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string (path) | Yes | Project ID |
| `message` | string | Yes | User message or instruction |

**Request Body:**
```json
{
  "message": "Can you make the intro shorter and add a fade transition between scene 2 and 3?"
}
```

**Response:**
```json
{
  "response": "I've shortened the intro from 10s to 5s and added a crossfade transition between scenes 2 and 3. The total project duration is now 115 seconds.",
  "actions_taken": [
    {"type": "trim", "clip_id": "clip-uuid-1", "new_duration": 5},
    {"type": "add_transition", "between": ["scene-2", "scene-3"], "transition": "crossfade"}
  ],
  "suggestions": [
    "Add background music to match the new pacing",
    "Consider adding text overlays for key points"
  ]
}
```

---

### Export Project

**`POST /api/video/projects/:id/export`**

Initiates video export/rendering.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string (path) | Yes | Project ID |
| `format` | string | No | Output format: `mp4`, `webm`, `mov` (default: `mp4`) |
| `quality` | string | No | Export quality: `draft`, `standard`, `high`, `ultra` |
| `codec` | string | No | Video codec: `h264`, `h265`, `vp9` |

**Request Body:**
```json
{
  "format": "mp4",
  "quality": "high",
  "codec": "h264"
}
```

**Response:**
```json
{
  "export_id": "export-uuid",
  "status": "queued",
  "estimated_time_seconds": 120,
  "websocket_url": "ws://localhost:8080/api/video/ws/export/export-uuid"
}
```

---

### Get Export Status

**`GET /api/video/exports/:id/status`**

Checks the status of a video export job.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string (path) | Yes | Export ID |

**Response:**
```json
{
  "export_id": "export-uuid",
  "status": "processing",
  "progress_percent": 65,
  "current_frame": 1950,
  "total_frames": 3000,
  "eta_seconds": 42,
  "output_url": null
}
```

**Status values:** `queued`, `processing`, `completed`, `failed`

---

### Track Analytics View

**`POST /api/video/analytics/view`**

Records an analytics event when a video is viewed.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_id` | string | Yes | Project ID |
| `viewer_id` | string | No | Viewer identifier |
| `duration_watched` | number | Yes | Seconds watched |
| `source` | string | No | View source: `web`, `embed`, `share` |

**Request Body:**
```json
{
  "project_id": "uuid-string",
  "duration_watched": 45.2,
  "source": "web"
}
```

**Response:**
```json
{
  "recorded": true,
  "total_views": 142,
  "avg_watch_percent": 78.5
}
```

---

### Export WebSocket Stream

**`GET /api/video/ws/export/:id`** (WebSocket)

Real-time export progress stream via WebSocket.

**Connection:**
```
ws://localhost:8080/api/video/ws/export/{export_id}
```

**Messages received:**
```json
{
  "type": "progress",
  "export_id": "export-uuid",
  "progress_percent": 65,
  "current_frame": 1950,
  "total_frames": 3000,
  "eta_seconds": 42
}
```

```json
{
  "type": "completed",
  "export_id": "export-uuid",
  "output_url": "/api/video/exports/export-uuid/download",
  "format": "mp4",
  "size_bytes": 52428800,
  "duration_seconds": 120
}
```

```json
{
  "type": "error",
  "export_id": "export-uuid",
  "error": "Encoding failed at frame 2100",
  "code": "ENCODING_ERROR"
}
```

---

## Video Formats

| Format | Extension | Use Case |
|--------|-----------|----------|
| `mp4` | .mp4 | Universal playback, web |
| `webm` | .webm | Web-optimized, smaller size |
| `mov` | .mov | Apple ecosystem, editing |
| `avi` | .avi | Legacy compatibility |

---

## See Also

- [Slides API](slides-api.md) - Presentation creation
- [Docs API](docs-api.md) - Word processor documents
- [Research API](research-api.md) - Research collections

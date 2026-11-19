use super::super::ApiEndpoint;

pub(super) static PLAYER_DASHBOARDS_AND_VIBE: &[ApiEndpoint] = &[
    ApiEndpoint {
        method: "POST",
        path: "/api/ui/research/web/summarize",
        summary: "Summarize web search results",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/ui/research/web/deep",
        summary: "Deep research on a topic",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/ui/research/web/instant",
        summary: "Instant answer from the web",
    },
    // Vibe / project workbench
    ApiEndpoint {
        method: "POST",
        path: "/api/vibe/run",
        summary: "Run an authenticated Vibe agent request against a selected project",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/vibe/runs",
        summary: "List Vibe agent runs",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/vibe/projects",
        summary: "List the user's Vibe projects",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/vibe/projects/:project_id",
        summary: "Get a Vibe project",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/vibe/projects/:project_id/preview",
        summary: "Resolve a selected project's live preview",
    },
    // Catalog / apps
    ApiEndpoint {
        method: "GET",
        path: "/api/apps/catalog",
        summary: "Suite application catalog",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/apps/integrations/catalog",
        summary: "Search the integration provider and action catalog",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/apps/integrations/catalog/:provider",
        summary: "Get one integration provider and its action catalog",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/catalog/products",
        summary: "Public product catalog",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/catalog/plans",
        summary: "Public plans catalog",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/catalog/prices.json",
        summary: "Price list (JSON-LD)",
    },
    // Organization / cloud
    ApiEndpoint {
        method: "GET",
        path: "/api/organizations/current",
        summary: "Current organization profile",
    },
    ApiEndpoint {
        method: "PUT",
        path: "/api/organizations/current",
        summary: "Update organization profile",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/organizations/current/stats",
        summary: "Organization statistics",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/cloud/auth/signup",
        summary: "Create a cloud workspace account",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/cloud/auth/login",
        summary: "Log into the cloud",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/cloud/domains",
        summary: "List bot domain mappings",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/cloud/domains",
        summary: "Map a domain to a bot",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/domains/resolve",
        summary: "Resolve a hostname to a bot",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/autotask/classify",
        summary: "Classify a plain-language intent into an automation plan",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/autotask/compile",
        summary: "Compile an automation plan into a BASIC script",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/autotask/decide",
        summary: "Decide immediate vs scheduled execution mode",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/autotask/execute",
        summary: "Execute an automation plan (compiled BASIC script)",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/autotask/create-and-execute",
        summary: "Turn a plain-language request into an automated task and run it",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/autotask/tasks",
        summary: "List AutoTask runs and their status",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/autotask/stats",
        summary: "AutoTask statistics: runs, success rate, pending approvals",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/autotask/tasks/:task_id/approve",
        summary: "Approve a pending automated task",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/autotask/tasks/:task_id/cancel",
        summary: "Cancel a queued or running automated task",
    },
    // Attendant / human handoff
    ApiEndpoint {
        method: "POST",
        path: "/api/attendant/attachments",
        summary: "Upload an attachment to an attendant conversation",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/attendant/attachments/:id",
        summary: "Download an attendant conversation attachment",
    },
    // Meet / recording
    ApiEndpoint {
        method: "POST",
        path: "/api/meet/rooms/:id/recording/start",
        summary: "Start recording a meeting (host/co-host only)",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/meet/rooms/:id/recording/stop",
        summary: "Stop the active meeting recording",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/meet/rooms/:id/recordings",
        summary: "List recordings for a meeting room",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/meet/recordings/:recording_id/file",
        summary: "Stream or download a meeting recording file",
    },
    // Player / playlists
    ApiEndpoint {
        method: "GET",
        path: "/api/player/playlists",
        summary: "List media playlists for the current user",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/player/playlists",
        summary: "Create a media playlist",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/player/playlists/:id",
        summary: "Get a playlist with its items",
    },
    ApiEndpoint {
        method: "PUT",
        path: "/api/player/playlists/:id",
        summary: "Rename a media playlist",
    },
    ApiEndpoint {
        method: "DELETE",
        path: "/api/player/playlists/:id",
        summary: "Delete a media playlist",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/player/playlists/:id/items",
        summary: "Add a media item to a playlist",
    },
    ApiEndpoint {
        method: "DELETE",
        path: "/api/player/playlists/:id/items/:item_id",
        summary: "Remove a media item from a playlist",
    },
    ApiEndpoint {
        method: "PUT",
        path: "/api/player/playlists/:id/reorder",
        summary: "Reorder playlist items",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/player/playlists/:id/analytics",
        summary: "Playlist analytics: counts, total items, recent activity",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/player/playbacks",
        summary: "Record a media playback event (play, pause, complete)",
    },
    // Dashboards / analytics
    ApiEndpoint {
        method: "GET",
        path: "/api/dashboards",
        summary: "List dashboards with their widgets",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/dashboards",
        summary: "Create a dashboard",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/dashboards/:id",
        summary: "Get a dashboard with its widgets and filters",
    },
    ApiEndpoint {
        method: "PUT",
        path: "/api/dashboards/:id",
        summary: "Rename or reconfigure a dashboard",
    },
    ApiEndpoint {
        method: "DELETE",
        path: "/api/dashboards/:id",
        summary: "Delete a dashboard",
    },
    ApiEndpoint {
        method: "POST",
        path: "/api/dashboards/:id/widgets",
        summary: "Add a widget to a dashboard",
    },
    ApiEndpoint {
        method: "PUT",
        path: "/api/dashboards/:id/widgets/:widget_id",
        summary: "Update a widget title, size or chart configuration",
    },
    ApiEndpoint {
        method: "DELETE",
        path: "/api/dashboards/:id/widgets/:widget_id",
        summary: "Remove a widget from a dashboard",
    },
    ApiEndpoint {
        method: "GET",
        path: "/api/dashboards/sources",
        summary: "List dashboard data sources",
    },
];

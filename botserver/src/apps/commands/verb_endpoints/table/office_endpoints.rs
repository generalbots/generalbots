use super::{VerbEndpoint, bind};

pub(super) static OFFICE_ENDPOINTS: &[VerbEndpoint] = &[
    // ——— #2037 — Docs (crates/botdocs) ———
    bind("docs.list", "GET", "/api/docs/list", &[], &[]),
    bind("docs.create", "POST", "/api/docs/new", &[], &[]),
    bind(
        "docs.update",
        "POST",
        "/api/docs/save",
        &[("file_id", "id")],
        &[],
    ),
    bind(
        "docs.get",
        "GET",
        "/api/docs/:id",
        &[("file_id", "id")],
        &[],
    ),
    bind(
        "docs.save",
        "POST",
        "/api/docs/save",
        &[("file_id", "id")],
        &[],
    ),
    bind(
        "docs.autosave",
        "POST",
        "/api/docs/autosave",
        &[("file_id", "id")],
        &[],
    ),
    bind(
        "docs.delete",
        "POST",
        "/api/docs/delete",
        &[("file_id", "id")],
        &[],
    ),
    bind(
        "docs.search",
        "GET",
        "/api/docs/search",
        &[("query", "q")],
        &[],
    ),
    bind(
        "docs.outline",
        "POST",
        "/api/docs/outline",
        &[("file_id", "doc_id")],
        &[],
    ),
    bind(
        "docs.collaborators",
        "GET",
        "/api/docs/:doc_id/collaborators",
        &[("file_id", "doc_id")],
        &[],
    ),
    bind(
        "docs.ai",
        "POST",
        "/api/docs/ai",
        &[("file_id", "doc_id"), ("instruction", "command")],
        &[],
    ),
    // #2448 — anchored comment add. `AddCommentRequest` takes `doc_id`,
    // `content`, `position` and `length`; the verb uses the uniform `file_id`
    // targeting key, so only that rename is needed.
    bind(
        "docs.comment.add",
        "POST",
        "/api/docs/comment",
        &[("file_id", "doc_id")],
        &[],
    ),
    // ——— #2448 — Docs utility verbs whose curated params map 1:1 onto the
    // registered handler fields. No new routes and no numeric bounds needed.
    // CompareDocumentsRequest { original_doc_id, modified_doc_id }.
    bind(
        "docs.compare",
        "POST",
        "/api/docs/compare",
        &[
            ("file_id", "original_doc_id"),
            ("version_id", "modified_doc_id"),
        ],
        &[],
    ),
    // EnableTrackChangesRequest { doc_id, enabled } — `enabled` passes through.
    bind(
        "docs.track_changes.enable",
        "POST",
        "/api/docs/track-changes/enable",
        &[("file_id", "doc_id")],
        &[],
    ),
    // OpenFromDriveRequest { bucket, path } — both pass through unchanged.
    bind(
        "docs.open_from_drive",
        "POST",
        "/api/docs/open-from-drive",
        &[],
        &[],
    ),
    // ——— #2037 — Sheet (crates/botsheet) ———
    bind("sheet.list", "GET", "/api/sheet/list", &[], &[]),
    bind("sheet.create", "POST", "/api/sheet/new", &[], &[]),
    bind(
        "sheet.cells.set",
        "POST",
        "/api/sheet/cells",
        &[("file_id", "sheet_id")],
        &[],
    ),
    bind(
        "sheet.get",
        "GET",
        "/api/sheet/:id",
        &[("file_id", "id")],
        &[],
    ),
    bind("sheet.new", "GET", "/api/sheet/new", &[], &[]),
    bind(
        "sheet.delete",
        "POST",
        "/api/sheet/delete",
        &[("file_id", "id")],
        &[],
    ),
    bind(
        "sheet.export",
        "POST",
        "/api/sheet/export",
        &[("file_id", "id")],
        &[],
    ),
    bind("sheet.import", "POST", "/api/sheet/import", &[], &[]),
    bind(
        "sheet.search",
        "GET",
        "/api/sheet/search",
        &[("query", "q")],
        &[],
    ),
    bind(
        "sheet.chart.create",
        "POST",
        "/api/sheet/chart",
        &[
            ("file_id", "id"),
            ("range", "data_range"),
            ("type", "chart_type"),
        ],
        &[],
    ),
    bind(
        "sheet.conditional_format.apply",
        "POST",
        "/api/sheet/conditional-format",
        &[("file_id", "id")],
        &[],
    ),
    bind(
        "sheet.pivot.create",
        "POST",
        "/api/sheet/pivot",
        &[("file_id", "sheet_id"), ("range", "source_range")],
        &[],
    ),
    // ——— #2448 — WorksheetActionRequest { id, name }: the generic targeting
    // key fills the required `id`; the sheet name passes through.
    bind(
        "sheet.worksheets.add",
        "POST",
        "/api/sheet/worksheets/add",
        &[("file_id", "id")],
        &[],
    ),
    // EvaluateAiPromptRequest { prompt, cell_ref?, sheet_id? }: the verb's
    // `instruction` is the prompt and `range` is the optional cell reference.
    bind(
        "sheet.ai.evaluate",
        "POST",
        "/api/sheet/ai/evaluate-prompt",
        &[("instruction", "prompt"), ("range", "cell_ref")],
        &[],
    ),
    // ——— #2037 — Slides (crates/botslides) ———
    bind("slides.list", "GET", "/api/slides/list", &[], &[]),
    bind("slides.create", "POST", "/api/slides/new", &[], &[]),
    bind(
        "slides.get",
        "GET",
        "/api/slides/:id",
        &[("file_id", "id")],
        &[],
    ),
    bind("slides.new", "GET", "/api/slides/new", &[], &[]),
    bind(
        "slides.delete",
        "POST",
        "/api/slides/delete",
        &[("file_id", "id")],
        &[],
    ),
    bind(
        "slides.export",
        "POST",
        "/api/slides/export",
        &[("file_id", "id")],
        &[],
    ),
    bind(
        "slides.search",
        "GET",
        "/api/slides/search",
        &[("query", "q")],
        &[],
    ),
    bind(
        "slides.slide.add",
        "POST",
        "/api/slides/slide/add",
        &[("file_id", "presentation_id"), ("after", "position")],
        &[("layout", "content")],
    ),
    bind(
        "slides.slide.delete",
        "POST",
        "/api/slides/slide/delete",
        &[("file_id", "presentation_id"), ("slide", "slide_index")],
        &[],
    ),
    bind(
        "slides.slide.duplicate",
        "POST",
        "/api/slides/slide/duplicate",
        &[("file_id", "presentation_id"), ("slide", "slide_index")],
        &[],
    ),
    bind(
        "slides.slide.notes",
        "POST",
        "/api/slides/slide/notes",
        &[("file_id", "presentation_id"), ("slide", "slide_index")],
        &[],
    ),
    bind(
        "slides.element.add",
        "POST",
        "/api/slides/element/add",
        &[("file_id", "presentation_id"), ("slide", "slide_index")],
        &[],
    ),
    // #2151 — the media verb had no endpoint, so a "deck with images" request
    // could only be answered with text. The element writer already stores an
    // `image` element with a `src`; bind the verb onto it.
    bind(
        "slides.media.add",
        "POST",
        "/api/slides/element/add",
        &[
            ("file_id", "presentation_id"),
            ("slide", "slide_index"),
            ("path", "src"),
            ("source", "src"),
        ],
        &[("element_type", "image")],
    ),
    bind(
        "slides.element.delete",
        "POST",
        "/api/slides/element/delete",
        &[("file_id", "presentation_id"), ("slide", "slide_index")],
        &[],
    ),
    bind(
        "slides.theme.set",
        "POST",
        "/api/slides/theme",
        &[("file_id", "presentation_id"), ("theme", "theme_name")],
        &[],
    ),
    // ——— #2448 — Slides verbs whose curated params map onto the registered
    // handler fields with string-only renames. ———
    // SavePresentationRequest { id?, name?, slides?, theme? }: a partial save.
    bind(
        "slides.save",
        "POST",
        "/api/slides/save",
        &[("file_id", "id")],
        &[],
    ),
    // StartPresenterRequest { presentation_id, settings? (default) }.
    bind(
        "slides.presenter.start",
        "POST",
        "/api/slides/presenter/start",
        &[("file_id", "presentation_id")],
        &[],
    ),
    // SlidesAiRequest { command (alias prompt), presentation_id?, slide_index? }.
    bind(
        "slides.ai",
        "POST",
        "/api/slides/ai",
        &[("file_id", "presentation_id"), ("instruction", "command")],
        &[],
    ),
    // SetTransitionRequest { presentation_id, slide_index (defaults to 0),
    // transition_type? } — the verb's transition name is the flat toolbar field.
    bind(
        "slides.transition.set",
        "POST",
        "/api/slides/transition",
        &[
            ("file_id", "presentation_id"),
            ("transition", "transition_type"),
        ],
        &[],
    ),
    // ——— #2037 — Canvas (crates/botcanvas) ———
    bind("canvas.list", "GET", "/api/canvas", &[], &[]),
    bind(
        "canvas.get",
        "GET",
        "/api/canvas/{canvas_id}",
        &[("file_id", "canvas_id")],
        &[],
    ),
    bind(
        "canvas.create",
        "POST",
        "/api/canvas",
        &[("title", "name")],
        &[],
    ),
];

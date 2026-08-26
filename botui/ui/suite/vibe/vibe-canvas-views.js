"use strict";

(function () {
    "use strict";

    var currentView = "design";
    var viewIds = ["design", "architecture", "diagrams", "metrics", "files"];

    var ICONS = {
        select: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 3l13 9-6 1 3 7-3 1-3-7-4 4z"/></svg>',
        rectangle: '<svg viewBox="0 0 24 24" aria-hidden="true"><rect x="4" y="4" width="16" height="16" rx="1"/></svg>',
        text: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 5h14M12 5v14M8 19h8"/></svg>',
        connector: '<svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="5" cy="18" r="2"/><circle cx="19" cy="6" r="2"/><path d="M7 17l10-10"/></svg>',
        clear: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 7h16M10 11v6M14 11v6M6 7l1 14h10l1-14M9 7V4h6v3"/></svg>',
        fit: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M8 3H3v5M16 3h5v5M8 21H3v-5M16 21h5v-5"/></svg>',
        generate: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 3l1.7 5.3L19 10l-5.3 1.7L12 17l-1.7-5.3L5 10l5.3-1.7z"/><path d="M19 16l.8 2.2L22 19l-2.2.8L19 22l-.8-2.2L16 19l2.2-.8z"/></svg>',
        design: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 4h16v16H4zM8 16l3-4 2 2 2-3 3 5"/></svg>',
        architecture: '<svg viewBox="0 0 24 24" aria-hidden="true"><rect x="3" y="4" width="7" height="5"/><rect x="14" y="4" width="7" height="5"/><rect x="8.5" y="15" width="7" height="5"/><path d="M10 6.5h4M17.5 9v3M12 12v3"/></svg>',
        diagrams: '<svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="5" cy="12" r="2"/><circle cx="19" cy="6" r="2"/><circle cx="19" cy="18" r="2"/><path d="M7 12h5l5-6M12 12l5 6"/></svg>',
        metrics: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 20V10M10 20V4M16 20v-7M22 20H2"/></svg>',
        files: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 4h7l2 3h7v13H4zM8 12h8M8 16h6"/></svg>',
    };

    function q(id) {
        return document.getElementById(id);
    }

    function esc(value) {
        var node = document.createElement("div");
        node.textContent = value == null ? "" : String(value);
        return node.innerHTML;
    }

    function projectName() {
        return typeof window.currentProject !== "undefined" && window.currentProject
            ? String(window.currentProject)
            : "Current project";
    }

    function projectId() {
        return typeof window.currentProjectId !== "undefined" && window.currentProjectId
            ? String(window.currentProjectId)
            : "";
    }

    function projectKind() {
        var name = projectName().toLowerCase();
        if (name.indexOf("calculator") !== -1) return "Web application";
        if (name.indexOf("bot") !== -1) return "Bot application";
        return "Project application";
    }

    function toolbarIcon(key) {
        return ICONS[key] || "";
    }

    function generatedProjectDesign() {
        var name = projectName();
        var kind = projectKind();
        return {
            version: 1,
            generatedAt: new Date().toISOString(),
            source: "project-context",
            title: name,
            svg: '<svg viewBox="0 0 920 520" xmlns="http://www.w3.org/2000/svg" aria-label="Generated project design"><defs><linearGradient id="vibeGeneratedGradient" x1="0" x2="1"><stop stop-color="#dbeafe"/><stop offset="1" stop-color="#dcfce7"/></linearGradient></defs><rect x="36" y="40" width="848" height="440" rx="18" fill="url(#vibeGeneratedGradient)" stroke="#93c5fd" stroke-width="2"/><rect x="76" y="88" width="230" height="320" rx="12" fill="#fff" stroke="#60a5fa"/><rect x="336" y="88" width="508" height="74" rx="12" fill="#fff" stroke="#86efac"/><rect x="336" y="188" width="242" height="220" rx="12" fill="#fff" stroke="#c4b5fd"/><rect x="602" y="188" width="242" height="220" rx="12" fill="#fff" stroke="#fdba74"/><circle cx="116" cy="128" r="12" fill="#2563eb"/><text x="142" y="134" font-family="system-ui" font-size="18" font-weight="700" fill="#0f172a">' + esc(name) + '</text><text x="100" y="184" font-family="system-ui" font-size="13" fill="#475569">' + esc(kind) + '</text><path d="M306 248H336M578 298H602" stroke="#64748b" stroke-width="3" stroke-dasharray="7 7"/><text x="100" y="360" font-family="system-ui" font-size="12" fill="#64748b">Generated starting point</text></svg>'
        };
    }

    function sanitizeSvg(svgText) {
        if (!svgText) return "";
        var parsed = new DOMParser().parseFromString(String(svgText), "image/svg+xml");
        var root = parsed.documentElement;
        if (!root || root.nodeName.toLowerCase() !== "svg") return "";
        parsed.querySelectorAll("script, foreignObject").forEach(function (node) { node.remove(); });
        parsed.querySelectorAll("*").forEach(function (node) {
            Array.prototype.slice.call(node.attributes).forEach(function (attribute) {
                if (/^on/i.test(attribute.name)) node.removeAttribute(attribute.name);
                if ((attribute.name === "href" || attribute.name === "xlink:href") && /^javascript:/i.test(attribute.value)) {
                    node.removeAttribute(attribute.name);
                }
            });
        });
        return new XMLSerializer().serializeToString(root);
    }

    function generateProjectDesign(force) {
        var designApi = window.VibeDesign;
        if (!designApi || typeof designApi.getState !== "function") return null;
        var state = designApi.getState();
        if (!force && state.generated) return state.generated;
        var design = generatedProjectDesign();
        design.svg = sanitizeSvg(design.svg);
        if (typeof designApi.setGeneratedDesign === "function") designApi.setGeneratedDesign(design);
        if (typeof designApi.saveSoon === "function") designApi.saveSoon();
        if (window.VibeCanvasViews && typeof window.VibeCanvasViews.refresh === "function") window.VibeCanvasViews.refresh();
        return design;
    }

    function manualCount() {
        var designApi = window.VibeDesign;
        if (!designApi || typeof designApi.getState !== "function") return 0;
        var state = designApi.getState();
        return (state.elements || []).length + (state.connectors || []).length;
    }

    function generatedAt() {
        var designApi = window.VibeDesign;
        if (!designApi || typeof designApi.getState !== "function") return "not generated";
        var generated = designApi.getState().generated;
        return generated && generated.generatedAt ? generated.generatedAt : "not generated";
    }


    function decorateToolbar() {
        var toolbar = q("vibeCanvasToolbar");
        if (!toolbar) return;
        toolbar.querySelectorAll("button").forEach(function (button) {
            if (button.querySelector(".vibe-canvas-icon")) return;
            var key = button.getAttribute("data-vibe-tool") ||
                (button.hasAttribute("data-vibe-canvas-clear") ? "clear" :
                    button.hasAttribute("data-vibe-zoom-reset") ? "fit" : "");
            if (!key) return;
            var label = button.textContent.trim();
            button.textContent = "";
            var icon = document.createElement("span");
            icon.className = "vibe-canvas-icon";
            icon.innerHTML = toolbarIcon(key);
            button.appendChild(icon);
            var text = document.createElement("span");
            text.textContent = label;
            button.appendChild(text);
        });
    }

    function tabMarkup(id, label, icon) {
        return '<button type="button" class="vibe-project-tab' + (id === currentView ? " active" : "") +
            '" data-project-canvas-tab="' + id + '" aria-selected="' + (id === currentView ? "true" : "false") + '">' +
            '<span class="vibe-project-tab-icon">' + toolbarIcon(icon) + '</span><span>' + label + "</span></button>";
    }

    function createTabs() {
        var canvas = q("vibeCanvas");
        var toolbar = q("vibeCanvasToolbar");
        if (!canvas || !toolbar || q("vibeProjectCanvasTabs")) return;
        var tabs = document.createElement("div");
        tabs.id = "vibeProjectCanvasTabs";
        tabs.className = "vibe-project-canvas-tabs";
        tabs.setAttribute("role", "tablist");
        tabs.innerHTML = tabMarkup("design", "Design", "design") +
            tabMarkup("architecture", "Architecture", "architecture") +
            tabMarkup("diagrams", "Diagrams", "diagrams") +
            tabMarkup("metrics", "Metrics", "metrics") +
            tabMarkup("files", "Files", "files");
        toolbar.parentNode.insertBefore(tabs, toolbar);
        tabs.addEventListener("click", function (event) {
            var tab = event.target.closest("[data-project-canvas-tab]");
            if (!tab) return;
            showView(tab.getAttribute("data-project-canvas-tab"));
        });

        var generate = document.createElement("button");
        generate.type = "button";
        generate.className = "vibe-canvas-generate";
        generate.title = "Generate or regenerate the project design baseline";
        generate.innerHTML = toolbarIcon("generate") + "<span>Generate Design</span>";
        generate.addEventListener("click", function () {
            if (window.VibeCanvasViews && typeof window.VibeCanvasViews.generateProjectDesign === "function") {
                window.VibeCanvasViews.generateProjectDesign(true);
            }
        });
        tabs.appendChild(generate);

        ["vibeSteps", "vibeDesignSurface", "vibeCanvasEmpty"].forEach(function (id) {
            var node = q(id);
            if (node) node.setAttribute("data-project-canvas-view", "design");
        });
        createViewPanels(canvas);
    }

    function createViewPanels(canvas) {
        var host = document.createElement("div");
        host.id = "vibeProjectCanvasViews";
        host.className = "vibe-project-canvas-views";
        host.innerHTML =
            '<section class="vibe-project-view" data-project-canvas-view="architecture" role="tabpanel"></section>' +
            '<section class="vibe-project-view" data-project-canvas-view="diagrams" role="tabpanel"></section>' +
            '<section class="vibe-project-view" data-project-canvas-view="metrics" role="tabpanel"></section>' +
            '<section class="vibe-project-view" data-project-canvas-view="files" role="tabpanel"></section>';
        canvas.appendChild(host);
        renderArchitecture();
        renderDiagrams();
        renderMetrics();
        renderFiles();
    }

    function showView(view) {
        if (viewIds.indexOf(view) === -1) return;
        currentView = view;
        var viewsHost = q("vibeProjectCanvasViews");
        if (viewsHost) viewsHost.classList.toggle("is-active", view !== "design");
        document.querySelectorAll("[data-project-canvas-view]").forEach(function (node) {
            node.style.display = node.getAttribute("data-project-canvas-view") === view ? "" : "none";
        });
        document.querySelectorAll("[data-project-canvas-tab]").forEach(function (tab) {
            var active = tab.getAttribute("data-project-canvas-tab") === view;
            tab.classList.toggle("active", active);
            tab.setAttribute("aria-selected", active ? "true" : "false");
        });
        if (view === "architecture") renderArchitecture();
        if (view === "diagrams") renderDiagrams();
        if (view === "metrics") renderMetrics();
        if (view === "files") renderFiles();
    }

    function viewPanel(view) {
        return document.querySelector('[data-project-canvas-view="' + view + '"]');
    }

    function renderArchitecture() {
        var panel = viewPanel("architecture");
        if (!panel) return;
        panel.innerHTML =
            '<article class="vibe-a4-sheet">' +
            '<header><div><span class="vibe-sheet-kicker">PROJECT ARCHITECTURE</span><h2>' + esc(projectName()) + '</h2></div><span class="vibe-sheet-page">A4 · 01</span></header>' +
            '<div class="vibe-a4-rule"></div>' +
            '<div class="vibe-a4-meta"><span>Type <b>' + esc(projectKind()) + '</b></span><span>Workspace <b>' + esc(projectId() || "local") + '</b></span><span>Revision <b>working copy</b></span></div>' +
            '<h3>System outline</h3><p>This project is represented as a user-editable visual baseline. Generated elements can be regenerated independently from the hand-drawn layer.</p>' +
            '<div class="vibe-architecture-grid"><div><b>Interface</b><span>Browser and project canvas</span></div><div><b>Application</b><span>Project runtime and API surface</span></div><div><b>Data</b><span>Persistence, files and integrations</span></div><div><b>Operations</b><span>VM, terminal and deployment</span></div></div>' +
            '<footer><span>Prepared in Project Canvas</span><span>Generated baseline · manual layer preserved</span></footer>' +
            '</article>';
    }

    function renderDiagrams() {
        var panel = viewPanel("diagrams");
        if (!panel) return;
        panel.innerHTML =
            '<div class="vibe-view-heading"><div><span class="vibe-sheet-kicker">PROJECT DIAGRAMS</span><h2>Runtime map</h2></div><span class="vibe-view-note">Generated from project context</span></div>' +
            '<svg class="vibe-runtime-diagram" viewBox="0 0 920 420" role="img" aria-label="Project runtime diagram">' +
            '<defs><marker id="vibeDiagramArrow" markerWidth="8" markerHeight="8" refX="7" refY="4" orient="auto"><path d="M0 0L8 4L0 8z" fill="#2563eb"/></marker></defs>' +
            diagramNode(52, 158, 160, 82, "Browser", "Preview surface", "#2563eb") +
            diagramNode(276, 158, 160, 82, "Application", projectKind(), "#16a34a") +
            diagramNode(500, 76, 160, 82, "API", "Project services", "#d97706") +
            diagramNode(500, 240, 160, 82, "Data", "Files and state", "#7c3aed") +
            diagramNode(724, 158, 150, 82, "Runtime", "VM / deploy", "#0891b2") +
            '<path class="vibe-diagram-edge" d="M212 199H276M436 199H500M436 199L500 281M660 117L724 178M660 281L724 220" marker-end="url(#vibeDiagramArrow)"/>' +
            '</svg>';
    }

    function diagramNode(x, y, w, h, title, subtitle, color) {
        return '<g transform="translate(' + x + ' ' + y + ')"><rect width="' + w + '" height="' + h + '" rx="6" fill="#fff" stroke="' + color + '" stroke-width="2"/><circle cx="20" cy="22" r="6" fill="' + color + '"/><text x="36" y="27" class="vibe-diagram-title">' + esc(title) + '</text><text x="20" y="54" class="vibe-diagram-subtitle">' + esc(subtitle) + '</text></g>';
    }

    function renderMetrics() {
        var panel = viewPanel("metrics");
        if (!panel) return;
        var manual = window.VibeDesign && typeof window.VibeDesign.manualCount === "function"
            ? window.VibeDesign.manualCount()
            : 0;
        var generated = window.VibeDesign && typeof window.VibeDesign.generatedAt === "function"
            ? window.VibeDesign.generatedAt()
            : "not generated";
        panel.innerHTML =
            '<div class="vibe-view-heading"><div><span class="vibe-sheet-kicker">PROJECT METRICS</span><h2>Canvas signals</h2></div><span class="vibe-view-note">Visual project telemetry</span></div>' +
            '<div class="vibe-metric-cards"><div><b>' + manual + '</b><span>Manual elements</span></div><div><b>5</b><span>Runtime zones</span></div><div><b>1</b><span>Generated baseline</span></div></div>' +
            '<svg class="vibe-metric-chart" viewBox="0 0 760 280" role="img" aria-label="Project metrics chart"><path d="M55 25V230H730" class="vibe-chart-axis"/><path d="M75 190L190 160L305 175L420 105L535 125L650 68L720 85" class="vibe-chart-line"/><path d="M75 190L190 160L305 175L420 105L535 125L650 68L720 85V230H75z" class="vibe-chart-area"/><text x="55" y="255">baseline</text><text x="620" y="255">current</text></svg>' +
            '<p class="vibe-view-footnote">Generated at ' + esc(generated) + '. Manual drawings remain outside this generated signal.</p>';
    }

    function renderFiles() {
        var panel = viewPanel("files");
        if (!panel) return;
        panel.innerHTML =
            '<div class="vibe-view-heading"><div><span class="vibe-sheet-kicker">PROJECT FILES</span><h2>Workspace surface</h2></div><span class="vibe-view-note">Read-only overview</span></div>' +
            '<div class="vibe-file-list"><div><span class="vibe-file-icon">' + toolbarIcon("files") + '</span><b>Project source</b><span>Editable through Code Editor</span></div><div><span class="vibe-file-icon">' + toolbarIcon("architecture") + '</span><b>Architecture baseline</b><span>Generated in this canvas</span></div><div><span class="vibe-file-icon">' + toolbarIcon("diagrams") + '</span><b>Runtime diagram</b><span>Derived project view</span></div></div>';
    }

    function init() {
        if (!q("vibeCanvas")) return;
        decorateToolbar();
        createTabs();
        showView("design");
        if (window.VibeDesign && typeof window.VibeDesign.load === "function") window.VibeDesign.load();
        document.addEventListener("gb:vibe-project", function () {
            renderArchitecture();
            renderDiagrams();
            renderMetrics();
            renderFiles();
        });
    }

    window.VibeCanvasViews = {
        init: init,
        show: showView,
        generateProjectDesign: generateProjectDesign,
        manualCount: manualCount,
        generatedAt: generatedAt,
        refresh: function () {
            renderArchitecture();
            renderDiagrams();
            renderMetrics();
            renderFiles();
        },
    };

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", init);
    } else {
        init();
    }
})();

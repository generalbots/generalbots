/* Vibe Knowledge Graph — split from vibe/vibe-graph.js for #1731 */

/**
 * Vibe Knowledge Graph visualization (Issues #522, #806, #1191)
 * Displays the knowledge graph from /api/vibe/graph/{use_case} on an
 * interactive Canvas with PAN + ZOOM, CLICK-for-details on every node and a
 * guaranteed non-empty baseline: when a project has no runs yet, the graph is
 * seeded from the runtime capabilities so it always explains something useful.
 *
 * Guard against double-declaration: the desktop window manager re-injects
 * this script every time the Vibe app opens (HTMX body swap), so window-
 * singleton assignment keeps one instance; init() re-binds from fresh DOM.
 */
Object.assign((window.VibeGraph = window.VibeGraph || {}), {
    canvas: null,
    ctx: null,
    nodes: [],
    edges: [],
    animationId: null,
    isDragging: false,
    dragNode: null,
    useCase: 'software_development',
    // Pan/zoom viewport: screen = world * k + offset
    view: { x: 0, y: 0, k: 1 },
    panning: null,
    selectedId: null,
    seeded: false,

    init: function (canvasId) {
        this.canvas = document.getElementById(canvasId);
        if (!this.canvas) return;
        this.ctx = this.canvas.getContext('2d');
        this.view = { x: 0, y: 0, k: 1 };
        this.setupEvents();
        this.wireToggle();
    },

    wireToggle: function () {
        const toggle = document.getElementById('vibeGraphToggle');
        const close = document.getElementById('vibeGraphClose');
        if (toggle) toggle.addEventListener('click', () => this.togglePanel());
        if (close) close.addEventListener('click', () => this.togglePanel(false));
    },

    togglePanel: function (show) {
        const panel = document.getElementById('vibeGraphPanel');
        if (!panel) return;
        const visible = show === undefined ? panel.style.display === 'none' : show;
        panel.style.display = visible ? 'flex' : 'none';
        if (visible) {
            this.resize();
            const useCase = window.VibeGraphUseCase || this.useCase;
            this.useCase = useCase;
            this.loadCapabilities(useCase).then(() => this.loadGraph(useCase));
            this.loadContext();
        }
    },

    /* Context rail: the graph is more useful when it also shows WHAT the
       user asked (key points), the TODO plan and the project's AGENTS.md
       directives — the same context the agent works from. */
    loadContext: function () {
        const rail = document.getElementById('vibeGraphContext');
        if (!rail) return;
        this.loadAsks(rail);
        this.loadTodos(rail);
        this.loadDirectives(rail);
    },

    loadAsks: function (rail) {
        const box = document.getElementById('vibeGraphAsks');
        if (!box) return;
        vibeAuthFetch('/api/vibe/runs?limit=8').then((r) => r.json()).then((data) => {
            const runs = Array.isArray(data) ? data : [];
            if (!runs.length) {
                box.innerHTML = '<div style="opacity:.6">No asks yet — press Run.</div>';
                return;
            }
            box.innerHTML = runs.slice(0, 8).map((run) => {
                const st = String(run.state || '').toLowerCase();
                const dot = st === 'completed' ? '#22c55e' : st === 'failed' || st === 'cancelled' ? '#ef4444' : '#3b82f6';
                return '<div style="display:flex;gap:7px;align-items:flex-start;padding:4px 0;border-bottom:1px solid rgba(128,128,128,.15);">' +
                    '<span style="width:8px;height:8px;border-radius:50%;background:' + dot + ';margin-top:4px;flex:0 0 auto;"></span>' +
                    '<span style="word-break:break-word;">' + this.esc(String(run.intent || 'run').substring(0, 120)) + '</span></div>';
            }).join('');
        }).catch(() => { box.innerHTML = '<div style="opacity:.6">Asks unavailable.</div>'; });
    },

    loadTodos: function (rail) {
        const box = document.getElementById('vibeGraphTodos');
        if (!box) return;
        const useCase = window.VibeGraphUseCase || this.useCase;
        vibeAuthFetch('/api/vibe/pipeline/' + useCase).then((r) => r.json()).then((data) => {
            const stages = (data && data.pipeline && data.pipeline.stages) || [];
            if (!stages.length) {
                box.innerHTML = '<div style="opacity:.6">No plan yet.</div>';
                return;
            }
            box.innerHTML = stages.map((st) =>
                '<div style="display:flex;gap:7px;align-items:flex-start;padding:3px 0;">' +
                '<span style="opacity:.75">○</span><span>' + this.esc(String(st.name || st.id)) + '</span></div>'
            ).join('');
        }).catch(() => { box.innerHTML = '<div style="opacity:.6">Plan unavailable.</div>'; });
    },

    loadDirectives: function (rail) {
        const box = document.getElementById('vibeGraphDirectives');
        if (!box) return;
        const pid = (typeof currentProjectId !== 'undefined' && currentProjectId) ? currentProjectId : null;
        if (!pid) {
            box.innerHTML = '<div style="opacity:.6">Select a project to see its AGENTS.md.</div>';
            return;
        }
        vibeAuthFetch('/api/vibe/projects/' + encodeURIComponent(pid) + '/files')
            .then((r) => r.json())
            .then((data) => {
                const files = (data && data.files) || [];
                const md = files.find((f) => /(^|\/)AGENTS\.md$/i.test(String(f)));
                if (!md) {
                    box.innerHTML = '<div style="opacity:.6">No AGENTS.md in this project.</div>';
                    return;
                }
                return vibeAuthFetch('/api/vibe/projects/' + encodeURIComponent(pid) + '/files/content?path=' + encodeURIComponent(md))
                    .then((r) => r.json())
                    .then((d) => {
                        const text = String(d.content || '');
                        const lines = text.split('\n').slice(0, 40);
                        box.innerHTML = '<div style="white-space:pre-wrap;word-break:break-word;font-size:11px;line-height:1.5;opacity:.92;">' +
                            this.esc(lines.join('\n')) + (text.split('\n').length > 40 ? '\n…' : '') + '</div>';
                    });
            })
            .catch(() => { box.innerHTML = '<div style="opacity:.6">Directives unavailable.</div>'; });
    },

    setupEvents: function () {
        this.canvas.addEventListener('mousedown', (e) => this.onMouseDown(e));
        this.canvas.addEventListener('mousemove', (e) => this.onMouseMove(e));
        window.addEventListener('mouseup', () => { this.isDragging = false; this.dragNode = null; this.panning = null; });
        this.canvas.addEventListener('dblclick', (e) => this.onDoubleClick(e));
        // Wheel = zoom anchored at cursor (world point under cursor stays put).
        this.canvas.addEventListener('wheel', (e) => {
            e.preventDefault();
            const rect = this.canvas.getBoundingClientRect();
            const mx = e.clientX - rect.left;
            const my = e.clientY - rect.top;
            const factor = e.deltaY < 0 ? 1.15 : 1 / 1.15;
            this.zoomAt(mx, my, factor);
        }, { passive: false });
        window.addEventListener('resize', () => this.resize());
        this.buildChrome();
        this.resize();
    },

    // Small in-canvas control strip: zoom in/out, fit, reload.
    buildChrome: function () {
        const parent = this.canvas && this.canvas.parentElement;
        if (!parent || parent.querySelector('.vg-chrome')) return;
        const bar = document.createElement('div');
        bar.className = 'vg-chrome';
        bar.style.cssText = 'position:absolute;top:8px;right:10px;display:flex;gap:4px;z-index:4;';
        const mk = (label, title, fn) => {
            const b = document.createElement('button');
            b.type = 'button'; b.title = title; b.textContent = label;
            b.style.cssText = 'background:rgba(20,20,46,.85);border:1px solid rgba(255,255,255,.25);color:#fff;border-radius:6px;padding:2px 9px;font-size:12px;cursor:pointer;';
            b.addEventListener('click', fn);
            return b;
        };
        bar.appendChild(mk('＋', 'Zoom in', () => {
            const r = this.canvas.getBoundingClientRect();
            this.zoomAt(r.width / 2, r.height / 2, 1.25);
        }));
        bar.appendChild(mk('－', 'Zoom out', () => {
            const r = this.canvas.getBoundingClientRect();
            this.zoomAt(r.width / 2, r.height / 2, 1 / 1.25);
        }));
        bar.appendChild(mk('⤢', 'Fit view', () => { this.fitView(); }));
        bar.appendChild(mk('⟳', 'Reload graph', () => { this.loadGraph(this.useCase); }));
        parent.appendChild(bar);

        const status = document.createElement('div');
        status.className = 'vg-status';
        status.id = 'vibeGraphStatus';
        status.style.cssText = 'position:absolute;left:10px;top:8px;font-size:10px;color:var(--text-muted,#94a3b8);z-index:4;background:rgba(0,0,0,.25);padding:2px 8px;border-radius:8px;pointer-events:none;';
        status.textContent = 'scroll = zoom · drag background = pan · click node = details';
        parent.appendChild(status);
    },

    zoomAt: function (mx, my, factor) {
        const k = Math.min(3, Math.max(0.3, this.view.k * factor));
        const realFactor = k / this.view.k;
        this.view.x = mx - (mx - this.view.x) * realFactor;
        this.view.y = my - (my - this.view.y) * realFactor;
        this.view.k = k;
        this.render();
        this.updateStatus();
    },

    updateStatus: function () {
        const el = document.getElementById('vibeGraphStatus');
        if (!el) return;
        const base = this.seeded ? 'baseline (no runs yet)' : 'live';
        el.textContent = `${base} · ${Math.round(this.view.k * 100)}% · scroll=zoom · drag bg=pan · click node`;
    },

    resize: function () {
        if (!this.canvas) return;
        const rect = this.canvas.parentElement.getBoundingClientRect();
        this.canvas.width = Math.max(1, rect.width);
        this.canvas.height = Math.max(1, rect.height);
        this.fitView();
        this.render();
    },
});

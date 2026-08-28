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
window.VibeGraph = {
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
                const dot = st === 'completed' ? '#22c55e' : st === 'failed' || st === 'cancelled' ? '#ef4444' : st === 'awaiting_approval' ? '#f7b500' : '#3b82f6';
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

    loadGraph: async function (useCase) {
        try {
            const label = document.getElementById('vibeGraphUseCaseLabel');
            if (label) label.textContent = useCase;
            let url = `/api/vibe/graph/${useCase}`;
            const pid = (typeof currentProjectId !== 'undefined' && currentProjectId) ? currentProjectId : null;
            if (pid) url += '?project_id=' + encodeURIComponent(pid);
            const resp = await vibeAuthFetch(url);
            const data = await resp.json();
            this.seeded = false;
            let rawNodes = (data.success && data.graph && data.graph.nodes) || [];
            let rawEdges = (data.success && data.graph && data.graph.edges) || [];
            // Never-empty guarantee: seed the baseline from runtime
            // capabilities when the backend has only sparse data (fresh
            // project without runs) so the graph always shows structure.
            const meaningful = rawNodes.filter(n => n.node_type !== 'use_case');
            if (meaningful.length === 0 && (this.capabilities || []).length) {
                const ucId = 'uc:' + useCase;
                const seededTools = (this.capabilities || []).map((c) => ({
                    id: 'tool-seed:' + c.id,
                    label: c.title,
                    node_type: 'tool',
                    properties: { tools: c.tools, description: c.description, baseline: true },
                }));
                const seeds = [{ id: ucId, label: useCase, node_type: 'use_case', properties: {} }].concat(seededTools);
                rawNodes = seeds;
                rawEdges = seededTools.map(t => ({
                    source: ucId,
                    target: t.id,
                    relationship: 'provides',
                    weight: 1,
                }));
                this.seeded = true;
            }
            this.rawEdges = rawEdges;
            this.nodes = rawNodes.map((n) => ({ ...n }));
            this.selectedId = null;
            this.renderDetails(null);
            this.layoutGraph();
            this.fitView(true);
            this.render();
            this.updateStatus();
        } catch (e) {
            console.error('Failed to load graph:', e);
            this.updateStatus('load failed');
        }
    },

    loadCapabilities: async function (useCase) {
        try {
            const resp = await vibeAuthFetch(`/api/vibe/capabilities/${useCase}`);
            const data = await resp.json();
            if (data.success) {
                this.capabilities = data.capabilities || [];
                this.capabilities.forEach(c => { c.tools = c.tools || []; });
                const target = document.getElementById('vibeCapabilities');
                if (target) {
                    target.innerHTML = this.capabilities.map((c) => {
                        const toolChips = c.tools.map(t => `<span class="vibe-cap-tool">${t}</span>`).join('');
                        const approval = c.requires_approval ? ' <span class="vibe-cap-warn">approval</span>' : '';
                        return `<div class="vibe-capability">
                            <div class="vibe-cap-title">${c.title} <small>${c.id}</small>${approval}</div>
                            <div class="vibe-cap-desc">${c.description}</div>
                            <div class="vibe-cap-tools">${toolChips}</div>
                        </div>`;
                    }).join('');
                }
            }
        } catch (e) {
            console.error('Failed to load capabilities:', e);
        }
    },

    nodeColor: function (node) {
        if (node.node_type === 'use_case') return '#2563eb';
        if (node.node_type === 'tool') return '#d97706';
        const state = node.properties && node.properties.state;
        if (state === 'failed') return '#f77';
        if (state === 'awaiting_approval') return '#f7b500';
        if (state === 'completed') return '#059669';
        return '#7c3aed';
    },

    measureNode: function (node) {
        this.ctx.font = '600 11px sans-serif';
        const textW = Math.min(node.width - 20, this.ctx.measureText(this.nodeLabel(node)).width);
        const typeText = node.node_type.replace('_', ' ').toUpperCase();
        this.ctx.font = '9px monospace';
        const typeW = this.ctx.measureText(typeText).width;
        return Math.ceil(Math.max(textW, typeW)) + 24;
    },

    layoutGraph: function () {
        if (!this.canvas || !this.nodes.length) return;
        const lanes = ['use_case', 'run', 'tool'];
        lanes.forEach((type) => {
            const laneNodes = this.nodes.filter(n => n.node_type === type);
            laneNodes.forEach((node, index) => {
                node.width = node.width || 160;
                node.height = 54;
            });
        });
        // Two-phase: size by content first, then place per lane column so
        // boxes do not overlap horizontally at any zoom level.
        const margin = 120;
        const usable = Math.max(240, this.canvas.width - margin * 2);
        const activeLanes = lanes.filter(type => this.nodes.some(n => n.node_type === type));
        const laneCount = Math.max(1, activeLanes.length);
        activeLanes.forEach((type, laneIndex) => {
            const laneNodes = this.nodes.filter(n => n.node_type === type);
            const x = margin + (laneCount > 1 ? laneIndex * (usable / (laneCount - 1)) : usable / 2);
            const gap = this.canvas.height / (laneNodes.length + 1);
            laneNodes.forEach((node, index) => {
                node.x = x;
                node.y = Math.max(40, gap * (index + 1));
                node.width = Math.max(70, this.measureNode(node));
            });
        });
    },

    // Fit all nodes into the visible canvas (reset & center the viewport).
    fitView: function (initial) {
        if (!this.canvas || !this.nodes.length) {
            this.view = { x: initial ? this.canvas.width * 0.1 : this.view.x, y: this.view.y, k: initial ? 1 : this.view.k };
            return;
        }
        let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
        this.nodes.forEach(n => {
            minX = Math.min(minX, n.x - n.width / 2);
            maxX = Math.max(maxX, n.x + n.width / 2);
            minY = Math.min(minY, n.y - n.height / 2);
            maxY = Math.max(maxY, n.y + n.height / 2);
        });
        const pad = 36;
        const w = this.canvas.width, h = this.canvas.height;
        const k = Math.min(3, Math.max(0.3, Math.min(
            (w - pad * 2) / Math.max(60, maxX - minX),
            (h - pad * 2) / Math.max(60, maxY - minY),
            1.4
        )));
        const cx = (minX + maxX) / 2, cy = (minY + maxY) / 2;
        this.view = { k: k, x: w / 2 - cx * k, y: h / 2 - cy * k };
        this.updateStatus();
    },

    render: function () {
        const ctx = this.ctx;
        if (!ctx) return;
        ctx.setTransform(1, 0, 0, 1, 0, 0);
        ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
        ctx.setTransform(this.view.k, 0, 0, this.view.k, this.view.x, this.view.y);

        for (const edge of (this.edgesForRender())) {
            const source = this.nodes.find(n => n.id === edge.source);
            const target = this.nodes.find(n => n.id === edge.target);
            if (!source || !target) continue;
            ctx.beginPath();
            ctx.moveTo(source.x + source.width / 2, source.y);
            ctx.lineTo(target.x - target.width / 2, target.y);
            ctx.strokeStyle = edge.relationship === 'triggered' ? 'rgba(217, 119, 6, 0.45)' : 'rgba(37, 99, 235, 0.4)';
            ctx.lineWidth = Math.max(1, (edge.weight || 1) * 1.5);
            ctx.stroke();
        }

        for (const node of this.nodes) {
            const x = node.x - node.width / 2;
            const y = node.y - node.height / 2;
            const selected = node.id === this.selectedId;
            ctx.beginPath();
            ctx.roundRect(x, y, node.width, node.height, 7);
            ctx.fillStyle = this.nodeColor(node);
            ctx.fill();
            ctx.strokeStyle = selected ? '#ffffff' : 'rgba(255,255,255,.75)';
            ctx.lineWidth = selected ? 2.5 : 1;
            ctx.stroke();

            ctx.fillStyle = '#fff';
            ctx.font = '600 11px sans-serif';
            ctx.textAlign = 'left';
            ctx.fillText(this.nodeLabel(node), x + 10, y + 22, node.width - 16);
            ctx.fillStyle = 'rgba(255,255,255,.82)';
            ctx.font = '9px monospace';
            ctx.fillText(node.node_type.replace('_', ' ').toUpperCase(), x + 10, y + 40);
        }
        ctx.setTransform(1, 0, 0, 1, 0, 0);
    },

    edgesForRender: function () { return this.rawEdges || []; },

    nodeLabel: function (node) {
        if (node.node_type === 'tool') {
            const calls = node.properties && node.properties.calls ? ` (${node.properties.calls})` : '';
            return String(node.label).substring(0, 22) + calls;
        }
        if (node.node_type === 'run') {
            const state = node.properties && node.properties.state ? ` [${node.properties.state}]` : '';
            return String(node.label).substring(0, 18) + state;
        }
        return String(node.label).substring(0, 26);
    },

    /* ------------------------- interaction ------------------------- */

    toWorld: function (mx, my) {
        return { x: (mx - this.view.x) / this.view.k, y: (my - this.view.y) / this.view.k };
    },

    hitNode: function (node, wx, wy) {
        return wx >= node.x - node.width / 2 && wx <= node.x + node.width / 2 &&
            wy >= node.y - node.height / 2 && wy <= node.y + node.height / 2;
    },

    onMouseDown: function (e) {
        const rect = this.canvas.getBoundingClientRect();
        const mx = e.clientX - rect.left;
        const my = e.clientY - rect.top;
        const w = this.toWorld(mx, my);
        const node = this.nodes.find(n => this.hitNode(n, w.x, w.y)) || null;
        if (node) {
            this.dragNode = node;
            this.isDragging = true;
            this.selectNode(node);
        } else {
            // Background: start panning (and clear selection like any IDE).
            this.panning = { sx: mx, sy: my, ox: this.view.x, oy: this.view.y };
            this.selectNode(null);
        }
    },

    onMouseMove: function (e) {
        const rect = this.canvas.getBoundingClientRect();
        const mx = e.clientX - rect.left;
        const my = e.clientY - rect.top;
        if (this.panning) {
            this.view.x = this.panning.ox + (mx - this.panning.sx);
            this.view.y = this.panning.oy + (my - this.panning.sy);
            this.render();
            return;
        }
        if (this.isDragging && this.dragNode) {
            const w = this.toWorld(mx, my);
            this.dragNode.x = w.x;
            this.dragNode.y = w.y;
            this.render();
        } else {
            const w = this.toWorld(mx, my);
            this.canvas.style.cursor = this.nodes.find(n => this.hitNode(n, w.x, w.y)) ? 'pointer' : 'grab';
        }
    },

    selectNode: function (node) {
        this.selectedId = node ? node.id : null;
        this.renderDetails(node);
        this.render();
    },

    // Click-a-node details card docked inside the graph panel.
    renderDetails: function (node) {
        let box = document.getElementById('vibeGraphDetails');
        if (!node) { if (box) box.remove(); return; }
        if (!box) {
            box = document.createElement('div');
            box.id = 'vibeGraphDetails';
            box.style.cssText = [
                'position:absolute', 'left:10px', 'bottom:10px', 'width:min(320px,80%)',
                'max-height:44%', 'overflow:auto', 'z-index:5',
                'background:rgba(13,17,35,.95)', 'border:1px solid rgba(255,255,255,.18)',
                'border-radius:8px', 'color:#e7ecff', 'font-size:11px', 'line-height:1.45',
                'padding:10px 12px', 'box-shadow:0 10px 30px rgba(0,0,0,.45)',
            ].join(';');
            const parent = this.canvas && this.canvas.parentElement;
            if (!parent) return;
            parent.appendChild(box);
        }
        const props = Object.assign({}, node.properties || {});
        if (node.id) props.id = node.id;
        const rows = Object.keys(props).filter(k => props[k] !== undefined && props[k] !== null)
            .map(k => `<tr><td style="opacity:.65;padding-right:8px;white-space:nowrap;">${String(k)}</td>` +
                `<td style="word-break:break-word;">${this.esc(String(typeof props[k] === 'object' ? JSON.stringify(props[k]) : props[k]))}</td></tr>`)
            .join('');
        box.innerHTML =
            `<div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:6px;">
                <b style="font-size:12px;">${this.esc(String(node.label || node.id))}</b>
                <span style="cursor:pointer;opacity:.7;" id="vibeGraphDetailsClose">✕</span>
             </div>
             <div style="margin-bottom:6px;"><span style="background:${this.nodeColor(node)};border-radius:4px;padding:1px 7px;color:#fff;font-size:10px;">${this.esc(node.node_type)}</span></div>
             <table style="border-collapse:collapse;width:100%;">${rows}</table>`;
        const closeBtn = box.querySelector('#vibeGraphDetailsClose');
        if (closeBtn) closeBtn.addEventListener('click', () => this.selectNode(null));
    },

    esc: function (s) {
        return String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
    },

    onDoubleClick: function (e) {
        const rect = this.canvas.getBoundingClientRect();
        const mx = e.clientX - rect.left;
        const my = e.clientY - rect.top;
        const w = this.toWorld(mx, my);
        const node = this.nodes.find(n => this.hitNode(n, w.x, w.y));
        if (node && node.node_type === 'run') {
            this.loadRunGraph(node.id.replace(/^run:/, ''));
        }
    },

    loadRunGraph: async function (runId) {
        try {
            const resp = await vibeAuthFetch(`/api/vibe/graph/run/${runId}`);
            const data = await resp.json();
            if (data.success && data.graph) {
                this.rawEdges = data.graph.edges;
                this.nodes = data.graph.nodes.map((n) => ({ ...n }));
                this.selectedId = null;
                this.renderDetails(null);
                this.layoutGraph();
                this.fitView(true);
                this.render();
            }
        } catch (e) {
            console.error('Failed to load run graph:', e);
        }
    },
};

function initVibeGraph() {
    VibeGraph.init('vibeGraphCanvas');
}

if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initVibeGraph);
} else {
    initVibeGraph();
}

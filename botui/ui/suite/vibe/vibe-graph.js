/**
 * Vibe Knowledge Graph visualization (Issues #522, #806)
 * Displays the knowledge graph from /api/vibe/graph/{use_case}
 * as an interactive force-directed graph using Canvas, plus the
 * runtime capability list from /api/vibe/capabilities/:use_case.
 */
// Guard against double-declaration: the desktop window manager re-injects
// this script every time the Vibe app opens (HTMX body swap), so a top-level
// `const` threw 'Identifier VibeGraph has already been declared' on the
// second open. Assigning to window keeps a single singleton; init() re-binds
// the canvas from the fresh DOM on each load.
window.VibeGraph = {
    canvas: null,
    ctx: null,
    nodes: [],
    edges: [],
    animationId: null,
    isDragging: false,
    dragNode: null,
    useCase: 'software_development',

    init: function(canvasId) {
        this.canvas = document.getElementById(canvasId);
        if (!this.canvas) return;
        this.ctx = this.canvas.getContext('2d');
        this.setupEvents();
        this.wireToggle();
    },

    wireToggle: function() {
        const toggle = document.getElementById('vibeGraphToggle');
        const close = document.getElementById('vibeGraphClose');
        if (toggle) toggle.addEventListener('click', () => this.togglePanel());
        if (close) close.addEventListener('click', () => this.togglePanel(false));
    },

    togglePanel: function(show) {
        const panel = document.getElementById('vibeGraphPanel');
        if (!panel) return;
        const visible = show === undefined ? panel.style.display === 'none' : show;
        panel.style.display = visible ? 'flex' : 'none';
        if (visible) {
            this.resize();
            const useCase = window.VibeGraphUseCase || this.useCase;
            this.useCase = useCase;
            this.loadGraph(useCase);
            this.loadCapabilities(useCase);
        }
    },

    setupEvents: function() {
        this.canvas.addEventListener('mousedown', (e) => this.onMouseDown(e));
        this.canvas.addEventListener('mousemove', (e) => this.onMouseMove(e));
        this.canvas.addEventListener('mouseup', () => this.onMouseUp());
        this.canvas.addEventListener('dblclick', (e) => this.onDoubleClick(e));
        window.addEventListener('resize', () => this.resize());
        this.resize();
    },

    resize: function() {
        if (!this.canvas) return;
        const rect = this.canvas.parentElement.getBoundingClientRect();
        this.canvas.width = Math.max(1, rect.width);
        this.canvas.height = Math.max(1, rect.height);
        this.layoutGraph();
        this.render();
    },

    loadGraph: async function(useCase) {
        try {
            const label = document.getElementById('vibeGraphUseCaseLabel');
            if (label) label.textContent = useCase;
            const resp = await vibeAuthFetch(`/api/vibe/graph/${useCase}`);
            const data = await resp.json();
            if (data.success && data.graph) {
                const w = Math.max(1, this.canvas.width);
                const h = Math.max(1, this.canvas.height);
                this.nodes = data.graph.nodes.map((n) => ({
                    ...n,
                    x: w / 2, y: h / 2,
                    width: n.node_type === 'use_case' ? 180 : 160,
                    height: 54,
                }));
                this.edges = data.graph.edges;
                this.layoutGraph();
                this.render();
            }
        } catch (e) {
            console.error('Failed to load graph:', e);
        }
    },

    loadCapabilities: async function(useCase) {
        const target = document.getElementById('vibeCapabilities');
        if (!target) return;
        try {
            const resp = await vibeAuthFetch(`/api/vibe/capabilities/${useCase}`);
            const data = await resp.json();
            if (!data.success) return;
            target.innerHTML = data.capabilities
                .map(c => {
                    const toolChips = c.tools.map(t => `<span class="vibe-cap-tool">${t}</span>`).join('');
                    const approval = c.requires_approval ? ' <span class="vibe-cap-warn">approval</span>' : '';
                    return `<div class="vibe-capability">
                        <div class="vibe-cap-title">${c.title} <small>${c.id}</small>${approval}</div>
                        <div class="vibe-cap-desc">${c.description}</div>
                        <div class="vibe-cap-tools">${toolChips}</div>
                    </div>`;
                })
                .join('');
        } catch (e) {
            console.error('Failed to load capabilities:', e);
        }
    },

    startSimulation: function() {
        this.layoutGraph();
        this.render();
    },

    nodeColor: function(node) {
        if (node.node_type === 'use_case') return '#2563eb';
        if (node.node_type === 'tool') return '#d97706';
        const state = node.properties && node.properties.state;
        if (state === 'failed') return '#f77';
        if (state === 'awaiting_approval') return '#f7b500';
        if (state === 'completed') return '#059669';
        return '#7c3aed';
    },

    nodeLabel: function(node) {
        if (node.node_type === 'tool') {
            const calls = node.properties && node.properties.calls ? ` (${node.properties.calls})` : '';
            return node.label.substring(0, 15) + calls;
        }
        if (node.node_type === 'run') {
            const state = node.properties && node.properties.state ? ` [${node.properties.state}]` : '';
            return node.label.substring(0, 15) + state;
        }
        return node.label.substring(0, 20);
    },

    layoutGraph: function() {
        if (!this.canvas || !this.nodes.length) return;
        const lanes = ['use_case', 'run', 'tool'];
        const margin = 110;
        const usable = Math.max(240, this.canvas.width - margin * 2);
        lanes.forEach((type, laneIndex) => {
            const laneNodes = this.nodes.filter(n => n.node_type === type);
            const x = margin + laneIndex * (usable / Math.max(1, lanes.length - 1));
            const gap = this.canvas.height / (laneNodes.length + 1);
            laneNodes.forEach((node, index) => {
                node.x = x;
                node.y = Math.max(38, gap * (index + 1));
            });
        });
    },

    render: function() {
        const ctx = this.ctx;
        ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);

        for (const edge of this.edges) {
            const source = this.nodes.find(n => n.id === edge.source);
            const target = this.nodes.find(n => n.id === edge.target);
            if (!source || !target) continue;
            ctx.beginPath();
            ctx.moveTo(source.x + source.width / 2, source.y);
            ctx.lineTo(target.x - target.width / 2, target.y);
            ctx.strokeStyle = edge.relationship === 'triggered' ? 'rgba(217, 119, 6, 0.45)' : 'rgba(37, 99, 235, 0.4)';
            ctx.lineWidth = Math.max(1, edge.weight * 3 || 1);
            ctx.stroke();
        }

        for (const node of this.nodes) {
            const x = node.x - node.width / 2;
            const y = node.y - node.height / 2;
            ctx.beginPath();
            ctx.roundRect(x, y, node.width, node.height, 7);
            ctx.fillStyle = this.nodeColor(node);
            ctx.fill();
            ctx.strokeStyle = 'rgba(255,255,255,.75)';
            ctx.lineWidth = 1;
            ctx.stroke();

            ctx.fillStyle = '#fff';
            ctx.font = '600 11px sans-serif';
            ctx.textAlign = 'left';
            ctx.textBaseline = 'alphabetic';
            ctx.fillText(this.nodeLabel(node), x + 10, y + 22, node.width - 20);
            ctx.fillStyle = 'rgba(255,255,255,.82)';
            ctx.font = '9px monospace';
            ctx.fillText(node.node_type.replace('_', ' ').toUpperCase(), x + 10, y + 40);
        }
    },

    onMouseDown: function(e) {
        const rect = this.canvas.getBoundingClientRect();
        const mx = e.clientX - rect.left;
        const my = e.clientY - rect.top;
        this.dragNode = this.nodes.find(n => this.hitNode(n, mx, my)) || null;
        if (this.dragNode) this.isDragging = true;
    },

    onMouseMove: function(e) {
        if (!this.isDragging || !this.dragNode) return;
        const rect = this.canvas.getBoundingClientRect();
        this.dragNode.x = e.clientX - rect.left;
        this.dragNode.y = e.clientY - rect.top;
        this.render();
    },

    onMouseUp: function() {
        this.isDragging = false;
        this.dragNode = null;
    },

    hitNode: function(node, x, y) {
        return x >= node.x - node.width / 2 && x <= node.x + node.width / 2 &&
            y >= node.y - node.height / 2 && y <= node.y + node.height / 2;
    },

    onDoubleClick: function(e) {
        const rect = this.canvas.getBoundingClientRect();
        const mx = e.clientX - rect.left;
        const my = e.clientY - rect.top;
        const node = this.nodes.find(n => this.hitNode(n, mx, my));
        if (node && node.node_type === 'run') {
            this.loadRunGraph(node.id.replace(/^run:/, ''));
        }
    },

    loadRunGraph: async function(runId) {
        try {
            const resp = await vibeAuthFetch(`/api/vibe/graph/run/${runId}`);
            const data = await resp.json();
            if (data.success && data.graph) {
                this.nodes = data.graph.nodes.map((n) => ({
                    ...n,
                    x: 0,
                    y: this.canvas.height / 2,
                    width: n.node_type === 'run' ? 180 : 160,
                    height: 54,
                }));
                this.edges = data.graph.edges;
                this.useCase = data.graph.nodes.length
                    ? (window.VibeGraphUseCase || this.useCase)
                    : this.useCase;
                this.layoutGraph();
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

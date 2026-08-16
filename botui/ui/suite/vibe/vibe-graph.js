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
window.VibeGraph = window.VibeGraph || {
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
    },

    loadGraph: async function(useCase) {
        try {
            const label = document.getElementById('vibeGraphUseCaseLabel');
            if (label) label.textContent = useCase;
            const resp = await fetch(`/api/vibe/graph/${useCase}`);
            const data = await resp.json();
            if (data.success && data.graph) {
                const w = Math.max(1, this.canvas.width);
                const h = Math.max(1, this.canvas.height);
                this.nodes = data.graph.nodes.map((n, i) => ({
                    ...n,
                    x: 40 + (i % 6) * (w / 6),
                    y: 40 + Math.floor(i / 6) * (h / Math.max(1, Math.ceil(data.graph.nodes.length / 6))),
                    vx: 0, vy: 0,
                    radius: n.node_type === 'use_case' ? 30 : n.node_type === 'run' ? 18 : 14,
                }));
                this.edges = data.graph.edges;
                this.startSimulation();
            }
        } catch (e) {
            console.error('Failed to load graph:', e);
        }
    },

    loadCapabilities: async function(useCase) {
        const target = document.getElementById('vibeCapabilities');
        if (!target) return;
        try {
            const resp = await fetch(`/api/vibe/capabilities/${useCase}`);
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
        if (this.animationId) cancelAnimationFrame(this.animationId);
        const simulate = () => {
            this.updatePhysics();
            this.render();
            this.animationId = requestAnimationFrame(simulate);
        };
        simulate();
    },

    nodeColor: function(node) {
        if (node.node_type === 'use_case') return '#84d669';
        if (node.node_type === 'tool') return '#f5a623';
        const state = node.properties && node.properties.state;
        if (state === 'failed') return '#f77';
        if (state === 'awaiting_approval') return '#f7b500';
        if (state === 'completed') return '#4a9eff';
        return '#9a6cff';
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

    updatePhysics: function() {
        const repulsion = 5000;
        const attraction = 0.005;
        const damping = 0.9;

        for (let i = 0; i < this.nodes.length; i++) {
            for (let j = i + 1; j < this.nodes.length; j++) {
                const dx = this.nodes[j].x - this.nodes[i].x;
                const dy = this.nodes[j].y - this.nodes[i].y;
                const dist = Math.max(1, Math.sqrt(dx * dx + dy * dy));
                const force = repulsion / (dist * dist);
                const fx = (dx / dist) * force;
                const fy = (dy / dist) * force;
                this.nodes[i].vx -= fx;
                this.nodes[i].vy -= fy;
                this.nodes[j].vx += fx;
                this.nodes[j].vy += fy;
            }
        }

        for (const edge of this.edges) {
            const source = this.nodes.find(n => n.id === edge.source);
            const target = this.nodes.find(n => n.id === edge.target);
            if (!source || !target) continue;
            const dx = target.x - source.x;
            const dy = target.y - source.y;
            const dist = Math.max(1, Math.sqrt(dx * dx + dy * dy));
            const force = (dist - 150) * attraction;
            const fx = (dx / dist) * force;
            const fy = (dy / dist) * force;
            source.vx += fx;
            source.vy += fy;
            target.vx -= fx;
            target.vy -= fy;
        }

        for (const node of this.nodes) {
            if (node === this.dragNode) continue;
            node.vx *= damping;
            node.vy *= damping;
            node.x += node.vx;
            node.y += node.vy;
            node.x = Math.max(node.radius, Math.min(this.canvas.width - node.radius, node.x));
            node.y = Math.max(node.radius, Math.min(this.canvas.height - node.radius, node.y));
        }
    },

    render: function() {
        const ctx = this.ctx;
        ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);

        for (const edge of this.edges) {
            const source = this.nodes.find(n => n.id === edge.source);
            const target = this.nodes.find(n => n.id === edge.target);
            if (!source || !target) continue;
            ctx.beginPath();
            ctx.moveTo(source.x, source.y);
            ctx.lineTo(target.x, target.y);
            ctx.strokeStyle = edge.relationship === 'triggered' ? 'rgba(245, 166, 35, 0.4)' : 'rgba(132, 214, 105, 0.4)';
            ctx.lineWidth = Math.max(1, edge.weight * 3 || 1);
            ctx.stroke();
        }

        for (const node of this.nodes) {
            ctx.beginPath();
            ctx.arc(node.x, node.y, node.radius, 0, Math.PI * 2);
            ctx.fillStyle = this.nodeColor(node);
            ctx.fill();
            ctx.strokeStyle = '#fff';
            ctx.lineWidth = 2;
            ctx.stroke();

            ctx.fillStyle = '#fff';
            ctx.font = '11px sans-serif';
            ctx.textAlign = 'center';
            ctx.textBaseline = 'middle';
            ctx.fillText(this.nodeLabel(node), node.x, node.y);
        }
    },

    onMouseDown: function(e) {
        const rect = this.canvas.getBoundingClientRect();
        const mx = e.clientX - rect.left;
        const my = e.clientY - rect.top;
        this.dragNode = this.nodes.find(n =>
            Math.hypot(n.x - mx, n.y - my) < n.radius
        ) || null;
        if (this.dragNode) this.isDragging = true;
    },

    onMouseMove: function(e) {
        if (!this.isDragging || !this.dragNode) return;
        const rect = this.canvas.getBoundingClientRect();
        this.dragNode.x = e.clientX - rect.left;
        this.dragNode.y = e.clientY - rect.top;
    },

    onMouseUp: function() {
        this.isDragging = false;
        this.dragNode = null;
    },

    onDoubleClick: function(e) {
        const rect = this.canvas.getBoundingClientRect();
        const mx = e.clientX - rect.left;
        const my = e.clientY - rect.top;
        const node = this.nodes.find(n =>
            Math.hypot(n.x - mx, n.y - my) < n.radius
        );
        if (node && node.node_type === 'run') {
            this.loadRunGraph(node.id.replace(/^run:/, ''));
        }
    },

    loadRunGraph: async function(runId) {
        try {
            const resp = await fetch(`/api/vibe/graph/run/${runId}`);
            const data = await resp.json();
            if (data.success && data.graph) {
                this.nodes = data.graph.nodes.map((n, i) => ({
                    ...n,
                    x: 60 + i * 90,
                    y: this.canvas.height / 2,
                    vx: 0, vy: 0,
                    radius: n.node_type === 'run' ? 18 : 14,
                }));
                this.edges = data.graph.edges;
                this.useCase = data.graph.nodes.length
                    ? (window.VibeGraphUseCase || this.useCase)
                    : this.useCase;
                this.startSimulation();
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
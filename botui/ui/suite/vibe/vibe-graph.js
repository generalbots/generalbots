/**
 * Vibe Knowledge Graph visualization (Issue #522)
 * Displays the knowledge graph from /api/vibe/graph/{use_case}
 * as an interactive force-directed graph using Canvas.
 */
const VibeGraph = {
    canvas: null,
    ctx: null,
    nodes: [],
    edges: [],
    animationId: null,
    isDragging: false,
    dragNode: null,

    init: function(canvasId) {
        this.canvas = document.getElementById(canvasId);
        if (!this.canvas) return;
        this.ctx = this.canvas.getContext('2d');
        this.setupEvents();
    },

    setupEvents: function() {
        this.canvas.addEventListener('mousedown', (e) => this.onMouseDown(e));
        this.canvas.addEventListener('mousemove', (e) => this.onMouseMove(e));
        this.canvas.addEventListener('mouseup', () => this.onMouseUp());
        window.addEventListener('resize', () => this.resize());
        this.resize();
    },

    resize: function() {
        if (!this.canvas) return;
        const rect = this.canvas.parentElement.getBoundingClientRect();
        this.canvas.width = rect.width;
        this.canvas.height = rect.height;
    },

    loadGraph: async function(useCase) {
        try {
            const resp = await fetch(`/api/vibe/graph/${useCase}`);
            const data = await resp.json();
            if (data.success && data.graph) {
                this.nodes = data.graph.nodes.map((n, i) => ({
                    ...n,
                    x: Math.random() * this.canvas.width,
                    y: Math.random() * this.canvas.height,
                    vx: 0, vy: 0,
                    radius: n.node_type === 'use_case' ? 30 : 20,
                }));
                this.edges = data.graph.edges;
                this.startSimulation();
            }
        } catch (e) {
            console.error('Failed to load graph:', e);
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
            ctx.strokeStyle = 'rgba(132, 214, 105, 0.4)';
            ctx.lineWidth = edge.weight || 1;
            ctx.stroke();
        }

        for (const node of this.nodes) {
            ctx.beginPath();
            ctx.arc(node.x, node.y, node.radius, 0, Math.PI * 2);
            ctx.fillStyle = node.node_type === 'use_case' ? '#84d669' : '#4a9eff';
            ctx.fill();
            ctx.strokeStyle = '#fff';
            ctx.lineWidth = 2;
            ctx.stroke();

            ctx.fillStyle = '#fff';
            ctx.font = '11px sans-serif';
            ctx.textAlign = 'center';
            ctx.textBaseline = 'middle';
            ctx.fillText(node.label.substring(0, 15), node.x, node.y);
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
};

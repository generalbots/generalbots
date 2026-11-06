/* Vibe Knowledge Graph — split from vibe/vibe-graph.js for #1731 */

Object.assign(window.VibeGraph, {
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
                        const approval = c.requires_approval ? ' <span class="vibe-cap-warn">guarded</span>' : '';
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
});

/* Vibe Knowledge Graph — split from vibe/vibe-graph.js for #1731 */

Object.assign(window.VibeGraph, {
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
});

function initVibeGraph() {
    VibeGraph.init('vibeGraphCanvas');
}

if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initVibeGraph);
} else {
    initVibeGraph();
}

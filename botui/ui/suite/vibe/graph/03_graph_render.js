/* Vibe Knowledge Graph — split from vibe/vibe-graph.js for #1731 */

Object.assign(window.VibeGraph, {
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
});

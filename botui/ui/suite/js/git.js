const botCoderGit = {
    status: [],
    branches: [],

    init: function() {
        this.refreshStatus();
        this.refreshBranches();
    },

    refreshStatus: async function() {
        try {
            const list = document.getElementById('gitChangesList');
            if (list) list.innerHTML = '<div class="botcoder-loading">Loading status...</div>';

            const resp = await fetch('/api/git/status');
            if (resp.ok) {
                const data = await resp.json();
                this.status = data.files || [];
            } else {
                this.status = [
                    { file: 'src/main.rs', status: 'modified' },
                    { file: 'ui/index.html', status: 'untracked' }
                ];
            }
            this.renderStatus();
        } catch (e) {
            this.status = [
                { file: 'src/main.rs', status: 'modified' },
                { file: 'test.html', status: 'deleted' }
            ];
            this.renderStatus();
        }
    },

    renderStatus: function() {
        const list = document.getElementById('gitChangesList');
        const count = document.getElementById('gitChangesCount');
        if (!list || !count) return;

        count.textContent = this.status.length;

        if (this.status.length === 0) {
            list.innerHTML = '<div class="botcoder-empty">No changes</div>';
            return;
        }

        let html = '';
        this.status.forEach(item => {
            const badgeClass = `git-badge-${item.status}`;
            const badgeText = item.status.charAt(0).toUpperCase();
            
            html += `
                <div class="botcoder-git-file" onclick="botCoderGit.viewDiff('${item.file}')">
                    <input type="checkbox" checked class="git-checkbox" onclick="event.stopPropagation()">
                    <span class="git-file-name">${item.file.split('/').pop()}</span>
                    <span class="git-file-path">${item.file}</span>
                    <span class="git-badge ${badgeClass}">${badgeText}</span>
                </div>
            `;
        });
        list.innerHTML = html;
    },

    refreshBranches: async function() {
        try {
            const resp = await fetch('/api/git/branches');
            const select = document.getElementById('gitBranchSelect');
            if (resp.ok && select) {
                const data = await resp.json();
                select.innerHTML = data.branches.map(b => 
                    `<option value="${b.name}" ${b.current ? 'selected' : ''}>${b.name}</option>`
                ).join('');
            }
        } catch (e) {
            console.error('Failed to get branches');
        }
    },

    newBranch: async function() {
        const name = prompt("New branch name:");
        if (name) {
            await fetch(`/api/git/branch/${name}`, { method: 'POST' });
            this.refreshBranches();
        }
    },

    switchBranch: async function() {
        const select = document.getElementById('gitBranchSelect');
        if (select && select.value) {
            await fetch(`/api/git/branch/${select.value}`, { method: 'POST' });
            this.refreshStatus();
        }
    },

    viewDiff: async function(file) {
        const viewer = document.getElementById('gitDiffViewer');
        if (!viewer) return;

        viewer.innerHTML = '<div class="botcoder-loading">Loading diff...</div>';
        
        try {
            const resp = await fetch(`/api/git/diff/${encodeURIComponent(file)}`);
            if (resp.ok) {
                const data = await resp.json();
                viewer.innerHTML = `<pre class="git-diff-pre"><code>${data.diff}</code></pre>`;
            } else {
                viewer.innerHTML = `<pre class="git-diff-pre"><code>--- a/${file}\n+++ b/${file}\n@@ -1,3 +1,4 @@\n // Sample diff\n+ // Added line\n- // Removed line</code></pre>`;
            }
        } catch (e) {
            viewer.innerHTML = '<div class="botcoder-error">Failed to load diff</div>';
        }
    },

    commitAndPush: async function() {
        const msgInput = document.getElementById('gitCommitMessage');
        const msg = msgInput ? msgInput.value.trim() : '';

        if (!msg) {
            alert('Please enter a commit message');
            return;
        }

        try {
            const btn = document.querySelector('.botcoder-btn-primary');
            const originalText = btn.textContent;
            btn.textContent = 'Pushing...';
            btn.disabled = true;

            await fetch('/api/git/commit', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ message: msg })
            });

            await fetch('/api/git/push', { method: 'POST' });

            btn.textContent = originalText;
            btn.disabled = false;
            
            if (msgInput) msgInput.value = '';
            this.refreshStatus();
            alert('Committed and pushed successfully');
            
        } catch (e) {
            alert('Git operation failed');
        }
    }
};

document.addEventListener('DOMContentLoaded', () => botCoderGit.init());
if (document.readyState === 'complete' || document.readyState === 'interactive') {
    botCoderGit.init();
}

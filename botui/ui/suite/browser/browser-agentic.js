/* Agentic browser (#1175/#1182): browsing memory with site chips and cited
   answers, plus the browser-driver contract console. Loaded by browser.html
   after the main GBBrowser IIFE. */
(function () {
    'use strict';
    if (window.GBAppLifecycle) GBAppLifecycle.begin('browser');

    const MEMORY_API = '/api/vibe/browser-memory';
    const DRIVER_API = '/api/vibe/browser-driver';
    let driverRunId = null;
    let driverSteps = [];

    function $(id) { return document.getElementById(id); }

    function currentUrl() {
        const input = $('browser-url');
        return input ? input.value.trim() : '';
    }

    function currentDomain() {
        const url = currentUrl();
        try { return new URL(url).hostname; } catch (e) { return ''; }
    }

    function api(path, method, body) {
        const opts = { method: method || 'GET', headers: { 'Content-Type': 'application/json' } };
        if (body) opts.body = JSON.stringify(body);
        return fetch(path, opts).then(function (r) { return r.json().catch(function () { return {}; }); });
    }

    function esc(s) {
        return String(s == null ? '' : s).replace(/[&<>"']/g, function (c) {
            return { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c];
        });
    }

    /* ---------- Memory tab (site chips + cited answers) ---------- */

    function renderMemory() {
        const domain = currentDomain();
        const body = $('browser-agentic-body');
        body.innerHTML =
            '<div class="browser-agentic-section">' +
            '<h4>Browsing memory</h4>' +
            '<div class="browser-agentic-row">' +
            '<input id="bmFact" class="browser-agentic-input" placeholder="Remember a fact about this page…" />' +
            '<button id="bmRemember" class="browser-agentic-btn">Remember</button>' +
            '</div>' +
            '<div id="bmChips" class="browser-agentic-chips"></div>' +
            '</div>' +
            '<div class="browser-agentic-section">' +
            '<h4>Ask from memory (cited)</h4>' +
            '<div class="browser-agentic-row">' +
            '<input id="bmQuestion" class="browser-agentic-input" placeholder="Question about pages you browsed…" />' +
            '<button id="bmAsk" class="browser-agentic-btn">Ask</button>' +
            '</div>' +
            '<div id="bmAnswer" class="browser-agentic-answer"></div>' +
            '</div>';

        const domainInput = $('bmFact');
        if (domainInput && domain) domainInput.placeholder = 'Remember a fact about ' + domain + '…';

        const remember = $('bmRemember');
        if (remember) {
            remember.addEventListener('click', function () {
                const fact = $('bmFact').value.trim();
                const url = currentUrl();
                const dom = currentDomain();
                if (!fact || !dom) { return; }
                api(MEMORY_API, 'POST', { domain: dom, url: url || dom, fact: fact }).then(function () {
                    $('bmFact').value = '';
                    loadChips();
                });
            });
        }

        const ask = $('bmAsk');
        if (ask) {
            ask.addEventListener('click', function () {
                const q = $('bmQuestion').value.trim();
                if (!q) { return; }
                const answerBox = $('bmAnswer');
                answerBox.innerHTML = '<em>Answering from memory…</em>';
                const dom = currentDomain();
                api(MEMORY_API + '/ask', 'POST', { question: q, domains: dom ? [dom] : [] }).then(function (d) {
                    if (!d.success) {
                        answerBox.innerHTML = '<p class="browser-agentic-dim">' + esc(d.error || 'no memory yet') + '</p>';
                        return;
                    }
                    let html = '<p>' + esc(d.answer.answer).replace(/\n/g, '<br>') + '</p>';
                    if (d.answer.citations && d.answer.citations.length) {
                        html += '<ul class="browser-agentic-citations">' + d.answer.citations.map(function (c) {
                            return '<li><a href="#" onclick="event.preventDefault();GBBrowser.go(\'' + esc(c) + '\');return false;">' + esc(c) + '</a></li>';
                        }).join('') + '</ul>';
                    }
                    answerBox.innerHTML = html;
                });
            });
        }

        loadChips();
    }

    function loadChips() {
        const box = $('bmChips');
        if (!box) return;
        const dom = currentDomain();
        const query = dom ? '?domain=' + encodeURIComponent(dom) : '';
        api(MEMORY_API + query).then(function (d) {
            const chips = (d.chips) || [];
            box.innerHTML = '';
            if (!chips.length) {
                box.innerHTML = '<p class="browser-agentic-dim">No memory for this domain yet. Browse and click “Remember”.</p>';
                return;
            }
            chips.forEach(function (chip) {
                const el = document.createElement('div');
                el.className = 'browser-agentic-chip';
                el.innerHTML = '<strong>' + esc(chip.domain) + '</strong> — ' + esc(chip.fact);
                box.appendChild(el);
            });
        });
    }

    /* ---------- Driver tab (#1182) ---------- */

    function renderDriver() {
        const body = $('browser-agentic-body');
        body.innerHTML =
            '<div class="browser-agentic-section">' +
            '<h4>Browser driver (task contract)</h4>' +
            '<div class="browser-agentic-row"><label>Goal</label><input id="bdGoal" class="browser-agentic-input" placeholder="e.g. extract pricing from the homepage" /></div>' +
            '<div class="browser-agentic-row"><label>Policy</label><input id="bdPolicy" class="browser-agentic-input" value="read-only, no logins" /></div>' +
            '<div class="browser-agentic-row"><label>Budget</label><input id="bdBudget" class="browser-agentic-input" type="number" min="1" max="50" value="8" style="width:4rem" /> steps</div>' +
            '<button id="bdStart" class="browser-agentic-btn">▶ Start contract</button>' +
            '<div id="bdPlan" class="browser-agentic-plan"></div>' +
            '</div>';

        $('bdStart').addEventListener('click', function () {
            const goal = $('bdGoal').value.trim();
            const policy = $('bdPolicy').value.trim();
            const budget = parseInt($('bdBudget').value, 10) || 8;
            const url = currentUrl();
            if (!url) { return; }
            api(DRIVER_API + '/start', 'POST', {
                contract: { url: url, goal: goal || 'browse and summarize', policy: policy, budget_steps: budget },
            }).then(function (d) {
                if (!d.success) {
                    $('bdPlan').innerHTML = '<p class="browser-agentic-dim">' + esc(d.error || 'start failed') + '</p>';
                    return;
                }
                driverRunId = d.run.run_id;
                driverSteps = d.run.plan || [];
                renderPlan(d.run);
                // Kick off the first step: navigate to the contract target.
                GBBrowser.go(url);
            });
        });
    }

    function renderPlan(run) {
        const box = $('bdPlan');
        const steps = run.plan || [];
        const done = (run.steps || []).length;
        let html = '<h5>Plan (' + done + '/' + steps.length + ' steps)</h5><ol class="browser-agentic-plan-list">';
        steps.forEach(function (s, i) {
            const completed = i < done;
            html += '<li class="' + (completed ? 'done' : '') + '">' + esc(s) +
                (completed ? ' ✓' : ' <button class="browser-agentic-btn small" data-step="' + i + '">done</button>') +
                '</li>';
        });
        html += '</ol>';
        if (done >= steps.length && run.status !== 'complete') {
            html += '<button id="bdComplete" class="browser-agentic-btn">Finish &amp; complete</button>';
        }
        box.innerHTML = html;

        box.querySelectorAll('button[data-step]').forEach(function (btn) {
            btn.addEventListener('click', function () {
                const i = parseInt(btn.dataset.step, 10);
                const step = driverSteps[i];
                if (!driverRunId || !step) return;
                api(DRIVER_API + '/runs/' + driverRunId + '/step', 'POST', { description: step, detail: 'executed by user' })
                    .then(function (d) {
                        if (d.run) renderPlan(d.run);
                    });
            });
        });
        const finish = $('bdComplete');
        if (finish) {
            finish.addEventListener('click', function () {
                api(DRIVER_API + '/runs/' + driverRunId + '/complete', 'POST').then(function (d) {
                    if (d.run) renderPlan(d.run);
                });
            });
        }
    }

    function wire() {
        const toggle = $('btn-agentic');
        const panel = $('browser-agentic-panel');
        if (!toggle || !panel) return;
        toggle.addEventListener('click', function () {
            const visible = panel.style.display !== 'none';
            panel.style.display = visible ? 'none' : 'flex';
            if (!visible) {
                const activeTab = panel.querySelector('.browser-agentic-tab.active');
                const tab = activeTab ? activeTab.dataset.tab : 'chips';
                if (tab === 'chips') renderMemory(); else renderDriver();
            }
        });
        panel.querySelectorAll('.browser-agentic-tab').forEach(function (tab) {
            tab.addEventListener('click', function () {
                panel.querySelectorAll('.browser-agentic-tab').forEach(function (t) { t.classList.remove('active'); });
                tab.classList.add('active');
                if (tab.dataset.tab === 'chips') renderMemory(); else renderDriver();
            });
        });
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', wire);
    } else {
        wire();
    }
    if (window.GBAppLifecycle) GBAppLifecycle.end('browser');
})();

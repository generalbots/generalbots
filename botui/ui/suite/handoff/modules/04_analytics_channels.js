/* Handoff app — module 4/5: analytics, channels and CSAT cards (split from handoff/handoff.js for #1731) */

(function (app) {
'use strict';
Object.assign(app, {
    async loadAnalytics() {
        try {
            const data = await this.api('/api/handoff/analytics');
            this.state.analytics = data || {};
            this.renderAnalytics();
        } catch (e) {
            document.getElementById('analytics-cards').innerHTML = '<div class="loading-row">Failed to load analytics</div>';
        }
    },

    renderAnalytics() {
        const a = this.state.analytics;
        const hourly = a.hourly || [];
        const maxH = hourly.length ? Math.max(...hourly.map(h => h.count || 0), 1) : 1;
        document.getElementById('analytics-cards').innerHTML = `
            <div class="analytics-card">
                <span class="analytics-card-title">Conversations Today</span>
                <span class="analytics-card-value accent">${a.conversations_today || 0}</span>
                <span class="analytics-card-detail">${a.conversations_change || ''} vs yesterday</span>
            </div>
            <div class="analytics-card">
                <span class="analytics-card-title">Avg Response Time</span>
                <span class="analytics-card-value">${this.fmtTime(a.avg_response_seconds)}</span>
                <span class="analytics-card-detail">${a.response_change || ''} vs yesterday</span>
            </div>
            <div class="analytics-card">
                <span class="analytics-card-title">Resolution Rate</span>
                <span class="analytics-card-value success">${a.resolution_rate || 0}%</span>
                <span class="analytics-card-detail">${a.resolved || 0} resolved</span>
            </div>
            <div class="analytics-card">
                <span class="analytics-card-title">Bot Containment</span>
                <span class="analytics-card-value">${a.bot_containment || 0}%</span>
                <span class="analytics-card-detail">${a.contained || 0} contained by bot</span>
            </div>
            <div class="analytics-chart">
                <h3>Hourly Conversations</h3>
                <div class="chart-bars">
                    ${hourly.map(h => {
                        const pct = maxH ? ((h.count || 0) / maxH * 100) : 0;
                        return `<div class="chart-bar" style="height:${Math.max(pct, 2)}%"><span class="chart-bar-value">${h.count || 0}</span><span class="chart-bar-label">${this.esc(h.hour || '')}</span></div>`;
                    }).join('')}
                </div>
            </div>
        `;
    },

    async loadChannels() {
        try {
            const data = await this.api('/api/handoff/channels');
            this.state.channels = Array.isArray(data) ? data : (data.items || []);
            this.renderChannels();
        } catch (e) {
            document.getElementById('channels-grid').innerHTML = '<div class="loading-row">Failed to load channels</div>';
        }
    },

    renderChannels() {
        const channels = this.state.channels;
        if (!channels.length) { document.getElementById('channels-grid').innerHTML = '<div class="empty-state"><p>No channels configured</p></div>'; return; }
        const iconMap = {
            whatsapp: '<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"/></svg>',
            telegram: '<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 2L11 13M22 2l-7 20-4-9-9-4 20-7z"/></svg>',
            teams: '<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="7" width="18" height="13" rx="2"/><path d="M16 3v4M8 3v4"/></svg>',
            web: '<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>'
        };
        document.getElementById('channels-grid').innerHTML = channels.map(ch => `
            <div class="channel-card">
                <div class="channel-card-header">
                    <span class="channel-card-name">${iconMap[ch.type] || ''} ${this.esc(ch.name)}</span>
                    <span class="badge ${ch.connected ? 'connected' : 'disconnected'}">${ch.connected ? 'Connected' : 'Disconnected'}</span>
                </div>
                <div class="channel-card-body">
                    <div class="channel-row"><span class="ch-label">Messages Today</span><span class="ch-val">${ch.messages_today || 0}</span></div>
                    <div class="channel-row"><span class="ch-label">Active Conversations</span><span class="ch-val">${ch.active_conversations || 0}</span></div>
                    <div class="channel-row"><span class="ch-label">Avg Response</span><span class="ch-val">${this.fmtTime(ch.avg_response_seconds)}</span></div>
                    <div class="channel-row"><span class="ch-label">Error Rate</span><span class="ch-val">${ch.error_rate || 0}%</span></div>
                </div>
            </div>
        `).join('');
    },

    async loadCSAT() {
        try {
            const data = await this.api('/api/handoff/csat');
            this.state.csat = data || {};
            this.renderCSAT();
        } catch (e) {
            document.getElementById('csat-grid').innerHTML = '<div class="loading-row">Failed to load CSAT</div>';
        }
    },

    renderCSAT() {
        const c = this.state.csat;
        const ratings = c.recent_ratings || [];
        const distribution = c.distribution || [];
        const maxDist = distribution.length ? Math.max(...distribution.map(d => d.count || 0), 1) : 1;
        document.getElementById('csat-grid').innerHTML = `
            <div class="csat-score-card">
                <span class="csat-score-label">Overall Score</span>
                <div class="csat-score-value">${(c.avg_score || 0).toFixed(1)}</div>
                <span class="csat-score-label">${c.total_ratings || 0} ratings</span>
            </div>
            <div class="csat-score-card">
                <span class="csat-score-label">Response Score</span>
                <div class="csat-score-value" style="color:#22c55e">${(c.response_score || 0).toFixed(1)}</div>
                <span class="csat-score-label">Helpfulness</span>
            </div>
            <div class="csat-score-card">
                <span class="csat-score-label">Resolution Score</span>
                <div class="csat-score-value" style="color:#60a5fa">${(c.resolution_score || 0).toFixed(1)}</div>
                <span class="csat-score-label">Problem solved</span>
            </div>
            <div class="csat-ratings">
                <h3 style="margin:0;font-size:14px;font-weight:600">Recent Ratings</h3>
                ${!ratings.length ? '<div class="empty-state"><p>No ratings yet</p></div>' : ratings.map(r => `
                    <div class="csat-rating-item">
                        <div class="csat-rating-stars">${this.renderStars(r.score)}</div>
                        <div class="csat-rating-body">
                            <span class="csat-rating-comment">${this.esc(r.comment || 'No comment')}</span>
                            <span class="csat-rating-meta">${this.esc(r.user_name || 'Anonymous')} - ${this.esc(r.created_at || '')}</span>
                        </div>
                    </div>
                `).join('')}
            </div>
        `;
    },

    renderStars(score) {
        return Array.from({length: 5}, (_, i) =>
            `<svg class="csat-star ${i < score ? '' : 'empty'}" width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg>`
        ).join('');
    }
});
})(window.HandoffApp);

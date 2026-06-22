if (typeof window.WindowManager === "undefined") {
  "use strict";

  const APPS_REGISTRY = [
    { id: "vibe", title: "Vibe", category: "ai", color: "#84d669", hxGet: "/suite/partials/vibe.html",
      icon: '<path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"/>' },
    { id: "crm", title: "CRM", category: "business", color: "#3b82f6", hxGet: "/suite/crm/crm.html",
      icon: '<path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/>' },
    { id: "campaigns", title: "Campaigns", category: "business", color: "#f59e0b", hxGet: "/suite/campaigns/campaigns.html",
      icon: '<path d="M22 12h-4l-3 9L9 3l-3 9H2"/>' },
    { id: "lists", title: "Lists", category: "business", color: "#8b5cf6", hxGet: "/suite/lists/lists.html",
      icon: '<line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><line x1="3" y1="6" x2="3.01" y2="6"/><line x1="3" y1="12" x2="3.01" y2="12"/><line x1="3" y1="18" x2="3.01" y2="18"/>' },
    { id: "templates", title: "Templates", category: "office", color: "#ec4899", hxGet: "/suite/templates/templates.html",
      icon: '<rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><line x1="3" y1="9" x2="21" y2="9"/><line x1="9" y1="21" x2="9" y2="9"/>' },
    { id: "tasks", title: "Tasks", category: "office", color: "#22c55e", hxGet: "/suite/tasks/task-window.html",
      icon: '<path d="M9 11l3 3L22 4"/><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/>' },
    { id: "chat", title: "Chat", category: "ai", color: "#84d669", hxGet: "/suite/partials/chat.html",
      icon: '<path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>' },
    { id: "terminal", title: "Terminal", category: "dev", color: "#64748b", hxGet: "/suite/terminal/terminal.html",
      icon: '<polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/>' },
    { id: "drive", title: "Explorer", category: "system", color: "#f59e0b", hxGet: "/suite/drive/drive.html",
      icon: '<path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>' },
    { id: "editor", title: "Editor", category: "dev", color: "#3b82f6", hxGet: "/suite/editor.html",
      icon: '<polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/>' },
    { id: "designer", title: "Designer", category: "office", color: "#ec4899", hxGet: "/suite/designer.html",
      icon: '<path d="M12 19l7-7 3 3-7 7-3-3z"/><path d="M18 13l-1.5-7.5L2 2l3.5 14.5L13 18l5-5z"/><path d="M2 2l7.586 7.586"/><circle cx="11" cy="11" r="2"/>' },
    { id: "bas-editor", title: "BASIC", category: "dev", color: "#84d669", hxGet: "/suite/partials/vibe.html?mode=bas",
      icon: '<polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" y2="20"/>' },
    { id: "browser", title: "Browser", category: "system", color: "#3b82f6", hxGet: "/suite/browser/browser.html",
      icon: '<circle cx="12" cy="12" r="10"/><polygon points="16.24 7.76 14.12 14.12 7.76 16.24 9.88 9.88 16.24 7.76"/>' },
    { id: "versions", title: "Versions", category: "dev", color: "#8b5cf6", hxGet: "/suite/partials/versions-panel.html",
      icon: '<circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/>' },
    { id: "database", title: "Database", category: "dev", color: "#f59e0b", hxGet: "/suite/database/database.html",
      icon: '<ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/>' },
    { id: "vdi", title: "VDI", category: "system", color: "#06b6d4", hxGet: "/suite/desktop/vdi.html",
      icon: '<rect x="2" y="3" width="20" height="14" rx="2" ry="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/>' },
    { id: "mail", title: "Mail", category: "office", color: "#3b82f6", hxGet: "/suite/mail/mail.html",
      icon: '<path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"/><polyline points="22,6 12,13 2,6"/>' },
    { id: "meet", title: "Meet", category: "office", color: "#ef4444", hxGet: "/suite/meet/meet.html",
      icon: '<polygon points="23 7 16 12 23 17 23 7"/><rect x="1" y="5" width="15" height="14" rx="2" ry="2"/>' },
    { id: "docs", title: "Docs", category: "office", color: "#3b82f6", hxGet: "/suite/docs/docs.html",
      icon: '<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/>' },
    { id: "plan", title: "Plan", category: "office", color: "#f59e0b", hxGet: "/suite/plan/plan.html",
      icon: '<rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/><line x1="10" y1="6" x2="14" y2="6"/><line x1="6" y1="10" x2="6" y2="14"/><line x1="10" y1="18" x2="14" y2="18"/><line x1="18" y1="10" x2="18" y2="14"/>' },
    { id: "calendar", title: "Calendar", category: "office", color: "#ec4899", hxGet: "/suite/calendar/calendar.html",
      icon: '<rect x="3" y="4" width="18" height="18" rx="2" ry="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/>' },
    { id: "billing", title: "Billing", category: "business", color: "#22c55e", hxGet: "/suite/billing/billing.html",
      icon: '<rect x="1" y="4" width="22" height="16" rx="2" ry="2"/><line x1="1" y1="10" x2="23" y2="10"/>' },
    { id: "products", title: "Products", category: "business", color: "#84d669", hxGet: "/suite/products/products.html",
      icon: '<path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/>' },
    { id: "research", title: "Research", category: "ai", color: "#8b5cf6", hxGet: "/suite/research/research.html",
      icon: '<circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>' },
    { id: "gb-office", title: "Beat MS", category: "office", color: "#10b981", hxGet: "/suite/beat-microsoft.html",
      icon: '<path d="M12 2L2 7l10 5 10-5-10-5z"/><path d="M2 17l10 5 10-5"/><path d="M2 12l10 5 10-5"/>' },
    { id: "people", title: "People", category: "business", color: "#3b82f6", hxGet: "/suite/people/people.html",
      icon: '<path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/>' },
    { id: "tickets", title: "Tickets", category: "business", color: "#ef4444", hxGet: "/suite/tickets/tickets.html",
      icon: '<path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/><polyline points="14 2 14 8 20 8"/><line x1="9" y1="15" x2="15" y2="15"/>' },
    { id: "social", title: "Social", category: "business", color: "#ec4899", hxGet: "/suite/social/social.html",
      icon: '<path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"/>' },
    { id: "compliance", title: "Compliance", category: "business", color: "#64748b", hxGet: "/suite/compliance/compliance.html",
      icon: '<path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>' },
    { id: "tax", title: "Tax", category: "business", color: "#f59e0b", hxGet: "/suite/tax/tax.html",
      icon: '<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/>' },
    { id: "video", title: "Video AI", category: "ai", color: "#ef4444", hxGet: "/suite/video/video.html",
      icon: '<path d="M23 7l-7 5 7 5V7z"/><rect x="1" y="5" width="15" height="14" rx="2" ry="2"/>' },
    { id: "vision", title: "Vision", category: "ai", color: "#06b6d4", hxGet: "/suite/vision/vision.html",
      icon: '<path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/>' },
    { id: "fraud", title: "Anti-Fraud", category: "business", color: "#ef4444", hxGet: "/suite/fraud/fraud.html",
      icon: '<path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/><line x1="9" y1="12" x2="15" y2="12"/>' },
    { id: "erp", title: "ERP", category: "business", color: "#3b82f6", hxGet: "/suite/erp/erp.html",
      icon: '<rect x="2" y="3" width="20" height="14" rx="2" ry="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/>' },
    { id: "integrations", title: "Integrations", category: "dev", color: "#8b5cf6", hxGet: "/suite/integrations/integrations.html",
      icon: '<circle cx="18" cy="5" r="3"/><circle cx="6" cy="12" r="3"/><circle cx="18" cy="19" r="3"/><line x1="8.59" y1="13.51" x2="15.42" y2="17.49"/><line x1="15.41" y1="6.51" x2="8.59" y2="10.49"/>' },
    { id: "itsm", title: "ITSM", category: "dev", color: "#06b6d4", hxGet: "/suite/itsm/itsm.html",
      icon: '<path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72c.127.96.362 1.903.7 2.81a2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45c.907.338 1.85.573 2.81.7A2 2 0 0 1 22 16.92z"/>' },
    { id: "hr", title: "HR", category: "business", color: "#ec4899", hxGet: "/suite/hr/hr.html",
      icon: '<path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/>' },
    { id: "banking", title: "Banking", category: "business", color: "#22c55e", hxGet: "/suite/banking/banking.html",
      icon: '<line x1="12" y1="1" x2="12" y2="23"/><path d="M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"/>' },
    { id: "sales", title: "Sales", category: "business", color: "#84d669", hxGet: "/suite/sales/sales.html",
      icon: '<polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/>' },
    { id: "pos", title: "POS", category: "business", color: "#f59e0b", hxGet: "/suite/pos/pos.html",
      icon: '<circle cx="9" cy="21" r="1"/><circle cx="20" cy="21" r="1"/><path d="M1 1h4l2.68 13.39a2 2 0 0 0 2 1.61h9.72a2 2 0 0 0 2-1.61L23 6H6"/>' },
    { id: "retail", title: "Retail", category: "business", color: "#ec4899", hxGet: "/suite/retail/retail.html",
      icon: '<path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/><polyline points="9 22 9 12 15 12 15 22"/>' },
    { id: "handoff", title: "Handoff", category: "business", color: "#06b6d4", hxGet: "/suite/handoff/handoff.html",
      icon: '<path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/>' },
    { id: "kyc", title: "KYC", category: "business", color: "#64748b", hxGet: "/suite/kyc/kyc.html",
      icon: '<rect x="3" y="4" width="18" height="16" rx="2"/><circle cx="12" cy="10" r="3"/><path d="M7 20l1-4h8l1 4"/>' },
    { id: "biometry", title: "Biometry", category: "system", color: "#22c55e", hxGet: "/suite/biometry/biometry.html",
      icon: '<path d="M12 2C8 2 5 5 5 9c0 5 7 13 7 13s7-8 7-13c0-4-3-7-7-7z"/><circle cx="12" cy="9" r="2.5"/>' },
    { id: "timeclock", title: "Time Clock", category: "office", color: "#f59e0b", hxGet: "/suite/timeclock/timeclock.html",
      icon: '<circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/>' },
    { id: "m365", title: "M365", category: "office", color: "#3b82f6", hxGet: "/suite/m365/m365.html",
      icon: '<rect x="2" y="2" width="9" height="9"/><rect x="13" y="2" width="9" height="9"/><rect x="2" y="13" width="9" height="9"/><rect x="13" y="13" width="9" height="9"/>' },
    { id: "office365", title: "Office 365", category: "office", color: "#ef4444", hxGet: "/suite/office365/office365.html",
      icon: '<rect x="3" y="3" width="18" height="18" rx="2"/><line x1="9" y1="3" x2="9" y2="21"/><line x1="15" y1="3" x2="15" y2="21"/><line x1="3" y1="9" x2="21" y2="9"/><line x1="3" y1="15" x2="21" y2="15"/>' },
    { id: "learn", title: "Learn", category: "ai", color: "#84d669", hxGet: "/suite/learn/learn-app.html",
      icon: '<path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"/><path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"/>' },
    { id: "minutes", title: "Minutes", category: "office", color: "#8b5cf6", hxGet: "/suite/minutes/minutes.html",
      icon: '<path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/>' },
  ];

  window.APPS_REGISTRY = APPS_REGISTRY;

  const CATEGORY_LABELS = {
    ai: "AI & Assistants",
    business: "Business",
    office: "Office & Productivity",
    dev: "Development",
    system: "System & Tools",
  };

  class WindowManager {
    constructor() {
      this.openWindows = [];
      this.activeWindowId = null;
      this.zIndexCounter = 100;
      this.workspace = null;
      this.taskbarCenter = null;
      this.startMenuOpen = false;
      this.useGlassWindows = true;
    }

    getWorkspace() {
      if (!this.workspace) {
        this.workspace = document.getElementById("desktop-content") || document.body;
      }
      return this.workspace;
    }

    getTaskbarCenter() {
      if (!this.taskbarCenter) {
        this.taskbarCenter = document.getElementById("taskbar-center");
      }
      return this.taskbarCenter;
    }

    getIconSvg(id) {
      const app = APPS_REGISTRY.find((a) => a.id === id);
      if (app) return `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">${app.icon}</svg>`;
      return '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/></svg>';
    }

    open(id, title, htmlContent) {
      const existingWindow = this.openWindows.find((w) => w.id === id);
      if (existingWindow) {
        this.focus(id);
        return;
      }

      const windowData = { id, title, isMinimized: false, isMaximized: false, previousState: null };
      this.openWindows.push(windowData);

      const workspace = this.getWorkspace();
      const offset = (this.openWindows.length * 24) % 120;
      const top = 60 + offset;
      const left = 180 + offset;

      const windowEl = document.createElement("div");
      windowEl.id = `window-${id}`;
      windowEl.style.top = `${top}px`;
      windowEl.style.left = `${left}px`;
      windowEl.style.zIndex = this.zIndexCounter++;

      if (this.useGlassWindows) {
        windowEl.className = "window-element-glass";
        windowEl.innerHTML = this._glassHeader(id, title) + this._glassBody(id);
      } else {
        windowEl.className = "window-element";
        windowEl.innerHTML = this._legacyHeader(id, title) + this._legacyBody(id);
      }

      workspace.appendChild(windowEl);
      this._injectBodyContent(id, htmlContent);
      this._addTaskbarDockItem(id);
      this._makeDraggable(windowEl);
      this._makeResizable(windowEl);
      this.focus(id);
      this._trackRecent(id, title);
      if (window.htmx) htmx.process(windowEl);
      if (window.Desktop3D && window.Desktop3D.initialized) {
        window.Desktop3D.createWindowPlane(id, title);
        window.Desktop3D.flipToWindow(id);
      }
    }

    _glassHeader(id, title) {
      return `<div class="window-header-glass">
        <div class="window-dot-controls">
          <div class="window-dot window-dot-close" onclick="window.WindowManager.close('${id}')"></div>
          <div class="window-dot window-dot-minimize" onclick="window.WindowManager.toggleMinimize('${id}')"></div>
          <div class="window-dot window-dot-maximize" onclick="window.WindowManager.toggleMaximize('${id}')"></div>
        </div>
        <div class="window-title">${title}</div>
      </div>`;
    }

    _glassBody(id) {
      return `<div id="window-body-${id}" class="window-body-glass"></div>`;
    }

    _legacyHeader(id, title) {
      return `<div class="window-header"><div class="font-mono text-xs font-bold text-brand-600 tracking-wide">${title}</div><div class="flex space-x-3 text-gray-400"><button class="btn-minimize hover:text-gray-600" onclick="window.WindowManager.toggleMinimize('${id}')"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="5" y1="12" x2="19" y2="12"/></svg></button><button class="btn-maximize hover:text-gray-600" onclick="window.WindowManager.toggleMaximize('${id}')"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/></svg></button><button class="btn-close hover:text-red-500" onclick="window.WindowManager.close('${id}')"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg></button></div></div>`;
    }

    _legacyBody(id) {
      return `<div id="window-body-${id}" class="window-body relative flex-1 overflow-y-auto"></div>`;
    }

    _injectBodyContent(id, htmlContent) {
      const body = document.getElementById(`window-body-${id}`);
      if (!body) return;
      const tempDiv = document.createElement("div");
      tempDiv.innerHTML = htmlContent;
      const scripts = Array.from(tempDiv.querySelectorAll("script")).map((s) => {
        const clone = document.createElement("script");
        Array.from(s.attributes).forEach((a) => clone.setAttribute(a.name, a.value));
        clone.textContent = s.textContent;
        s.remove();
        return clone;
      });
      body.innerHTML = tempDiv.innerHTML;
      scripts.forEach((s) => body.appendChild(s));
    }

    _addTaskbarDockItem(id) {
      const center = this.getTaskbarCenter();
      if (!center) return;
      const app = APPS_REGISTRY.find((a) => a.id === id);
      const dockItem = document.createElement("div");
      dockItem.id = `dock-item-${id}`;
      dockItem.className = "taskbar-dock-item";
      dockItem.title = app ? app.title : id;
      dockItem.onclick = () => this.toggleMinimize(id);
      dockItem.innerHTML = this.getIconSvg(id);
      center.appendChild(dockItem);
    }

    _trackRecent(id, title) {
      try {
        let recent = JSON.parse(localStorage.getItem("gb-recent-apps") || "[]");
        recent = recent.filter((r) => r.id !== id);
        recent.unshift({ id, title, ts: Date.now() });
        recent = recent.slice(0, 10);
        localStorage.setItem("gb-recent-apps", JSON.stringify(recent));
      } catch (e) {}
    }

    getRecentApps() {
      try {
        const recent = JSON.parse(localStorage.getItem("gb-recent-apps") || "[]");
        return recent.map((r) => {
          const app = APPS_REGISTRY.find((a) => a.id === r.id);
          return app || { id: r.id, title: r.title, category: "recent", color: "#666", icon: "", hxGet: "" };
        });
      } catch (e) {
        return [];
      }
    }

    focus(id) {
      this.activeWindowId = id;
      const el = document.getElementById(`window-${id}`);
      if (el) el.style.zIndex = this.zIndexCounter++;
      
      const obj = this.openWindows.find((w) => w.id === id);
      if (obj) document.title = `${obj.title} - General Bots`;
      this._updateDockActive();
    }

    _updateDockActive() {
      const center = this.getTaskbarCenter();
      if (!center) return;
      center.querySelectorAll(".taskbar-dock-item").forEach((item) => item.classList.remove("active"));
      if (this.activeWindowId) {
        const active = document.getElementById(`dock-item-${this.activeWindowId}`);
        if (active) active.classList.add("active");
      }
    }

    close(id) {
      const el = document.getElementById(`window-${id}`);
      if (el) {
        if (this.useGlassWindows) {
          el.classList.add("closing");
          setTimeout(() => el.remove(), 200);
        } else {
          el.remove();
        }
      }
      const dockEl = document.getElementById(`dock-item-${id}`);
      if (dockEl) dockEl.remove();
      this.openWindows = this.openWindows.filter((w) => w.id !== id);
      if (this.activeWindowId === id) {
        this.activeWindowId = null;
        this._updateDockActive();
        if (this.openWindows.length === 0) {
          document.title = "General Bots Desktop";
        }
      }
      if (window.Desktop3D && window.Desktop3D.initialized) {
        window.Desktop3D.removeWindow(id);
      }
    }

    toggleMinimize(id) {
      const obj = this.openWindows.find((w) => w.id === id);
      if (!obj) return;
      const el = document.getElementById(`window-${id}`);
      if (!el) return;
      if (obj.isMinimized) {
        el.style.display = "flex";
        obj.isMinimized = false;
        this.focus(id);
      } else {
        el.style.display = "none";
        obj.isMinimized = true;
        if (this.activeWindowId === id) {
          this.activeWindowId = null;
          this._updateDockActive();
        }
      }
    }

    toggleMaximize(id) {
      const obj = this.openWindows.find((w) => w.id === id);
      if (!obj) return;
      const el = document.getElementById(`window-${id}`);
      if (!el) return;
      if (obj.isMaximized) {
        el.style.width = obj.previousState.width;
        el.style.height = obj.previousState.height;
        el.style.top = obj.previousState.top;
        el.style.left = obj.previousState.left;
        el.style.borderRadius = "";
        obj.isMaximized = false;
        const anyMaximized = this.openWindows.some((w) => w.isMaximized);
        if (!anyMaximized) document.body.classList.remove("window-maximized");
      } else {
        obj.previousState = { width: el.style.width, height: el.style.height, top: el.style.top, left: el.style.left };
        el.style.width = "100%";
        el.style.height = "100%";
        el.style.top = "0";
        el.style.left = "0";
        el.style.borderRadius = "0";
        obj.isMaximized = true;
        document.body.classList.add("window-maximized");
      }
      this.focus(id);
    }

    _makeDraggable(el) {
      const header = el.querySelector(".window-header-glass") || el.querySelector(".window-header");
      if (!header) return;
      let isDragging = false, startX, startY, initialLeft, initialTop;

      const onDown = (e) => {
        if (e.target.closest(".window-dot") || e.target.closest("button")) return;
        isDragging = true;
        startX = e.clientX; startY = e.clientY;
        initialLeft = parseInt(el.style.left || 0, 10);
        initialTop = parseInt(el.style.top || 0, 10);
        this.focus(el.id.replace("window-", ""));
        document.addEventListener("mousemove", onMove);
        document.addEventListener("mouseup", onUp);
      };
      const onMove = (e) => {
        if (!isDragging) return;
        requestAnimationFrame(() => {
          el.style.left = `${Math.max(-200, initialLeft + e.clientX - startX)}px`;
          el.style.top = `${Math.max(0, initialTop + e.clientY - startY)}px`;
        });
      };
      const onUp = () => {
        isDragging = false;
        document.removeEventListener("mousemove", onMove);
        document.removeEventListener("mouseup", onUp);
      };
      header.addEventListener("mousedown", onDown);
      header.addEventListener("dblclick", (e) => {
        if (e.target.closest(".window-dot") || e.target.closest("button")) return;
        this.toggleMaximize(el.id.replace("window-", ""));
      });
      el.addEventListener("mousedown", () => this.focus(el.id.replace("window-", "")));
    }

    _makeResizable(el) {
      el.style.resize = "both";
      el.style.overflow = "auto";
    }

    /* ─── START MENU ─── */
    toggleStartMenu() {
      if (this.startMenuOpen) {
        this.closeStartMenu();
      } else {
        this.openStartMenu();
      }
    }

    openStartMenu() {
      if (this.startMenuOpen) return;
      this.closeStartMenu();
      const overlay = document.createElement("div");
      overlay.id = "startMenuOverlay";
      overlay.className = "start-menu-overlay";
      overlay.onclick = (e) => { if (e.target === overlay) this.closeStartMenu(); };

      const menu = document.createElement("div");
      menu.id = "startMenu";
      menu.className = "start-menu";
      menu.innerHTML = this._buildStartMenuHTML();
      menu.addEventListener("click", (e) => e.stopPropagation());

      overlay.appendChild(menu);
      document.body.appendChild(overlay);
      document.body.classList.add("start-menu-open");
      this.startMenuOpen = true;

      const input = menu.querySelector("#startMenuSearchInput");
      if (input) {
        setTimeout(() => input.focus(), 100);
        input.addEventListener("input", () => this._filterStartMenu());
      }

      document.addEventListener("keydown", this._startMenuKeyHandler);
    }

    closeStartMenu() {
      const overlay = document.getElementById("startMenuOverlay");
      if (overlay) overlay.remove();
      document.body.classList.remove("start-menu-open");
      this.startMenuOpen = false;
      document.removeEventListener("keydown", this._startMenuKeyHandler);
    }

    _startMenuKeyHandler = (e) => {
      if (e.key === "Escape") this.closeStartMenu();
    };

    _buildStartMenuHTML() {
      const recent = this.getRecentApps();
      const categories = ["ai", "business", "office", "dev", "system"];

      // Filter apps by product config if available
      const enabledApps = window.productConfig && window.productConfig.apps
        ? new Set(window.productConfig.apps.map(function (a) { return a.toLowerCase(); }))
        : null;

      let html = `<div class="start-menu-search-wrap"><div class="start-menu-search">
        <svg viewBox="0 0 24 24" fill="none" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
        <input type="text" id="startMenuSearchInput" placeholder="Type to find an app..."/>
      </div></div>`;

      html += `<div class="start-menu-body" id="startMenuBody">`;

      if (recent.length > 0) {
        html += `<div class="start-menu-recent"><div class="start-menu-section-title">Recent</div><div class="start-menu-grid">`;
        recent.forEach(function (app) {
          if (!enabledApps || enabledApps.has(app.id)) {
            html += this._appTileHTML(app);
          }
        }.bind(this));
        html += `</div></div>`;
      }

      categories.forEach(function (cat) {
        var apps = APPS_REGISTRY.filter(function (a) { return a.category === cat; });
        if (enabledApps) {
          apps = apps.filter(function (a) { return enabledApps.has(a.id); });
        }
        if (!apps.length) return;
        html += `<div class="start-menu-category" data-category="${cat}"><div class="start-menu-section-title">${CATEGORY_LABELS[cat] || cat}</div><div class="start-menu-grid">`;
        apps.forEach(function (app) { html += this._appTileHTML(app); }.bind(this));
        html += `</div></div>`;
      }.bind(this));

      html += `<div id="startMenuEmpty" class="start-menu-filter-empty" style="display:none">No apps found</div>`;
      html += `</div>`;

      html += `<div class="start-menu-footer">
        <div class="start-menu-user"><div class="start-menu-avatar">U</div><div class="start-menu-user-name">User</div></div>
        <div class="start-menu-power" onclick="window.location.href='/suite/auth/login.html'">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="18" height="18"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><polyline points="16 17 21 12 16 7"/><line x1="21" y1="12" x2="9" y2="12"/></svg>
        </div>
      </div>`;

      return html;
    }

    _appTileHTML(app) {
      const iconColor = app.color || "#88ccff";
      return `<div class="start-menu-app" data-app-id="${app.id}" onclick="window.WindowManager.launchFromMenu('${app.id}', '${app.title.replace(/'/g, "\\'")}', '${app.hxGet}')">
        <div class="start-menu-app-icon" style="background:${iconColor}22;color:${iconColor}"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">${app.icon}</svg></div>
        <div class="start-menu-app-label">${app.title}</div>
      </div>`;
    }

    _filterStartMenu() {
      const input = document.getElementById("startMenuSearchInput");
      const query = (input ? input.value : "").toLowerCase();
      const body = document.getElementById("startMenuBody");
      if (!body) return;
      const tiles = body.querySelectorAll(".start-menu-app");
      const categories = body.querySelectorAll(".start-menu-category");
      const recent = body.querySelector(".start-menu-recent");
      const empty = document.getElementById("startMenuEmpty");
      let anyVisible = false;

      tiles.forEach((tile) => {
        const label = (tile.querySelector(".start-menu-app-label")?.textContent || "").toLowerCase();
        const match = !query || label.includes(query);
        tile.style.display = match ? "" : "none";
        if (match) anyVisible = true;
      });

      categories.forEach((cat) => {
        const visTiles = cat.querySelectorAll(".start-menu-app[style*='display: none'], .start-menu-app:not([style])");
        let allHidden = true;
        cat.querySelectorAll(".start-menu-app").forEach((t) => {
          if (t.style.display !== "none") allHidden = false;
        });
        cat.style.display = (query && allHidden) ? "none" : "";
      });

      if (recent) {
        let recentHidden = true;
        recent.querySelectorAll(".start-menu-app").forEach((t) => {
          if (t.style.display !== "none") recentHidden = false;
        });
        recent.style.display = (query && recentHidden) ? "none" : "";
      }

      if (empty) empty.style.display = anyVisible ? "none" : "";
    }

    launchFromMenu(id, title, hxGet) {
      this.closeStartMenu();
      this.open(id, title, "");
      fetch(hxGet).then((r) => r.text()).then((html) => {
        const body = document.getElementById(`window-body-${id}`);
        if (body) this._injectBodyContent(id, html);
      }).catch(() => {
        const body = document.getElementById(`window-body-${id}`);
        if (body) this._injectBodyContent(id, `<div style="padding:20px"><h3>${title}</h3><p>Application loading...</p></div>`);
      });
    }
  }

  window.WindowManager = new WindowManager();

  document.addEventListener("keydown", (e) => {
    if ((e.metaKey || e.ctrlKey) && e.key === "k") {
      e.preventDefault();
      window.WindowManager.toggleStartMenu();
    }
  });

  window.toggleChatSidebar = function () {
    const sidebar = document.getElementById("chatSidebar");
    if (sidebar) sidebar.classList.toggle("collapsed");
  };
}
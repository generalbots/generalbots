if (typeof window.WindowManager === "undefined") {
  "use strict";

  const APPS_REGISTRY = [
    { id: "admin", title: "Admin Panel", category: "system", color: "#ef4444", hxGet: "/suite/admin/index.html",
      icon: '<path d="M12 15v2m-6 4h12a2 2 0 0 0 2-2v-6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v6a2 2 0 0 0 2 2zm10-10V7a4 4 0 0 0-8 0v4h8z"/>' },
    { id: "vibe", title: "Vibe", category: "ai", color: "#84d669", hxGet: "/suite/partials/vibe.html",
      icon: '<path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"/>' },

    { id: "vibe-graph", title: "Knowledge Graph", category: "ai", color: "#7c3aed", hxGet: "/suite/vibe/graph.html",
      icon: '<circle cx="18" cy="5" r="3"/><circle cx="6" cy="12" r="3"/><circle cx="18" cy="19" r="3"/><line x1="8.59" y1="13.51" x2="15.42" y2="17.49"/><line x1="15.41" y1="6.51" x2="8.59" y2="10.49"/>' },
    { id: "vibe-metrics", title: "Vibe Metrics", category: "ai", color: "#f59e0b", hxGet: "/suite/vibe/metrics.html",
      icon: '<path d="M18 20V10"/><path d="M12 20V4"/><path d="M6 20v-6"/>' },
    { id: "vibe-members", title: "Project Members", category: "ai", color: "#06b6d4", hxGet: "/suite/vibe/members.html",
      icon: '<path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/>' },
    { id: "vibe-deploy", title: "Vibe Deploy", category: "dev", color: "#22c55e", hxGet: "/suite/vibe/deploy.html",
      icon: '<path d="M4 17l6-6-6-6"/><path d="M12 19h8"/>' },
    { id: "vibe-db", title: "Vibe Database", category: "dev", color: "#3b82f6", hxGet: "/suite/vibe/db.html",
      icon: '<ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/>' },
    { id: "vibe-metering", title: "Compute Metering", category: "system", color: "#f97316", hxGet: "/suite/vibe/metering.html",
      icon: '<path d="M12 1v22"/><path d="M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"/>' },
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
    { id: "chat", title: "Chat", category: "ai", color: "#84d669", hxGet: "/suite/partials/chat.html?v=4",
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
    { id: "browser", title: "Browser", category: "system", color: "#3b82f6", hxGet: "/suite/browser/browser.html?v=2",
      icon: '<circle cx="12" cy="12" r="10"/><polygon points="16.24 7.76 14.12 14.12 7.76 16.24 9.88 9.88 16.24 7.76"/>' },
    { id: "canvas", title: "Canvas", category: "office", color: "#0ea5e9", hxGet: "/suite/canvas/canvas.html",
      icon: '<rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 15l5-5 4 4 3-3 6 6"/><circle cx="8" cy="8" r="1.5"/>' },
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
    { id: "project", title: "Projects", category: "office", color: "#0ea5e9", hxGet: "/suite/project/project.html",
      icon: '<rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><line x1="3" y1="9" x2="21" y2="9"/><line x1="9" y1="21" x2="9" y2="9"/>' },
    { id: "calendar", title: "Calendar", category: "office", color: "#ec4899", hxGet: "/suite/calendar/calendar.html",
      icon: '<rect x="3" y="4" width="18" height="18" rx="2" ry="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/>' },
    { id: "billing", title: "Billing", category: "business", color: "#22c55e", hxGet: "/suite/billing/billing.html",
      icon: '<rect x="1" y="4" width="22" height="16" rx="2" ry="2"/><line x1="1" y1="10" x2="23" y2="10"/>' },
    { id: "products", title: "Products", category: "business", color: "#84d669", hxGet: "/suite/products/products.html",
      icon: '<path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/>' },
    { id: "research", title: "Research", category: "ai", color: "#8b5cf6", hxGet: "/suite/research/research.html",
      icon: '<circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>' },
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
    { id: "jukebox", title: "JukeBox", category: "ai", color: "#f59e0b", hxGet: "/suite/jukebox/jukebox.html",
      icon: '<path d="M3 12h2l2-7 4 14 4-14 2 7h4"/>' },
    { id: "vision", title: "Vision", category: "ai", color: "#06b6d4", hxGet: "/suite/vision/vision.html",
      icon: '<path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/>' },
    { id: "fraud", title: "Anti-Fraud", category: "business", color: "#ef4444", hxGet: "/suite/fraud/fraud.html",
      icon: '<path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/><line x1="9" y1="12" x2="15" y2="12"/>' },
    { id: "integrations", title: "Integrations", category: "dev", color: "#8b5cf6", hxGet: "/suite/integrations/integrations.html",
      icon: '<circle cx="18" cy="5" r="3"/><circle cx="6" cy="12" r="3"/><circle cx="18" cy="19" r="3"/><line x1="8.59" y1="13.51" x2="15.42" y2="17.49"/><line x1="15.41" y1="6.51" x2="8.59" y2="10.49"/>' },
    { id: "itsm", title: "ITSM", category: "dev", color: "#06b6d4", hxGet: "/suite/tickets/tickets.html",
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
    { id: "o365", title: "o365", category: "office", color: "#3b82f6", hxGet: "/suite/o365/o365.html",
      icon: '<rect x="2" y="2" width="9" height="9"/><rect x="13" y="2" width="9" height="9"/><rect x="2" y="13" width="9" height="9"/><rect x="13" y="13" width="9" height="9"/>' },
    { id: "learn", title: "Learn", category: "ai", color: "#84d669", hxGet: "/suite/learn/learn-app.html",
      icon: '<path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"/><path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"/>' },
    { id: "minutes", title: "Minutes", category: "office", color: "#8b5cf6", hxGet: "/suite/minutes/minutes.html",
      icon: '<path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/>' },
    { id: "sheet", title: "Sheets", category: "office", color: "#0f9d58", hxGet: "/suite/sheet/sheet.html",
      icon: '<rect x="3" y="3" width="18" height="18" rx="2"/><line x1="3" y1="9" x2="21" y2="9"/><line x1="9" y1="21" x2="9" y2="9"/>' },
    { id: "calculator", title: "Calculator", category: "system", color: "#0ea5e9", hxGet: "/suite/calculator/calculator.html",
      icon: '<rect x="5" y="2" width="14" height="20" rx="2"/><line x1="8" y1="6" x2="16" y2="6"/><line x1="8.01" y1="11" x2="8" y2="11"/><line x1="12.01" y1="11" x2="12" y2="11"/><line x1="16.01" y1="11" x2="16" y2="11"/><line x1="8.01" y1="15" x2="8" y2="15"/><line x1="12.01" y1="15" x2="12" y2="15"/><line x1="16.01" y1="15" x2="16" y2="15"/><line x1="8.01" y1="19" x2="8" y2="19"/><line x1="12.01" y1="19" x2="12" y2="19"/><line x1="16.01" y1="19" x2="16" y2="19"/>' },
    { id: "notepad", title: "Notepad", category: "office", color: "#f59e0b", hxGet: "/suite/notepad/notepad.html",
      icon: '<path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>' },
    { id: "snapshot", title: "Snapshot", category: "system", color: "#ec4899", hxGet: "/suite/snapshot/snapshot.html",
      icon: '<path d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z"/><circle cx="12" cy="13" r="4"/>' },
    { id: "clockapp", title: "Clock", category: "office", color: "#06b6d4", hxGet: "/suite/clock/clock.html",
      icon: '<circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/>' },
    { id: "store", title: "App Store", category: "system", color: "#3b82f6", hxGet: "/suite/store/store.html",
      icon: '<circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="16"/><line x1="8" y1="12" x2="16" y2="12"/>' },
    { id: "concierge", title: "Concierge", category: "ai", color: "#84d669", hxGet: "/suite/concierge/concierge.html",
      icon: '<path d="M12 2l2.4 4.9 5.4.8-3.9 3.8.9 5.4-4.8-2.5-4.8 2.5.9-5.4L4.2 7.7l5.4-.8z"/>' },
    { id: "notes", title: "Sticky Notes", category: "office", color: "#f59e0b", hxGet: "/suite/notes/notes.html",
      icon: '<path d="M4 4h16a1 1 0 0 1 1 1v11a1 1 0 0 1-1 1H9l-5 4V5a1 1 0 0 1 1-1z"/><line x1="8" y1="9" x2="16" y2="9"/><line x1="8" y1="13" x2="13" y2="13"/>' },
    { id: "photos", title: "Photos", category: "system", color: "#ec4899", hxGet: "/suite/photos/photos.html",
      icon: '<rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><path d="M21 15l-5-5L5 21"/>' },
    { id: "timer", title: "Timer", category: "office", color: "#06b6d4", hxGet: "/suite/timer/timer.html",
      icon: '<circle cx="12" cy="13" r="8"/><path d="M12 9v4l2 2"/><path d="M9 2h6"/>' },
    { id: "weather", title: "Weather", category: "system", color: "#0ea5e9", hxGet: "/suite/weather/weather.html",
      icon: '<path d="M17.5 19a4.5 4.5 0 1 0-.9-8.9 6 6 0 0 0-11.1 2.4A3.5 3.5 0 0 0 6 19z"/><line x1="12" y1="2" x2="12" y2="4"/><line x1="2" y1="12" x2="4" y2="12"/>' },
    { id: "recycle", title: "Recycle Bin", category: "system", color: "#64748b", hxGet: "/suite/recycle/recycle.html",
      icon: '<polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><line x1="10" y1="11" x2="10" y2="17"/><line x1="14" y1="11" x2="14" y2="17"/>' },
  ];

  window.APPS_REGISTRY = APPS_REGISTRY;

  // Load the authoritative catalog from the backend and MERGE it with the
  // embedded registry: backend entries win on id conflicts, while
  // embedded-only apps (true offline fallback, e.g. Calculator) survive a
  // catalog load. Launchers read window.APPS_REGISTRY.
  (function loadAppsCatalog() {
    fetch("/api/apps/catalog")
      .then(function (r) { if (!r.ok) throw new Error("catalog unavailable"); return r.json(); })
      .then(function (data) {
        if (!data || !Array.isArray(data.apps) || !data.apps.length) return;
        var merged = data.apps
          .filter(function (a) { return a.enabled !== false && a.compiled !== false; })
          .map(function (a) {
            return {
              id: a.id,
              title: a.title,
              category: a.category,
              color: a.color,
              hxGet: a.url,
              description: a.description,
              icon: a.icon,
              // #1289/#1291 — bot and vibe-app tiles deep-link their window
              // (chat bot binding, browser URL); launchFromMenu passes them
              // to openDeepLink.
              deep_link_params: a.deep_link_params || null,
            };
          });
        var known = {};
        merged.forEach(function (a) { known[a.id] = true; });
        APPS_REGISTRY.forEach(function (a) {
          if (!known[a.id]) merged.push(a);
        });
        window.APPS_REGISTRY = merged;
        window.dispatchEvent(
          new CustomEvent("gb-apps-catalog-loaded", {
            detail: { apps: merged },
          })
        );
      })
      .catch(function () { /* keep embedded fallback */ });
  })();


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
      const app = (window.APPS_REGISTRY || APPS_REGISTRY).find((a) => a.id === id);
      if (app) return `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">${app.icon}</svg>`;
      return '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/></svg>';
    }

    // Tool windows (VB6/Delphi-style floating palettes) pass opts.tool:
    // they get a slim title bar, NO status bar (no "client portion") and a
    // smaller default footprint. opts.popup makes an even smaller, compact
    // popup-sized tool window (e.g. the Vibe Members list).
    open(id, title, htmlContent, opts) {
      opts = opts || {};
      const existingWindow = this.openWindows.find((w) => w.id === id);
      if (existingWindow) {
        if (opts.ownerId) existingWindow.ownerId = opts.ownerId;
        this.focus(id);
        return;
      }
      // The user explicitly opened the Vibe workbench again (launcher,
      // command palette, deep link): forget the remembered closed state so
      // the next page load restores the bar instead of skipping it.
      if (id === "vibe") {
        try {
          localStorage.removeItem("gb.vibe.closed");
        } catch (e) {
          /* storage may be disabled — the open still works */
        }
      }

      const windowData = {
        id,
        title,
        ownerId: opts.ownerId || null,
        noMaximize: opts.noMaximize === true,
        isMinimized: false,
        isMaximized: false,
        previousState: null,
      };
      this.openWindows.push(windowData);

      const workspace = this.getWorkspace();
      // #1288 — windows open with a ~10% top margin instead of hugging the
      // title bar (the old 60px base stacked popups right under the chrome).
      // They still cascade BELOW/RIGHT of the compact Vibe bar (VB4
      // workbench) so a freshly opened Terminal/Browser never lands on top
      // of it and blocks its toolbar buttons.
      const wRect = workspace.getBoundingClientRect();
      const tenPct = Math.max(60, Math.round(wRect.height * 0.10));
      let topBase = tenPct;
      let leftBase = 180;
      const vibeEl = document.getElementById("window-vibe");
      if (vibeEl) {
        const vRect = vibeEl.getBoundingClientRect();
        topBase = Math.max(tenPct, vRect.bottom - wRect.top + 14);
        leftBase = Math.max(180, vRect.left - wRect.left + 12);
      }
      const offset = (this.openWindows.length * 28) % 140;
      const top = topBase + offset;
      const left = leftBase + offset;

      const windowEl = document.createElement("div");
      windowEl.id = `window-${id}`;
      windowEl.style.top = `${top}px`;
      windowEl.style.left = `${left}px`;
      windowEl.style.zIndex = this.zIndexCounter++;

      if (this.useGlassWindows) {
        windowEl.className = "window-element-glass";
        windowEl.innerHTML = this._glassHeader(id, title, opts) + this._glassBody(id);
      } else {
        windowEl.className = "window-element";
        windowEl.innerHTML = this._legacyHeader(id, title, opts) + this._legacyBody(id);
      }

      if (opts.tool) {
        windowEl.classList.add("window-tool");
        if (opts.popup) windowEl.classList.add("window-popup");
      } else if (!opts.noStatusBar) {
        // Issue #725: every app window gets an inverted 3D bevel status bar.
        // The Vibe workbench opts out: VB-style design, no status bar.
        windowEl.insertAdjacentHTML("beforeend", this._statusBar(id));
      }

      workspace.appendChild(windowEl);
      this._injectBodyContent(id, htmlContent);
      this._addTaskbarDockItem(id);
      this._makeDraggable(windowEl);
      this._makeResizable(windowEl);
      this.focus(id);
      if (!opts.tool) this._trackRecent(id, title);
      // The cascade position above is computed while the desktop may still
      // be laying out; an app auto-opened on load (e.g. ?app=vibe) can land
      // with a stale/negative offset that pushes it off-screen. Clamp the
      // window into the visible workspace so it can never be unreachable.
      this._clampWindowIntoView(id);
      document.dispatchEvent(new CustomEvent("gb-window-changed", { detail: { action: "open", id } }));
      if (window.htmx) htmx.process(windowEl);
      if (window.Desktop3D && window.Desktop3D.initialized) {
        window.Desktop3D.createWindowPlane(id, title);
        window.Desktop3D.flipToWindow(id);
      }
    }

    // Keep an open window inside the viewport. If layout shifted between
    // computing the cascade position and the window being appended
    // (auto-opened apps on load are the usual victim), the window can sit
    // off-screen (negative rect) even though its inline top/left look sane.
    // Shift it back into view using pure viewport-rect math: whatever the
    // offsetParent is doing, the window must end up fully visible.
    _clampWindowIntoView(id) {
      // A maximized window fills the workspace by design; never clamp it.
      if (document.body.classList.contains("window-maximized")) return;
      const win = document.getElementById(`window-${id}`);
      if (!win) return;
      const rect = win.getBoundingClientRect();
      const margin = 8;
      const vw = window.innerWidth;
      const vh = window.innerHeight;
      // #1278 — the real lower bound is the taskbar's top edge, not the
      // viewport bottom: a window whose footer slides under the taskbar
      // (e.g. the New Project popup after its content grows) leaves its
      // primary buttons unclickable. Measure the taskbar when present and
      // keep windows above it.
      let bottomBound = vh - margin;
      const taskbar = document.getElementById("taskbar");
      if (taskbar) {
        const tb = taskbar.getBoundingClientRect();
        if (tb.top > 0 && tb.top < vh) bottomBound = tb.top - margin;
      }
      // Already fully inside — nothing to do.
      if (rect.top >= margin && rect.left >= margin &&
          rect.bottom <= bottomBound && rect.right <= vw - margin) return;
      const curTop = parseInt(win.style.top || "0", 10) || 0;
      const curLeft = parseInt(win.style.left || "0", 10) || 0;
      let top = curTop;
      let left = curLeft;
      if (rect.top < margin) top = curTop + (margin - rect.top);
      if (rect.left < margin) left = curLeft + (margin - rect.left);
      if (rect.right > vw - margin) left = curLeft - (rect.right - (vw - margin));
      if (rect.bottom > bottomBound) top = curTop - (rect.bottom - bottomBound);
      win.style.top = `${Math.max(margin, top)}px`;
      win.style.left = `${Math.max(margin, left)}px`;
    }

    _glassHeader(id, title, opts) {
      const maximize = opts && opts.noMaximize !== true
        ? `<div class="window-dot window-dot-maximize" onclick="window.WindowManager.toggleMaximize('${id}')"></div>`
        : "";
      return `<div class="window-header-glass">
        <div class="window-title">${title}</div>
        <div class="window-dot-controls">
          <div class="window-dot window-dot-minimize" onclick="window.WindowManager.toggleMinimize('${id}')"></div>
          ${maximize}
          <div class="window-dot window-dot-close" onclick="window.WindowManager.close('${id}')"></div>
        </div>
      </div>`;
    }

    _glassBody(id) {
      return `<div id="window-body-${id}" class="window-body-glass"></div>`;
    }

    _legacyHeader(id, title, opts) {
      const maximize = opts && opts.noMaximize !== true
        ? `<button class="btn-maximize hover:text-gray-600" onclick="window.WindowManager.toggleMaximize('${id}')"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/></svg></button>`
        : "";
      return `<div class="window-header"><div class="font-mono text-xs font-bold text-brand-600 tracking-wide">${title}</div><div class="flex space-x-3 text-gray-400"><button class="btn-minimize hover:text-gray-600" onclick="window.WindowManager.toggleMinimize('${id}')"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="5" y1="12" x2="19" y2="12"/></svg></button>${maximize}<button class="btn-close hover:text-red-500" onclick="window.WindowManager.close('${id}')"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg></button></div></div>`;
    }

    _legacyBody(id) {
      return `<div id="window-body-${id}" class="window-body relative flex-1 overflow-y-auto"></div>`;
    }

    // Issue #725: an inverted 3D bevel status bar, styled to match the active
    // theme. Shows a small status glyph and the window title so every app
    // window carries a visible footer bar.
    _statusBar(id) {
      const title = ((this.openWindows.find((w) => w.id === id)) || {}).title || "";
      return `<div class="window-statusbar" data-window-id="${id}">
        <span class="window-statusbar-status">Ready</span>
        <span class="window-statusbar-spacer"></span>
        <span class="window-statusbar-title">${String(title).replace(/"/g, "&quot;")}</span>
      </div>`;
    }

    _injectBodyContent(id, htmlContent) {
      const body = document.getElementById(`window-body-${id}`);
      if (!body) return;
      body.dataset.windowId = id;
      const tempDiv = document.createElement("div");
      tempDiv.innerHTML = htmlContent;
      const scripts = Array.from(tempDiv.querySelectorAll("script")).map((s) => {
        const clone = document.createElement("script");
        Array.from(s.attributes).forEach((a) => clone.setAttribute(a.name, a.value));
        clone.textContent = s.textContent;
        // Dynamic scripts are async by default and execute in arbitrary
        // order, which breaks modules that depend on earlier ones (e.g.
        // vibe-dialog-*.js registering into vibe-dialogs.js). Force
        // insertion-order execution so the fragment's script order holds.
        if (clone.hasAttribute("src")) clone.async = false;
        s.remove();
        return clone;
      });
      // Vendor scripts define globals (Terminal, _amdLoaderGlobal, etc.).
      // Re-injecting them in a second window breaks with "already declared"
      // errors, so skip src-based scripts that were already loaded once.
      // NOTE: dedup is scoped PER WINDOW (not page-global) — app modules
      // (vibe-*, chat-*, etc.) bind listeners to the fresh injected DOM and
      // MUST re-run on every window open; only vendor bundles are skipped
      // after their first load anywhere.
      //
      // Loading gate: the fragment is hidden behind a spinner until its own
      // stylesheets have applied, so windows never flash unstyled markup
      // (FOUC). Apps without local stylesheets reveal immediately.
      body.classList.remove("gb-window-ready");
      body.innerHTML =
        '<div class="gb-window-loading" role="status" aria-label="Loading app">' +
        '<div class="gb-window-loading-spinner"></div></div>' +
        '<div class="gb-window-content"></div>';
      const content = body.querySelector(".gb-window-content");
      content.innerHTML = tempDiv.innerHTML;
      const VENDOR = /(xterm|vendor\/|amd-loader|monaco|@|three\.)/;
      window.__gbLoadedScripts = window.__gbLoadedScripts || {};
      scripts.forEach((s) => {
        const src = s.getAttribute("src");
        if (src && VENDOR.test(src)) {
          if (window.__gbLoadedScripts[src]) return;
          window.__gbLoadedScripts[src] = true;
        }
        content.appendChild(s);
      });
      if (window.htmx) htmx.process(content);
      this._revealWhenReady(body);
    }

    // Reveal the app body once every local stylesheet of the fragment has
    // loaded (or failed). A timeout guards apps whose CSS 404s or hangs.
    _revealWhenReady(body) {
      const links = Array.from(body.querySelectorAll('link[rel="stylesheet"]'));
      const reclampAfterReveal = () => {
        // #1278 — the open-time clamp runs before the app content settles;
        // a popup whose body grows after reveal (New Project modal) can end
        // up extending under the taskbar. Clamp once more now that the
        // window has its final height.
        const wid = body.dataset && body.dataset.windowId;
        if (wid && this._clampWindowIntoView) this._clampWindowIntoView(wid);
      };
      if (!links.length) {
        body.classList.add("gb-window-ready");
        reclampAfterReveal();
        return;
      }
      const sheetReady = (link) => {
        try {
          return !!(link.sheet && link.sheet.cssRules && link.sheet.cssRules.length);
        } catch (e) {
          return false; // cross-origin sheet: fall through to load event
        }
      };
      let remaining = links.length;
      let done = false;
      const finish = () => {
        if (done) return;
        done = true;
        clearTimeout(timer);
        body.classList.add("gb-window-ready");
        reclampAfterReveal();
      };
      const onOne = () => {
        remaining -= 1;
        if (remaining <= 0) finish();
      };
      const timer = setTimeout(finish, 3000);
      links.forEach((link) => {
        if (sheetReady(link)) {
          onOne();
          return;
        }
        link.addEventListener("load", onOne);
        link.addEventListener("error", onOne);
      });
    }

    _addTaskbarDockItem(id) {
      const center = this.getTaskbarCenter();
      if (!center) return;
      const app = (window.APPS_REGISTRY || APPS_REGISTRY).find((a) => a.id === id);
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
          const app = (window.APPS_REGISTRY || APPS_REGISTRY).find((a) => a.id === r.id);
          return app || { id: r.id, title: r.title, category: "recent", color: "#666", icon: "", hxGet: "" };
        });
      } catch (e) {
        return [];
      }
    }

    focus(id) {
      this.activeWindowId = id;
      const el = document.getElementById(`window-${id}`);
      if (el) {
        el.style.zIndex = this.zIndexCounter++;
        el.classList.add("window-focused");
      }
      this.openWindows.forEach((w) => {
        if (w.id !== id) {
          const other = document.getElementById(`window-${w.id}`);
          if (other) other.classList.remove("window-focused");
        }
      });

      const obj = this.openWindows.find((w) => w.id === id);
      if (obj) document.title = `${obj.title} - General Bots`;
      this._updateDockActive();
      document.dispatchEvent(new CustomEvent("gb-window-focus", { detail: obj || { id } }));
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
      // Remember an explicit close of the Vibe workbench (localStorage) so
      // the auto-open on load (desktop.html ?app=vibe / ?vibe=) does not
      // fight the user: close the bar once, reload keeps it closed; opening
      // it again clears the flag so the next reload restores it.
      if (id === "vibe" && this.getWindow(id)) {
        try {
          localStorage.setItem("gb.vibe.closed", "1");
        } catch (e) {
          /* storage may be disabled — the close still works for the session */
        }
      }
      // Close owned tool windows first. This gives an IDE workbench a real
      // parent/child lifecycle: closing Vibe cannot leave a Browser, dialog,
      // terminal or run dock orphaned behind it.
      const children = this.openWindows
        .filter((w) => w.ownerId === id)
        .map((w) => w.id);
      children.forEach((childId) => this.close(childId));

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
      document.dispatchEvent(new CustomEvent("gb-window-close", { detail: { id } }));
      document.dispatchEvent(new CustomEvent("gb-window-changed", { detail: { action: "close", id } }));
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
      if (!obj || obj.noMaximize) return;
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
        // Dragging a maximized window is nonsensical (it fills the screen),
        // so restore it to its previous size/position first — standard
        // desktop behavior.
        const id = el.id.replace("window-", "");
        const wd = this.openWindows.find((w) => w.id === id);
        if (wd && wd.isMaximized && !wd.noMaximize) this.toggleMaximize(id);
        isDragging = true;
        startX = e.clientX; startY = e.clientY;
        initialLeft = parseInt(el.style.left || 0, 10);
        initialTop = parseInt(el.style.top || 0, 10);
        this.focus(id);
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
        const id = el.id.replace("window-", "");
        const wd = this.openWindows.find((w) => w.id === id);
        if (wd && !wd.noMaximize) this.toggleMaximize(id);
      });
      el.addEventListener("mousedown", () => this.focus(el.id.replace("window-", "")));
    }

    _makeResizable(el) {
      // Fixed popup windows (e.g. Vibe New Project / Members) are NOT
      // resizable (product spec): no resize handle and no window scrollbar —
      // all fields are visible at once. Skip the CSS resize + auto overflow
      // so a popup never grows a corner grip or clips its content.
      if (el.classList.contains("window-popup")) {
        el.style.resize = "none";
        el.style.overflow = "hidden";
        return;
      }
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
        var apps = (window.APPS_REGISTRY || APPS_REGISTRY).filter(function (a) { return a.category === cat; });
        if (enabledApps) {
          apps = apps.filter(function (a) {
            // #1289/#1291 — dynamic catalog tiles (user bots `bot-*`,
            // vibe-published apps `vibeapp-*`) never appear in the static
            // product `apps=` list; their presence in the catalog already IS
            // the server's visibility decision, so exempt them here.
            if (a.id.startsWith("bot-") || a.id.startsWith("vibeapp-")) return true;
            return enabledApps.has(a.id);
          });
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
        <div class="start-menu-power" onclick="window.location.href=(window.GB_LOGIN_URL||'/login')">
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
      // #1289/#1291 — catalog tiles may carry deep-link params (chat bot
      // binding, vibe app URL). Route through openDeepLink so the params
      // reach the app window (__gbAppParams__ + gb:deep-link retarget).
      const app = (window.APPS_REGISTRY || APPS_REGISTRY).find((a) => a.id === id);
      if (app && app.deep_link_params && Object.keys(app.deep_link_params).length) {
        this.openDeepLink(id, app.deep_link_params);
        return;
      }
      const existed = this.getWindow(id) !== null;
      // The Vibe workbench keeps the glass chrome but no status bar.
      this.open(id, title, "", {
        noMaximize: id === "vibe" });
      // Never re-inject into an existing window: the app HTML declares
      // top-level consts (e.g. drive's API_BASE) and re-running it throws
      // "Identifier ... has already been declared".
      if (existed) return;
      const sep = hxGet.indexOf("?") === -1 ? "?" : "&";
      fetch(hxGet + sep + "_=" + Date.now()).then((r) => r.text()).then((html) => {
        const body = document.getElementById(`window-body-${id}`);
        if (body) this._injectBodyContent(id, html);
      }).catch(() => {
        const body = document.getElementById(`window-body-${id}`);
        if (body) this._injectBodyContent(id, `<div style="padding:20px"><h3>${title}</h3><p>Application loading...</p></div>`);
      });
    }

    // Deep-link: open an app window contextualized with query params. The app
    // HTML receives the params both as URL query args and via
    // window.__gbAppParams__ so it can select/filter the referenced record.
    // Only available in the desktop shell (web channel).
    openDeepLink(appId, params, opts) {
      opts = opts || {};
      const app = (window.APPS_REGISTRY || APPS_REGISTRY).find((a) => a.id === appId);
      const title = app ? app.title : appId;
      const hxGet = app ? app.hxGet : `/suite/partials/${appId}.html`;
      this.closeStartMenu();
      const existed = this.getWindow(appId) !== null;
      // The Vibe workbench keeps the glass chrome but no status bar.
      this.open(appId, title, "", {
        noMaximize: appId === "vibe",
        ownerId: opts.ownerId || null,
      });
      window.__gbAppParams__ = Object.assign({}, window.__gbAppParams__ || {}, params || {});
      if (existed) {
        // #1288 — a re-targeted window must adopt the caller's ownership
        // (e.g. the Vibe toolbar opening the shared Terminal/Browser/Chat):
        // a window first opened without an owner (desktop sidebar) would
        // otherwise never be closable by its new owner's cleanup.
        if (opts.ownerId) {
          const rec = this.openWindows.find((w) => w.id === appId);
          if (rec) rec.ownerId = opts.ownerId;
        }
        // Re-target an already-open app window (new project URL, session, ...)
        // so deep-links always apply, not only on first open.
        document.dispatchEvent(new CustomEvent("gb:deep-link", {
          detail: { appId, params: params || {} },
        }));
        return;
      }
      const qs = Object.keys(params || {}).map((k) => `${encodeURIComponent(k)}=${encodeURIComponent(params[k])}`).join("&");
      const sep = hxGet.indexOf("?") === -1 ? "?" : "&";
      fetch(hxGet + sep + (qs ? qs + "&" : "") + "_=" + Date.now()).then((r) => r.text()).then((html) => {
        const body = document.getElementById(`window-body-${appId}`);
        if (body) this._injectBodyContent(appId, html);
        // Content can push the workspace layout; re-clamp so an auto-opened
        // app never sits off-screen after its body settles.
        this._clampWindowIntoView(appId);
      }).catch(() => {
        const body = document.getElementById(`window-body-${appId}`);
        if (body) this._injectBodyContent(appId, `<div style="padding:20px"><h3>${title}</h3><p>Application loading...</p></div>`);
      });
    }

    // VB6/Adobe-style floating tool window: open (or focus) a window under a
    // unique id and fetch a partial into its body. Unlike openDeepLink, the
    // id is caller-controlled (e.g. "vibe-run", "vibe-terminal") so several
    // accessory windows can float beside the main app window.
    openToolWindow(id, title, hxGet, params, opts) {
      this.closeStartMenu();
      const existed = this.getWindow(id) !== null;
      opts = opts || {};
      this.open(id, title, "", { tool: true, ownerId: opts.ownerId || null, noMaximize: opts.noMaximize === true });
      if (existed) return;
      window.__gbAppParams__ = Object.assign({}, window.__gbAppParams__ || {}, params || {});
      const qs = Object.keys(params || {}).map((k) => `${encodeURIComponent(k)}=${encodeURIComponent(params[k])}`).join("&");
      const sep = hxGet.indexOf("?") === -1 ? "?" : "&";
      fetch(hxGet + sep + (qs ? qs + "&" : "") + "_=" + Date.now())
        .then((r) => r.text())
        .then((html) => {
          const body = document.getElementById(`window-body-${id}`);
          if (body) this._injectBodyContent(id, html);
        })
        .catch(() => {
          const body = document.getElementById(`window-body-${id}`);
          if (body) this._injectBodyContent(id, `<div style="padding:20px"><h3>${title}</h3><p>Tool window failed to load.</p></div>`);
        });
    }

    // Open (or focus) a tool window and return its body element so callers can
    // build DOM directly (dialog modules, panels, modals). Content is cleared
    // on every call so re-opening a dialog never stacks stale markup.
    openToolWindowBody(id, title, opts) {
      opts = opts || {};
      this.closeStartMenu();
      const existed = this.getWindow(id) !== null;
      this.open(id, title, "", {
        tool: true,
        popup: !!opts.popup,
        ownerId: opts.ownerId || null,
        noMaximize: opts.noMaximize === true,
      });
      const body = document.getElementById(`window-body-${id}`);
      if (!existed && body && opts.htmlContent) {
        this._injectBodyContent(id, opts.htmlContent);
        return body;
      }
      return body;
    }

    // VB6-style floating confirmation (no native confirm/alert modals).
    // Opens a small tool window; onYes/onNo are called on button press.
    confirmFloating(title, message, onYes, onNo, yesLabel) {
      const html =
        '<div class="gb-confirm-floating">' +
        "<p>" + String(message == null ? "" : message) + "</p>" +
        '<div class="gb-confirm-actions">' +
        '<button data-c-no class="gb-confirm-btn">Cancel</button>' +
        '<button data-c-yes class="gb-confirm-btn primary">' + (yesLabel || "OK") + "</button>" +
        "</div></div>";
      const body = this.openToolWindowBody("gb-confirm", title || "Confirm", { htmlContent: html });
      if (!body) return;
      body.querySelector("[data-c-no]").addEventListener("click", () => {
        this.close("gb-confirm");
        if (onNo) onNo();
      });
      body.querySelector("[data-c-yes]").addEventListener("click", () => {
        this.close("gb-confirm");
        if (onYes) onYes();
      });
    }

    // VB6-style floating text input (replaces native window.prompt).
    promptFloating(title, message, defaultValue, onOk) {
      const safe = String(defaultValue == null ? "" : defaultValue);
      const html =
        '<div class="gb-confirm-floating">' +
        (message ? "<p>" + String(message) + "</p>" : "") +
        '<input type="text" class="gb-prompt-input" value="' + safe.replace(/"/g, "&quot;") + '" />' +
        '<div class="gb-confirm-actions">' +
        '<button data-c-no class="gb-confirm-btn">Cancel</button>' +
        '<button data-c-yes class="gb-confirm-btn primary">OK</button>' +
        "</div></div>";
      const body = this.openToolWindowBody("gb-prompt", title || "Input", { htmlContent: html });
      if (!body) return;
      const input = body.querySelector(".gb-prompt-input");
      input.focus();
      input.select();
      const finish = (value) => {
        this.close("gb-prompt");
        if (onOk) onOk(value);
      };
      body.querySelector("[data-c-no]").addEventListener("click", () => finish(null));
      body.querySelector("[data-c-yes]").addEventListener("click", () => finish(input.value.trim()));
      input.addEventListener("keydown", (e) => {
        if (e.key === "Enter") finish(input.value.trim());
        if (e.key === "Escape") finish(null);
      });
    }

    // Issue #1160: isolated launch — open the app in its own top-level tab
    // with a fresh context. Deep-link params travel via the URL query string.
    openIsolated(appId, params) {
      const app = (window.APPS_REGISTRY || APPS_REGISTRY).find((a) => a.id === appId);
      let url = app ? app.hxGet : `/suite/partials/${appId}.html`;
      const qs = Object.keys(params || {}).map((k) => `${encodeURIComponent(k)}=${encodeURIComponent(params[k])}`).join("&");
      const sep = url.indexOf("?") === -1 ? "?" : "&";
      url = url + sep + (qs ? qs + "&" : "") + "isolated=1";
      window.open(url, "_blank", "noopener");
    }

    // Registry lookup helper shared by launchers and the widget pane.
    getApp(appId) {
      return (window.APPS_REGISTRY || APPS_REGISTRY).find((a) => a.id === appId) || null;
    }

    /* ─── Multitasking helpers (#1155) ─── */
    listWindows() {
      return this.openWindows.slice();
    }

    getWindow(id) {
      return this.openWindows.find((w) => w.id === id) || null;
    }

    minimizeWindow(id) {
      this.toggleMinimize(id);
    }

    maximizeWindow(id) {
      this.toggleMaximize(id);
    }

    focusWindow(id) {
      this.focus(id);
    }

    restoreWindow(id) {
      const obj = this.openWindows.find((w) => w.id === id);
      if (!obj) return;
      if (obj.isMinimized) this.toggleMinimize(id);
      if (obj.isMaximized) this.toggleMaximize(id);
      if (obj.snapLayout) {
        const el = document.getElementById(`window-${id}`);
        if (el && obj.previousState) {
          el.style.left = obj.previousState.left;
          el.style.top = obj.previousState.top;
          el.style.width = obj.previousState.width;
          el.style.height = obj.previousState.height;
          delete obj.snapLayout;
        }
      }
      this.focus(id);
    }

    // Snap-assist geometry (#1155): lay the window out per a named layout
    // (halves, quarters, thirds) relative to the viewport.
    snapWindow(id, layout) {
      const obj = this.openWindows.find((w) => w.id === id);
      if (!obj) return;
      const el = document.getElementById(`window-${id}`);
      if (!el) return;
      const L = {
        left: { x: 0, y: 0, w: 0.5, h: 1 },
        right: { x: 0.5, y: 0, w: 0.5, h: 1 },
        top: { x: 0, y: 0, w: 1, h: 0.5 },
        bottom: { x: 0, y: 0.5, w: 1, h: 0.5 },
        "top-left": { x: 0, y: 0, w: 0.5, h: 0.5 },
        "top-right": { x: 0.5, y: 0, w: 0.5, h: 0.5 },
        "bottom-left": { x: 0, y: 0.5, w: 0.5, h: 0.5 },
        "bottom-right": { x: 0.5, y: 0.5, w: 0.5, h: 0.5 },
        "third-left": { x: 0, y: 0, w: 1 / 3, h: 1 },
        "third-right": { x: 1 / 3, y: 0, w: 2 / 3, h: 1 },
      };
      const l = L[layout];
      if (!l) return;
      if (obj.isMaximized) this.toggleMaximize(id);
      const pad = 4;
      el.style.left = `${l.x * window.innerWidth + pad}px`;
      el.style.top = `${l.y * window.innerHeight + pad}px`;
      el.style.width = `${l.w * window.innerWidth - pad * 2}px`;
      el.style.height = `${l.h * window.innerHeight - pad * 2}px`;
      el.style.borderRadius = "0";
      obj.isMaximized = false;
      obj.snapLayout = layout;
      obj.previousState = { width: el.style.width, height: el.style.height, top: el.style.top, left: el.style.left };
      this.focus(id);
    }
  }

  window.WindowManager = new WindowManager();
  window.openDeepLink = (appId, params, opts) => window.WindowManager.openDeepLink(appId, params, opts);

  // Ctrl+K is owned by the unified command palette (command-palette.js) which
  // is loaded after this file. Expose the start menu for launchers but do NOT
  // bind Ctrl+K here — command-palette.js handles it with one handler only.

  // Chat sidebar collapse — the choice is remembered (localStorage) so the
  // sidebar stays collapsed across reloads and app switches. The collapsed
  // rail (51px) keeps the toggle button reachable.
  const SIDEBAR_STATE_KEY = "gb.chatSidebar.collapsed";

  window.toggleChatSidebar = function () {
    const sidebar = document.getElementById("chatSidebar");
    if (!sidebar) return;
    const collapsed = sidebar.classList.toggle("collapsed");
    try {
      localStorage.setItem(SIDEBAR_STATE_KEY, collapsed ? "1" : "0");
    } catch (e) {
      /* storage may be disabled; the toggle still works for the session */
    }
  };

  // Restore the remembered state on boot. window-manager.js loads in the
  // <head>, before desktop.html's sidebar exists, so defer until the DOM is
  // ready — and also re-apply after HTMX swaps that re-create the shell.
  function restoreChatSidebar() {
    const sidebar = document.getElementById("chatSidebar");
    if (!sidebar) return;
    let collapsed = "";
    try {
      collapsed = localStorage.getItem(SIDEBAR_STATE_KEY) || "";
    } catch (e) {
      /* storage unavailable — leave default */
    }
    sidebar.classList.toggle("collapsed", collapsed === "1");
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () {
      // Apply the remembered state on first paint AND keep it applied across
      // later HTMX swaps that re-create the shell. The sidebar lives in the
      // initial desktop.html markup (not a swapped partial), so restoring only
      // on htmx:afterSwap misses loads where no swap occurs.
      restoreChatSidebar();
      hookChatSidebarAfterSwap();
    });
  } else {
    restoreChatSidebar();
    hookChatSidebarAfterSwap();
  }
  // window-manager.js loads in the <head>, before <body> exists, so attach
  // the htmx listener only once the body element is present (binding it at
  // parse time would throw a null addEventListener error in the console).
  function hookChatSidebarAfterSwap() {
    if (window.htmx && document.body) {
      document.body.addEventListener("htmx:afterSwap", restoreChatSidebar);
      return;
    }
    if (document.readyState !== "loading") document.addEventListener("DOMContentLoaded", hookChatSidebarAfterSwap);
  }
}

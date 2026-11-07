/* WindowManager — app registry, preview gating and catalog (1/7). */
(function (window) {
  "use strict";
  if (typeof window.WindowManager !== "undefined" || window.__GBWindowManager) return;
  var WM = (window.__GBWindowManager = {});

const APPS_REGISTRY = [
  { id: "admin", title: "Settings", category: "system", color: "#64748b", hxGet: "/suite/settings/index.html?view=admin",
    icon: '<path d="M12 15v2m-6 4h12a2 2 0 0 0 2-2v-6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v6a2 2 0 0 0 2 2zm10-10V7a4 4 0 0 0-8 0v4h8z"/>' },
  { id: "settings", title: "Settings", category: "system", color: "#64748b", hxGet: "/suite/settings/index.html",
    icon: '<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>' },
  { id: "about", title: "About", category: "system", color: "#64748b", hxGet: "/suite/about/about.html",
    icon: '<circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/>' },
  { id: "vibe", title: "Vibe", category: "ai", color: "#84d669", hxGet: "/suite/partials/vibe.html",
    icon: '<path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"/>' },
  // #1546 — the Vibe sub-surfaces (Knowledge Graph, Vibe Metrics, Project
  // Members, Vibe Deploy, Vibe Database, Compute Metering) are NOT apps.
  // They are toolwindow views inside the Vibe window (vibe-windows.js /
  // vibe-dialogs.js); no standalone registry entry ships here so they can
  // never leak into a launcher (start menu, sidebar rail, palette).
  { id: "crm", title: "CRM", category: "business", color: "#3b82f6", hxGet: "/suite/crm/crm.html",
    icon: '<path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/>' },
  { id: "campaigns", title: "Campaigns", category: "business", color: "#f59e0b", hxGet: "/suite/campaigns/campaigns.html",
    icon: '<path d="M22 12h-4l-3 9L9 3l-3 9H2"/>' },
  { id: "lists", title: "Lists", category: "business", color: "#8b5cf6", hxGet: "/suite/lists/lists.html",
    icon: '<line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><line x1="3" y1="6" x2="3.01" y2="6"/><line x1="3" y1="12" x2="3.01" y2="12"/><line x1="3" y1="18" x2="3.01" y2="18"/>' },
  { id: "templates", title: "Templates", category: "office", color: "#ec4899", hxGet: "/suite/templates/templates.html",
    icon: '<rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><line x1="3" y1="9" x2="21" y2="9"/><line x1="9" y1="21" x2="9" y2="9"/>' },
  { id: "tasks", title: "Tasks", category: "office", color: "#22c55e", hxGet: "/suite/tasks/tasks.html",
    launcher_default: true,
    icon: '<path d="M9 11l3 3L22 4"/><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/>' },
  { id: "chat", title: "Chat", category: "ai", color: "#84d669", hxGet: "/suite/partials/chat.html?v=87",
    icon: '<path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>' },
  { id: "terminal", title: "Terminal", category: "dev", color: "#64748b", hxGet: "/suite/partials/terminal.html",
    icon: '<polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/>' },
  { id: "preview", title: "Preview", category: "tools", color: "#0ea5e9", hxGet: "/suite/partials/preview.html",
    width: 780, height: 640,
    icon: '<path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/>' },
  { id: "drive", title: "Explorer", category: "system", color: "#f59e0b", hxGet: "/suite/drive/drive.html",
    icon: '<path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>' },
  { id: "editor", title: "Editor", category: "dev", color: "#3b82f6", hxGet: "/suite/editor.html",
    icon: '<polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/>' },
  { id: "designer", title: "Designer", category: "office", color: "#ec4899", hxGet: "/suite/designer.html",
    icon: '<path d="M12 19l7-7 3 3-7 7-3-3z"/><path d="M18 13l-1.5-7.5L2 2l3.5 14.5L13 18l5-5z"/><path d="M2 2l7.586 7.586"/><circle cx="11" cy="11" r="2"/>' },
  { id: "browser", title: "Browser", category: "system", color: "#3b82f6", hxGet: "/suite/browser/browser.html?v=2",
    icon: '<circle cx="12" cy="12" r="10"/><polygon points="16.24 7.76 14.12 14.12 7.76 16.24 9.88 9.88 16.24 7.76"/>' },
  { id: "canvas", title: "Canvas", category: "office", color: "#0ea5e9", hxGet: "/suite/canvas/canvas.html",
    icon: '<rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 15l5-5 4 4 3-3 6 6"/><circle cx="8" cy="8" r="1.5"/>' },
  { id: "database", title: "Database", category: "dev", color: "#f59e0b", hxGet: "/suite/database/database.html",
    icon: '<ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/>' },
  { id: "desktop", title: "Desktop", category: "system", color: "#06b6d4", hxGet: "/suite/desktop/vdi.html",
    icon: '<rect x="2" y="3" width="20" height="14" rx="2" ry="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/>' },
  { id: "player", title: "Player", category: "system", color: "#8b5cf6", hxGet: "/suite/player/player.html",
    icon: '<polygon points="5 3 19 12 5 21 5 3"/>' },
  { id: "mail", title: "Mail", category: "office", color: "#3b82f6", hxGet: "/suite/mail/mail.html",
    icon: '<path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"/><polyline points="22,6 12,13 2,6"/>' },
  { id: "meet", title: "Meet", category: "office", color: "#ef4444", hxGet: "/suite/meet/meet.html",
    icon: '<polygon points="23 7 16 12 23 17 23 7"/><rect x="1" y="5" width="15" height="14" rx="2" ry="2"/>' },
  { id: "docs", title: "Docs", category: "office", color: "#3b82f6", hxGet: "/suite/docs/docs.html",
    icon: '<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/>' },
  { id: "plan", title: "Plan", category: "office", color: "#f59e0b", hxGet: "/suite/plan/plan.html",
    icon: '<rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/><line x1="10" y1="6" x2="14" y2="6"/><line x1="6" y1="10" x2="6" y2="14"/><line x1="10" y1="18" x2="14" y2="18"/><line x1="18" y1="10" x2="18" y2="14"/>' },
  { id: "goals", title: "Goals", category: "office", color: "#22c55e", hxGet: "/suite/goals/goals.html",
    icon: '<path d="M22 12h-4l-3 9L9 3l-3 9H2"/>' },
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
  { id: "plugins", title: "Plugins", category: "dev", color: "#8b5cf6", hxGet: "/suite/plugins/plugins.html",
    icon: '<path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/>' },
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

// Preview mode (#1348). Applications listed under `preview_apps` in the
// `.product` file are withheld from every launcher — the start menu, the apps
// menu and the sidebar all read APPS_REGISTRY — until the user turns the
// Preview switch on. The preference is per browser, like the other sidebar
// switches.
var PREVIEW_KEY = "gb-preview-mode";
var GBPreviewMode = {
  isOn: function () {
    try {
      return localStorage.getItem(PREVIEW_KEY) === "1";
    } catch (e) {
      return false;
    }
  },
  set: function (on) {
    try {
      localStorage.setItem(PREVIEW_KEY, on ? "1" : "0");
    } catch (e) {
      /* storage unavailable — the switch still applies to this page */
    }
    refreshAppsCatalog();
    window.dispatchEvent(new CustomEvent("gb-preview-changed", { detail: { preview: !!on } }));
  },
};

// State the rebuild reads: the embedded list (offline fallback), the
// authoritative catalog once it answers, and the preview identifiers the
// product file declares.
var embeddedApps = APPS_REGISTRY.slice();
var catalogApps = null;
var catalogLoaded = false;
var productPreviewIds = {};
// #1841 — Terminal needs both the PTY routes and the agent VM. The product
// endpoint reports the combined capability; null means "not loaded yet".
var productTerminalAvailable = null;
// Apps explicitly listed in `.product`'s `apps=` line. They stay visible
// even when they are not part of the always-on core set (weather, notes,
// calculator …), because the product owner enabled them on purpose.
var productAppIds = {};

// #1811 — only core surfaces are hardcoded here. Everything else is classified
// by `.product`'s `apps` / `preview_apps` / `hidden_apps` lists, which the
// catalog endpoint mirrors. Mirrors `CORE_APPS` in botserver/src/apps/mod.rs.
var CORE_APP_IDS = ["about", "admin", "auth", "settings"];
function isCoreAppId(id) {
  return CORE_APP_IDS.indexOf(id) !== -1;
}

function isPreviewId(id) {
  if (isCoreAppId(id)) return false;
  if (productPreviewIds[id]) return true;
  if (!catalogApps) return false;
  for (var i = 0; i < catalogApps.length; i++) {
    if (catalogApps[i].id === id && catalogApps[i].preview) return true;
  }
  return false;
}

// The catalog speaks its own shape (`url`, `enabled`, `compiled`); the
// embedded list is already in registry shape.
function toRegistryApp(a) {
  if (a.hxGet !== undefined) return a;
  return {
    id: a.id,
    title: a.title,
    category: a.category,
    color: a.color,
    hxGet: a.url,
    description: a.description,
    icon: a.icon,
    // #1386c — launcher visibility flag; surfaces that list apps to open
    // (start menu, sidebar rail) honour it, programmatic openers do not.
    launcher_default: a.launcher_default !== false,
    // Catalog entries carry the authoritative enabled/preview/hidden flags
    // from the product file; the embedded fallback has no such fields.
    enabled: a.enabled !== false,
    hidden: a.hidden === true,
    // #1289/#1291 — bot and vibe-app tiles deep-link their window (chat bot
    // binding, browser URL); launchFromMenu passes them to openDeepLink.
    deep_link_params: a.deep_link_params || null,
  };
}

function rebuildAppsRegistry() {
  var previewOn = GBPreviewMode.isOn();
  var base = catalogApps && catalogApps.length ? catalogApps : embeddedApps;
  var merged = [];
  var known = {};

  base.forEach(function (a) {
    if (a.compiled === false || a.hidden === true) return;
    if (typeof window.isAppHidden === "function" && window.isAppHidden(a.id)) return;
    var preview = !!a.preview || isPreviewId(a.id);
    var core = isCoreAppId(a.id);
    // A preview application is surfaced only while Preview mode is on. Core
    // surfaces are the only hardcoded exception; everything else follows the
    // product profile.
    if (preview && !previewOn && !core) return;
    if (!preview && a.hxGet === undefined && a.enabled === false) return;
    var app = toRegistryApp(a);
    app.preview = preview;
    merged.push(app);
    known[app.id] = true;
  });

  // Embedded-only tiles (the true offline fallback, e.g. Calculator) survive
  // a catalog load — but never a preview app the switch is holding back, nor
  // a hidden app. Core surfaces (#1811) surface even when the catalog has
  // loaded but omits them (e.g. before the backend catalog catches up).
  // #1429 — Vibe sub-surface toolwindows (`vibe-*`) are views inside the
  // single Vibe app, not catalog ids: they survive the catalog load so
  // Vibe's toolbar and deep links keep resolving them (they stay out of
  // every launcher via the toolwindow / `vibe-` prefix filters).
  embeddedApps.forEach(function (a) {
    if (known[a.id]) return;
    if (typeof window.isAppHidden === "function" && window.isAppHidden(a.id)) return;
    // A catalog entry marked `compiled: false` is authoritative even for the
    // always-on list: re-adding it put Terminal in builds that do not ship
    // the terminal routes (#1841).
    if (catalogLoaded && catalogApps) {
      for (var c = 0; c < catalogApps.length; c++) {
        if (catalogApps[c].id === a.id && catalogApps[c].compiled === false) return;
      }
    }
    if (catalogLoaded && !isCoreAppId(a.id) && String(a.id).indexOf("vibe") !== 0) return;
    var preview = isPreviewId(a.id);
    if (preview && !previewOn) return;
    a.preview = preview;
    merged.push(a);
    known[a.id] = true;
  });

  // The product result also wins over the embedded fallback when the catalog
  // endpoint is unavailable: never offer Terminal in a build that reports it
  // as unavailable.
  if (productTerminalAvailable === false) {
    merged = merged.filter(function (app) { return app.id !== "terminal"; });
  }

  // #1544: Drive belongs to authenticated users. Anonymous visitors keep
  // chat but must not see the Drive entry point (the API is also guarded
  // server-side). The no-security desktop build is exempt.
  var noSecurityBuild = !!(window.productConfig && window.productConfig.no_security === true);
  var authenticated = false;
  try {
    authenticated = !!(
      (window.GBSecurity && GBSecurity.isAuthenticated && GBSecurity.isAuthenticated()) ||
      localStorage.getItem("gb-access-token") ||
      sessionStorage.getItem("gb-access-token")
    );
  } catch (e) {}
  if (!authenticated && !noSecurityBuild) {
    // Anonymous visitors get the conversation only: no Drive, no workspace
    // shell, no admin surface (the APIs already answer 401).
    var privileged = ["drive", "terminal", "editor", "browser", "integrations", "admin"];
    merged = merged.filter(function (app) {
      return privileged.indexOf(app.id) === -1;
    });
  }

  // #1539/#1540: enforce the core set even before the server catalog loads
  // (the embedded fallback otherwise kept every app visible).
  if (!previewOn) {
    // Keep the always-on core, every app the product owner enabled, and
    // catalog entries the backend marked enabled. The old core-only filter
    // hid product-enabled apps such as Weather until Preview mode was on.
    merged = merged.filter(function (app) {
      return isCoreAppId(app.id) ||
        productAppIds[app.id] === true ||
        app.enabled === true;
    });
  }

  window.APPS_REGISTRY = merged;
  if (catalogLoaded) window.__gbAppsCatalogLoaded = true;
  window.dispatchEvent(
    new CustomEvent("gb-apps-catalog-loaded", {
      detail: { apps: merged, preview: previewOn },
    })
  );
}

function refreshAppsCatalog() {
  rebuildAppsRegistry();
}


// The public product endpoint carries `preview_apps`, so the switch holds
// preview applications back even when the authenticated catalog cannot
// answer (anonymous desktop): the embedded registry is all there is then.
(function loadProductPreviewList() {
  fetch("/api/product")
    .then(function (r) { return r.ok ? r.json() : null; })
    .then(function (data) {
      if (!data) return;
      if (Array.isArray(data.preview_apps)) {
        productPreviewIds = {};
        data.preview_apps.forEach(function (id) {
          productPreviewIds[String(id)] = true;
        });
      }
      if (Array.isArray(data.apps)) {
        productAppIds = {};
        data.apps.forEach(function (id) {
          productAppIds[String(id)] = true;
        });
      }
      if (typeof data.terminal_available === "boolean") {
        productTerminalAvailable = data.terminal_available;
      }
      rebuildAppsRegistry();
    })
    .catch(function () { /* embedded list stays authoritative */ });
})();

// Load the authoritative catalog from the backend and MERGE it with the
// embedded registry: backend entries win on id conflicts, while
// embedded-only apps (true offline fallback, e.g. Calculator) survive a
// catalog load. Launchers read window.APPS_REGISTRY.
(function loadAppsCatalog() {
  fetch("/api/apps/catalog")
    .then(function (r) { if (!r.ok) throw new Error("catalog unavailable"); return r.json(); })
    .then(function (data) {
      if (!data || !Array.isArray(data.apps) || !data.apps.length) return;
      catalogApps = data.apps;
      catalogLoaded = true;
      rebuildAppsRegistry();
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

  WM.APPS_REGISTRY = APPS_REGISTRY;
  WM.CATEGORY_LABELS = CATEGORY_LABELS;
  WM.GBPreviewMode = GBPreviewMode;
  WM.refreshAppsCatalog = refreshAppsCatalog;
  WM.isCoreAppId = isCoreAppId;
  WM.isPreviewId = isPreviewId;
})(window);

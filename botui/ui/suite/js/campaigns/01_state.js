"use strict";

window.CampStudio = window.CampStudio || {};

window.CampStudio.state = {
  channel: "email",
  device: "desktop",
  editingId: null,
  content: "",
  campaigns: [],
  igCampaigns: [],
  monitorFilter: "all",
};

window.CampStudio.api = {
  baseUrl: "/api/crm/campaigns",
  instagramUrl: "/api/instagram/campaigns",

  async list() {
    const resp = await fetch(this.baseUrl);
    if (!resp.ok) throw new Error("Failed to list campaigns: " + resp.status);
    return resp.json();
  },

  async get(id) {
    const resp = await fetch(this.baseUrl + "/" + id);
    if (!resp.ok) throw new Error("Failed to get campaign: " + resp.status);
    return resp.json();
  },

  async create(data) {
    const resp = await fetch(this.baseUrl, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(data),
    });
    if (!resp.ok) throw new Error("Failed to create campaign: " + resp.status);
    return resp.json();
  },

  async update(id, data) {
    const resp = await fetch(this.baseUrl + "/" + id, {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(data),
    });
    if (!resp.ok) throw new Error("Failed to update campaign: " + resp.status);
    return resp.json();
  },

  async remove(id) {
    const resp = await fetch(this.baseUrl + "/" + id, { method: "DELETE" });
    if (!resp.ok) throw new Error("Failed to delete campaign: " + resp.status);
  },

  async listInstagram() {
    try {
      const resp = await fetch(this.instagramUrl);
      if (!resp.ok) return { campaigns: [] };
      const data = await resp.json();
      return Array.isArray(data) ? { campaigns: data } : data;
    } catch (e) {
      return { campaigns: [] };
    }
  },

  async createInstagram(prompt, numImages, schedule) {
    const resp = await fetch(this.instagramUrl + "/create", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ prompt: prompt, num_images: numImages, scheduled_at: schedule }),
    });
    if (!resp.ok) throw new Error("Failed to create Instagram campaign: " + resp.status);
    return resp.json();
  },
};

window.CampStudio.channels = {
  email: { label: "Email", emoji: "📧" },
  whatsapp: { label: "WhatsApp", emoji: "💬" },
  instagram: { label: "Instagram", emoji: "📸" },
  facebook: { label: "Facebook", emoji: "📘" },
  sms: { label: "SMS", emoji: "✉️" },
  multi: { label: "Multi-Channel", emoji: "🔄" },
};

window.CampStudio.events = {
  listeners: {},
  on: function (name, cb) {
    if (!this.listeners[name]) this.listeners[name] = [];
    this.listeners[name].push(cb);
  },
  emit: function (name, data) {
    (this.listeners[name] || []).forEach(function (cb) {
      try { cb(data); } catch (e) { console.error("CampStudio listener error:", e); }
    });
  },
};
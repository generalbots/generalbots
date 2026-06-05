"use strict";

/**
 * Module 26: AI assistant for Slides.
 * Side-panel chat with LLM proxy endpoint. Context (current slide
 * title, body, element types) is auto-injected. Provides preset
 * commands: Generate bullets, Rewrite this text, Suggest layout,
 * Create agenda, Translate.
 *
 * Public API: window.SlidesAI = { open, close, send, preset, getHistory }.
 */

(function () {
  function getState() { return window.state || null; }
  function getSlide() {
    const s = getState();
    return s ? (s.slides || [])[s.currentSlide || 0] : null;
  }
  function getEndpoint() { return "/api/llm/chat"; }
  let _history = [];
  let _open = false;

  function buildContext() {
    const slide = getSlide();
    if (!slide) return { slideIndex: 0, title: "", body: "", elements: [] };
    const elements = (slide.elements || []).map((e) => ({ type: e.type, text: (e.text || "").slice(0, 200) }));
    const titleEl = (slide.elements || []).find((e) => e.type === "title");
    const bodyEl = (slide.elements || []).find((e) => e.type === "body" || e.type === "text");
    return {
      slideIndex: (getState() || {}).currentSlide || 0,
      layout: slide.layout || "title-and-content",
      title: titleEl ? (titleEl.text || "") : "",
      body: bodyEl ? (bodyEl.text || "") : "",
      elements: elements.slice(0, 12),
    };
  }

  async function send(prompt, options) {
    const ctx = buildContext();
    const payload = {
      messages: _history.concat([{ role: "user", content: prompt }]),
      context: { slides: ctx, app: "slides" },
      stream: !!(options && options.stream),
    };
    try {
      const res = await fetch(getEndpoint(), {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });
      if (!res.ok) throw new Error("AI endpoint returned " + res.status);
      const data = await res.json();
      const text = (data && (data.choices && data.choices[0] && data.choices[0].message && data.choices[0].message.content) || data.text || data.content || data.response || "");
      _history.push({ role: "user", content: prompt });
      _history.push({ role: "assistant", content: text });
      if (_history.length > 40) _history = _history.slice(-40);
      renderHistory();
      return text;
    } catch (err) {
      _history.push({ role: "user", content: prompt });
      _history.push({ role: "assistant", content: "[offline] " + (err.message || "Falha de conexão com LLM. Tente novamente mais tarde.") });
      renderHistory();
      return null;
    }
  }

  const PRESETS = [
    { id: "bullets", label: "Gerar bullets", prompt: "Liste 5 tópicos curtos em bullets sobre: " },
    { id: "rewrite", label: "Reescrever texto", prompt: "Reescreva de forma profissional e concisa o seguinte texto: " },
    { id: "layout", label: "Sugerir layout", prompt: "Sugira o melhor layout para um slide sobre: " },
    { id: "agenda", label: "Criar agenda", prompt: "Crie uma agenda em tópicos para uma apresentação sobre: " },
    { id: "translate-en", label: "Traduzir p/ inglês", prompt: "Traduza o texto a seguir para inglês mantendo tom profissional: " },
    { id: "summarize", label: "Resumir slide", prompt: "Resuma em 2 frases o conteúdo deste slide." },
  ];

  function preset(id, context) {
    const p = PRESETS.find((x) => x.id === id);
    if (!p) return null;
    const slide = getSlide();
    const ctx = context || (slide ? (((slide.elements || []).find((e) => e.type === "title" || e.type === "body" || e.type === "text") || {}).text || "") : "");
    const full = p.prompt + (ctx || "");
    return send(full);
  }

  function open() {
    _open = true;
    let panel = document.querySelector(".slides-ai-panel");
    if (!panel) {
      panel = document.createElement("div");
      panel.className = "slides-ai-panel";
      panel.style.cssText = "position:fixed;right:0;top:0;bottom:0;width:340px;background:#fff;border-left:1px solid #dadce0;z-index:9500;display:flex;flex-direction:column;font-family:Inter,Roboto,Arial,sans-serif;";
      panel.innerHTML = "<div style='padding:12px 16px;border-bottom:1px solid #e8eaed;display:flex;justify-content:space-between;align-items:center;'><strong>Assistente IA</strong><button data-action='close-ai' style='background:none;border:none;cursor:pointer;font-size:18px;'>&times;</button></div>" +
        "<div class='ai-history' style='flex:1;overflow-y:auto;padding:12px 16px;font-size:13px;line-height:1.4;'></div>" +
        "<div class='ai-presets' style='padding:8px 12px;display:flex;flex-wrap:wrap;gap:6px;border-top:1px solid #e8eaed;'></div>" +
        "<div style='padding:8px 12px;border-top:1px solid #e8eaed;display:flex;gap:6px;'><input class='ai-input' type='text' placeholder='Pergunte algo...' style='flex:1;padding:8px;border:1px solid #dadce0;border-radius:4px;'><button data-action='ai-send' style='padding:8px 12px;background:#1a73e8;color:#fff;border:none;border-radius:4px;cursor:pointer;'>Enviar</button></div>";
      document.body.appendChild(panel);
      const presetsBox = panel.querySelector(".ai-presets");
      for (const p of PRESETS) {
        const b = document.createElement("button");
        b.textContent = p.label;
        b.style.cssCssText = "";
        b.style.cssText = "font-size:11px;padding:5px 9px;background:#f1f3f4;border:1px solid #dadce0;border-radius:14px;cursor:pointer;";
        b.addEventListener("click", function () { preset(p.id); });
        presetsBox.appendChild(b);
      }
      panel.querySelector("[data-action='close-ai']").addEventListener("click", close);
      const input = panel.querySelector(".ai-input");
      const sendBtn = panel.querySelector("[data-action='ai-send']");
      sendBtn.addEventListener("click", function () { const v = input.value.trim(); if (v) { send(v); input.value = ""; } });
      input.addEventListener("keydown", function (e) { if (e.key === "Enter") { const v = input.value.trim(); if (v) { send(v); input.value = ""; } } });
    }
    panel.style.display = "flex";
    renderHistory();
  }

  function close() {
    _open = false;
    const panel = document.querySelector(".slides-ai-panel");
    if (panel) panel.style.display = "none";
  }

  function renderHistory() {
    const box = document.querySelector(".ai-history");
    if (!box) return;
    box.innerHTML = "";
    for (const m of _history) {
      const d = document.createElement("div");
      d.style.cssText = "margin-bottom:8px;padding:8px 10px;border-radius:8px;background:" + (m.role === "user" ? "#e8f0fe" : "#f1f3f4") + ";color:#202124;";
      d.textContent = m.content;
      box.appendChild(d);
    }
    box.scrollTop = box.scrollHeight;
  }

  function getHistory() { return _history.slice(); }

  function attach() {
    const btn = document.querySelector("[data-toolbar-action='ai-assistant'], [data-action='ai-open']");
    if (btn) btn.addEventListener("click", function (e) { e.preventDefault(); _open ? close() : open(); });
    document.addEventListener("keydown", function (e) { if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === "a") { e.preventDefault(); _open ? close() : open(); } });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", attach);
  } else {
    setTimeout(attach, 50);
  }

  window.SlidesAI = { open, close, send, preset, getHistory, buildContext, PRESETS };
})();

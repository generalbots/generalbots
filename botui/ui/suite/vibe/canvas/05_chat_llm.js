/* Vibe Canvas — split from vibe/vibe-canvas.js for #1731 */
(function () {
    "use strict";
    var V = window.VibeDesign;

    // Draw-by-chat offline parser (#1191). Recognized grammar:
    //   add rect[angle] <label> | rectangle <label> | box <label>
    //   add text <text>          | text <text>
    //   connect|link A -> B      | connect A to B
    //   rename <from> to <to>    | delete|remove <label> | clear
    function chatFeedback(msg) {
        var bar = document.getElementById("vibeDesignChatLog");
        if (bar) {
            bar.textContent = msg;
            clearTimeout(chatFeedback._t);
            chatFeedback._t = setTimeout(function () { bar.textContent = ""; }, 5000);
        }
        V.setStatus(msg);
    }

    function findLabelled(label) {
        var needle = label.toLowerCase();
        return V.state.elements.filter(function (e) {
            return e.text && String(e.text).toLowerCase() === needle;
        });
    }

    function labelAnchor(label) {
        var items = findLabelled(label);
        var best = null;
        items.forEach(function (e) {
            if (!best || (e.type === "rectangle" && best.type !== "rectangle")) best = e;
        });
        return best;
    }

    function addRect(label, opts) {
        opts = opts || {};
        var base = Date.now() + Math.floor(Math.random() * 1000);
        var rect = { id: "el-" + base, type: "rectangle", x: opts.x != null ? opts.x : 80 + (V.state.elements.length % 5) * 40,
            y: opts.y != null ? opts.y : 80 + Math.floor(V.state.elements.length / 5) * 40 + 60, w: 170, h: 90 };
        V.state.elements.push(rect);
        if (label) {
            V.state.elements.push({ id: "el-" + base + "t", type: "text", x: rect.x + 12, y: rect.y + rect.h / 2 - 14,
                w: Math.max(60, rect.w - 24), h: 28, text: label });
        }
        V.selectedId = rect.id;
        V.render(); V.scheduleSave();
        return rect;
    }

    function centerOf(el) { return { x: el.x + el.w / 2, y: el.y + el.h / 2 }; }

    function connect(fromLabel, toLabel) {
        var a = labelAnchor(fromLabel), b = labelAnchor(toLabel);
        if (!a || !b) return false;
        var ca = centerOf(a), cb = centerOf(b);
        V.state.connectors.push({ id: "line-" + Date.now(), x1: ca.x, y1: ca.y, x2: cb.x, y2: cb.y });
        V.render(); V.scheduleSave();
        return true;
    }

    function chatCommand(raw) {
        var text = raw.trim().replace(/;+$/g, "");
        if (!text) return false;
        var m;
        if ((m = text.match(/^(?:add\s+)?(?:rect(?:angle)?|box)\s+(.{1,40})$/i))) {
            addRect(m[1].trim());
            chatFeedback("▭ Added “" + m[1].trim() + "”");
            return true;
        }
        if ((m = text.match(/^(?:add\s+)?text\s+(.{1,120})$/i))) {
            var t = m[1].trim();
            V.state.elements.push({ id: "el-" + Date.now(), type: "text", x: 110, y: 110 + (V.state.elements.length % 6) * 30, w: 220, h: 30, text: t });
            V.render(); V.scheduleSave();
            chatFeedback("🔤 Added text “" + t + "”");
            return true;
        }
        if ((m = text.match(/^(?:connect|link)\s+['"]?(.+?)['"]?\s*(?:->|→|to)\s*['"]?(.+?)['"]?$/i))) {
            if (connect(m[1].trim(), m[2].trim())) {
                chatFeedback("↔ Connected “" + m[1].trim() + "” → “" + m[2].trim() + "”");
                return true;
            }
            chatFeedback("⚠ Not found. Try: add rect " + m[1].trim() + " first.");
            return true;
        }
        if ((m = text.match(/^rename\s+['"]?(.+?)['"]?\s+to\s+['"]?(.+?)['"]?$/i))) {
            var target = labelAnchor(m[1].trim());
            if (!target) { chatFeedback("⚠ No element named “" + m[1].trim() + "”."); return true; }
            target.text = m[2].trim();
            V.render(); V.scheduleSave();
            chatFeedback("✏ Renamed → “" + m[2].trim() + "”");
            return true;
        }
        if ((m = text.match(/^(?:delete|remove)\s+['"]?(.+?)['"]?$/i))) {
            var doomed = findLabelled(m[1].trim());
            if (!doomed.length) { chatFeedback("⚠ Nothing named “" + m[1].trim() + "”."); return true; }
            var ids = {}; doomed.forEach(function (d) { ids[d.id] = 1; });
            V.state.elements = V.state.elements.filter(function (e) { return !ids[e.id]; });
            V.render(); V.scheduleSave();
            chatFeedback("🗑 Removed “" + m[1].trim() + "”");
            return true;
        }
        if (/^clear(\s+all)?$|^reset$/i.test(text)) {
            V.state = { elements: [], connectors: [], generated: V.state.generated };
            V.selectedId = null;
            V.render(); V.scheduleSave();
            chatFeedback("🧽 Canvas cleared (baseline kept)");
            return true;
        }
        chatFeedback("Try: add rect API · add text Login · connect UI to API · rename A to B · delete A · clear");
        return false;
    }

    // Pinned draw-by-chat input docked at the bottom of the drawing surface.
    function installChatBar(hostOverride) {
        var host = hostOverride || V.visEl("vibeCanvas");
        if (!host || host.dataset.chatReady === "1") return;
        if (host.querySelector("#vibeDesignChatInput")) { host.dataset.chatReady = "1"; return; }
        host.dataset.chatReady = "1";
        var wrap = document.createElement("div");
        wrap.style.cssText = [
            "position:absolute", "left:12px", "bottom:12px", "z-index:20",
            "display:flex", "align-items:center", "gap:8px",
            "background:var(--surface,#14142a)", "border:1px solid var(--border,#333)",
            "border-radius:10px", "padding:7px 10px", "box-shadow:0 10px 26px rgba(0,0,0,.35)",
        ].join(";");
        wrap.innerHTML = '<span title="Draw by chat" style="font-size:13px;">💬</span>' +
            '<input id="vibeDesignChatInput" type="text" autocomplete="off" placeholder=' +
            '"draw by chat: add rect API · connect UI to API" ' +
            'style="width:min(340px,42vw);background:transparent;border:none;color:var(--text,#eee);font-size:12px;outline:none;" />' +
            '<span id="vibeDesignChatLog" style="font-size:11px;color:var(--accent,#84d669);max-width:220px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;"></span>';
        host.appendChild(wrap);
        var input = wrap.querySelector("#vibeDesignChatInput");
        input.addEventListener("keydown", function (ev) {
            if (ev.key === "Enter") {
                ev.preventDefault();
                window.VibeDesign && window.VibeDesign.chat(input.value);
                input.value = "";
            }
        });
    }

    // LLM architecture generation: sends the project context through the
    // Vibe assistant (bot LLM = tokenrouter) asking for a strict JSON zone
    // map, then renders it as the generated diagram and persists project.draw.
    function llmGenerateArchitecture() {
        var pid = V.projectId();
        if (!pid) { chatFeedback("⚠ Select a project first."); return; }
        chatFeedback("✦ Asking the LLM for the architecture…");
        V.api("/api/vibe/projects/" + encodeURIComponent(pid) + "/files", { method: "GET", headers: {} })
            .then(function (d) {
                var files = (d && (d.files || d.data && d.data.files)) || [];
                var fileList = (files || []).slice(0, 40).join("\n");
                var prompt = "Design the software architecture for this project. Return ONLY a JSON object, no prose, no markdown, with this exact shape: {" +
                    "\"title\":\"<project name>\",\"zones\":[{\"name\":\"Web\",\"items\":[<short module names>]},{\"name\":\"App\",...},{\"name\":\"API\",...},{\"name\":\"Data\",...}]}. " +
                    "Use the real modules found in this project file list, max 5 items per zone, short names only (no paths). Project files:\n" + fileList;
                window.vibeQuickSubmit(prompt);
                var attempts = 0;
                var timer = setInterval(function () {
                    attempts++;
                    // Bot responses stream into the hidden #vibeChatMirror
                    // (the Run Dock runner log is run-events only, no chat).
                    var messages = document.getElementById("vibeChatMirror");
                    if (!messages) { if (attempts > 24) clearInterval(timer); return; }
                    var nodes = messages.querySelectorAll(".vibe-bot-msg, .bot-message, .message.bot");
                    if (!nodes.length) return;
                    var last = nodes[nodes.length - 1];
                    var text = (last.textContent || "").trim();
                    var m = text.match(/\{[\s\S]*\}/);
                    if (!m || attempts > 24) { if (attempts > 24) clearInterval(timer); return; }
                    var parsed = null;
                    try { parsed = JSON.parse(m[0]); } catch (e) { /* not JSON yet */ }
                    if (!parsed || !parsed.zones) return;
                    clearInterval(timer);
                    var design = { kind: "vibe-design", version: 3, generatedAt: new Date().toISOString(),
                        source: "llm", title: parsed.title || V.projectKey(), svg: svgFromZones(parsed.zones, parsed.title || V.projectKey()) };
                    V.setGeneratedDesign(design);
                    V.render();
                    var content = { kind: "vibe-design", version: 3, elements: V.state.elements, connectors: V.state.connectors, generated: V.state.generated };
                    V.writeProjectDraw(pid, content).then(function () {
                        chatFeedback("✦ LLM architecture saved to project.draw ✓");
                        V.setStatus("Saved · project.draw (LLM) ✓");
                    }).catch(function () {
                        chatFeedback("✦ Architecture generated (project.draw save pending)");
                    });
                }, 700);
            }).catch(function () {
                chatFeedback("⚠ Could not read project files.");
            });
    }

    function svgFromZones(zones, title) {
        var cols = ["Web", "App", "API", "Data"];
        var colors = { Web: "#2563eb", App: "#16a34a", API: "#d97706", Data: "#7c3aed" };
        var h = "";
        cols.forEach(function (col, i) {
            var z = (zones || []).find(function (x) { return String(x.name).toLowerCase() === String(col).toLowerCase(); });
            var items = (z && z.items) || [];
            if (!items.length) items = [col.toLowerCase()];
            var x = 40 + i * 225;
            h += '<rect x="' + x + '" y="70" width="185" height="340" rx="12" fill="#ffffff" stroke="' + colors[col] + '" stroke-width="2"/>';
            h += '<text x="' + (x + 16) + '" y="98" font-family="system-ui" font-size="16" font-weight="700" fill="' + colors[col] + '">' + col + "</text>";
            var y = 130;
            items.slice(0, 5).forEach(function (item) {
                h += '<rect x="' + (x + 14) + '" y="' + (y - 14) + '" width="157" height="22" rx="6" fill="#f1f5f9"/>';
                h += '<text x="' + (x + 24) + '" y="' + y + '" font-family="monospace" font-size="11" fill="#334155">' + item + "</text>";
                y += 32;
            });
        });
        return '<svg viewBox="0 0 960 520" xmlns="http://www.w3.org/2000/svg">' +
            '<rect x="16" y="20" width="928" height="480" rx="14" fill="#f8fafc" stroke="#cbd5e1"/>' +
            '<text x="40" y="48" font-family="system-ui" font-size="16" font-weight="700" fill="#0f172a">' + (title || "Architecture") + "</text>" +
            h + "</svg>";
    }

    V.getState = function () { return V.state; };
    V.chat = function (raw) { return chatCommand(String(raw || "")); };
    V.installChat = function (hostOverride) { return installChatBar(hostOverride); };
    V.llmGenerate = llmGenerateArchitecture;

    V.init();
    V.installChat();
})();

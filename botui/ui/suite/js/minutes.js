"use strict";
/* Minutes — meeting minutes engine.
 *
 * Features:
 *   - Meeting metadata: title, date, attendees, agenda
 *   - Notes with timestamps
 *   - Action items: who, what, when, status
 *   - Decisions: log of formal decisions
 *   - Auto-summary: extract decisions and action items from text
 *   - Export to Markdown / DOCX
 *   - Search across meetings
 *
 * Public: window.Minutes
 */
(function (window) {
  const STORAGE_KEY = "gb-minutes-meetings";

  function readArr(k) { try { return JSON.parse(localStorage.getItem(k) || "[]"); } catch (_) { return []; } }
  function writeArr(k, arr) { try { localStorage.setItem(k, JSON.stringify(arr)); } catch (_) {} }
  function uid() { return "mt_" + Math.random().toString(36).slice(2, 12); }

  function createMeeting(opts) {
    const meetings = readArr(STORAGE_KEY);
    const m = {
      id: uid(),
      title: opts.title || "Reunião sem título",
      date: opts.date || new Date().toISOString(),
      duration: opts.duration || 0,
      location: opts.location || "",
      organizer: opts.organizer || "anon",
      attendees: opts.attendees || [],
      agenda: opts.agenda || [],
      notes: [],
      actionItems: [],
      decisions: [],
      status: "in-progress",
      createdAt: new Date().toISOString()
    };
    meetings.push(m);
    writeArr(STORAGE_KEY, meetings);
    return m;
  }

  function getMeeting(id) {
    return readArr(STORAGE_KEY).find(m => m.id === id) || null;
  }

  function listMeetings(opts) {
    const all = readArr(STORAGE_KEY);
    if (!opts) return all.sort((a, b) => b.date.localeCompare(a.date));
    return all.filter(m => {
      if (opts.status && m.status !== opts.status) return false;
      if (opts.organizer && m.organizer !== opts.organizer) return false;
      if (opts.startDate && new Date(m.date) < new Date(opts.startDate)) return false;
      if (opts.endDate && new Date(m.date) > new Date(opts.endDate)) return false;
      if (opts.search) {
        const q = opts.search.toLowerCase();
        const text = (m.title + " " + m.notes.map(n => n.text).join(" ") + " " + m.actionItems.map(a => a.what).join(" ")).toLowerCase();
        if (text.indexOf(q) < 0) return false;
      }
      return true;
    }).sort((a, b) => b.date.localeCompare(a.date));
  }

  function addNote(meetingId, opts) {
    const meetings = readArr(STORAGE_KEY);
    const m = meetings.find(x => x.id === meetingId);
    if (!m) return null;
    const note = {
      id: uid(),
      timestamp: opts.timestamp || new Date().toISOString(),
      elapsed: opts.elapsed || 0,
      author: opts.author || "anon",
      text: opts.text || "",
      type: opts.type || "discussion"
    };
    m.notes.push(note);
    writeArr(STORAGE_KEY, meetings);
    if (opts.type === "decision" || /^decidid[oa]|aprovad[oa]|definid[oa]/i.test(opts.text)) {
      m.decisions.push({ id: uid(), text: opts.text, madeAt: new Date().toISOString(), madeBy: opts.author });
    }
    return note;
  }

  function addActionItem(meetingId, opts) {
    const meetings = readArr(STORAGE_KEY);
    const m = meetings.find(x => x.id === meetingId);
    if (!m) return null;
    const item = {
      id: uid(),
      what: opts.what,
      who: opts.who || "anon",
      when: opts.when || null,
      priority: opts.priority || "normal",
      status: "open",
      createdAt: new Date().toISOString(),
      completedAt: null
    };
    m.actionItems.push(item);
    writeArr(STORAGE_KEY, meetings);
    return item;
  }

  function completeActionItem(meetingId, actionId) {
    const meetings = readArr(STORAGE_KEY);
    const m = meetings.find(x => x.id === meetingId);
    if (!m) return false;
    const a = m.actionItems.find(x => x.id === actionId);
    if (a) { a.status = "completed"; a.completedAt = new Date().toISOString(); writeArr(STORAGE_KEY, meetings); return true; }
    return false;
  }

  function addDecision(meetingId, opts) {
    const meetings = readArr(STORAGE_KEY);
    const m = meetings.find(x => x.id === meetingId);
    if (!m) return null;
    const d = {
      id: uid(),
      text: opts.text,
      madeBy: opts.madeBy || "anon",
      madeAt: new Date().toISOString(),
      context: opts.context || ""
    };
    m.decisions.push(d);
    writeArr(STORAGE_KEY, meetings);
    return d;
  }

  function closeMeeting(meetingId, duration) {
    const meetings = readArr(STORAGE_KEY);
    const m = meetings.find(x => x.id === meetingId);
    if (!m) return false;
    m.status = "closed";
    m.closedAt = new Date().toISOString();
    m.duration = duration || m.duration;
    writeArr(STORAGE_KEY, meetings);
    return true;
  }

  function autoExtract(text) {
    const result = { decisions: [], actionItems: [], questions: [], risks: [] };
    const lines = text.split(/[\n.!?]+/).map(l => l.trim()).filter(l => l.length > 5);
    lines.forEach(line => {
      if (/^decidid[oa]s?|aprovad[oa]s?|definid[oa]s?|acordad[oa]s?|deliberad[oa]s?/i.test(line)) {
        result.decisions.push(line);
      }
      if (/^(\w+)\s+(deve|precisa|vai|fica respons[áa]vel|tem que|pode)\s+/i.test(line)) {
        const m = line.match(/^(\w+)\s+(deve|precisa|vai|fica respons[áa]vel|tem que|pode)\s+(.+)/i);
        if (m) {
          result.actionItems.push({ who: m[1], what: m[3], dueDate: null });
        }
      }
      if (/\?$/.test(line) || /^(ser[áa] que|como|quando|onde|por que|qual)/i.test(line)) {
        result.questions.push(line);
      }
      if (/^(risco|perigo|aten[çc][ãa]o|cuidado|preocupa[çc][ãa]o)/i.test(line)) {
        result.risks.push(line);
      }
    });
    return result;
  }

  function exportMarkdown(meeting) {
    const m = meeting;
    let md = "# " + m.title + "\n\n";
    md += "**Data:** " + new Date(m.date).toLocaleString("pt-BR") + "\n";
    md += "**Local:** " + (m.location || "—") + "\n";
    md += "**Organizador:** " + m.organizer + "\n";
    md += "**Duração:** " + Math.floor(m.duration / 60) + " minutos\n\n";
    if (m.attendees && m.attendees.length) {
      md += "## Participantes\n\n";
      m.attendees.forEach(a => { md += "- " + a + "\n"; });
      md += "\n";
    }
    if (m.agenda && m.agenda.length) {
      md += "## Pauta\n\n";
      m.agenda.forEach((a, i) => { md += (i + 1) + ". " + a + "\n"; });
      md += "\n";
    }
    if (m.notes && m.notes.length) {
      md += "## Notas\n\n";
      m.notes.forEach(n => { md += "- **" + new Date(n.timestamp).toLocaleTimeString("pt-BR") + "** [" + n.author + "]: " + n.text + "\n"; });
      md += "\n";
    }
    if (m.decisions && m.decisions.length) {
      md += "## Decisões\n\n";
      m.decisions.forEach(d => { md += "- " + d.text + " _(" + d.madeBy + ", " + new Date(d.madeAt).toLocaleString("pt-BR") + ")_\n"; });
      md += "\n";
    }
    if (m.actionItems && m.actionItems.length) {
      md += "## Itens de Ação\n\n";
      md += "| Responsável | Ação | Prazo | Status |\n";
      md += "|---|---|---|---|\n";
      m.actionItems.forEach(a => {
        const due = a.when ? new Date(a.when).toLocaleDateString("pt-BR") : "—";
        const status = a.status === "completed" ? "✅ Concluído" : "🟡 Aberto";
        md += "| " + a.who + " | " + a.what + " | " + due + " | " + status + " |\n";
      });
      md += "\n";
    }
    return md;
  }

  function getActionItems(opts) {
    const meetings = listMeetings(opts);
    const all = [];
    meetings.forEach(m => {
      m.actionItems.forEach(a => {
        if (opts && opts.status && a.status !== opts.status) return;
        if (opts && opts.who && a.who !== opts.who) return;
        all.push({ ...a, meetingId: m.id, meetingTitle: m.title });
      });
    });
    return all;
  }

  function meetingStats(meeting) {
    return {
      notesCount: meeting.notes.length,
      decisionsCount: meeting.decisions.length,
      actionItemsCount: meeting.actionItems.length,
      openActionItems: meeting.actionItems.filter(a => a.status === "open").length,
      completedActionItems: meeting.actionItems.filter(a => a.status === "completed").length,
      duration: meeting.duration,
      attendeesCount: meeting.attendees.length
    };
  }

  window.Minutes = {
    createMeeting: createMeeting,
    getMeeting: getMeeting,
    listMeetings: listMeetings,
    addNote: addNote,
    addActionItem: addActionItem,
    completeActionItem: completeActionItem,
    addDecision: addDecision,
    closeMeeting: closeMeeting,
    autoExtract: autoExtract,
    exportMarkdown: exportMarkdown,
    getActionItems: getActionItems,
    meetingStats: meetingStats
  };
})(window);

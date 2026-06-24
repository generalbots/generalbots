"use strict";
/* TimeClock — time tracking, attendance, hours calculation.
 *
 * Features:
 *   - Clock in/out with timestamps
 *   - Daily, weekly, monthly hours calculation
 *   - Overtime, night-shift, holiday multipliers
 *   - Break time deduction
 *   - Project/task tracking
 *   - Timesheet generation
 *   - Approval workflow
 *
 * Public: window.TimeClock
 */
(function (window) {
  const STORAGE_KEY = "gb-timeclock-entries";
  const PROJECT_KEY = "gb-timeclock-projects";

  function readArr(k) { try { return JSON.parse(localStorage.getItem(k) || "[]"); } catch (_) { return []; } }
  function writeArr(k, arr) { try { localStorage.setItem(k, JSON.stringify(arr)); } catch (_) {} }
  function readObj(k) { try { return JSON.parse(localStorage.getItem(k) || "{}"); } catch (_) { return {}; } }
  function writeObj(k, obj) { try { localStorage.setItem(k, JSON.stringify(obj)); } catch (_) {} }
  function uid() { return "tc_" + Math.random().toString(36).slice(2, 12) + "_" + Date.now().toString(36); }

  function clockIn(opts) {
    const entries = readArr(STORAGE_KEY);
    const lastOpen = entries.find(e => e.userId === opts.userId && !e.clockOut);
    if (lastOpen) return { ok: false, msg: "Já existe um ponto aberto", entry: lastOpen };
    const entry = {
      id: uid(),
      userId: opts.userId || "anon",
      projectId: opts.projectId || null,
      taskId: opts.taskId || null,
      clockIn: new Date().toISOString(),
      clockOut: null,
      breaks: [],
      notes: opts.notes || "",
      approved: false
    };
    entries.push(entry);
    writeArr(STORAGE_KEY, entries);
    return { ok: true, entry: entry };
  }

  function clockOut(opts) {
    const entries = readArr(STORAGE_KEY);
    const lastOpen = entries.find(e => e.userId === opts.userId && !e.clockOut);
    if (!lastOpen) return { ok: false, msg: "Nenhum ponto aberto" };
    lastOpen.clockOut = new Date().toISOString();
    writeArr(STORAGE_KEY, entries);
    return { ok: true, entry: lastOpen };
  }

  function startBreak(opts) {
    const entries = readArr(STORAGE_KEY);
    const open = entries.find(e => e.userId === opts.userId && !e.clockOut);
    if (!open) return { ok: false, msg: "Nenhum ponto aberto" };
    if (open.breaks && open.breaks.length && !open.breaks[open.breaks.length - 1].end) {
      return { ok: false, msg: "Já existe um intervalo aberto" };
    }
    if (!open.breaks) open.breaks = [];
    open.breaks.push({ start: new Date().toISOString(), end: null });
    writeArr(STORAGE_KEY, entries);
    return { ok: true };
  }

  function endBreak(opts) {
    const entries = readArr(STORAGE_KEY);
    const open = entries.find(e => e.userId === opts.userId && !e.clockOut);
    if (!open || !open.breaks || !open.breaks.length) return { ok: false, msg: "Nenhum intervalo aberto" };
    const last = open.breaks[open.breaks.length - 1];
    if (last.end) return { ok: false, msg: "Intervalo já encerrado" };
    last.end = new Date().toISOString();
    writeArr(STORAGE_KEY, entries);
    return { ok: true };
  }

  function calcDurationMs(entry) {
    if (!entry.clockIn) return 0;
    const start = new Date(entry.clockIn).getTime();
    const end = entry.clockOut ? new Date(entry.clockOut).getTime() : Date.now();
    let breakMs = 0;
    if (entry.breaks) {
      entry.breaks.forEach(b => {
        if (b.start) {
          const bs = new Date(b.start).getTime();
          const be = b.end ? new Date(b.end).getTime() : Date.now();
          breakMs += (be - bs);
        }
      });
    }
    return Math.max(0, end - start - breakMs);
  }

  function calcDurationHours(entry) {
    return calcDurationMs(entry) / (1000 * 60 * 60);
  }

  function formatDuration(ms) {
    const totalSec = Math.floor(ms / 1000);
    const h = Math.floor(totalSec / 3600);
    const m = Math.floor((totalSec % 3600) / 60);
    const s = totalSec % 60;
    return h.toString().padStart(2, "0") + ":" + m.toString().padStart(2, "0") + ":" + s.toString().padStart(2, "0");
  }

  function isNightHour(date) {
    const h = new Date(date).getHours();
    return h >= 22 || h < 5;
  }

  function isWeekend(date) {
    const d = new Date(date).getDay();
    return d === 0 || d === 6;
  }

  function isHoliday(date, holidays) {
    const d = new Date(date);
    const key = (d.getMonth() + 1).toString().padStart(2, "0") + "-" + d.getDate().toString().padStart(2, "0");
    return (holidays || []).indexOf(key) >= 0;
  }

  function calcPayableHours(entry, opts) {
    const opts2 = opts || {};
    const standardDay = opts2.standardDayHours || 8;
    const overtimeRate = opts2.overtimeRate || 1.5;
    const nightRate = opts2.nightRate || 1.2;
    const weekendRate = opts2.weekendRate || 2.0;
    const holidays = opts2.holidays || [];

    const totalMs = calcDurationMs(entry);
    const totalHours = totalMs / (1000 * 60 * 60);
    const start = new Date(entry.clockIn);
    const end = entry.clockOut ? new Date(entry.clockOut) : new Date();
    let nightMs = 0, weekendMs = 0, holidayMs = 0, normalMs = 0;
    const cur = new Date(start);
    while (cur < end) {
      const next = new Date(cur.getTime() + 60 * 1000);
      const ms = next - cur;
      if (isWeekend(cur)) weekendMs += ms;
      else if (isHoliday(cur, holidays)) holidayMs += ms;
      else if (isNightHour(cur)) nightMs += ms;
      else normalMs += ms;
      cur.setTime(next.getTime());
    }
    const normalHours = normalMs / (1000 * 60 * 60);
    const nightHours = nightMs / (1000 * 60 * 60);
    const weekendHours = weekendMs / (1000 * 60 * 60);
    const holidayHours = holidayMs / (1000 * 60 * 60);

    const overtime = Math.max(0, normalHours - standardDay);
    const baseHours = Math.min(normalHours, standardDay);

    return {
      total: totalHours,
      normal: baseHours,
      overtime: overtime,
      night: nightHours,
      weekend: weekendHours,
      holiday: holidayHours,
      payable: baseHours + (overtime * overtimeRate) + (nightHours * nightRate) + (weekendHours * weekendRate) + (holidayHours * weekendRate)
    };
  }

  function listEntries(opts) {
    const all = readArr(STORAGE_KEY);
    if (!opts) return all;
    return all.filter(e => {
      if (opts.userId && e.userId !== opts.userId) return false;
      if (opts.projectId && e.projectId !== opts.projectId) return false;
      if (opts.startDate && new Date(e.clockIn) < new Date(opts.startDate)) return false;
      if (opts.endDate && new Date(e.clockIn) > new Date(opts.endDate)) return false;
      if (opts.approved !== undefined && e.approved !== opts.approved) return false;
      return true;
    });
  }

  function approveEntry(id, approverId) {
    const entries = readArr(STORAGE_KEY);
    const e = entries.find(x => x.id === id);
    if (e) {
      e.approved = true;
      e.approverId = approverId;
      e.approvedAt = new Date().toISOString();
      writeArr(STORAGE_KEY, entries);
      return true;
    }
    return false;
  }

  function addProject(opts) {
    const projects = readObj(PROJECT_KEY);
    const id = uid();
    projects[id] = { id: id, name: opts.name, code: opts.code || id, color: opts.color || "#3b82f6", hourlyRate: opts.hourlyRate || 0, active: true, createdAt: new Date().toISOString() };
    writeObj(PROJECT_KEY, projects);
    return projects[id];
  }

  function listProjects() {
    return Object.values(readObj(PROJECT_KEY));
  }

  function getProject(id) {
    return readObj(PROJECT_KEY)[id] || null;
  }

  function timesheet(entries, opts) {
    const start = (opts && opts.start) || new Date(new Date().getFullYear(), new Date().getMonth(), 1);
    const end = (opts && opts.end) || new Date();
    const filtered = entries.filter(e => new Date(e.clockIn) >= start && new Date(e.clockIn) <= end);
    const groups = {};
    filtered.forEach(e => {
      const day = e.clockIn.slice(0, 10);
      if (!groups[day]) groups[day] = { day: day, entries: [], total: 0 };
      const hours = calcDurationHours(e);
      groups[day].entries.push(e);
      groups[day].total += hours;
    });
    return Object.values(groups).sort((a, b) => a.day.localeCompare(b.day));
  }

  function payrollSummary(entries, hourlyRate) {
    const total = entries.reduce((acc, e) => acc + calcDurationHours(e), 0);
    return { hours: total, gross: total * (hourlyRate || 0), entries: entries.length };
  }

  window.TimeClock = {
    clockIn: clockIn,
    clockOut: clockOut,
    startBreak: startBreak,
    endBreak: endBreak,
    calcDurationMs: calcDurationMs,
    calcDurationHours: calcDurationHours,
    formatDuration: formatDuration,
    isNightHour: isNightHour,
    isWeekend: isWeekend,
    isHoliday: isHoliday,
    calcPayableHours: calcPayableHours,
    listEntries: listEntries,
    approveEntry: approveEntry,
    addProject: addProject,
    listProjects: listProjects,
    getProject: getProject,
    timesheet: timesheet,
    payrollSummary: payrollSummary
  };
})(window);

"use strict";
/**
 * 12d_block_vm.js - An agent action in the Virtual Machine, in the steps
 * block (12b_block_live.js) and in the history: the exact command, path or
 * URL, how it was allowed or why it did not run, exit code, files and the
 * first output lines (long text folds behind "ver mais").
 */
(function () {
var A = window.GBAssistant;
var B = A._blk;
var esc = B.esc, t = B.t;
/* An agent action (vm_consent): exact command / path / URL, how it was
   allowed, and its result. */
var VM_VERBS = {
    command: ["Comando", "Command"], file_read: ["Leitura", "Read"], file_write: ["Arquivo", "File"], file_delete: ["Apagar", "Delete"],
    browser: ["Endereço", "URL"], network: ["Instalar", "Install"],
    drive_save: ["Salvar no Drive", "Save to Drive"], external_save: ["Enviar para drive externo", "Send to external drive"],
    web_search: ["Pesquisa na web (sem pedido de permissão)", "Web search (no permission asked)"],
    mcp_tool: ["Ferramenta externa (sem pedido de permissão)", "External tool (no permission asked)"]
};
var VM_ALLOWED = {
    once: ["Permitido por “Permitir uma vez”", "Allowed with “Allow once”"],
    conversation: ["Permitido por “Permitir nesta conversa”", "Allowed with “Allow in this conversation”"],
    all: ["Permitido por “Permitir tudo”", "Allowed with “Allow all”"],
    person: ["Pedido na sua mensagem", "Asked in your message"]
};
var VM_DENIED = {
    denied: ["Negado por você: nada foi executado", "Denied by you: nothing ran"],
    no_answer: ["Sem resposta em 5 minutos: nada foi executado", "No answer in 5 minutes: nothing ran"],
    cancelled: ["Cancelado", "Cancelled"],
    unavailable: ["Permissão indisponível: nada foi executado", "Permission unavailable: nothing ran"]
};
var DENIED_CHIP = { denied: ["Negado", "Denied"], cancelled: ["Cancelado", "Cancelled"],
    no_answer: ["Sem resposta", "No answer"], unavailable: ["Indisponível", "Unavailable"] };
function vmLabel(step) {
    var verb = VM_VERBS[step.action.kind] || VM_VERBS.command;
    var first = String(step.action.text || "").split("\n")[0];
    return t(verb[0], verb[1]) + ": " + (first.length > 140 ? first.slice(0, 140) + "…" : first);
}
/* Under the row: how it was allowed (or why it did not run), exit code,
   files, the full command when the label cut it, and the first output
   lines. Long text folds behind "ver mais". */
function vmExtra(s) {
    if (s.key !== "vm_action") { return ""; }
    var facts = [];
    var pair;
    if (s.denied) { pair = VM_DENIED[s.denied] || VM_DENIED.denied; facts.push(t(pair[0], pair[1])); }
    else if (s.waiting && s.state === "running") { facts.push(t("Aguardando sua permissão", "Waiting for your permission")); }
    else if (s.allowedBy) { pair = VM_ALLOWED[s.allowedBy] || VM_ALLOWED.once; facts.push(t(pair[0], pair[1])); }
    if (s.state !== "running" && !s.denied) {
        facts.push(s.exit != null ? t("Código de saída ", "Exit code ") + s.exit : (s.state === "ok" ? t("Concluído", "Done") : t("Falhou", "Failed")));
    }
    if (s.files && s.files.length) { facts.push(t("Arquivos: ", "Files: ") + s.files.join(", ")); }
    var text = String((s.action && s.action.text) || "");
    var full = text.indexOf("\n") !== -1 || text.length > 140 ? text : "";
    var lines = s.output || [];
    var out = lines.join("\n");
    var long = !!full || out.length > 240 || lines.some(function (line) { return String(line).length > 110; });
    return '<div class="gb-run-extra' + (s.denied ? ' gb-run-extra--denied' : '') + (long && !s.open ? ' gb-run-clip' : '') + '">' +
        '<div class="gb-run-facts">' + facts.map(function (f) { return '<span>' + esc(f) + '</span>'; }).join("") + '</div>' +
        (full ? '<pre class="gb-run-cmd">' + esc(full) + '</pre>' : '') +
        (out ? '<pre class="gb-run-output">' + esc(out) + '</pre>' : '') +
        (long ? '<button type="button" class="gb-run-more" data-step-id="' + esc(s.id) + '" aria-expanded="' + String(!!s.open) + '">' +
            esc(s.open ? t("ver menos", "show less") : t("ver mais", "show more")) + '</button>' : '') +
        '</div>';
}
/* A Virtual Machine permission in the history: what was asked and how it
   was answered. */
var basePermission = A.blockPermission;
A.blockPermission = function (ev) {
    var el = basePermission(ev);
    var s = ev.summary || ev.data || {};
    var verb = VM_VERBS[s.action];
    if (!el || !verb) { return el; }
    var answer = s.decision === "denied" ? VM_DENIED.denied :
        (s.decision === "pending" ? ["Aguardando sua permissão", "Waiting for your permission"] : VM_ALLOWED[s.decision]);
    el.querySelector(".gb-block-summary").textContent = t(verb[0], verb[1]) + (answer ? " · " + t(answer[0], answer[1]) : "");
    if (s.decision === "denied") { el.querySelector(".gb-block-chip").textContent = t("Negado", "Denied"); }
    return el;
};


A._vmStep = { verbs: VM_VERBS, label: vmLabel, extra: vmExtra, chip: DENIED_CHIP };
})();

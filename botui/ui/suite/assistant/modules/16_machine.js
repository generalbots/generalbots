"use strict";
/**
 * 16_machine.js - Agent machine: state, live counters and the footer line.
 * The page itself is drawn by 16b_machine_view.js.
 */
(function () {
    var A = window.GBAssistant;
    var M = A.machine = { model: { status: "loading", snapshots: [], accounts: [] }, busy: false, activity: [] };
    var sequence = 0;
    var sample = null;
    var timer = null;
    var POLL_MS = 5000;
    var FULL_RATIO = 0.95;

    var T = {
        pt: {
            loading: "Consultando", running: "Ligada", stopped: "Desligada", provisioning: "Preparando",
            error: "Atenção", full: "Disco cheio", unavailable: "Indisponível", restoring: "Restaurando",
            machine: "Máquina", failed: "Falha na operação", kicker: "// MÁQUINA DE AGENTES",
            title: "Um computador só seu, onde os agentes trabalham.",
            lead: "Os agentes por assinatura e o modo Agente rodam aqui: executam comandos, instalam ferramentas e geram arquivos, separados do seu Windows.",
            sections: "Seções", secA: "Sua máquina", secB: "Uso agora", secC: "Ajustes", secD: "Restauração",
            secE: "O que vive nela", secF: "Atividade", secG: "Cuidado",
            isolated: "Isolada dentro deste computador. Ela não vê seus arquivos do Windows. O que acontece lá fica lá.",
            subRunning: "Pronta para os agentes", subStopped: "Desligada. Liga sozinha quando um agente precisar dela, ou pelo botão.",
            subProvisioning: "Preparando o espaço dos agentes. Isso leva alguns minutos na primeira vez.",
            subError: "A máquina informou um problema. Atualize ou desligue e ligue de novo.",
            subFull: "O disco da máquina está cheio. Apague pontos de restauração ou arquivos dentro dela.",
            subUnavailable: "O espaço dos agentes requer Incus. No Windows, verifique o WSL e o Incus.",
            subLoading: "Consultando o estado da máquina…",
            start: "Ligar", stop: "Desligar", restart: "Reiniciar", refresh: "Atualizar", soon: "Em breve",
            confirmStop: "Desligar a máquina e interromper as tarefas em andamento?",
            confirmRestore: "Restaurar este ponto e substituir o estado atual da máquina?",
            lastActive: "Última atividade", processes: "Processos", arch: "Arquitetura", name: "Nome",
            every: "Atualiza a cada 5 s", cpu: "Processador", memory: "Memória", disk: "Disco",
            of: "de", cores: "núcleos", core: "núcleo", light: "leve", moderate: "moderado", high: "alto",
            cpuLimit: "Limite de CPU", memLimit: "Limite de memória", diskLimit: "Limite de disco",
            changeLimits: "Alterar limites", noLimit: "Sem limite definido",
            noSnaps: "Nenhum ponto de restauração.", snapsOff: "Pontos indisponíveis.",
            restore: "Restaurar", createSnap: "Criar ponto de restauração", export: "Exportar",
            snapCreated: "Ponto criado", snapRestored: "Ponto restaurado",
            image: "Imagem", noAccounts: "Nenhum agente por assinatura conectado.", configure: "Configurar agentes",
            noActivity: "Nenhuma operação nesta sessão.", diskUsed: "Disco usado", snapCount: "Pontos de restauração",
            cleanTemp: "Limpar temporários", erase: "Apagar a máquina", stateLabel: "Estado"
        },
        en: {
            loading: "Checking", running: "Running", stopped: "Off", provisioning: "Preparing",
            error: "Needs attention", full: "Disk full", unavailable: "Unavailable", restoring: "Restoring",
            machine: "Machine", failed: "Operation failed", kicker: "// AGENT MACHINE",
            title: "A computer of your own, where agents work.",
            lead: "Subscription agents and Agent mode run here: they run commands, install tools and create files, apart from your Windows.",
            sections: "Sections", secA: "Your machine", secB: "Usage now", secC: "Settings", secD: "Recovery",
            secE: "What lives here", secF: "Activity", secG: "Maintenance",
            isolated: "Isolated inside this computer. It cannot see your Windows files. What happens there stays there.",
            subRunning: "Ready for your agents", subStopped: "Off. It starts when an agent needs it, or with the button.",
            subProvisioning: "Preparing the agent workspace. The first time takes a few minutes.",
            subError: "The machine reported a problem. Refresh, or shut it down and start it again.",
            subFull: "The machine's disk is full. Delete restore points or files inside it.",
            subUnavailable: "The agent workspace requires Incus. On Windows, check WSL and Incus.",
            subLoading: "Checking the machine state…",
            start: "Start", stop: "Shut down", restart: "Restart", refresh: "Refresh", soon: "Coming soon",
            confirmStop: "Shut down the machine and interrupt running tasks?",
            confirmRestore: "Restore this point and replace the current state of the machine?",
            lastActive: "Last activity", processes: "Processes", arch: "Architecture", name: "Name",
            every: "Refreshes every 5 s", cpu: "Processor", memory: "Memory", disk: "Disk",
            of: "of", cores: "cores", core: "core", light: "light", moderate: "moderate", high: "high",
            cpuLimit: "CPU limit", memLimit: "Memory limit", diskLimit: "Disk limit",
            changeLimits: "Change limits", noLimit: "No limit set",
            noSnaps: "No restore points.", snapsOff: "Restore points unavailable.",
            restore: "Restore", createSnap: "Create restore point", export: "Export",
            snapCreated: "Restore point created", snapRestored: "Restore point restored",
            image: "Image", noAccounts: "No subscription agent connected.", configure: "Configure agents",
            noActivity: "No operations in this session.", diskUsed: "Disk used", snapCount: "Restore points",
            cleanTemp: "Clear temporary files", erase: "Delete the machine", stateLabel: "Status"
        }
    };
    M.t = function (key) {
        var table = A.locale && A.locale.startsWith("pt") ? T.pt : T.en;
        return table[key] || T.en[key] || key;
    };
    M.stateLabel = function (state) { return M.t(T.en[state] ? state : "error"); };

    M.bytes = function (value) {
        if (typeof value !== "number" || value < 0) return "—";
        var units = ["B", "KB", "MB", "GB", "TB"], i = 0;
        while (value >= 1024 && i < units.length - 1) { value /= 1024; i++; }
        return value.toLocaleString(A.locale, {maximumFractionDigits: i > 1 ? 1 : 0}) + " " + units[i];
    };
    M.parseSize = function (text) {
        var match = /^\s*([\d.]+)\s*([kmgtp]?)(i?)b?\s*$/i.exec(String(text == null ? "" : text));
        if (!match) return null;
        var power = " kmgtp".indexOf(match[2].toLowerCase() || " ");
        return parseFloat(match[1]) * Math.pow(match[3] ? 1024 : 1000, Math.max(power, 0));
    };

    M.cores = function (limit) {
        var text = String(limit == null ? "" : limit).trim(), range = /^(\d+)-(\d+)$/.exec(text);
        if (/^\d+$/.test(text) && +text > 0) return +text;
        if (range) return +range[2] - +range[1] + 1;
        return navigator.hardwareConcurrency || 1;
    };

    function capabilities() {
        var cfg = A.productConfig || {};
        return (cfg.compiled_features || []).includes("agent-vm") && !cfg.agent_backend_unavailable;
    }
    async function request(path, body) {
        var response = await A.apiFetch(path, body === undefined ? {} :
            {method: "POST", headers: {"Content-Type": "application/json"}, body: JSON.stringify(body)});
        if (!response.ok) throw new Error(M.t("failed") + " (HTTP " + response.status + ")");
        return response.json();
    }
    M.request = request;

    function crumbMeta() {
        var crumb = document.querySelector(".gb-breadcrumb");
        if (!crumb) return;
        var node = crumb.querySelector('.gb-crumb-meta[data-for="machine"]');
        if (!node) {
            node = document.createElement("small");
            node.className = "gb-crumb-meta";
            node.dataset.for = "machine";
            crumb.appendChild(node);
        }
        node.dataset.state = M.model.status;
        node.textContent = M.stateLabel(M.model.status);
    }
    function footer() {
        var button = document.getElementById("gbNavMachine");
        var line = document.getElementById("gbRailMachineState");
        var r = M.model.resources || {};
        var summary = [M.stateLabel(M.model.status)];
        if (typeof r.memory_bytes === "number") summary.push(M.bytes(r.memory_bytes));
        if (M.model.cpu != null) summary.push(M.model.cpu + "%");
        if (button) {
            button.dataset.state = M.model.status;
            button.title = M.t("machine") + " · " + summary.join(" · ");
        }
        if (line) line.textContent = summary.join(" · ");
        crumbMeta();
    }

    A.refreshMachineStatus = async function (detail) {
        if (M.busy) return;
        var current = ++sequence;
        var next = {status: "unavailable", snapshots: [], accounts: []};
        try {
            if (capabilities()) {
                var vm = await request("/api/agent/sessions/current");
                next.row = vm.found && vm.item;
                next.status = next.row ? String(next.row.status).toLowerCase() : "stopped";
                if (next.row && detail) {
                    var targets = [
                        "/api/agent/vm/resources",
                        "/api/agent/sessions/" + encodeURIComponent(next.row.session_id) + "/snapshots",
                        "/api/ai-accounts"
                    ];
                    var results = await Promise.allSettled(targets.map(function (url) { return request(url); }));
                    next.resources = results[0].value || {};
                    next.snapshots = results[1].value && results[1].value.items;
                    next.accounts = results[2].value && results[2].value.accounts;
                    var r = next.resources;
                    if (r.status) next.status = String(r.status).toLowerCase() === "frozen" ? "error" : String(r.status).toLowerCase();
                    if (!T.en[next.status]) next.status = "error";
                    if (r.disk_total_bytes > 0 && r.disk_bytes >= r.disk_total_bytes * FULL_RATIO) next.status = "full";
                    if (typeof r.cpu_ns === "number") {
                        var now = performance.now();
                        var last = sample && sample.vm === next.row.vm_name ? sample : null;
                        if (last && r.cpu_ns >= last.ns && now > last.time)
                            next.cpu = Math.min(100, Math.round((r.cpu_ns - last.ns) / ((now - last.time) * 1e6 * M.cores(r.cpu_limit)) * 100));
                        else if (last) next.cpu = M.model.cpu;
                        sample = {time: now, ns: r.cpu_ns, vm: next.row.vm_name};
                    }
                }
            }
        } catch (err) { next.status = "error"; next.error = err.message; }
        if (sequence !== current) return;
        M.model = next; footer();
        if (A.currentView === "machine" && M.draw) M.draw();
    };

    M.operate = async function (label, url, body) {
        if (M.busy) return;
        M.busy = true; sequence++; M.model.error = ""; M.draw();
        try {
            await request(url, body);
            M.activity.unshift({time: new Date().toISOString(), label: label}); M.activity.splice(12);
        } catch (err) {
            M.busy = false; M.model.error = err.message; M.draw(); return;
        }
        M.busy = false; await A.refreshMachineStatus(true);
    };

    function poll() {
        if (A.currentView !== "machine") { clearInterval(timer); timer = null; return; }
        if (!document.hidden && !M.busy && M.model.row) A.refreshMachineStatus(true);
    }
    A.renderMachine = function () {
        if (M.draw) M.draw();
        A.refreshMachineStatus(true);
        if (!timer) timer = setInterval(poll, POLL_MS);
    };
})();

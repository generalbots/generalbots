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
    var poll = null, refreshing = null;
    var POLL_MS = 5000, POLL_BACKOFF = 30000;
    var FULL_RATIO = 0.95;

    var T = {
        pt: {
            loading: "Verificando…", running: "Ligada", stopped: "Desligada", provisioning: "Preparando",
            error: "Atenção", full: "Disco cheio", unavailable: "Indisponível", restoring: "Restaurando", starting: "Iniciando",
            machine: "Máquina Virtual", failed: "Falha na operação", kicker: "// MÁQUINA VIRTUAL DE AGENTES",
            title: "Um computador só seu, onde os agentes trabalham.",
            lead: "Os agentes por assinatura e o modo Agente rodam aqui: executam comandos, instalam ferramentas e geram arquivos, separados do seu Windows.",
            sections: "Seções", secA: "Sua máquina virtual", secB: "Uso agora", secC: "Ajustes", secD: "Restauração",
            secE: "O que vive nela", secF: "Atividade", secG: "Cuidado",
            isolated: "Isolada dentro deste computador. Ela não vê seus arquivos do Windows. O que acontece lá fica lá.",
            subRunning: "Pronta para os agentes", subStopped: "Desligada. Liga sozinha quando um agente precisar dela, ou pelo botão.",
            subProvisioning: "Preparando o espaço dos agentes. Isso leva alguns minutos na primeira vez.",
            subError: "A máquina virtual informou um problema. Atualize ou desligue e ligue de novo.",
            subFull: "O disco da máquina virtual está cheio. Apague pontos de restauração ou arquivos dentro dela.",
            subUnavailable: "O espaço dos agentes requer Incus. No Windows, verifique o WSL e o Incus.",
            subLoading: "Consultando o estado da máquina virtual…",
            subStarting: "A máquina virtual está iniciando. Isso leva alguns segundos.",
            noAnswer: "A máquina virtual não respondeu a tempo. Tentaremos de novo em instantes.",
            start: "Ligar", stop: "Desligar", restart: "Reiniciar", refresh: "Atualizar", soon: "Em breve",
            confirmStop: "Desligar a máquina virtual e interromper as tarefas em andamento?",
            confirmRestore: "Restaurar este ponto e substituir o estado atual da máquina virtual?",
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
            cleanTemp: "Limpar temporários", erase: "Apagar a máquina virtual", stateLabel: "Estado"
        },
        en: {
            loading: "Checking…", running: "Running", stopped: "Off", provisioning: "Preparing",
            error: "Needs attention", full: "Disk full", unavailable: "Unavailable", restoring: "Restoring", starting: "Starting",
            machine: "Virtual machine", failed: "Operation failed", kicker: "// AGENT VIRTUAL MACHINE",
            title: "A computer of your own, where agents work.",
            lead: "Subscription agents and Agent mode run here: they run commands, install tools and create files, apart from your Windows.",
            sections: "Sections", secA: "Your virtual machine", secB: "Usage now", secC: "Settings", secD: "Recovery",
            secE: "What lives here", secF: "Activity", secG: "Maintenance",
            isolated: "Isolated inside this computer. It cannot see your Windows files. What happens there stays there.",
            subRunning: "Ready for your agents", subStopped: "Off. It starts when an agent needs it, or with the button.",
            subProvisioning: "Preparing the agent workspace. The first time takes a few minutes.",
            subError: "The virtual machine reported a problem. Refresh, or shut it down and start it again.",
            subFull: "The virtual machine's disk is full. Delete restore points or files inside it.",
            subUnavailable: "The agent workspace requires Incus. On Windows, check WSL and Incus.",
            subLoading: "Checking the virtual machine state…",
            subStarting: "The virtual machine is starting. This takes a few seconds.",
            noAnswer: "The virtual machine did not answer in time. Trying again shortly.",
            start: "Start", stop: "Shut down", restart: "Restart", refresh: "Refresh", soon: "Coming soon",
            confirmStop: "Shut down the virtual machine and interrupt running tasks?",
            confirmRestore: "Restore this point and replace the current state of the virtual machine?",
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
            cleanTemp: "Clear temporary files", erase: "Delete the virtual machine", stateLabel: "Status"
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
    /* Reads carry a deadline (02b_requests.js); operations (POST) run as
       long as the server needs. */
    async function request(path, body) {
        if (body === undefined) {
            var res = await A.fetchJSON(path, null, 8000).catch(function (err) {
                throw new Error(err.timeout ? M.t("noAnswer") : err.message);
            });
            var state = A.vmState(res);
            if (state) return {status: state};
            if (!res.ok) throw new Error(M.t("failed") + " (HTTP " + res.status + ")");
            return res.data || {};
        }
        var response = await A.apiFetch(path,
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

    /* One refresh at a time: a second call while one runs gets the same
       answer. The reads go one after another, so a stalled one holds a
       single connection until its deadline. Resolves false on failure. */
    A.refreshMachineStatus = function (detail, force) {
        if (M.busy) return Promise.resolve(true);
        if (refreshing && !force) return refreshing;
        var run = refresh(detail).finally(function () { if (refreshing === run) refreshing = null; });
        refreshing = run;
        return run;
    };
    async function refresh(detail) {
        var current = ++sequence;
        var configLoaded = !!(A.productConfig);
        var next = {status: configLoaded ? "unavailable" : "loading", snapshots: [], accounts: []};
        try {
            if (capabilities()) {
                var vm = await request("/api/agent/sessions/current");
                next.row = vm.found && vm.item;
                next.status = vm.status && !vm.item ? vm.status : next.row ? String(next.row.status).toLowerCase() : "stopped";
                if (next.row && detail) {
                    var targets = [
                        "/api/agent/vm/resources",
                        "/api/agent/sessions/" + encodeURIComponent(next.row.session_id) + "/snapshots",
                        "/api/ai-accounts"
                    ];
                    var results = [];
                    for (var i = 0; i < targets.length; i++) {
                        results.push(await request(targets[i]).catch(function () { return null; }));
                    }
                    next.resources = results[0] || {};
                    next.snapshots = results[1] && results[1].items;
                    next.accounts = results[2] && results[2].accounts;
                    var r = next.resources;
                    if (r.status) next.status = String(r.status).toLowerCase() === "frozen" ? "error" : String(r.status).toLowerCase();
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
                if (!T.en[next.status]) next.status = "error";
            }
        } catch (err) { next.status = "error"; next.error = err.message; }
        if (sequence !== current) return true;
        M.model = next; footer();
        if (A.currentView === "machine" && M.draw) M.draw();
        return !next.error;
    }

    /** A state learnt elsewhere (the sidebar's processor poll). "running"
        only replaces a state that said the machine was not running. */
    M.noteState = function (state) {
        if (!state || M.busy || refreshing) return;
        var s = String(state).toLowerCase() === "frozen" ? "error" : String(state).toLowerCase();
        if (!T.en[s] || s === M.model.status) return;
        if (s === "running" && ["stopped", "starting", "unavailable", "loading"].indexOf(M.model.status) < 0) return;
        M.model.status = s;
        if (s !== "running") M.model.cpu = null;
        footer();
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
        M.busy = false; await A.refreshMachineStatus(true, true);
    };

    /* Every 5 s while the page is open and the window visible; 30 s after
       a failure. A machine that is off is not polled until it starts. */
    function tick() {
        if (M.busy || !(M.model.row || M.model.status === "starting")) return POLL_MS;
        return A.refreshMachineStatus(true).then(function (ok) { return ok ? POLL_MS : POLL_BACKOFF; });
    }
    A.renderMachine = function () {
        if (M.draw) M.draw();
        if (!poll) poll = A.poller(tick, {every: POLL_MS, backoff: POLL_BACKOFF,
            when: function () { return A.currentView === "machine"; }});
        A.refreshMachineStatus(true);
        poll.start(POLL_MS);
    };
})();

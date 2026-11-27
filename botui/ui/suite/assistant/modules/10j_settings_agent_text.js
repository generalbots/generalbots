"use strict";
/**
 * 10j_settings_agent_text.js -- Agent runtime text: short chip labels,
 * body descriptions, error messages with remediation, step names, policy
 * text, and formatting helpers.
 *
 * Loaded before 10k_settings_agent_runtime.js, which reads A._artText.
 */
(function () {
var A = window.GBAssistant;

function tr(pt, en) { return A.locale && A.locale.startsWith("pt") ? pt : en; }
function esc(s) {
    return String(s == null ? "" : s)
        .replace(/&/g, "&amp;").replace(/</g, "&lt;")
        .replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}

/* ---- formatting helpers ---- */

function fmtBytes(bytes) {
    if (bytes == null || bytes <= 0) { return ""; }
    var isPt = A.locale && A.locale.startsWith("pt");
    if (bytes >= 1073741824) {
        var gb = (bytes / 1073741824).toFixed(1);
        if (isPt) { gb = gb.replace(".", ","); }
        return gb + " GB";
    }
    var mb = (bytes / 1048576).toFixed(0);
    return mb + " MB";
}

/* ---- short chip labels (one line, no sentence) ---- */

function chipLabel(state) {
    var map = {
        unsupported:          tr("Não disponível neste computador", "Not available on this computer"),
        policy_blocked:       tr("Bloqueado pela organização", "Blocked by your organization"),
        needs_virtualization: tr("Falta ativar a virtualização", "Virtualization is off"),
        needs_wsl:            tr("Não instalado", "Not installed"),
        not_installed:        tr("Não instalado", "Not installed"),
        needs_reboot:         tr("Falta reiniciar", "Restart needed"),
        installing:           tr("Instalando", "Installing"),
        paused:               tr("Instalação pausada", "Installation paused"),
        broken:               tr("Instalação com problema", "Install problem"),
        ready:                tr("Instalado", "Installed")
    };
    return map[state] || state;
}

/* badge CSS class per state: neutral, gb-badge-warn, gb-badge-danger, gb-badge-active, gb-badge-ok */
function chipClass(state) {
    var map = {
        unsupported: "gb-badge-danger", policy_blocked: "gb-badge-danger",
        broken: "gb-badge-danger", paused: "gb-badge-warn",
        needs_virtualization: "gb-badge-warn", needs_wsl: "gb-badge-warn",
        needs_reboot: "gb-badge-warn", not_installed: "gb-badge-warn",
        installing: "gb-badge-active", ready: "gb-badge-ok"
    };
    return map[state] || "";
}

/* ---- body descriptions (plain language for a rookie) ---- */

function stateDesc(state, status) {
    switch (state) {
    case "unsupported":
        return tr("Este computador não tem os requisitos necessários para o Modo Agente. É necessário um computador com Windows 10 ou 11 (64 bits) que suporte virtualização.",
                  "This computer does not meet the requirements for Agent Mode. A computer with Windows 10 or 11 (64-bit) with virtualisation support is needed.");
    case "policy_blocked":
        return tr("Uma configuração da sua organização impede a instalação do Modo Agente. Fale com o time de TI para liberar o recurso.",
                  "A setting from your organization prevents Agent Mode installation. Talk to your IT team to enable this feature.");
    case "needs_virtualization":
        return tr("O Modo Agente precisa de uma configuração do próprio computador (virtualização de hardware) que está desativada. Essa configuração é alterada pelo fabricante, pelo time de TI ou no menu de inicialização do computador (BIOS/UEFI).",
                  "Agent Mode needs a hardware setting (hardware virtualisation) that is currently off. This setting is changed by the manufacturer, IT team, or in the computer startup menu (BIOS/UEFI).");
    case "needs_wsl": case "not_installed":
        return installDesc(status);
    case "needs_reboot":
        return tr("Reinicie o computador. Depois abra o General Bots Lite e volte a esta tela: a instalação continua de onde parou.",
                  "Restart the computer. Then open General Bots Lite and come back to this screen: the installation will continue where it left off.");
    case "installing":
        return tr("A instalação está em andamento.",
                  "Installation is in progress.");
    case "paused":
        return tr("A instalação foi pausada. O que já foi baixado está guardado.",
                  "Installation was paused. What has been downloaded is kept.");
    case "broken":
        return ""; /* error text replaces this */
    case "ready":
        return tr("O Modo Agente está instalado e funcionando. Você pode usar assistentes por assinatura no menu de modelos.",
                  "Agent Mode is installed and running. You can use subscription assistants in the model menu.");
    default: return "";
    }
}

function installDesc(status) {
    var text = tr(
        "O Modo Agente usa um recurso do Windows para rodar um ambiente Linux isolado. Ele ainda não está ativado neste computador.",
        "Agent Mode uses a Windows feature to run an isolated Linux environment. It is not yet set up on this computer.");
    var parts = [];
    if (status && status.download_bytes) {
        parts.push("Download: " + fmtBytes(status.download_bytes));
    }
    if (status && status.disk_bytes) {
        parts.push(tr("Espaço em disco: ", "Disk space: ") + fmtBytes(status.disk_bytes));
    }
    if (parts.length) { text += "\n" + parts.join(" | "); }
    if (status && status.needs_admin) {
        text += "\n" + tr("O Windows vai pedir permissão de administrador uma vez e pode pedir para reiniciar.",
                          "Windows will ask for administrator permission once and may ask for a restart.");
    }
    return text;
}

/* ---- error text with remediation per code ---- */

function errorText(code, status) {
    var detail = (status && status.last_error && status.last_error.detail) || "";
    var isCert = code === "offline" && /certificate/i.test(detail);
    var map = {
        cancelled: tr("A instalação foi cancelada.",
                       "Installation was cancelled."),
        offline:   isCert
                   ? tr("A rede deste computador inspeciona conexões seguras e o download foi recusado. Fale com o time de TI.",
                        "This computer’s network inspects secure connections and the download was refused. Talk to your IT team.")
                   : tr("Sem conexão com a internet. Verifique a rede e tente de novo.",
                        "No internet connection. Check the network and try again."),
        checksum:  tr("O download veio corrompido. Ele será baixado de novo automaticamente.",
                       "The download came corrupted. It will be downloaded again automatically."),
        disk_full: tr("Não há espaço suficiente em disco.",
                       "Not enough disk space.") +
                   (status && status.disk_bytes
                    ? " " + tr("São necessários pelo menos ", "At least ") + fmtBytes(status.disk_bytes) + tr(" livres.", " free.")
                    : ""),
        wsl_failed:        tr("A instalação do suporte a Linux falhou. Tente de novo. Se o problema repetir, remova e instale de novo.",
                               "Linux support installation failed. Try again. If the problem repeats, remove and reinstall."),
        import_failed:     tr("A importação do ambiente falhou. Tente de novo. Se o problema repetir, remova e instale de novo.",
                               "Environment import failed. Try again. If the problem repeats, remove and reinstall."),
        first_boot_failed: tr("A primeira inicialização do ambiente falhou. Tente de novo. Se o problema repetir, remova e instale de novo.",
                               "First boot of the environment failed. Try again. If the problem repeats, remove and reinstall."),
        needs_reboot:      tr("Reinicie o computador e tente de novo.",
                               "Restart the computer and try again."),
        unknown:           tr("Erro desconhecido.",
                               "Unknown error.")
    };
    return map[code] || map.unknown;
}

/* ---- step names and numbering ---- */

var STEPS = ["wsl", "rootfs_download", "distro_import", "first_boot",
             "image_download", "image_import", "finish"];

function stepName(step) {
    var map = {
        wsl:             tr("Suporte a Linux", "Linux support"),
        rootfs_download: tr("Baixando o ambiente", "Downloading the environment"),
        distro_import:   tr("Importando o ambiente", "Importing the environment"),
        first_boot:      tr("Primeira inicialização", "First boot"),
        image_download:  tr("Baixando a área de trabalho", "Downloading the workspace image"),
        image_import:    tr("Importando a área de trabalho", "Importing the workspace image"),
        finish:          tr("Concluindo", "Finishing")
    };
    return map[step] || step;
}

function stepIndex(step) {
    var idx = STEPS.indexOf(step);
    return idx >= 0 ? idx + 1 : null;
}

function stepHasBytes(step) {
    return step === "rootfs_download" || step === "image_download";
}

/* ---- progress HTML ---- */

function progressHtml(status) {
    var step = status.progress_step || status.step;
    var percent = status.progress_percent != null ? status.progress_percent : (status.percent || 0);
    var doneBytes = status.progress_done_bytes || status.done_bytes || 0;
    var totalBytes = status.progress_total_bytes || status.total_bytes || 0;
    var idx = stepIndex(step);
    var html = '<div class="gb-art-progress">';
    if (idx) {
        html += '<p class="gb-set-desc" style="margin-bottom:4px">' +
            esc(tr("Etapa ", "Step ") + idx + tr(" de ", " of ") + STEPS.length + ": ") +
            '<strong>' + esc(stepName(step)) + '</strong></p>';
    }
    var hasBytes = stepHasBytes(step);
    var indeterminate = !hasBytes || percent <= 0;
    html += '<div class="gb-dl-progress"><div class="gb-dl-bar' +
        (indeterminate ? ' gb-dl-bar-wait' : '') +
        '"><i style="width:' + (indeterminate ? '0' : Math.min(100, Math.round(percent))) + '%"></i></div></div>';
    if (hasBytes && totalBytes > 0) {
        html += '<p class="gb-dl-text" style="margin-top:2px">' +
            fmtBytes(doneBytes) + ' / ' + fmtBytes(totalBytes) +
            (percent > 0 ? ' (' + Math.round(percent) + '%)' : '') + '</p>';
    } else if (!hasBytes) {
        html += '<p class="gb-dl-text" style="margin-top:2px">' +
            esc(tr("Isso pode levar alguns minutos.", "This may take a few minutes.")) + '</p>';
    }
    html += '</div>';
    return html;
}

/* ---- IT policy text ---- */

function policyText() {
    return tr(
        "Para ativar o Modo Agente neste computador, é necessário que o departamento de TI libere a instalação do Windows Subsystem for Linux (WSL2) e a criação de ambientes virtuais. Solicite ao time de TI que habilite a funcionalidade \"Subsistema do Windows para Linux\" e \"Plataforma de Máquina Virtual\" nas Funcionalidades do Windows, e que a política de grupo não bloqueie o comando wsl.exe.",
        "To enable Agent Mode on this computer, the IT department needs to allow Windows Subsystem for Linux (WSL2) installation and virtual environment creation. Ask the IT team to enable \"Windows Subsystem for Linux\" and \"Virtual Machine Platform\" in Windows Features, and ensure group policy does not block wsl.exe."
    );
}

/* ---- technical details for install ---- */

function techDetails() {
    return tr(
        "O Modo Agente instala o WSL2 (Windows Subsystem for Linux), importa uma distribuição Linux leve (GeneralBotsLite) e configura a área de trabalho dos agentes. Os dados ficam em %LOCALAPPDATA%\\generalbots\\stack\\agent-runtime.",
        "Agent Mode installs WSL2 (Windows Subsystem for Linux), imports a lightweight Linux distribution (GeneralBotsLite) and sets up the agent workspace. Data is stored in %LOCALAPPDATA%\\generalbots\\stack\\agent-runtime.");
}

/* ---- expose ---- */

A._artText = {
    tr: tr, esc: esc,
    fmtBytes: fmtBytes,
    chipLabel: chipLabel, chipClass: chipClass,
    stateDesc: stateDesc, errorText: errorText,
    stepName: stepName, policyText: policyText,
    progressHtml: progressHtml, techDetails: techDetails
};

})();

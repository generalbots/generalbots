"use strict";
/**
 * dev/fixtures.js - Sample activity events matching the server contract.
 * Loaded only with ?fixtures=1.
 */
(function () {
    if (!/[?&]fixtures=1/.test(location.search)) return;
    var A = window.GBAssistant;
    function wait() {
        if (!A || !A.showConversation || !A.renderBlockStack) { setTimeout(wait, 200); return; }
        run();
    }
    function run() {
        A.showConversation();
        var msgs = document.getElementById("gbMessages");
        if (!msgs) return; msgs.innerHTML = "";
        addUser(msgs, "Analise o relatorio trimestral e a planilha de metas.", [
            { name: "relatorio-q3-2026.pdf", type: "application/pdf", size: 2456000 },
            { name: "metas-equipe.xlsx", type: "application/vnd.ms-excel", size: 184320 },
            { name: "foto-equipe.jpg", type: "image/jpeg", size: 5242880 }
        ]);
        var t1 = addBot(msgs);
        stack(t1, [
            { id:"a1", kind:"attachment_read", status:"completed", summary:{ name:"relatorio-q3-2026.pdf", type:"application/pdf", size:2456000, chars_read:14200, truncated:false } },
            { id:"a2", kind:"attachment_read", status:"completed", summary:{ name:"metas-equipe.xlsx", type:"application/vnd.ms-excel", size:184320, chars_read:3200, truncated:true } },
            { id:"a3", kind:"kb_search", status:"completed", summary:{ query:"avaliacao desempenho", results:[
                { document:"Manual de Avaliacao", excerpt:"A avaliacao trimestral deve considerar indicadores..." },
                { document:"Plano de Metas 2026", excerpt:"As metas sao derivadas dos objetivos estrategicos..." }
            ], total:2 } },
            { id:"a4", kind:"drive_list", status:"completed", summary:{ path:"/Relatorios/Q3/", entries:[
                { name:"relatorio-q3.pdf", size:2456000, kind:"file" },
                { name:"dashboard.html", size:89000, kind:"file" },
                { name:"Anexos", kind:"folder" }
            ], total:3 } },
            { id:"a5", kind:"thinking", status:"completed", summary:{ elapsed:4, reasoning:"Produtividade acima da meta, qualidade 2 pontos abaixo." } }
        ]);
        body(t1, "## Analise Q3\n\nCrescimento de **12%**.\n\n| Ind. | Meta | Real |\n|---|---|---|\n| Produtividade | 85% | 91% |\n| Qualidade | 90% | 88% |\n\n```python\ndf = pd.read_excel('metas.xlsx')\nprint(df.describe())\n```");
        addUser(msgs, "Busque documentos sobre expansao.");
        var t2 = addBot(msgs);
        stack(t2, [{ id:"a6", kind:"drive_search", status:"started", summary:{ query:"plano expansao", entries:[], total:0 } }]);
        addUser(msgs, "Envie resumo por e-mail.");
        var t3 = addBot(msgs);
        stack(t3, [{ id:"a7", kind:"permission", status:"needs_input", summary:{ action:"email.send", request_id:"perm-001", detail:"POST /api/email/send" } }]);
        addUser(msgs, "Execute o script de backup.");
        var t4 = addBot(msgs);
        stack(t4, [
            { id:"a8", kind:"notice", status:"completed", summary:{ message:"vm_exec indisponivel" } },
            { id:"a9", kind:"error", status:"failed", summary:{ message:"Modelo nao respondeu." } }
        ]);
        addUser(msgs, "Classifique.", [{ name:"nota.txt", type:"text/plain", size:73 }]);
        var t5 = addBot(msgs);
        stack(t5, [{ id:"a10", kind:"tool_run", status:"completed", summary:{ tool:"classify_media", outcome:"Relatorio Gerencial" } }]);
        body(t5, "Classificado como **Relatorio Gerencial**.");
        addUser(msgs, "Etapa desconhecida.");
        var t6 = addBot(msgs);
        stack(t6, [{ id:"a11", kind:"exotic_new_thing", status:"completed", summary:{ foo:"bar" } }]);
        body(t6, "Feito.");
    }
    function addUser(c, text, att) {
        var el = document.createElement("div"); el.className = "gb-msg-user"; el.dataset.originalContent = text;
        var b = document.createElement("div"); b.className = "gb-msg-user-bubble"; b.textContent = text; el.appendChild(b);
        if (att && att.length) {
            var cards = document.createElement("div"); cards.className = "gb-attach-list";
            att.forEach(function (f) { cards.appendChild(A.attachCard(f, { removable: false })); });
            el.appendChild(cards);
        }
        c.appendChild(el);
    }
    function addBot(c) { var el = document.createElement("div"); el.className = "gb-msg-assistant"; c.appendChild(el); return el; }
    function stack(turn, evts) { var s = A.renderBlockStack(evts); if (s) turn.appendChild(s); }
    function body(turn, md) {
        var d = document.createElement("div"); d.className = "gb-msg-body";
        d.innerHTML = A.mdToHtml(md); A.enhanceCodeBlocks(d); turn.appendChild(d);
        A.addMsgToolbar(turn, md, false);
    }
    setTimeout(wait, 500);
})();

//! Task specifications for the Vibe benchmark suite (Issue #800).
//!
//! Each spec is authored once and localized at build time into English and
//! Brazilian Portuguese entries. Keeping the data here separates content
//! from the deterministic builder in `vibe_suite`.

/// One authored task, localized into English and Brazilian Portuguese.
#[derive(Debug, Clone)]
pub(crate) struct TaskSpec {
    /// English prompt.
    pub prompt_en: &'static str,
    /// Brazilian Portuguese prompt.
    pub prompt_pt: &'static str,
    /// Phrases required in the English response.
    pub contains_en: &'static [&'static str],
    /// Phrases required in the Brazilian Portuguese response.
    pub contains_pt: &'static [&'static str],
    /// Phrases that must never appear in the response.
    pub forbid: &'static [&'static str],
    /// Minimum tokens hint, when relevant.
    pub min_tokens: Option<u32>,
    /// Maximum tokens hint, when relevant.
    pub max_tokens: Option<u32>,
    /// #817 — minimum tool calls required for harness-tagged entries.
    pub requires_tool_calls: Option<u32>,
}

macro_rules! spec {
    ($en:expr, $pt:expr, $ce:expr, $cp:expr) => {
        TaskSpec {
            prompt_en: $en,
            prompt_pt: $pt,
            contains_en: $ce,
            contains_pt: $cp,
            forbid: &[],
            min_tokens: None,
            max_tokens: None,
            requires_tool_calls: None,
        }
    };
}

/// Like `spec!` but marks the entry as harness-tagged in the suite builder
/// (#800): the live agent must invoke at least two real harness tools
/// (file/read, test/run, git/*, shell) — a chat-only answer fails the floor.
macro_rules! harness_spec {
    ($en:expr, $pt:expr, $ce:expr, $cp:expr) => {
        TaskSpec {
            prompt_en: $en,
            prompt_pt: $pt,
            contains_en: $ce,
            contains_pt: $cp,
            forbid: &[],
            min_tokens: None,
            max_tokens: None,
            requires_tool_calls: Some(2),
        }
    };
}

/// Twenty authored tasks per Vibe use case.
pub(crate) fn use_case_tasks(use_case: &str) -> Vec<TaskSpec> {
    match use_case {
        "software_development" => vec![
            harness_spec!("Write a function that reverses a string in Python.", "Escreva uma função que inverte uma string em Python.", &["def ", "reverse"], &["def ", "inverter"]),
            spec!("Explain the difference between the stack and the heap in memory.", "Explique a diferença entre stack e heap na memória.", &["stack", "heap"], &["pilha", "stack", "heap"]),
            spec!("Here is code: for i in range(10): print(i.upper()). Find the bug and explain the fix.", "Aqui está código: for i in range(10): print(i.upper()). Encontre o erro e explique a correção.", &["bug", "fix"], &["erro", "correção"]),
            harness_spec!("Write SQL to select all users created in the last month.", "Escreva SQL para selecionar todos os usuários criados no último mês.", &["SELECT", "FROM"], &["SELECT", "FROM"]),
            spec!("What is a unit test and what are its three typical parts?", "O que é um teste unitário e quais são suas três partes típicas?", &["unit test"], &["teste unitário"]),
            spec!("Explain the difference between git rebase and git merge.", "Explique a diferença entre git rebase e git merge.", &["rebase", "merge"], &["rebase", "merge"]),
            spec!("Write a Python list comprehension that doubles each number in a list.", "Escreva uma compreensão de lista em Python que dobre cada número de uma lista.", &["n * 2", "for"], &["* 2", "for"]),
            spec!("What does Big-O notation describe in algorithm analysis?", "O que a notação Big-O descreve na análise de algoritmos?", &["complexity"], &["complexidade"]),
            spec!("Explain HTTP status codes 200, 404, and 500.", "Explique os códigos de status HTTP 200, 404 e 500.", &["200", "404", "500"], &["200", "404", "500"]),
            spec!("Write a function that checks whether a number is prime.", "Escreva uma função que verifique se um número é primo.", &["prime"], &["primo"]),
            spec!("What is an API and how does it differ from a library?", "O que é uma API e como ela difere de uma biblioteca?", &["API"], &["API"]),
            spec!("What is a database index and why is it useful?", "O que é um índice de banco de dados e por que ele é útil?", &["index"], &["índice"]),
            spec!("What is continuous integration in software engineering?", "O que é integração contínua em engenharia de software?", &["automated", "build"], &["automatizado", "build"]),
            spec!("Write a regex pattern that validates a simple email address.", "Escreva um padrão regex que valide um endereço de e-mail simples.", &["@", "regex"], &["@", "regex"]),
            spec!("What is a deadlock in concurrent programming?", "O que é um deadlock em programação concorrente?", &["deadlock"], &["deadlock"]),
            spec!("What is dependency injection and why is it used?", "O que é injeção de dependência e por que ela é usada?", &["dependency"], &["dependência"]),
            spec!("Implement a binary search function over a sorted array.", "Implemente uma função de busca binária sobre um array ordenado.", &["binary search"], &["busca binária"]),
            spec!("What is the difference between TCP and UDP?", "Qual é a diferença entre TCP e UDP?", &["TCP", "UDP"], &["TCP", "UDP"]),
            spec!("Write a bash command that lists files larger than 100MB.", "Escreva um comando bash que liste arquivos maiores que 100MB.", &["find", "size"], &["find", "tamanho"]),
            spec!("What is a JSON Web Token and how is it used for authentication?", "O que é um JSON Web Token e como ele é usado para autenticação?", &["JWT", "token"], &["JWT", "token"]),
        ],
        "customer_support" => vec![
            harness_spec!("A customer wants to open a new support ticket. What information should you collect?", "Um cliente quer abrir um novo ticket de suporte. Quais informações você deve coletar?", &["ticket", "description"], &["ticket", "descrição"]),
            harness_spec!("Explain the standard refund policy to a customer.", "Explique a política de reembolso padrão ao cliente.", &["refund"], &["reembolso"]),
            spec!("A customer requests to speak with a human agent. How do you proceed?", "Um cliente pede para falar com um atendente humano. Como você procede?", &["transfer", "human"], &["transferir", "humano"]),
            spec!("A customer asks about the status of their order. What steps do you take?", "Um cliente pergunta sobre o status do pedido dele. Quais passos você toma?", &["order", "status"], &["pedido", "status"]),
            spec!("Explain the product return process to a customer.", "Explique o processo de devolução de produto ao cliente.", &["return"], &["devolução"]),
            spec!("Walk a customer through resetting their account password.", "Oriente um cliente a redefinir a senha da conta dele.", &["password", "reset"], &["senha", "redefinir"]),
            spec!("A customer wants to cancel their subscription. What is the process?", "Um cliente quer cancelar a assinatura dele. Qual é o processo?", &["cancel", "subscription"], &["cancelar", "assinatura"]),
            spec!("A customer complains that their shipment is late. How do you respond?", "Um cliente reclama que a entrega está atrasada. Como você responde?", &["apolog", "shipment"], &["desculpa", "entrega"]),
            spec!("Is a two-year-old laptop still under warranty? Explain how warranty works.", "Um notebook de dois anos ainda está na garantia? Explique como a garantia funciona.", &["warranty"], &["garantia"]),
            spec!("What support channels are available and how can a customer reach each one?", "Quais canais de suporte estão disponíveis e como o cliente acessa cada um?", &["email", "phone"], &["e-mail", "telefone"]),
            spec!("A customer's payment failed. List the steps to troubleshoot.", "O pagamento de um cliente falhou. Liste os passos para diagnosticar.", &["payment", "card"], &["pagamento", "cartão"]),
            spec!("A customer asks how their personal data is protected. What do you say?", "Um cliente pergunta como os dados pessoais dele são protegidos. O que você responde?", &["privacy", "data"], &["privacidade", "dados"]),
            spec!("What are the business hours for support?", "Quais são os horários de funcionamento do suporte?", &["hours"], &["horário"]),
            spec!("Explain the loyalty program benefits to a customer.", "Explique os benefícios do programa de fidelidade ao cliente.", &["points", "loyalty"], &["pontos", "fidelidade"]),
            spec!("A customer wants to redeem a gift code. Explain the steps.", "Um cliente quer resgatar um código de presente. Explique os passos.", &["code", "redeem"], &["código", "resgatar"]),
            spec!("A customer requests a copy of their invoice. How do you provide it?", "Um cliente solicita uma cópia da nota fiscal dele. Como você fornece?", &["invoice"], &["nota fiscal"]),
            spec!("A customer needs help installing the mobile app. Give setup guidance.", "Um cliente precisa de ajuda para instalar o aplicativo. Dê instruções de configuração.", &["install", "app"], &["instalar", "aplicativo"]),
            spec!("The customer insists on speaking to a manager. How do you de-escalate?", "O cliente insiste em falar com um gerente. Como você reduz a escalação?", &["manager", "escalat"], &["gerente", "escalar"]),
            spec!("A customer wants to suggest a new feature. Explain the process.", "Um cliente quer sugerir um novo recurso. Explique o processo.", &["feature", "feedback"], &["recurso", "feedback"]),
            spec!("How should a complaint about a defective product be handled end to end?", "Como uma reclamação sobre um produto defeituoso deve ser tratada de ponta a ponta?", &["replace", "record"], &["substituir", "registrar"]),
        ],
        "financial_analysis" => vec![
            harness_spec!("Explain the difference between net income and gross income.", "Explique a diferença entre lucro líquido e renda bruta.", &["net", "gross"], &["líquido", "bruto"]),
            spec!("List the main components of an income statement.", "Liste os principais componentes de uma demonstração de resultados.", &["revenue", "expenses"], &["receitas", "despesas"]),
            harness_spec!("What is a balance sheet and what does it show?", "O que é um balanço patrimonial e o que ele mostra?", &["assets", "liabilities"], &["ativos", "passivos"]),
            spec!("What is the purpose of a cash flow statement?", "Qual é o propósito de uma demonstração de fluxo de caixa?", &["cash", "flow"], &["caixa", "fluxo"]),
            spec!("Define EBITDA and explain why companies use it.", "Defina EBITDA e explique por que as empresas o usam.", &["EBITDA"], &["EBITDA"]),
            spec!("What is the difference between gross margin and net margin?", "Qual é a diferença entre margem bruta e margem líquida?", &["margin"], &["margem"]),
            spec!("What does the current ratio measure in liquidity analysis?", "O que o índice de liquidez corrente mede?", &["current assets", "current liabilities"], &["ativo circulante", "passivo circulante"]),
            spec!("Explain the difference between depreciation and amortization.", "Explique a diferença entre depreciação e amortização.", &["depreciation", "amortization"], &["depreciação", "amortização"]),
            spec!("What is variance analysis and when is it used?", "O que é análise de variação e quando ela é usada?", &["variance", "budget"], &["variação", "orçamento"]),
            spec!("What is the break-even point and how is it calculated?", "O que é o ponto de equilíbrio e como ele é calculado?", &["break-even"], &["equilíbrio"]),
            spec!("What is the difference between CAPEX and OPEX?", "Qual é a diferença entre CAPEX e OPEX?", &["CAPEX", "OPEX"], &["CAPEX", "OPEX"]),
            spec!("Define working capital and explain its importance.", "Defina capital de giro e explique sua importância.", &["working capital"], &["capital de giro"]),
            spec!("What is revenue recognition and why is it important?", "O que é reconhecimento de receita e por que é importante?", &["revenue"], &["receita"]),
            spec!("What is portfolio diversification and what risk does it reduce?", "O que é diversificação de carteira e qual risco ela reduz?", &["risk"], &["risco"]),
            spec!("Describe the basic steps to build a simple revenue forecast.", "Descreva os passos básicos para construir uma previsão simples de receita.", &["forecast"], &["previsão"]),
            spec!("How is CAGR calculated and what does it represent?", "Como o CAGR é calculado e o que ele representa?", &["CAGR"], &["CAGR"]),
            spec!("What is goodwill on a balance sheet?", "O que é goodwill em um balanço patrimonial?", &["goodwill", "acquisition"], &["goodwill", "aquisição"]),
            spec!("List the typical components of the cost of goods sold.", "Liste os componentes típicos do custo dos produtos vendidos.", &["materials", "labor"], &["materiais", "mão de obra"]),
            spec!("Describe the main steps of a financial statement audit.", "Descreva as principais etapas de uma auditoria de demonstrações financeiras.", &["audit"], &["auditoria"]),
            spec!("What does return on equity measure and how is it computed?", "O que o retorno sobre o patrimônio mede e como é calculado?", &["return on equity", "shareholders"], &["retorno sobre o patrimônio", "acionistas"]),
        ],
        _ => vec![],
    }
}

/// Forty-five authored general capability tasks shared by every use case.
pub(crate) fn general_tasks() -> Vec<TaskSpec> {
    vec![
        harness_spec!("Hello! How are you today?", "Olá! Como você está hoje?", &["glad", "help"], &["alegria", "ajudar"]),
        spec!("What can you help me with?", "Com o que você pode me ajudar?", &["help", "questions"], &["ajudar", "perguntas"]),
        spec!("Summarize this in one sentence: The bot assists customers with orders, refunds, and technical issues.", "Resuma em uma frase: O bot ajuda clientes com pedidos, reembolsos e problemas técnicos.", &["customers"], &["clientes"]),
        spec!("List three benefits of automation for businesses.", "Liste três benefícios da automação para empresas.", &["efficiency", "cost"], &["eficiência", "custo"]),
        spec!("Translate the word goodbye to Portuguese and the word obrigado to English.", "Traduza a palavra goodbye para português e a palavra obrigado para inglês.", &["adeus", "thank you"], &["adeus", "thank you"]),
        spec!("What is 15 percent of 200?", "Quanto é 15 por cento de 200?", &["30"], &["30"]),
        spec!("Convert 100 kilometers to miles.", "Converta 100 quilômetros para milhas.", &["62"], &["62"]),
        spec!("Write a short professional email requesting a meeting next week.", "Escreva um e-mail profissional curto solicitando uma reunião na próxima semana.", &["meeting"], &["reunião"]),
        spec!("What is the capital of France?", "Qual é a capital da França?", &["Paris"], &["Paris"]),
        spec!("Draft a polite message declining an invitation.", "Redija uma mensagem educada recusando um convite.", &["thank you", "unable"], &["obrigado", "impossível"]),
        spec!("Give three examples of renewable energy sources.", "Dê três exemplos de fontes de energia renovável.", &["solar", "wind"], &["solar", "eólica"]),
        spec!("What is the main cause of the seasons on Earth?", "Qual é a principal causa das estações do ano na Terra?", &["axis", "tilt"], &["eixo", "inclinação"]),
        spec!("Suggest a 3-step plan to learn a new language.", "Sugira um plano de 3 passos para aprender um novo idioma.", &["practice"], &["prática"]),
        spec!("Explain what a keyboard shortcut is and give one example.", "Explique o que é um atalho de teclado e dê um exemplo.", &["Ctrl"], &["Ctrl"]),
        spec!("How many days are in a leap year?", "Quantos dias tem um ano bissexto?", &["366"], &["366"]),
        spec!("Write a to-do list for a product launch week.", "Escreva uma lista de tarefas para a semana de lançamento de um produto.", &["launch", "test"], &["lançamento", "teste"]),
        spec!("What is the largest planet in our solar system?", "Qual é o maior planeta do nosso sistema solar?", &["Jupiter"], &["Júpiter"]),
        spec!("Explain the difference between weather and climate.", "Explique a diferença entre tempo e clima.", &["short-term", "long-term"], &["curto prazo", "longo prazo"]),
        spec!("Give an example of a healthy breakfast.", "Dê um exemplo de café da manhã saudável.", &["protein"], &["proteína"]),
        spec!("What should you do if you receive a suspicious email attachment?", "O que você deve fazer se receber um anexo de e-mail suspeito?", &["delete", "sender"], &["excluir", "remetente"]),
        spec!("Describe a backup strategy for important files.", "Descreva uma estratégia de backup para arquivos importantes.", &["backup"], &["backup"]),
        spec!("What is two-factor authentication?", "O que é autenticação de dois fatores?", &["two", "code"], &["dois", "código"]),
        spec!("Recommend a brief warm-up before exercising.", "Recomende um aquecimento breve antes de se exercitar.", &["stretch"], &["alongamento"]),
        spec!("Name three search engines.", "Cite três mecanismos de busca.", &["Google"], &["Google"]),
        spec!("Explain why drinking water is important.", "Explique por que beber água é importante.", &["hydrat"], &["hidratação"]),
        spec!("Write a friendly reminder message for an overdue invoice.", "Escreva uma mensagem amigável de lembrete para uma fatura vencida.", &["invoice", "payment"], &["fatura", "pagamento"]),
        spec!("What is the square root of 144?", "Qual é a raiz quadrada de 144?", &["12"], &["12"]),
        spec!("Give two tips for reducing screen time.", "Dê duas dicas para reduzir o tempo de tela.", &["break"], &["pausas"]),
        spec!("Convert 25 degrees Celsius to Fahrenheit.", "Converta 25 graus Celsius para Fahrenheit.", &["77"], &["77"]),
        spec!("Explain what a cookie is in web browsing.", "Explique o que é um cookie na navegação web.", &["website", "data"], &["site", "dados"]),
        spec!("What are the three primary colors?", "Quais são as três cores primárias?", &["red", "blue", "yellow"], &["vermelho", "azul", "amarelo"]),
        spec!("Describe how to set a strong password.", "Descreva como definir uma senha forte.", &["length", "symbols"], &["comprimento", "símbolos"]),
        spec!("Suggest a good structure for a weekly status report.", "Sugira uma boa estrutura para um relatório semanal de status.", &["progress", "blockers"], &["progresso", "impedimentos"]),
        spec!("What time zone does UTC stand for?", "O que significa o fuso horário UTC?", &["Universal", "Coordinated"], &["Universal", "Coordenado"]),
        spec!("Name three Git commands used every day.", "Cite três comandos Git usados diariamente.", &["commit", "push"], &["commit", "push"]),
        spec!("Give one advantage and one disadvantage of working remotely.", "Dê uma vantagem e uma desvantagem do trabalho remoto.", &["flexibility"], &["flexibilidade"]),
        spec!("Explain what a PDF is.", "Explique o que é um PDF.", &["document", "format"], &["documento", "formato"]),
        spec!("How many sides does a hexagon have?", "Quantos lados tem um hexágono?", &["6", "six"], &["6", "seis"]),
        spec!("Write a polite phrase to ask for clarification.", "Escreva uma frase educada para pedir esclarecimento.", &["could you"], &["poderia"]),
        spec!("List two ways to reduce plastic waste.", "Liste duas formas de reduzir o lixo plástico.", &["reuse", "recycle"], &["reutilizar", "reciclar"]),
        spec!("What does FM and AM stand for in radio?", "O que significam FM e AM no rádio?", &["frequency", "modulation"], &["frequência", "modulação"]),
        spec!("Give a short definition of machine learning.", "Dê uma definição curta de aprendizado de máquina.", &["data", "learn"], &["dados", "aprender"]),
        spec!("What is the difference between hardware and software?", "Qual é a diferença entre hardware e software?", &["physical", "programs"], &["físico", "programas"]),
        spec!("Suggest a title for a presentation about customer satisfaction.", "Sugira um título para uma apresentação sobre satisfação do cliente.", &["customer"], &["cliente"]),
        spec!("Write a concise confirmation message for an appointment.", "Escreva uma mensagem curta de confirmação para um compromisso.", &["confirmed", "time"], &["confirmado", "horário"]),
    ]
}
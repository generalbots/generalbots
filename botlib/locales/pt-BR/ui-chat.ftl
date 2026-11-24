# =============================================================================
# General Bots — pt-BR — Chat
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Chat
# -----------------------------------------------------------------------------
chat-title = Chat
chat-placeholder = Digite sua mensagem...
chat-send = Enviar
chat-new-conversation = Nova Conversa
chat-history = Histórico do Chat
chat-clear = Limpar Chat
chat-export = Exportar Chat
chat-typing = { $name } está digitando...
chat-online = Online
chat-offline = Offline
chat-mention-title = Referenciar Entidade
chat-mention-placeholder = Mensagem... (digite @ para mencionar)
chat-mention-search = Buscar entidades...
chat-mention-no-results = Nenhum resultado encontrado
chat-mention-type-hint = Digite : para buscar
chat-last-seen = Visto por último { $time }

# Traduções adicionais do Chat
chat-camera = Câmera
chat-camera-title = Tirar uma foto
chat-camera-capture = Tirar foto
chat-camera-cancel = Cancelar
chat-camera-sent = Foto enviada.
chat-camera-denied = A permissão da câmera foi negada. Autorize o acesso à câmera para o aplicativo de desktop nas configurações do sistema para tirar fotos.
chat-camera-missing = Nenhuma câmera foi encontrada neste dispositivo.
chat-camera-error = Não foi possível iniciar a câmera.
chat-camera-unsupported = Este aplicativo de desktop não oferece acesso à câmera.
voice-mode-title = Conversa por voz
voice-mode-listening = Ouvindo… fale naturalmente
voice-mode-thinking = Pensando…
voice-mode-speaking = Falando…
voice-mode-idle = Conversa por voz
voice-mode-end = Encerrar conversa por voz
chat-message-placeholder = Mensagem...

# Chat message controls and notices
chat-copy-message = Copiar mensagem
chat-copied = Copiado
chat-message-copied = Mensagem copiada
chat-copy-failed = Falha ao copiar
chat-listen-tts = Ouvir (TTS)
chat-listen-message = Ouvir esta mensagem
chat-image-unavailable = imagem indisponível
chat-open = Abrir
chat-expand = Expandir
chat-collapse = Recolher
chat-preview = Pré-visualizar
chat-close-preview = Fechar pré-visualização
chat-preview-load-failed = Não foi possível carregar a pré-visualização.
chat-show-more = Mostrar mais
chat-show-less = Mostrar menos
chat-tool = ferramenta
chat-param = parâmetro
chat-params = parâmetros
chat-tool-result = Resultado da ferramenta
chat-retry = Tentar novamente
chat-open-as-document = Abrir como documento
chat-show-more-history = mostrar mais…
chat-disconnected-reconnecting = Chat desconectado. Reconectando...
chat-connect-retrying = Não foi possível alcançar o servidor de chat. Tentando novamente...
chat-connection-lost = Conexão perdida. Tentando reconectar...
chat-attachment-sent = Anexo enviado: 
chat-attachments-sent = Anexos enviados: 
chat-message-not-delivered = Mensagem não entregue — reconectando
chat-attachment-read-failed = Não foi possível ler um anexo. Anexe novamente.
chat-attachment-queued = Conexão retornará em breve — anexo na fila
chat-reconnecting-queued = Reconectando — sua mensagem será enviada quando a conexão voltar.
chat-bot-private-signin = Este bot é privado — faça login para continuar
chat-approved-plan = Plano aprovado: 
chat-retrying-attachment = Tentando reenviar anexo
chat-expanded-preview = Visualização expandida

# #2362 — chat code/artifact control labels
chat-code-show-all = Show all
chat-code-lines = lines
chat-artifact-save-drive = Save to Drive
chat-artifact-download-svg = Download SVG
chat-artifact-download-png = Download PNG
chat-artifact-open-canvas = Open in Canvas
chat-artifact-folder = Folder (optional)
chat-artifact-kind-markdown = Markdown document
chat-artifact-kind-html = HTML page
chat-artifact-kind-code = Code
chat-artifact-words = words

# #2362 — attachment action label
chat-open-in-app = Open in app
# Assistant-look chat: transcript export and answer feedback
chat-export-actions = Ações da conversa
chat-export-md = Exportar conversa em Markdown
chat-export-print = Imprimir ou salvar em PDF
chat-export-done = Conversa exportada
chat-export-empty = Nada para exportar ainda
chat-export-untitled = Conversa
chat-export-user = Você
chat-export-assistant = Assistente
chat-feedback-label = Avaliação da resposta
chat-feedback-good = Boa resposta
chat-feedback-bad = Resposta ruim
chat-feedback-comment = O que poderia melhorar?
chat-feedback-send = Enviar
chat-staged-files = Arquivos anexados
chat-staged-close = Fechar pré-visualização
chat-camera-waiting = Waiting for the camera to start…
chat-camera-waiting-frames = Permission granted; waiting for video frames
chat-camera-ready = Camera ready. Take the photo or cancel.
chat-camera-started = Camera started — tap Take photo.
chat-camera-no-device = no video device
chat-camera-device = video device
chat-camera-no-frames = The camera preview did not start. Choose a photo from the system picker instead.
chat-camera-timeout = The camera did not answer. Choose a photo from the system picker instead.

# Chat engineer status strip (#2342)
chat-status-bot = Bot
chat-status-session = Sessão
chat-status-model = Modelo
chat-status-state = Estado
chat-status-no-session = —
chat-status-copy-session = Copiar ID da sessão
chat-status-copied = Copiado
chat-status-idle = Ocioso
chat-status-streaming = Transmitindo
chat-status-thinking = Pensando
chat-status-error = Erro
chat-status-stopped = Parado

chat-response-ready = Resposta pronta
chat-response-started = O assistente está respondendo
chat-tool-result-ready = Resultado da ferramenta disponível

# semantic i18n key (#2542)
chat-no-suggestions-available = Nenhuma sugestão disponível

# semantic i18n key (#2542)
chat-start-conversation = Iniciar uma conversa

# semantic i18n key (#2542)
chat-empty-hint = Digite uma mensagem abaixo ou use entrada de voz

# semantic i18n key (#2542)
chat-no-conversations-yet = Ainda não há conversas

# semantic i18n key (#2542)
chat-context-label = Contexto:

# semantic i18n key (#2542)
chat-select-context = Selecionar contexto...

# semantic i18n key (#2542)
chat-no-contexts-configured = Nenhum contexto configurado

# semantic i18n key (#2542)
projector-title = Visualizador de conteúdo

# semantic i18n key (#2542)
projector-loading-content = Carregando conteúdo...

# semantic i18n key (#2542)
projector-slide-info = Slide 1 de 1

# semantic i18n key (#2542)
projector-code-info = code.rs

# Projector viewer control (#2542)
chat-projector-rotate = Girar

# Chat header / app chrome (#2542)
chat-conversation-clear = Limpar contexto
chat-conversation-cleared = Contexto da conversa limpo
chat-conversation-copy-link = Copiar link
chat-conversation-delete = Excluir
chat-conversation-delete-confirm = Excluir esta conversa?
chat-conversation-delete-failed = Falha ao excluir a conversa
chat-conversation-delete-title = Excluir conversa
chat-conversation-delete-unavailable = Diálogo de confirmação indisponível
chat-conversation-deleted = Conversa excluída
chat-conversation-export-json = Exportar transcrição (.json)
chat-conversation-export-md = Exportar transcrição (.md)
chat-conversation-exported = Conversa exportada
chat-conversation-link-copied = Link da conversa copiado
chat-conversation-link-copy-failed = Falha ao copiar o link
chat-conversation-menu = Menu da conversa
chat-conversation-rename = Renomear
chat-conversation-rename-failed = Falha ao renomear a conversa
chat-conversation-renamed = Conversa renomeada
chat-conversation-title = Título da conversa
chat-conversation-untitled = Conversa sem título

# -----------------------------------------------------------------------------
# #2328 — loading / empty / error state copy
# -----------------------------------------------------------------------------
chat-empty-title = Comece a conversa
chat-empty-desc = Digite uma mensagem abaixo ou escolha uma sugestão quando o bot oferecer.
chat-empty-action = Escrever a primeira mensagem

# -----------------------------------------------------------------------------
# #2342 — chat engineer UX: slash commands, help, inspect/copy-markdown
# -----------------------------------------------------------------------------
chat-cmd-menu-title = Comandos
chat-cmd-help = Mostrar atalhos de teclado
chat-cmd-new = Iniciar uma nova conversa
chat-cmd-clear = Limpar esta conversa
chat-cmd-stop = Parar geração
chat-stop = Parar
chat-cmd-failed = Falha no comando
chat-help = Atalhos de teclado
chat-inspect = Inspecionar
chat-inspect-title = Detalhes da mensagem
chat-inspect-sender = Remetente
chat-inspect-id = ID da mensagem
chat-inspect-no-id = nenhum
chat-inspect-length = Caracteres
chat-copy-markdown = Copiar como Markdown
chat-copied-markdown = Copiado como Markdown

# Model attribution on each bot answer
chat-model-written-by = Resposta escrita por

# Assistant parity in chat
assistant-parity-block-allow = Permitir
assistant-parity-block-allowed = Permitido
assistant-parity-block-api-command = Comando de API
assistant-parity-block-attachment-read = Anexo lido
assistant-parity-block-chars = caracteres
assistant-parity-block-command = Comando
assistant-parity-block-decision-failed = Não foi possível registrar a decisão
assistant-parity-block-denied = Negado
assistant-parity-block-deny = Negar
assistant-parity-block-detail = Detalhe
assistant-parity-block-drive-list = Listagem do Drive
assistant-parity-block-drive-read = Arquivo do Drive lido
assistant-parity-block-drive-search = Busca no Drive
assistant-parity-block-elapsed = Decorrido
assistant-parity-block-error = Erro
assistant-parity-block-file = Arquivo
assistant-parity-block-kb-search = Busca no conhecimento
assistant-parity-block-notice = Aviso
assistant-parity-block-path = Caminho
assistant-parity-block-permission = Permissão solicitada
assistant-parity-block-query = Consulta
assistant-parity-block-read = Ler
assistant-parity-block-reason = Motivo
assistant-parity-block-result = Resultado
assistant-parity-block-retry = Tentar novamente
assistant-parity-block-seconds = s
assistant-parity-block-size = Tamanho
assistant-parity-block-status-done = Concluído
assistant-parity-block-status-failed = Falhou
assistant-parity-block-status-needs-you = Precisa de você
assistant-parity-block-status-running = Em execução
assistant-parity-block-step = Etapa
assistant-parity-block-thinking = Pensando
assistant-parity-block-tool = Ferramenta
assistant-parity-block-tool-run = Execução de ferramenta
assistant-parity-block-total = Total
assistant-parity-block-truncated = truncado
assistant-parity-block-type = Tipo
assistant-parity-block-yes = Sim
chat-parity-attach-title = Prévia do anexo
chat-parity-attach-unavailable = Conteúdo indisponível - use Download para abrir este arquivo.
chat-parity-attach-download-failed = Não foi possível baixar o anexo.
chat-parity-model-tiers = Auto e níveis
chat-parity-model-accounts = Contas de provedores
chat-parity-model-local = Local e gratuito
chat-drive-parity-attaching = Anexando...
chat-drive-parity-cancel = Cancelar
chat-drive-parity-delete = Excluir
chat-drive-parity-deleted = Excluído
chat-drive-parity-delete-failed = Não foi possível excluir o arquivo.
chat-drive-parity-delete-question = Excluir este arquivo?
chat-drive-parity-delete-title = Excluir arquivo
chat-drive-parity-deleting = Excluindo...
chat-drive-parity-download = Baixar
chat-drive-parity-download-failed = Não foi possível baixar o arquivo.
chat-drive-parity-downloading = Baixando...
chat-drive-parity-more = Mais ações
chat-drive-parity-new-name = Novo nome
chat-drive-parity-preview = Prévia
chat-drive-parity-preview-failed = Não foi possível visualizar o arquivo.
chat-drive-parity-previewing = Abrindo prévia...
chat-drive-parity-rename = Renomear
chat-drive-parity-renamed = Renomeado
chat-drive-parity-rename-failed = Não foi possível renomear o arquivo.
chat-drive-parity-rename-title = Renomear arquivo
chat-drive-parity-renaming = Renomeando...
chat-drive-parity-unavailable = Esta ação não está disponível nesta instalação.
chat-drive-parity-use-in-chat = Usar na conversa
chat-parity-agent-does = O agente faz
chat-parity-back = Voltar
chat-parity-checked-now = Verificado agora
chat-parity-close = Fechar
chat-parity-coming-soon = Em breve
chat-parity-cost-contract = Conforme seu contrato
chat-parity-cost-free = Grátis
chat-parity-cost-free-limit = Grátis, com limites de uso
chat-parity-cost-sub = Sua assinatura
chat-parity-cuda-no = Nenhuma GPU adequada detectada
chat-parity-cuda-yes = Placa de vídeo detectada
chat-parity-depends-agent = Depende do agente
chat-parity-driver = Driver
chat-parity-engine = Motor em uso
chat-parity-feature-docs = Documentos
chat-parity-feature-ocr = Texto em imagens
chat-parity-feature-photos = Fotos e gráficos
chat-parity-feature-web = Pesquisa na web
chat-parity-features = Recursos ligados
chat-parity-gains = Seus ganhos
chat-parity-go-agent = Vai para o provedor do agente
chat-parity-go-ext = Vai para um provedor externo
chat-parity-go-net = Fica na rede da sua organização
chat-parity-go-server = Vai para o servidor que você indicou
chat-parity-guide = Não sabe qual escolher?
chat-parity-guide-sub = Responda duas perguntas.
chat-parity-in-use = Em uso
chat-parity-load-failed = Não foi possível carregar estas informações.
chat-parity-loading = Carregando...
chat-parity-machine = Este computador
chat-parity-machine-ok = Este computador tem: %d, %v GB
chat-parity-machine-slow = Este computador não tem placa de vídeo adequada: funcionaria devagar
chat-parity-need-acp = Instale o agente e entre na sua conta
chat-parity-need-free = Nada
chat-parity-need-local = Uma placa de vídeo com 6 GB ou mais e um download de 4 a 6 GB
chat-parity-need-own = O endereço do servidor e uma chave
chat-parity-no-read = Não lê
chat-parity-no-work = Não funciona
chat-parity-none-found = Nenhum encontrado
chat-parity-not-yet = Ainda não
chat-parity-only-net = Só com servidor na sua rede
chat-parity-optional = Opcional, você liga
chat-parity-pick = Escolha uma opção
chat-parity-priv-ext = Suas conversas são enviadas para um host externo (%h).
chat-parity-priv-local = Suas conversas ficam neste computador.
chat-parity-priv-net = Suas conversas são enviadas para o servidor da sua organização (%h).
chat-parity-privacy = Privacidade
chat-parity-q1 = O que você escreve pode sair deste computador?
chat-parity-q1-help = Pense em informações confidenciais ou pessoais.
chat-parity-q1-no = Não, precisa ficar aqui
chat-parity-q1-no-desc = Apenas processamento local é aceitável.
chat-parity-q1-yes = Sim, pode sair
chat-parity-q1-yes-desc = Um serviço externo pode processar.
chat-parity-q2-no = Não, ou não sei
chat-parity-q2-no-desc = Sem servidor ou chave da empresa.
chat-parity-q2-open = Sua organização oferece um servidor ou chave de IA?
chat-parity-q2-open-help = Em caso de dúvida, consulte o TI.
chat-parity-q2-secret = Sua organização permite IA externa?
chat-parity-q2-secret-help = Algumas políticas proíbem serviços externos.
chat-parity-q2-secret-no-desc = IA externa não é permitida.
chat-parity-q2-secret-yes-desc = IA externa é permitida.
chat-parity-q2-yes = Sim
chat-parity-q2-yes-desc = Há servidor ou chave da empresa.
chat-parity-reads = Lê
chat-parity-ready = Pronto
chat-parity-rec-free = Comece pelo modelo gratuito: não precisa configurar nada. Para assuntos sigilosos, prefira neste computador.
chat-parity-rec-local = Use neste computador: nada sai daqui e esta máquina tem placa de vídeo.
chat-parity-rec-local-slow = Esta máquina não tem placa de vídeo adequada para modelo local. Use o servidor da sua organização na rede interna.
chat-parity-rec-local-tight = Um modelo local funciona, mas pode ficar lento com esta placa de vídeo.
chat-parity-rec-own = Use o servidor ou a chave da sua organização: o contrato define custo e limites.
chat-parity-rec-own-net = Use o servidor da sua organização na rede interna para manter os dados privados.
chat-parity-restart = Recomeçar
chat-parity-retry = Tentar novamente
chat-parity-row-cost = Custo
chat-parity-row-docs = Documentos
chat-parity-row-needs = Você precisa de
chat-parity-row-ocr = Texto em imagens
chat-parity-row-offline = Sem internet
chat-parity-row-photos = Fotos e gráficos
chat-parity-row-privacy = Seus textos
chat-parity-row-web = Pesquisa na web
chat-parity-rule = Regra
chat-parity-show-me = Ver como ativar
chat-parity-stay-here = Fica neste computador
chat-parity-step = Pergunta %n de 2
chat-parity-sub-acp = Sua assinatura de agente
chat-parity-sub-free = Pronto para usar
chat-parity-sub-local = Privado e sem internet
chat-parity-sub-own = Da sua organização
chat-parity-suggested = Sugerido
chat-parity-system-memory = Memória do sistema
chat-parity-verdict-bad-gpu = Nenhuma placa de vídeo adequada
chat-parity-verdict-bad-gpu-long = Nenhuma placa NVIDIA com CUDA foi encontrada. Modelos locais rodariam no processador, muito mais devagar: uma resposta pode levar minutos. Para manter seu texto na rede, prefira o servidor da sua organização.
chat-parity-verdict-capacity = Esta placa executa modelos que precisam de até %v de memória de vídeo.
chat-parity-verdict-good = Confortável
chat-parity-verdict-low-vram = Placa com memória insuficiente (%v)
chat-parity-verdict-low-vram-long = A placa tem %v de memória de vídeo; o menor modelo local precisa de %m.
chat-parity-verdict-mid = Apertado, mas utilizável
chat-parity-verdict-none = Não foi possível examinar este computador
chat-parity-video-memory = Memória de vídeo
chat-parity-where = Escolha o motor
chat-parity-where-acp = Agente (ACP)
chat-parity-where-free = Modelo gratuito
chat-parity-where-local = Neste computador
chat-parity-where-own = Servidor ou chave própria
chat-parity-where-sub = Compare as quatro formas de usar o assistente. Clique em uma para ver detalhes e ativar.
chat-parity-works = Funciona
chat-parity-workshop = Oficina do assistente
chat-parity-workshop-lead = Ajuste onde e como seu assistente pensa.
chat-parity-workshop-title = Oficina do assistente
chat-parity-graphics-card = Placa de vídeo

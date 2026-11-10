# General Bots — shell do desktop e assistente de configuração (#2089)

desktop-document-title = General Bots Desktop
sidebar-toggle = Alternar barra lateral
sidebar-collapsed-chat = Chat
sidebar-collapsed-terminal = Terminal
sidebar-collapsed-drive = Arquivos
sidebar-collapsed-settings = Configurações
sidebar-collapsed-account = Conta
setup-welcome-title = Bem-vindo ao General Bots
setup-welcome-subtitle = Vamos configurar tudo em alguns passos rápidos
setup-welcome-quick-subtitle = Dois passos rápidos para começar
setup-step-llm = Provedor de LLM
setup-step-profile = Perfil de Usuário
setup-step-bot = Configuração do Bot
setup-step-training = Treinamento
setup-step-ai-model = Modelo de IA
setup-step-your-name = Seu Nome
setup-choose-provider-title = Escolha seu Provedor de LLM
setup-choose-provider-subtitle = Selecione o provedor de modelo de IA para seu bot
setup-provider-openai-desc = Modelos GPT-4, GPT-3.5
setup-provider-nvidia-desc = Inferência NVIDIA NIM
setup-provider-anthropic-desc = Modelos Claude 3, Claude 3.5
setup-provider-local-desc = llama.cpp, Ollama, etc.
setup-profile-title = O que melhor descreve você?
setup-profile-subtitle = Selecione seu tipo de perfil para padrões personalizados
setup-profile-developer = Desenvolvedor
setup-profile-developer-desc = Crie e implante bots, escreva scripts BASIC
setup-profile-business = Negócios
setup-profile-business-desc = Automatize fluxos de trabalho, gerencie CRM
setup-profile-support = Suporte
setup-profile-support-desc = Atendimento ao cliente, gestão de chamados
setup-bot-title = Configure seu Bot
setup-bot-subtitle = Dê ao seu bot um nome, um propósito e um modelo opcional
setup-bot-name = Nome do Bot
setup-bot-name-placeholder = Meu Assistente
setup-bot-purpose = Propósito do Bot
setup-bot-purpose-placeholder = Descreva o que seu bot deve fazer...
setup-bot-template = Modelo de Bot
setup-optional = (opcional)
setup-template-scratch = — Começar do zero —
setup-name-title = Como devemos chamar você?
setup-name-subtitle = Este nome aparece nas suas conversas
setup-name-label = Seu Nome
setup-name-placeholder = ex.: Maria
setup-training-title = Treine seu Bot
setup-training-subtitle = Arraste arquivos .pdf, .xls, .doc para o treinamento inicial. Você terá acesso ao Drive e poderá mudar depois.
setup-drop-text = Solte arquivos aqui ou clique para selecionar
setup-drop-hint = .pdf, .xls, .doc — até 10MB cada
setup-select-files = Selecionar Arquivos
setup-file-status-pending = Pendente
setup-file-status-uploading = Enviando
setup-file-status-done = Concluído
setup-file-status-error = Erro
setup-saving = Salvando...
setup-complete-message = Configuração concluída! Bem-vindo ao General Bots.
setup-failed = Falha na configuração: {error}
setup-unknown-error = Erro desconhecido
setup-back = Voltar
setup-next = Avançar
setup-finish = Concluir
setup-skip = Pular Configuração
agent-all-apps = Todos os apps

# --- Painéis do shell do desktop: versões, acessibilidade, barra lateral,
# --- colaboração, atualizações, modais e uso (suite/js/*.js) — #2095

versions-component = Componente
versions-version = Versão
versions-status = Status
versions-action = Ação
versions-up-to-date = Atualizado
versions-unknown = Desconhecido
versions-check-update = Verificar atualização
versions-checking = Verificando...
versions-checked = Verificado
versions-error = Erro
versions-update-available = Atualizar: { $version }
versions-could-not-load = Não foi possível carregar as informações de versão.
versions-ensure-backend = Verifique se o servidor backend está em execução.
versions-retry = Tentar novamente

a11y-chat-conversation = Conversa do chat
a11y-chat-messages = Mensagens do chat
a11y-send-message = Enviar mensagem
a11y-type-message = Digite sua mensagem
a11y-keyboard-shortcuts = Atalhos de teclado
a11y-shortcut-focus-chat = Focar o campo do chat
a11y-shortcut-show-help = Mostrar esta ajuda
a11y-shortcut-escape = Fechar modal / limpar seleção
a11y-shortcut-command-palette = Paleta de comandos
a11y-shortcut-privacy = Configurações de privacidade
a11y-shortcut-scroll-history = Rolar o histórico do chat
a11y-shortcut-navigate = Navegar entre elementos
a11y-shortcut-activate = Ativar o elemento em foco
a11y-shortcut-delete-files = Excluir arquivos selecionados (Drive)
a11y-shortcut-select-all = Selecionar todos os arquivos (Drive)
a11y-shortcut-clipboard = Copiar / Recortar / Colar arquivos
a11y-shortcut-navigate-up = Subir um nível no Drive
a11y-shortcut-rename = Renomear o arquivo selecionado

sidebar-locked-conversations = Conversas bloqueadas
sidebar-no-conversations = Nenhuma conversa ainda
sidebar-sign-in-conversations = Entre para ver suas conversas
sidebar-history-unavailable = Histórico indisponível
sidebar-back-to-conversations = ← Voltar para as conversas
sidebar-no-locked-conversations = Nenhuma conversa bloqueada
sidebar-locked-conversation = Conversa bloqueada
# #2095 — conversation-history time buckets.
sidebar-bucket-today = Hoje
sidebar-bucket-yesterday = Ontem
sidebar-bucket-previous-7-days = Últimos 7 dias
sidebar-bucket-older = Mais antigas

sidebar-sign-in = Entrar
sidebar-add-pin = Adicionar fixado

collab-collaborators = Colaboradores ({ $count })
collab-no-one-else = Ninguém mais está aqui

update-available-title = Atualização disponível — v{ $version }
update-now = Atualizar agora
update-later = Depois
update-skip-version = Ignorar esta versão
update-platform = Plataforma: { $name }
update-asset = Arquivo: { $name }
update-check-failed = Falha ao verificar atualização: { $error }
update-view-release-notes = Ver notas da versão
update-release-notes-title = Notas da versão — v{ $version }
update-no-release-notes = Nenhuma nota de versão disponível.

modal-alert-title = Aviso
modal-input-title = Entrada

external-save-to = Salvar em { $target }

usage-title = Uso e custo
usage-last-days = Últimos { $days } dias
usage-loading = Carregando uso…
usage-per-model = Por modelo
usage-per-day = Por dia
usage-model = Modelo
usage-tokens = Tokens
usage-messages = Mensagens
usage-cost = Custo
usage-day = Dia
usage-days-7 = 7 dias
usage-days-30 = 30 dias
usage-days-90 = 90 dias

pin-close = Fechar
pin-length-hint = 4–8 dígitos
pin-cancel = Cancelar
pin-ok = OK

# --- Superfícies de colaboração compartilhadas por todos os apps do suite:
# --- diálogo de compartilhamento, trilha de atividades, banner de conflito e
# --- histórico de versões (suite/js/collab-*.js) — #2095

role-owner = proprietário
role-editor = Editor
role-commenter = Comentarista
role-viewer = Visualizador

share-request-failed = Falha na requisição ({ $status })
share-warning-title = Links compartilhados ainda não estão acessíveis
share-warning-body = Esta instalação só é acessível em localhost, então um link aqui não abre em lugar nenhum. Configure uma destas opções primeiro: ative o P2P em Configurações → Rede, ou aponte Links Compartilhados para o seu próprio domínio.
share-open-shared-links = Abrir Links Compartilhados
share-open-network = Abrir Rede
share-invite-people = Convidar pessoas
share-anyone-with-link = Qualquer pessoa com o link
share-create-link = Criar link
share-transfer-ownership = Transferir propriedade
share-transfer = Transferir
share-new-owner-placeholder = e-mail do novo proprietário
share-owner-suffix = · proprietário
share-no-people = Nenhuma pessoa adicionada ainda.
share-no-links = Nenhum link público ainda.
share-copied = Copiado
share-revoke = Revogar
share-enter-email = Informe um e-mail
share-enter-new-owner = Informe o e-mail do novo proprietário
share-needs-base = Configure o P2P ou o seu próprio domínio antes de criar um link compartilhado.

activity-title = Atividades
activity-request-failed = Falha na requisição ({ $status })
activity-load-more = Carregar mais
activity-loading = Carregando…
activity-empty = Nenhuma atividade ainda
activity-action-create = criou
activity-action-edit = editou
activity-action-comment = comentou em
activity-action-delete = excluiu
activity-action-resolve = resolveu um comentário em
activity-action-reopen = reabriu um comentário em
activity-action-reaction = reagiu a
activity-action-share = compartilhou
activity-action-restore = restaurou
activity-action-transfer = transferiu

conflict-title = Edições conflitantes
conflict-dismiss = Dispensar
conflict-edited-concurrently = editou simultaneamente ({ $op })
conflict-keep-server = Manter versão do servidor
conflict-keep-mine = Manter minha alteração
conflict-not-started = Monitoramento de conflitos não iniciado
conflict-request-failed = Falha na requisição ({ $status })

versions-title = Histórico de versões
versions-loading = Carregando…
versions-empty = Nenhuma versão ainda — o primeiro salvamento cria uma.
versions-version-n = Versão { $n }
versions-current-tag = (atual)
versions-name = Nomear…
versions-restore-failed = Falha ao restaurar: { $error }
versions-rename-failed = Falha ao renomear: { $error }
versions-restore-dialog-title = Restaurar versão
versions-restore-confirm = Restaurar esta versão? Uma nova versão atual será criada — nada é perdido.
versions-name-dialog-title = Nomear versão
versions-name-prompt = Nomeie esta versão (ex.: "v2 — aprovada"):
versions-request-failed = Falha na requisição ({ $status })

# --- Shell do desktop: seletor de encaixe, abas de workspace, desktops
# --- virtuais, menu de contexto, painel git, painel de navegador, widgets e
# --- exportação de dados (suite/js/*.js)

snap-title = Encaixar "{ $app }"
snap-layout-top-left = Canto superior esquerdo
snap-layout-top = Metade superior
snap-layout-top-right = Canto superior direito
snap-layout-left = Metade esquerda
snap-layout-full = Maximizar
snap-layout-right = Metade direita
snap-layout-bottom-left = Canto inferior esquerdo
snap-layout-bottom = Metade inferior
snap-layout-bottom-right = Canto inferior direito
snap-layout-third-left = Terço esquerdo
snap-layout-two-thirds = Dois terços
snap-layout-third-right = Terço direito

wstabs-close-tab = Fechar aba
wstabs-new-tab = Nova aba de workspace
wstabs-default-name = Workspace { $n }
wstabs-default-main = Principal
wstabs-rename-title = Renomear workspace
wstabs-name-prompt = Nome do workspace:

vdesk-default-name = Desktop { $n }
vdesk-title = Desktops Virtuais
vdesk-new = Novo desktop
vdesk-rename-title = Renomear desktop
vdesk-name-prompt = Nome do desktop:
vdesk-restore-title = Restaurar sessão
vdesk-restore-body = Reabrir as { $count } janela(s) da sua última sessão neste desktop?
vdesk-restore-body-short = Restaurar sua última sessão ({ $count } janela(s))?

ctxmenu-search = Pesquisar no DuckDuckGo
ctxmenu-search-prompt = Pesquisar no DuckDuckGo:
ctxmenu-wikipedia = Definição na Wikipédia
ctxmenu-wikipedia-prompt = Definição na Wikipédia:

git-loading-status = Carregando status...
git-no-changes = Sem alterações
git-new-branch-prompt = Nome da nova branch:
git-loading-diff = Carregando diff...
git-diff-failed = Falha ao carregar o diff
git-commit-message-required = Informe uma mensagem de commit
git-pushing = Enviando...
git-committed = Commit e push realizados com sucesso
git-failed = Falha na operação do Git

browser-session-active = Sessão de navegador { $id } ativa.
browser-init-failed = Falha ao inicializar o navegador
browser-init-first = Inicialize o navegador primeiro
browser-capture = Captura
browser-no-session = Nenhuma sessão ativa
browser-export-failed = Falha na exportação
browser-no-actions = Nenhuma ação registrada

widget-load-failed = Falha ao carregar o widget
widget-no-source = Nenhuma fonte de widget configurada
widget-open-windowed = Abrir em janela
widget-pin-launcher = Fixar no launcher

dataexport-title = Exportação e Backup de Dados
dataexport-desc = Exporte todos os seus dados, incluindo histórico de chat, memória, metadados do drive e configurações, como um arquivo JSON de backup.
dataexport-export-all = Exportar Todos os Dados
dataexport-import = Importar Backup
dataexport-exported = Backup exportado com sucesso
dataexport-too-large = Backup rejeitado: o arquivo tem mais de 4 MB
dataexport-invalid = Arquivo de backup inválido: { $error }
dataexport-nothing = Este backup não contém configurações restauráveis
dataexport-unknown-date = data desconhecida
dataexport-restore-confirm = Isso restaurará { $count } configuração(ões) do backup de { $date }. Credenciais e tokens de sessão nunca são importados. Continuar?
dataexport-restored = Backup restaurado ({ $count } itens).
dataexport-rejected = { $count } entrada(s) rejeitada(s) pela lista de permissões.
# #2095 — backup-import validation details shown inside
# `dataexport-invalid`.
dataexport-error-not-object = o arquivo não é um backup válido
dataexport-error-version = versão de backup não suportada
dataexport-error-date = o backup não tem uma data de exportação válida
dataexport-error-section = seção inesperada: { $section }
dataexport-reloading = Recarregando...

comments-title = Comentários
comments-request-failed = Falha na requisição ({ $status })
comments-loading = Carregando comentários…
comments-load-failed = Falha ao carregar comentários: { $error }
comments-empty-open = Nenhum comentário aberto. Use @nome para mencionar alguém.
comments-empty-resolved = Nenhum comentário resolvido. Use @nome para mencionar alguém.
comments-resolved-chip = Resolvido
comments-resolve = Resolver
comments-reopen = Reabrir
comments-resolved-announce = Comentário resolvido
comments-reopened-announce = Comentário reaberto
comments-resolve-failed = Falha ao resolver: { $error }
comments-added = Comentário adicionado
comments-post = Publicar
comments-post-failed = Falha ao comentar: { $error }
comments-reaction-failed = Falha na reação: { $error }
comments-delete-failed = Falha ao excluir: { $error }
comments-reply-failed = Falha ao responder: { $error }
comments-delete-title = Excluir comentário
comments-delete-confirm = Excluir este comentário?
comments-reply-title = Responder
comments-reply-prompt = Responder ao comentário:
comments-filter-open = Abertos
comments-filter-all = Todos
comments-filter-resolved = Resolvidos
comments-input-placeholder = Adicione um comentário… use @nome para mencionar
comments-someone = alguém
comments-mention-announce = Você foi mencionado em um comentário por { $author }

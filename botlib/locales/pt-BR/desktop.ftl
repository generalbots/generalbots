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

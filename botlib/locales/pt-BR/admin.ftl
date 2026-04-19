# =============================================================================
# General Bots - Traduções de Administração (Português Brasileiro)
# =============================================================================
# Traduções da interface administrativa para o Painel Admin do GB
# =============================================================================

# -----------------------------------------------------------------------------
# Navegação Admin & Painel
# -----------------------------------------------------------------------------
admin-title = Administração
admin-dashboard = Painel Administrativo
admin-overview = Visão Geral
admin-welcome = Bem-vindo ao Painel Administrativo

admin-nav-dashboard = Painel
admin-nav-users = Usuários
admin-nav-bots = Bots
admin-nav-tenants = Inquilinos
admin-nav-settings = Configurações
admin-nav-logs = Logs
admin-nav-analytics = Análises
admin-nav-security = Segurança
admin-nav-integrations = Integrações
admin-nav-billing = Faturamento
admin-nav-support = Suporte
admin-nav-groups = Grupos
admin-nav-dns = DNS
admin-nav-system = Sistema

# -----------------------------------------------------------------------------
# Ações Rápidas Admin
# -----------------------------------------------------------------------------
admin-quick-actions = Ações Rápidas
admin-create-user = Criar Usuário
admin-create-group = Criar Grupo
admin-register-dns = Registrar DNS
admin-recent-activity = Atividade Recente
admin-system-health = Saúde do Sistema

# -----------------------------------------------------------------------------
# Gerenciamento de Usuários
# -----------------------------------------------------------------------------
admin-users-title = Gerenciamento de Usuários
admin-users-list = Lista de Usuários
admin-users-add = Adicionar Usuário
admin-users-edit = Editar Usuário
admin-users-delete = Excluir Usuário
admin-users-search = Buscar usuários...
admin-users-filter = Filtrar Usuários
admin-users-export = Exportar Usuários
admin-users-import = Importar Usuários
admin-users-total = Total de Usuários
admin-users-active = Usuários Ativos
admin-users-inactive = Usuários Inativos
admin-users-suspended = Usuários Suspensos
admin-users-pending = Verificação Pendente
admin-users-last-login = Último Login
admin-users-created = Criado em
admin-users-role = Função
admin-users-status = Status
admin-users-actions = Ações
admin-users-no-users = Nenhum usuário encontrado
admin-users-confirm-delete = Tem certeza que deseja excluir este usuário?
admin-users-deleted = Usuário excluído com sucesso
admin-users-saved = Usuário salvo com sucesso
admin-users-invite = Convidar Usuário
admin-users-invite-sent = Convite enviado com sucesso
admin-users-bulk-actions = Ações em Massa
admin-users-select-all = Selecionar Todos
admin-users-deselect-all = Desmarcar Todos

# Detalhes do Usuário
admin-user-details = Detalhes do Usuário
admin-user-profile = Perfil
admin-user-email = E-mail
admin-user-name = Nome
admin-user-phone = Telefone
admin-user-avatar = Avatar
admin-user-timezone = Fuso Horário
admin-user-language = Idioma
admin-user-role-admin = Administrador
admin-user-role-manager = Gerente
admin-user-role-user = Usuário
admin-user-role-viewer = Visualizador
admin-user-status-active = Ativo
admin-user-status-inactive = Inativo
admin-user-status-suspended = Suspenso
admin-user-status-pending = Pendente
admin-user-permissions = Permissões
admin-user-activity = Log de Atividades
admin-user-sessions = Sessões Ativas
admin-user-terminate-session = Encerrar Sessão
admin-user-terminate-all = Encerrar Todas as Sessões
admin-user-reset-password = Redefinir Senha
admin-user-force-logout = Forçar Logout
admin-user-enable-2fa = Ativar 2FA
admin-user-disable-2fa = Desativar 2FA

# -----------------------------------------------------------------------------
# Gerenciamento de Grupos
# -----------------------------------------------------------------------------
admin-groups-title = Gerenciamento de Grupos
admin-groups-subtitle = Gerencie grupos, membros e permissões
admin-groups-list = Lista de Grupos
admin-groups-add = Adicionar Grupo
admin-groups-create = Criar Grupo
admin-groups-edit = Editar Grupo
admin-groups-delete = Excluir Grupo
admin-groups-search = Buscar grupos...
admin-groups-filter = Filtrar Grupos
admin-groups-total = Total de Grupos
admin-groups-active = Grupos Ativos
admin-groups-no-groups = Nenhum grupo encontrado
admin-groups-confirm-delete = Tem certeza que deseja excluir este grupo?
admin-groups-deleted = Grupo excluído com sucesso
admin-groups-saved = Grupo salvo com sucesso
admin-groups-created = Grupo criado com sucesso
admin-groups-loading = Carregando grupos...

# Detalhes do Grupo
admin-group-details = Detalhes do Grupo
admin-group-name = Nome do Grupo
admin-group-description = Descrição
admin-group-visibility = Visibilidade
admin-group-visibility-public = Público
admin-group-visibility-private = Privado
admin-group-visibility-hidden = Oculto
admin-group-join-policy = Política de Entrada
admin-group-join-invite = Apenas por Convite
admin-group-join-request = Solicitar Entrada
admin-group-join-open = Aberto
admin-group-members = Membros
admin-group-member-count = { $count ->
    [one] { $count } membro
   *[other] { $count } membros
}
admin-group-add-member = Adicionar Membro
admin-group-remove-member = Remover Membro
admin-group-permissions = Permissões
admin-group-settings = Configurações
admin-group-analytics = Análises
admin-group-overview = Visão Geral

# Modos de Visualização de Grupos
admin-groups-view-grid = Visualização em Grade
admin-groups-view-list = Visualização em Lista
admin-groups-all-visibility = Todas as Visibilidades

# -----------------------------------------------------------------------------
# Gerenciamento de DNS
# -----------------------------------------------------------------------------
admin-dns-title = Gerenciamento de DNS
admin-dns-subtitle = Registre e gerencie hostnames DNS para seus bots
admin-dns-register = Registrar Hostname
admin-dns-registered = Hostnames Registrados
admin-dns-search = Buscar hostnames...
admin-dns-refresh = Atualizar
admin-dns-loading = Carregando registros DNS...
admin-dns-no-records = Nenhum registro DNS encontrado
admin-dns-confirm-delete = Tem certeza que deseja remover este hostname?
admin-dns-deleted = Hostname removido com sucesso
admin-dns-saved = Registro DNS salvo com sucesso
admin-dns-created = Hostname registrado com sucesso

# Campos do Formulário DNS
admin-dns-hostname = Hostname
admin-dns-hostname-placeholder = meubot.exemplo.com
admin-dns-hostname-help = Digite o nome de domínio completo que deseja registrar
admin-dns-record-type = Tipo de Registro
admin-dns-record-type-a = A (IPv4)
admin-dns-record-type-aaaa = AAAA (IPv6)
admin-dns-record-type-cname = CNAME
admin-dns-ttl = TTL (segundos)
admin-dns-ttl-5min = 5 minutos (300)
admin-dns-ttl-1hour = 1 hora (3600)
admin-dns-ttl-1day = 1 dia (86400)
admin-dns-target = Destino/Endereço IP
admin-dns-target-placeholder-ipv4 = 192.168.1.1
admin-dns-target-placeholder-ipv6 = 2001:db8::1
admin-dns-target-placeholder-cname = destino.exemplo.com
admin-dns-target-help-a = Digite o endereço IPv4 de destino
admin-dns-target-help-aaaa = Digite o endereço IPv6 de destino
admin-dns-target-help-cname = Digite o nome de domínio de destino
admin-dns-auto-ssl = Provisionar certificado SSL automaticamente

# Cabeçalhos da Tabela DNS
admin-dns-col-hostname = Hostname
admin-dns-col-type = Tipo
admin-dns-col-target = Destino
admin-dns-col-ttl = TTL
admin-dns-col-ssl = SSL
admin-dns-col-status = Status
admin-dns-col-actions = Ações

# Status DNS
admin-dns-status-active = Ativo
admin-dns-status-pending = Pendente
admin-dns-status-error = Erro
admin-dns-ssl-enabled = SSL Ativado
admin-dns-ssl-disabled = Sem SSL
admin-dns-ssl-pending = SSL Pendente

# Cards de Ajuda DNS
admin-dns-help-title = Ajuda de Configuração DNS
admin-dns-help-a-record = Registro A
admin-dns-help-a-record-desc = Mapeia um nome de domínio para um endereço IPv4. Use para apontar seu hostname diretamente para um IP de servidor.
admin-dns-help-aaaa-record = Registro AAAA
admin-dns-help-aaaa-record-desc = Mapeia um nome de domínio para um endereço IPv6. Similar ao registro A, mas para conectividade IPv6.
admin-dns-help-cname-record = Registro CNAME
admin-dns-help-cname-record-desc = Cria um alias de um domínio para outro. Útil para apontar subdomínios para seu domínio principal.
admin-dns-help-ssl = SSL/TLS
admin-dns-help-ssl-desc = Provisiona automaticamente certificados Let's Encrypt para conexões HTTPS seguras.

# Modais de Edição/Remoção DNS
admin-dns-edit-title = Editar Registro DNS
admin-dns-remove-title = Remover Hostname
admin-dns-remove-warning = Isso excluirá o registro DNS e quaisquer certificados SSL associados. O hostname não será mais resolvido.

# -----------------------------------------------------------------------------
# Gerenciamento de Bots
# -----------------------------------------------------------------------------
admin-bots-title = Gerenciamento de Bots
admin-bots-list = Lista de Bots
admin-bots-add = Adicionar Bot
admin-bots-edit = Editar Bot
admin-bots-delete = Excluir Bot
admin-bots-search = Buscar bots...
admin-bots-filter = Filtrar Bots
admin-bots-total = Total de Bots
admin-bots-active = Bots Ativos
admin-bots-inactive = Bots Inativos
admin-bots-draft = Bots em Rascunho
admin-bots-published = Bots Publicados
admin-bots-no-bots = Nenhum bot encontrado
admin-bots-confirm-delete = Tem certeza que deseja excluir este bot?
admin-bots-deleted = Bot excluído com sucesso
admin-bots-saved = Bot salvo com sucesso
admin-bots-duplicate = Duplicar Bot
admin-bots-export = Exportar Bot
admin-bots-import = Importar Bot
admin-bots-publish = Publicar
admin-bots-unpublish = Despublicar
admin-bots-test = Testar Bot
admin-bots-logs = Logs do Bot
admin-bots-analytics = Análises do Bot
admin-bots-conversations = Conversas
admin-bots-templates = Templates
admin-bots-dialogs = Diálogos
admin-bots-knowledge-base = Base de Conhecimento

# Detalhes do Bot
admin-bot-details = Detalhes do Bot
admin-bot-name = Nome do Bot
admin-bot-description = Descrição
admin-bot-avatar = Avatar do Bot
admin-bot-language = Idioma
admin-bot-timezone = Fuso Horário
admin-bot-greeting = Mensagem de Saudação
admin-bot-fallback = Mensagem de Fallback
admin-bot-channels = Canais
admin-bot-channel-web = Chat Web
admin-bot-channel-whatsapp = WhatsApp
admin-bot-channel-telegram = Telegram
admin-bot-channel-slack = Slack
admin-bot-channel-teams = Microsoft Teams
admin-bot-channel-email = E-mail
admin-bot-model = Modelo de IA
admin-bot-temperature = Temperatura
admin-bot-max-tokens = Máximo de Tokens
admin-bot-system-prompt = Prompt do Sistema

# -----------------------------------------------------------------------------
# Gerenciamento de Inquilinos
# -----------------------------------------------------------------------------
admin-tenants-title = Gerenciamento de Inquilinos
admin-tenants-list = Lista de Inquilinos
admin-tenants-add = Adicionar Inquilino
admin-tenants-edit = Editar Inquilino
admin-tenants-delete = Excluir Inquilino
admin-tenants-search = Buscar inquilinos...
admin-tenants-total = Total de Inquilinos
admin-tenants-active = Inquilinos Ativos
admin-tenants-suspended = Inquilinos Suspensos
admin-tenants-trial = Inquilinos em Teste
admin-tenants-no-tenants = Nenhum inquilino encontrado
admin-tenants-confirm-delete = Tem certeza que deseja excluir este inquilino?
admin-tenants-deleted = Inquilino excluído com sucesso
admin-tenants-saved = Inquilino salvo com sucesso

# Detalhes do Inquilino
admin-tenant-details = Detalhes do Inquilino
admin-tenant-name = Nome do Inquilino
admin-tenant-domain = Domínio
admin-tenant-plan = Plano
admin-tenant-plan-free = Gratuito
admin-tenant-plan-starter = Inicial
admin-tenant-plan-professional = Profissional
admin-tenant-plan-enterprise = Empresarial
admin-tenant-users = Usuários
admin-tenant-bots = Bots
admin-tenant-storage = Armazenamento Usado
admin-tenant-api-calls = Chamadas de API
admin-tenant-limits = Limites de Uso
admin-tenant-billing = Informações de Faturamento

# -----------------------------------------------------------------------------
# Configurações do Sistema
# -----------------------------------------------------------------------------
admin-settings-title = Configurações do Sistema
admin-settings-general = Configurações Gerais
admin-settings-security = Configurações de Segurança
admin-settings-email = Configurações de E-mail
admin-settings-storage = Configurações de Armazenamento
admin-settings-integrations = Integrações
admin-settings-api = Configurações de API
admin-settings-appearance = Aparência
admin-settings-localization = Localização
admin-settings-notifications = Notificações
admin-settings-backup = Backup e Restauração
admin-settings-maintenance = Modo de Manutenção
admin-settings-saved = Configurações salvas com sucesso
admin-settings-reset = Restaurar Padrões
admin-settings-confirm-reset = Tem certeza que deseja restaurar todas as configurações para os padrões?

# Configurações Gerais
admin-settings-site-name = Nome do Site
admin-settings-site-url = URL do Site
admin-settings-admin-email = E-mail do Admin
admin-settings-support-email = E-mail de Suporte
admin-settings-default-language = Idioma Padrão
admin-settings-default-timezone = Fuso Horário Padrão
admin-settings-date-format = Formato de Data
admin-settings-time-format = Formato de Hora
admin-settings-currency = Moeda

# Configurações de E-mail
admin-settings-smtp-host = Host SMTP
admin-settings-smtp-port = Porta SMTP
admin-settings-smtp-user = Usuário SMTP
admin-settings-smtp-password = Senha SMTP
admin-settings-smtp-encryption = Criptografia
admin-settings-smtp-from-name = Nome do Remetente
admin-settings-smtp-from-email = E-mail do Remetente
admin-settings-smtp-test = Enviar E-mail de Teste
admin-settings-smtp-test-success = E-mail de teste enviado com sucesso
admin-settings-smtp-test-failed = Falha ao enviar e-mail de teste

# Configurações de Armazenamento
admin-settings-storage-provider = Provedor de Armazenamento
admin-settings-storage-local = Armazenamento Local
admin-settings-storage-s3 = Amazon S3
admin-settings-storage-minio = MinIO
admin-settings-storage-gcs = Google Cloud Storage
admin-settings-storage-azure = Azure Blob Storage
admin-settings-storage-bucket = Nome do Bucket
admin-settings-storage-region = Região
admin-settings-storage-access-key = Chave de Acesso
admin-settings-storage-secret-key = Chave Secreta
admin-settings-storage-endpoint = URL do Endpoint

# -----------------------------------------------------------------------------
# Logs do Sistema
# -----------------------------------------------------------------------------
admin-logs-title = Logs do Sistema
admin-logs-search = Buscar logs...
admin-logs-filter-level = Filtrar por Nível
admin-logs-filter-source = Filtrar por Origem
admin-logs-filter-date = Filtrar por Data
admin-logs-level-all = Todos os Níveis
admin-logs-level-debug = Debug
admin-logs-level-info = Info
admin-logs-level-warning = Aviso
admin-logs-level-error = Erro
admin-logs-level-critical = Crítico
admin-logs-export = Exportar Logs
admin-logs-clear = Limpar Logs
admin-logs-confirm-clear = Tem certeza que deseja limpar todos os logs?
admin-logs-cleared = Logs limpos com sucesso
admin-logs-no-logs = Nenhum log encontrado
admin-logs-refresh = Atualizar
admin-logs-auto-refresh = Atualização Automática
admin-logs-timestamp = Data/Hora
admin-logs-level = Nível
admin-logs-source = Origem
admin-logs-message = Mensagem
admin-logs-details = Detalhes

# -----------------------------------------------------------------------------
# Análises
# -----------------------------------------------------------------------------
admin-analytics-title = Análises
admin-analytics-overview = Visão Geral
admin-analytics-users = Análises de Usuários
admin-analytics-bots = Análises de Bots
admin-analytics-conversations = Análises de Conversas
admin-analytics-performance = Desempenho
admin-analytics-period = Período
admin-analytics-period-today = Hoje
admin-analytics-period-week = Esta Semana
admin-analytics-period-month = Este Mês
admin-analytics-period-quarter = Este Trimestre
admin-analytics-period-year = Este Ano
admin-analytics-period-custom = Período Personalizado
admin-analytics-export = Exportar Relatório
admin-analytics-total-users = Total de Usuários
admin-analytics-new-users = Novos Usuários
admin-analytics-active-users = Usuários Ativos
admin-analytics-total-bots = Total de Bots
admin-analytics-active-bots = Bots Ativos
admin-analytics-total-conversations = Total de Conversas
admin-analytics-avg-response-time = Tempo Médio de Resposta
admin-analytics-satisfaction-rate = Taxa de Satisfação
admin-analytics-resolution-rate = Taxa de Resolução

# -----------------------------------------------------------------------------
# Segurança
# -----------------------------------------------------------------------------
admin-security-title = Segurança
admin-security-overview = Visão Geral de Segurança
admin-security-audit-log = Log de Auditoria
admin-security-login-attempts = Tentativas de Login
admin-security-blocked-ips = IPs Bloqueados
admin-security-api-keys = Chaves de API
admin-security-webhooks = Webhooks
admin-security-cors = Configurações CORS
admin-security-rate-limiting = Limitação de Taxa
admin-security-encryption = Criptografia
admin-security-2fa = Autenticação de Dois Fatores
admin-security-sso = Login Único (SSO)
admin-security-password-policy = Política de Senhas

# Chaves de API
admin-api-keys-title = Chaves de API
admin-api-keys-add = Criar Chave de API
admin-api-keys-name = Nome da Chave
admin-api-keys-key = Chave de API
admin-api-keys-secret = Chave Secreta
admin-api-keys-created = Criada em
admin-api-keys-last-used = Último Uso
admin-api-keys-expires = Expira em
admin-api-keys-never = Nunca
admin-api-keys-revoke = Revogar
admin-api-keys-confirm-revoke = Tem certeza que deseja revogar esta chave de API?
admin-api-keys-revoked = Chave de API revogada com sucesso
admin-api-keys-created-success = Chave de API criada com sucesso
admin-api-keys-copy = Copiar para Área de Transferência
admin-api-keys-copied = Copiado!
admin-api-keys-warning = Certifique-se de copiar sua chave de API agora. Você não poderá vê-la novamente!

# -----------------------------------------------------------------------------
# Faturamento
# -----------------------------------------------------------------------------
admin-billing-title = Faturamento
admin-billing-overview = Visão Geral do Faturamento
admin-billing-current-plan = Plano Atual
admin-billing-usage = Uso
admin-billing-invoices = Faturas
admin-billing-payment-methods = Métodos de Pagamento
admin-billing-upgrade = Fazer Upgrade do Plano
admin-billing-downgrade = Fazer Downgrade do Plano
admin-billing-cancel = Cancelar Assinatura
admin-billing-invoice-date = Data da Fatura
admin-billing-invoice-amount = Valor
admin-billing-invoice-status = Status
admin-billing-invoice-paid = Pago
admin-billing-invoice-pending = Pendente
admin-billing-invoice-overdue = Atrasado
admin-billing-invoice-download = Baixar Fatura

# -----------------------------------------------------------------------------
# Backup e Restauração
# -----------------------------------------------------------------------------
admin-backup-title = Backup e Restauração
admin-backup-create = Criar Backup
admin-backup-restore = Restaurar Backup
admin-backup-schedule = Agendar Backups
admin-backup-list = Histórico de Backups
admin-backup-name = Nome do Backup
admin-backup-size = Tamanho
admin-backup-created = Criado em
admin-backup-download = Baixar
admin-backup-delete = Excluir
admin-backup-confirm-restore = Tem certeza que deseja restaurar este backup? Isso irá sobrescrever os dados atuais.
admin-backup-confirm-delete = Tem certeza que deseja excluir este backup?
admin-backup-in-progress = Backup em andamento...
admin-backup-completed = Backup concluído com sucesso
admin-backup-failed = Falha no backup
admin-backup-restore-in-progress = Restauração em andamento...
admin-backup-restore-completed = Restauração concluída com sucesso
admin-backup-restore-failed = Falha na restauração

# -----------------------------------------------------------------------------
# Modo de Manutenção
# -----------------------------------------------------------------------------
admin-maintenance-title = Modo de Manutenção
admin-maintenance-enable = Ativar Modo de Manutenção
admin-maintenance-disable = Desativar Modo de Manutenção
admin-maintenance-status = Status Atual
admin-maintenance-active = Modo de manutenção está ativo
admin-maintenance-inactive = Modo de manutenção está inativo
admin-maintenance-message = Mensagem de Manutenção
admin-maintenance-default-message = Estamos realizando manutenção programada. Por favor, volte em breve.
admin-maintenance-allowed-ips = Endereços IP Permitidos
admin-maintenance-confirm-enable = Tem certeza que deseja ativar o modo de manutenção? Os usuários não poderão acessar o sistema.

# -----------------------------------------------------------------------------
# Elementos Comuns da Interface Admin
# -----------------------------------------------------------------------------
admin-required = Obrigatório
admin-optional = Opcional
admin-loading = Carregando...
admin-saving = Salvando...
admin-deleting = Excluindo...
admin-confirm = Confirmar
admin-cancel = Cancelar
admin-save = Salvar
admin-create = Criar
admin-update = Atualizar
admin-delete = Excluir
admin-edit = Editar
admin-view = Visualizar
admin-close = Fechar
admin-back = Voltar
admin-next = Próximo
admin-previous = Anterior
admin-refresh = Atualizar
admin-export = Exportar
admin-import = Importar
admin-search = Buscar
admin-filter = Filtrar
admin-clear = Limpar
admin-select = Selecionar
admin-select-all = Selecionar Todos
admin-deselect-all = Desmarcar Todos
admin-actions = Ações
admin-more-actions = Mais Ações
admin-no-data = Nenhum dado disponível
admin-error = Ocorreu um erro
admin-success = Sucesso
admin-warning = Atenção
admin-info = Informação

# Paginação de Tabelas
admin-showing = Mostrando { $from } a { $to } de { $total } resultados
admin-page = Página { $current } de { $total }
admin-items-per-page = Itens por página
admin-go-to-page = Ir para página

# Ações em Massa
admin-bulk-delete = Excluir Selecionados
admin-bulk-export = Exportar Selecionados
admin-bulk-activate = Ativar Selecionados
admin-bulk-deactivate = Desativar Selecionados
admin-selected-count = { $count ->
    [one] { $count } item selecionado
   *[other] { $count } itens selecionados
}

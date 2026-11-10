# General Bots - Mensagens de Erro (Português Brasileiro)
# Este arquivo contém todas as traduções de mensagens de erro

# =============================================================================
# Erros HTTP
# =============================================================================

error-http-400 = Requisição inválida. Por favor, verifique seus dados.
error-http-401 = Autenticação necessária. Por favor, faça login.
error-http-403 = Você não tem permissão para acessar este recurso.
error-http-404 = { $entity } não encontrado.
error-http-409 = Conflito: { $message }
error-http-429 = Muitas requisições. Por favor, aguarde { $seconds } segundos.
error-http-500 = Erro interno do servidor. Por favor, tente novamente mais tarde.
error-http-502 = Gateway inválido. O servidor recebeu uma resposta inválida.
error-http-503 = Serviço temporariamente indisponível. Por favor, tente novamente mais tarde.
error-http-504 = Tempo limite da requisição excedido após { $milliseconds }ms.

# =============================================================================
# Erros de Validação
# =============================================================================

error-validation-required = { $field } é obrigatório.
error-validation-email = Por favor, insira um endereço de e-mail válido.
error-validation-url = Por favor, insira uma URL válida.
error-validation-phone = Por favor, insira um número de telefone válido.
error-validation-min-length = { $field } deve ter pelo menos { $min } caracteres.
error-validation-max-length = { $field } deve ter no máximo { $max } caracteres.
error-validation-min-value = { $field } deve ser pelo menos { $min }.
error-validation-max-value = { $field } deve ser no máximo { $max }.
error-validation-pattern = O formato de { $field } é inválido.
error-validation-unique = { $field } já existe.
error-validation-mismatch = { $field } não corresponde a { $other }.
error-validation-date-format = Por favor, insira uma data válida no formato { $format }.
error-validation-date-past = { $field } deve estar no passado.
error-validation-date-future = { $field } deve estar no futuro.

# =============================================================================
# Erros de Autenticação
# =============================================================================

error-auth-invalid-credentials = E-mail ou senha inválidos.
error-auth-account-locked = Sua conta foi bloqueada. Por favor, entre em contato com o suporte.
error-auth-account-disabled = Sua conta foi desativada.
error-auth-session-expired = Sua sessão expirou. Por favor, faça login novamente.
error-auth-token-invalid = Token inválido ou expirado.
error-auth-token-missing = Token de autenticação é obrigatório.
error-auth-mfa-required = Autenticação de dois fatores é obrigatória.
error-auth-mfa-invalid = Código de verificação inválido.
error-auth-password-weak = A senha é muito fraca. Por favor, use uma senha mais forte.
error-auth-password-expired = Sua senha expirou. Por favor, redefina-a.

# =============================================================================
# Erros de Configuração
# =============================================================================

error-config = Erro de configuração: { $message }
error-config-missing = Configuração ausente: { $key }
error-config-invalid = Valor de configuração inválido para { $key }: { $reason }
error-config-file-not-found = Arquivo de configuração não encontrado: { $path }
error-config-parse = Falha ao analisar configuração: { $message }

# =============================================================================
# Erros de Banco de Dados
# =============================================================================

error-database = Erro de banco de dados: { $message }
error-database-connection = Falha ao conectar ao banco de dados.
error-database-timeout = Operação do banco de dados expirou.
error-database-constraint = Violação de restrição do banco de dados: { $constraint }
error-database-duplicate = Um registro com este { $field } já existe.
error-database-migration = Migração do banco de dados falhou: { $message }

# =============================================================================
# Erros de Arquivo e Armazenamento
# =============================================================================

error-file-not-found = Arquivo não encontrado: { $filename }
error-file-too-large = Arquivo muito grande. Tamanho máximo é { $maxSize }.
error-file-type-not-allowed = Tipo de arquivo não permitido. Tipos permitidos: { $allowedTypes }.
error-file-upload-failed = Falha no envio do arquivo: { $message }
error-file-read = Falha ao ler arquivo: { $message }
error-file-write = Falha ao escrever arquivo: { $message }
error-storage-full = Cota de armazenamento excedida.
error-storage-unavailable = Serviço de armazenamento indisponível.

# =============================================================================
# Erros de Rede e Serviços Externos
# =============================================================================

error-network = Erro de rede: { $message }
error-network-timeout = Conexão expirou.
error-network-unreachable = Servidor inacessível.
error-service-unavailable = Serviço indisponível: { $service }
error-external-api = Erro de API externa: { $message }
error-rate-limit = Limite de requisições excedido. Tente novamente após { $seconds }s.

# =============================================================================
# Erros de Bot e Diálogo
# =============================================================================

error-bot-not-found = Bot não encontrado: { $botId }
error-bot-disabled = Este bot está desativado no momento.
error-bot-script-error = Erro de script na linha { $line }: { $message }
error-bot-timeout = Tempo de resposta do bot expirou.
error-bot-quota-exceeded = Cota de uso do bot excedida.
error-dialog-not-found = Diálogo não encontrado: { $dialogId }
error-dialog-invalid = Configuração de diálogo inválida: { $message }

# =============================================================================
# Erros de LLM e IA
# =============================================================================

error-llm-unavailable = Serviço de IA está indisponível no momento.
error-llm-timeout = Tempo limite da requisição de IA expirou.
error-llm-rate-limit = Limite de requisições de IA excedido. Por favor, aguarde antes de tentar novamente.
error-llm-content-filter = Conteúdo foi filtrado pelas diretrizes de segurança.
error-llm-context-length = Entrada muito longa. Por favor, encurte sua mensagem.
error-llm-invalid-response = Resposta inválida recebida do serviço de IA.
error-llm-empty-response = Desculpe, não consegui processar sua mensagem agora. Tente novamente em alguns segundos.

# =============================================================================
# Erros de E-mail
# =============================================================================

error-email-send-failed = Falha ao enviar e-mail: { $message }
error-email-invalid-recipient = Endereço de e-mail do destinatário inválido: { $email }
error-email-attachment-failed = Falha ao anexar arquivo: { $filename }
error-email-template-not-found = Modelo de e-mail não encontrado: { $template }

# =============================================================================
# Erros de Calendário e Agendamento
# =============================================================================

error-calendar-conflict = Horário conflita com evento existente.
error-calendar-past-date = Não é possível agendar eventos no passado.
error-calendar-invalid-recurrence = Padrão de recorrência inválido.
error-calendar-event-not-found = Evento não encontrado: { $eventId }

# =============================================================================
# Erros de Tarefa
# =============================================================================

error-task-not-found = Tarefa não encontrada: { $taskId }
error-task-already-completed = A tarefa já foi concluída.
error-task-circular-dependency = Dependência circular detectada nas tarefas.
error-task-invalid-status = Transição de status de tarefa inválida.

# =============================================================================
# Erros de Permissão
# =============================================================================

error-permission-denied = Você não tem permissão para realizar esta ação.
error-permission-resource = Você não tem acesso a este { $resource }.
error-permission-action = Você não pode { $action } este { $resource }.
error-permission-owner-only = Apenas o proprietário pode realizar esta ação.

# =============================================================================
# Erros Genéricos
# =============================================================================

error-internal = Erro interno: { $message }
error-unexpected = Ocorreu um erro inesperado. Por favor, tente novamente.
error-not-implemented = Este recurso ainda não foi implementado.
error-maintenance = Sistema em manutenção. Por favor, tente novamente mais tarde.
error-unknown = Ocorreu um erro desconhecido.

# =============================================================================
# Contrato de API localizado (#2094) — códigos estáveis, mensagens localizadas
# =============================================================================

error-locale-invalid = O idioma solicitado não é válido.
error-locale-unsupported = Este idioma não está disponível.
error-locale-persist-failed = Não foi possível salvar sua preferência de idioma.
error-bot-id-required = O id do bot é obrigatório.
error-bot-id-invalid = O id do bot não é válido.
error-bot-name-required = O nome do bot é obrigatório.
error-token-invalid-format = O formato do token não é válido.
error-token-signing-failed = Não foi possível assinar o token.
error-token-signature-invalid = A assinatura do token não é válida.
error-token-payload-invalid = O conteúdo do token não é válido.
error-token-payload-json-invalid = O conteúdo do token não é um JSON válido.
error-cloud-token-required = Informe um cloud_token.
error-llm-subscription-token-missing = Token de assinatura ausente.
error-llm-token-store-unavailable = O armazenamento de tokens está indisponível.
error-llm-subscription-token-expired = O token de assinatura expirou; envie a mensagem novamente.
error-llm-prompt-missing = Não há prompt na requisição.
error-llm-account-unlinked = Conecte sua conta Claude ou ChatGPT em Configurações → Contas de IA primeiro.

error-timeout = A solicitação expirou. Tente novamente.

# =============================================================================
# Erros de API localizados — rotas, sincronização, pares, uso do computador (#2094)
# =============================================================================

error-agent-drive-database-unavailable = O armazenamento de arquivos está temporariamente indisponível.
error-agent-drive-lookup-failed = Não foi possível ler a conversa. Tente novamente.
error-agent-drive-conversation-not-found = Conversa não encontrada.
error-agent-drive-bad-owner = O registro do dono da conversa não é válido.
error-agent-drive-forbidden = Esta conversa pertence a outro usuário.
error-agent-drive-bad-bot = O registro do bot desta conversa não é válido.
error-agent-drive-workspace-stopped = O workspace do agente não está em execução.
error-agent-drive-no-files = Não há arquivos na pasta de saída do workspace.
error-agent-drive-named-files-missing = Os arquivos solicitados não estão na pasta de saída do workspace.
error-agent-drive-save-failed = Não foi possível salvar nenhum arquivo no Drive.
error-agent-drive-file-missing = O arquivo não está na pasta de saída do workspace.
error-agent-drive-save-one-failed = Não foi possível salvar o arquivo no Drive.
error-agent-drive-bad-path = O caminho do Drive retornado pelo armazenamento não é válido.
error-agent-drive-link-failed = Não foi possível criar o link de compartilhamento.
error-subagent-no-llm = Nenhum provedor de LLM está configurado para subagentes.
error-goal-run-not-running = Esta execução de meta não está em andamento.
error-goal-run-not-paused = Esta execução de meta não está pausada.
error-autotask-not-found = Tarefa automática não encontrada.
error-autotask-intent-required = Descreva o que a tarefa deve fazer.
error-peer-network-disabled = A rede de pares está desativada neste nó.
error-peer-network-unauthorized = É necessário um token de par válido.
error-peer-network-invalid = A requisição de par não é válida: { $message }
error-peer-network-unknown-node = Nó de par desconhecido: { $message }
error-peer-network-registry-full = O registro de pares está cheio.
error-peer-network-payload-store-full = O armazenamento de payloads de par está cheio: { $message }
error-peer-network-recipient-required = O parâmetro de consulta recipient é obrigatório.
error-drive-sync-field-required = { $field } é obrigatório.
error-drive-sync-field-absolute = { $field } deve ser um caminho absoluto.
error-drive-sync-field-parent = { $field } não pode conter '..'.
error-drive-sync-field-not-folder = { $field } não é uma pasta nesta máquina: { $path }
error-drive-sync-field-local-folder = A pasta local
error-drive-sync-field-drive-folder = A pasta do Drive
error-drive-sync-job-not-found = A tarefa de sincronização { $id } não foi encontrada.
error-drive-sync-job-name-required = O nome da tarefa de sincronização é obrigatório.
error-drive-sync-job-exists = Já existe uma tarefa de sincronização chamada { $name }.
error-drive-sync-conflict-path-required = O caminho do conflito é obrigatório.
error-drive-sync-conflict-path-escape = O caminho do conflito escapa da pasta de sincronização.
error-drive-sync-task-failed = A tarefa { $task } falhou: { $error }
error-org-auth-service-unavailable = O serviço de diretório não está disponível.
error-org-default-delete = A organização padrão não pode ser excluída.
error-org-settings-save-failed = Não foi possível salvar as configurações da organização.
error-org-name-required = O nome da organização é obrigatório.
error-org-tenant-missing = Ainda não existe tenant — execute as migrações ou o bootstrap primeiro.
error-computer-use-session-required = O id da sessão é obrigatório.
error-computer-use-kind-invalid = O tipo deve ser 'browser' ou 'terminal'.
error-computer-use-action-not-found = Ação não encontrada.
error-computer-use-no-consent = Não há registro de consentimento para esta ação; aprove primeiro.
error-computer-use-browser-only = Esta ação deve ser executada no painel do navegador.
error-computer-use-url-missing = A ação não tem URL.
error-terminal-approval-required = A execução no terminal requer aprovação explícita do usuário.
error-terminal-action-id-required = A execução no terminal requer um id de ação aprovado.
error-terminal-command-required = O comando é obrigatório.
error-terminal-command-too-long = O comando está limitado a 16000 caracteres.
error-terminal-blocked-pattern = O comando corresponde a um padrão destrutivo bloqueado.
error-terminal-workspace-stopped = O workspace do agente não está em execução. Conclua o provisionamento (ou reinicie o aplicativo) e tente novamente.
error-terminal-timeout = O comando expirou.

# =============================================================================
# Colaboração, grupos, autotarefas e contas de IA (#2094)
# =============================================================================

error-db-unavailable = O banco de dados está indisponível no momento. Tente novamente.
error-collab-invalid-resource = O tipo ou o id do recurso é inválido.
error-collab-owner-only = Apenas o proprietário pode fazer isso.
error-collab-invalid-grantee-type = O tipo de destinatário deve ser 'user' ou 'domain'.
error-collab-invalid-role = A função é inválida.
error-collab-invalid-grantee-id = O id do destinatário é inválido.
error-collab-grant-not-found = A concessão não foi encontrada.
error-collab-invalid-new-owner = O id do novo proprietário é inválido.
error-collab-link-not-found = O link de compartilhamento não foi encontrado.
error-collab-comment-not-found = O comentário não foi encontrado.
error-collab-comment-resolve-forbidden = O comentário não foi encontrado ou não é seu para resolver.
error-collab-comment-body-required = O corpo do comentário é obrigatório.
error-collab-comment-too-long = O comentário é limitado a 8000 caracteres.
error-collab-invalid-emoji = O emoji é inválido.
error-collab-invalid-cursor = O cursor 'before' é inválido.
error-collab-invalid-action = A ação é inválida.
error-collab-invalid-name = O nome é inválido.
error-collab-content-required = O conteúdo é obrigatório.
error-collab-content-too-large = O conteúdo é muito grande.
error-collab-version-not-found = A versão não foi encontrada.
error-collab-invalid-base-version = A versão base é inválida.
error-collab-invalid-parent = O id do pai é inválido.
error-collab-invalid-op-type = O tipo de operação é inválido.
error-collab-operation-not-found = A operação não foi encontrada.
error-collab-not-a-conflict = A operação não é um conflito não resolvido.
error-collab-invalid-resolution = A resolução deve ser 'accept-server' ou 'accept-client'.
error-group-budget-not-found = O orçamento do grupo não foi encontrado.
error-group-budget-invalid-id = O id do orçamento é inválido.
error-group-budget-amount-invalid = O valor deve ser zero ou positivo.
error-group-budget-caps-invalid = Os limites devem ser zero ou positivos (zero significa sem limite).
error-autotask-fields-required = Título, agendamento e comando são obrigatórios.
error-autotask-kind-invalid = O tipo deve ser 'cron' ou 'systemd'.
error-conversation-not-found = A conversa não foi encontrada.
error-conversation-session-invalid = A sessão da conversa é inválida.
error-subagent-invalid-tasks = As tarefas do subagente são inválidas.
error-ai-account-unknown-provider = O provedor de conta de IA é desconhecido.
error-ai-account-workspace-unavailable = O workspace do agente não está disponível nesta versão.
error-ai-account-agent-invalid = O agente deve ser reasoning, agentic ou fast.
error-ai-account-not-connected-vibe = Conecte a conta antes de atribuí-la ao Vibe.
error-ai-account-not-connected-chat = Conecte a conta antes de selecioná-la para o chat.
error-computer-use-action-id-mismatch = O id da ação não corresponde ao caminho.
error-http-no-route = Nenhuma rota corresponde a este caminho.
error-goal-run-advance-refused = A execução não pode avançar no estado atual.

# =============================================================================
# People, attendance and products routes (#2094)
# =============================================================================

error-person-not-found = Pessoa não encontrada.
error-team-not-found = Equipe não encontrada.
error-time-off-not-found = Solicitação de folga não encontrada.
error-people-invalid-start-date = A data inicial é inválida. Use o formato AAAA-MM-DD.
error-people-invalid-end-date = A data final é inválida. Use o formato AAAA-MM-DD.
error-attendance-webhook-not-found = Webhook não encontrado.
error-attendance-session-id-invalid = O id da sessão é inválido.
error-attendance-session-not-found = Sessão não encontrada.
error-attendance-attendant-id-required = O id do atendente é obrigatório.
attendance-no-attendant-matching-skills = Nenhum atendente foi encontrado com as habilidades exigidas.
attendance-response-sent = Resposta enviada.
error-product-not-found = Produto não encontrado.
error-service-not-found = Serviço não encontrado.
error-service-invalid-value = O valor do serviço é inválido.
error-service-price-required = Este serviço não tem preço; informe um valor.

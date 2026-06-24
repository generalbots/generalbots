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

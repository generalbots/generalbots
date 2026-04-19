# =============================================================================
# General Bots - Authentication Translations (Portuguese - Brazil)
# =============================================================================
# Traduções de autenticação, Passkey/WebAuthn e segurança
# =============================================================================

# -----------------------------------------------------------------------------
# Authentication General
# -----------------------------------------------------------------------------
auth-title = Autenticação
auth-login = Entrar
auth-logout = Sair
auth-signup = Cadastrar
auth-welcome = Bem-vindo
auth-welcome-back = Bem-vindo de volta, { $name }!
auth-session-expired = Sua sessão expirou
auth-session-timeout = Sessão expira em { $minutes } minutos

# -----------------------------------------------------------------------------
# Login Form
# -----------------------------------------------------------------------------
auth-login-title = Entre na sua conta
auth-login-subtitle = Digite suas credenciais para continuar
auth-login-email = Endereço de E-mail
auth-login-username = Nome de Usuário
auth-login-password = Senha
auth-login-remember = Lembrar-me
auth-login-forgot = Esqueceu a senha?
auth-login-submit = Entrar
auth-login-loading = Entrando...
auth-login-or = ou continue com
auth-login-no-account = Não tem uma conta?
auth-login-create-account = Criar uma conta

# -----------------------------------------------------------------------------
# Passkey/WebAuthn
# -----------------------------------------------------------------------------
passkey-title = Chaves de Acesso
passkey-subtitle = Autenticação segura sem senha
passkey-description = Chaves de acesso usam a biometria ou PIN do seu dispositivo para login seguro e resistente a phishing
passkey-what-is = O que é uma chave de acesso?
passkey-benefits = Benefícios das chaves de acesso
passkey-benefit-secure = Mais seguro que senhas
passkey-benefit-easy = Fácil de usar - sem senhas para lembrar
passkey-benefit-fast = Login rápido com biometria
passkey-benefit-phishing = Resistente a ataques de phishing

# -----------------------------------------------------------------------------
# Passkey Registration
# -----------------------------------------------------------------------------
passkey-register-title = Configurar Chave de Acesso
passkey-register-subtitle = Crie uma chave de acesso para login mais rápido e seguro
passkey-register-description = Seu dispositivo pedirá para verificar sua identidade usando impressão digital, rosto ou bloqueio de tela
passkey-register-button = Criar Chave de Acesso
passkey-register-name = Nome da Chave de Acesso
passkey-register-name-placeholder = ex: MacBook Pro, iPhone
passkey-register-name-hint = Dê um nome à sua chave de acesso para identificá-la depois
passkey-register-loading = Configurando chave de acesso...
passkey-register-verifying = Verificando com seu dispositivo...
passkey-register-success = Chave de acesso criada com sucesso
passkey-register-error = Falha ao criar chave de acesso
passkey-register-cancelled = Configuração de chave de acesso cancelada
passkey-register-not-supported = Seu navegador não suporta chaves de acesso

# -----------------------------------------------------------------------------
# Passkey Authentication
# -----------------------------------------------------------------------------
passkey-login-title = Entrar com Chave de Acesso
passkey-login-subtitle = Use sua chave de acesso para login seguro sem senha
passkey-login-button = Entrar com Chave de Acesso
passkey-login-loading = Autenticando...
passkey-login-verifying = Verificando chave de acesso...
passkey-login-success = Login realizado com sucesso
passkey-login-error = Falha na autenticação
passkey-login-cancelled = Autenticação cancelada
passkey-login-no-passkeys = Nenhuma chave de acesso encontrada para esta conta
passkey-login-try-another = Tentar outro método

# -----------------------------------------------------------------------------
# Passkey Management
# -----------------------------------------------------------------------------
passkey-manage-title = Gerenciar Chaves de Acesso
passkey-manage-subtitle = Visualize e gerencie suas chaves de acesso registradas
passkey-manage-count = { $count ->
    [one] { $count } chave de acesso registrada
   *[other] { $count } chaves de acesso registradas
}
passkey-manage-add = Adicionar Nova Chave de Acesso
passkey-manage-rename = Renomear
passkey-manage-delete = Excluir
passkey-manage-created = Criada em { $date }
passkey-manage-last-used = Último uso em { $date }
passkey-manage-never-used = Nunca usada
passkey-manage-this-device = Este dispositivo
passkey-manage-cross-platform = Multiplataforma
passkey-manage-platform = Autenticador de plataforma
passkey-manage-security-key = Chave de segurança
passkey-manage-empty = Nenhuma chave de acesso registrada
passkey-manage-empty-description = Adicione uma chave de acesso para login mais rápido e seguro

# -----------------------------------------------------------------------------
# Passkey Deletion
# -----------------------------------------------------------------------------
passkey-delete-title = Excluir Chave de Acesso
passkey-delete-confirm = Tem certeza de que deseja excluir esta chave de acesso?
passkey-delete-warning = Você não poderá mais usar esta chave de acesso para entrar
passkey-delete-last-warning = Esta é sua única chave de acesso. Você precisará usar autenticação por senha após excluí-la.
passkey-delete-success = Chave de acesso excluída com sucesso
passkey-delete-error = Falha ao excluir chave de acesso

# -----------------------------------------------------------------------------
# Password Fallback
# -----------------------------------------------------------------------------
passkey-fallback-title = Usar Senha
passkey-fallback-description = Se você não pode usar sua chave de acesso, pode entrar com sua senha
passkey-fallback-button = Usar Senha
passkey-fallback-or-passkey = Ou entre com chave de acesso
passkey-fallback-setup-prompt = Configure uma chave de acesso para login mais rápido na próxima vez
passkey-fallback-setup-later = Talvez depois
passkey-fallback-setup-now = Configurar agora
passkey-fallback-locked = Conta temporariamente bloqueada
passkey-fallback-locked-description = Muitas tentativas falhas. Tente novamente em { $minutes } minutos.
passkey-fallback-attempts = { $remaining } tentativas restantes

# -----------------------------------------------------------------------------
# Multi-Factor Authentication
# -----------------------------------------------------------------------------
mfa-title = Autenticação de Dois Fatores
mfa-subtitle = Adicione uma camada extra de segurança à sua conta
mfa-enabled = Autenticação de dois fatores está ativada
mfa-disabled = Autenticação de dois fatores está desativada
mfa-enable = Ativar 2FA
mfa-disable = Desativar 2FA
mfa-setup = Configurar 2FA
mfa-verify = Verificar Código
mfa-code = Código de Verificação
mfa-code-placeholder = Digite o código de 6 dígitos
mfa-code-sent = Código enviado para { $destination }
mfa-code-expired = O código expirou
mfa-code-invalid = Código inválido
mfa-resend = Reenviar código
mfa-resend-in = Reenviar em { $seconds }s
mfa-methods = Métodos de Autenticação
mfa-method-app = Aplicativo Autenticador
mfa-method-sms = SMS
mfa-method-email = E-mail
mfa-method-passkey = Chave de Acesso
mfa-backup-codes = Códigos de Backup
mfa-backup-codes-description = Guarde esses códigos em um lugar seguro. Cada código só pode ser usado uma vez.
mfa-backup-codes-remaining = { $count } códigos de backup restantes
mfa-backup-codes-generate = Gerar Novos Códigos
mfa-backup-codes-download = Baixar Códigos
mfa-backup-codes-copy = Copiar Códigos

# -----------------------------------------------------------------------------
# Password Management
# -----------------------------------------------------------------------------
password-title = Senha
password-change = Alterar Senha
password-current = Senha Atual
password-new = Nova Senha
password-confirm = Confirmar Nova Senha
password-requirements = Requisitos da Senha
password-requirement-length = Pelo menos { $length } caracteres
password-requirement-uppercase = Pelo menos uma letra maiúscula
password-requirement-lowercase = Pelo menos uma letra minúscula
password-requirement-number = Pelo menos um número
password-requirement-special = Pelo menos um caractere especial
password-strength = Força da Senha
password-strength-weak = Fraca
password-strength-fair = Razoável
password-strength-good = Boa
password-strength-strong = Forte
password-match = As senhas coincidem
password-mismatch = As senhas não coincidem
password-changed = Senha alterada com sucesso
password-change-error = Falha ao alterar senha

# -----------------------------------------------------------------------------
# Password Reset
# -----------------------------------------------------------------------------
password-reset-title = Redefinir Senha
password-reset-subtitle = Digite seu e-mail para receber um link de redefinição
password-reset-email-sent = E-mail de redefinição de senha enviado
password-reset-email-sent-description = Verifique seu e-mail para instruções de redefinição de senha
password-reset-invalid-token = Link de redefinição inválido ou expirado
password-reset-success = Senha redefinida com sucesso
password-reset-error = Falha ao redefinir senha

# -----------------------------------------------------------------------------
# Session Management
# -----------------------------------------------------------------------------
session-title = Sessões Ativas
session-subtitle = Gerencie suas sessões ativas em diferentes dispositivos
session-current = Sessão Atual
session-device = Dispositivo
session-location = Localização
session-last-active = Última Atividade
session-ip-address = Endereço IP
session-browser = Navegador
session-os = Sistema Operacional
session-sign-out = Encerrar Sessão
session-sign-out-all = Encerrar Todas as Outras Sessões
session-sign-out-confirm = Tem certeza de que deseja encerrar esta sessão?
session-sign-out-all-confirm = Tem certeza de que deseja encerrar todas as outras sessões?

# -----------------------------------------------------------------------------
# Security Settings
# -----------------------------------------------------------------------------
security-title = Segurança
security-subtitle = Gerencie as configurações de segurança da sua conta
security-overview = Visão Geral de Segurança
security-last-login = Último Login
security-password-last-changed = Última Alteração de Senha
security-security-checkup = Verificação de Segurança
security-checkup-description = Revise suas configurações de segurança
security-recommendation = Recomendação
security-add-passkey = Adicione uma chave de acesso para login mais seguro
security-enable-mfa = Ative a autenticação de dois fatores
security-update-password = Atualize sua senha regularmente

# -----------------------------------------------------------------------------
# Error Messages
# -----------------------------------------------------------------------------
auth-error-invalid-credentials = E-mail ou senha inválidos
auth-error-account-locked = Conta bloqueada. Por favor, entre em contato com o suporte.
auth-error-account-disabled = A conta foi desativada
auth-error-email-not-verified = Por favor, verifique seu endereço de e-mail
auth-error-too-many-attempts = Muitas tentativas falhas. Por favor, tente novamente mais tarde.
auth-error-network = Erro de rede. Por favor, verifique sua conexão.
auth-error-server = Erro do servidor. Por favor, tente novamente mais tarde.
auth-error-unknown = Ocorreu um erro desconhecido
auth-error-session-invalid = Sessão inválida. Por favor, entre novamente.
auth-error-token-expired = Sua sessão expirou. Por favor, entre novamente.
auth-error-unauthorized = Você não está autorizado a realizar esta ação

# -----------------------------------------------------------------------------
# Success Messages
# -----------------------------------------------------------------------------
auth-success-login = Login realizado com sucesso
auth-success-logout = Logout realizado com sucesso
auth-success-signup = Conta criada com sucesso
auth-success-password-changed = Senha alterada com sucesso
auth-success-email-verified = E-mail verificado com sucesso
auth-success-mfa-enabled = Autenticação de dois fatores ativada
auth-success-mfa-disabled = Autenticação de dois fatores desativada
auth-success-session-terminated = Sessão encerrada com sucesso

# -----------------------------------------------------------------------------
# Notifications
# -----------------------------------------------------------------------------
auth-notify-new-login = Novo login de { $device } em { $location }
auth-notify-password-changed = Sua senha foi alterada
auth-notify-mfa-enabled = Autenticação de dois fatores foi ativada
auth-notify-passkey-added = Nova chave de acesso foi adicionada à sua conta
auth-notify-suspicious-activity = Atividade suspeita detectada em sua conta

notification-title-new-message = Nova Mensagem
notification-title-task-due = Tarefa Vencendo
notification-title-task-assigned = Tarefa Atribuída
notification-title-task-completed = Tarefa Concluída
notification-title-meeting-reminder = Lembrete de Reunião
notification-title-meeting-started = Reunião Iniciada
notification-title-file-shared = Arquivo Compartilhado
notification-title-file-uploaded = Arquivo Enviado
notification-title-comment-added = Novo Comentário
notification-title-mention = Você foi mencionado
notification-title-system = Notificação do Sistema
notification-title-security = Alerta de Segurança
notification-title-update = Atualização Disponível
notification-title-error = Erro Ocorrido
notification-title-success = Sucesso
notification-title-warning = Atenção
notification-title-info = Informação

notification-message-new = Você tem uma nova mensagem de { $sender }
notification-message-unread = Você tem { $count ->
    [one] { $count } mensagem não lida
   *[other] { $count } mensagens não lidas
}
notification-task-due-soon = A tarefa "{ $task }" vence em { $time }
notification-task-due-today = A tarefa "{ $task }" vence hoje
notification-task-due-overdue = A tarefa "{ $task }" está atrasada há { $time }
notification-task-assigned-to-you = Você foi atribuído à tarefa "{ $task }"
notification-task-assigned-by = { $assigner } atribuiu você à tarefa "{ $task }"
notification-task-completed-by = { $user } concluiu a tarefa "{ $task }"
notification-task-status-changed = O status da tarefa "{ $task }" mudou para { $status }

notification-meeting-in-minutes = A reunião "{ $meeting }" começa em { $minutes } minutos
notification-meeting-starting-now = A reunião "{ $meeting }" está começando agora
notification-meeting-cancelled = A reunião "{ $meeting }" foi cancelada
notification-meeting-rescheduled = A reunião "{ $meeting }" foi reagendada para { $datetime }
notification-meeting-invite = { $inviter } convidou você para "{ $meeting }"
notification-meeting-response = { $user } { $response } seu convite de reunião

notification-file-shared-with-you = { $sharer } compartilhou "{ $filename }" com você
notification-file-uploaded-by = { $uploader } enviou "{ $filename }"
notification-file-modified = "{ $filename }" foi modificado por { $user }
notification-file-deleted = "{ $filename }" foi excluído por { $user }
notification-file-download-ready = Seu arquivo "{ $filename }" está pronto para download
notification-file-upload-complete = Upload de "{ $filename }" concluído com sucesso
notification-file-upload-failed = Falha no upload de "{ $filename }"

notification-comment-on-task = { $user } comentou na tarefa "{ $task }"
notification-comment-on-file = { $user } comentou em "{ $filename }"
notification-comment-reply = { $user } respondeu ao seu comentário
notification-mention-in-comment = { $user } mencionou você em um comentário
notification-mention-in-chat = { $user } mencionou você em { $channel }

notification-login-new-device = Novo login detectado de { $device } em { $location }
notification-login-failed = Tentativa de login falhou em sua conta
notification-password-changed = Sua senha foi alterada com sucesso
notification-password-expiring = Sua senha expira em { $days } dias
notification-session-expired = Sua sessão expirou
notification-account-locked = Sua conta foi bloqueada
notification-two-factor-enabled = Autenticação de dois fatores foi ativada
notification-two-factor-disabled = Autenticação de dois fatores foi desativada

notification-subscription-expiring = Sua assinatura expira em { $days } dias
notification-subscription-expired = Sua assinatura expirou
notification-subscription-renewed = Sua assinatura foi renovada até { $date }
notification-payment-successful = Pagamento de { $amount } realizado com sucesso
notification-payment-failed = Pagamento de { $amount } falhou
notification-invoice-ready = Sua fatura de { $period } está pronta

notification-bot-response = { $bot } respondeu à sua consulta
notification-bot-error = { $bot } encontrou um erro
notification-bot-offline = { $bot } está offline no momento
notification-bot-online = { $bot } está online agora
notification-bot-updated = { $bot } foi atualizado

notification-system-maintenance = Manutenção do sistema agendada para { $datetime }
notification-system-update = Atualização do sistema disponível: { $version }
notification-system-restored = Sistema foi restaurado
notification-system-degraded = Sistema está com desempenho degradado

notification-action-view = Ver
notification-action-dismiss = Dispensar
notification-action-mark-read = Marcar como lida
notification-action-mark-all-read = Marcar todas como lidas
notification-action-settings = Configurações de notificação
notification-action-reply = Responder
notification-action-open = Abrir
notification-action-join = Entrar
notification-action-accept = Aceitar
notification-action-decline = Recusar

notification-time-just-now = Agora mesmo
notification-time-minutes = { $count ->
    [one] { $count } minuto atrás
   *[other] { $count } minutos atrás
}
notification-time-hours = { $count ->
    [one] { $count } hora atrás
   *[other] { $count } horas atrás
}
notification-time-days = { $count ->
    [one] { $count } dia atrás
   *[other] { $count } dias atrás
}
notification-time-weeks = { $count ->
    [one] { $count } semana atrás
   *[other] { $count } semanas atrás
}

notification-preference-all = Todas as notificações
notification-preference-important = Apenas importantes
notification-preference-none = Nenhuma
notification-preference-email = Notificações por e-mail
notification-preference-push = Notificações push
notification-preference-in-app = Notificações no aplicativo
notification-preference-sound = Som ativado
notification-preference-vibration = Vibração ativada

notification-empty = Sem notificações
notification-empty-description = Você está em dia!
notification-load-more = Carregar mais
notification-clear-all = Limpar todas as notificações
notification-filter-all = Todas
notification-filter-unread = Não lidas
notification-filter-mentions = Menções
notification-filter-tasks = Tarefas
notification-filter-messages = Mensagens
notification-filter-system = Sistema

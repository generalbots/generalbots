bot-greeting-default = Olá! Como posso ajudar você hoje?
bot-greeting-named = Olá, { $name }! Como posso ajudar você hoje?
bot-goodbye = Até logo! Tenha um ótimo dia!
bot-help-prompt = Posso ajudar com: { $topics }. O que você gostaria de saber?
bot-thank-you = Obrigado pela sua mensagem. Como posso ajudá-lo?
bot-echo-intro = Bot Echo: Vou repetir tudo que você disser. Digite 'sair' para encerrar.
bot-you-said = Você disse: { $message }
bot-thinking = Deixe-me pensar sobre isso...
bot-processing = Processando sua solicitação...
bot-error-occurred = Desculpe, algo deu errado. Por favor, tente novamente.
bot-not-understood = Não entendi. Você poderia reformular?
bot-confirm-action = Tem certeza que deseja continuar?
bot-action-cancelled = Ação cancelada.
bot-action-completed = Pronto!

bot-lead-welcome = Bem-vindo! Deixe-me ajudá-lo a começar.
bot-lead-ask-name = Qual é o seu nome?
bot-lead-ask-email = E seu e-mail?
bot-lead-ask-company = De qual empresa você é?
bot-lead-ask-phone = Qual é o seu telefone?
bot-lead-hot = Ótimo! Nossa equipe de vendas entrará em contato em breve.
bot-lead-nurture = Obrigado pelo seu interesse! Enviaremos alguns materiais.
bot-lead-score = Sua pontuação de lead é { $score } de 100.
bot-lead-saved = Suas informações foram salvas com sucesso.

bot-schedule-created = Executando tarefa agendada: { $name }
bot-schedule-next = Próxima execução agendada para { $datetime }
bot-schedule-cancelled = Agendamento cancelado.
bot-schedule-paused = Agendamento pausado.
bot-schedule-resumed = Agendamento retomado.

bot-monitor-alert = Alerta: { $subject } foi alterado
bot-monitor-threshold = { $metric } excedeu o limite: { $value }
bot-monitor-recovered = { $subject } voltou ao normal.
bot-monitor-status = Status atual: { $status }

bot-order-welcome = Bem-vindo à nossa loja! Como posso ajudar?
bot-order-track = Rastrear meu pedido
bot-order-browse = Ver produtos
bot-order-support = Falar com suporte
bot-order-enter-id = Por favor, digite o número do seu pedido:
bot-order-status = Status do pedido: { $status }
bot-order-shipped = Seu pedido foi enviado! Código de rastreamento: { $tracking }
bot-order-delivered = Seu pedido foi entregue.
bot-order-processing = Seu pedido está sendo processado.
bot-order-cancelled = Seu pedido foi cancelado.
bot-order-ticket = Ticket de suporte criado: #{ $ticket }
bot-order-products-available = Aqui estão nossos produtos disponíveis:
bot-order-product-item = { $name } - { $price }
bot-order-cart-added = { $product } adicionado ao seu carrinho.
bot-order-cart-total = O total do seu carrinho é { $total }.
bot-order-checkout = Prosseguindo para o pagamento...

bot-hr-welcome = Assistente de RH aqui. Como posso ajudar?
bot-hr-request-leave = Solicitar folga
bot-hr-check-balance = Consultar saldo
bot-hr-view-policies = Ver políticas
bot-hr-leave-type = Qual tipo de folga? (férias/médica/pessoal)
bot-hr-start-date = Data de início? (DD/MM/AAAA)
bot-hr-end-date = Data de término? (DD/MM/AAAA)
bot-hr-leave-submitted = Solicitação de folga enviada! Seu gestor irá revisar.
bot-hr-leave-approved = Sua solicitação de folga foi aprovada.
bot-hr-leave-rejected = Sua solicitação de folga foi rejeitada.
bot-hr-leave-pending = Sua solicitação de folga está pendente de aprovação.
bot-hr-balance-title = Seu saldo de folgas:
bot-hr-vacation-days = Férias: { $days } dias
bot-hr-sick-days = Licença médica: { $days } dias
bot-hr-personal-days = Pessoal: { $days } dias
bot-hr-policy-found = Aqui estão as informações da política solicitada:
bot-hr-policy-not-found = Política não encontrada. Por favor, verifique o nome da política.

bot-health-welcome = Bem-vindo ao nosso centro de saúde. Como posso ajudar?
bot-health-book = Agendar consulta
bot-health-cancel = Cancelar consulta
bot-health-view = Ver minhas consultas
bot-health-reschedule = Reagendar consulta
bot-health-type = Qual tipo de consulta? (clínica geral/especialista/laboratório)
bot-health-doctor = Qual médico você prefere?
bot-health-date = Qual data funciona melhor para você?
bot-health-time = Qual horário você prefere?
bot-health-confirmed = Sua consulta foi confirmada para { $datetime } com { $doctor }.
bot-health-cancelled = Sua consulta foi cancelada.
bot-health-rescheduled = Sua consulta foi reagendada para { $datetime }.
bot-health-reminder = Lembrete: Você tem uma consulta em { $datetime }.
bot-health-no-appointments = Você não tem consultas agendadas.
bot-health-appointments-list = Suas próximas consultas:

bot-support-welcome = Como posso ajudá-lo hoje?
bot-support-describe = Por favor, descreva seu problema:
bot-support-category = Qual categoria melhor descreve seu problema?
bot-support-priority = Qual a urgência deste problema?
bot-support-ticket-created = Ticket de suporte #{ $ticket } foi criado.
bot-support-ticket-status = Status do ticket #{ $ticket }: { $status }
bot-support-ticket-updated = Seu ticket foi atualizado.
bot-support-ticket-resolved = Seu ticket foi resolvido. Por favor, nos avise se precisar de mais ajuda.
bot-support-transfer = Transferindo você para um atendente humano...
bot-support-wait-time = Tempo estimado de espera: { $minutes } minutos.
bot-support-agent-joined = O atendente { $name } entrou na conversa.

bot-survey-intro = Adoraríamos ouvir sua opinião!
bot-survey-question = { $question }
bot-survey-scale = Em uma escala de 1 a 10, como você avalia { $subject }?
bot-survey-open = Por favor, compartilhe comentários adicionais:
bot-survey-thanks = Obrigado pelo seu feedback!
bot-survey-completed = Pesquisa concluída com sucesso.
bot-survey-skip = Você pode pular esta pergunta se preferir.

bot-notification-new-message = Você tem uma nova mensagem de { $sender }.
bot-notification-task-due = A tarefa "{ $task }" vence { $when }.
bot-notification-reminder = Lembrete: { $message }
bot-notification-update = Atualização: { $message }
bot-notification-alert = Alerta: { $message }

bot-command-help = Comandos disponíveis:
bot-command-unknown = Comando desconhecido. Digite 'ajuda' para ver os comandos disponíveis.
bot-command-invalid = Sintaxe de comando inválida. Uso: { $usage }

bot-transfer-to-human = Transferindo você para um atendente humano. Por favor, aguarde...
bot-transfer-complete = Você está agora conectado com { $agent }.
bot-transfer-unavailable = Nenhum atendente disponível no momento. Por favor, tente novamente mais tarde.
bot-transfer-queue-position = Você é o número { $position } na fila.

bot-auth-login-prompt = Por favor, insira suas credenciais para continuar.
bot-auth-login-success = Login realizado com sucesso.
bot-auth-login-failed = Falha no login. Por favor, verifique suas credenciais.
bot-auth-logout-success = Logout realizado com sucesso.
bot-auth-session-expired = Sua sessão expirou. Por favor, faça login novamente.

bot-file-upload-prompt = Por favor, envie seu arquivo.
bot-file-upload-success = Arquivo "{ $filename }" enviado com sucesso.
bot-file-upload-failed = Falha ao enviar arquivo. Por favor, tente novamente.
bot-file-download-ready = Seu arquivo está pronto para download.
bot-file-processing = Processando seu arquivo...

bot-payment-amount = O valor total é { $amount }.
bot-payment-method = Por favor, selecione um método de pagamento.
bot-payment-processing = Processando seu pagamento...
bot-payment-success = Pagamento realizado com sucesso! ID da transação: { $transactionId }
bot-payment-failed = Falha no pagamento. Por favor, tente novamente ou use outro método de pagamento.
bot-payment-refund = Seu reembolso de { $amount } foi processado.

bot-subscription-active = Sua assinatura está ativa até { $endDate }.
bot-subscription-expired = Sua assinatura expirou.
bot-subscription-renew = Você gostaria de renovar sua assinatura?
bot-subscription-upgraded = Sua assinatura foi atualizada para o plano { $plan }.
bot-subscription-cancelled = Sua assinatura foi cancelada.

bot-feedback-positive = Obrigado pelo seu feedback positivo!
bot-feedback-negative = Lamentamos saber disso. Como podemos melhorar?
bot-feedback-rating = Você avaliou esta interação com { $rating } de 5.

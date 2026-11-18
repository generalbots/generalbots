# Transactional email text (#2094). Subject and body only; MIME and envelope stay untouched.

email-greeting = Olá, { $name },
email-signoff = Atenciosamente,
email-team = Equipe General Bots
email-welcome-subject = Bem-vindo ao General Bots Cloud — plano { $plan }
email-welcome-body = Bem-vindo ao General Bots Cloud! Seu plano { $plan } está ativo. Você já pode criar bots e gerenciar seu workspace em { $url }
email-invoice-subject = Fatura #{ $invoice } — General Bots Cloud
email-invoice-body = Uma nova fatura foi criada. Fatura: { $invoice }. Valor: { $amount } { $currency }. Veja os detalhes em { $url }
email-payment-success-subject = Pagamento confirmado — General Bots Cloud
email-payment-success-body = Seu pagamento de { $amount } { $currency } foi confirmado. Plano: { $plan }. Obrigado por assinar!
email-subscription-activated-subject = Assinatura ativada — General Bots Cloud
email-subscription-activated-body = Sua assinatura { $plan } está ativa. Você pode gerenciá-la em { $url }
email-recurring-charge-subject = Cobrança recorrente — fatura #{ $invoice }
email-recurring-charge-body = Sua assinatura recorrente foi cobrada. Fatura: { $invoice }. Valor: { $amount } { $currency }. Veja os detalhes em { $url }
email-payment-failed-subject = Falha no pagamento — ação necessária
email-payment-failed-body = Seu pagamento de { $amount } { $currency } falhou. Fatura: { $invoice }. Atualize sua forma de pagamento em { $url }
email-subscription-cancelled-subject = Assinatura cancelada — General Bots Cloud
email-subscription-cancelled-body = Sua assinatura { $plan } foi cancelada. Se quiser reativá-la, acesse { $url }
email-grace-warning-subject = Pagamento em atraso — aviso de suspensão do serviço
email-grace-warning-body = Seu pagamento está em atraso. O serviço será suspenso em { $days } dias. Atualize sua forma de pagamento imediatamente em { $url }
email-invitation-subject = Convite para participar de { $org }
email-invitation-body = Você foi convidado a participar de { $org }. Aceite o convite em { $url }. Este convite expira em { $expires }.
email-quota-limit-reached = Você atingiu o limite da sua cota de { $metric }.
email-quota-threshold = Você usou { $current }% da sua cota de { $metric } (limite de alerta: { $threshold }%).
email-metric-messages = mensagens
email-metric-storage = armazenamento
email-metric-api-calls = chamadas de API
email-metric-bots = bots
email-metric-users = usuários
email-metric-kb-documents = documentos da base de conhecimento
email-metric-apps = aplicativos
email-metric-emails-per-day = e-mails por dia
email-metric-builds-per-day = builds por dia
email-metric-uploads-per-hour = envios por hora

# Sharing and mention notifications (#2094)
email-share-subject = { $sender } compartilhou "{ $file }" com você
email-share-download = Baixar { $file }
email-share-link-expires = Este link expira em { $date }. Você pode revogá-lo a qualquer momento.
email-share-link-no-expiry = Este link não expira. Você pode revogá-lo a qualquer momento.
email-mention-subject = { $author } mencionou você em um comentário
email-mention-body = { $author } mencionou você em { $resource_type } ({ $resource_id }). "{ $preview }" Abra o aplicativo para ver e responder.

# semantic i18n key (#2542)
mail-new-message = Nova mensagem

# semantic i18n key (#2542)
mail-cc-bcc = Cc/Cco

# semantic i18n key (#2542)
mail-subject-label = Assunto:

# semantic i18n key (#2542)
mail-track-opens = Rastrear aberturas

# semantic i18n key (#2542)
mail-request-read-receipt = Solicitar confirmação de leitura

# semantic i18n key (#2542)
mail-priority-normal = Prioridade normal

# semantic i18n key (#2542)
mail-priority-high = Alta prioridade

# semantic i18n key (#2542)
mail-priority-low = Baixa prioridade

# semantic i18n key (#2542)
mail-dialog-templates-title = Modelos de e-mail

# semantic i18n key (#2542)
mail-dialog-new-template = + Novo modelo

# semantic i18n key (#2542)
mail-dialog-signatures-title = Assinaturas de e-mail

# semantic i18n key (#2542)
mail-dialog-new-signature = + Nova assinatura

# semantic i18n key (#2542)
mail-dialog-rules-title = Regras de e-mail

# semantic i18n key (#2542)
mail-dialog-new-rule = + Nova regra

# semantic i18n key (#2542)
mail-dialog-automatic-replies-title = Respostas automáticas

# semantic i18n key (#2542)
mail-dialog-send-automatic-replies = Enviar respostas automáticas

# semantic i18n key (#2542)
mail-dialog-start-date = Data de início

# semantic i18n key (#2542)
mail-dialog-end-date = Data de término

# semantic i18n key (#2542)
mail-dialog-reply-only-org = Enviar apenas para pessoas da minha organização

# semantic i18n key (#2542)
mail-dialog-add-account-title = Adicionar conta de e-mail

# semantic i18n key (#2542)
mail-dialog-account-name = Nome da conta

# semantic i18n key (#2542)
mail-dialog-imap-server = Servidor IMAP

# semantic i18n key (#2542)
mail-dialog-imap-port = Porta IMAP

# semantic i18n key (#2542)
mail-dialog-smtp-server = Servidor SMTP

# semantic i18n key (#2542)
mail-dialog-smtp-port = Porta SMTP

# semantic i18n key (#2542)
mail-dialog-use-ssl-tls = Usar SSL/TLS

# semantic i18n key (#2542)
mail-dialog-add-account-action = Adicionar conta

# semantic i18n key (#2542)
mail-dialog-schedule-send-title = Agendar envio

# semantic i18n key (#2542)
mail-dialog-schedule-action = Agendar

# semantic i18n key (#2542)
mail-manage-labels-title = Gerenciar marcadores

# semantic i18n key (#2542)
mail-label-name = Nome do marcador

# semantic i18n key (#2542)
mail-label-color = Cor

# semantic i18n key (#2542)
mail-new-signature-title = Nova assinatura de e-mail

# semantic i18n key (#2542)
mail-signature-name = Nome da assinatura

# semantic i18n key (#2542)
mail-html-content = Conteúdo HTML

# semantic i18n key (#2542)
mail-new-rule-title = Nova regra de e-mail

# semantic i18n key (#2542)
mail-rule-name = Nome da regra

# semantic i18n key (#2542)
mail-condition-sender-contains = Condição (remetente contém)

# semantic i18n key (#2542)
mail-action-star = Marcar com estrela

# semantic i18n key (#2542)
mail-new-template-title = Novo modelo de e-mail

# semantic i18n key (#2542)
mail-template-name = Nome do modelo

# semantic i18n key (#2542)
mail-body-html = Corpo HTML

# semantic i18n key (#2542)
mail-placeholder-label-name = ex.: Trabalho
mail-placeholder-signature-name = ex.: Assinatura profissional
mail-placeholder-signature-content = Atenciosamente,<br><b>Seu nome</b>

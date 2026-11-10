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

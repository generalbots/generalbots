# Transactional email text (#2094). Subject and body only; MIME and envelope stay untouched.

email-greeting = Bonjour { $name },
email-signoff = Cordialement,
email-team = L'équipe General Bots
email-welcome-subject = Bienvenue dans General Bots Cloud — forfait { $plan }
email-welcome-body = Bienvenue dans General Bots Cloud ! Votre forfait { $plan } est maintenant actif. Vous pouvez commencer à créer des bots et à gérer votre espace de travail sur { $url }
email-invoice-subject = Facture n° { $invoice } — General Bots Cloud
email-invoice-body = Une nouvelle facture a été créée. Facture : { $invoice }. Montant : { $amount } { $currency }. Consultez les détails sur { $url }
email-payment-success-subject = Paiement confirmé — General Bots Cloud
email-payment-success-body = Votre paiement de { $amount } { $currency } a été confirmé. Forfait : { $plan }. Merci pour votre abonnement !
email-subscription-activated-subject = Abonnement activé — General Bots Cloud
email-subscription-activated-body = Votre abonnement { $plan } est maintenant actif. Vous pouvez le gérer sur { $url }
email-recurring-charge-subject = Prélèvement récurrent — facture n° { $invoice }
email-recurring-charge-body = Votre abonnement récurrent a été débité. Facture : { $invoice }. Montant : { $amount } { $currency }. Consultez les détails sur { $url }
email-payment-failed-subject = Paiement échoué — action requise
email-payment-failed-body = Votre paiement de { $amount } { $currency } a échoué. Facture : { $invoice }. Veuillez mettre à jour votre moyen de paiement sur { $url }
email-subscription-cancelled-subject = Abonnement résilié — General Bots Cloud
email-subscription-cancelled-body = Votre abonnement { $plan } a été résilié. Pour le réactiver, rendez-vous sur { $url }
email-grace-warning-subject = Paiement en retard — avertissement de suspension du service
email-grace-warning-body = Votre paiement est en retard. Votre service sera suspendu dans { $days } jours. Veuillez mettre à jour votre moyen de paiement immédiatement sur { $url }
email-invitation-subject = Invitation à rejoindre { $org }
email-invitation-body = Vous avez été invité à rejoindre { $org }. Acceptez votre invitation sur { $url }. Cette invitation expire le { $expires }.
email-quota-limit-reached = Vous avez atteint votre limite de quota de { $metric }.
email-quota-threshold = Vous avez utilisé { $current } % de votre quota de { $metric } (seuil : { $threshold } %).
email-metric-messages = messages
email-metric-storage = stockage
email-metric-api-calls = appels API
email-metric-bots = bots
email-metric-users = utilisateurs
email-metric-kb-documents = documents de la base de connaissances
email-metric-apps = applications
email-metric-emails-per-day = e-mails par jour
email-metric-builds-per-day = compilations par jour
email-metric-uploads-per-hour = téléversements par heure

# Sharing and mention notifications (#2094)
email-share-subject = { $sender } a partagé « { $file } » avec vous
email-share-download = Télécharger { $file }
email-share-link-expires = Ce lien expire le { $date }. Vous pouvez le révoquer à tout moment.
email-share-link-no-expiry = Ce lien n'expire pas. Vous pouvez le révoquer à tout moment.
email-mention-subject = { $author } vous a mentionné dans un commentaire
email-mention-body = { $author } vous a mentionné dans { $resource_type } ({ $resource_id }). « { $preview } » Ouvrez l'application pour consulter et répondre.

# semantic i18n key (#2542)
mail-new-message = Nouveau message

# semantic i18n key (#2542)
mail-cc-bcc = Cc/Cci

# semantic i18n key (#2542)
mail-subject-label = Objet :

# semantic i18n key (#2542)
mail-track-opens = Suivre les ouvertures

# semantic i18n key (#2542)
mail-request-read-receipt = Demander un accusé de lecture

# semantic i18n key (#2542)
mail-priority-normal = Priorité normale

# semantic i18n key (#2542)
mail-priority-high = Haute priorité

# semantic i18n key (#2542)
mail-priority-low = Basse priorité

# semantic i18n key (#2542)
mail-dialog-templates-title = Modèles d'e-mail

# semantic i18n key (#2542)
mail-dialog-new-template = + Nouveau modèle

# semantic i18n key (#2542)
mail-dialog-signatures-title = Signatures d'e-mail

# semantic i18n key (#2542)
mail-dialog-new-signature = + Nouvelle signature

# semantic i18n key (#2542)
mail-dialog-rules-title = Règles d'e-mail

# semantic i18n key (#2542)
mail-dialog-new-rule = + Nouvelle règle

# semantic i18n key (#2542)
mail-dialog-automatic-replies-title = Réponses automatiques

# semantic i18n key (#2542)
mail-dialog-send-automatic-replies = Envoyer des réponses automatiques

# semantic i18n key (#2542)
mail-dialog-start-date = Date de début

# semantic i18n key (#2542)
mail-dialog-end-date = Date de fin

# semantic i18n key (#2542)
mail-dialog-reply-only-org = Envoyer uniquement aux personnes de mon organisation

# semantic i18n key (#2542)
mail-dialog-add-account-title = Ajouter un compte de messagerie

# semantic i18n key (#2542)
mail-dialog-account-name = Nom du compte

# semantic i18n key (#2542)
mail-dialog-imap-server = Serveur IMAP

# semantic i18n key (#2542)
mail-dialog-imap-port = Port IMAP

# semantic i18n key (#2542)
mail-dialog-smtp-server = Serveur SMTP

# semantic i18n key (#2542)
mail-dialog-smtp-port = Port SMTP

# semantic i18n key (#2542)
mail-dialog-use-ssl-tls = Utiliser SSL/TLS

# semantic i18n key (#2542)
mail-dialog-add-account-action = Ajouter un compte

# semantic i18n key (#2542)
mail-dialog-schedule-send-title = Planifier l'envoi

# semantic i18n key (#2542)
mail-dialog-schedule-action = Planifier

# semantic i18n key (#2542)
mail-manage-labels-title = Gérer les libellés

# semantic i18n key (#2542)
mail-label-name = Nom du libellé

# semantic i18n key (#2542)
mail-label-color = Couleur

# semantic i18n key (#2542)
mail-new-signature-title = Nouvelle signature d'e-mail

# semantic i18n key (#2542)
mail-signature-name = Nom de la signature

# semantic i18n key (#2542)
mail-html-content = Contenu HTML

# semantic i18n key (#2542)
mail-new-rule-title = Nouvelle règle d'e-mail

# semantic i18n key (#2542)
mail-rule-name = Nom de la règle

# semantic i18n key (#2542)
mail-condition-sender-contains = Condition (l'expéditeur contient)

# semantic i18n key (#2542)
mail-action-star = Marquer d'une étoile

# semantic i18n key (#2542)
mail-new-template-title = Nouveau modèle d'e-mail

# semantic i18n key (#2542)
mail-template-name = Nom du modèle

# semantic i18n key (#2542)
mail-body-html = Corps HTML

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

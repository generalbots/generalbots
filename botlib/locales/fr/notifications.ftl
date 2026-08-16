notification-title-new-message = Nouveau message
notification-title-task-due = Tâche due
notification-title-task-assigned = Tâche assignée
notification-title-task-completed = Tâche terminée
notification-title-meeting-reminder = Rappel de réunion
notification-title-meeting-started = Réunion commencée
notification-title-file-shared = Fichier partagé
notification-title-file-uploaded = Fichier téléchargé
notification-title-comment-added = Nouveau commentaire
notification-title-mention = Vous avez été mentionné
notification-title-system = Notification système
notification-title-security = Alerte de sécurité
notification-title-update = Mise à jour disponible
notification-title-error = Une erreur s'est produite
notification-title-success = Succès
notification-title-warning = Avertissement
notification-title-info = Informations

notification-message-new = Vous avez un nouveau message de { $sender }
notification-message-unread = Vous avez { $count ->
    [one] { $count } unread message
   *[other] { $count } unread messages
}
notification-task-due-soon = La tâche "{ $task }" est due en { $time }
notification-task-due-today = La tâche "{ $task }" est due aujourd'hui
notification-task-due-overdue = La tâche "{ $task }" est en retard d'ici { $time }
notification-task-assigned-to-you = Vous avez été affecté à la tâche "{ $task }"
notification-task-assigned-by = { $assigner } vous a assigné à "{ $task }"
notification-task-completed-by = { $user } tâche terminée "{ $task }"
notification-task-status-changed = Le statut de la tâche "{ $task }" est passé à { $status }

notification-meeting-in-minutes = La réunion "{ $meeting }" commence dans { $minutes } minutes
notification-meeting-starting-now = La réunion "{ $meeting }" commence maintenant
notification-meeting-cancelled = La réunion "{ $meeting }" a été annulée
notification-meeting-rescheduled = La réunion "{ $meeting }" a été reportée à { $datetime }
notification-meeting-invite = { $inviter } vous a invité au "{ $meeting }"
notification-meeting-response = { $user } { $response } votre invitation à une réunion

notification-file-shared-with-you = { $sharer } a partagé "{ $filename }" avec vous
notification-file-uploaded-by = { $uploader } a téléchargé "{ $filename }"
notification-file-modified = "{ $filename }" a été modifié par { $user }
notification-file-deleted = "{ $filename }" a été supprimé le { $user }
notification-file-download-ready = Votre fichier "{ $filename }" est prêt à être téléchargé
notification-file-upload-complete = Le téléchargement de "{ $filename }" s'est terminé avec succès
notification-file-upload-failed = Échec du téléchargement de "{ $filename }"

notification-comment-on-task = { $user } a commenté la tâche "{ $task }"
notification-comment-on-file = { $user } a commenté "{ $filename }"
notification-comment-reply = { $user } a répondu à votre commentaire
notification-mention-in-comment = { $user } vous a mentionné dans un commentaire
notification-mention-in-chat = { $user } vous a mentionné dans { $channel }

notification-login-new-device = Nouvelle connexion détectée du { $device } au { $location }
notification-login-failed = Échec de la tentative de connexion à votre compte
notification-password-changed = Votre mot de passe a été modifié avec succès
notification-password-expiring = Votre mot de passe expirera dans { $days } jours
notification-session-expired = Votre session a expiré
notification-account-locked = Votre compte a été verrouillé
notification-two-factor-enabled = L'authentification à deux facteurs a été activée
notification-two-factor-disabled = L'authentification à deux facteurs a été désactivée

notification-subscription-expiring = Votre abonnement expire dans { $days } jours
notification-subscription-expired = Votre abonnement a expiré
notification-subscription-renewed = Votre abonnement a été renouvelé jusqu'au { $date }
notification-payment-successful = Le paiement de { $amount } a réussi
notification-payment-failed = Le paiement de { $amount } a échoué
notification-invoice-ready = Votre facture de { $period } est prête

notification-bot-response = { $bot } a répondu à votre requête
notification-bot-error = { $bot } a rencontré une erreur
notification-bot-offline = { $bot } est actuellement hors ligne
notification-bot-online = { $bot } est maintenant en ligne
notification-bot-updated = { $bot } a été mis à jour

notification-system-maintenance = Maintenance du système prévue pour le { $datetime }
notification-system-update = Mise à jour du système disponible : { $version }
notification-system-restored = Le système a été restauré
notification-system-degraded = Le système connaît des performances dégradées

notification-action-view = Voir
notification-action-dismiss = Rejeter
notification-action-mark-read = Marquer comme lu
notification-action-mark-all-read = Marquer tout comme lu
notification-action-settings = Paramètres de notification
notification-action-reply = Répondre
notification-action-open = Ouvert
notification-action-join = Rejoindre
notification-action-accept = Accepter
notification-action-decline = Refuser

notification-time-just-now = Juste maintenant
notification-time-minutes = { $compte ->
    [one] { $count } minute ago
   *[other] { $count } minutes ago
}
notification-time-hours = { $compte ->
    [one] { $count } hour ago
   *[other] { $count } hours ago
}
notification-time-days = { $compte ->
    [one] { $count } day ago
   *[other] { $count } days ago
}
notification-time-weeks = { $compte ->
    [one] { $count } week ago
   *[other] { $count } weeks ago
}

notification-preference-all = Toutes les notifications
notification-preference-important = Important seulement
notification-preference-none = Aucun
notification-preference-email = Notifications par courrier électronique
notification-preference-push = Notifications poussées
notification-preference-in-app = Notifications dans l'application
notification-preference-sound = Son activé
notification-preference-vibration = Vibration activée

notification-empty = Aucune notification
notification-empty-description = Vous êtes tous rattrapés !
notification-load-more = Charger plus
notification-clear-all = Effacer toutes les notifications
notification-filter-all = Tout
notification-filter-unread = Non lu
notification-filter-mentions = Mentionné
notification-filter-tasks = Tâches
notification-filter-messages = Messages
notification-filter-system = Système

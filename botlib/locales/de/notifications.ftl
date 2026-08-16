notification-title-new-message = Neue Nachricht
notification-title-task-due = Aufgabe fällig
notification-title-task-assigned = Aufgabe zugewiesen
notification-title-task-completed = Aufgabe abgeschlossen
notification-title-meeting-reminder = Besprechungserinnerung
notification-title-meeting-started = Besprechung hat begonnen
notification-title-file-shared = Dateifreigabe
notification-title-file-uploaded = Datei hochgeladen
notification-title-comment-added = Neuer Kommentar
notification-title-mention = Du wurdest erwähnt
notification-title-system = Systembenachrichtigung
notification-title-security = Sicherheitswarnung
notification-title-update = Update verfügbar
notification-title-error = Es ist ein Fehler aufgetreten
notification-title-success = Erfolg
notification-title-warning = Warnung
notification-title-info = Informationen

notification-message-new = Sie haben eine neue Nachricht von { $sender }
notification-message-unread = Sie haben { $count ->
    [one] { $count } unread message
   *[other] { $count } unread messages
}
notification-task-due-soon = Aufgabe „{ $task }“ ist in { $time } fällig
notification-task-due-today = Aufgabe „{ $task }“ ist heute fällig
notification-task-due-overdue = Aufgabe „{ $task }“ ist um { $time } überfällig
notification-task-assigned-to-you = Ihnen wurde die Aufgabe „{ $task }“ zugewiesen.
notification-task-assigned-by = { $assigner } hat dir „{ $task }“ zugewiesen
notification-task-completed-by = { $user } abgeschlossene Aufgabe „{ $task }“
notification-task-status-changed = Der Status der Aufgabe „{ $task }“ wurde in { $status } geändert.

notification-meeting-in-minutes = Besprechung „{ $meeting }“ beginnt in { $minutes } Minuten
notification-meeting-starting-now = Das Treffen „{ $meeting }“ beginnt jetzt
notification-meeting-cancelled = Treffen „{ $meeting }“ wurde abgesagt
notification-meeting-rescheduled = Treffen „{ $meeting }“ wurde auf { $datetime } verschoben
notification-meeting-invite = { $inviter } hat dich zu „{ $meeting }“ eingeladen
notification-meeting-response = { $user } { $response } Ihre Besprechungseinladung

notification-file-shared-with-you = { $sharer } hat „{ $filename }“ mit Ihnen geteilt
notification-file-uploaded-by = { $uploader } hat „{ $filename }“ hochgeladen
notification-file-modified = „{ $filename }“ wurde von { $user } geändert
notification-file-deleted = „{ $filename }“ wurde um { $user } gelöscht
notification-file-download-ready = Ihre Datei „{ $filename }“ steht zum Download bereit
notification-file-upload-complete = Der Upload von „{ $filename }“ wurde erfolgreich abgeschlossen
notification-file-upload-failed = Das Hochladen von „{ $filename }“ ist fehlgeschlagen

notification-comment-on-task = { $user } hat die Aufgabe „{ $task }“ kommentiert
notification-comment-on-file = { $user } kommentierte „{ $filename }“
notification-comment-reply = { $user } hat auf Ihren Kommentar geantwortet
notification-mention-in-comment = { $user } hat dich in einem Kommentar erwähnt
notification-mention-in-chat = { $user } hat dich in { $channel } erwähnt

notification-login-new-device = Neue Anmeldung von { $device } in { $location } erkannt
notification-login-failed = Fehlgeschlagener Anmeldeversuch für Ihr Konto
notification-password-changed = Ihr Passwort wurde erfolgreich geändert
notification-password-expiring = Ihr Passwort läuft in { $days } Tagen ab
notification-session-expired = Ihre Sitzung ist abgelaufen
notification-account-locked = Ihr Konto wurde gesperrt
notification-two-factor-enabled = Die Zwei-Faktor-Authentifizierung wurde aktiviert
notification-two-factor-disabled = Die Zwei-Faktor-Authentifizierung wurde deaktiviert

notification-subscription-expiring = Ihr Abonnement läuft in { $days } Tagen ab
notification-subscription-expired = Ihr Abonnement ist abgelaufen
notification-subscription-renewed = Ihr Abonnement wurde bis { $date } verlängert
notification-payment-successful = Die Zahlung von { $amount } war erfolgreich
notification-payment-failed = Die Zahlung von { $amount } ist fehlgeschlagen
notification-invoice-ready = Ihre Rechnung über { $period } ist fertig

notification-bot-response = { $bot } hat auf Ihre Anfrage geantwortet
notification-bot-error = { $bot } ist ein Fehler aufgetreten
notification-bot-offline = { $bot } ist derzeit offline
notification-bot-online = { $bot } ist jetzt online
notification-bot-updated = { $bot } wurde aktualisiert

notification-system-maintenance = Systemwartung geplant für { $datetime }
notification-system-update = Systemupdate verfügbar: { $version }
notification-system-restored = Das System wurde wiederhergestellt
notification-system-degraded = Die Leistung des Systems ist beeinträchtigt

notification-action-view = Ansicht
notification-action-dismiss = Entlassen
notification-action-mark-read = Als gelesen markieren
notification-action-mark-all-read = Alle als gelesen markieren
notification-action-settings = Benachrichtigungseinstellungen
notification-action-reply = Antwort
notification-action-open = Offen
notification-action-join = Machen Sie mit
notification-action-accept = Akzeptiere
notification-action-decline = Ablehnen

notification-time-just-now = Gerade eben
notification-time-minutes = { $count ->
    [one] { $count } minute ago
   *[other] { $count } minutes ago
}
notification-time-hours = { $count ->
    [one] { $count } hour ago
   *[other] { $count } hours ago
}
notification-time-days = { $count ->
    [one] { $count } day ago
   *[other] { $count } days ago
}
notification-time-weeks = { $count ->
    [one] { $count } week ago
   *[other] { $count } weeks ago
}

notification-preference-all = Alle Benachrichtigungen
notification-preference-important = Nur wichtig
notification-preference-none = Keine
notification-preference-email = E-Mail-Benachrichtigungen
notification-preference-push = Push-Benachrichtigungen
notification-preference-in-app = In-App-Benachrichtigungen
notification-preference-sound = Ton aktiviert
notification-preference-vibration = Vibration aktiviert

notification-empty = Keine Benachrichtigungen
notification-empty-description = Ihr seid alle beschäftigt!
notification-load-more = Mehr laden
notification-clear-all = Alle Benachrichtigungen löschen
notification-filter-all = Alle
notification-filter-unread = Ungelesen
notification-filter-mentions = Erwähnungen
notification-filter-tasks = Aufgaben
notification-filter-messages = Nachrichten
notification-filter-system = System

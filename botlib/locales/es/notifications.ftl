notification-title-new-message = Nuevo Mensaje
notification-title-task-due = Tarea Vence
notification-title-task-assigned = Tarea Asignada
notification-title-task-completed = Tarea Completada
notification-title-meeting-reminder = Recordatorio de Reunión
notification-title-meeting-started = Reunión Iniciada
notification-title-file-shared = Archivo Compartido
notification-title-file-uploaded = Archivo Subido
notification-title-comment-added = Nuevo Comentario
notification-title-mention = Te mencionaron
notification-title-system = Notificación del Sistema
notification-title-security = Alerta de Seguridad
notification-title-update = Actualización Disponible
notification-title-error = Ocurrió un Error
notification-title-success = Éxito
notification-title-warning = Advertencia
notification-title-info = Información

notification-message-new = Tienes un nuevo mensaje de { $sender }
notification-message-unread = Tienes { $count ->
    [one] { $count } mensaje sin leer
   *[other] { $count } mensajes sin leer
}
notification-task-due-soon = La tarea "{ $task }" vence en { $time }
notification-task-due-today = La tarea "{ $task }" vence hoy
notification-task-due-overdue = La tarea "{ $task }" está vencida por { $time }
notification-task-assigned-to-you = Te han asignado a la tarea "{ $task }"
notification-task-assigned-by = { $assigner } te asignó a "{ $task }"
notification-task-completed-by = { $user } completó la tarea "{ $task }"
notification-task-status-changed = El estado de la tarea "{ $task }" cambió a { $status }

notification-meeting-in-minutes = La reunión "{ $meeting }" comienza en { $minutes } minutos
notification-meeting-starting-now = La reunión "{ $meeting }" está comenzando ahora
notification-meeting-cancelled = La reunión "{ $meeting }" ha sido cancelada
notification-meeting-rescheduled = La reunión "{ $meeting }" ha sido reprogramada para { $datetime }
notification-meeting-invite = { $inviter } te invitó a "{ $meeting }"
notification-meeting-response = { $user } { $response } tu invitación a la reunión

notification-file-shared-with-you = { $sharer } compartió "{ $filename }" contigo
notification-file-uploaded-by = { $uploader } subió "{ $filename }"
notification-file-modified = "{ $filename }" fue modificado por { $user }
notification-file-deleted = "{ $filename }" fue eliminado por { $user }
notification-file-download-ready = Tu archivo "{ $filename }" está listo para descargar
notification-file-upload-complete = La subida de "{ $filename }" se completó exitosamente
notification-file-upload-failed = La subida de "{ $filename }" falló

notification-comment-on-task = { $user } comentó en la tarea "{ $task }"
notification-comment-on-file = { $user } comentó en "{ $filename }"
notification-comment-reply = { $user } respondió a tu comentario
notification-mention-in-comment = { $user } te mencionó en un comentario
notification-mention-in-chat = { $user } te mencionó en { $channel }

notification-login-new-device = Nuevo inicio de sesión detectado desde { $device } en { $location }
notification-login-failed = Intento de inicio de sesión fallido en tu cuenta
notification-password-changed = Tu contraseña fue cambiada exitosamente
notification-password-expiring = Tu contraseña expirará en { $days } días
notification-session-expired = Tu sesión ha expirado
notification-account-locked = Tu cuenta ha sido bloqueada
notification-two-factor-enabled = La autenticación de dos factores ha sido habilitada
notification-two-factor-disabled = La autenticación de dos factores ha sido deshabilitada

notification-subscription-expiring = Tu suscripción expira en { $days } días
notification-subscription-expired = Tu suscripción ha expirado
notification-subscription-renewed = Tu suscripción ha sido renovada hasta { $date }
notification-payment-successful = El pago de { $amount } fue exitoso
notification-payment-failed = El pago de { $amount } falló
notification-invoice-ready = Tu factura de { $period } está lista

notification-bot-response = { $bot } respondió a tu consulta
notification-bot-error = { $bot } encontró un error
notification-bot-offline = { $bot } está actualmente fuera de línea
notification-bot-online = { $bot } está ahora en línea
notification-bot-updated = { $bot } ha sido actualizado

notification-system-maintenance = Mantenimiento del sistema programado para { $datetime }
notification-system-update = Actualización del sistema disponible: { $version }
notification-system-restored = El sistema ha sido restaurado
notification-system-degraded = El sistema está experimentando rendimiento degradado

notification-action-view = Ver
notification-action-dismiss = Descartar
notification-action-mark-read = Marcar como leído
notification-action-mark-all-read = Marcar todo como leído
notification-action-settings = Configuración de notificaciones
notification-action-reply = Responder
notification-action-open = Abrir
notification-action-join = Unirse
notification-action-accept = Aceptar
notification-action-decline = Rechazar

notification-time-just-now = Ahora mismo
notification-time-minutes = { $count ->
    [one] hace { $count } minuto
   *[other] hace { $count } minutos
}
notification-time-hours = { $count ->
    [one] hace { $count } hora
   *[other] hace { $count } horas
}
notification-time-days = { $count ->
    [one] hace { $count } día
   *[other] hace { $count } días
}
notification-time-weeks = { $count ->
    [one] hace { $count } semana
   *[other] hace { $count } semanas
}

notification-preference-all = Todas las notificaciones
notification-preference-important = Solo importantes
notification-preference-none = Ninguna
notification-preference-email = Notificaciones por correo
notification-preference-push = Notificaciones push
notification-preference-in-app = Notificaciones en la app
notification-preference-sound = Sonido habilitado
notification-preference-vibration = Vibración habilitada

notification-empty = Sin notificaciones
notification-empty-description = ¡Estás al día!
notification-load-more = Cargar más
notification-clear-all = Limpiar todas las notificaciones
notification-filter-all = Todas
notification-filter-unread = Sin leer
notification-filter-mentions = Menciones
notification-filter-tasks = Tareas
notification-filter-messages = Mensajes
notification-filter-system = Sistema

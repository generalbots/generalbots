notification-title-new-message = Новое сообщение
notification-title-task-due = Задача
notification-title-task-assigned = Задача
notification-title-task-completed = Завершена
notification-title-meeting-reminder = Напоминание
notification-title-meeting-started = Начало встречи
notification-title-file-shared = Файл общий
notification-title-file-uploaded = Файл Загружен
notification-title-comment-added = Новое замечание
notification-title-mention = Тебя упоминали.
notification-title-system = Системное уведомление
notification-title-security = Внимание
notification-title-update = Обновление
notification-title-error = Ошибка
notification-title-success = Успех
notification-title-warning = Внимание
notification-title-info = Сведения

notification-message-new = У вас новое сообщение от{ $sender }
notification-message-unread = У тебя есть { $count ->
    [one] { $count }Нечитаемое сообщение
    *[other] { $count }Нечитаемые сообщения
    }- Я не знаю.
notification-task-due-soon = Задача "{ $task }"уже в{ $time }
notification-task-due-today = Задача "{ $task }" должна быть сегодня
notification-task-due-overdue = Задача "{ $task }"опоздало{ $time }
notification-task-assigned-to-you = Вы получили задание "{ $task }"
notification-task-assigned-by = { $assigner }Поручил тебе...{ $task }"
notification-task-completed-by = { $user }выполненная задача"{ $task }"
notification-task-status-changed = Задача "{ $task }"изменение статуса{ $status }

notification-meeting-in-minutes = Встреча "{ $meeting }"начало{ $minutes }минут
notification-meeting-starting-now = Начинается встреча "{ $meeting }".
notification-meeting-cancelled = Совещание "{ $meeting }" отменено
notification-meeting-rescheduled = Встреча "{ $meeting }"перенесено на{ $datetime }
notification-meeting-invite = { $inviter }Пригласил тебя..."{ $meeting }"
notification-meeting-response = { $user } { $response }приглашение на встречу

notification-file-shared-with-you = { $sharer }Разделяем".{ $filename }"с тобой
notification-file-uploaded-by = { $uploader }Загрузить "{ $filename }"
notification-file-modified = "{ $filename }"...было изменено на{ $user }
notification-file-deleted = "{ $filename }" был исключен{ $user }
notification-file-download-ready = Ваш файл "{ $filename }" готов для загрузки
notification-file-upload-complete = Успешно завершена загрузка "{ $filename }"
notification-file-upload-failed = Ошибка загрузки "{ $filename }"

notification-comment-on-task = { $user }, комментировал задачу "{ $task }"
notification-comment-on-file = { $user }, комментирует "{ $filename }"
notification-comment-reply = { $user }Ответ на ваш комментарий
notification-mention-in-comment = { $user }упомянул тебя в комментарии
notification-mention-in-chat = { $user }упомянул тебя в{ $channel }

notification-login-new-device = Новое входное отверстие, обнаруженное из{ $device }В{ $location }
notification-login-failed = Ошибка доступа к вашему аккаунту
notification-password-changed = Ваш пароль был успешно изменен
notification-password-expiring = Ваш пароль истекает в{ $days }дней
notification-session-expired = Ваш сеанс истек.
notification-account-locked = Твой счёт закрыт.
notification-two-factor-enabled = Версия двухфакторов включена
notification-two-factor-disabled = Отключено двухфакторное удостоверение

notification-subscription-expiring = Ваша подписка истекает в{ $days }дней
notification-subscription-expired = Твоя подписка истекла.
notification-subscription-renewed = Ваша подписка была продлена до тех пор, пока{ $date }
notification-payment-successful = Выплата{ $amount }был успешным
notification-payment-failed = Выплата{ $amount }Ошибка
notification-invoice-ready = Ваш счет за{ $period }готов.

notification-bot-response = { $bot }Ответ на ваш вопрос
notification-bot-error = { $bot }с ошибкой
notification-bot-offline = { $bot }в настоящее время отключено
notification-bot-online = { $bot }сейчас он в сети
notification-bot-updated = { $bot }обновлено

notification-system-maintenance = Техническое обслуживание системы запланировано на{ $datetime }
notification-system-update = Обновление системы:{ $version }
notification-system-restored = Система восстановлена
notification-system-degraded = Система испытывает ухудшение показателей работы

notification-action-view = Вид
notification-action-dismiss = Расторжение
notification-action-mark-read = Отметить как следует
notification-action-mark-all-read = Отметить все как читать
notification-action-settings = Настройка уведомления
notification-action-reply = Ответ
notification-action-open = Открыто
notification-action-join = Присоединяйтесь
notification-action-accept = Принять
notification-action-decline = Сокращение

notification-time-just-now = Только что
notification-time-minutes = - Я не знаю.{ $count ->
    [one] { $count }Минуту назад
    *[other] { $count }Несколько минут назад
    }- Я не знаю.
notification-time-hours = - Я не знаю.{ $count ->
    [one] { $count }час назад
    *[other] { $count }Час назад
    }- Я не знаю.
notification-time-days = - Я не знаю.{ $count ->
    [one] { $count }дня назад
    *[other] { $count }несколько дней назад
    }- Я не знаю.
notification-time-weeks = - Я не знаю.{ $count ->
    [one] { $count }Неделю назад
    *[other] { $count }Неделю назад
    }- Я не знаю.

notification-preference-all = Все уведомления
notification-preference-important = Только важное
notification-preference-none = Нет
notification-preference-email = Уведомления по электронной почте
notification-preference-push = Направить уведомления
notification-preference-in-app = Применяемые уведомления
notification-preference-sound = Звуковой прибор включен
notification-preference-vibration = Вибрация включена

notification-empty = Нет уведомлений
notification-empty-description = Ты все поймал!
notification-load-more = Загружай больше
notification-clear-all = Очистить все уведомления
notification-filter-all = Все
notification-filter-unread = Нечитаемый
notification-filter-mentions = Ссылки
notification-filter-tasks = Задачи
notification-filter-messages = Сообщения
notification-filter-system = Система

notification-digest-subject = Еженедельный сборник "General Bots"

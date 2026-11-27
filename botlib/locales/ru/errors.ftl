# General Bots - Error Messages (English)
# This file contains all error message translations

# =============================================================================
# HTTP Errors
# =============================================================================

error-http-400 = Проверьте данные.
error-http-401 = Требуется аутентификация.
error-http-403 = У вас нет разрешения на доступ к этому ресурсу.
error-http-404 = { $entity }Не найден.
error-http-409 = Конфликт:{ $message }
error-http-429 = Слишком много просьб. Пожалуйста, подождите.{ $seconds }секунды.
error-http-500 = Внутренняя ошибка сервера. Попробуйте позже.
error-http-502 = Неправильный вход. Сервер получил неверный ответ.
error-http-503 = Служба временно недоступна. Попробуйте позже.
error-http-504 = Запрос отсрочен после{ $milliseconds }Мс.

# =============================================================================
# Validation Errors
# =============================================================================

error-validation-required = { $field }требуется.
error-validation-email = Введите правильный адрес электронной почты.
error-validation-url = Введите действительный URL.
error-validation-phone = Пожалуйста, введите действительный номер телефона.
error-validation-min-length = { $field }должен быть по крайней мере{ $min }персонажи.
error-validation-max-length = { $field }должно быть не более{ $max }персонажи.
error-validation-min-value = { $field }должен быть по крайней мере{ $min }.
error-validation-max-value = { $field }должно быть не более{ $max }.
error-validation-pattern = { $field }Формат недействителен.
error-validation-unique = { $field }уже существует.
error-validation-mismatch = { $field }не совпадает{ $other }.
error-validation-date-format = Просьба указать действительные даты в формате{ $format }.
error-validation-date-past = { $field }Должно быть, в прошлом.
error-validation-date-future = { $field }должно быть в будущем.

# =============================================================================
# Authentication Errors
# =============================================================================

error-auth-invalid-credentials = Неверный адрес электронной почты или пароль.
error-auth-account-locked = Ваш счет закрыт. Пожалуйста, свяжитесь с поддержкой.
error-auth-account-disabled = Ваш счет был отключен.
error-auth-session-expired = Ваш сеанс истек. Пожалуйста, заходите еще раз.
error-auth-token-invalid = Неверный или просроченный символ.
error-auth-token-missing = Требуется аутентификация.
error-auth-mfa-required = Требуется многофакторная аутентификация.
error-auth-mfa-invalid = Неверный код проверки.
error-auth-password-weak = Пароль слишком слабый. Используйте пароль покрепче.
error-auth-password-expired = У вас истек пароль. Пожалуйста, перезагрузите его.

# =============================================================================
# Configuration Errors
# =============================================================================

error-config = Ошибка настройки:{ $message }
error-config-missing = Недостающая конфигурация:{ $key }
error-config-invalid = Неверное значение конфигурации для{ $key }::{ $reason }
error-config-file-not-found = Файл конфигурации не найден:{ $path }
error-config-parse = Не удалось определить конфигурацию петли:{ $message }

# =============================================================================
# Database Errors
# =============================================================================

error-database = Ошибка базы данных:{ $message }
error-database-connection = Не удалось подключиться к базе данных.
error-database-timeout = Работа с базой данных отсрочена.
error-database-constraint = Нарушение базы данных:{ $constraint }
error-database-duplicate = Это запись{ $field }уже существует.
error-database-migration = Не удалось осуществить перевод базы данных:{ $message }

# =============================================================================
# File & Storage Errors
# =============================================================================

error-file-not-found = Файл не найден:{ $filename }
error-file-too-large = Файл слишком большой. Максимальный размер{ $maxSize }.
error-file-type-not-allowed = Тип файла не допускается. Разрешенные типы:{ $allowedTypes }.
error-file-empty = Загруженный файл пуст.
error-file-name-invalid = Имя файла неправильно.
error-file-content-mismatch = Содержание файла не соответствует его типу ({ $expected }ожидаемый,{ $detected }Найдено).
error-file-archive-unsafe = Архиву было отказано:{ $reason }.
error-file-extension-blocked = Файлы с расширением .{ $extension } не допускаются.
error-file-extension-unknown = Расширение .{ $extension } не признается.
error-file-upload-failed = Ошибка загрузки файла:{ $message }
error-file-read = Не удалось прочитать файл:{ $message }
error-file-write = Не удалось записать файл:{ $message }
error-storage-full = Превышен квота хранения.
error-storage-unavailable = Хранение не обеспечивается.

# =============================================================================
# Network & External Service Errors
# =============================================================================

error-network = Ошибка сети:{ $message }
error-network-timeout = Связь отключилась.
error-network-unreachable = Сервер недоступен.
error-service-unavailable = Недоступное обслуживание:{ $service }
error-external-api = Внешние ошибки API:{ $message }
error-rate-limit = Ограниченный уровень.{ $seconds }С.

# =============================================================================
# Bot & Dialog Errors
# =============================================================================

error-bot-not-found = Бота не найдено:{ $botId }
error-bot-disabled = Этот бот сейчас отключен.
error-bot-script-error = Ошибка скрипта на строке{ $line }::{ $message }
error-bot-timeout = Ответ бота выключен.
error-bot-quota-exceeded = Превышен предел потребления бота.
error-dialog-not-found = Диалог не найден:{ $dialogId }
error-dialog-invalid = Неверная конфигурация диалога:{ $message }

# =============================================================================
# LLM & AI Errors
# =============================================================================

error-llm-unavailable = В настоящее время услуги ИИ недоступны.
error-llm-timeout = Запрос AI отсрочен.
error-llm-rate-limit = Предельная скорость ИИ превышена. Пожалуйста, подождите, прежде чем попробовать снова.
error-llm-content-filter = Содержание фильтруется в соответствии с руководящими принципами безопасности.
error-llm-context-length = Ввод слишком длинный. Пожалуйста, сократите ваше сообщение.
error-llm-invalid-response = Получил от службы АИ неверный ответ.
error-llm-empty-response = Простите, я не могу сейчас обработать ваше сообщение.

# =============================================================================
# Email Errors
# =============================================================================

error-email-send-failed = Не удалось отправить по электронной почте:{ $message }
error-email-invalid-recipient = Неверный адрес электронной почты получателя:{ $email }
error-email-attachment-failed = Не удалось вложить файл:{ $filename }
error-email-template-not-found = Не найден шаблон электронной почты:{ $template }

# =============================================================================
# Calendar & Scheduling Errors
# =============================================================================

error-calendar-conflict = Время не совпадает с существующим событием.
error-calendar-past-date = Невозможно планировать события в прошлом.
error-calendar-invalid-recurrence = Неверный характер повторения.
error-calendar-event-not-found = Не найденное событие:{ $eventId }

# =============================================================================
# Task Errors
# =============================================================================

error-task-not-found = Задача не найдена:{ $taskId }
error-task-already-completed = Задача уже выполнена.
error-task-circular-dependency = Обширная зависимость, выявленная при выполнении задач.
error-task-invalid-status = Неверный статус задачи.

# =============================================================================
# Permission Errors
# =============================================================================

error-permission-denied = У тебя нет разрешения на это.
error-permission-resource = У тебя нет доступа к этому.{ $resource }.
error-permission-action = Ты не можешь{ $action }это{ $resource }.
error-permission-owner-only = Только владелец может совершить это действие.

# =============================================================================
# Generic Errors
# =============================================================================

error-internal = Внутренняя ошибка:{ $message }
error-unexpected = Неожиданная ошибка. Попробуйте еще раз.
error-not-implemented = Эта функция еще не реализована.
error-maintenance = Система находится в эксплуатации. Попробуйте позже.
error-unknown = Неизвестная ошибка.

# =============================================================================
# Localized API contract (#2094) — stable codes, localized messages
# =============================================================================

error-locale-invalid = Запрошенный язык не является действительным.
error-locale-unsupported = Этот язык недоступен.
error-locale-persist-failed = Не смог сохранить предпочтение языка.
error-bot-id-required = Идентификатор бота требуется.
error-bot-id-invalid = Идентификатор бота недействителен.
error-bot-name-required = Имя бота требуется.
error-token-invalid-format = Символ недействителен.
error-token-signing-failed = Не удалось подписать знак.
error-token-signature-invalid = Символическая подпись недействительна.
error-token-payload-invalid = Символическая нагрузка недействительна.
error-token-payload-json-invalid = Символическая нагрузка не является действительной JSON.
error-cloud-token-required = Дайте облако.
error-llm-subscription-token-missing = Пропавший символ подписки.
error-llm-token-store-unavailable = Магазин знаков недоступен.
error-llm-subscription-token-expired = Срок действия символа подписки истек; отправьте сообщение снова.
error-llm-prompt-missing = В запросе нет никакой оперативной информации.
error-llm-account-unlinked = Сперва подключите свой аккаунт Клода или ЧатгПТ в настройках : IA.

# =============================================================================
# Localized API errors — routes, drive sync, peers, computer use (#2094)
# =============================================================================

error-agent-drive-database-unavailable = Файловый магазин временно недоступен.
error-agent-drive-lookup-failed = Не смог прочитать разговор. Попробуйте еще раз.
error-agent-drive-conversation-not-found = Не найдены разговоры.
error-agent-drive-bad-owner = Запись о владельце разговора недействительна.
error-agent-drive-forbidden = Этот разговор принадлежит другому пользователю.
error-agent-drive-bad-bot = Отчет о ботах для этого разговора недействителен.
error-agent-drive-workspace-stopped = Рабочее пространство агента не работает.
error-agent-drive-no-files = В папке вывода рабочего пространства нет файлов.
error-agent-drive-named-files-missing = Запрашиваемые файлы не в папке вывода рабочего пространства.
error-agent-drive-save-failed = Невозможно сохранить файл для управления.
error-agent-drive-file-missing = Файл не в папке вывода рабочего пространства.
error-agent-drive-save-one-failed = Невозможно сохранить файл для водителя.
error-agent-drive-bad-path = Дорога, по которой в магазине возвращается Драйв, недействительна.
error-agent-drive-link-failed = Невозможно создать ссылку на долю.
error-subagent-no-llm = Ни один поставщик LLM не имеет конфигурации для подагентов.
error-goal-run-not-running = Эта цель не работает.
error-goal-run-not-paused = Этот запуск не задерживается.
error-autotask-not-found = Автозадание не найдено.
error-autotask-intent-required = Опишите, что должно делаться.
error-peer-network-disabled = Сеть сверстников отключена на этом узле.
error-peer-network-unauthorized = Требуется действительный символ наследника.
error-peer-network-unconfigured = У этого узла нет символа сверстника, поэтому точка сверстника закрыта.
error-peer-network-forbidden = Для такого коллегиального контроля требуется сессия администратора.
error-peer-network-invalid = Запрос со стороны коллег не является действительным:{ $message }
error-peer-network-unknown-node = Неизвестный узел сверстников:{ $message }
error-peer-network-registry-full = Список коллег заполнен.
error-peer-network-payload-store-full = Магазин полезной нагрузки для сверстников заполнен:{ $message }
error-peer-network-recipient-required = Требуется параметр запроса получателя.
error-drive-sync-field-required = { $field }требуется.
error-drive-sync-field-absolute = { $field }Должно быть, это абсолютный путь.
error-drive-sync-field-parent = { $field }не должны содержать "..".
error-drive-sync-field-not-folder = { $field }не является папкой на этом механизме:{ $path }
error-drive-sync-field-local-folder = Местная папка
error-drive-sync-field-drive-folder = Папка Движок
error-drive-sync-job-not-found = Синхронизация работы{ $id }не нашли.
error-drive-sync-job-name-required = Требуется синхронное название работы.
error-drive-sync-job-exists = Синхронизация{ $name }уже существует.
error-drive-sync-conflict-path-required = Требуется путь к конфликту.
error-drive-sync-conflict-path-escape = Путь к конфликту убегает от синхронизации папки.
error-drive-sync-task-failed = - Это не так.{ $task }Ошибка задачи:{ $error }
error-org-auth-service-unavailable = Услуги по каталогу отсутствуют.
error-org-default-delete = Организация по умолчанию не может быть исключена.
error-org-settings-save-failed = Невозможно сохранить параметры организации.
error-org-name-required = Название организации требуется.
error-org-tenant-missing = Пока нет арендаторов, сначала займись миграцией или ботинками.
error-computer-use-session-required = Требуется сессионное обозначение.
error-computer-use-kind-invalid = Вид должен быть "разведчик" или "терминальный".
error-computer-use-action-not-found = Действия не найдены.
error-computer-use-no-consent = Нет никаких свидетельств согласия на это; сначала утверждайте.
error-computer-use-browser-only = Это действие должно быть сделано в панели браузера.
error-computer-use-url-missing = У действия нет URL.
error-terminal-approval-required = Для выполнения терминала требуется четкое одобрение пользователя.
error-agent-vm-disabled = Агент/ВИС отключен для этого разговора. Включите его в чат-контроль, чтобы использовать рабочее пространство.
error-terminal-action-id-required = Для выполнения терминала требуется утвержденный документ.
error-terminal-command-required = Требуется команда.
error-terminal-command-too-long = Команда ограничена 16000 символами.
error-terminal-blocked-pattern = Команда совпадает с заблокированной разрушительной схемой.
error-terminal-workspace-stopped = Рабочее пространство агента не работает. Заканчивайте снабжение (или перезапуск рабочего приложения), затем перепробуйте.
error-terminal-timeout = Команда отключилась.

# =============================================================================
# Collaboration, groups, autotasks and AI accounts (#2094)
# =============================================================================

error-db-unavailable = База данных сейчас недоступна. Попробуйте еще раз.
error-collab-invalid-resource = Вид ресурсов или идентификатор ресурсов недействителен.
error-collab-owner-only = Только владелец может это сделать.
error-collab-invalid-grantee-type = Тип грантополучателя должен быть "пользователем" или "доманом".
error-collab-invalid-role = Эта роль недействительна.
error-collab-invalid-grantee-id = Идентификатор получателя субсидии недействителен.
error-collab-grant-not-found = Грант не был найден.
error-collab-invalid-new-owner = Новый владелец недействителен.
error-collab-link-not-found = Связь с долей не была найдена.
error-collab-comment-not-found = Комментарий не был найден.
error-collab-comment-resolve-forbidden = Комментарий не был найден, иначе решать его не тебе.
error-collab-comment-body-required = Требуется орган для комментариев.
error-collab-comment-too-long = Комментарий ограничен 8000 символами.
error-collab-invalid-emoji = Эмоджи недействителен.
error-collab-invalid-cursor = Курс "до" недействителен.
error-collab-invalid-action = Это действие недействительно.
error-collab-invalid-name = Имя недействительно.
error-collab-content-required = Требуется содержание.
error-collab-content-too-large = Содержание слишком большое.
error-collab-version-not-found = Версия не была найдена.
error-collab-invalid-base-version = Базовая версия недействительна.
error-collab-invalid-parent = Идентификатор родителя недействителен.
error-collab-invalid-op-type = Тип операции недействителен.
error-collab-operation-not-found = Операция не была найдена.
error-collab-not-a-conflict = Операция не является неурегулированным конфликтом.
error-collab-invalid-resolution = Резолюция должна быть "приемлемым-сервером" или "приемным клиентом".
error-group-budget-not-found = Бюджет группы не был найден.
error-group-budget-invalid-id = Бюджетный идентификатор недействителен.
error-group-budget-amount-invalid = Сумма должна быть нулевой или позитивной.
error-group-budget-caps-invalid = Колпаки должны быть нулевыми или положительными (нулевой предел не должен превышать).
error-autotask-fields-required = Требуется название, расписание и командование.
error-autotask-kind-invalid = Наверное, "крон" или "система".
error-autotask-schedule-invalid = График должен быть правильным пятиполосным выражением.
error-conversation-not-found = Этот разговор не был найден.
error-conversation-session-invalid = Беседа недействительна.
error-subagent-invalid-tasks = Задачи подагентов недействительны.
error-ai-account-unknown-provider = Поставщик счетов АИ неизвестен.
error-ai-account-workspace-unavailable = В этом здании не имеется рабочего пространства агента.
error-ai-account-agent-invalid = Агент должен быть рассудительным, агентом или быстрым.
error-ai-account-not-connected-vibe = Подключите счет, прежде чем передать его Вибе.
error-ai-account-not-connected-chat = Подключите счет, прежде чем выбрать его для разговора.
error-computer-use-action-id-mismatch = Идентификатор действия не соответствует маршруту.
error-http-no-route = Ни один маршрут не совпадает с этим.
error-goal-run-advance-refused = Бег не может развиваться в своем нынешнем состоянии.

# =============================================================================
# People, attendance and products routes (#2094)
# =============================================================================

error-person-not-found = Человек не найден.
error-team-not-found = Команда не найдена.
error-time-off-not-found = Запрос на отсрочку не найден.
error-people-invalid-start-date = Дата начала недействительна. Используйте формат YYYY-MM-DD.
error-people-invalid-end-date = Дата окончания недействительна. Используйте формат YYYY-MM-DD.
error-attendance-webhook-not-found = Веб-хук не найден.
error-attendance-session-id-invalid = Идентификатор сеанса недействителен.
error-attendance-session-not-found = Сеанс не найден.
error-attendance-attendant-id-required = Требуется сопроводительное удостоверение.
attendance-no-attendant-matching-skills = Ни один из помощников не был найден с требуемыми навыками.
attendance-response-sent = Ответ отправлен.
error-product-not-found = Продукт не найден.
error-service-not-found = Обслуживание не найдено.
error-service-invalid-value = Стоимость услуг недействительна.
error-service-price-required = Эта услуга не имеет цены; она дает стоимость.

# Service-tax value validation (#2094)
error-invalid-value = Поставляемая стоимость не может быть скорректирована.
error-value-required = Для этой услуги требуется определенная стоимость.

# Language model errors shown in the chat
error-model-unreachable = Невозможно добраться до языковой модели. Проверьте связь и попробуйте снова.
error-model-auth = Языковая модель отклонила полномочия. Проверьте ключ в параметрах.
error-model-rate-limited = Языковая модель ограничивает запросы прямо сейчас.
error-model-error = Языковая модель вернула ошибку. Попробуйте снова.
error-model-timeout = Языковая модель слишком долго отвечала. Попробуй еще раз.
error-model-context = Разговор длиннее, чем модель может прочитать сразу.

# Agent workspace start before a subscription turn
workspace-step-starting-vm = Запуск виртуальной машины
workspace-step-starting-vm-detail = Первое начало может занять несколько минут.
error-workspace-start-timeout = Виртуальная машина не запустилась вовремя. Она продолжает работать на фоне, попробуйте еще раз через несколько минут.
error-workspace-start-failed = Виртуальная машина не могла быть запущена. Попробуйте снова; если она продолжает не работать, проверьте виртуальную машину в параметрах.
error-workspace-service-restarting = Перезапуск службы Виртуальной Машины...
error-workspace-service-failed = Служба Виртуальной Машины не ответила. Перезапуск компьютера или WSL.

# Chat attachments that could not be kept in Drive
error-attachment-storage-full = Диск этого компьютера заполнен: файл не был сохранен. Свободное пространство и прикрепите его снова.
error-attachment-not-stored = Файл не удалось сохранить в Драйве.
error-attachment-too-large = Файл превышает предельное значение 50 МБ.
error-attachment-not-read = Файл не может храниться или считываться.

# Chat answers given without the model
error-memory-not-updated = Я не смог обновить твою память. Попробуй еще раз.
memory-forgot-none = Ничего в твоей памяти не совпадало с этим, так что ничего не было удалено.
memory-forgot-one = Я убрал 1 вещь из вашей памяти.
memory-forgot-many = Сделано: я удалена{ $count }- Из твоей памяти.
error-command-failed = Я не смог завершить это действие. Попробуй снова или спроси другими словами.
error-tool-invalid-name = Неверное имя инструмента:{ $name }
error-tool-not-available = Инструмент Δ{ $name }Δ здесь нет.

# Drive listing in the chat (the model reads its own English copy)
chat-drive-unavailable = В этом рабочем пространстве нет двигателя.
chat-drive-list-failed = Невозможно перечислить файлы в вашем Драйве.
chat-drive-list-empty = В твоем Драйве нет файлов.
chat-drive-list-empty-query = Нет файлов, соответствующих β{ $query }▪ были найдены в вашем Драйв.
chat-drive-list-heading = Файлы в вашем драйвере (сперва -{ $limit }):
chat-drive-list-heading-query = Файлы в вашем драйвере совпадают Δ{ $query }▪ (новый, до{ $limit }):
chat-drive-open = Открыть в движении

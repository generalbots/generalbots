# =============================================================================
# General Bots — en — Productivity apps (tasks, calendar, meet, mail, research, paper)
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Tasks
# -----------------------------------------------------------------------------
tasks-title = Задачи
tasks-new = Новая задача
tasks-due-today = На сегодня
tasks-overdue = Просроченные
tasks-completed = Завершено
tasks-all = Все задачи
tasks-priority-high = Высокая степень приоритетности
tasks-priority-medium = Средний приоритет
tasks-priority-low = Низкий приоритет
tasks-no-due-date = Дата не должна быть установлена
tasks-add-subtask = Добавить подзадачу
tasks-mark-complete = Отметить как полный
tasks-mark-incomplete = Отметить как неполное
tasks-delete-confirm = Вы уверены, что хотите удалить это задание?
tasks-count = - Я не знаю.{ $count ->
    [zero] Нет задач
    [one] { $count }задача
    *[other] { $count }задачи
    }- Я не знаю.

# -----------------------------------------------------------------------------
# Calendar
# -----------------------------------------------------------------------------
calendar-title = Календарь
calendar-today = Сегодня
calendar-new-event = Новое событие
calendar-all-day = Весь день
calendar-repeat = Повторить
calendar-reminder = Напоминание
calendar-view-day = День
calendar-view-week = Неделя
calendar-view-month = Месяц
calendar-view-year = Год
calendar-no-events = Никаких запланированных мероприятий не запланировано
calendar-event-title = Название мероприятия
calendar-event-location = Место проведения
calendar-event-description = Описание
calendar-event-attendees = Участники

# -----------------------------------------------------------------------------
# Meet / Video Conferencing
# -----------------------------------------------------------------------------
meet-title = Встречайте
meet-join = Присоединяйтесь к совещанию
meet-start = Начать совещание
meet-mute = Выключить
meet-unmute = Немного
meet-video-on = Камера включена
meet-video-off = Камера отключена
meet-share-screen = Показать экран
meet-stop-sharing = Прекратить делиться
meet-end-call = Конец вызова
meet-leave = Отправиться на совещание
meet-participants = - Я не знаю.{ $count ->
    [one] { $count }участник
    *[other] { $count }Участники
    }- Я не знаю.
meet-waiting-room = Ждущая комната
meet-admit = Признайся
meet-remove = Удалить
meet-chat = Встреча с болтанием
meet-raise-hand = Поднять руку
meet-lower-hand = Рука
meet-recording = Регистрация
meet-start-recording = Запуск записи
meet-stop-recording = Остановить запись

# -----------------------------------------------------------------------------
# Mail / Email
# -----------------------------------------------------------------------------
mail-title = Почта
mail-compose = Состав
mail-inbox = В ящик
mail-sent = Отправлено
mail-drafts = Проекты
mail-trash = Мусор
mail-spam = Спам
mail-starred = Звездный
mail-archive = Архив
mail-to = К
mail-cc = КК
mail-bcc = БКК
mail-subject = Тема
mail-body = Сообщение
mail-reply = Ответ
mail-reply-all = Ответ
mail-forward = Вперед
mail-send = Отправить
mail-discard = Отключить
mail-save-draft = Сохранить
mail-attach = Вставить файлы
mail-unread = - Я не знаю.{ $count ->
    [one] { $count }нечитаемый
    *[other] { $count }нечитаемый
    }- Я не знаю.
mail-empty-inbox = Твоя коробка пуста.
mail-no-subject = (Никаких предметов)

# -----------------------------------------------------------------------------
# Paper (Document Editor)
# -----------------------------------------------------------------------------
paper-title = Бумага
paper-new-note = Новое примечание
paper-search-notes = Заметки для поиска...
paper-quick-start = Быстрый запуск
paper-template-blank = Пробел
paper-template-meeting = Совещание
paper-template-todo = Задача
paper-template-research = Исследования
paper-untitled = Неимоверно
paper-placeholder = Начинайте писать или печатать / для команд...
paper-commands = Команды
paper-heading1 = Раздел 1
paper-heading1-desc = Заголовок большого раздела
paper-heading2 = Раздел 2
paper-heading2-desc = Название среднего участка
paper-heading3 = Раздел 3
paper-heading3-desc = Название небольшого раздела
paper-paragraph = Пункт
paper-paragraph-desc = Простой текст
paper-bullet-list = Список пулевых ранений
paper-bullet-list-desc = Незаказный список
paper-numbered-list = Номер списка
paper-numbered-list-desc = Заказный список
paper-todo-list = Список задач
paper-todo-list-desc = Поддающийся проверке список задач
paper-quote = Квадрат
paper-quote-desc = Блок-нотка для ссылок
paper-divider = Разделитель
paper-divider-desc = Горизонтальная линия
paper-code-block = Кодовый блок
paper-code-block-desc = Форматированный код
paper-table = Таблица
paper-table-desc = Вставить таблицу
paper-image = Фото
paper-image-desc = Вставить изображение из URL
paper-callout = Вызов
paper-callout-desc = Подчеркнутая информационная ячейка
paper-ai-write = IA Написание
paper-ai-write-desc = Создать текст с АИ
paper-ai-summarize = AI Обобщать
paper-ai-summarize-desc = Обобщение выбранного текста
paper-ai-expand = AI Расширить
paper-ai-expand-desc = Расширить выбранный текст
paper-ai-improve = IA Улучшить
paper-ai-improve-desc = Повышение качества письменной формы
paper-ai-translate = IA Перевод
paper-ai-translate-desc = Перевод на другой язык
paper-ai-assistant = Помощник по вопросам МА
paper-ai-quick-actions = Быстрые действия
paper-ai-rewrite = Переписать
paper-ai-make-shorter = Сделать короче
paper-ai-make-longer = Сделай дольше
paper-ai-fix-grammar = Исправь Граммар
paper-ai-tone = Тон
paper-ai-tone-professional = Категория специалистов
paper-ai-tone-casual = Случайность
paper-ai-tone-friendly = Дружелюбный
paper-ai-tone-formal = Формальное
paper-ai-translate-to = Перевод на
paper-ai-custom-prompt = Незамедлительное
paper-ai-custom-placeholder = Опишите, чего вы хотите...
paper-ai-generate = Генерировать
paper-ai-response = Ответ МА
paper-ai-apply = Применить
paper-ai-regenerate = Режиссировать
paper-ai-copy = Копировать
paper-word-count = { $count }слова
paper-char-count = { $count }символы
paper-saved = Спасён
paper-saving = Спасение...
paper-last-edited = Последний раз редактировался:{ $time }
paper-last-edited-now = Последний раз редактировался:
paper-export = Экспортный документ
paper-export-pdf = PDF
paper-export-docx = Слово (.docx)
paper-export-markdown = Отметка
paper-export-html = HTML
paper-export-txt = Простой текст

# Tasks translations
tasks-active = Активные намерения
tasks-awaiting = Ожидает решения
tasks-paused = Приостановлено
tasks-blocked = Блокированные/проблемные
tasks-time-saved = Сохранено активное время:
tasks-input-placeholder = Что бы вы хотели сделать? Например, создать приложение для CRM или "позвонить Джону завтра"

# Calendar additional translations
calendar-my-calendars = Мои календари

# Email additional translations
email-scheduled = Запланировано
email-tracking = Отслеживание

# Email folder translations
email-inbox = В ящик
email-starred = Звездный
email-sent = Отправлено
email-drafts = Проекты
email-spam = Спам
email-trash = Мусор
email-compose = Состав

# -----------------------------------------------------------------------------
# Research
# -----------------------------------------------------------------------------
research-title = Исследования
research-search-placeholder = Спроси что-нибудь...
research-collections = Сбор средств
research-new-collection = Новая коллекция
research-recent = Последние
research-academic = Образование
research-code = Код
research-internal = Внутренний
research-search-all = Обыщите все
research-academic-papers = Научные работы
research-code-docs = & Кодовая документация
research-internal-kb = Внутренняя база знаний
research-sources = Источники
research-trending = Изменение
research-pro-search = Про поиск
research-include-images = Включить изображения
research-try-asking = Попробуй спросить о
research-related = и смежным вопросам
research-view-all-sources = Вид всех источников
research-export-citations = Экспортные цитаты
research-save-to-collection = Сохранить в коллекцию

# -----------------------------------------------------------------------------
# Meet (additional keys)
# -----------------------------------------------------------------------------
meet-new-meeting = Новое совещание
meet-join-meeting = Присоединяйтесь к совещанию
meet-active-rooms = Активные комнаты
meet-room-title = Зал заседаний
meet-record = Запись
meet-camera = Камера
meet-share = Доля
meet-info = Сведения
meet-more = Больше
meet-share-meeting = Собирайтесь
meet-meeting-title = Название совещания
meet-meeting-code = Код проведения совещания
meet-meeting-link = Ссылка на встречу
meet-send-invite = Отправить приглашение

# ── #2307 Tasks goal/monitor detail panel ──────────────────────────
tasks-detail-untitled-goal = Цель
tasks-detail-goal-progress = Прогресс в достижении целей
tasks-detail-current-value = Текущая стоимость
tasks-detail-target-value = Целевая стоимость
tasks-detail-percent = Ход работы
tasks-detail-no-actions = Пока никаких действий не принято
tasks-detail-pending-info = Недостающая информация
tasks-detail-scheduler-info = Информация о графике
tasks-detail-monitor-info = Контрольная информация
tasks-detail-unavailable = Недоступно
tasks-detail-load-failed = Невозможно загрузить детали цели


# ── #2307 Database insert/confirm dialogs ──────────────────────────
db-insert-row = Вставить строчку
db-insert-into = Вставить
db-provide-value = По меньшей мере одно значение
db-insert-failed = Ошибка вставки
db-confirm-title = Подтвердить
db-confirm-accept = Подтвердить
db-confirm-delete = Удалить этот ряд?
db-confirm-drop = Сбросить стол
db-delete-failed = Удалить не удалось
db-drop-failed = Ошибка

# Tasks detail view (#2307)
tasks-detail-type-goal = Цель
tasks-detail-status-pending = Нерассмотренные
tasks-detail-pending-title = Пункты, по которым еще не принято решение
tasks-detail-required-reason = Причина нужна.
tasks-detail-type-scheduler = График
tasks-detail-type-monitor = Монитор
tasks-detail-last-action = Последнее действие
tasks-detail-next-run = Следующий запуск
tasks-detail-file = Файл
tasks-detail-interval = Интервал
tasks-detail-last-check = Последний чек
tasks-detail-last-value = Последнее значение
tasks-detail-progress = Ход работы
tasks-detail-app = Прием
tasks-detail-field = Полевая
tasks-detail-reason = Обоснование
tasks-detail-cron = Крон

# Database dialog copy (#2307)
db-error = Ошибка
db-database = База данных
db-empty-query = Выполнить запрос, чтобы увидеть результаты
db-insert = Вставить
db-select-table = Выбрать таблицу
tasks-load-error = Невозможно загрузить задания.
tasks-sign-in-to-load = Подпишитесь на загрузку.
calendar-collapse-sidebar = Сбоку от сцепления
calendar-prev-month = Предыдущий месяц
calendar-next-month = В следующем месяце
calendar-prev-period = Предыдущий период
calendar-next-period = Следующий период
calendar-add-calendar = Добавить календарь
paper-toggle-sidebar = Боковой блок@ info: whatsthis
paper-insert-ai-response = Вставить ответ АИ
paper-replace-selection = Заменить выбор
paper-highlight = Подсветка
paper-align-left = Слева
paper-align-center = Центр аккорда
paper-align-right = Присоединить право
paper-insert-link = Вставить ссылку
paper-insert-image = Вставить изображение
paper-insert-table = Вставить таблицу
paper-bold = Крепкое
paper-italic = курсив
paper-underline = Подчёркиваем
paper-strikethrough = Зачеркивание
products-view-grid = Вид сетки
products-view-list = Вид списка
sheet-freeze-panes = Замерзающие стекла

# semantic i18n key (#2542)
db-schema-title = ▪ Схема базы данных

# semantic i18n key (#2542)
db-loading-tables = Загрузка таблиц...

# semantic i18n key (#2542)
db-query-builder = Строитель запроса

# semantic i18n key (#2542)
db-run-query = Запустить запрос

# semantic i18n key (#2542)
db-select-table-hint = Выберите таблицу для просмотра данных

# semantic i18n key (#2542)
db-prev-page = ▪ Превзятая

# semantic i18n key (#2542)
db-page-info-default = Стр. 1 из 1

# semantic i18n key (#2542)
db-next-page = Следующий

# semantic i18n key (#2542)
research-searching-sources = Поиск источников...

# semantic i18n key (#2542)
research-suggestion-latest-ai = Последние изменения в области МА

# semantic i18n key (#2542)
research-suggestion-quantum = Объяснение квантовой вычислительной системы

# semantic i18n key (#2542)
research-suggestion-software-architecture = Архитектура программного обеспечения

# semantic i18n key (#2542)
research-suggestion-ai-productivity = Инструменты производительности АИ

# semantic i18n key (#2542)
research-related-heading = См.

# semantic i18n key (#2542)
research-loading = Загрузка исследований...

# semantic i18n key (#2542)
research-service-unavailable = Исследовательная служба отсутствует

# semantic i18n key (#2542)
calendar-mini-month-placeholder = Январь 2025 года

# semantic i18n key (#2542)
calendar-upcoming = Предстоящие

# semantic i18n key (#2542)
calendar-import-export = Импорт/экспорт

# semantic i18n key (#2542)
calendar-popup-event-title = Название мероприятия

# semantic i18n key (#2542)
calendar-popup-location-placeholder = Зал заседаний A

# semantic i18n key (#2542)
calendar-popup-event-description = Описание события

# semantic i18n key (#2542)
calendar-personal = Личные сведения

# semantic i18n key (#2542)
calendar-import-ical-title = Импортировать файл iCal

# semantic i18n key (#2542)
calendar-select-ics-file = Выбрать файл .ics

# semantic i18n key (#2542)
calendar-import-to-calendar-label = Импортировать в календарь

# semantic i18n key (#2542)
calendar-default-calendar = Календарь

# semantic i18n key (#2542)
calendar-loading = Загрузка календаря...

# semantic i18n key (#2542)
calendar-unavailable = Календарь отсутствует

# semantic i18n key (#2542)
tasks-progress-status = Статус

# semantic i18n key (#2542)
tasks-progress-sample-title = Внедрение системы аутентизации пользователей

# semantic i18n key (#2542)
tasks-progress-runtime = Время запуска:

# semantic i18n key (#2542)
tasks-progress-sample-action = Выберите стратегию &quot; Ток-Исчерпание &quot;

# semantic i18n key (#2542)
tasks-progress-estimated = Расчетный показатель:

# semantic i18n key (#2542)
tasks-progress-sample-decision = Потребуется разрешение на установку системы безопасности

# semantic i18n key (#2542)
tasks-progress-log = ХОД РАБОТЫ

# semantic i18n key (#2542)
tasks-progress-terminal-live = ТЕРМИНОЛОГИЯ (ЖИЛАЯ ДЕЯТЕЛЬНОСТЬ ПО СЛУЧАЮ АГЕНТСТВА)

# semantic i18n key (#2542)
tasks-progress-processed = Обработано:

# semantic i18n key (#2542)
tasks-progress-data-points = данные

# semantic i18n key (#2542)
tasks-progress-sample-speed = ~8 источников/мин.

# semantic i18n key (#2542)
tasks-progress-sample-insight = Объединение конкурентных ощущений...

# semantic i18n key (#2542)
tasks-progress-sample-pricing = Перекрестные ссылки на данные о ценах...

# semantic i18n key (#2542)
tasks-progress-view-details = Вид Подробности β

# semantic i18n key (#2542)
tasks-create-run = Создать & запуск

# semantic i18n key (#2542)
tasks-loading-tasks = Загрузка задач...

# semantic i18n key (#2542)
tasks-select-task = Выберите задачу

# semantic i18n key (#2542)
tasks-select-task-desc = Нажмите на задачу из списка, чтобы увидеть подробности

# semantic i18n key (#2542)
tasks-bot-database-label = База данных бота:

# semantic i18n key (#2542)
tasks-shared-resources-label = Общие ресурсы:

# semantic i18n key (#2542)
tasks-starting = Начинаем...

# semantic i18n key (#2542)
tasks-create-new-intent = Создать новый замысел

# semantic i18n key (#2542)
tasks-intent-accomplish-label = Чего бы вы хотели добиться?

# semantic i18n key (#2542)
tasks-execution-mode = Режим выполнения

# semantic i18n key (#2542)
tasks-mode-automatic = Автоматическое

# semantic i18n key (#2542)
tasks-mode-supervised = Контролируемый

# semantic i18n key (#2542)
tasks-mode-manual-approval = Утверждение вручную

# semantic i18n key (#2542)
tasks-make-decision = Принимать решения

# semantic i18n key (#2542)
tasks-decision-required = Требуемое решение

# semantic i18n key (#2542)
tasks-loading-decision-details = Загрузка деталей решения...

# semantic i18n key (#2542)
tasks-skip-for-now = Пропустите сейчас

# semantic i18n key (#2542)
tasks-confirm-decision = Подтвердить решение

# semantic i18n key (#2542)
tasks-due-soon = Вскоре

# semantic i18n key (#2542)
tasks-autotask-items = ПОЛОЖЕНИЕ В АВТОТАСКОМ НОМЕРЕ

# semantic i18n key (#2542)
tasks-edit-task = Правка Задача

# semantic i18n key (#2542)
tasks-assignee-user-id = Назначенный (идентификационный номер пользователя)

# semantic i18n key (#2542)
tasks-parent-task-subtask-of = Задача родителей (подзадача...)

# semantic i18n key (#2542)
tasks-reopen-action = ▪ Повторное открытие

# semantic i18n key (#2542)
tasks-complete-action = Завершено

# semantic i18n key (#2542)
tasks-delete-action = ▸ Удалить

# semantic i18n key (#2542)
db-schema-browser = Схема Броузер

# semantic i18n key (#2542)
db-loading-schema = Загрузка схемы...

# semantic i18n key (#2542)
db-new-table = + Новая таблица

# semantic i18n key (#2542)
db-previous = ▪ Предыдущая

# semantic i18n key (#2542)
db-next = Следующий

# semantic i18n key (#2542)
db-sql-console = СВЛ Консоле

# semantic i18n key (#2542)
db-execute-query = Выполнить (Ctrl+Inter)

# semantic i18n key (#2542)
db-explain = ПОЯСНИТЕЛЬНЫЕ ЗАМЕЧАНИЯ

# semantic i18n key (#2542)
db-create-new-table = Создать новую таблицу

# semantic i18n key (#2542)
db-table-name = Название таблицы

# semantic i18n key (#2542)
db-columns = Колонки

# semantic i18n key (#2542)
db-not-null = Не совсем

# semantic i18n key (#2542)
db-type-varchar-255 = VARCHAR (255)

# semantic i18n key (#2542)
db-type-text = ТЕКСТ

# semantic i18n key (#2542)
db-type-integer = ИНТЕГР

# semantic i18n key (#2542)
db-type-bigint = Взрыв

# semantic i18n key (#2542)
db-type-decimal = ДЕКЛАРАЦИЯ

# semantic i18n key (#2542)
db-type-boolean = БОЛЕЕ

# semantic i18n key (#2542)
db-type-date = Дата

# semantic i18n key (#2542)
db-type-timestamptz = ТИМЕСТАМПТЦ

# semantic i18n key (#2542)
db-type-jsonb = СПОНБ

# semantic i18n key (#2542)
db-add-column = + Добавить колонку

# semantic i18n key (#2542)
db-create-table = Создать таблицу

# semantic i18n key (#2542)
db-edit-row = Правка Строка

# semantic i18n key (#2542)
db-import-csv = Импорт CSV

# semantic i18n key (#2542)
db-select-csv-file = Выбрать файл CSV

# semantic i18n key (#2542)
analytics-ask-placeholder = Спросите о ваших аналитических данных...
tasks-intent-description-placeholder = Опишите, что вы хотите создать или автоматизировать...
tasks-assignee-placeholder = УВНУХ цессионария (факультативно)
tasks-parent-placeholder = НУИД родительской задачи (факультативно)

# semantic i18n key (#2542)
db-placeholder-search-tables = Поисковые столы...
db-aria-close-sql-console = Закрыть SQL консоль
db-placeholder-sql-query = СЛЕКТРИЯ * Из таблицы_ имени Где состояние...
db-aria-sql-query = Запрос SQL
db-placeholder-table-name-example = Мой_таблиец
db-placeholder-column-id = Идентификатор
db-placeholder-column-name-example = колонка_ имя

# semantic i18n key (#2542)
calendar-event-attendees-placeholder = электронная почта1, электронная почта2

# semantic i18n key (#2542)
db-sql-editor-placeholder = СЛЕЧЕНИЕ * Из стола...
db-sql-editor-aria-label = Запрос SQL

# Chat header / app chrome (#2542)
paper-save-failed = Не спасено - повторное испытание
paper-save-retry-hint = Нажмите на повторное сохранение
paper-saved-locally = Сохранить на этом устройстве
paper-status-bar = Статус документа
tasks-detail-region = Подробности задач
tasks-filters = Целевые фильтры
tasks-progress-bar = Панель хода выполнения задач
tasks-progress-close = Закрытая работа
tasks-progress-minimize = Сведение к минимуму прогресса
tasks-progress-output = Целевые результаты
tasks-progress-region = Ход выполнения задачи
tasks-terminal-log = Деятельность живых агентов

# -----------------------------------------------------------------------------
# #2328 — loading / empty / error state copy
# -----------------------------------------------------------------------------
tasks-state-empty-title = Пока никаких задач
tasks-state-empty-desc = Опишите, что вам нужно сделать в ящике выше, и помощник превращает его в задание.
tasks-state-empty-action = Начать с намерения
calendar-state-empty-calendars-title = Календаров пока нет.
calendar-state-empty-calendars-desc = Календарь содержит ваши события. Создайте календарь для начала планирования.
calendar-state-empty-upcoming-title = Ничего не запланировано
calendar-state-empty-upcoming-desc = Твоя следующая встреча здесь, чтобы день никогда не удивил тебя.
calendar-state-empty-upcoming-action = Расписание события

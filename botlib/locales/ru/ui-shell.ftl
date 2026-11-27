# =============================================================================
# General Bots — en — Desktop shell, navigation, controls
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# =============================================================================
# General Bots - English UI Translations
# =============================================================================
# -----------------------------------------------------------------------------
# Navigation
# -----------------------------------------------------------------------------
nav-home = Домой
nav-chat = Чат
nav-drive = Поехали.
nav-tasks = Задачи
nav-mail = Почта
nav-calendar = Календарь
nav-meet = Встречайте
nav-paper = Бумага
nav-video = Видео
nav-research = Исследования
nav-analytics = Анализ
nav-settings = Параметры
nav-admin = Административные органы
nav-monitoring = Контроль
nav-sources = Источники
nav-tools = Инструменты
nav-attendant = Участвующий
nav-learn = Учись
nav-crm = МРС
nav-billing = Выставление счетов
nav-products = Продукты
nav-tickets = Билеты
nav-docs = Доксы
nav-sheet = Лист
nav-slides = Слайды
nav-social = Социальные
nav-all-apps = Все приложения
nav-people = Люди
nav-editor = Редактор
nav-dashboards = Доски
nav-security = Безопасность
nav-designer = Проектировщик
nav-project = Проект
nav-canvas = Канвы
nav-goals = Цели
nav-player = Игрок
nav-workspace = Рабочее пространство

# -----------------------------------------------------------------------------
# Dashboard
# -----------------------------------------------------------------------------
dashboard-title = Доска
dashboard-welcome = С возвращением,{ $name }- Я не знаю.
dashboard-quick-actions = Быстрые действия
dashboard-recent-activity = Деятельность, осуществлявшаяся в последнее время
dashboard-no-activity = Не было никакой новой деятельности.
dashboard-analytics = Анализ

# -----------------------------------------------------------------------------
# Quick Actions
# -----------------------------------------------------------------------------
quick-start-chat = Начать разговор
quick-upload-files = Загрузка файлов
quick-new-task = Новая задача
quick-compose-email = Сделать электронную почту
quick-start-meeting = Начать совещание
quick-new-event = Новое событие

# -----------------------------------------------------------------------------
# Application Cards
# -----------------------------------------------------------------------------
app-chat-name = Чат
app-chat-desc = Задавайте вопросы, обращайтесь за помощью и автоматизуйте задания.
app-drive-name = Поехали.
app-drive-desc = Облачное хранение всех ваших файлов.
app-tasks-name = Задачи
app-tasks-desc = Оставайтесь в организации, с списками задач, приоритетами и сроками.
app-mail-name = Почта
app-mail-desc = Клиент электронной почты с помощью АИ и умной организацией.
app-calendar-name = Календарь
app-calendar-desc = Запланируйте встречи, мероприятия и эффективно распоряжайтесь своим временем.
app-meet-name = Встречайте
app-meet-desc = Видеоконференция с обменом экранами и трансляциями в прямом эфире.
app-paper-name = Бумага
app-paper-desc = Напишите документы с помощью ИИ. Записки, отчеты и т.д.
app-research-name = Исследования
app-research-desc = Поиски и открытия с помощью АИ по всем вашим источникам.
app-analytics-name = Анализ
app-analytics-desc = Доски и отчеты для отслеживания использования и понимания.

# -----------------------------------------------------------------------------
# Suite Header
# -----------------------------------------------------------------------------
suite-title = Комнат General Bots
suite-tagline = Твоя производительность с помощью АИ, болтай, сотрудничай и создай.
suite-new-intent = Новый замысел

# -----------------------------------------------------------------------------
# AI Panel
# -----------------------------------------------------------------------------
ai-developer = ИИ Разработчик
ai-developing = Разработка:{ $project }
ai-quick-actions = Быстрые действия
ai-add-field = Добавить поле
ai-change-color = Изменить цвет
ai-add-validation = Добавить подтверждение
ai-export-data = Экспорт данных
ai-placeholder = Напечатайте ваши модификации...
ai-thinking = МА думает...
ai-status-online = Онлайн
ai-status-offline = Отключить

# -----------------------------------------------------------------------------
# Search
# -----------------------------------------------------------------------------
search-placeholder = Поиск...
search-no-results = Результаты отсутствуют
search-results = - Я не знаю.{ $count ->
    [one] { $count }результат
    *[other] { $count }результаты
    }- Я не знаю.
search-in-progress = Поиск...
search-advanced = Усовершенствованный поиск
search-filters = Фильтры
search-clear-filters = Очистить фильтры

# -----------------------------------------------------------------------------
# Pagination
# -----------------------------------------------------------------------------
# -----------------------------------------------------------------------------
# Tables
# -----------------------------------------------------------------------------
table-no-data = Данные отсутствуют
table-loading = Загрузка данных...
table-actions = Меры
table-select-all = Выбрать все
table-deselect-all = Отменить все
table-export = Экспорт
table-import = Импорт

# -----------------------------------------------------------------------------
# Forms
# -----------------------------------------------------------------------------
form-required = Требуемо
form-optional = Факультативная
form-submit = Представить
form-reset = Перезагрузить
form-clear = Очистить
form-uploading = Загрузка...
form-processing = Обработка...

# -----------------------------------------------------------------------------
# Modals / Dialogs
# -----------------------------------------------------------------------------
modal-confirm-title = Подтвердить действие
modal-confirm-message = Вы уверены, что хотите продолжить?
modal-delete-title = Удалить подтверждение
modal-delete-message = Это действие невозможно отменить.

# -----------------------------------------------------------------------------
# Tooltips
# -----------------------------------------------------------------------------
tooltip-copy = Копировать в буфер обмена
tooltip-copied = - Получил!
tooltip-expand = Расширение
tooltip-collapse = Свернуть
tooltip-refresh = Обновить
tooltip-download = Загрузка
tooltip-upload = Загрузка
tooltip-print = Печать
tooltip-fullscreen = Полноэкранный экран
tooltip-exit-fullscreen = Выход из полного экрана

# -----------------------------------------------------------------------------
# Common Labels (additional)
# -----------------------------------------------------------------------------
label-username = Имя пользователя
label-email = Электронная почта
label-display-name = Показать имя
label-password = Пароль
label-role = Роль
label-group-name = Название группы
label-hostname = Имя
label-record-type = Тип записи
label-target = Цель
label-your-name = Имя

# -----------------------------------------------------------------------------
# Actions (additional)
# -----------------------------------------------------------------------------
action-register = Регистрация

# -----------------------------------------------------------------------------
# Notifications
# -----------------------------------------------------------------------------
notifications-title = Уведомления
notifications-clear = Очистить все
notifications-empty = Нет уведомлений

# #1853 — desktop chrome, launchers and notification settings.
window-ready = Готов
window-loading = Загрузка приложения...
window-minimize = Сведение к минимуму
window-maximize = Максимальное
window-fullscreen = Полноэкранный экран
window-exit-fullscreen = Выход из полного экрана
window-close = Закрыть
app-loading = Загрузка приложения...
start-menu-search-placeholder = Найти приложение...
start-menu-recent = Последние
start-menu-no-apps = Не найдено приложений
start-menu-user = Пользователь
category-ai = МА
category-business = Бизнес
category-office = Отделение
category-dev = Разработчик
category-system = Система
category-tools = Инструменты
sidebar-apps = Приложения
sidebar-apps-toggle = Показать/поместить приложения
sidebar-apps-toggle-aria = Панель приложений
sidebar-apps-filter = Фильтры...
sidebar-apps-filter-aria = Фильтры
sidebar-preview = Режим просмотра
sidebar-preview-hint = Показать приложения, которые еще находятся в предварительном просмотре
sidebar-preview-toggle = Режим просмотра
sidebar-app-preview-title = {app}(обзор)
sidebar-apps-count = {count}приложения
sidebar-apps-count-preview = {count}(+){preview}Предварительный просмотр)
spotlight-placeholder = Поиск приложений, файлов... или набрать цель для консьержа
spotlight-hint = β β Навигатор · Введите запуск ·  kde+Space или Esc закрытия
spotlight-empty = Нет результатов
spotlight-goal = Цель: Δ{goal}
spotlight-goal-subtitle = Консьерж · план & выполнять все приложения
palette-placeholder = Поиск приложений и команд. Или напечатать @ <command>, чтобы отправить помощнику...
palette-empty = Нет соответствующих приложений или команд
palette-kind-app = Прием
palette-hint = ▪ Навигатор · Введите запуск · Esc Закрыть ·  kde+K toggle
palette-open-app = Открыто{app}
palette-value-prompt = {name}::{summary}- стоимость для{key}::
palette-execute = Выполнять{method} {path}и резюмирует результат.
palette-use-command = Использовать{command}командование{summary}.
toast-dismiss = Расторжение
settings-language-display = Показать язык
settings-language-display-desc = Выберите язык для интерфейса приложения
settings-language-regional = Региональные параметры
settings-language-regional-desc = Настроить форматы даты, времени и числа
settings-language-date-format = Дата Формат
settings-language-time-format = Формат времени
settings-language-timezone = Время
settings-language-first-day = Первый день недели
settings-language-time-12h = 12 часов (1:30 ТЧ)
settings-language-time-24h = 24 часа (13:30)
settings-language-sunday = Воскресенье
settings-language-monday = Понедельник
settings-email-notifications-desc = Отправлено отправителем уведомления о рабочем пространстве, никогда не с помощью подключенного почтового ящика Gmail/Outlook.
settings-push-enable = Включить уведомления
settings-push-enable-desc = Получать уведомления о нажатии браузера
settings-push-sound = Звук
settings-push-sound-desc = Воспроизвести звук для уведомлений
settings-desktop-notifications = Уведомления о рабочем столе
settings-desktop-notifications-desc = Показать уведомления на рабочем столе
settings-badge-count = Значок
settings-badge-count-desc = Показать нечитаемый значок
settings-save-preferences = Сохранить преференции
settings-notifications-saved = Экономия в отношении преференций в уведомлении
settings-notification-recipient = Почтовый ящик для доставки
settings-notification-recipient-desc = Оставьте пустой для использования вашего основного почтового аккаунта или профиля.
language-search-placeholder = Поиск на языках...
settings-profile-title = Параметры профиля
settings-security-title = Параметры безопасности

# semantic i18n key (#2542)
apps-menu-placeholder-search = Поиск приложений...
apps-menu-aria-search = Поиск приложений
apps-menu-aria-close = Закрыть меню приложений

# --- #2921 missing keys added to every locale catalogue ---
palette-kind-tour = Тур
palette-show-tour = Показать тур
palette-show-tour-hint = Показать тур по продукту
sidebar-app-hide = Скрыть{ $app }из этого бара
sidebar-apps-count-off = { $count }выключено
sidebar-apps-empty = Нет приложений, которые можно было бы показать
sidebar-apps-manage = Управление приложениями
sidebar-apps-manage-all = Показать все
sidebar-apps-manage-close = Закрыть
sidebar-apps-manage-empty = Приложения отсутствуют
sidebar-apps-manage-summary = { $on }В процентах{ $total }показано
sidebar-apps-none = Без приложений
toast-open = Открыто

# Assistant shell — knowledge-base source blocks
trecho = выдержки
trechos = выдержки

# Assistant/Drive shell keys integrated from the gbgov edition
arquivo = файл
arquivos = файлы
assistant-agent-backend-off = Агент отключился.
assistant-requires-agent-mode = Требуется режим агента
assistant-agent-mode-disabled = Для использования агентов по подписке необходимо установить режим агента.
assistant-agent-locked-hint = Агент заперт для этой модели.
assistant-agent-locked-tip = Выберите другую модель для использования агента
assistant-regenerate-older = Регенерация старого ответа начинает новый поворот
caracteres = символы
corrigido = фиксированный
de = В процентах
drive-actions = Меры
drive-folders = Папки
drive-usage-hint = Используемое хранение
drive-used = Используется
item = Пункт
itens = Пункты
linha = линия
linhas = линии
resultado = результат
resultados = результаты
shell-close = Закрыть
shell-conversations = Беседы
shell-machine = Машина

# Assistant shell — literal keys
assistant-vm-wants = Агент хочет совершить действие
assistant-vm-wants-command = Агент хочет выполнить команду на машине.
drive-label = Поехали.

# Chat / assistant parity — VM consent and subscription-agent UI keys
chat-vm-consent-assistant = Помощник
chat-vm-consent-subject = Агент
chat-vm-consent-wants = Агент хочет совершить действие
chat-vm-consent-wants-command = Агент хочет управлять командой
chat-vm-consent-kicker = Нужна ваша помощь.
chat-vm-consent-command-label = Команда
chat-vm-consent-where = Где
chat-vm-consent-where-vm = Виртуальная машина
chat-vm-consent-where-drive = Ваш двигатель
chat-vm-consent-destination = Назначение
chat-vm-consent-language = Язык
chat-vm-consent-packages = Установки из Интернета
chat-vm-consent-from = Источник
chat-vm-consent-background = Пробегает на заднем плане
chat-vm-consent-repair = Исправленная версия программы
chat-vm-consent-protected = Эта просьба касается полномочий агента; она может быть разрешена только один раз.
chat-vm-consent-all-note = Позволить всем описывать все действия этого агента в этом разговоре, только внутри Виртуальной Машины.
chat-vm-consent-outside-note = Это действие оставляет Виртуальную Машину: позволить всем не покрывать его.
chat-vm-consent-allow-once = Разрешите один раз
chat-vm-consent-allow-conversation = Разрешите в этом разговоре
chat-vm-consent-allow-all = Разрешите всем
chat-vm-consent-deny = Отрицание
chat-vm-consent-outcome-all = Полное разрешение в этом разговоре
chat-vm-consent-outcome-conversation = Разрешено в этом разговоре
chat-vm-consent-outcome-once = Разрешено
chat-vm-consent-outcome-denied = Отрицание: ничего не было убежано
chat-vm-consent-outcome-failed = Не смог записать ваше решение.
chat-vm-consent-forbidden = Это решение не может быть зарегистрировано здесь.
chat-vm-consent-announce = Запрос на разрешение открыт. Нажмите Альт+П для ответа.
chat-vm-consent-still-open = Запрос на разрешение еще открыт. Выберите опцию; Дени не работает.
chat-vm-consent-waiting = Жду твоего разрешения.
chat-vm-all-chip = Полное разрешение
chat-vm-all-scope = только внутри виртуальной машины
chat-vm-all-revoke = Отмена
chat-vm-all-revoked = Полное разрешение отменено
chat-vm-send-head = Подтверждение
chat-vm-send-text = Отправка теперь отменяет открытый запрос разрешения.
chat-vm-send-back = Возвращаемся к просьбе
chat-vm-send-go = Отправить
chat-vm-engine-local-short = Местный
chat-vm-engine-local-group = Местные модели
chat-vm-engine-local-detail = На этом компьютере
chat-vm-engine-free-short = Свободные
chat-vm-engine-free-group = Свободная модель
chat-vm-engine-free-detail = Свободная модель
chat-vm-engine-own-short = Собственный ключ
chat-vm-engine-own-group = Свои ключи
chat-vm-engine-own-detail = Собственный сервер или ключ
chat-vm-engine-sub-short = Подписка
chat-vm-engine-sub-group = Подписные издания
chat-vm-engine-sub-detail = Подписные агенты
chat-vm-agent-locked-tip = Выбранный агент работает в виртуальной машине; выбирает другую модель, чтобы выключить его
chat-vm-agent-locked-note = Виртуальная машина работает, так что агент выключает выключатель, пока он выбран. Выберите другую модель, чтобы выключить его.
atende = Участники
assistant-no-update = Нет доступных обновлений

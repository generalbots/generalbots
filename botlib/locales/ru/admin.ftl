# =============================================================================
# General Bots - Admin Translations (English)
# =============================================================================
# Administrative interface translations for the GB Admin Panel
# =============================================================================

# -----------------------------------------------------------------------------
# Admin Navigation & Dashboard
# -----------------------------------------------------------------------------
admin-title = Административное обслуживание
admin-dashboard = Административный аппарат
admin-overview = Общий обзор
admin-welcome = Добро пожаловать в группу админ

admin-nav-dashboard = Доска
admin-nav-users = Пользователи
admin-nav-bots = Боты
admin-nav-tenants = Арендаторы
admin-nav-settings = Параметры
admin-nav-logs = Журналы
admin-nav-analytics = Анализ
admin-nav-security = Безопасность
admin-nav-integrations = Интеграция
admin-nav-billing = Выставление счетов
admin-nav-support = Поддержка
admin-nav-groups = Группы
admin-nav-dns = ДНС
admin-nav-system = Система

# -----------------------------------------------------------------------------
# Admin Quick Actions
# -----------------------------------------------------------------------------
admin-quick-actions = Быстрые действия
admin-create-user = Создать пользователя
admin-create-group = Создать группу
admin-register-dns = Регистр DNS
admin-recent-activity = Деятельность, осуществлявшаяся в последнее время
admin-system-health = Система здравоохранения

# -----------------------------------------------------------------------------
# User Management
# -----------------------------------------------------------------------------
admin-users-title = Управление пользователями
admin-users-list = Список пользователей
admin-users-add = Добавить пользователя
admin-users-edit = Правка пользователя
admin-users-delete = Удалить пользователя
admin-users-search = Поиск пользователей...
admin-users-filter = Пользователи фильтра
admin-users-export = Экспортировать пользователей
admin-users-import = Импортировать пользователей
admin-users-total = Всего пользователей
admin-users-active = Активные пользователи
admin-users-inactive = Неактивные пользователи
admin-users-suspended = Перерывные пользователи
admin-users-pending = До проверки
admin-users-last-login = Последний вход
admin-users-created = Создан
admin-users-role = Роль
admin-users-status = Статус
admin-users-actions = Меры
admin-users-no-users = Не найдены пользователи
admin-users-confirm-delete = Вы уверены, что хотите удалить этого пользователя?
admin-users-deleted = Удаление пользователя
admin-users-saved = Удалось сохранить пользователя
admin-users-invite = Приглашать пользователя
admin-users-invite-sent = Приглашение направлено успешно
admin-users-bulk-actions = Действия навалом
admin-users-select-all = Выбрать все
admin-users-deselect-all = Отменить все

# User Details
admin-user-details = Подробности пользователей
admin-user-profile = Профиль
admin-user-email = Электронная почта
admin-user-name = Имя
admin-user-phone = Телефон
admin-user-avatar = Аватар
admin-user-timezone = Время
admin-user-language = Язык
admin-user-role-admin = Администратор
admin-user-role-manager = Менеджер
admin-user-role-user = Пользователь
admin-user-role-viewer = Просмотр
admin-user-status-active = Активные
admin-user-status-inactive = Неактивные
admin-user-status-suspended = Отложено
admin-user-status-pending = Нерассмотренные
admin-user-permissions = Разрешения
admin-user-activity = Журнал мероприятий
admin-user-sessions = Активные сессии
admin-user-terminate-session = Прекращение сессии
admin-user-terminate-all = Прекращение всех сессий
admin-user-reset-password = Перезагрузить пароль
admin-user-force-logout = Выход сил
admin-user-enable-2fa = Включить 2FA
admin-user-disable-2fa = Отключить 2FA

# -----------------------------------------------------------------------------
# Group Management
# -----------------------------------------------------------------------------
admin-groups-title = Группа по управлению
admin-groups-subtitle = Управление группами, членами и разрешениями
admin-groups-list = Список групп
admin-groups-add = Добавить группу
admin-groups-create = Создать группу
admin-groups-edit = Правка Группа
admin-groups-delete = Удалить группу
admin-groups-search = Поисковые группы...
admin-groups-filter = Группы фильтров
admin-groups-total = Всего по группам
admin-groups-active = Активные группы
admin-groups-no-groups = Группы не найдены
admin-groups-confirm-delete = Вы уверены, что хотите удалить эту группу?
admin-groups-deleted = Группа успешно удалена
admin-groups-saved = Группа успешно спасена
admin-groups-created = Группа, созданная успешно
admin-groups-loading = Загрузка групп...

# Group Details
admin-group-details = Подробности группы
admin-group-name = Название группы
admin-group-description = Описание
admin-group-visibility = Видимость
admin-group-visibility-public = Общественность
admin-group-visibility-private = Частные
admin-group-visibility-hidden = Скрытая
admin-group-join-policy = Присоединиться к политике
admin-group-join-invite = Только приглашение
admin-group-join-request = Просьба о присоединении
admin-group-join-open = Открыто
admin-group-members = Члены
admin-group-member-count = - Я не знаю.{ $count ->
    [one] { $count }Член
    *[other] { $count }Члены
    }- Я не знаю.
admin-group-add-member = Добавить член
admin-group-remove-member = Удалить
admin-group-permissions = Разрешения
admin-group-settings = Параметры
admin-group-analytics = Анализ
admin-group-overview = Общий обзор

# Group View Modes
admin-groups-view-grid = Вид сетки
admin-groups-view-list = Список
admin-groups-all-visibility = Вся видимость

# -----------------------------------------------------------------------------
# DNS Management
# -----------------------------------------------------------------------------
admin-dns-title = Управление ДНП
admin-dns-subtitle = Регистрация и управление DNS имена носителя для ваших ботов
admin-dns-register = Регистрация
admin-dns-registered = Зарегистрированные имена
admin-dns-search = Поиск имён...
admin-dns-refresh = Обновить
admin-dns-loading = Загрузка записей DNS...
admin-dns-no-records = Не найдены записи DNS
admin-dns-confirm-delete = Вы уверены, что хотите удалить это имя?
admin-dns-deleted = Имя узла успешно удалено
admin-dns-saved = Запись DNS успешно сохранена
admin-dns-created = Имя принимающей страны успешно зарегистрировано

# DNS Form Fields
admin-dns-hostname = Имя
admin-dns-hostname-placeholder = mibot. Пример.com
admin-dns-hostname-help = Введите полное имя домена, которое вы хотите зарегистрироваться
admin-dns-record-type = Тип записи
admin-dns-record-type-a = A (IPv4)
admin-dns-record-type-aaaa = АААА (IPv6)
admin-dns-record-type-cname = КНАМ
admin-dns-ttl = TTL (секунды)
admin-dns-ttl-5min = 5 минут (300)
admin-dns-ttl-1hour = 1 час (3600)
admin-dns-ttl-1day = 1 день (86400)
admin-dns-target = Адрес цели/IP
admin-dns-target-placeholder-ipv4 = 192.168.1.1
admin-dns-target-placeholder-ipv6 = 2001 год: db8:1:
admin-dns-target-placeholder-cname = Цель. Пример.com
admin-dns-target-help-a = Введите адрес IPv4 для указания
admin-dns-target-help-aaaa = Введите адрес IPv6 для указания
admin-dns-target-help-cname = Введите имя домена
admin-dns-auto-ssl = Автоматическое положение Сертификат SSL

# DNS Table Headers
admin-dns-col-hostname = Имя
admin-dns-col-type = Тип
admin-dns-col-target = Цель
admin-dns-col-ttl = TTL
admin-dns-col-ssl = SSL
admin-dns-col-status = Статус
admin-dns-col-actions = Меры

# DNS Status
admin-dns-status-active = Активные
admin-dns-status-pending = Нерассмотренные
admin-dns-status-error = Ошибка
admin-dns-ssl-enabled = SSL Включено
admin-dns-ssl-disabled = Нет SSL
admin-dns-ssl-pending = SSL Ожидает рассмотрения

# DNS Info Cards
admin-dns-help-title = DNS Справка о конфигурации
admin-dns-help-a-record = Запись
admin-dns-help-a-record-desc = Картирование домена на адрес IPv4. Используйте это, чтобы указать имя своего хоста непосредственно на IP сервера.
admin-dns-help-aaaa-record = Запись AAAA
admin-dns-help-aaaa-record-desc = Картирует доменное имя на адрес IPv6.
admin-dns-help-cname-record = КНАМЕЙСКАЯ РЕГИСТРАЦИЯ
admin-dns-help-cname-record-desc = Создаёт псевдоним из одного домена в другой, который помогает направлять поддомы в ваш главный домен.
admin-dns-help-ssl = SSL/TLS
admin-dns-help-ssl-desc = Автоматические положения Давайте зашифруем сертификаты для защищенных соединений HTTPS.

# DNS Edit/Remove Modals
admin-dns-edit-title = Правка записи DNS
admin-dns-remove-title = Удалить имя хоста
admin-dns-remove-warning = Это позволит удалить запись DNS и любые связанные с ней сертификаты SSL. Имя носителя больше не будет решать.

# -----------------------------------------------------------------------------
# Bot Management
# -----------------------------------------------------------------------------
admin-bots-title = Управление ботами
admin-bots-list = Список ботов
admin-bots-add = Добавить ботинок
admin-bots-edit = Правка Бот
admin-bots-delete = Удалить ботинок
admin-bots-search = Поиск ботов...
admin-bots-filter = Фильтровочные боты
admin-bots-total = Всего
admin-bots-active = Активные боты
admin-bots-inactive = Неактивные боты
admin-bots-draft = Проекты ботов
admin-bots-published = Опубликованные боты
admin-bots-no-bots = Боты не найдены
admin-bots-confirm-delete = Вы уверены, что хотите удалить этот бота?
admin-bots-deleted = Удаление бота
admin-bots-saved = Бот успешно спасен
admin-bots-duplicate = Дублирующий боток
admin-bots-export = Экспортировать ботаник
admin-bots-import = Импортировать ботаник
admin-bots-publish = Опубликование
admin-bots-unpublish = Неиздательские
admin-bots-test = Испытательный боток
admin-bots-logs = Бот-журналы
admin-bots-analytics = Анализ бота
admin-bots-conversations = Беседы
admin-bots-templates = Типовые формы
admin-bots-dialogs = Диалоги
admin-bots-knowledge-base = База знаний

# Bot Details
admin-bot-details = Подробности бота
admin-bot-name = Имя бота
admin-bot-description = Описание
admin-bot-avatar = Бота Аватар
admin-bot-language = Язык
admin-bot-timezone = Время
admin-bot-greeting = Приветственное сообщение
admin-bot-fallback = Отклонение сообщения
admin-bot-channels = Каналы
admin-bot-channel-web = Веб-чат
admin-bot-channel-whatsapp = Что-то вроде
admin-bot-channel-telegram = Телеграмма
admin-bot-channel-slack = Слать
admin-bot-channel-teams = Группы Microsoft
admin-bot-channel-email = Электронная почта
admin-bot-model = AI Модель
admin-bot-temperature = Температура
admin-bot-max-tokens = Макс Токенс
admin-bot-system-prompt = Быстрое функционирование системы

# -----------------------------------------------------------------------------
# Tenant Management
# -----------------------------------------------------------------------------
admin-tenants-title = Управление арендодателями
admin-tenants-list = Список арендаторов
admin-tenants-add = Добавить арендатор
admin-tenants-edit = Правка
admin-tenants-delete = Удалить арендатора
admin-tenants-search = Поиск арендаторов...
admin-tenants-total = Всего арендаторов
admin-tenants-active = Активные арендаторы
admin-tenants-suspended = Отстраненные арендаторы
admin-tenants-trial = Судебные арендаторы
admin-tenants-no-tenants = Жильцы не найдены
admin-tenants-confirm-delete = Вы уверены, что хотите удалить этого арендатора?
admin-tenants-deleted = Удаление арендодателя успешно
admin-tenants-saved = :: Успешное сохранение арендатора

# Tenant Details
admin-tenant-details = Подробности арендаторов
admin-tenant-name = Имя арендатора
admin-tenant-domain = Домена
admin-tenant-plan = План
admin-tenant-plan-free = Свободные
admin-tenant-plan-starter = Запуск
admin-tenant-plan-professional = Категория специалистов
admin-tenant-plan-enterprise = Предприятие
admin-tenant-users = Пользователи
admin-tenant-bots = Боты
admin-tenant-storage = Используемое хранение
admin-tenant-api-calls = Звонки API
admin-tenant-limits = Ограничения использования
admin-tenant-billing = Выписка

# -----------------------------------------------------------------------------
# System Settings
# -----------------------------------------------------------------------------
admin-settings-title = Параметры системы
admin-settings-general = Общие параметры
admin-settings-security = Параметры безопасности
admin-settings-email = Параметры электронной почты
admin-settings-storage = Параметры хранения
admin-settings-integrations = Интеграция
admin-settings-api = Параметры API
admin-settings-appearance = Внешний вид
admin-settings-localization = Локализация
admin-settings-notifications = Уведомления
admin-settings-backup = & Восстановление
admin-settings-maintenance = Режим обслуживания
admin-settings-saved = Сохранённые параметры
admin-settings-reset = Перезагрузить в по умолчанию
admin-settings-confirm-reset = Вы уверены, что хотите перенастроить все настройки в по умолчанию?

# General Settings
admin-settings-site-name = Название сайта
admin-settings-site-url = Адрес сайта
admin-settings-admin-email = Электронная почта
admin-settings-support-email = Поддержка электронной почты
admin-settings-default-language = Язык
admin-settings-default-timezone = По умолчанию
admin-settings-date-format = Дата Формат
admin-settings-time-format = Формат времени
admin-settings-currency = Валюта

# Email Settings
admin-settings-smtp-host = Щит СМТП
admin-settings-smtp-port = Порт СМТП
admin-settings-smtp-user = Имя пользователя SMTP
admin-settings-smtp-password = Пароль SMTP
admin-settings-smtp-encryption = Зашифрование
admin-settings-smtp-from-name = Имя
admin-settings-smtp-from-email = С электронной почты
admin-settings-smtp-test = Отправить тест-почту
admin-settings-smtp-test-success = Отправка успешно тестового письма
admin-settings-smtp-test-failed = Не удалось отправить тестовый email

# Storage Settings
admin-settings-storage-provider = Поставщик
admin-settings-storage-local = Местное хранение
admin-settings-storage-s3 = Amazon S3
admin-settings-storage-otterio = OtterIO
admin-settings-storage-gcs = Сохранить облако в Google
admin-settings-storage-azure = Хранение в азурном слое
admin-settings-storage-bucket = Имя вешалки
admin-settings-storage-region = Регион
admin-settings-storage-access-key = Ключ доступа
admin-settings-storage-secret-key = Секретный ключ
admin-settings-storage-endpoint = Адрес конечного пункта

# -----------------------------------------------------------------------------
# System Logs
# -----------------------------------------------------------------------------
admin-logs-title = Системные журналы
admin-logs-search = Поисковые журналы...
admin-logs-filter-level = Фильтр по уровню
admin-logs-filter-source = Фильтр по источникам
admin-logs-filter-date = Фильтр по дате
admin-logs-level-all = Все уровни
admin-logs-level-debug = Отладка
admin-logs-level-info = Сведения
admin-logs-level-warning = Внимание
admin-logs-level-error = Ошибка
admin-logs-level-critical = Критические
admin-logs-export = Экспортные журналы
admin-logs-clear = Очистить журналы
admin-logs-confirm-clear = Вы уверены, что хотите очистить все журналы?
admin-logs-cleared = Обработка журналов успешно завершена
admin-logs-no-logs = Не найдены журналы
admin-logs-refresh = Обновить
admin-logs-auto-refresh = Автообновить
admin-logs-timestamp = Время
admin-logs-level = Класс должности
admin-logs-source = Источник
admin-logs-message = Сообщение
admin-logs-details = Подробности

# -----------------------------------------------------------------------------
# Analytics
# -----------------------------------------------------------------------------
admin-analytics-title = Анализ
admin-analytics-overview = Общий обзор
admin-analytics-users = Анализ пользователей
admin-analytics-bots = Анализ бота
admin-analytics-conversations = Анализы
admin-analytics-performance = Эффективность
admin-analytics-period = Период времени
admin-analytics-period-today = Сегодня
admin-analytics-period-week = Эта неделя
admin-analytics-period-month = Этот месяц
admin-analytics-period-quarter = Этот квартал
admin-analytics-period-year = В этом году
admin-analytics-period-custom = Другой диапазон
admin-analytics-export = Доклад об экспорте
admin-analytics-total-users = Всего пользователей
admin-analytics-new-users = Новые пользователи
admin-analytics-active-users = Активные пользователи
admin-analytics-total-bots = Всего
admin-analytics-active-bots = Активные боты
admin-analytics-total-conversations = Всего
admin-analytics-avg-response-time = Время реагирования
admin-analytics-satisfaction-rate = Уровень удовлетворенности
admin-analytics-resolution-rate = Показатель разрешения

# -----------------------------------------------------------------------------
# Security
# -----------------------------------------------------------------------------
admin-security-title = Безопасность
admin-security-overview = Обзор положения в области безопасности
admin-security-audit-log = Журнал ревизий
admin-security-login-attempts = Попытка взлома
admin-security-blocked-ips = Заблокированные ПИ
admin-security-api-keys = Ключи API
admin-security-webhooks = Веб-хуки
admin-security-cors = Параметры CORS
admin-security-rate-limiting = Коэффициент ограничения
admin-security-encryption = Зашифрование
admin-security-2fa = Двухфактная аутентификация
admin-security-sso = Единый знак
admin-security-password-policy = Политика пароля

# API Keys
admin-api-keys-title = Ключи API
admin-api-keys-add = Создать ключ API
admin-api-keys-name = Имя ключа
admin-api-keys-key = Ключ API
admin-api-keys-secret = Секретный ключ
admin-api-keys-created = Создан
admin-api-keys-last-used = Последний использован
admin-api-keys-expires = Прерывание
admin-api-keys-never = Никогда
admin-api-keys-revoke = Отмена
admin-api-keys-confirm-revoke = Вы уверены, что хотите отменить этот ключ?
admin-api-keys-revoked = Ключ API успешно аннулирован
admin-api-keys-created-success = Ключ API успешно создан
admin-api-keys-copy = Копировать в буфер обмена
admin-api-keys-copied = - Получил!
admin-api-keys-warning = Скопируй ключ от API сейчас.

# -----------------------------------------------------------------------------
# Billing
# -----------------------------------------------------------------------------
admin-billing-title = Выставление счетов
admin-billing-overview = Обзор счетов
admin-billing-current-plan = Текущий план
admin-billing-usage = Использование
admin-billing-invoices = Счета-фактуры
admin-billing-payment-methods = Методы оплаты
admin-billing-upgrade = План модернизации
admin-billing-downgrade = План понижения класса
admin-billing-cancel = Отмена подписки
admin-billing-invoice-date = Дата счета
admin-billing-invoice-amount = Сумма
admin-billing-invoice-status = Статус
admin-billing-invoice-paid = Выплачено
admin-billing-invoice-pending = Нерассмотренные
admin-billing-invoice-overdue = Просроченные
admin-billing-invoice-download = Загрузка счета-фактуры

# -----------------------------------------------------------------------------
# Backup & Restore
# -----------------------------------------------------------------------------
admin-backup-title = & Восстановление
admin-backup-create = Создать резервную копию
admin-backup-restore = Восстановить резервную копию
admin-backup-schedule = Резервирование графика
admin-backup-list = История резервного копирования
admin-backup-name = Имя резервного копирования
admin-backup-size = Размер
admin-backup-created = Создан
admin-backup-download = Загрузка
admin-backup-delete = Удалить
admin-backup-confirm-restore = Вы уверены, что хотите восстановить это резервное копирование? Это перезапишет текущие данные.
admin-backup-confirm-delete = Вы уверены, что хотите удалить это резервное копирование?
admin-backup-in-progress = Подкрепление...
admin-backup-completed = Успешное выполнение резервного копирования
admin-backup-failed = Ошибка резервного копирования
admin-backup-restore-in-progress = Всё в порядке...
admin-backup-restore-completed = Успешное восстановление
admin-backup-restore-failed = Не удалось восстановить

# -----------------------------------------------------------------------------
# Maintenance Mode
# -----------------------------------------------------------------------------
admin-maintenance-title = Режим обслуживания
admin-maintenance-enable = Включить режим обслуживания
admin-maintenance-disable = Отключить режим обслуживания
admin-maintenance-status = Текущее положение дел
admin-maintenance-active = Режим обслуживания активирован
admin-maintenance-inactive = Режим обслуживания не работает
admin-maintenance-message = Сообщение о техническом обслуживании
admin-maintenance-default-message = Мы сейчас проводим плановое техническое обслуживание. Проверьте, пожалуйста, как можно скорее.
admin-maintenance-allowed-ips = Разрешенные IP адреса
admin-maintenance-confirm-enable = Вы уверены, что хотите создать режим обслуживания? Пользователи не смогут получить доступ к системе.

# -----------------------------------------------------------------------------
# Common Admin UI Elements
# -----------------------------------------------------------------------------
admin-required = Требуемо
admin-optional = Факультативная
admin-loading = Загрузка...
admin-saving = Спасение...
admin-deleting = Удалять...
admin-confirm = Подтвердить
admin-cancel = Отмена
admin-save = Сохранить
admin-create = Создать
admin-update = Обновление
admin-delete = Удалить
admin-edit = Правка
admin-view = Вид
admin-close = Закрыть
admin-back = Назад
admin-next = Следующий
admin-previous = Предыдущая
admin-refresh = Обновить
admin-export = Экспорт
admin-import = Импорт
admin-search = Поиск
admin-filter = Фильтр
admin-clear = Очистить
admin-select = Выбрать
admin-select-all = Выбрать все
admin-deselect-all = Отменить все
admin-actions = Меры
admin-more-actions = Больше действий
admin-no-data = Данные отсутствуют
admin-error = Ошибка
admin-success = Успех
admin-warning = Внимание
admin-info = Сведения

# Table Pagination
admin-showing = Показать{ $from }в адрес{ $to }В процентах{ $total }результаты
admin-page = Стр.{ $current }В процентах{ $total }
admin-items-per-page = Пункты на страницу
admin-go-to-page = Перейти к странице

# Bulk Actions
admin-bulk-delete = Удалить выбранное
admin-bulk-export = Экспорт
admin-bulk-activate = Активировать выбранный
admin-bulk-deactivate = Выключить выбранное
admin-selected-count = - Я не знаю.{ $count ->
    [one] { $count }выбранный пункт
    *[other] { $count }Отобранные позиции
    }- Я не знаю.


# ── Settings navigation (issue #1304) ──────────────────────────────
# The settings sidebar referenced these ids directly; without them the menu
# rendered the raw key instead of the label ("[id] instead of real text").
admin-audit = Журнал ревизий
admin-audit-subtitle = Происхождение действия системы
admin-users = Пользователи
admin-users-subtitle = Люди, имеющие доступ к этому рабочему пространству
admin-groups = Группы
admin-bots = Боты
admin-bots-subtitle = Управление ассистентами
admin-dns = ДНС


# ── #2307 Users & Groups dialogs / status regions ──────────────────
admin-add-user = Добавить пользователя
admin-add-group = Добавить группу
admin-search-users = Поиск пользователей...
admin-search-groups = Поисковые группы...
admin-confirm-title = Подтвердить действие
admin-confirm-accept = Подтвердить
admin-col-user = Пользователь
admin-col-email = Электронная почта
admin-col-organization = Организация работы
admin-col-roles = Функции
admin-col-status = Статус
admin-col-actions = Меры
admin-status-active = Активные
admin-status-inactive = Неактивные
admin-action-edit = Правка
admin-action-delete = Удалить
admin-view-members = Члены просмотра
admin-members-suffix = Члены
admin-creating = Создаёт...
admin-user-created = Создание и добавление пользователя в организацию
admin-group-created = Группа, созданная успешно
admin-user-deleted = Удалённый пользователь
admin-group-deleted = Группа удалена
admin-member-removed = Член исключен
admin-confirm-delete-user = Удалить этого пользователя?
admin-confirm-delete-group = Удалить эту группу?
admin-confirm-remove-member = Член группы?
admin-failed-load-users = Не удалось загрузить пользователей.
admin-failed-load-groups = Не удалось загрузить группы.
admin-failed-create-user = Невозможно создать пользователя.
admin-failed-create-group = Не удалось создать группу.
admin-no-users = Не найдены пользователи
admin-no-groups = Группы не найдены

# semantic i18n key (#2542)
admin-org-switch-title = Организация работы

# semantic i18n key (#2542)
admin-org-create-new-title = Создать новую организацию

# semantic i18n key (#2542)
admin-org-name-label = Название организации

# semantic i18n key (#2542)
admin-org-url-slug-label = Слизистая лента

# semantic i18n key (#2542)
admin-org-slug-preview = Организация-разведка

# semantic i18n key (#2542)
admin-org-plan-label = План

# semantic i18n key (#2542)
admin-org-plan-free = Свободные

# semantic i18n key (#2542)
admin-org-plan-starter = Запуск

# semantic i18n key (#2542)
admin-org-create-btn = Создать организацию

# semantic i18n key (#2542)
admin-filter-all-roles = Все роли

# semantic i18n key (#2542)
admin-user-role-moderator = Координатор

# semantic i18n key (#2542)
admin-filter-all-status = Все состояние

# semantic i18n key (#2542)
admin-users-last-active = Последний актив

# semantic i18n key (#2542)
admin-users-loading = Загрузка пользователей...

# semantic i18n key (#2542)
admin-users-create-new = Создать новый пользователь

# semantic i18n key (#2542)
admin-users-send-welcome-email = Отправить приветственное письмо

# semantic i18n key (#2542)
admin-users-require-password-change = Требовать изменения пароля при первом входе

# semantic i18n key (#2542)
admin-users-delete-warning = Это действие невозможно отменить. Все данные пользователя будут удалены навсегда.

# semantic i18n key (#2542)
accounts-subtitle = Управление процессом интеграции социальных сетей и каналов обмена сообщениями

# semantic i18n key (#2542)
accounts-channel-facebook = Facebook

# semantic i18n key (#2542)
accounts-channel-twitter-x = Твиттер / X

# semantic i18n key (#2542)
accounts-channel-bluesky = Блюски

# semantic i18n key (#2542)
accounts-channel-threads = Столбцы

# semantic i18n key (#2542)
accounts-channel-tiktok = TikTok

# semantic i18n key (#2542)
accounts-channel-youtube = YouTube

# semantic i18n key (#2542)
accounts-channel-pinterest = Процент

# semantic i18n key (#2542)
accounts-channel-reddit = Реддит

# semantic i18n key (#2542)
accounts-channel-snapchat = Снэпчат

# semantic i18n key (#2542)
accounts-messaging = Сообщение

# semantic i18n key (#2542)
accounts-channel-whatsapp-business = WhatsApp бизнес

# semantic i18n key (#2542)
accounts-channel-twilio-sms = Twilio SMS

# semantic i18n key (#2542)
accounts-channel-wechat = Векот

# semantic i18n key (#2542)
accounts-channel-gmail-google-workspace = Gmail/Google Workspace

# semantic i18n key (#2542)
accounts-channel-outlook-office-365 = Прогноз/Управление 365

# semantic i18n key (#2542)
accounts-channel-custom-smtp = Обычный SMTP

# semantic i18n key (#2542)
accounts-redirecting-authorization = Перенаправлять на разрешение...

# semantic i18n key (#2542)
accounts-configure-smtp-server = Настроить сервер SMTP

# semantic i18n key (#2542)
accounts-save-configuration = Сохранить конфигурацию

# semantic i18n key (#2542)
accounts-account-details = Подробности счетов

# semantic i18n key (#2542)
accounts-connection-status = Статус подключения

# semantic i18n key (#2542)
accounts-sample-username = @user имени

# semantic i18n key (#2542)
accounts-sample-connected-since = 15 января 2025 года

# semantic i18n key (#2542)
accounts-permissions = Разрешения

# semantic i18n key (#2542)
admin-groups-hidden = Скрытая

# semantic i18n key (#2542)
admin-groups-detail-title = Подробности группы

# semantic i18n key (#2542)
admin-groups-tab-overview = Общий обзор

# semantic i18n key (#2542)
admin-groups-tab-members = Члены

# semantic i18n key (#2542)
admin-groups-tab-permissions = Разрешения

# semantic i18n key (#2542)
admin-groups-tab-kbs = Базы знаний

# semantic i18n key (#2542)
admin-groups-create-title = Создать новую группу

# semantic i18n key (#2542)
admin-groups-visibility = Видимость

# semantic i18n key (#2542)
admin-groups-join-policy = Присоединиться к политике

# semantic i18n key (#2542)
admin-groups-invite-only = Только приглашение

# semantic i18n key (#2542)
admin-groups-request-join = Просьба о присоединении

# semantic i18n key (#2542)
admin-groups-allow-member-invite = Позвольте членам Комитета пригласить других

# semantic i18n key (#2542)
admin-groups-enable-chat = Использовать групповой чат

# semantic i18n key (#2542)
admin-groups-add-members = Добавить членов

# semantic i18n key (#2542)
admin-groups-search-users = Поиск пользователей

# semantic i18n key (#2542)
admin-groups-selected-users = Выбранные пользователи

# semantic i18n key (#2542)
admin-groups-no-users-selected = Нет выбранных пользователей

# semantic i18n key (#2542)
admin-groups-role-member = Член

# semantic i18n key (#2542)
admin-groups-role-moderator = Координатор

# semantic i18n key (#2542)
admin-groups-send-invites = Отправить приглашения

# semantic i18n key (#2542)
admin-groups-email-addresses = Адреса электронной почты

# semantic i18n key (#2542)
admin-groups-emails-help = Введите один адрес электронной почты на строчку

# semantic i18n key (#2542)
admin-groups-personal-message-optional = Личное сообщение (факультативно)

# semantic i18n key (#2542)
admin-groups-delete-warning = Это позволит удалить все члены и удалить все данные о группах. Это действие не может быть отменено.

# semantic i18n key (#2542)
admin-billing-dashboard-title = Выписка счетов

# semantic i18n key (#2542)
admin-billing-dashboard-subtitle = Всеобъемлющая аналитическая работа и анализ работы вашей организации по выставлению счетов и использованию

# semantic i18n key (#2542)
admin-billing-dashboard-current-period = Текущий период

# semantic i18n key (#2542)
admin-billing-dashboard-year-to-date = Год с начала

# semantic i18n key (#2542)
admin-billing-dashboard-current-spending = Текущие расходы

# semantic i18n key (#2542)
admin-billing-dashboard-quota-utilization = Использование квот

# semantic i18n key (#2542)
admin-billing-dashboard-average-quotas = Средний показатель по всем квотам

# semantic i18n key (#2542)
admin-billing-dashboard-api-calls = Звонки API

# semantic i18n key (#2542)
admin-billing-dashboard-storage-used = Используемое хранение

# semantic i18n key (#2542)
admin-billing-dashboard-spending-trends = Тенденции расходов

# semantic i18n key (#2542)
admin-billing-dashboard-daily = Ежедневно

# semantic i18n key (#2542)
admin-billing-dashboard-weekly = Еженедельно

# semantic i18n key (#2542)
admin-billing-dashboard-cost-breakdown = Разбивка расходов по видам услуг

# semantic i18n key (#2542)
admin-billing-dashboard-quota-management = Управление квотами

# semantic i18n key (#2542)
admin-billing-dashboard-team-members = Члены группы

# semantic i18n key (#2542)
admin-billing-dashboard-no-alerts = Нет спланированных предупреждений

# semantic i18n key (#2542)
admin-billing-dashboard-usage-by-bot = Использование Ботом

# semantic i18n key (#2542)
admin-billing-dashboard-bot-customer-support = Бот по поддержке клиентов

# semantic i18n key (#2542)
admin-billing-dashboard-bot-cs-01 = Бот_cs_01

# semantic i18n key (#2542)
admin-billing-dashboard-bot-sales-assistant = Помощник по продажам

# semantic i18n key (#2542)
admin-billing-dashboard-bot-sa-01 = Бот_sa_01

# semantic i18n key (#2542)
admin-billing-dashboard-bot-hr-assistant = Помощник по кадровым вопросам

# semantic i18n key (#2542)
admin-billing-dashboard-bot-hr-01 = бот_hr_01

# semantic i18n key (#2542)
admin-billing-dashboard-bot-it-helpdesk = Служба технической помощи ИТ

# semantic i18n key (#2542)
admin-billing-dashboard-bot-it-01 = бот_it_01

# semantic i18n key (#2542)
admin-billing-dashboard-recent-invoices = Последние счета

# semantic i18n key (#2542)
admin-billing-dashboard-alerts = Оповещение о выставлении счетов

# semantic i18n key (#2542)
admin-billing-dashboard-invoice-generated-january = Счет-фактура, составленная за январь

# semantic i18n key (#2542)
admin-billing-dashboard-quota-settings = Параметры кв.

# semantic i18n key (#2542)
admin-billing-dashboard-invoices-limit = Ограничение в отношении счетов-фактур

# semantic i18n key (#2542)
admin-billing-dashboard-team-members-limit = Предельный показатель для членов группы

# semantic i18n key (#2542)
admin-billing-dashboard-bots-limit = Предел ботов

# semantic i18n key (#2542)
admin-billing-dashboard-alert-thresholds = Предупреждения о порогах

# semantic i18n key (#2542)
admin-billing-subscription-title = Подписка на счета

# semantic i18n key (#2542)
admin-billing-manage-subscription-desc = Управление подпиской на свою организацию, методами оплаты и историей выставления счетов

# semantic i18n key (#2542)
admin-billing-per-month = /месяц

# semantic i18n key (#2542)
admin-billing-next-billing-date = Следующая дата выставления счетов:

# semantic i18n key (#2542)
admin-billing-next-billing-date-value = 21 февраля 2025 года

# semantic i18n key (#2542)
admin-billing-usage-this-period = Использование этого периода

# semantic i18n key (#2542)
admin-billing-usage-period-value = 21 января - 21 февраля 2025 года

# semantic i18n key (#2542)
admin-billing-payment-method = Метод оплаты

# semantic i18n key (#2542)
admin-billing-card-expiry-value = Срок действия истекает 12/2026

# semantic i18n key (#2542)
admin-billing-billing-address = Адрес выставления счетов

# semantic i18n key (#2542)
admin-billing-address-suite = Номер 456

# semantic i18n key (#2542)
admin-billing-address-city = Сан-Франциско, CA 94102

# semantic i18n key (#2542)
admin-billing-address-country = Соединенные Штаты

# semantic i18n key (#2542)
admin-billing-upgrade-your-plan = Улучшить свой план

# semantic i18n key (#2542)
admin-billing-recommended = Рекомендуемые

# semantic i18n key (#2542)
admin-billing-priority-support = Приоритетная поддержка

# semantic i18n key (#2542)
admin-billing-upgrade-to-business = Модернизация в бизнесе

# semantic i18n key (#2542)
admin-billing-enterprise = Предприятие

# semantic i18n key (#2542)
admin-billing-unlimited-everything = Не ограничивая все

# semantic i18n key (#2542)
admin-billing-dedicated-support = Специальная поддержка

# semantic i18n key (#2542)
admin-billing-custom-integrations = Интеграция в обыкновении

# semantic i18n key (#2542)
admin-billing-sla-guarantee = Гарантия ОАС

# semantic i18n key (#2542)
admin-billing-contact-sales = Продажи контактных документов

# semantic i18n key (#2542)
admin-billing-prorated-notice = Ваш следующий счет будет пропорционально распределен по оставшемуся периоду выставления счетов.

# semantic i18n key (#2542)
admin-billing-sorry-to-see-you-go = Жаль, что вы ушли!

# semantic i18n key (#2542)
admin-billing-cancel-consider-options = Прежде чем отменить, не хотите ли рассмотреть любой из этих вариантов?

# semantic i18n key (#2542)
admin-billing-stay-reduced-rate = Оставайся с нами по сниженной цене

# semantic i18n key (#2542)
admin-billing-accept-offer = Принять предложение

# semantic i18n key (#2542)
admin-billing-pause-your-subscription = Приостановите подписку

# semantic i18n key (#2542)
admin-billing-pause-up-to-3-months = Отдохни до трех месяцев.

# semantic i18n key (#2542)
admin-billing-pause-instead = Вместо этого приостановить

# semantic i18n key (#2542)
admin-billing-downgrade-smaller-plan = Понижен до менее крупного плана

# semantic i18n key (#2542)
admin-billing-keep-essential-features = Сохранение основных характеристик при меньших затратах

# semantic i18n key (#2542)
admin-billing-view-plans = Просмотр

# semantic i18n key (#2542)
admin-billing-why-cancelling = Почему ты отменяешь?

# semantic i18n key (#2542)
admin-billing-select-reason = Выберите причину...

# semantic i18n key (#2542)
admin-billing-reason-too-expensive = Слишком дорого.

# semantic i18n key (#2542)
admin-billing-reason-missing-features = Мне нужны недостающие характеристики

# semantic i18n key (#2542)
admin-billing-reason-not-using = Недостаточно.

# semantic i18n key (#2542)
admin-billing-reason-switching = Переключиться на другой продукт

# semantic i18n key (#2542)
admin-billing-reason-temporary = Временно - вернётся

# semantic i18n key (#2542)
admin-billing-additional-feedback = Дополнительные отзывы (факультативно)

# semantic i18n key (#2542)
admin-billing-keep-subscription = Сохранить подписку

# semantic i18n key (#2542)
admin-groups-placeholder-name = Инженерная группа
admin-groups-placeholder-description = Опишите цель этой группы...
admin-groups-placeholder-search-members = Поиск по имени или электронной почте...
admin-groups-placeholder-invite-emails = Введите адреса электронной почты, по одному на строчку...
admin-groups-placeholder-invite-message = Добавьте личное сообщение к приглашению...

# semantic i18n key (#2542)
admin-org-placeholder-search = Поиски организаций...

# semantic i18n key (#2542)
accounts-placeholder-smtp-host = Smtp.aximle.com
accounts-placeholder-smtp-username = Ваша почта: com
accounts-placeholder-smtp-password = • ● ● ● ● ● ● ●
accounts-placeholder-smtp-from-address = noreply@ample.com
accounts-placeholder-smtp-from-name = Ваша компания

# semantic i18n key (#2542)
admin-placeholder-username = Имя пользователя
admin-placeholder-email = пользователь@ample.com
admin-placeholder-display-name = Неизвестный
admin-placeholder-password = • ● ● ● ● ● ● ●
admin-placeholder-group-name = Инженерная группа
admin-placeholder-group-description = Описание группы...

# semantic i18n key (#2542)
admin-org-placeholder-name = Введите название организации
admin-org-placeholder-slug = Организация-разведка
admin-billing-placeholder-feedback = Помогите нам улучшить...
admin-billing-dashboard-placeholder-invoices-limit = 100
admin-billing-dashboard-placeholder-team-limit = 50
admin-billing-dashboard-placeholder-bots-limit = 10
admin-billing-dashboard-warning-threshold = Порог предупреждения (%)
admin-billing-dashboard-critical-threshold = Критическое пороговое значение (%)
admin-billing-dashboard-exceeded-threshold = Превышение порогового показателя (%)
admin-billing-dashboard-placeholder-warning = 80
admin-billing-dashboard-placeholder-critical = 90
admin-billing-dashboard-placeholder-exceeded = 100
# semantic i18n key (#2542)
admin-billing-dashboard-toggle-breakdown-view = Вид слома панели

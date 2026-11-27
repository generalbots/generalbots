# =============================================================================
# General Bots — en — Compliance
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# =============================================================================
# TOOLS / COMPLIANCE SCREENS
# =============================================================================
# -----------------------------------------------------------------------------
# Compliance Main
# -----------------------------------------------------------------------------
compliance-title = Доклад о соблюдении АПИ
compliance-subtitle = Сканирование безопасности всех ботов - Проверка паролей, хрупкого кода и ошибок в выборе
compliance-export-report = Доклад об экспорте
compliance-run-scan = Проверьте соответствие
compliance-scanning = Сканирую...

# -----------------------------------------------------------------------------
# Bot Selector
# -----------------------------------------------------------------------------
compliance-all-bots = Все боты
compliance-select-bots = Выбрать боты

# -----------------------------------------------------------------------------
# Stats Cards
# -----------------------------------------------------------------------------
compliance-critical = Критические
compliance-critical-desc = Требует незамедлительных действий
compliance-high = Высокая
compliance-high-desc = Риск для безопасности
compliance-medium = Средний
compliance-medium-desc = Следует рассмотреть
compliance-low = Низкая
compliance-low-desc = Передовая практика
compliance-info = Сведения
compliance-info-desc = Информационное

# -----------------------------------------------------------------------------
# Filters
# -----------------------------------------------------------------------------
compliance-filter-severity = Последовательность
compliance-filter-type = Тип
compliance-filter-all-severities = Все суммы
compliance-filter-all-types = Все типы
compliance-search-issues = Проблемы с поиском...

# -----------------------------------------------------------------------------
# Issue Types
# -----------------------------------------------------------------------------
compliance-type-password = Пароль в Конфиге
compliance-type-hardcoded = Зашифрованные секреты
compliance-type-deprecated = Ужасные ключевые слова
compliance-type-fragile = Код хрупкого состояния
compliance-type-config = Вопросы конфигурации

# -----------------------------------------------------------------------------
# Results Table
# -----------------------------------------------------------------------------
compliance-results = Результаты
compliance-results-count = - Я не знаю.{ $count ->
    [one] { $count }обнаруженная проблема
    *[other] { $count }В. Выявленные проблемы
    }- Я не знаю.
compliance-col-severity = Последовательность
compliance-col-issue = Вопрос
compliance-col-location = Место проведения
compliance-col-details = Подробности
compliance-col-action = Меры
compliance-view-details = Вид Подробности
compliance-fix-issue = Исправить вопрос
compliance-ignore = Игнорировать
compliance-no-issues = Никаких проблем не выявлено
compliance-no-issues-desc = Отлично, твои боты в порядке.

# -----------------------------------------------------------------------------
# Scan Progress
# -----------------------------------------------------------------------------
compliance-scan-in-progress = Сканирование продолжается...
compliance-scan-checking = Проверка{ $item }...
compliance-scan-complete = Сканирование завершено
compliance-scan-failed = Сканирование не удалось

## Compliance frameworks (enterprise-grade release)
compliance-frameworks = Рамки
compliance-new-framework = Новые должности
compliance-framework-name = Имя
compliance-framework-version = Версия
compliance-framework-description = Описание
compliance-create-framework = Создать систему
compliance-controls = Контроль
compliance-add-control = Добавить контроль
compliance-control-id = Идентификатор контрольного устройства
compliance-control-title = Название
compliance-control-category = Категория
compliance-control-description = Описание
compliance-mandatory = Обязательно
compliance-optional = Факультативная
compliance-evidence = Доказательства
compliance-attach-evidence = Приложить доказательства
compliance-evidence-path = Файловый путь (привод артефакт)
compliance-evidence-type = Тип
compliance-approve = Утвердить
compliance-covered = Охватываемые
compliance-no-evidence = Нет доказательств
compliance-export-csv = Экспорт CSV
compliance-archive = Архив
compliance-total-controls = Общий объем мер контроля
compliance-coverage = Охват
compliance-no-frameworks = Пока никаких структур.

# semantic i18n key (#2542)
admin-compliance-dashboard-title = Сетка для проверки соответствия

# semantic i18n key (#2542)
admin-compliance-subtitle = СЗ 2 Состояние соблюдения требований типа II, контроль за контролем и аудиторские доказательства

# semantic i18n key (#2542)
admin-compliance-compliant = Сотрудничаю

# semantic i18n key (#2542)
admin-compliance-overall-score = Общий показатель соблюдения

# semantic i18n key (#2542)
admin-compliance-controls-passing = Проход контроля

# semantic i18n key (#2542)
admin-compliance-evidence-items-due = Доказательства

# semantic i18n key (#2542)
admin-compliance-until-next-audit = До следующей ревизии

# semantic i18n key (#2542)
admin-compliance-next-audit-date = Запланировано на 7 марта 2025 года

# semantic i18n key (#2542)
admin-compliance-trust-service-criteria = Критерии доверительной службы

# semantic i18n key (#2542)
admin-compliance-availability = Наличие

# semantic i18n key (#2542)
admin-compliance-processing-integrity = Целостность обработки

# semantic i18n key (#2542)
admin-compliance-confidentiality = Конфиденциальность

# semantic i18n key (#2542)
admin-compliance-controls-requiring-attention = Контроль требует внимания

# semantic i18n key (#2542)
admin-compliance-control-mfa-privileged = Многофакторная аутентификация привилегированных счетов

# semantic i18n key (#2542)
admin-compliance-due-jan-25-2025 = Дата: 25 января 2025 года

# semantic i18n key (#2542)
admin-compliance-remediate = Реанимация

# semantic i18n key (#2542)
admin-compliance-control-incident-response-testing = Проверка плана реагирования на инциденты, связанные с безопасностью

# semantic i18n key (#2542)
admin-compliance-issue-annual-test-overdue = Ежегодное испытание просрочено на 15 дней

# semantic i18n key (#2542)
admin-compliance-control-dpa-documentation = Документация, касающаяся соглашения о обработке данных

# semantic i18n key (#2542)
admin-compliance-due-feb-1-2025 = Срок: 1 февраля 2025 года

# semantic i18n key (#2542)
admin-compliance-control-data-classification-review = Обзор политики в области классификации данных

# semantic i18n key (#2542)
admin-compliance-issue-policy-review-pending = Обзор политики в ожидании утверждения

# semantic i18n key (#2542)
admin-compliance-due-feb-15-2025 = Дата: 15 февраля 2025 года

# semantic i18n key (#2542)
admin-compliance-control-privacy-notice-updates = Обновление уведомлений о неприкосновенности частной жизни

# semantic i18n key (#2542)
admin-compliance-issue-privacy-policy-update = Необходимо обновить политику в области обеспечения конфиденциальности для сбора новых данных

# semantic i18n key (#2542)
admin-compliance-due-feb-28-2025 = Дата: 28 февраля 2025 года

# semantic i18n key (#2542)
admin-compliance-evidence-collection = Сбор доказательств

# semantic i18n key (#2542)
admin-compliance-current = Текущая

# semantic i18n key (#2542)
admin-compliance-due-soon = Скоро должен быть

# semantic i18n key (#2542)
admin-compliance-evidence-pentest-q4 = Протокол испытания на проникновение Q4 2024

# semantic i18n key (#2542)
admin-compliance-evidence-cc4-vuln = CC4.1.1 - Оценка уязвимости

# semantic i18n key (#2542)
admin-compliance-evidence-access-review-jan = Доклад о рассмотрении доступа - январь 2025 года

# semantic i18n key (#2542)
admin-compliance-evidence-cc6-access = CC6.2.1 - Контроль доступа

# semantic i18n key (#2542)
admin-compliance-due-in-3-days = Придется через 3 дня

# semantic i18n key (#2542)
admin-compliance-evidence-backup-restore = Результаты испытания на восстановление

# semantic i18n key (#2542)
admin-compliance-evidence-a1-backup = A1.2.3 - Резервная и восстановительная система

# semantic i18n key (#2542)
admin-compliance-due-in-5-days = Придется через 5 дней

# semantic i18n key (#2542)
admin-compliance-evidence-security-training = Учебная документация по вопросам безопасности

# semantic i18n key (#2542)
admin-compliance-evidence-cc1-training = CC1.4.1 - Подготовка по вопросам безопасности

# semantic i18n key (#2542)
admin-compliance-view-all-evidence = Вид Все доказательства

# semantic i18n key (#2542)
admin-compliance-access-events = Мероприятия по доступу

# semantic i18n key (#2542)
admin-compliance-configuration-changes = Изменения конфигурации

# semantic i18n key (#2542)
admin-compliance-security-events = Мероприятия по обеспечению безопасности

# semantic i18n key (#2542)
admin-compliance-compliance-events = Мероприятия по обеспечению соблюдения

# semantic i18n key (#2542)
admin-compliance-failed-login-attempt = Ошибка ввода

# semantic i18n key (#2542)
admin-compliance-failed-login-details = :: Причина: недействительный пароль

# semantic i18n key (#2542)
admin-compliance-user-logged-in = Пользователи зарегистрированы

# semantic i18n key (#2542)
admin-compliance-user-logged-in-details = Пользователь: sarah.johnson@com.com ix IP: 10.0,0.45
# semantic i18n key (#2542) — compliance app shell
compliance-run-check = Проверьте
compliance-stat-total-checks = Всего проверок
compliance-stat-passed = Прошедший
compliance-stat-failed = Ошибка
compliance-stat-warnings = Предупреждение
compliance-stat-score = Счёт
compliance-tab-dashboard = Доска
compliance-tab-checks = Контроль
compliance-tab-issues = Проблемы
compliance-tab-audit-log = Журнал ревизий
compliance-tab-risk-matrix = Матрица рисков
compliance-tab-training = Подготовка кадров
compliance-recent-activity = Деятельность, осуществлявшаяся в последнее время
compliance-score-heading = Счёт соблюдения
compliance-check-results-heading = Проверить результаты
compliance-checks-heading = Проверка соответствия
compliance-issues-heading = Проблемы
compliance-audit-log-heading = Журнал ревизий
compliance-risk-matrix-title = Матрица рисков (сходство с х ударом)
compliance-impact-axis = Воздействие β
compliance-likelihood-axis = Сходство
compliance-training-records = Учебная документация
compliance-loading = Загрузка...
compliance-no-recent-activity = Недавняя деятельность
compliance-no-checks-found = Проверка не найдена
compliance-no-issues-found = Никаких проблем не выявлено
compliance-no-audit-entries = Отсутствие записей о ревизии
compliance-no-risks-recorded = Не зарегистрированы риски
compliance-no-training-records = Отсутствие данных о профессиональной подготовке
compliance-overall-compliance-score = Общий показатель соблюдения
compliance-col-id = Идентификационный номер
compliance-col-framework = Рамки
compliance-col-status = Статус
compliance-col-created = Создан
compliance-col-timestamp = Время
compliance-col-user = Пользователь
compliance-action-run = Беги
compliance-create-issue = Создать выпуск
compliance-new-training = Новая подготовка
compliance-framework-details = Подробности рамок
compliance-failed-load-framework = Не удалось загрузить систему
compliance-risk-score = Счёт:{ $score }
compliance-issue-title-placeholder = Название выпуска
compliance-issue-assignee-placeholder = Имя получателя
compliance-issue-description-placeholder = Опишите проблему...
compliance-training-title-placeholder = Название учебной подготовки
compliance-training-assignee-placeholder = Имя получателя
compliance-due-date = Дата
compliance-due-label = Причитается:
compliance-col-assignee = Назначенный сотрудник
compliance-create-training = Создать обучение

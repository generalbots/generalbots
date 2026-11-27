# =============================================================================
# General Bots — en — Billing, products and tickets
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Analytics (additional UI keys)
# -----------------------------------------------------------------------------
analytics-dashboard-title = Анализы
analytics-last-hour = Последний час
analytics-last-6h = Последние 6 часов
analytics-last-24h = Последние 24 часа
analytics-last-7d = Последние 7 дней
analytics-last-30d = Последние 30 дней

# =============================================================================
# Billing
# =============================================================================
# -----------------------------------------------------------------------------
# Billing Navigation & General
# -----------------------------------------------------------------------------
billing-title = Выставление счетов
billing-invoices = Счета-фактуры
billing-payments = Выплаты
billing-quotes = Котировки
billing-dashboard = Доска

# -----------------------------------------------------------------------------
# Billing Entities
# -----------------------------------------------------------------------------
billing-invoice = Счет
billing-invoice-desc = Счет клиенту
billing-payment = Платежи
billing-payment-desc = Полученные платежи
billing-quote = Квадрат
billing-quote-desc = Ценовая кавычка

# -----------------------------------------------------------------------------
# Billing Status
# -----------------------------------------------------------------------------
billing-due-date = Дата
billing-overdue = Просроченные
billing-paid = Выплачено
billing-pending = Нерассмотренные
billing-draft = Проект
billing-sent = Отправлено
billing-partial = Частичная
billing-cancelled = Отменено

# -----------------------------------------------------------------------------
# Billing Actions
# -----------------------------------------------------------------------------
billing-new-invoice = Новый счет-фактура
billing-new-quote = Новый кот
billing-new-payment = Новый платеж
billing-send-invoice = Отправить счет-фактуру
billing-record-payment = Платежи
billing-mark-paid = Отметить как оплаченное
billing-void = Пустота

# -----------------------------------------------------------------------------
# Billing Fields
# -----------------------------------------------------------------------------
billing-amount = Сумма
billing-tax = Налог
billing-subtotal = Итого
billing-total = Всего
billing-discount = Скидка
billing-line-items = Линейные позиции
billing-add-item = Добавить пункт
billing-remove-item = Удалить элемент
billing-customer = Клиент
billing-issue-date = Дата выпуска
billing-payment-terms = Условия оплаты
billing-notes = Примечания
billing-invoice-number = Номер счета-фактуры
billing-quote-number = Количество

# -----------------------------------------------------------------------------
# Billing Reports
# -----------------------------------------------------------------------------
billing-revenue = Поступления
billing-outstanding = Невыплаченные
billing-this-month = Этот месяц
billing-last-month = Последний месяц
billing-total-paid = Итого, оплачено
billing-total-overdue = Итого, просрочено
billing-subtitle = Счета, платежи и цитаты
billing-revenue-month = Поступления в этом месяце
billing-total-revenue = Общая сумма поступлений
billing-paid-month = Выплачено в этом месяце

# -----------------------------------------------------------------------------
# Billing Empty States
# -----------------------------------------------------------------------------
billing-no-invoices = Не найдены счета-фактуры
billing-no-payments = Платежи не установлены
billing-no-quotes = Не найдены цитаты

# =============================================================================
# Products
# =============================================================================
# -----------------------------------------------------------------------------
# Products Navigation & General
# -----------------------------------------------------------------------------
products-title = Продукты
products-catalog = Каталог
products-services = Услуги
products-price-lists = Списки цен
products-inventory = Инвентаризация

# -----------------------------------------------------------------------------
# Products Entities
# -----------------------------------------------------------------------------
products-product = Продукт
products-product-desc = Физический или цифровой продукт
products-service = Обслуживание
products-service-desc = Предоставление услуг
products-price-list = Список цен
products-price-list-desc = Ценообразование

# -----------------------------------------------------------------------------
# Products Actions
# -----------------------------------------------------------------------------
products-new-product = Новый продукт
products-new-service = Новая служба
products-new-price-list = Новый список цен
products-new-pricelist = Новый список цен
products-edit-product = Правка Продукт
products-duplicate = Дублировать

# -----------------------------------------------------------------------------
# Products Fields
# -----------------------------------------------------------------------------
products-sku = СКЮ
products-category = Категория
products-price = Цена
products-unit = Единица
products-stock = Активы
products-cost = Стоимость
products-margin = Маргин
products-barcode = Баркод

# -----------------------------------------------------------------------------
# Products Status
# -----------------------------------------------------------------------------
products-in-stock = В товарном запасе
products-out-of-stock = Неучтенный
products-low-stock = Низкий запас
products-active = Активные
products-inactive = Неактивные
products-featured = Определённые
products-archived = Архивировано

# -----------------------------------------------------------------------------
# Products Stats & Metrics
# -----------------------------------------------------------------------------
products-subtitle = Управление товарами, услугами и ценообразованием
products-items = Продукты
products-pricelists = Списки цен
products-total-products = Всего продуктов
products-total-services = Всего услуг

# -----------------------------------------------------------------------------
# Products Empty States
# -----------------------------------------------------------------------------
products-no-products = Продукты не найдены
products-no-services = Услуги не найдены
products-no-price-lists = Не найдены списки цен

# =============================================================================
# Tickets (Support Cases)
# =============================================================================
# -----------------------------------------------------------------------------
# Tickets Navigation & General
# -----------------------------------------------------------------------------
tickets-title = Билеты
tickets-cases = Случаи
tickets-open = Открыто
tickets-closed = Закрыто
tickets-all = Все билеты
tickets-my-tickets = Мои билеты

# -----------------------------------------------------------------------------
# Tickets Entities
# -----------------------------------------------------------------------------
tickets-case = Случай
tickets-case-desc = Вспомогательный билет
tickets-resolution = Резолюция
tickets-resolution-desc = Предлагаемый раствор АИ

# -----------------------------------------------------------------------------
# Tickets Priority
# -----------------------------------------------------------------------------
tickets-priority = Приоритет
tickets-priority-low = Низкая
tickets-priority-medium = Средний
tickets-priority-high = Высокая
tickets-priority-urgent = Срочно

# -----------------------------------------------------------------------------
# Tickets Status
# -----------------------------------------------------------------------------
tickets-status = Статус
tickets-status-new = Новые должности
tickets-status-open = Открыто
tickets-status-pending = Нерассмотренные
tickets-status-resolved = Постановлено
tickets-status-closed = Закрыто
tickets-status-on-hold = Приостановить

# -----------------------------------------------------------------------------
# Tickets Actions
# -----------------------------------------------------------------------------
tickets-new-ticket = Новый билет
tickets-assign = Назначение
tickets-reassign = Перезапуск
tickets-escalate = Побег
tickets-resolve = Постановляет
tickets-reopen = Повторное открытие
tickets-close = Закрыть
tickets-merge = Объединить

# -----------------------------------------------------------------------------
# Tickets Fields
# -----------------------------------------------------------------------------
tickets-subject = Тема
tickets-description = Описание
tickets-category = Категория
tickets-assigned = Назначены
tickets-unassigned = Нераспределенные должности
tickets-created = Создан
tickets-updated = Обновление
tickets-response-time = Время реагирования
tickets-resolution-time = Время принятия резолюции
tickets-customer = Клиент
tickets-internal-notes = Внутренние примечания
tickets-attachments = Добавления

# -----------------------------------------------------------------------------
# Tickets AI Features
# -----------------------------------------------------------------------------
tickets-ai-suggestion = Предложение МА
tickets-apply-suggestion = Применить предложение
tickets-ai-summary = Резюме МА
tickets-similar-tickets = Аналогичные билеты
tickets-suggested-articles = Предлагаемые статьи

# -----------------------------------------------------------------------------
# Tickets Empty States
# -----------------------------------------------------------------------------
tickets-no-tickets = Билеты не найдены
tickets-no-open = Без открытых билетов
tickets-no-closed = Без закрытых билетов

# ============================================================================
# Office & business apps (#2091 #2092)
# ============================================================================
# -----------------------------------------------------------------------------
# Products
# -----------------------------------------------------------------------------
common-no = Нет
common-yes = Выполнено
products-billing-cycle = Цикл выставления счетов
products-billing-type = Тип выставления счетов*
products-cat-all = Все категории
products-cat-consulting = Консультирование
products-cat-development = Разработка
products-cat-hardware = Аппаратное обеспечение
products-cat-implementation = Осуществление
products-cat-maintenance = Техническое обслуживание
products-cat-other = Прочие
products-cat-software = Программное обеспечение
products-cat-subscription = Подписка
products-cat-support = Поддержка
products-cat-training = Подготовка кадров
products-col-actions = Меры
products-col-currency = Валюта
products-col-default = Ошибка
products-col-description = Описание
products-col-items = Пункты
products-col-name = Имя
products-col-price = Цена
products-col-status = Статус
products-col-type = Тип
products-col-unit = Единица
products-col-valid-from = Действительно с
products-col-valid-to = Действительно
products-create = Создать продукт
products-create-service = Создать службу
products-currency-all = Все валюты
products-cycle-annually = Ежегодно
products-cycle-monthly = Месячные
products-cycle-quarterly = Ежеквартально
products-description = Описание
products-image = Фото продукта
products-max-hours = Макс. часы (расчетный)
products-min-hours = Минимум часов
products-name = Название продукта*
products-new = Новый продукт
products-pricelist-create = Создать список цен
products-pricelist-currency = Валюта
products-pricelist-description = Описание
products-pricelist-markup = По умолчанию Отметка %
products-pricelist-name = Название списка цен*
products-pricelist-type = Тип списка цен
products-pricelist-type-contract = Контракт
products-pricelist-type-promotional = Пропагандистская деятельность
products-pricelist-type-standard = Стандартные
products-pricelist-valid-from = Действительно с
products-pricelist-valid-until = Действительно до
products-search-placeholder = Поиск продуктов, услуг...
products-select-category = Выбрать категорию...
products-select-type = Выберите тип...
products-service-name = Служебное наименование*
products-status = Статус
products-status-active = Активные
products-status-all = Все
products-status-inactive = Неактивные
products-taxable = Налоговые расходы
products-trial-days = Судебные периоды (дни)
products-type-all = Все типы
products-type-fixed = Стабильная цена
products-type-hourly = Часовая скорость
products-type-recurring = Повторная (подписка)
products-unit-day = В день
products-unit-hour = Час
products-unit-license = Лицензия
products-unit-month = Месяц
products-unit-project = По проектам
products-unit-seat = Сиденье
products-unit-unit = Единица
products-unit-year = Год
products-upload-image = Нажмите на изображение для загрузки

# semantic i18n key (#2542)
billing-placeholder-invoice-amount = 0,00 долл. США
billing-placeholder-amount = 0,00
billing-placeholder-reference = Идентификационный номер операции, проверка No и т.д.
billing-placeholder-payment-notes = Дополнительные платежные записи...
products-placeholder-service-name = Например, технические консультационные услуги
products-placeholder-service-description = Опишите услуги, которые предлагают...
products-placeholder-price = 0,00
products-placeholder-trial-days = 0
products-placeholder-min-hours = 1
products-placeholder-max-hours = 40
products-placeholder-pricelist-name = Например, стандартная ценовая система 2026 года
products-placeholder-pricelist-description = Описание списка цен...
products-placeholder-markup = 0,00
products-placeholder-product-name = Введите название продукта
products-placeholder-sku = PRD-001
products-placeholder-product-description = Описание продукта...
products-placeholder-cost = 0,00
# semantic i18n key (#2542)
billing-notes-placeholder = Дополнительные записи для клиента...
billing-invoice-notes-placeholder = Оплатные инструкции или дополнительные примечания...
billing-remove-line-item = Удалить строку

# -----------------------------------------------------------------------------
# #2328 — loading / empty / error state copy
# -----------------------------------------------------------------------------
products-state-empty-title = Продуктов пока нет.
products-state-empty-desc = Продукты, которые вы добавляете в каталог, появляются здесь, готовые к продаже.
products-state-empty-action = Добавить первый продукт
products-state-empty-services-title = Услуги пока отсутствуют
products-state-empty-services-desc = Услуги, которые вы предлагаете, появляются здесь с их ценой и единицей.
products-state-empty-services-action = Добавить первую службу
products-state-empty-pricelists-title = Пока нет списков цен
products-state-empty-pricelists-desc = Перечень цен группирует товары под одной валютой и окном действительности.
products-state-empty-pricelists-action = Создать список цен
billing-payments-empty-title = Платежи пока не зарегистрированы
billing-payments-empty-desc = Вы регистрируете платежи по счету-фактуре, который появляется здесь с указанием их метода и даты.
billing-quotes-empty-title = Пока никаких цитат
billing-quotes-empty-desc = Отправьте цитату клиенту, и она будет отслеживаться здесь, пока она не будет принята.
billing-inventory-empty-title = Еще нет инвентарных запасов
billing-inventory-empty-desc = Запасы и их количество в наличии перечислены здесь, как только вы их добавите.
billing-gl-empty-title = В бухгалтерской книге еще нет счетов
billing-gl-empty-desc = Для каждого перевода нужен счет, чтобы установить баланс между судебным процессом и показаниями.
billing-procurement-empty-title = Заказы на поставку пока отсутствуют
billing-procurement-empty-desc = Заказы на поставку, которые вы размещаете у поставщиков, отслеживаются здесь, пока они не получены.

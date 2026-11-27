# =============================================================================
# General Bots — en — Attendant (support queue)
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# =============================================================================
# ATTENDANT / CRM SCREENS
# =============================================================================
# -----------------------------------------------------------------------------
# CRM Disabled State
# -----------------------------------------------------------------------------
attendant-crm-disabled = Недопущенные элементы CRM
attendant-crm-disabled-desc = В рамках программы Centent Console требуется, чтобы для этого бота были созданы функции CRM. Это позволяет человеческим агентам получать и отвечать на разговоры, переданные с бота.
attendant-crm-enable-instruction = Чтобы создать возможности для CRM, добавьте эту линию к вашей боту
attendant-crm-config-file = config.csv
attendant-crm-create-attendant = Тогда сотвори
attendant-crm-attendant-file = Спутник.csv
attendant-crm-configure-team = Файл для настройки команды

# -----------------------------------------------------------------------------
# Queue Sidebar
# -----------------------------------------------------------------------------
attendant-title = Участвующий Консоле
attendant-status-online = Онлайн
attendant-status-busy = Занятые
attendant-status-away = Уходим.
attendant-status-offline = Отключить
attendant-status-ready = Онлайн - Готов к беседам
attendant-status-busy-msg = Занятые разговоры
attendant-status-away-msg = Уезжай, скоро вернёшься.
attendant-status-offline-msg = Открытая - данных нет

# -----------------------------------------------------------------------------
# Queue Stats
# -----------------------------------------------------------------------------
attendant-waiting = Ожидание
attendant-active = Активные
attendant-resolved = Постановлено
attendant-mine = Мой

# -----------------------------------------------------------------------------
# Queue Filters
# -----------------------------------------------------------------------------
attendant-filter-all = Все
attendant-filter-waiting = Ожидание
attendant-filter-mine = Мой
attendant-filter-priority = Приоритет

# -----------------------------------------------------------------------------
# Conversation List
# -----------------------------------------------------------------------------
attendant-no-conversations = Никаких разговоров в очереди
attendant-new-conversations-appear = Здесь появятся новые разговоры.
attendant-unread = Нечитаемый
attendant-typing = Напечатать...
attendant-select-conversation = Выберите разговор
attendant-select-conversation-desc = Выберите разговор из очереди, чтобы начать отвечать

# -----------------------------------------------------------------------------
# Channel Tags
# -----------------------------------------------------------------------------
attendant-channel-whatsapp = Что-то вроде
attendant-channel-teams = Группы
attendant-channel-instagram = Инстаграм
attendant-channel-web = Веб
attendant-channel-telegram = Телеграмма
attendant-channel-email = Электронная почта

# -----------------------------------------------------------------------------
# Priority Tags
# -----------------------------------------------------------------------------
attendant-priority-urgent = Срочно
attendant-priority-high = Высокая
attendant-priority-normal = Нормальный

# -----------------------------------------------------------------------------
# Chat Area
# -----------------------------------------------------------------------------
attendant-message-placeholder = Напечатайте сообщение...
attendant-send = Отправить
attendant-attach-file = Приложить файл
attendant-insert-emoji = Вставить эмоджи
attendant-quick-responses = Быстрые ответы
attendant-transfer = Передача
attendant-resolve = Постановляет
attendant-more-actions = Больше действий

# -----------------------------------------------------------------------------
# Quick Responses
# -----------------------------------------------------------------------------
attendant-quick-greeting = Чем могу помочь?
attendant-quick-thanks = Спасибо за терпение.
attendant-quick-checking = Давай я проверю.
attendant-quick-moment = Минутку, пожалуйста.

# -----------------------------------------------------------------------------
# Transfer Modal
# -----------------------------------------------------------------------------
attendant-transfer-title = Обсуждение передачи
attendant-transfer-to = Перевод в
attendant-transfer-reason = Причина (факультативно)
attendant-transfer-reason-placeholder = Зачем ты переводишь этот разговор?
attendant-transfer-cancel = Отмена
attendant-transfer-confirm = Передача

# -----------------------------------------------------------------------------
# AI Insights Sidebar
# -----------------------------------------------------------------------------
attendant-ai-insights = IA Понимание
attendant-ai-summary = Резюме обсуждения
attendant-ai-sentiment = Сенсибилизация клиентов
attendant-sentiment-positive = Положительные
attendant-sentiment-neutral = Нейтральный
attendant-sentiment-negative = Отрицательно
attendant-smart-replies = Умные ответы
attendant-confidence = Доверие
attendant-source = Источник

# -----------------------------------------------------------------------------
# Customer Details
# -----------------------------------------------------------------------------
attendant-customer-details = Подробности клиентов
attendant-customer-name = Имя
attendant-customer-email = Электронная почта
attendant-customer-phone = Телефон
attendant-customer-location = Место проведения
attendant-customer-tags = Метки

# -----------------------------------------------------------------------------
# Conversation History
# -----------------------------------------------------------------------------
attendant-history = История
attendant-history-resolved = Постановлено
attendant-history-transferred = Переведено
attendant-history-abandoned = Брошенные
attendant-view-history = Вид Полная история

# -----------------------------------------------------------------------------
# Toast Messages
# -----------------------------------------------------------------------------
attendant-toast-transferred = Успешное обсуждение
attendant-toast-resolved = Обсуждение, помеченное как решенное
attendant-toast-assigned = Вы получили задание поговорить
attendant-toast-error = Ошибка
attendant-toast-connection-lost = Связь потеряна.
attendant-toast-connection-restored = Восстановление связи

# semantic i18n key (#2542)
handoff-title = Передача и анализ

# semantic i18n key (#2542)
handoff-tab-queue = Крючок передачи

# semantic i18n key (#2542)
handoff-tab-agents = Агенты

# semantic i18n key (#2542)
handoff-tab-channels = Каналы

# semantic i18n key (#2542)
handoff-tab-csat = ССАТ

# semantic i18n key (#2542)
handoff-tab-transcripts = Переписные листы

# semantic i18n key (#2542)
handoff-tab-sla = Монитор ОАС

# semantic i18n key (#2542)
handoff-transfer-title = Перенести разговор

# semantic i18n key (#2542)
handoff-transfer-reason-skill = Несоответствие навыков

# semantic i18n key (#2542)
handoff-transfer-reason-workload = Балансирование рабочей нагрузки

# semantic i18n key (#2542)
handoff-transfer-reason-escalation = Нарастание числа клиентов

# semantic i18n key (#2542)
handoff-transfer-reason-vip = Обработка VIP

# semantic i18n key (#2542)
handoff-placeholder-search-conversations = Поиск разговоров...
handoff-placeholder-search-transcripts = Поисковые записи по ключевому слову или агенту...
handoff-placeholder-transfer-note = Факультативный контекст для принимающего агента

# semantic i18n key (#2542)
attendant-placeholder-transfer-reason = Например, технический вопрос, специалист по потребностям

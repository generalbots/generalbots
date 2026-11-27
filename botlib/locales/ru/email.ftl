# Transactional email text (#2094). Subject and body only; MIME and envelope stay untouched.

email-greeting = Привет{ $name },
email-signoff = Привет,
email-team = Группа по общим ботам
email-welcome-subject = Добро пожаловать в "Облако генерала Ботов"{ $plan }План
email-welcome-body = Добро пожаловать в "Облако" генерала Ботса!{ $plan }Вы можете начать создавать боты и управлять своим рабочим пространством в{ $url }
email-invoice-subject = Счет #{ $invoice } — общее облако ботов
email-invoice-body = Был создан новый счет-фактура.{ $invoice }Сумма:{ $amount } { $currency }. Посмотрите на детали на{ $url }
email-payment-success-subject = Подтвержденные платежи — общее облако ботов
email-payment-success-body = Ваша оплата{ $amount } { $currency }План:{ $plan }Спасибо за подписку!
email-subscription-activated-subject = Подписка активирована — общее облако ботов
email-subscription-activated-body = Твой{ $plan }Подписка сейчас активна. Вы можете справиться с ней в{ $url }
email-recurring-charge-subject = Регулярный заряд — счет-фактура #{ $invoice }
email-recurring-charge-body = Ваш постоянный подписный контракт был за счет.{ $invoice }Сумма:{ $amount } { $currency }. Посмотрите на детали на{ $url }
email-payment-failed-subject = Неудачный платеж — требуется принятие мер
email-payment-failed-body = Ваша оплата{ $amount } { $currency }Произошло неудачу.{ $invoice }Просьба обновить ваш метод оплаты по адресу:{ $url }
email-subscription-cancelled-subject = Подписка отменена — General Bots Cloud
email-subscription-cancelled-body = Твой{ $plan }Если вы хотите возобновить его, посетите его.{ $url }
email-grace-warning-subject = Просроченная оплата — предупреждение о приостановлении службы
email-grace-warning-body = Вы задержали оплату.{ $days }Просьба незамедлительно обновить свой метод оплаты по адресу:{ $url }
email-invitation-subject = Приглашение присоединиться{ $org }
email-invitation-body = Вас пригласили присоединиться{ $org }Примите приглашение по адресу:{ $url }1. Срок действия этого приглашения истекает в{ $expires }.
email-quota-limit-reached = Вы дозвонились до своего{ $metric }предел квот.
email-quota-threshold = Ты использовал{ $current }% от вашего{ $metric }квота (порог:{ $threshold }%).
email-metric-messages = сообщения
email-metric-storage = хранение
email-metric-api-calls = Звонит API
email-metric-bots = Боты
email-metric-users = пользователей
email-metric-kb-documents = справочных базовых документов
email-metric-apps = приложения
email-metric-emails-per-day = ежедневная электронная почта
email-metric-builds-per-day = сборки в день
email-metric-uploads-per-hour = Загрузка в час

# Sharing and mention notifications (#2094)
email-share-subject = { $sender }Разделяем".{ $file }"с тобой
email-share-download = Загрузка{ $file }
email-share-link-expires = Эта связь истекает{ $date }Вы можете отменить его в любое время.
email-share-link-no-expiry = Эта ссылка не истечет. Вы можете отменить ее в любое время.
email-mention-subject = { $author }упомянул тебя в комментарии
email-mention-body = { $author }упомянул тебя{ $resource_type }(В тыс. долл. США){ $resource_id })".{ $preview }Откройте приложение, чтобы посмотреть и ответить.

# semantic i18n key (#2542)
mail-new-message = Новое сообщение

# semantic i18n key (#2542)
mail-cc-bcc = Сс/Бк

# semantic i18n key (#2542)
mail-subject-label = Тема:

# semantic i18n key (#2542)
mail-track-opens = Открыть дорожку

# semantic i18n key (#2542)
mail-request-read-receipt = Просьба читать:

# semantic i18n key (#2542)
mail-priority-normal = Обычный приоритет

# semantic i18n key (#2542)
mail-priority-high = Высокая степень приоритетности

# semantic i18n key (#2542)
mail-priority-low = Низкий приоритет

# semantic i18n key (#2542)
mail-dialog-templates-title = Типовые формы электронной почты

# semantic i18n key (#2542)
mail-dialog-new-template = + Новая модель

# semantic i18n key (#2542)
mail-dialog-signatures-title = Электронная почта Подписание документов

# semantic i18n key (#2542)
mail-dialog-new-signature = + Новая подпись

# semantic i18n key (#2542)
mail-dialog-rules-title = Правила электронной почты

# semantic i18n key (#2542)
mail-dialog-new-rule = + Новое правило

# semantic i18n key (#2542)
mail-dialog-automatic-replies-title = Автоматические ответы

# semantic i18n key (#2542)
mail-dialog-send-automatic-replies = Отправить автоматические ответы

# semantic i18n key (#2542)
mail-dialog-start-date = Дата начала

# semantic i18n key (#2542)
mail-dialog-end-date = Дата окончания

# semantic i18n key (#2542)
mail-dialog-reply-only-org = Отправьте только людям в моей организации.

# semantic i18n key (#2542)
mail-dialog-add-account-title = Добавить счет электронной почты

# semantic i18n key (#2542)
mail-dialog-account-name = Имя счета

# semantic i18n key (#2542)
mail-dialog-imap-server = IMAP Сервер

# semantic i18n key (#2542)
mail-dialog-imap-port = Порт IMAP

# semantic i18n key (#2542)
mail-dialog-smtp-server = SMTP Сервер

# semantic i18n key (#2542)
mail-dialog-smtp-port = Порт СМТП

# semantic i18n key (#2542)
mail-dialog-use-ssl-tls = Использовать SSL/TLS

# semantic i18n key (#2542)
mail-dialog-add-account-action = Добавить счет

# semantic i18n key (#2542)
mail-dialog-schedule-send-title = Расписание

# semantic i18n key (#2542)
mail-dialog-schedule-action = График

# semantic i18n key (#2542)
mail-manage-labels-title = Управление знаками опасности

# semantic i18n key (#2542)
mail-label-name = Название знака

# semantic i18n key (#2542)
mail-label-color = Цвет

# semantic i18n key (#2542)
mail-new-signature-title = Подпись новой электронной почты

# semantic i18n key (#2542)
mail-signature-name = Подпись

# semantic i18n key (#2542)
mail-html-content = Содержание HTML

# semantic i18n key (#2542)
mail-new-rule-title = Правило о новой электронной почте

# semantic i18n key (#2542)
mail-rule-name = Имя и фамилия правила

# semantic i18n key (#2542)
mail-condition-sender-contains = Состояние (включено отправителем)

# semantic i18n key (#2542)
mail-action-star = Звезда

# semantic i18n key (#2542)
mail-new-template-title = Новая модель электронной почты

# semantic i18n key (#2542)
mail-template-name = Имя шаблона

# semantic i18n key (#2542)
mail-body-html = Тело HTML

# semantic i18n key (#2542)
mail-placeholder-label-name = Например, работа
mail-placeholder-signature-name = Например, профессиональная подпись
mail-placeholder-signature-content = Наилучшее приветствие, <br>b>Ваше имя>/b>

# semantic i18n key (#2542)
mail-placeholder-rule-name = Например, архивные бюллетени
mail-placeholder-rule-condition = Например, информационный бюллетень
mail-placeholder-template-name = Например, добро пожаловать по электронной почте
mail-placeholder-template-subject = Стандартный предмет
mail-placeholder-template-body = Привет{{name}}Добро пожаловать на нашу платформу!
mail-placeholder-autoreply-subject = Ответ
mail-placeholder-autoreply-message = Ваше сообщение...

# semantic i18n key (#2542)
mail-placeholder-search-emails = Поиск писем...
mail-placeholder-recipients = Получатели
mail-placeholder-carbon-copy = Копия углерода
mail-placeholder-blind-carbon-copy = Слепой углеродный экземпляр
mail-placeholder-subject = Тема
mail-placeholder-compose-message = Напиши свое сообщение...
mail-placeholder-account-name = Работа, личные дела и т.д.

# semantic i18n key (#2542)
mail-placeholder-imap-server = Imap.ample.com
mail-placeholder-smtp-server = Smtp.aximle.com
mail-placeholder-username-or-email = Имя пользователя или электронная почта
mail-placeholder-password = • ● ● ● ● ● ● ●

# -----------------------------------------------------------------------------
# #2328 — loading / empty / error state copy
# -----------------------------------------------------------------------------
mail-state-empty-title = Эта папка пуста
mail-state-empty-desc = Приходят новые сообщения, сами напишите, если вам нужно начать разговор.
mail-state-empty-action = Написать сообщение
mail-state-loading = Загрузка писем...
mail-templates-empty-title = Пока никаких шаблонов
mail-templates-empty-desc = Форма спасает объект и тело, которое вы часто посылаете, так что вы никогда не перепечатываете его.
mail-templates-empty-action = Создать шаблон
mail-signatures-empty-title = Подписей пока нет.
mail-signatures-empty-desc = К сообщениям, которые вы отправляете, прилагается подпись.
mail-signatures-empty-action = Создать подпись
mail-rules-empty-title = Пока никаких правил.
mail-rules-empty-desc = Файлы правил, звезды или удаляет входящую почту автоматически, например, все от отправителя.
mail-rules-empty-action = Создать правило

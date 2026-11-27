# =============================================================================
# General Bots — en — Office and business apps
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# VDI (remote desktop)
vdi-title = Виртуальный рабочий стол
vdi-new-connection = Новое подключение
vdi-connection-name = Имя соединения
vdi-host = Принимающая сторона
vdi-port = Порт
vdi-protocol = Протокол
vdi-rdp-password = Пароль RDP
vdi-rdp-domain = Домена ПРМ (факультативно)
vdi-save-connect = Сохранить & подсоединение
vdi-cancel = Отмена
vdi-connect = Подключиться
vdi-delete = Удалить
vdi-no-connections = Никаких связей пока.
vdi-create-first = Создать новую связь для начала
vdi-connecting = Подключение...
vdi-connected = Подключено
vdi-disconnected = Отключено
vdi-error = Ошибка
vdi-clipboard-sent = Отправлено на буфер обмена
vdi-ctrl-alt-del-sent = linkki+Alt+Del отправлен
vdi-rdp = ППР
vdi-vnc = ВНК
attendant-attach = Приложить файл
attendant-emoji = Эмоджи
attendant-uploading = Загрузка...
attendant-attach-error = Ошибка загрузки
attendant-emoji-search = Поиск эмоджи...
meet-record-title = Запись
meet-recordings = Запись
meet-recordings-empty = Записей пока нет.
player-playlists = Список воспроизведения
player-playlist-new = Новый список воспроизведения
player-playlist-name = Имя списка воспроизведения
player-playlist-create = Создать
player-playlist-rename = Переименовать
player-playlist-delete = Удалить список воспроизведения
player-playlist-add = Добавить в список воспроизведения
player-playlist-remove = Удалить
player-playlist-play = Воспроизвести
player-playlist-empty = Не создан список воспроизведения
player-playlist-empty-items = Пустой список воспроизведения
player-playlist-add-current = + Добавить текущие средства
player-playlist-created = Создан список воспроизведения
player-playlist-deleted = Список воспроизведения удален
player-playlist-renamed = Список воспроизведения
player-playlist-item-added = Добавить в список воспроизведения
player-playlist-item-removed = Удалить из списка воспроизведения
player-playlist-error = Не удалось загрузить списки воспроизведения

# -----------------------------------------------------------------------------
# Billing & ERP
# -----------------------------------------------------------------------------
billing-account = Счет*
billing-col-account = Счет
billing-col-actions = Меры
billing-col-amount = Сумма
billing-col-date = Дата
billing-col-invoice = Счет
billing-col-method = Метод
billing-col-opportunity = Возможности
billing-col-payment-id = Идентификационный номер платежа
billing-col-quote-number = Квитанция #
billing-col-status = Статус
billing-col-valid-until = Действительно до
billing-contact = Контактные данные
billing-create-send = Создать & отправку
billing-currency = Валюта
billing-details = Подробности счета-фактуры
billing-discount-fixed = Стационарная сумма
billing-discount-none = Без дисконтирования
billing-discount-percent = Процент (%)
billing-discount-type = Скидка
billing-discount-value = Скидочная стоимость
billing-invoice-amount = Сумма счета-фактуры
billing-invoice-date = Дата счета*
billing-invoice-details = Подробности счета-фактуры
billing-item-description = Описание
billing-item-price = Цена
billing-item-qty = Qty
billing-item-total = Всего
billing-method-all = Все методы
billing-method-bank = Банковский перевод
billing-method-boleto = Болето
billing-method-card = Кредитная карточка
billing-method-cash = Наличность
billing-method-check = Проверить
billing-method-other = Прочие
billing-method-pix = PIX
billing-opportunity = Возможности
billing-payment-date = Дата платежа*
billing-payment-details = Подробная информация об платежах
billing-payment-method = Метод оплаты*
billing-quote-accepted = Принимается
billing-quote-all = Все котировки
billing-quote-date = Дата
billing-quote-details = Подробности
billing-quote-draft = Проект
billing-quote-expired = Прерван
billing-quote-name = Имя
billing-quote-rejected = Отклонен
billing-quote-sent = Отправлено
billing-record = Платежи
billing-reference = Исходный номер
billing-save-draft = Сохранить в виде проекта
billing-search-placeholder = Поиск счетов, цитат...
billing-select-account = Выбрать счет...
billing-select-contact = Выберите контакт...
billing-select-invoice = Выбрать счет-фактуру...
billing-select-method = Выбор метода...
billing-select-opportunity = Выбрать возможность...
billing-terms = Термины & условия
billing-terms-immediate = Причитается по получении
billing-terms-net15 = Чистый 15
billing-terms-net30 = Чистый 30
billing-terms-net45 = Чистый 45
billing-terms-net60 = Чистый 60
billing-valid-until = Действительно до*
erp-gl = Гл
erp-inventory = Инвентаризация
erp-procurement = Закупки
player-sign-in-drive-media = Подпишитесь, чтобы посмотреть ваши средства массовой информации.
player-sign-in-recent-media = Подпишитесь, чтобы посмотреть последние СМИ.
player-load-drive-media-error = Не смог загрузить ваши диски.
player-load-recent-media-error = Не удалось загрузить последние СМИ.
player-filter-video = Фильтровальные видеоматериалы
player-filter-audio = Фильтр
player-filter-document = Документы фильтра
player-filter-image = Фильтр
player-fullscreen = Полноэкранный экран
player-toggle-playlist = Панель списка воспроизведения@ info: whatsthis

# semantic i18n key (#2542)
vdi-placeholder-quick-connect = : порт (например, 10.0,0.1:5900)
vdi-placeholder-host = 10.0.01
# semantic i18n key (#2542)
billing-quote-name-placeholder = Например, предложение по лицензированию программного обеспечения Q1
billing-item-description-placeholder = Описание продукта или услуги
billing-terms-placeholder = Условия оплаты, условия доставки и т.д.

# --- #2921 missing keys added to every locale catalogue ---
billing-filter-all = Все
billing-filter-draft = Проект
billing-filter-sent = Отправлено
billing-filter-paid = Выплачено
billing-filter-overdue = Просроченные
billing-filter-cancelled = Отменено
billing-period-all = Все периоды
billing-period-month = Месяц
billing-period-quarter = Четверо
billing-period-year = Год
billing-export = Экспорт
billing-col-number = Число
billing-col-due = Дата представления

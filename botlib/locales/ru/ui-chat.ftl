# =============================================================================
# General Bots — en — Chat
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Chat
# -----------------------------------------------------------------------------
chat-title = Чат
chat-placeholder = Напечатайте сообщение...
chat-send = Отправить
chat-new-conversation = Новые разговоры
chat-history = История разговоров
chat-clear = Очистить разговор
chat-export = Экспортировать чаевые
chat-typing = { $name }Напечатает...
chat-online = Онлайн
chat-offline = Отключить
chat-last-seen = Последний раз видел.{ $time }
chat-mention-title = Справочное подразделение
chat-mention-placeholder = Сообщение... (тип @, чтобы упомянуть)
chat-mention-search = Поиски...
chat-mention-no-results = Результаты отсутствуют
chat-mention-type-hint = Тип: для поиска

# Additional Chat translations
chat-camera = Камера
chat-camera-title = Сфотографируйся.
chat-camera-capture = Сфотографируйся.
chat-camera-cancel = Отмена
chat-camera-waiting = Жду, когда запустит камеру...
chat-camera-waiting-frames = Разрешение дано; ожидание видеокассет
chat-camera-ready = Камера готова, сфотографируйте или отмените.
chat-camera-started = Запущена камера — кран сфотографируйте.
chat-camera-sent = Фото отправлено.
chat-camera-denied = Отказывает в разрешении камеры.
chat-camera-missing = На этом устройстве не было найдено камеры.
chat-camera-no-device = нет видеоприбора
chat-camera-device = видеоприбор
chat-camera-no-frames = Предварительный просмотр камеры не начался. Выберите фото из сборщика системы.
chat-camera-timeout = Камера не ответила. Выбери фото из сборщика системы.
chat-camera-error = Не смог запустить камеру.
chat-camera-unsupported = Это приложение не обеспечивает доступ к камерам.
voice-mode-title = Голосовой разговор
voice-mode-listening = Слушать... говорить естественно
voice-mode-thinking = Думаете...
voice-mode-speaking = Говорю...
voice-mode-idle = Голосовой разговор
voice-mode-end = Конец разговору
chat-message-placeholder = Сообщение...

# Chat message controls and notices
chat-copy-message = Копировать сообщение
chat-copied = Копированная
chat-message-copied = Скопировано сообщение
chat-copy-failed = Ошибка копирования
chat-listen-tts = Слушай (СТО)
chat-listen-message = Послушай это сообщение.
chat-image-unavailable = Недоступно изображение
chat-open = Открыто
chat-expand = Расширение
chat-collapse = Свернуть
chat-preview = Просмотр
chat-consent-affected-count = Затрагиваемые записи
chat-close-preview = Закрыть просмотр
chat-preview-load-failed = Невозможно загрузить предварительный просмотр.
chat-show-more = Показать больше
chat-show-less = Показать меньше
chat-tool = инструмент
chat-param = пункт 1
chat-params = парамы
chat-tool-result = Результат инструмента
chat-retry = Повторить
chat-open-as-document = Открыто в качестве документа
chat-show-more-history = Покажи больше...
chat-disconnected-reconnecting = Переключаю на новый канал...
chat-connect-retrying = Не удалось связаться с сервером чата.
chat-connection-lost = Потеряна связь, попытка восстановить связь...
chat-attachment-sent = Приложение направлено:
chat-attachments-sent = Направлено приложение:
chat-message-not-delivered = Непоставленное сообщение — восстановление связи
chat-attachment-read-failed = Не смог прочитать вложение. Прикрепите его снова.
chat-attachment-queued = Скоро связь вернется — привязанность установлена
chat-reconnecting-queued = Восстановляя связь, ты получишь сообщение, когда связь вернется.
chat-bot-private-signin = Этот бобот - частный, и он должен продолжать
chat-approved-plan = Утвержденный план:
chat-retrying-attachment = Повторное вложение
chat-expanded-preview = Расширенный просмотр

# #2362 — chat code/artifact control labels
chat-code-show-all = Показать все
chat-code-lines = линии
chat-artifact-save-drive = Сохранить для управления
chat-artifact-download-svg = Загрузка SVG
chat-artifact-download-png = Загрузка PNG
chat-artifact-open-canvas = Открыто в Канвасе
chat-artifact-folder = Папка (факультативно)
chat-artifact-kind-markdown = Документ с отметкой
chat-artifact-kind-html = Страница HTML
chat-artifact-kind-code = Код
chat-artifact-words = слова

# #2362 — attachment action label
chat-open-in-app = Открыть приложение
# Assistant-look chat: transcript export and answer feedback
chat-export-actions = Меры по проведению обсуждений
chat-export-md = Экспортировать разговор как отметка
chat-export-print = Печать или сохранение в качестве PDF
chat-export-done = Экспортировать переговоры
chat-export-empty = Пока ничего не экспортировать.
chat-export-untitled = Обсуждение
chat-export-user = Ты
chat-export-assistant = Помощник
chat-feedback-label = Ответные ответы
chat-feedback-good = Хороший ответ
chat-feedback-bad = Плохой ответ
chat-feedback-comment = Что может быть лучше?
chat-feedback-send = Отправить
chat-staged-files = Прикрепленные файлы
chat-staged-close = Закрыть просмотр

# Chat engineer status strip (#2342)
chat-status-bot = Бот
chat-status-session = Сессия
chat-status-model = Модель
chat-status-state = Состояние
chat-status-no-session = -
chat-status-copy-session = Копировать идентификатор сеанса
chat-status-copied = Копированная
chat-status-idle = Проход
chat-status-streaming = Поток
chat-status-thinking = Мысли
chat-status-error = Ошибка
chat-status-stopped = Остановлено

chat-response-ready = Ответ готов
chat-response-started = Помощник отвечает
chat-tool-result-ready = Результат использования инструмента

# semantic i18n key (#2542)
chat-no-suggestions-available = Никаких предложений не поступило

# semantic i18n key (#2542)
chat-start-conversation = Начать разговор

# semantic i18n key (#2542)
chat-empty-hint = Наберите сообщение внизу или используйте голосовой ввод

# semantic i18n key (#2542)
chat-no-conversations-yet = Пока никаких разговоров.

# semantic i18n key (#2542)
chat-context-label = Контекст:

# semantic i18n key (#2542)
chat-select-context = Выберите контекст...

# semantic i18n key (#2542)
chat-no-contexts-configured = Контекстов нет

# semantic i18n key (#2542)
projector-title = Просмотр содержания

# semantic i18n key (#2542)
projector-loading-content = Загрузка содержания...

# semantic i18n key (#2542)
projector-slide-info = Слайд 1 из 1

# semantic i18n key (#2542)
projector-code-info = коды

# Projector viewer control (#2542)
chat-projector-rotate = Повернуть

# Chat header / app chrome (#2542)
chat-conversation-clear = Четкий контекст
chat-conversation-cleared = Разминирование контекста обсуждения
chat-conversation-copy-link = Копировать ссылку
chat-conversation-delete = Удалить
chat-conversation-delete-confirm = Удалить этот разговор?
chat-conversation-delete-failed = Не удалось удалить разговор
chat-conversation-delete-title = Удалить разговор
chat-conversation-delete-unavailable = Диалог подтверждения отсутствует
chat-conversation-deleted = Удалено
chat-conversation-export-json = Выписка экспорта (.json)
chat-conversation-export-md = Протокол экспорта (мд)
chat-conversation-exported = Экспортировать переговоры
chat-conversation-link-copied = Скопировано ссылка на разговоры
chat-conversation-link-copy-failed = Не удалось скопировать ссылку
chat-conversation-menu = Меню Для обсуждения
chat-conversation-rename = Переименовать
chat-conversation-rename-failed = Не удалось переименовать разговор
chat-conversation-renamed = Переименовано в разговор
chat-conversation-title = Название обсуждения
chat-conversation-untitled = Бессмысленный разговор

# -----------------------------------------------------------------------------
# #2328 — loading / empty / error state copy
# -----------------------------------------------------------------------------
chat-empty-title = Начинайте разговор.
chat-empty-desc = Наберите сообщение внизу или выберите предложение, когда ваш бобот предлагает его.
chat-empty-action = Написать первое сообщение

# -----------------------------------------------------------------------------
# #2342 — chat engineer UX: slash commands, help, inspect/copy-markdown
# -----------------------------------------------------------------------------
chat-cmd-menu-title = Команды
chat-cmd-help = Показать клавиши
chat-cmd-new = Начать новый разговор
chat-cmd-clear = Очистить этот разговор
chat-cmd-stop = Прекратить генерацию
chat-stop = Стой.
chat-cmd-failed = Ошибка команды
chat-help = Короткие клавиши
chat-inspect = Инспектирование
chat-inspect-title = Подробности сообщения
chat-inspect-sender = Отправитель
chat-inspect-id = Идентификатор сообщения
chat-inspect-no-id = нет
chat-inspect-length = Символы
chat-copy-markdown = Копировать как отметка
chat-copied-markdown = Нарисовано как отметка

# Model attribution on each bot answer
chat-model-written-by = Ответ

# Assistant parity in chat
assistant-parity-block-allow = Разрешить
assistant-parity-block-allowed = Разрешено
assistant-parity-block-api-command = Команда API
assistant-parity-block-attachment-read = Добавление читать:
assistant-parity-block-chars = символы
assistant-parity-block-command = Команда
assistant-parity-block-decision-failed = Не смог записать решение
assistant-parity-block-denied = Отказ
assistant-parity-block-deny = Отрицание
assistant-parity-block-detail = Подробная информация
assistant-parity-block-drive-list = Список водителей
assistant-parity-block-drive-read = Файл привода
assistant-parity-block-drive-search = Поиск привода
assistant-parity-block-elapsed = Свернуто
assistant-parity-block-error = Ошибка
assistant-parity-block-file = Файл
assistant-parity-block-kb-search = Поиск знаний
assistant-parity-block-notice = Уведомление
assistant-parity-block-path = Путь
assistant-parity-block-permission = Запрошенное разрешение
assistant-parity-block-query = Запрос
assistant-parity-block-read = Читать
assistant-parity-block-reason = Обоснование
assistant-parity-block-result = Результат
assistant-parity-block-retry = Повторить
assistant-parity-block-seconds = с
assistant-parity-block-size = Размер
assistant-parity-block-status-done = Сделано
assistant-parity-block-status-failed = Ошибка
assistant-parity-block-status-needs-you = Нужен ваш вклад
assistant-parity-block-status-running = Работает
assistant-parity-block-step = Шаг
assistant-parity-block-thinking = Мысли
assistant-parity-block-tool = Инструмент
assistant-parity-block-tool-run = Пропуск инструментов
assistant-parity-block-total = Всего
assistant-parity-block-truncated = Урезанные
assistant-parity-block-type = Тип
assistant-parity-block-yes = Выполнено
chat-parity-attach-title = Предварительный просмотр добавления
chat-parity-attach-unavailable = Недоступный контент - использовать загрузку для открытия этого файла.
chat-parity-attach-download-failed = Невозможно загрузить вложение.
chat-parity-model-tiers = Авто & уровни
chat-parity-model-accounts = Счета поставщиков
chat-parity-model-local = Местные & свободные
chat-drive-parity-attaching = Прикрепляем...
chat-drive-parity-cancel = Отмена
chat-drive-parity-delete = Удалить
chat-drive-parity-deleted = Исключен
chat-drive-parity-delete-failed = Невозможно удалить файл.
chat-drive-parity-delete-question = Удалить этот файл?
chat-drive-parity-delete-title = Удалить файл
chat-drive-parity-deleting = Удалять...
chat-drive-parity-download = Загрузка
chat-drive-parity-download-failed = Невозможно загрузить файл.
chat-drive-parity-downloading = Загрузка...
chat-drive-parity-more = Дополнительные действия
chat-drive-parity-new-name = Новое имя
chat-drive-parity-preview = Просмотр
chat-drive-parity-preview-failed = Невозможно просмотреть файл.
chat-drive-parity-previewing = Вступительный просмотр...
chat-drive-parity-rename = Переименовать
chat-drive-parity-renamed = Переименовано
chat-drive-parity-rename-failed = Невозможно переименовать файл.
chat-drive-parity-rename-title = Переименовать файл
chat-drive-parity-renaming = Переименование...
chat-drive-parity-unavailable = В ходе этого развертывания эта мера отсутствует.
chat-drive-parity-use-in-chat = Использование в разговоре
chat-parity-agent-does = Агент делает это
chat-parity-back = Назад
chat-parity-checked-now = Только что проверили.
chat-parity-close = Закрыть
chat-parity-coming-soon = Скоро приедет.
chat-parity-cost-contract = По вашему контракту
chat-parity-cost-free = Свободные
chat-parity-cost-free-limit = Бесплатно с ограничениями использования
chat-parity-cost-sub = Твоя подписка
chat-parity-cuda-no = Не обнаружено подходящих ГПУ
chat-parity-cuda-yes = Обнаружена карта графических данных
chat-parity-depends-agent = Зависит от агента
chat-parity-driver = Водитель
chat-parity-engine = Двигатель в эксплуатации
chat-parity-feature-docs = Документы
chat-parity-feature-ocr = Текст в изображениях
chat-parity-feature-photos = Фотографии и диаграммы
chat-parity-feature-web = Поиск в Интернете
chat-parity-features = Особенности
chat-parity-gains = Твои доходы
chat-parity-go-agent = Поедет к поставщику агента
chat-parity-go-ext = Поездка к внешнему поставщику
chat-parity-go-net = Остается в сети вашей организации
chat-parity-go-server = Идет на сервер, который вы установили
chat-parity-guide = Не уверен, что выбрать?
chat-parity-guide-sub = Ответь на два вопроса.
chat-parity-in-use = Используется
chat-parity-load-failed = Невозможно загрузить эту информацию.
chat-parity-loading = Загрузка...
chat-parity-machine = Этот компьютер
chat-parity-machine-ok = Этот компьютер имеет: %d, %v GB
chat-parity-machine-slow = У этого компьютера нет подходящей карты графики: он будет медленным.
chat-parity-need-acp = Установите агент и подпишитесь на ваш счет.
chat-parity-need-free = Ничего.
chat-parity-need-local = Карточка графики с 6 и более ГБ и 4-6 ГБ скачивания
chat-parity-need-own = Адрес сервера и ключ
chat-parity-no-read = Не читается
chat-parity-no-work = Не работает
chat-parity-none-found = Не найдено
chat-parity-not-yet = Пока нет.
chat-parity-only-net = Только с сервером в вашей сети
chat-parity-optional = Факультативно, вы включите его.
chat-parity-pick = Выберите опцию
chat-parity-priv-ext = Ваши разговоры направляются внешнему носителю (% h).
chat-parity-priv-local = Ты будешь говорить на этом компьютере.
chat-parity-priv-net = Ваши разговоры направляются на сервер вашей организации (% h).
chat-parity-privacy = Личность
chat-parity-q1 = Можете ли вы написать это?
chat-parity-q1-help = Подумайте о конфиденциальной или личной информации.
chat-parity-q1-no = Нет, он должен остаться здесь.
chat-parity-q1-no-desc = Приемлема только местная обработка.
chat-parity-q1-yes = Да, он может уйти.
chat-parity-q1-yes-desc = Внешние услуги могут обрабатывать его.
chat-parity-q2-no = Нет, или я не знаю.
chat-parity-q2-no-desc = Нет ни сервера, ни ключа.
chat-parity-q2-open = Ваша организация предлагает сервер или ключ АИ?
chat-parity-q2-open-help = Проверьте свою область ИТ, если не уверен.
chat-parity-q2-secret = Ваша организация допускает внешний АИ?
chat-parity-q2-secret-help = Некоторые стратегии в области данных запрещают внешние услуги.
chat-parity-q2-secret-no-desc = Внешний АИ не допускается.
chat-parity-q2-secret-yes-desc = Допускается использование внешнего АИ.
chat-parity-q2-yes = Выполнено
chat-parity-q2-yes-desc = Имеется сервер или ключ компании.
chat-parity-reads = Читать
chat-parity-ready = Готов
chat-parity-rec-free = Начните с свободной модели: нет необходимости в установке. Для конфиденциальных вопросов, предпочитайте на этом компьютере.
chat-parity-rec-local = Используйте этот компьютер: здесь ничего не останется, и у этой машины есть карта графики.
chat-parity-rec-local-slow = У этой машины нет подходящей графической карты для местной модели. Используйте сервер вашей организации в своей сети.
chat-parity-rec-local-tight = Местная модель будет работать, но может быть медленной с этой графической картой.
chat-parity-rec-own = Используйте сервер или ключ вашей организации: контракт определяет стоимость и пределы.
chat-parity-rec-own-net = Используйте сервер вашей организации в своей сети, чтобы сохранить данные в тайне.
chat-parity-restart = Начни сначала.
chat-parity-retry = Повторить
chat-parity-row-cost = Стоимость
chat-parity-row-docs = Документы
chat-parity-row-needs = Тебе нужно
chat-parity-row-ocr = Текст в изображениях
chat-parity-row-offline = Без Интернета
chat-parity-row-photos = Фотографии и диаграммы
chat-parity-row-privacy = Твоё сообщение
chat-parity-row-web = Поиск в Интернете
chat-parity-rule = Правило
chat-parity-show-me = Посмотрим, как включить.
chat-parity-stay-here = Остаётся на этом компьютере
chat-parity-step = Вопрос в % от 2
chat-parity-sub-acp = Подписка на агента
chat-parity-sub-free = Готовы к использованию
chat-parity-sub-local = Частные и офлайновые
chat-parity-sub-own = Ваша организация
chat-parity-suggested = Предлагаемые
chat-parity-system-memory = Система памяти
chat-parity-verdict-bad-gpu = Нет соответствующей карточки графики
chat-parity-verdict-bad-gpu-long = Не было найдено графической карты NVIDIA с CUDA. Местные модели работали бы на процессоре гораздо медленнее: один ответ может занять минуты. Чтобы сохранить ваш текст в сети, предпочитайте сервер вашей организации.
chat-parity-verdict-capacity = Эта карта работает с моделями, которые нуждаются в до %v видео памяти.
chat-parity-verdict-good = Удобно
chat-parity-verdict-low-vram = Карта с слишком малой памятью (% v)
chat-parity-verdict-low-vram-long = Карта имеет %v видео памяти; наименьшая местная модель нуждается в %m.
chat-parity-verdict-mid = Крепко, но полезно
chat-parity-verdict-none = Этот компьютер не был изучен
chat-parity-video-memory = Видеозапись
chat-parity-where = Выберите двигатель
chat-parity-where-acp = Агент (АКТ)
chat-parity-where-free = Свободная модель
chat-parity-where-local = На этом компьютере
chat-parity-where-own = Собственный сервер или ключ
chat-parity-where-sub = Сравните четыре способа использования ассистента. Нажмите на один, чтобы увидеть детали и включить их.
chat-parity-works = Работы
chat-parity-workshop = Помощник по вопросам проведения практикума
chat-parity-workshop-lead = Скажи, где и как думает твой помощник.
chat-parity-workshop-title = Помощник по вопросам проведения практикума
chat-parity-graphics-card = Карточка графических данных

# --- #2921 missing keys added to every locale catalogue ---
chat-confirm-title = Подтвердить действие
chat-confirm-unavailable = Подтверждение отсутствует
chat-copy = Копировать
chat-delete = Удалить
chat-delete-message-confirm = Удалите это сообщение из разговора?
chat-delete-title = Удалить сообщение
chat-edit = Правка
chat-edit-unavailable = Композитор отсутствует
chat-message-actions = Действия сообщений
chat-regenerate = Режиссировать
chat-regenerate-no-source = Нет предыдущего сообщения пользователя для регенерации
chat-regenerate-unavailable = Не может регенерировать этот поворот
chat-voice-capture-silent = Нет звука
chat-voice-duplex-audio = Звук
chat-voice-duplex-closed = Голосовое заседание закрыто
chat-voice-duplex-connect = Подключить голос
chat-voice-duplex-connected = Голос подключен
chat-voice-duplex-disabled = Голос отключен
chat-voice-duplex-error = Ошибка голоса
chat-voice-duplex-fallback = Отклонение к тексту
chat-voice-duplex-live = Живой голос
chat-voice-duplex-lost = Потеря связи
chat-voice-duplex-timeout = Время Голосовой сессии
chat-voice-duplex-unavailable = Голос отсутствует
chat-voice-mic-failed = Микрофон отсутствует
chat-web-title = Поиск в Интернете
voice-mode-connecting = Подключение...
voice-mode-fallback-idle = Проход
voice-mode-fallback-listening = Слушание
voice-mode-fallback-speaking = Говорит
voice-mode-fallback-thinking = Мысли
voice-mode-live-listening = Слушание
voice-mode-live-ready = Готов
voice-mode-live-speaking = Говорит

# gbgov assistant parity: busy-send and actionable engine refusals (#2563)
chat-busy-send-head = Ответ все еще написан.
chat-busy-send-text = Ждите ответа, или остановите его, и отправьте сообщение.
chat-busy-send-wait = Продолжай ждать
chat-busy-send-stop = Остановись и отправь
chat-engine-error-not-linked = Ваш счет маркировки % не связан. Выберите другую модель или соедините счет в параметрах.
chat-engine-error-switch-off = % маркировка требует включения агента в этом разговоре.
chat-engine-error-unavailable = На этом компьютере нет подписных агентов. Выберите другую модель.
chat-engine-error-model-unavailable = Модель % не доступна на маркировке %. Выберите другую модель.
chat-engine-error-workspace-failed = Виртуальная машина агента не могла быть запущена. Проверьте ее состояние и попробуйте снова.
chat-engine-error-account-model-off = Модель % не может быть использована для указания %. Выберите другую модель.
chat-engine-error-free-unavailable = Свободная модель сейчас недоступна. Попробуйте позже или выберите другую модель.
chat-engine-error-limit-reached = Вы достигли предела использования вашего плана на% маркировке. %, когда выбираете другую модель.
chat-engine-error-unsafe-version = Эта версия указания % (%-версия) была заблокирована, потому что она не спрашивала разрешения перед действием. Выберите другую модель.
chat-engine-error-choose-model = Выберите другую модель
chat-engine-error-back-at = Он возвращается в %, когда.

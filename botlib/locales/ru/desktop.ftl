# General Bots — desktop shell and setup wizard (#2089)

desktop-document-title = Настольный стол для общих ботов
sidebar-toggle = Боковой блок@ info: whatsthis
sidebar-collapsed-chat = Чат
sidebar-collapsed-terminal = Терминал
sidebar-collapsed-drive = Поехали.
sidebar-collapsed-settings = Параметры
sidebar-collapsed-account = Счет
setup-welcome-title = Добро пожаловать в генерал Ботс
setup-welcome-subtitle = Давайте подготовимся к нескольким быстрым шагам.
setup-welcome-quick-subtitle = Два быстрых шага, чтобы начать
setup-step-llm = ЛЛМ поставщик
setup-step-profile = Профиль пользователя
setup-step-bot = Ботинок Конфиг
setup-step-training = Подготовка кадров
setup-step-ai-model = AI Модель
setup-step-your-name = Имя
setup-choose-provider-title = Выберите своего поставщика LLM
setup-choose-provider-subtitle = Выберите провайдер модели IA для вашего бота
setup-provider-openai-desc = Модели GPT-4, GPT-3.5
setup-provider-nvidia-desc = NVIDIA ЗАКЛЮЧЕНИЕ NIM
setup-provider-anthropic-desc = Клод 3, модели Клод 3.5
setup-provider-local-desc = Лама.
setup-profile-title = Что лучше всего описывает тебя?
setup-profile-subtitle = Выберите тип профиля для индивидуализированных по умолчанию
setup-profile-developer = Разработчик
setup-profile-developer-desc = Создать и развертывать боты, написать сценарии BASIC
setup-profile-business = Бизнес
setup-profile-business-desc = Автоматизировать рабочие процессы, управлять УИК
setup-profile-support = Поддержка
setup-profile-support-desc = Обслуживание клиентов, управление билетами
setup-bot-title = Настроить ботинок
setup-bot-subtitle = Назовите имя, цель и факультативный шаблон
setup-bot-name = Имя бота
setup-bot-name-placeholder = Мой помощник
setup-bot-purpose = Цель бота
setup-bot-purpose-placeholder = Опишите, что должен сделать ваш бобот...
setup-bot-template = Модель бота
setup-optional = (факультативно)
setup-template-scratch = - Начнём с нуля
setup-name-title = Как нам тебя называть?
setup-name-subtitle = Это имя появляется в ваших разговорах
setup-name-label = Имя
setup-name-placeholder = Например, Мария
setup-training-title = Тренируйся в боте
setup-training-subtitle = Вы можете получить доступ к .pdf, .xls, .doc для начальной подготовки. Вы можете изменить его позже.
setup-drop-text = Сбрасывать файлы здесь или щелчок для выбора
setup-drop-hint = pdf, .xls, .doc — до 10 МВт каждый
setup-select-files = Выбрать файлы
setup-file-status-pending = Нерассмотренные
setup-file-status-uploading = Загрузка
setup-file-status-done = Сделано
setup-file-status-error = Ошибка
setup-saving = Спасение...
setup-complete-message = Добро пожаловать в генерал Ботс.
setup-failed = Ошибка установки:{error}
setup-unknown-error = Неизвестная ошибка
setup-back = Назад
setup-next = Следующий
setup-finish = Заканчивать
setup-skip = Пропустить монтаж
agent-all-apps = Все приложения

# --- Desktop shell panels: versions, accessibility, sidebar, collaboration,
# --- updates, modals and usage (suite/js/*.js) — #2095

versions-component = Компонент
versions-version = Версия
versions-status = Статус
versions-action = Меры
versions-up-to-date = Обновленная информация
versions-unknown = Неизвестно
versions-check-update = Проверить обновление
versions-checking = Проверяю...
versions-checked = Проверено
versions-error = Ошибка
versions-update-available = Обновление:{ $version }
versions-could-not-load = Невозможно загрузить информацию о версии.
versions-ensure-backend = Убедись, что сервер работает.
versions-retry = Повторить

a11y-chat-conversation = Говорить
a11y-chat-messages = Сообщения для разговора
a11y-send-message = Отправить сообщение
a11y-type-message = Напечатайте сообщение
a11y-keyboard-shortcuts = Короткие отрубы клавишной доски
a11y-shortcut-focus-chat = Ввод в фокус-чат
a11y-shortcut-show-help = Показать помощь
a11y-shortcut-escape = Закрытая система/четкий выбор
a11y-shortcut-command-palette = Командная палитра
a11y-shortcut-privacy = Параметры конфиденциальности
a11y-shortcut-scroll-history = Прокрутить историю разговоров
a11y-shortcut-navigate = Навигация между элементами
a11y-shortcut-activate = Активировать фокусный элемент
a11y-shortcut-delete-files = Удалить выбранные файлы (сбросить)
a11y-shortcut-select-all = Выберите все файлы (сбросить)
a11y-shortcut-clipboard = Копировать/вырезать/вставить файлы
a11y-shortcut-navigate-up = Навигационная система
a11y-shortcut-rename = Переименовать выбранный файл

sidebar-locked-conversations = Закрытые разговоры
sidebar-no-conversations = Пока никаких разговоров.
sidebar-sign-in-conversations = Подпишитесь, чтобы увидеть ваши разговоры.
sidebar-history-unavailable = История отсутствует
sidebar-back-to-conversations = ▪ Возвращайтесь к разговорам
sidebar-no-locked-conversations = Никаких закрытых разговоров
sidebar-locked-conversation = Закрытая беседа
# #2095 — conversation-history time buckets.
sidebar-bucket-today = Сегодня
sidebar-bucket-yesterday = Вчера
sidebar-bucket-previous-7-days = Предыдущие 7 дней
sidebar-bucket-older = Старше

sidebar-sign-in = Подпишитесь
sidebar-add-pin = Добавить булавку

collab-collaborators = Сотрудничающие ({ $count })
collab-no-one-else = Здесь больше никого нет.
collab-unavailable = Отсутствие сотрудничества
collab-retry = Повторить

update-available-title = Обновление: v{ $version }
update-now = Обновить
update-later = Позже
update-skip-version = Пропустить эту версию
update-platform = Платформа:{ $name }
update-asset = Активы:{ $name }
update-check-failed = Ошибка проверки обновления:{ $error }
update-view-release-notes = Вид Заметки для выпуска
update-release-notes-title = Выпуск записей - v{ $version }
update-no-release-notes = Записей о выпуске нет.

modal-alert-title = Тревога
modal-input-title = Ввод

external-save-to = Сохранить{ $target }

usage-title = Использование и стоимость
usage-last-days = Последний{ $days }дней
usage-loading = Пользование загрузкой...
usage-per-model = По каждой модели
usage-per-day = В день
usage-model = Модель
usage-tokens = Токены
usage-messages = Сообщения
usage-cost = Стоимость
usage-day = День
usage-days-7 = 7 дней
usage-days-30 = 30 дней
usage-days-90 = 90 дней

pin-close = Закрыть
pin-length-hint = 4-8 цифр
pin-cancel = Отмена
pin-ok = Хорошо.

# --- Collaboration surfaces shared by every suite app: share dialog, activity
# --- trail, conflict banner and version history (suite/js/collab-*.js) — #2095

role-owner = владелец
role-editor = Редактор
role-commenter = Комментарий
role-viewer = Просмотр

share-request-failed = Запрос не удовлетворен ({ $status })
share-warning-title = Общие связи пока не достигнуты
share-warning-body = Эта установка доступна только на локальном приводе, так что здесь нигде не открывается ссылка. Установите одну из этих первых: включить P2P в параметры β Network, или точки Share Links в вашем собственном домене.
share-open-shared-links = Открыть общие ссылки
share-open-network = Открытая сеть
share-invite-people = Приглашать людей
share-anyone-with-link = Кто-нибудь с связным
share-create-link = Создать ссылку
share-transfer-ownership = Передача права собственности
share-transfer = Передача
share-new-owner-placeholder = новый адрес электронной почты владельца
share-owner-suffix = - владелец
share-no-people = Пока никто не добавил.
share-no-links = Пока нет связи.
share-copied = Копированная
share-revoke = Отмена
share-enter-email = Введите email
share-enter-new-owner = Введите email нового владельца
share-needs-base = Установите P2P или свой собственный домен до создания общей связи.

activity-title = Вид деятельности
activity-request-failed = Запрос не удовлетворен ({ $status })
activity-load-more = Загружай больше
activity-loading = Загрузка...
activity-empty = Пока не было деятельности
activity-action-create = Создан
activity-action-edit = Отредактировано
activity-action-comment = замечания по поводу
activity-action-delete = исключить
activity-action-resolve = принял решение по замечаниям относительно
activity-action-reopen = вновь подтвердил комментарий по вопросу о
activity-action-reaction = Ответ на
activity-action-share = Совместное использование
activity-action-restore = восстановлено
activity-action-transfer = Передача

conflict-title = Коллизионные редактирования
conflict-dismiss = Расторжение
conflict-edited-concurrently = Отредактировано одновременно ({ $op })
conflict-keep-server = Сохранить серверную версию
conflict-keep-mine = Оставь мне сдачу.
conflict-not-started = Наблюдение за конфликтами не началось
conflict-request-failed = Запрос не удовлетворен ({ $status })

versions-title = История версии
versions-loading = Загрузка...
versions-empty = Пока нет вариантов — первое спасение создает одно.
versions-version-n = Версия{ $n }
versions-current-tag = (текущий)
versions-name = Имя...
versions-restore-failed = Не удалось восстановить:{ $error }
versions-rename-failed = Не удалось переименовать:{ $error }
versions-restore-dialog-title = Версия восстановления
versions-restore-confirm = Будет создана новая нынешняя версия — ничто не теряется.
versions-name-dialog-title = Вариант имени
versions-name-prompt = Название этого варианта (например, "v2 - утвержденное"):
versions-request-failed = Запрос не удовлетворен ({ $status })

# --- Desktop shell: snap picker, workspace tabs, virtual desktops, context
# --- menu, git panel, browser panel, widgets and data export (suite/js/*.js)

snap-title = "Стоп"{ $app }"
snap-layout-top-left = Верхний левый
snap-layout-top = Верхняя половина
snap-layout-top-right = Верхний правый
snap-layout-left = Левую половину
snap-layout-full = Максимальное
snap-layout-right = Правая половина
snap-layout-bottom-left = Слева внизу
snap-layout-bottom = Нижняя половина
snap-layout-bottom-right = Внизу справа
snap-layout-third-left = Третья
snap-layout-two-thirds = Две трети
snap-layout-third-right = Третья

wstabs-close-tab = Закрыть вкладку
wstabs-new-tab = Новая вкладка рабочего пространства
wstabs-default-name = Рабочее пространство{ $n }
wstabs-default-main = Главное
wstabs-rename-title = Переименовать рабочее пространство
wstabs-name-prompt = Название рабочего пространства:

vdesk-default-name = Рабочий стол{ $n }
vdesk-title = Виртуальные рабочие столы
vdesk-new = Новый рабочий стол
vdesk-rename-title = Переименовать рабочий стол
vdesk-name-prompt = Название рабочего стола:
vdesk-restore-title = Повторное заседание
vdesk-restore-body = Вскрыть{ $count }окно(а) с вашего последнего сеанса на этом рабочем столе?
vdesk-restore-body-short = Восстановить ваш последний сеанс ({ $count } окно(а))?

ctxmenu-search = Поиск уток Go
ctxmenu-search-prompt = Поиск Уток Go:
ctxmenu-wikipedia = Определение Википедии
ctxmenu-wikipedia-prompt = Определение Википедии:

git-loading-status = Статус загрузки...
git-no-changes = Никаких изменений
git-new-branch-prompt = Новое название ветки:
git-loading-diff = Загрузка...
git-diff-failed = Не удалось загрузить diff
git-commit-message-required = Введите сообщение об обязательствах
git-pushing = Толкается...
git-committed = Успешное выполнение и продвижение
git-failed = Ошибка операции Git

browser-session-active = Просмотр{ $id }Активно.
browser-init-failed = Не удалось инициализировать браузер
browser-init-first = Пожалуйста, сначала инициализировать браузер
browser-capture = Захват
browser-no-session = Активной сессии не проводилось
browser-export-failed = Не удалось экспортировать
browser-no-actions = Никаких зарегистрированных действий не зарегистрировано

widget-load-failed = Виджет не загрузился
widget-no-source = Нет строения источника виджета
widget-open-windowed = Открытое окно
widget-pin-launcher = Рычаг до пусковой установки

dataexport-title = Экспорт данных и резервное копирование
dataexport-desc = Экспортировать все ваши данные, включая историю чата, память, метаданные о движении и настройки, в качестве резервного файла JSON.
dataexport-export-all = Экспортировать все данные
dataexport-import = Импортировать резервную копию
dataexport-exported = Успешно экспортируется резервная система
dataexport-too-large = Отклонили резервную копию: файл больше 4 МБ
dataexport-invalid = Неверный файл резервного копирования:{ $error }
dataexport-nothing = Это резервное копирование не содержит никаких приятных настройок
dataexport-unknown-date = неизвестной даты
dataexport-restore-confirm = Это восстановит{ $count }регулировка (регулировки) из резервного(ых){ $date }. Полномочия и сессионные жетоны никогда не импортируются.
dataexport-restored = Ремонт резервного копирования ({ $count }).
dataexport-rejected = { $count }позиция (вписи), отклоненные (отклоненные) списком разрешений.
# #2095 — backup-import validation details shown inside
# `dataexport-invalid`.
dataexport-error-not-object = Файл не является резервным объектом
dataexport-error-version = неподдерживаемая резервная версия
dataexport-error-date = - запасное копирование не имеет действительной даты экспорта
dataexport-error-section = неожиданное отделение:{ $section }
dataexport-reloading = Перезагрузка...

comments-title = Замечания
comments-request-failed = Запрос не удовлетворен ({ $status })
comments-loading = Загрузка комментариев...
comments-load-failed = Не удалось загрузить комментарии:{ $error }
comments-empty-open = Нет открытых комментариев. Используйте @ имени, чтобы упомянуть кого-то.
comments-empty-resolved = Никаких комментариев. Используйте @ имени, чтобы упомянуть кого-то.
comments-resolved-chip = Постановлено
comments-resolve = Постановляет
comments-reopen = Повторное открытие
comments-resolved-announce = Комментарий решен
comments-reopened-announce = Повторное изложение замечания
comments-resolve-failed = Ошибка решения:{ $error }
comments-added = Замечание добавлено
comments-post = Должность
comments-post-failed = Не удалось прокомментировать:{ $error }
comments-reaction-failed = Не удалось дать ответ:{ $error }
comments-delete-failed = Не удалось исключить:{ $error }
comments-reply-failed = Ответ не был получен:{ $error }
comments-delete-title = Удалить комментарий
comments-delete-confirm = Удалить этот комментарий?
comments-reply-title = Ответ
comments-reply-prompt = Ответ на вопрос:
comments-filter-open = Открыто
comments-filter-all = Все
comments-filter-resolved = Постановлено
comments-input-placeholder = Добавить комментарий... использовать @ имя для ссылки
comments-someone = Кто-то
comments-mention-announce = Тебя упомянули в комментарии{ $author }

# semantic i18n key (#2542)
user-menu-help-support = & Поддержка

# semantic i18n key (#2542)
user-menu-sign-out = Подпишитесь.

# semantic i18n key (#2542)
desktop-session-management = Управление сессиями

# semantic i18n key (#2542)
desktop-sessions-hint = Активные дистанционные сеансы настольных компьютеров по всем VMM

# semantic i18n key (#2542)
desktop-active-sessions = Активные сессии

# semantic i18n key (#2542)
desktop-session-idle = Проход

# semantic i18n key (#2542)
desktop-session-state = Состояние

# semantic i18n key (#2542)
desktop-session-duration = Продолжительность

# semantic i18n key (#2542)
desktop-no-active-sessions = Никаких активных сеансов.

# semantic i18n key (#2542)
setup-provider-nvidia-title = НВИДИА

# semantic i18n key (#2542)
setup-bot-config-desc = Назови имя своего бота и опиши его цель.

# semantic i18n key (#2542)
setup-media-features = Освещение в средствах массовой информации

# semantic i18n key (#2542)
git-commit-placeholder = Сообщение (Ctrl+Inter to dection)

# --- #2921 missing keys added to every locale catalogue ---
collab-typing-one = { $name }Напечатает...
collab-typing-two = { $first }и{ $second }- Напечатает...
collab-typing-many = { $name }и{ $count }другие печатают...

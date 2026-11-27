# =============================================================================
# General Bots — en — Drive and files
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Drive / Files
# -----------------------------------------------------------------------------
drive-title = Поехали.
drive-upload = Загрузка
drive-new-folder = Новая папка
drive-empty = Загрузите что-нибудь!
drive-search = Поиск файлов...
drive-sort-name = Имя
drive-sort-date = Дата
drive-sort-size = Размер
drive-sort-type = Тип
drive-view-grid = Вид сетки
drive-view-list = Список
drive-selected = - Я не знаю.{ $count ->
    [one] { $count }выбранный пункт
    *[other] { $count }Отобранные позиции
    }- Я не знаю.
drive-file-size = - Я не знаю.{ $size ->
    [bytes] { $value }B
    [kb] { $value }КБ
    [mb] { $value }БМ
    [gb] { $value }ГБ
    *[other] { $value }байт
    }- Я не знаю.
drive-drop-files = Сбросить файлы здесь, чтобы загрузить

# Drive translations
drive-my-drive = Мой путь
drive-shared = Совместно со мной
drive-recent = Последние
drive-starred = Звездный
drive-trash = Мусор
drive-loading-storage = Загрузка хранилища...
drive-storage-used = { $used }В процентах{ $total }использованные
drive-empty-folder = Эта папка пуста

# Drive → Use in chat (#2568)
drive-use-in-chat = Использовать в чатах
drive-use-in-chat-preparing = Подготовка %n...
drive-use-in-chat-attached = Присоединен к разговору: %n
drive-use-in-chat-failed = Невозможно прикрепить файл к разговору.
drive-use-in-chat-timeout = Откройте чат, чтобы прикрепить файл.
# Share intake (PWA share target)
drive-share-intake-title = Сохранить общие файлы для управления
drive-share-intake-intro = Проверьте все, что есть ниже, и сохраните их на вашем Драйвере.
drive-share-intake-heading = Общие пункты
drive-share-intake-loading = Загрузка совместно используемых предметов...
drive-share-intake-auth-hint = Если сохранённый не сработает, подпишитесь на General Bots и попробуйте снова. Общие файлы хранятся в этом браузере временно до сохранения или очистки.
drive-share-intake-save = Сохранить для управления
drive-share-intake-open-drive = Открыть двигатель
drive-share-intake-no-shared = Не было найдено никаких общих файлов. Вы можете открыть Драйв и загрузить вручную.
drive-share-intake-no-files = Никакого обмена файлами не проводилось.
drive-share-intake-text-shown = Приведенный выше общий текст/ссылка.
drive-share-intake-unavailable = отсутствует
drive-share-intake-read-error = Не удалось прочитать общие пункты.
drive-share-intake-read-failed = Невозможно прочитать общие файлы. Открыть Драйв для загрузки вручную.
drive-share-intake-sign-in-required = Подпишитесь, чтобы сохранить файлы для управления.
drive-share-intake-uploading = Загрузка
drive-share-intake-saved = Сохранили файлы для Драйв.
drive-share-intake-saved-partial = Некоторые файлы не могут быть сохранены. Вы можете попробовать еще раз.

# semantic i18n key (#2542)
drive-my-files = Мои файлы

# semantic i18n key (#2542)
drive-loading-files = Загрузка файлов...

# semantic i18n key (#2542)
drive-drop-hint = Сбрасывать файлы или папки здесь для загрузки

# semantic i18n key (#2542)
drive-search-all-placeholder = Поиск всех файлов...
drive-search-all-aria-label = Поиск всех файлов по имени и содержанию

# -----------------------------------------------------------------------------
# #2328 — loading / empty / error state copy
# -----------------------------------------------------------------------------
drive-starred-empty-title = Нет звездных файлов
drive-starred-empty-desc = Посмотрите любой файл или используйте его меню β, чтобы держать его в одном щелчке.
drive-shared-empty-title = Нет общих файлов
drive-shared-empty-desc = Файлы, которыми вы делитесь, появляются здесь.
drive-list-error-title = Этот список не мог быть загружен
drive-list-error-desc = Не отвечает во времени. Проверьте связь и попробуйте снова.

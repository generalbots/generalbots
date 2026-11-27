# =============================================================================
# General Bots - Authentication Translations (English)
# =============================================================================
# Authentication, Passkey/WebAuthn, and security interface translations
# =============================================================================

# -----------------------------------------------------------------------------
# Authentication General
# -----------------------------------------------------------------------------
auth-title = Аутентификация
auth-login = Запустить
auth-logout = Выход
auth-signup = Подпишитесь
auth-welcome = Добро пожаловать.
auth-welcome-back = С возвращением,{ $name }- Я не знаю.
auth-session-timeout = Время проведения сессии в{ $minutes }минут

# -----------------------------------------------------------------------------
# Login Form
# -----------------------------------------------------------------------------
auth-login-title = Подпишитесь на ваш счет.
auth-login-subtitle = Введите свои полномочия, чтобы продолжить
auth-login-email = Адрес электронной почты
auth-login-username = Имя пользователя
auth-login-password = Пароль
auth-login-remember = Помни меня
auth-login-forgot = Пароль забыл?
auth-login-submit = Подписать
auth-login-loading = Подписываюсь...
auth-login-or = или продолжать
auth-login-no-account = У тебя нет счёта?
auth-login-create-account = Создать счет

# -----------------------------------------------------------------------------
# Passkey/WebAuthn
# -----------------------------------------------------------------------------
passkey-title = Ключи
passkey-subtitle = Защищенное удостоверение подлинности без пароля
passkey-description = Для обеспечения безопасности и устойчивости к фишингу в ключах используются биометрические данные вашего устройства или PIN.
passkey-what-is = Что такое ключ?
passkey-benefits = Выгоды от ключей
passkey-benefit-secure = Более надежны, чем пароли
passkey-benefit-easy = Легко использовать - нет паролей, чтобы помнить
passkey-benefit-fast = Быстрое включение с биометрическими характеристиками
passkey-benefit-phishing = Непрерывные попытки заглушить нападения

# -----------------------------------------------------------------------------
# Passkey Registration
# -----------------------------------------------------------------------------
passkey-register-title = Настроить ключ
passkey-register-subtitle = Создать ключ для более быстрого и безопасного ввода
passkey-register-description = Ваша машина будет просить вас проверить вашу личность с помощью вашего отпечатка, лица или замка экрана
passkey-register-button = Создать ключ
passkey-register-name = Имя ключа
passkey-register-name-placeholder = Например, MacBook Pro, iPhone
passkey-register-name-hint = Назови имя своего парня, чтобы потом его опознать.
passkey-register-loading = Подготовить ключ...
passkey-register-verifying = Проверяю с помощью устройства...
passkey-register-success = Ключ, созданный успешно
passkey-register-error = Не удалось создать ключ
passkey-register-cancelled = Отмена установки на ключе
passkey-register-not-supported = Ваш браузер не поддерживает ключи

# -----------------------------------------------------------------------------
# Passkey Authentication
# -----------------------------------------------------------------------------
passkey-login-title = Подпишитесь с пропуском
passkey-login-subtitle = Используйте ключ для защищенного, без пароля значка
passkey-login-button = Подпишитесь с пропуском
passkey-login-loading = Подтверждаю...
passkey-login-verifying = Проверка ключа...
passkey-login-success = Успешное подписание
passkey-login-error = Ошибка аутентизации
passkey-login-cancelled = Отмена аутентификации
passkey-login-no-passkeys = Для этого счета не найдено ключей.
passkey-login-try-another = Попробуй другой метод

# -----------------------------------------------------------------------------
# Passkey Management
# -----------------------------------------------------------------------------
passkey-manage-title = Управление ключами
passkey-manage-subtitle = Просмотр и управление вашими зарегистрированными ключами
passkey-manage-count = - Я не знаю.{ $count ->
    [one] { $count }Количество зарегистрированных ключей
    *[other] { $count }Количество зарегистрированных ключей
    }- Я не знаю.
passkey-manage-add = Добавить новый ключ
passkey-manage-rename = Переименовать
passkey-manage-delete = Удалить
passkey-manage-created = Создан{ $date }
passkey-manage-last-used = Последний раз используется{ $date }
passkey-manage-never-used = Никогда не использовался
passkey-manage-this-device = Это устройство
passkey-manage-cross-platform = Перекрестная плазма
passkey-manage-platform = Аутентич Платформы
passkey-manage-security-key = Ключ безопасности
passkey-manage-empty = Никаких зарегистрированных ключей
passkey-manage-empty-description = Добавить ключ для более быстрого и безопасного ввода

# -----------------------------------------------------------------------------
# Passkey Deletion
# -----------------------------------------------------------------------------
passkey-delete-title = Удалить ключ
passkey-delete-confirm = Вы уверены, что хотите удалить этот ключ?
passkey-delete-warning = Ты больше не сможешь использовать этот ключ, чтобы подписаться.
passkey-delete-last-warning = Это ваш единственный ключ. Вам нужно использовать пароль аутентификации после удаления.
passkey-delete-success = Удаление ключа успешно
passkey-delete-error = Не удалось удалить ключ

# -----------------------------------------------------------------------------
# Password Fallback
# -----------------------------------------------------------------------------
passkey-fallback-title = Вместо этого использовать пароль
passkey-fallback-description = Если вы не можете использовать свой ключ, вы можете подписаться со своим паролем
passkey-fallback-button = Использовать пароль
passkey-fallback-or-passkey = Или подпиши с парикмахерской.
passkey-fallback-setup-prompt = Установите ключ для более быстрого приема в следующий раз
passkey-fallback-setup-later = Может, позже.
passkey-fallback-setup-now = Готовьтесь.
passkey-fallback-locked = Временно заблокированный счет
passkey-fallback-locked-description = Слишком много неудачных попыток. Попробуйте еще раз.{ $minutes }минут.
passkey-fallback-attempts = { $remaining }Оставшиеся попытки

# -----------------------------------------------------------------------------
# Multi-Factor Authentication
# -----------------------------------------------------------------------------
mfa-title = Двухфактная аутентификация
mfa-subtitle = Добавить дополнительный слой безопасности к вашему счету
mfa-enabled = Версия двухфакторов включена
mfa-disabled = Отключено двухфакторное удостоверение
mfa-enable = Включить 2FA
mfa-disable = Отключить 2FA
mfa-setup = Подключить 2FA
mfa-verify = Проверить код
mfa-code = Код проверки
mfa-code-placeholder = Введите шестизначный код
mfa-code-sent = Код отправлен в{ $destination }
mfa-code-expired = Код истек
mfa-code-invalid = Неверный код
mfa-resend = Код изменения
mfa-resend-in = Перенос{ $seconds }с
mfa-methods = Методы аутентизации
mfa-method-app = Аутогентное приложение
mfa-method-sms = СМС
mfa-method-email = Электронная почта
mfa-method-passkey = Ключ
mfa-backup-codes = Резервные коды
mfa-backup-codes-description = Сохранить эти коды в безопасном месте. Каждый код может использоваться только один раз.
mfa-backup-codes-remaining = { $count }Оставшиеся резервные коды
mfa-backup-codes-generate = Создать новые коды
mfa-backup-codes-download = Загрузка кодов
mfa-backup-codes-copy = Копировать коды

# -----------------------------------------------------------------------------
# Password Management
# -----------------------------------------------------------------------------
password-title = Пароль
password-change = Сменить пароль
password-current = Текущий пароль
password-new = Новый пароль
password-confirm = Подтвердить новый пароль
password-requirements = Требования к паролю
password-requirement-length = По крайней мере{ $length }символы
password-requirement-uppercase = По крайней мере одна буква в верхнем ящике
password-requirement-lowercase = По крайней мере, одна нижная буква
password-requirement-number = По крайней мере один номер
password-requirement-special = По крайней мере один особый символ
password-strength = Прочность пароля
password-strength-weak = Слабость
password-strength-fair = Справедливость
password-strength-good = Хорошо.
password-strength-strong = Сильные
password-match = Пароли совпадают
password-mismatch = Пароли не совпадают
password-changed = Пароль успешно изменился
password-change-error = Не удалось изменить пароль

# -----------------------------------------------------------------------------
# Password Reset
# -----------------------------------------------------------------------------
password-reset-title = Перезагрузить пароль
password-reset-subtitle = Введите свой email для получения ссылки на перезагрузку
password-reset-email-sent = Перезагрузить пароль отправлено письмо
password-reset-email-sent-description = Проверьте свой email для инструкций по перезагрузке пароля
password-reset-invalid-token = Неверная или просроченная связь с перезагрузкой
password-reset-success = Успешное восстановление пароля
password-reset-error = Не удалось перезапустить пароль

# -----------------------------------------------------------------------------
# Session Management
# -----------------------------------------------------------------------------
session-title = Активные сессии
session-subtitle = Регулирование активных сеансов через устройства
session-current = Текущая сессия
session-device = Устройство
session-location = Место проведения
session-last-active = Последний актив
session-ip-address = IP-адрес
session-browser = Просмотр
session-os = Оперативная система
session-sign-out = Подписать
session-sign-out-all = Подпишите все другие сессии
session-sign-out-confirm = Вы уверены, что хотите подписаться на этом сеансе?
session-sign-out-all-confirm = Вы уверены, что хотите подписаться на всех других сеансах?

# -----------------------------------------------------------------------------
# Security Settings
# -----------------------------------------------------------------------------
security-title = Безопасность
security-subtitle = Управлять системами защиты вашего счета
security-overview = Обзор положения в области безопасности
security-last-login = Последний знак
security-password-last-changed = Пароль Последний изменён
security-security-checkup = Проверка безопасности
security-checkup-description = Просмотр настройки безопасности
security-recommendation = Рекомендация
security-add-passkey = Добавить ключ для более надежной регистрации
security-enable-mfa = Включить двухфакторную аутентификацию
security-update-password = Регулярно обновлять пароль

# -----------------------------------------------------------------------------
# Error Messages
# -----------------------------------------------------------------------------
auth-error-invalid-credentials = Неверный адрес электронной почты или пароль
auth-error-account-locked = Счет закрыт. Пожалуйста, свяжитесь с поддержкой.
auth-error-account-disabled = Счет был отключен
auth-error-email-not-verified = Проверьте адрес электронной почты
auth-error-too-many-attempts = Слишком много неудачных попыток. Пожалуйста, попробуйте позже.
auth-error-network = Ошибка сети. Проверьте связь.
auth-error-server = Ошибка сервера. Попробуйте позже.
auth-error-unknown = Неизвестная ошибка
auth-error-session-invalid = Неверный сеанс. Пожалуйста, подпишитесь еще раз.
auth-error-token-expired = У вас закончился сеанс.
auth-error-unauthorized = Вы не уполномочены совершать это действие

# -----------------------------------------------------------------------------
# Success Messages
# -----------------------------------------------------------------------------
auth-success-login = Успешное подписание
auth-success-logout = Выписано успешно
auth-success-signup = Успешно создан счет
auth-success-password-changed = Пароль успешно изменился
auth-success-email-verified = Электронная почта успешно проверялась
auth-success-mfa-enabled = Включено двухфакторное удостоверение подлинности
auth-success-mfa-disabled = Отключено двухфакторное удостоверение подлинности
auth-success-session-terminated = Сессия успешно завершена

# -----------------------------------------------------------------------------
# Notifications
# -----------------------------------------------------------------------------
auth-notify-new-login = Новая запись из{ $device }В{ $location }
auth-notify-password-changed = Ваш пароль был изменен
auth-notify-mfa-enabled = Версия двухфакторов была включена
auth-notify-passkey-added = Новый ключ был добавлен к вашему счету
auth-notify-suspicious-activity = Подозрительная активность, обнаруженная на вашем счету

# =============================================================================
# General Bots — en — Sheets grid accessibility (#2493)
# English is the source of truth: every key must be mirrored in all locales.
# Placeholders ({value}, {range}, ...) are preserved by the FTL loader.
# =============================================================================

# Grid semantics
sheet-a11y-grid-label = Сетка распределителя
sheet-a11y-rowgroup-label = Границы сетки

# Formula bar and name box
sheet-a11y-formula-bar-label = Панель формулы
sheet-a11y-name-box-label = Имя, введите ссылку на ячейку
sheet-a11y-name-box-placeholder = A1
sheet-a11y-invalid-reference = Неверная ссылка на ячейку:{value}

# Cell labels and live announcements
sheet-a11y-cell-label = Ячейка{address}::{value}
sheet-a11y-cell-formula-label = Формула{formula}
sheet-a11y-live-selection = {range},{rows}ряды{cols}колонки, сводная сумма{sum}
sheet-a11y-live-edited = Ячейка{address}Обновленные{value}
sheet-a11y-clear-too-large = Выбор слишком большой, чтобы расчистить клавиатуру

# Accessibility checker
sheet-a11y-checker-title = Проверка доступности
sheet-a11y-checker-close = Проверка доступности
sheet-a11y-checker-run = Проверить доступность
sheet-a11y-checker-summary = {count}Выявленные проблемы доступности
sheet-a11y-checker-empty = Проблемы с доступностью не найдены.
sheet-a11y-checker-sheet = Спецификация
sheet-a11y-checker-sheet-fallback = Спецификация{index}
sheet-a11y-severity-warning = Внимание
sheet-a11y-finding-merged = Слияние клеток в{range}может путать считывающих экраны; избегать объединения в доступные рабочие книги.
sheet-a11y-finding-color-only = Условный формат в{range}- только цвет; добавить значок, знак или шаблон.
sheet-a11y-finding-no-alt = Фото{name}не имеет альтернативного текста.
sheet-a11y-unnamed-image = без имени
sheet-a11y-finding-no-header = Рабочий лист{sheet}имеет данные, но не имеет заголовков.

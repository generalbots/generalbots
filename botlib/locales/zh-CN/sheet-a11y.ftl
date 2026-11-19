# =============================================================================
# General Bots — zh-CN — Sheets grid accessibility (#2493)
# English is the source of truth: every key must be mirrored in all locales.
# Placeholders ({value}, {range}, ...) are preserved by the FTL loader.
# =============================================================================

# Grid semantics
sheet-a11y-grid-label = 电子表格网格
sheet-a11y-rowgroup-label = 网格行

# Formula bar and name box
sheet-a11y-formula-bar-label = 公式栏
sheet-a11y-name-box-label = 名称框，输入单元格引用
sheet-a11y-name-box-placeholder = A1
sheet-a11y-invalid-reference = 无效的单元格引用：{value}

# Cell labels and live announcements
sheet-a11y-cell-label = 单元格 {address}: {value}
sheet-a11y-cell-formula-label = 公式 {formula}
sheet-a11y-live-selection = {range}，{rows} 行 × {cols} 列，求和 {sum}
sheet-a11y-live-edited = 单元格 {address} 已更新为 {value}
sheet-a11y-clear-too-large = 选区太大，无法用键盘清除

# Accessibility checker
sheet-a11y-checker-title = 辅助功能检查
sheet-a11y-checker-close = 关闭辅助功能检查
sheet-a11y-checker-run = 检查辅助功能
sheet-a11y-checker-summary = 发现 {count} 个辅助功能问题
sheet-a11y-checker-empty = 未发现辅助功能问题。
sheet-a11y-checker-sheet = 工作表
sheet-a11y-checker-sheet-fallback = 工作表 {index}
sheet-a11y-severity-warning = 警告
sheet-a11y-finding-merged = {range} 处的合并单元格可能使屏幕阅读器混淆；请避免在无障碍工作簿中合并单元格。
sheet-a11y-finding-color-only = {range} 处的条件格式仅使用颜色；请添加图标、标签或图案。
sheet-a11y-finding-no-alt = 图片 {name} 没有替代文本。
sheet-a11y-unnamed-image = 未命名
sheet-a11y-finding-no-header = 工作表 {sheet} 有数据但没有标题行。

# =============================================================================
# General Bots — ja — Billing, products and tickets
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Analytics (additional UI keys)
# -----------------------------------------------------------------------------
analytics-dashboard-title = 分析ダッシュボード
analytics-last-hour = 最後の1時間
analytics-last-6h = 過去6時間
analytics-last-24h = 過去 24 時間
analytics-last-7d = 過去 7 日間
analytics-last-30d = 過去 30 日間

# =============================================================================
# Billing
# =============================================================================
# -----------------------------------------------------------------------------
# Billing Navigation & General
# -----------------------------------------------------------------------------
billing-title = 請求
billing-invoices = 請求書
billing-payments = 支払い
billing-quotes = 引用
billing-dashboard = ダッシュボード

# -----------------------------------------------------------------------------
# Billing Entities
# -----------------------------------------------------------------------------
billing-invoice = 請求書
billing-invoice-desc = 顧客への請求書
billing-payment = お支払い
billing-payment-desc = 支払いを受け取りました
billing-quote = 引用
billing-quote-desc = 価格見積

# -----------------------------------------------------------------------------
# Billing Status
# -----------------------------------------------------------------------------
billing-due-date = 期限
billing-overdue = 期限を過ぎました
billing-paid = 有料
billing-pending = 保留中
billing-draft = 草案
billing-sent = 送信済み
billing-partial = 部分的
billing-cancelled = キャンセルされました

# -----------------------------------------------------------------------------
# Billing Actions
# -----------------------------------------------------------------------------
billing-new-invoice = 新しい請求書
billing-new-quote = 新しい見積書
billing-new-payment = 新規支払い
billing-send-invoice = 請求書の送信
billing-record-payment = 支払いの記録
billing-mark-paid = 有料としてマークする
billing-void = ボイド

# -----------------------------------------------------------------------------
# Billing Fields
# -----------------------------------------------------------------------------
billing-amount = 金額
billing-tax = 税金
billing-subtotal = 小計
billing-total = 合計
billing-discount = 割引
billing-line-items = 品目
billing-add-item = アイテムの追加
billing-remove-item = アイテムの削除
billing-customer = お客様
billing-issue-date = 発行日
billing-payment-terms = 支払い条件
billing-notes = 注意事項
billing-invoice-number = 請求書番号
billing-quote-number = 見積書番号

# -----------------------------------------------------------------------------
# Billing Reports
# -----------------------------------------------------------------------------
billing-revenue = 収益
billing-outstanding = 優れた
billing-this-month = 今月
billing-last-month = 先月
billing-total-paid = 支払総額
billing-total-overdue = 延滞合計
billing-subtitle = 請求書、支払い、見積書
billing-revenue-month = 今月の収益
billing-total-revenue = 総収益
billing-paid-month = 今月支払い済み

# -----------------------------------------------------------------------------
# Billing Empty States
# -----------------------------------------------------------------------------
billing-no-invoices = 請求書が見つかりません
billing-no-payments = 支払いが見つかりませんでした
billing-no-quotes = 引用符が見つかりませんでした

# =============================================================================
# Products
# =============================================================================
# -----------------------------------------------------------------------------
# Products Navigation & General
# -----------------------------------------------------------------------------
products-title = 製品
products-catalog = カタログ
products-services = サービス
products-price-lists = 価格表
products-inventory = 在庫

# -----------------------------------------------------------------------------
# Products Entities
# -----------------------------------------------------------------------------
products-product = 製品
products-product-desc = 物理的またはデジタル製品
products-service = サービス
products-service-desc = サービス内容
products-price-list = 価格表
products-price-list-desc = 価格帯

# -----------------------------------------------------------------------------
# Products Actions
# -----------------------------------------------------------------------------
products-new-product = 新製品
products-new-service = 新サービス
products-new-price-list = 新しい価格表
products-new-pricelist = 新しい価格表
products-edit-product = 製品の編集
products-duplicate = 重複

# -----------------------------------------------------------------------------
# Products Fields
# -----------------------------------------------------------------------------
products-sku = SKU
products-category = カテゴリ
products-price = 価格
products-unit = ユニット
products-stock = 在庫
products-cost = コスト
products-margin = マージン
products-barcode = バーコード

# -----------------------------------------------------------------------------
# Products Status
# -----------------------------------------------------------------------------
products-in-stock = 在庫あり
products-out-of-stock = 在庫切れ
products-low-stock = 在庫僅少
products-active = アクティブ
products-inactive = 非アクティブ
products-featured = 注目の
products-archived = アーカイブ済み

# -----------------------------------------------------------------------------
# Products Stats & Metrics
# -----------------------------------------------------------------------------
products-subtitle = 製品、サービス、価格を管理する
products-items = 製品
products-pricelists = 価格表
products-total-products = 総製品数
products-total-services = トータルサービス

# -----------------------------------------------------------------------------
# Products Empty States
# -----------------------------------------------------------------------------
products-no-products = 製品が見つかりませんでした
products-no-services = サービスが見つかりませんでした
products-no-price-lists = 価格表が見つかりません

# =============================================================================
# Tickets (Support Cases)
# =============================================================================
# -----------------------------------------------------------------------------
# Tickets Navigation & General
# -----------------------------------------------------------------------------
tickets-title = チケット
tickets-cases = 事例
tickets-open = 開く
tickets-closed = 閉店
tickets-all = すべてのチケット
tickets-my-tickets = 私のチケット

# -----------------------------------------------------------------------------
# Tickets Entities
# -----------------------------------------------------------------------------
tickets-case = ケース
tickets-case-desc = サポートチケット
tickets-resolution = 解像度
tickets-resolution-desc = AIが提案するソリューション

# -----------------------------------------------------------------------------
# Tickets Priority
# -----------------------------------------------------------------------------
tickets-priority = 優先順位
tickets-priority-low = 低い
tickets-priority-medium = 中
tickets-priority-high = 高
tickets-priority-urgent = 緊急

# -----------------------------------------------------------------------------
# Tickets Status
# -----------------------------------------------------------------------------
tickets-status = ステータス
tickets-status-new = 新しい
tickets-status-open = 開く
tickets-status-pending = 保留中
tickets-status-resolved = 解決済み
tickets-status-closed = 閉店
tickets-status-on-hold = 保留中

# -----------------------------------------------------------------------------
# Tickets Actions
# -----------------------------------------------------------------------------
tickets-new-ticket = 新しいチケット
tickets-assign = 割り当てる
tickets-reassign = 再割り当て
tickets-escalate = エスカレーション
tickets-resolve = 解決する
tickets-reopen = 再開
tickets-close = 閉じる
tickets-merge = マージ

# -----------------------------------------------------------------------------
# Tickets Fields
# -----------------------------------------------------------------------------
tickets-subject = 件名
tickets-description = 説明
tickets-category = カテゴリ
tickets-assigned = 担当者
tickets-unassigned = 未割り当て
tickets-created = 作成されました
tickets-updated = 更新されました
tickets-response-time = 応答時間
tickets-resolution-time = 解決時間
tickets-customer = お客様
tickets-internal-notes = 内部メモ
tickets-attachments = 添付ファイル

# -----------------------------------------------------------------------------
# Tickets AI Features
# -----------------------------------------------------------------------------
tickets-ai-suggestion = AIによる提案
tickets-apply-suggestion = 提案を適用する
tickets-ai-summary = AIの概要
tickets-similar-tickets = 類似のチケット
tickets-suggested-articles = おすすめの記事

# -----------------------------------------------------------------------------
# Tickets Empty States
# -----------------------------------------------------------------------------
tickets-no-tickets = チケットが見つかりませんでした
tickets-no-open = オープンチケットはありません
tickets-no-closed = 終了したチケットはありません

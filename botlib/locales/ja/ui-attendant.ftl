# =============================================================================
# General Bots — ja — Attendant (support queue)
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
attendant-crm-disabled = CRM機能が有効になっていません
attendant-crm-disabled-desc = アテンダント コンソールでは、このボットに対して CRM 機能を有効にする必要があります。これにより、人間のエージェントがボットから転送された会話を受信して​​応答できるようになります。
attendant-crm-enable-instruction = CRM 機能を有効にするには、ボットの
attendant-crm-config-file = config.csv
attendant-crm-create-attendant = 次に、
attendant-crm-attendant-file = アテンダント.csv
attendant-crm-configure-team = チームを構成するファイル

# -----------------------------------------------------------------------------
# Queue Sidebar
# -----------------------------------------------------------------------------
attendant-title = アテンダントコンソール
attendant-status-online = オンライン
attendant-status-busy = 忙しい
attendant-status-away = 離れて
attendant-status-offline = オフライン
attendant-status-ready = オンライン - 会話の準備完了
attendant-status-busy-msg = 忙しい - 会話の処理
attendant-status-away-msg = 離れています - すぐに戻ってきます
attendant-status-offline-msg = オフライン - 利用できません

# -----------------------------------------------------------------------------
# Queue Stats
# -----------------------------------------------------------------------------
attendant-waiting = 待っています
attendant-active = アクティブ
attendant-resolved = 解決済み
attendant-mine = 私のもの

# -----------------------------------------------------------------------------
# Queue Filters
# -----------------------------------------------------------------------------
attendant-filter-all = すべて
attendant-filter-waiting = 待っています
attendant-filter-mine = 私のもの
attendant-filter-priority = 優先順位

# -----------------------------------------------------------------------------
# Conversation List
# -----------------------------------------------------------------------------
attendant-no-conversations = キューに会話はありません
attendant-new-conversations-appear = 新しい会話がここに表示されます
attendant-unread = 未読
attendant-typing = タイピング中...
attendant-select-conversation = 会話を選択してください
attendant-select-conversation-desc = キューから会話を選択して応答を開始します

# -----------------------------------------------------------------------------
# Channel Tags
# -----------------------------------------------------------------------------
attendant-channel-whatsapp = ワッツアップ
attendant-channel-teams = チーム
attendant-channel-instagram = インスタグラム
attendant-channel-web = ウェブ
attendant-channel-telegram = 電報
attendant-channel-email = 電子メール

# -----------------------------------------------------------------------------
# Priority Tags
# -----------------------------------------------------------------------------
attendant-priority-urgent = 緊急
attendant-priority-high = 高
attendant-priority-normal = ノーマル

# -----------------------------------------------------------------------------
# Chat Area
# -----------------------------------------------------------------------------
attendant-message-placeholder = メッセージを入力してください...
attendant-send = 送信
attendant-attach-file = ファイルを添付する
attendant-insert-emoji = 絵文字の挿入
attendant-quick-responses = 素早い対応
attendant-transfer = 転送
attendant-resolve = 解決する
attendant-more-actions = さらなるアクション

# -----------------------------------------------------------------------------
# Quick Responses
# -----------------------------------------------------------------------------
attendant-quick-greeting = こんにちは！今日はどのようにお手伝いできますか?
attendant-quick-thanks = ご理解いただきありがとうございます。
attendant-quick-checking = それを確認させてください。
attendant-quick-moment = ちょっとお待ちください。

# -----------------------------------------------------------------------------
# Transfer Modal
# -----------------------------------------------------------------------------
attendant-transfer-title = 転送の会話
attendant-transfer-to = 転送先
attendant-transfer-reason = 理由（任意）
attendant-transfer-reason-placeholder = なぜこの会話を転送するのですか?
attendant-transfer-cancel = キャンセル
attendant-transfer-confirm = 転送

# -----------------------------------------------------------------------------
# AI Insights Sidebar
# -----------------------------------------------------------------------------
attendant-ai-insights = AI の洞察
attendant-ai-summary = 会話の要約
attendant-ai-sentiment = 顧客感情
attendant-sentiment-positive = ポジティブ
attendant-sentiment-neutral = ニュートラル
attendant-sentiment-negative = ネガティブ
attendant-smart-replies = スマート リプライ
attendant-confidence = 自信
attendant-source = ソース

# -----------------------------------------------------------------------------
# Customer Details
# -----------------------------------------------------------------------------
attendant-customer-details = 顧客の詳細
attendant-customer-name = 名前
attendant-customer-email = 電子メール
attendant-customer-phone = 電話
attendant-customer-location = 場所
attendant-customer-tags = タグ

# -----------------------------------------------------------------------------
# Conversation History
# -----------------------------------------------------------------------------
attendant-history = 歴史
attendant-history-resolved = 解決済み
attendant-history-transferred = 転送されました
attendant-history-abandoned = 放棄された
attendant-view-history = 全履歴を表示

# -----------------------------------------------------------------------------
# Toast Messages
# -----------------------------------------------------------------------------
attendant-toast-transferred = 会話は正常に転送されました
attendant-toast-resolved = 会話が解決済みとしてマークされました
attendant-toast-assigned = あなたに割り当てられた会話
attendant-toast-error = エラーが発生しました
attendant-toast-connection-lost = 接続が失われました。再接続中...
attendant-toast-connection-restored = 接続が回復しました

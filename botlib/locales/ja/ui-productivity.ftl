# =============================================================================
# General Bots — ja — Productivity apps (tasks, calendar, meet, mail, research, paper)
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Tasks
# -----------------------------------------------------------------------------
tasks-title = タスク
tasks-new = 新しいタスク
tasks-due-today = 今日が期限
tasks-overdue = 期限を過ぎました
tasks-completed = 完了しました
tasks-all = すべてのタスク
tasks-priority-high = 高優先度
tasks-priority-medium = 中優先度
tasks-priority-low = 低優先度
tasks-no-due-date = 期限なし
tasks-add-subtask = サブタスクの追加
tasks-mark-complete = 完了としてマークする
tasks-mark-incomplete = 未完了としてマークする
tasks-delete-confirm = このタスクを削除してもよろしいですか?
tasks-count = { $count ->
    [zero] No tasks
    [one] { $count } task
   *[other] { $count } tasks
}

# -----------------------------------------------------------------------------
# Calendar
# -----------------------------------------------------------------------------
calendar-title = カレンダー
calendar-today = 今日
calendar-new-event = 新しいイベント
calendar-all-day = 一日中
calendar-repeat = リピート
calendar-reminder = リマインダー
calendar-view-day = 日
calendar-view-week = 週
calendar-view-month = 月
calendar-view-year = 年
calendar-no-events = 予定されているイベントはありません
calendar-event-title = イベントタイトル
calendar-event-location = 場所
calendar-event-description = 説明
calendar-event-attendees = 出席者

# -----------------------------------------------------------------------------
# Meet / Video Conferencing
# -----------------------------------------------------------------------------
meet-title = 会う
meet-join = ミーティングに参加する
meet-start = ミーティングを開始する
meet-mute = ミュート
meet-unmute = ミュートを解除する
meet-video-on = カメラオン
meet-video-off = カメラオフ
meet-share-screen = 画面共有
meet-stop-sharing = 共有を停止する
meet-end-call = 通話を終了する
meet-leave = 会議から退席する
meet-participants = { $count ->
    [one] { $count } participant
   *[other] { $count } participants
}
meet-waiting-room = 待合室
meet-admit = 認める
meet-remove = 削除する
meet-chat = ミーティングチャット
meet-raise-hand = 挙手
meet-lower-hand = 低い手
meet-recording = 録画中
meet-start-recording = 録音を開始する
meet-stop-recording = 録音を停止する

# -----------------------------------------------------------------------------
# Mail / Email
# -----------------------------------------------------------------------------
mail-title = メール
mail-compose = 作曲する
mail-inbox = 受信箱
mail-sent = 送信済み
mail-drafts = 下書き
mail-trash = ゴミ箱
mail-spam = スパムメール
mail-starred = スター付き
mail-archive = アーカイブ
mail-to = に
mail-cc = CC
mail-bcc = BCC
mail-subject = 件名
mail-body = メッセージ
mail-reply = 返信
mail-reply-all = 全員に返信
mail-forward = 進む
mail-send = 送信
mail-discard = 廃棄する
mail-save-draft = ドラフトの保存
mail-attach = ファイルを添付する
mail-unread = { $count ->
    [one] { $count } unread
   *[other] { $count } unread
}
mail-empty-inbox = 受信箱は空です
mail-no-subject = (件名なし)

# -----------------------------------------------------------------------------
# Paper (Document Editor)
# -----------------------------------------------------------------------------
paper-title = 紙
paper-new-note = 新しいメモ
paper-search-notes = メモを検索...
paper-quick-start = クイックスタート
paper-template-blank = 空白
paper-template-meeting = 会議
paper-template-todo = やるべきこと
paper-template-research = 研究
paper-untitled = 無題
paper-placeholder = 書き込みを開始するか、コマンドとして / を入力します...
paper-commands = コマンド
paper-heading1 = 見出し1
paper-heading1-desc = 大きなセクション見出し
paper-heading2 = 見出し2
paper-heading2-desc = 中セクションの見出し
paper-heading3 = 見出し 3
paper-heading3-desc = 小さなセクションの見出し
paper-paragraph = 段落
paper-paragraph-desc = プレーンテキスト
paper-bullet-list = 箇条書きリスト
paper-bullet-list-desc = 順序なしリスト
paper-numbered-list = 番号付きリスト
paper-numbered-list-desc = 順序付きリスト
paper-todo-list = やることリスト
paper-todo-list-desc = 確認可能なタスクリスト
paper-quote = 引用
paper-quote-desc = 引用のための引用文
paper-divider = ディバイダー
paper-divider-desc = 水平線
paper-code-block = コードブロック
paper-code-block-desc = フォーマットされたコード
paper-table = テーブル
paper-table-desc = テーブルの挿入
paper-image = 画像
paper-image-desc = URLから画像を挿入
paper-callout = 吹き出し
paper-callout-desc = ハイライト表示された情報ボックス
paper-ai-write = AI書き込み
paper-ai-write-desc = AIでテキストを生成
paper-ai-summarize = AIサマリー
paper-ai-summarize-desc = 選択したテキストを要約する
paper-ai-expand = AI拡張
paper-ai-expand-desc = 選択したテキストを展開する
paper-ai-improve = AIの改善
paper-ai-improve-desc = 文章の質を向上させる
paper-ai-translate = AI翻訳
paper-ai-translate-desc = 別の言語に翻訳する
paper-ai-assistant = AIアシスタント
paper-ai-quick-actions = クイックアクション
paper-ai-rewrite = リライト
paper-ai-make-shorter = 短くする
paper-ai-make-longer = 長くする
paper-ai-fix-grammar = 文法を修正する
paper-ai-tone = トーン
paper-ai-tone-professional = プロフェッショナル
paper-ai-tone-casual = カジュアル
paper-ai-tone-friendly = フレンドリー
paper-ai-tone-formal = フォーマル
paper-ai-translate-to = に翻訳する
paper-ai-custom-prompt = カスタムプロンプト
paper-ai-custom-placeholder = 欲しいものを説明してください...
paper-ai-generate = 生成する
paper-ai-response = AIの対応
paper-ai-apply = 申し込む
paper-ai-regenerate = 再生する
paper-ai-copy = コピー
paper-word-count = { $count } 単語
paper-char-count = { $count } 文字
paper-saved = 保存されました
paper-saving = 保存中...
paper-last-edited = 最終編集日: { $time }
paper-last-edited-now = 最終編集日: たった今
paper-export = ドキュメントのエクスポート
paper-export-pdf = PDF
paper-export-docx = ワード (.docx)
paper-export-markdown = マークダウン
paper-export-html = HTML
paper-export-txt = プレーンテキスト

# Tasks translations
tasks-active = アクティブなインテント
tasks-awaiting = 決定待ち
tasks-paused = 一時停止中
tasks-blocked = ブロック/問題
tasks-time-saved = 保存されたアクティブ時間:
tasks-input-placeholder = 何をしたいですか?例: 「CRM アプリを作成する」または「明日ジョンに電話するようリマインドする」

# Calendar additional translations
calendar-my-calendars = 私のカレンダー

# Email additional translations
email-scheduled = 予定されている
email-tracking = 追跡

# Email folder translations
email-inbox = 受信箱
email-starred = スター付き
email-sent = 送信済み
email-drafts = 下書き
email-spam = スパムメール
email-trash = ゴミ箱
email-compose = 作曲する

# -----------------------------------------------------------------------------
# Research
# -----------------------------------------------------------------------------
research-title = 研究
research-search-placeholder = 何でも聞いてください...
research-collections = コレクション
research-new-collection = 新しいコレクション
research-recent = 最近の
research-academic = アカデミック
research-code = コード
research-internal = 内部
research-search-all = すべてを検索
research-academic-papers = 学術論文
research-code-docs = コードとドキュメント
research-internal-kb = 社内ナレッジベース
research-sources = 情報源
research-trending = トレンド
research-pro-search = プロ検索
research-include-images = 画像を含める
research-try-asking = について聞いてみてください
research-related = 関連する質問
research-view-all-sources = すべてのソースを表示
research-export-citations = 引用のエクスポート
research-save-to-collection = コレクションに保存

# -----------------------------------------------------------------------------
# Meet (additional keys)
# -----------------------------------------------------------------------------
meet-new-meeting = 新しい会議
meet-join-meeting = ミーティングに参加する
meet-active-rooms = アクティブルーム
meet-room-title = 会議室
meet-record = 録画
meet-camera = カメラ
meet-share = シェアする
meet-info = 情報
meet-more = もっと見る
meet-share-meeting = シェアミーティング
meet-meeting-title = 会議のタイトル
meet-meeting-code = 会議コード
meet-meeting-link = ミーティングリンク
meet-send-invite = 招待を送信する

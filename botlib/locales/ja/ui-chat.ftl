# =============================================================================
# General Bots — ja — Chat
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Chat
# -----------------------------------------------------------------------------
chat-title = チャット
chat-placeholder = メッセージを入力してください...
chat-send = 送信
chat-new-conversation = 新しい会話
chat-history = チャット履歴
chat-clear = チャットをクリアする
chat-export = チャットのエクスポート
chat-typing = { $name } が入力中です...
chat-online = オンライン
chat-offline = オフライン
chat-last-seen = 最後に見たもの { $time }
chat-mention-title = 参照エンティティ
chat-mention-placeholder = メッセージ... (メンションするには @ を入力してください)
chat-mention-search = エンティティを検索...
chat-mention-no-results = 結果が見つかりませんでした
chat-mention-type-hint = 入力: 検索するには

# Additional Chat translations
chat-camera = カメラ
chat-camera-title = 写真を撮る
chat-camera-capture = 撮影
chat-camera-cancel = キャンセル
chat-camera-sent = 写真を送信しました。
chat-camera-denied = カメラの許可が拒否されました。写真を撮るには、システム設定でデスクトップアプリのカメラアクセスを許可してください。
chat-camera-missing = このデバイスにカメラが見つかりません。
chat-camera-error = カメラを起動できませんでした。
chat-camera-unsupported = このデスクトップアプリはカメラへのアクセスを提供していません。
voice-mode-title = 音声会話
voice-mode-listening = 聞き取り中… 自然に話してください
voice-mode-thinking = 考え中…
voice-mode-speaking = 話しています…
voice-mode-idle = 音声会話
voice-mode-end = 音声会話を終了
chat-message-placeholder = メッセージ...

# Chat message controls and notices
chat-copy-message = メッセージをコピー
chat-copied = コピーしました
chat-message-copied = メッセージをコピーしました
chat-copy-failed = コピーに失敗しました
chat-listen-tts = 読み上げ (TTS)
chat-listen-message = このメッセージを読み上げる
chat-image-unavailable = 画像を利用できません
chat-open = 開く
chat-expand = 展開
chat-collapse = 折りたたむ
chat-preview = プレビュー
chat-close-preview = プレビューを閉じる
chat-preview-load-failed = プレビューを読み込めませんでした。
chat-show-more = もっと見る
chat-show-less = 表示を減らす
chat-tool = ツール
chat-param = パラメータ
chat-params = パラメータ
chat-tool-result = ツール結果
chat-retry = 再試行
chat-open-as-document = ドキュメントとして開く
chat-show-more-history = さらに表示…
chat-disconnected-reconnecting = チャットが切断されました。再接続中...
chat-connect-retrying = チャットサーバーに接続できません。再試行中...
chat-connection-lost = 接続が失われました。再接続を試みています...
chat-attachment-sent = 添付ファイルを送信しました: 
chat-attachments-sent = 添付ファイルを送信しました: 
chat-message-not-delivered = メッセージが配信されませんでした — 再接続中
chat-attachment-read-failed = 添付ファイルを読み取れませんでした。もう一度添付してください。
chat-attachment-queued = まもなく接続が戻ります — 添付ファイルをキューに追加しました
chat-reconnecting-queued = 再接続中 — 接続が戻るとメッセージが送信されます。
chat-bot-private-signin = このボットは非公開です — 続行するにはサインインしてください
chat-approved-plan = 承認されたプラン: 
chat-retrying-attachment = 添付ファイルを再試行中
chat-expanded-preview = 拡大プレビュー

# #2362 — chat code/artifact control labels
chat-code-show-all = Show all
chat-code-lines = lines
chat-artifact-save-drive = Save to Drive
chat-artifact-download-svg = Download SVG
chat-artifact-download-png = Download PNG
chat-artifact-open-canvas = Open in Canvas
chat-artifact-folder = Folder (optional)
chat-artifact-kind-markdown = Markdown document
chat-artifact-kind-html = HTML page
chat-artifact-kind-code = Code
chat-artifact-words = words

# #2362 — attachment action label
chat-open-in-app = Open in app
# Assistant-look chat: transcript export and answer feedback
chat-export-actions = 会話の操作
chat-export-md = 会話をMarkdownでエクスポート
chat-export-print = 印刷またはPDFとして保存
chat-export-done = 会話をエクスポートしました
chat-export-empty = まだエクスポートする内容がありません
chat-export-untitled = 会話
chat-export-user = あなた
chat-export-assistant = アシスタント
chat-feedback-label = 回答への評価
chat-feedback-good = 良い回答
chat-feedback-bad = 良くない回答
chat-feedback-comment = どこを改善できますか？
chat-feedback-send = 送信
chat-staged-files = 添付ファイル
chat-staged-close = プレビューを閉じる
chat-camera-waiting = Waiting for the camera to start…
chat-camera-waiting-frames = Permission granted; waiting for video frames
chat-camera-ready = Camera ready. Take the photo or cancel.
chat-camera-started = Camera started — tap Take photo.
chat-camera-no-device = no video device
chat-camera-device = video device
chat-camera-no-frames = The camera preview did not start. Choose a photo from the system picker instead.
chat-camera-timeout = The camera did not answer. Choose a photo from the system picker instead.

# Chat engineer status strip (#2342)
chat-status-bot = ボット
chat-status-session = セッション
chat-status-model = モデル
chat-status-state = 状態
chat-status-no-session = —
chat-status-copy-session = セッションIDをコピー
chat-status-copied = コピーしました
chat-status-idle = 待機中
chat-status-streaming = ストリーミング
chat-status-thinking = 思考中
chat-status-error = エラー
chat-status-stopped = 停止

chat-response-ready = 応答の準備ができました
chat-response-started = アシスタントが応答しています

# semantic i18n key (#2542)
chat-no-suggestions-available = 利用可能な提案はありません

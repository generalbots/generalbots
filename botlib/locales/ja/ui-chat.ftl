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
chat-consent-affected-count = 影響を受けるレコード
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
chat-tool-result-ready = ツールの結果を利用できます

# semantic i18n key (#2542)
chat-no-suggestions-available = 利用可能な提案はありません

# semantic i18n key (#2542)
chat-start-conversation = 会話を開始

# semantic i18n key (#2542)
chat-empty-hint = 下にメッセージを入力するか音声入力を使用してください

# semantic i18n key (#2542)
chat-no-conversations-yet = まだ会話はありません

# semantic i18n key (#2542)
chat-context-label = コンテキスト:

# semantic i18n key (#2542)
chat-select-context = コンテキストを選択...

# semantic i18n key (#2542)
chat-no-contexts-configured = 設定されたコンテキストはありません

# semantic i18n key (#2542)
projector-title = コンテンツビューア

# semantic i18n key (#2542)
projector-loading-content = コンテンツを読み込み中...

# semantic i18n key (#2542)
projector-slide-info = スライド 1 / 1

# semantic i18n key (#2542)
projector-code-info = code.rs

# Projector viewer control (#2542)
chat-projector-rotate = 回転

# -----------------------------------------------------------------------------
# #2328 — loading / empty / error state copy
# -----------------------------------------------------------------------------
chat-empty-title = 会話を始めましょう
chat-empty-desc = 下にメッセージを入力するか、ボットの候補から選んでください。
chat-empty-action = 最初のメッセージを書く

# -----------------------------------------------------------------------------
# #2342 — chat engineer UX: slash commands, help, inspect/copy-markdown
# -----------------------------------------------------------------------------
chat-cmd-menu-title = Commands
chat-cmd-help = Show keyboard shortcuts
chat-cmd-new = Start a new conversation
chat-cmd-clear = Clear this conversation
chat-cmd-stop = Stop generating
chat-stop = 停止
chat-cmd-failed = Command failed
chat-help = Keyboard shortcuts
chat-inspect = Inspect
chat-inspect-title = Message details
chat-inspect-sender = Sender
chat-inspect-id = Message id
chat-inspect-no-id = none
chat-inspect-length = Characters
chat-copy-markdown = Copy as Markdown
chat-copied-markdown = Copied as Markdown

# Model attribution on each bot answer
chat-model-written-by = 回答を作成したモデル:

# Assistant parity in chat
assistant-parity-block-allow = 許可する
assistant-parity-block-allowed = 許可しました
assistant-parity-block-api-command = API コマンド
assistant-parity-block-attachment-read = 添付ファイルを読み取り
assistant-parity-block-chars = 文字
assistant-parity-block-command = コマンド
assistant-parity-block-decision-failed = 決定を記録できませんでした
assistant-parity-block-denied = 拒否しました
assistant-parity-block-deny = 拒否する
assistant-parity-block-detail = 詳細
assistant-parity-block-drive-list = ドライブ一覧
assistant-parity-block-drive-read = ドライブのファイルを読み取り
assistant-parity-block-drive-search = ドライブ検索
assistant-parity-block-elapsed = 経過時間
assistant-parity-block-error = エラー
assistant-parity-block-file = ファイル
assistant-parity-block-kb-search = ナレッジ検索
assistant-parity-block-notice = お知らせ
assistant-parity-block-path = パス
assistant-parity-block-permission = 権限を要求しました
assistant-parity-block-query = クエリ
assistant-parity-block-read = 読み取り
assistant-parity-block-reason = 理由
assistant-parity-block-result = 結果
assistant-parity-block-retry = 再試行
assistant-parity-block-seconds = 秒
assistant-parity-block-size = サイズ
assistant-parity-block-status-done = 完了
assistant-parity-block-status-failed = 失敗
assistant-parity-block-status-needs-you = あなたの操作が必要です
assistant-parity-block-status-running = 実行中
assistant-parity-block-step = 手順
assistant-parity-block-thinking = 考え中
assistant-parity-block-tool = ツール
assistant-parity-block-tool-run = ツール実行
assistant-parity-block-total = 合計
assistant-parity-block-truncated = 切り詰め
assistant-parity-block-type = 種類
assistant-parity-block-yes = はい
chat-parity-attach-title = 添付ファイルのプレビュー
chat-parity-attach-unavailable = 内容を表示できません。ダウンロードして開いてください。
chat-parity-attach-download-failed = 添付ファイルをダウンロードできませんでした。
chat-parity-model-tiers = 自動と段階
chat-parity-model-accounts = プロバイダーアカウント
chat-parity-model-local = ローカルと無料
chat-drive-parity-attaching = 添付中...
chat-drive-parity-cancel = キャンセル
chat-drive-parity-delete = 削除
chat-drive-parity-deleted = 削除しました
chat-drive-parity-delete-failed = ファイルを削除できませんでした。
chat-drive-parity-delete-question = このファイルを削除しますか？
chat-drive-parity-delete-title = ファイルの削除
chat-drive-parity-deleting = 削除中...
chat-drive-parity-download = ダウンロード
chat-drive-parity-download-failed = ファイルをダウンロードできませんでした。
chat-drive-parity-downloading = ダウンロード中...
chat-drive-parity-more = その他の操作
chat-drive-parity-new-name = 新しい名前
chat-drive-parity-preview = プレビュー
chat-drive-parity-preview-failed = ファイルをプレビューできませんでした。
chat-drive-parity-previewing = プレビューを開いています...
chat-drive-parity-rename = 名前を変更
chat-drive-parity-renamed = 名前を変更しました
chat-drive-parity-rename-failed = ファイルの名前を変更できませんでした。
chat-drive-parity-rename-title = ファイル名の変更
chat-drive-parity-renaming = 名前を変更中...
chat-drive-parity-unavailable = この導入ではこの操作は利用できません。
chat-drive-parity-use-in-chat = 会話で使用
chat-parity-agent-does = エージェントが実行します
chat-parity-back = 戻る
chat-parity-checked-now = たった今確認しました
chat-parity-close = 閉じる
chat-parity-coming-soon = 近日公開
chat-parity-cost-contract = 契約によります
chat-parity-cost-free = 無料
chat-parity-cost-free-limit = 無料（利用制限あり）
chat-parity-cost-sub = サブスクリプション
chat-parity-cuda-no = 適切なGPUが見つかりません
chat-parity-cuda-yes = グラフィックカードを検出
chat-parity-depends-agent = エージェントに依存
chat-parity-driver = ドライバー
chat-parity-engine = 使用中のエンジン
chat-parity-feature-docs = ドキュメント
chat-parity-feature-ocr = 画像内の文字
chat-parity-feature-photos = 写真とグラフ
chat-parity-feature-web = ウェブ検索
chat-parity-features = 有効な機能
chat-parity-gains = メリット
chat-parity-go-agent = エージェントの提供者へ
chat-parity-go-ext = 外部プロバイダーへ
chat-parity-go-net = 組織内ネットワークに残る
chat-parity-go-server = 設定したサーバーへ
chat-parity-guide = どれを選べばよいですか？
chat-parity-guide-sub = 2つの質問に答えてください。
chat-parity-in-use = 使用中
chat-parity-load-failed = 情報を読み込めませんでした。
chat-parity-loading = 読み込み中...
chat-parity-machine = このコンピューター
chat-parity-machine-ok = このコンピューター: %d、%v GB
chat-parity-machine-slow = 適切なグラフィックカードがありません。動作は遅くなります
chat-parity-need-acp = エージェントをインストールしてサインインしてください
chat-parity-need-free = 不要
chat-parity-need-local = 6GB以上のグラフィックカードと4〜6GBのダウンロード
chat-parity-need-own = サーバーアドレスとキー
chat-parity-no-read = 読み取りません
chat-parity-no-work = 動作しません
chat-parity-none-found = 見つかりません
chat-parity-not-yet = まだ
chat-parity-only-net = ネットワーク内のサーバーのみ
chat-parity-optional = 任意。オンにすると有効
chat-parity-pick = オプションを選択してください
chat-parity-priv-ext = 会話は外部ホスト（%h）に送信されます。
chat-parity-priv-local = 会話はこのコンピューター内に留まります。
chat-parity-priv-net = 会話は組織のサーバー（%h）に送信されます。
chat-parity-privacy = プライバシー
chat-parity-q1 = 書いた内容はこのコンピューター外に出てもよいですか？
chat-parity-q1-help = 機密情報や個人情報を考えてください。
chat-parity-q1-no = いいえ、ここに残す必要があります
chat-parity-q1-no-desc = ローカル処理のみ許容されます。
chat-parity-q1-yes = はい、外出してもよい
chat-parity-q1-yes-desc = 外部サービスで処理される可能性があります。
chat-parity-q2-no = いいえ、またはわかりません
chat-parity-q2-no-desc = 会社のサーバーやキーがありません。
chat-parity-q2-open = 組織はAIサーバーまたはキーを提供していますか？
chat-parity-q2-open-help = わからなければITに確認してください。
chat-parity-q2-secret = 組織は外部AIを許可していますか？
chat-parity-q2-secret-help = 一部のポリシーでは外部サービスが禁止されています。
chat-parity-q2-secret-no-desc = 外部AIは許可されていません。
chat-parity-q2-secret-yes-desc = 外部AIは許可されています。
chat-parity-q2-yes = はい
chat-parity-q2-yes-desc = 会社のサーバーやキーがあります。
chat-parity-reads = 読み取り
chat-parity-ready = 準備完了
chat-parity-rec-free = 無料モデルから始めましょう。設定不要です。機密性の高い内容はローカルを選んでください。
chat-parity-rec-local = このコンピューターを使ってください。情報は外に出ず、GPUもあります。
chat-parity-rec-local-slow = ローカルモデルに適したGPUがありません。組織内のサーバーを使用してください。
chat-parity-rec-local-tight = ローカルモデルは動作しますが、このGPUでは遅い可能性があります。
chat-parity-rec-own = 組織のサーバーまたはキーを使用してください。コストと制限は契約によります。
chat-parity-rec-own-net = データを守るため、組織内ネットワークのサーバーを使用してください。
chat-parity-restart = 最初からやり直す
chat-parity-retry = 再試行
chat-parity-row-cost = コスト
chat-parity-row-docs = ドキュメント
chat-parity-row-needs = 必要なもの
chat-parity-row-ocr = 画像内の文字
chat-parity-row-offline = オフライン
chat-parity-row-photos = 写真とグラフ
chat-parity-row-privacy = あなたのテキスト
chat-parity-row-web = ウェブ検索
chat-parity-rule = ルール
chat-parity-show-me = 有効化方法を見る
chat-parity-stay-here = このコンピューターに残る
chat-parity-step = 質問 %n / 2
chat-parity-sub-acp = エージェントのサブスクリプション
chat-parity-sub-free = すぐ使える
chat-parity-sub-local = プライベートでオフライン
chat-parity-sub-own = 組織のもの
chat-parity-suggested = 推奨
chat-parity-system-memory = システムメモリ
chat-parity-verdict-bad-gpu = 適切なグラフィックカードなし
chat-parity-verdict-bad-gpu-long = CUDA対応のNVIDIA GPUが見つかりません。ローカルモデルはCPUで動作し、回答に数分かかることがあります。情報をネットワーク内に保つには組織のサーバーをご利用ください。
chat-parity-verdict-capacity = このGPUは最大%vのVRAMを必要とするモデルを実行できます。
chat-parity-verdict-good = 快適
chat-parity-verdict-low-vram = VRAMが不足したGPU（%v）
chat-parity-verdict-low-vram-long = このGPUのVRAMは%vですが、最小のローカルモデルは%m必要です。
chat-parity-verdict-mid = 余裕は少ないが利用可
chat-parity-verdict-none = このコンピューターを確認できませんでした
chat-parity-video-memory = VRAM
chat-parity-where = エンジンを選択
chat-parity-where-acp = エージェント (ACP)
chat-parity-where-free = 無料モデル
chat-parity-where-local = このコンピューター上
chat-parity-where-own = 独自サーバーまたはキー
chat-parity-where-sub = アシスタントの4つの利用方法を比較できます。クリックすると詳細と有効化方法を表示します。
chat-parity-works = 動作します
chat-parity-workshop = アシスタント工房
chat-parity-workshop-lead = アシスタントがどこでどのように考えるかを調整します。
chat-parity-workshop-title = アシスタント工房
chat-parity-graphics-card = グラフィックカード

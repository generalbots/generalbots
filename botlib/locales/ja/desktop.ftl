# General Bots — desktop shell and setup wizard (#2089)

desktop-document-title = General Bots デスクトップ
sidebar-toggle = サイドバーの表示切替
sidebar-collapsed-chat = チャット
sidebar-collapsed-terminal = ターミナル
sidebar-collapsed-drive = Drive
sidebar-collapsed-settings = 設定
sidebar-collapsed-account = アカウント
setup-welcome-title = General Bots へようこそ
setup-welcome-subtitle = いくつかの簡単な手順ですぐに設定できます
setup-welcome-quick-subtitle = 2 つの簡単な手順で開始
setup-step-llm = LLM プロバイダー
setup-step-profile = ユーザープロフィール
setup-step-bot = ボット設定
setup-step-training = トレーニング
setup-step-ai-model = AI モデル
setup-step-your-name = お名前
setup-choose-provider-title = LLM プロバイダーを選択
setup-choose-provider-subtitle = ボットで使用する AI モデルプロバイダーを選択してください
setup-provider-openai-desc = GPT-4、GPT-3.5 モデル
setup-provider-nvidia-desc = NVIDIA NIM 推論
setup-provider-anthropic-desc = Claude 3、Claude 3.5 モデル
setup-provider-local-desc = llama.cpp、Ollama など
setup-profile-title = あなたに最も当てはまるものは?
setup-profile-subtitle = パーソナライズされた既定値のためにプロフィールの種類を選択してください
setup-profile-developer = 開発者
setup-profile-developer-desc = ボットの構築とデプロイ、BASIC スクリプトの作成
setup-profile-business = ビジネス
setup-profile-business-desc = ワークフローの自動化、CRM の管理
setup-profile-support = サポート
setup-profile-support-desc = カスタマーサービス、チケット管理
setup-bot-title = ボットを設定
setup-bot-subtitle = ボットに名前、目的、必要に応じてテンプレートを指定します
setup-bot-name = ボット名
setup-bot-name-placeholder = マイアシスタント
setup-bot-purpose = ボットの目的
setup-bot-purpose-placeholder = ボットに何をさせたいかを説明してください...
setup-bot-template = ボットテンプレート
setup-optional = (任意)
setup-template-scratch = — ゼロから始める —
setup-name-title = 何とお呼びすればよいですか?
setup-name-subtitle = この名前は会話の中に表示されます
setup-name-label = お名前
setup-name-placeholder = 例: Maria
setup-training-title = ボットをトレーニング
setup-training-subtitle = 初回トレーニング用に .pdf、.xls、.doc ファイルをドロップしてください。Drive にアクセスでき、後で変更できます。
setup-drop-text = ここにファイルをドロップするか、クリックして選択
setup-drop-hint = .pdf、.xls、.doc — 各最大 10MB
setup-select-files = ファイルを選択
setup-file-status-pending = 保留中
setup-file-status-uploading = アップロード中
setup-file-status-done = 完了
setup-file-status-error = エラー
setup-saving = 保存しています...
setup-complete-message = 設定が完了しました!General Bots へようこそ。
setup-failed = 設定に失敗しました: {error}
setup-unknown-error = 不明なエラー
setup-back = 戻る
setup-next = 次へ
setup-finish = 完了
setup-skip = 設定をスキップ
agent-all-apps = すべてのアプリ

# --- Desktop shell panels: versions, accessibility, sidebar, collaboration,
# --- updates, modals and usage (suite/js/*.js) — #2095

versions-component = コンポーネント
versions-version = バージョン
versions-status = 状態
versions-action = 操作
versions-up-to-date = 最新
versions-unknown = 不明
versions-check-update = 更新を確認
versions-checking = 確認しています...
versions-checked = 確認済み
versions-error = エラー
versions-update-available = 更新: { $version }
versions-could-not-load = バージョン情報を読み込めませんでした。
versions-ensure-backend = バックエンドサーバーが実行中であることを確認してください。
versions-retry = 再試行

a11y-chat-conversation = チャットの会話
a11y-chat-messages = チャットメッセージ
a11y-send-message = メッセージを送信
a11y-type-message = メッセージを入力
a11y-keyboard-shortcuts = キーボードショートカット
a11y-shortcut-focus-chat = チャット入力にフォーカス
a11y-shortcut-show-help = このヘルプを表示
a11y-shortcut-escape = モーダルを閉じる / 選択を解除
a11y-shortcut-command-palette = コマンドパレット
a11y-shortcut-privacy = プライバシー設定
a11y-shortcut-scroll-history = チャット履歴をスクロール
a11y-shortcut-navigate = 要素間を移動
a11y-shortcut-activate = フォーカス中の要素を実行
a11y-shortcut-delete-files = 選択したファイルを削除 (Drive)
a11y-shortcut-select-all = すべてのファイルを選択 (Drive)
a11y-shortcut-clipboard = ファイルのコピー / 切り取り / 貼り付け
a11y-shortcut-navigate-up = Drive で上に移動
a11y-shortcut-rename = 選択したファイルの名前を変更

sidebar-locked-conversations = ロックされた会話
sidebar-no-conversations = 会話はまだありません
sidebar-sign-in-conversations = サインインすると会話が表示されます
sidebar-history-unavailable = 履歴を利用できません
sidebar-back-to-conversations = ← 会話に戻る
sidebar-no-locked-conversations = ロックされた会話はありません
sidebar-locked-conversation = ロックされた会話
sidebar-sign-in = サインイン
sidebar-add-pin = PIN を追加

collab-collaborators = 共同編集者 ({ $count })
collab-no-one-else = 他の人はここにいません

update-available-title = 更新が利用可能 — v{ $version }
update-now = 今すぐ更新
update-later = 後で
update-skip-version = このバージョンをスキップ
update-platform = プラットフォーム: { $name }
update-asset = アセット: { $name }
update-check-failed = 更新の確認に失敗しました: { $error }
update-view-release-notes = リリースノートを表示
update-release-notes-title = リリースノート — v{ $version }
update-no-release-notes = リリースノートはありません。

modal-alert-title = 警告
modal-input-title = 入力

external-save-to = { $target } に保存

usage-title = 使用量とコスト
usage-last-days = 過去 { $days } 日間
usage-loading = 使用量を読み込んでいます…
usage-per-model = モデル別
usage-per-day = 日別
usage-model = モデル
usage-tokens = トークン
usage-messages = メッセージ
usage-cost = コスト
usage-day = 日
usage-days-7 = 7 日間
usage-days-30 = 30 日間
usage-days-90 = 90 日間

pin-close = 閉じる
pin-length-hint = 4〜8 桁
pin-cancel = キャンセル
pin-ok = OK

# --- Collaboration surfaces shared by every suite app: share dialog, activity
# --- trail, conflict banner and version history (suite/js/collab-*.js) — #2095

role-owner = 所有者
role-editor = 編集者
role-commenter = コメント者
role-viewer = 閲覧者

share-request-failed = リクエストに失敗しました ({ $status })
share-warning-title = 共有リンクはまだ到達可能ではありません
share-warning-body = このインストールは localhost からのみ到達可能なため、ここで作ったリンクはどこにも開きません。まず次のいずれかを設定してください:設定 → ネットワークで P2P を有効にする、または共有リンクを独自ドメインに向けます。
share-open-shared-links = 共有リンクを開く
share-open-network = ネットワークを開く
share-invite-people = メンバーを招待
share-anyone-with-link = リンクを知っている人
share-create-link = リンクを作成
share-transfer-ownership = 所有権を移譲
share-transfer = 移譲
share-new-owner-placeholder = 新しい所有者のメール
share-owner-suffix = · 所有者
share-no-people = まだ誰も追加されていません。
share-no-links = 公開リンクはまだありません。
share-copied = コピーしました
share-revoke = 無効化
share-enter-email = メールアドレスを入力
share-enter-new-owner = 新しい所有者のメールアドレスを入力
share-needs-base = 共有リンクを作成する前に、P2P または独自ドメインを設定してください。

activity-title = アクティビティ
activity-request-failed = リクエストに失敗しました ({ $status })
activity-load-more = さらに読み込む
activity-loading = 読み込んでいます…
activity-empty = アクティビティはまだありません
activity-action-create = が作成しました
activity-action-edit = が編集しました
activity-action-comment = がコメントしました
activity-action-delete = が削除しました
activity-action-resolve = のコメントを解決しました
activity-action-reopen = のコメントを再開しました
activity-action-reaction = がリアクションしました
activity-action-share = が共有しました
activity-action-restore = が復元しました
activity-action-transfer = が移譲しました

conflict-title = 編集の競合
conflict-dismiss = 閉じる
conflict-edited-concurrently = 同時に編集されました ({ $op })
conflict-keep-server = サーバーのバージョンを保持
conflict-keep-mine = 自分の変更を保持
conflict-not-started = 競合監視が開始されていません
conflict-request-failed = リクエストに失敗しました ({ $status })

versions-title = バージョン履歴
versions-loading = 読み込んでいます…
versions-empty = バージョンはまだありません — 最初の保存で作成されます。
versions-version-n = バージョン { $n }
versions-current-tag = (現在)
versions-name = 名前…
versions-restore-failed = 復元に失敗しました: { $error }
versions-rename-failed = 名前の変更に失敗しました: { $error }
versions-restore-dialog-title = バージョンを復元
versions-restore-confirm = このバージョンを復元しますか?新しい現在のバージョンが作成されます — 失われるものはありません。
versions-name-dialog-title = バージョンに名前を付ける
versions-name-prompt = このバージョンに名前を付けてください(例:「v2 — 承認済み」):
versions-request-failed = リクエストに失敗しました ({ $status })

# --- Desktop shell: snap picker, workspace tabs, virtual desktops, context
# --- menu, git panel, browser panel, widgets and data export (suite/js/*.js)

snap-title = 「{ $app }」をスナップ
snap-layout-top-left = 左上
snap-layout-top = 上半分
snap-layout-top-right = 右上
snap-layout-left = 左半分
snap-layout-full = 最大化
snap-layout-right = 右半分
snap-layout-bottom-left = 左下
snap-layout-bottom = 下半分
snap-layout-bottom-right = 右下
snap-layout-third-left = 左 1/3
snap-layout-two-thirds = 2/3
snap-layout-third-right = 右 1/3

wstabs-close-tab = タブを閉じる
wstabs-new-tab = 新しいワークスペースタブ
wstabs-default-name = ワークスペース { $n }
wstabs-default-main = メイン
wstabs-rename-title = ワークスペースの名前を変更
wstabs-name-prompt = ワークスペース名:

vdesk-default-name = デスクトップ { $n }
vdesk-title = 仮想デスクトップ
vdesk-new = 新しいデスクトップ
vdesk-rename-title = デスクトップの名前を変更
vdesk-name-prompt = デスクトップ名:
vdesk-restore-title = セッションを復元
vdesk-restore-body = このデスクトップで前回のセッションの { $count } 個のウィンドウを再度開きますか?
vdesk-restore-body-short = 前回のセッションを復元しますか({ $count } 個のウィンドウ)?

ctxmenu-search = DuckDuckGo で検索
ctxmenu-search-prompt = DuckDuckGo で検索:
ctxmenu-wikipedia = Wikipedia の定義
ctxmenu-wikipedia-prompt = Wikipedia の定義:

git-loading-status = 状態を読み込んでいます...
git-no-changes = 変更なし
git-new-branch-prompt = 新しいブランチ名:
git-loading-diff = diff を読み込んでいます...
git-diff-failed = diff の読み込みに失敗しました
git-commit-message-required = コミットメッセージを入力してください
git-pushing = プッシュしています...
git-committed = コミットとプッシュに成功しました
git-failed = Git 操作に失敗しました

browser-session-active = ブラウザセッション { $id } が有効です。
browser-init-failed = ブラウザの初期化に失敗しました
browser-init-first = 先にブラウザを初期化してください
browser-capture = キャプチャ
browser-no-session = 有効なセッションがありません
browser-export-failed = エクスポートに失敗しました
browser-no-actions = 記録された操作はありません

widget-load-failed = ウィジェットの読み込みに失敗しました
widget-no-source = ウィジェットのソースが設定されていません
widget-open-windowed = ウィンドウで開く
widget-pin-launcher = ランチャーにピン留め

dataexport-title = データのエクスポートとバックアップ
dataexport-desc = チャット履歴、メモリ、Drive のメタデータ、設定を含むすべてのデータを JSON バックアップファイルとしてエクスポートします。
dataexport-export-all = すべてのデータをエクスポート
dataexport-import = バックアップをインポート
dataexport-exported = バックアップをエクスポートしました
dataexport-too-large = バックアップを拒否しました:ファイルが 4 MB を超えています
dataexport-invalid = 無効なバックアップファイル: { $error }
dataexport-nothing = このバックアップには復元可能な設定が含まれていません
dataexport-unknown-date = 不明な日付
dataexport-restore-confirm = { $date } のバックアップから { $count } 件の設定を復元します。資格情報とセッショントークンはインポートされません。続行しますか?
dataexport-restored = バックアップを復元しました({ $count } 件)。
dataexport-rejected = { $count } 件のエントリが許可リストにより拒否されました。
dataexport-reloading = 再読み込みしています...

comments-title = コメント
comments-request-failed = リクエストに失敗しました ({ $status })
comments-loading = コメントを読み込んでいます…
comments-load-failed = コメントの読み込みに失敗しました: { $error }
comments-empty-open = 未解決のコメントはありません。@name でメンションできます。
comments-empty-resolved = 解決済みのコメントはありません。@name でメンションできます。
comments-resolved-chip = 解決済み
comments-resolve = 解決
comments-reopen = 再開
comments-resolved-announce = コメントを解決しました
comments-reopened-announce = コメントを再開しました
comments-resolve-failed = 解決に失敗しました: { $error }
comments-added = コメントを追加しました
comments-post = 投稿
comments-post-failed = コメントに失敗しました: { $error }
comments-reaction-failed = リアクションに失敗しました: { $error }
comments-delete-failed = 削除に失敗しました: { $error }
comments-reply-failed = 返信に失敗しました: { $error }
comments-delete-title = コメントを削除
comments-delete-confirm = このコメントを削除しますか?
comments-reply-title = 返信
comments-reply-prompt = コメントに返信:
comments-filter-open = 未解決
comments-filter-all = すべて
comments-filter-resolved = 解決済み
comments-input-placeholder = コメントを追加… @name でメンション
comments-someone = 誰か
comments-mention-announce = { $author } がコメントであなたをメンションしました

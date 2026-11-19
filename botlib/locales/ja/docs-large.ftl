# Docs 大きな .docx の読み込み経路 (#2533) — ja カタログ。8 言語すべてで同じキーセット。

docs-large-note-local = ローカルのみ — プレビューであり、サーバーの読み込み経路ではありません
docs-large-deferred = 後回し: ストリーミング取り込み、サーバー本文サイズ制限、gzip/brotli 転送、サーバー側 ETag/If-Match 判定、非同期エクスポート、実測ベースライン。
docs-large-open = 大きなドキュメントの読み込み経路 (ローカルプレビュー)
docs-large-title = 大きなドキュメントの読み込み経路
docs-large-close = 閉じる

docs-large-size-label = 模擬サイズ
docs-large-size-5 = 5 MB
docs-large-size-10 = 10 MB
docs-large-size-20 = 20 MB
docs-large-load = フィクスチャを読み込む
docs-large-controls = フィクスチャと保存の操作
docs-large-mode-label = 保存モード
docs-large-mode-auto = 自動
docs-large-mode-manual = 手動
docs-large-toggle-mode = 自動/手動を切り替え
docs-large-etag-label = ETag (If-Match)
docs-large-save-incremental = 変更ブロックを保存
docs-large-save-full = 本文全体を保存
docs-large-bytes-label = 回線バイト数 (推定)
docs-large-bytes-incremental = 増分バイト数: { $n }

docs-large-degradation = 編集状態
docs-large-degraded-banner = 編集は遅くなります — 読み取り専用プレビューで開きました
docs-large-edit-ready = 編集を有効にしました — ドキュメントはしきい値内です
docs-large-enable-anyway = それでも編集を有効にする

docs-large-window-title = ウィンドウ表示編集
docs-large-range-label = マウント範囲
docs-large-scroll-label = スクロール位置
docs-large-window = マウント範囲: { $s }–{ $e } / { $t }
docs-large-scope = contenteditable は表示ウィンドウのみに限定
docs-large-inspector = インスペクター
docs-large-queue-depth = キューの深さ: { $n }

docs-large-baseline-title = ベースライン (ローカル推定)
docs-large-baseline-local = ローカル推定 — ブラウザー/サーバーの実測値ではありません
docs-large-col-size = サイズ
docs-large-col-heap = ヒープ峰值 (推定)
docs-large-col-ttfp = 最初の描画までの時間
docs-large-col-keystroke = キー入力レイテンシ
docs-large-col-autosave = 自動保存の所要時間
docs-large-col-wire = 回線バイト

docs-large-save-ok = { $n } バイト保存しました
docs-large-save-stale = 古い保存を拒否しました — ETag が一致せず、データの損失はありません
docs-large-save-413 = 本文全体の保存には大きすぎます — 増分保存 / 分割を使用してください
docs-large-save-inc-413 = 変更バッチが大きすぎます — 分割してください。損失はありません

docs-large-status-loaded = フィクスチャを読み込みました: { $n } ブロック
docs-large-status-mode = 保存モード: { $s }
docs-large-status-enabled = 編集を有効にしました

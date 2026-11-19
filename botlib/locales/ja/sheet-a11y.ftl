# =============================================================================
# General Bots — ja — Sheets grid accessibility (#2493)
# English is the source of truth: every key must be mirrored in all locales.
# Placeholders ({value}, {range}, ...) are preserved by the FTL loader.
# =============================================================================

# Grid semantics
sheet-a11y-grid-label = スプレッドシートのグリッド
sheet-a11y-rowgroup-label = グリッドの行

# Formula bar and name box
sheet-a11y-formula-bar-label = 数式バー
sheet-a11y-name-box-label = 名前ボックス、セル参照を入力
sheet-a11y-name-box-placeholder = A1
sheet-a11y-invalid-reference = 無効なセル参照です: {value}

# Cell labels and live announcements
sheet-a11y-cell-label = セル {address}: {value}
sheet-a11y-cell-formula-label = 数式 {formula}
sheet-a11y-live-selection = {range}、{rows} 行 x {cols} 列、合計 {sum}
sheet-a11y-live-edited = セル {address} を {value} に更新しました
sheet-a11y-clear-too-large = 選択範囲が大きすぎるため、キーボードで消去できません

# Accessibility checker
sheet-a11y-checker-title = アクセシビリティチェック
sheet-a11y-checker-close = アクセシビリティチェックを閉じる
sheet-a11y-checker-run = アクセシビリティをチェック
sheet-a11y-checker-summary = {count} 件のアクセシビリティの問題が見つかりました
sheet-a11y-checker-empty = アクセシビリティの問題は見つかりませんでした。
sheet-a11y-checker-sheet = シート
sheet-a11y-checker-sheet-fallback = シート {index}
sheet-a11y-severity-warning = 警告
sheet-a11y-finding-merged = {range} の結合セルはスクリーンリーダーを混乱させる可能性があります。アクセシブなブックでは結合を避けてください。
sheet-a11y-finding-color-only = {range} の条件付き書式は色のみを使用しています。アイコン、ラベル、またはパターンを追加してください。
sheet-a11y-finding-no-alt = 画像 {name} に代替テキストがありません。
sheet-a11y-unnamed-image = 無名
sheet-a11y-finding-no-header = ワークシート {sheet} にはデータがありますが、ヘッダー行がありません。

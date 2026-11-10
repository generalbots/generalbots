# =============================================================================
# General Bots — ja — Compliance
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# =============================================================================
# TOOLS / COMPLIANCE SCREENS
# =============================================================================
# -----------------------------------------------------------------------------
# Compliance Main
# -----------------------------------------------------------------------------
compliance-title = APIコンプライアンスレポート
compliance-subtitle = すべてのボットのセキュリティ スキャン - パスワード、脆弱なコード、構成ミスをチェックします。
compliance-export-report = レポートのエクスポート
compliance-run-scan = コンプライアンススキャンの実行
compliance-scanning = スキャン中...

# -----------------------------------------------------------------------------
# Bot Selector
# -----------------------------------------------------------------------------
compliance-all-bots = すべてのボット
compliance-select-bots = ボットの選択

# -----------------------------------------------------------------------------
# Stats Cards
# -----------------------------------------------------------------------------
compliance-critical = クリティカル
compliance-critical-desc = 即時の対応が必要です
compliance-high = 高
compliance-high-desc = セキュリティリスク
compliance-medium = 中
compliance-medium-desc = 対処すべき
compliance-low = 低い
compliance-low-desc = ベストプラクティス
compliance-info = 情報
compliance-info-desc = 情報提供

# -----------------------------------------------------------------------------
# Filters
# -----------------------------------------------------------------------------
compliance-filter-severity = 重大度
compliance-filter-type = 種類
compliance-filter-all-severities = すべての重大度
compliance-filter-all-types = 全種類
compliance-search-issues = 検索の問題...

# -----------------------------------------------------------------------------
# Issue Types
# -----------------------------------------------------------------------------
compliance-type-password = 設定内のパスワード
compliance-type-hardcoded = ハードコードされたシークレット
compliance-type-deprecated = 非推奨のキーワード
compliance-type-fragile = 脆弱なコード
compliance-type-config = 構成の問題

# -----------------------------------------------------------------------------
# Results Table
# -----------------------------------------------------------------------------
compliance-results = 結果
compliance-results-count = { $count ->
    [one] { $count } issue found
   *[other] { $count } issues found
}
compliance-col-severity = 重大度
compliance-col-issue = 問題
compliance-col-location = 場所
compliance-col-details = 詳細
compliance-col-action = アクション
compliance-view-details = 詳細を見る
compliance-fix-issue = 問題を修正する
compliance-ignore = 無視する
compliance-no-issues = 問題は見つかりませんでした
compliance-no-issues-desc = 素晴らしい！ボットは準拠しています。

# -----------------------------------------------------------------------------
# Scan Progress
# -----------------------------------------------------------------------------
compliance-scan-in-progress = スキャン中...
compliance-scan-checking = { $item } を確認中...
compliance-scan-complete = スキャンが完了しました
compliance-scan-failed = スキャンに失敗しました

## Compliance frameworks (enterprise-grade release)
compliance-frameworks = フレームワーク
compliance-new-framework = 新しい
compliance-framework-name = 名前
compliance-framework-version = バージョン
compliance-framework-description = 説明
compliance-create-framework = フレームワークの作成
compliance-controls = コントロール
compliance-add-control = コントロールの追加
compliance-control-id = コントロールID
compliance-control-title = タイトル
compliance-control-category = カテゴリ
compliance-control-description = 説明
compliance-mandatory = 必須
compliance-optional = オプション
compliance-evidence = 証拠
compliance-attach-evidence = 証拠を添付する
compliance-evidence-path = ファイルパス (ドライブアーティファクト)
compliance-evidence-type = 種類
compliance-approve = 承認する
compliance-covered = カバーされた
compliance-no-evidence = 証拠がない
compliance-export-csv = CSVのエクスポート
compliance-archive = アーカイブ
compliance-total-controls = トータルコントロール
compliance-coverage = 適用範囲
compliance-no-frameworks = まだフレームワークが構成されていません。

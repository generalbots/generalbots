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

# semantic i18n key (#2542)
admin-compliance-dashboard-title = コンプライアンスダッシュボード

# semantic i18n key (#2542)
admin-compliance-subtitle = SOC 2 Type II コンプライアンス状況、コントロール監視、監査証跡

# semantic i18n key (#2542)
admin-compliance-compliant = 準拠

# semantic i18n key (#2542)
admin-compliance-overall-score = 総合コンプライアンススコア

# semantic i18n key (#2542)
admin-compliance-controls-passing = 合格コントロール

# semantic i18n key (#2542)
admin-compliance-evidence-items-due = 期限の証跡項目

# semantic i18n key (#2542)
admin-compliance-until-next-audit = 次回監査まで

# semantic i18n key (#2542)
admin-compliance-next-audit-date = 予定: 2025年3月7日

# semantic i18n key (#2542)
admin-compliance-trust-service-criteria = トラストサービス基準

# semantic i18n key (#2542)
admin-compliance-availability = 可用性

# semantic i18n key (#2542)
admin-compliance-processing-integrity = 処理の完全性

# semantic i18n key (#2542)
admin-compliance-confidentiality = 機密性

# semantic i18n key (#2542)
admin-compliance-controls-requiring-attention = 対応が必要なコントロール

# semantic i18n key (#2542)
admin-compliance-control-mfa-privileged = 特権アカウントの多要素認証

# semantic i18n key (#2542)
admin-compliance-due-jan-25-2025 = 期限: 2025年1月25日

# semantic i18n key (#2542)
admin-compliance-remediate = 修正

# semantic i18n key (#2542)
admin-compliance-control-incident-response-testing = セキュリティインシデント対応計画のテスト

# semantic i18n key (#2542)
admin-compliance-issue-annual-test-overdue = 年次テストが15日遅延

# semantic i18n key (#2542)
admin-compliance-control-dpa-documentation = データ処理契約の文書化

# semantic i18n key (#2542)
admin-compliance-due-feb-1-2025 = 期限: 2025年2月1日

# semantic i18n key (#2542)
admin-compliance-control-data-classification-review = データ分類ポリシーのレビュー

# semantic i18n key (#2542)
admin-compliance-issue-policy-review-pending = ポリシーレビュー承認待ち

# semantic i18n key (#2542)
admin-compliance-due-feb-15-2025 = 期限: 2025年2月15日

# semantic i18n key (#2542)
admin-compliance-control-privacy-notice-updates = プライバシー通知の更新

# semantic i18n key (#2542)
admin-compliance-issue-privacy-policy-update = 新しいデータ収集に合わせてプライバシーポリシーの更新が必要です

# semantic i18n key (#2542)
admin-compliance-due-feb-28-2025 = 期限: 2025年2月28日

# semantic i18n key (#2542)
admin-compliance-evidence-collection = 証跡収集

# semantic i18n key (#2542)
admin-compliance-current = 最新

# semantic i18n key (#2542)
admin-compliance-due-soon = まもなく期限

# semantic i18n key (#2542)
admin-compliance-evidence-pentest-q4 = 侵入テストレポート 2024年第4四半期

# semantic i18n key (#2542)
admin-compliance-evidence-cc4-vuln = CC4.1.1 - 脆弱性評価

# semantic i18n key (#2542)
admin-compliance-evidence-access-review-jan = アクセスレビューレポート - 2025年1月

# semantic i18n key (#2542)
admin-compliance-evidence-cc6-access = CC6.2.1 - アクセス制御

# semantic i18n key (#2542)
admin-compliance-due-in-3-days = 3日後に期限

# semantic i18n key (#2542)
admin-compliance-evidence-backup-restore = バックアップ復元テスト結果

# semantic i18n key (#2542)
admin-compliance-evidence-a1-backup = A1.2.3 - バックアップと復旧

# semantic i18n key (#2542)
admin-compliance-due-in-5-days = 5日後に期限

# semantic i18n key (#2542)
admin-compliance-evidence-security-training = セキュリティ意識向上トレーニング記録

# semantic i18n key (#2542)
admin-compliance-evidence-cc1-training = CC1.4.1 - セキュリティトレーニング

# semantic i18n key (#2542)
admin-compliance-view-all-evidence = すべての証跡を表示

# semantic i18n key (#2542)
admin-compliance-access-events = アクセスイベント

# semantic i18n key (#2542)
admin-compliance-configuration-changes = 構成変更

# semantic i18n key (#2542)
admin-compliance-security-events = セキュリティイベント

# semantic i18n key (#2542)
admin-compliance-compliance-events = コンプライアンスイベント

# semantic i18n key (#2542)
admin-compliance-failed-login-attempt = ログイン試行失敗

# semantic i18n key (#2542)
admin-compliance-failed-login-details = ユーザー: admin@company.com | IP: 192.168.1.105 | 理由: パスワードが無効です

# semantic i18n key (#2542)
admin-compliance-user-logged-in = ユーザーがログインしました

# semantic i18n key (#2542)
admin-compliance-user-logged-in-details = ユーザー: sarah.johnson@company.com | IP: 10.0.0.45

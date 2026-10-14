### name: Japanese
### native: 日本語
### author: Crow (first translation; corrections welcome)
### crow: 0.8.2
### font: Noto Sans CJK JP

## appearance
appearance-choose-a-picture-for-the = フリートページの画像を選択

## app-archive
app-archive-stored-data-purged = 保存データは削除済み
app-archive-data-purged-in-days = { $days } 日後にデータを削除
app-archive-data-purged-in-1-day = 1 日後にデータを削除
app-archive-data-purged-today = 今日データを削除
app-archive-purge-due = 削除予定
app-archive-purging-is-off-data-kept = 削除はオフ · データは保持
app-archive-never-purging-is-switched-off = しない — 削除はオフ
app-archive-is-the-bastion-for-move = { $name } は { $users } の踏み台です。先にそれらを移動してからアーカイブしてください
app-archive-could-not-archive = { $name } をアーカイブできませんでした: { $e }
app-archive-could-not-archive-the-vault = アーカイブできませんでした: ボールトを使用できません
app-archive-archived-its-data-is-purged = { $name } をアーカイブしました · データの削除: { $purge_due_text }
app-archive-could-not-restore = { $name } を復元できませんでした: { $e }
app-archive-restored-to-the-fleet = { $name } をフリートに復元しました
app-archive-could-not-purge-s-data = { $name } のデータを削除できませんでした: { $e }

## app-bastions
app-bastions-is-still-the-bastion-for = { $name } はまだ { $users } の踏み台です。先にそれらを移動してください
app-bastions-is-a-bastion-servers-can = { $name } は踏み台です。これを経由してサーバーを追加できます
app-bastions-is-no-longer-a-bastion = { $name } は踏み台ではなくなりました

## app-certificates
app-certificates-isnt-managed-by-certbot-or = { $cert_path } は { $name } の certbot でも acme.sh でも管理されていません。発行元で更新してください。
app-certificates-certbot-dry-run-renew-reload = certbot: ドライラン、{ $name } を更新、再読み込み{ $inv }{ $inv2 }、確認
app-certificates-acme-sh-renew-reload-confirm = acme.sh: { $domain } を更新、再読み込み、確認
app-certificates-renew = 更新 { $label }

## app-checks
app-checks-http-a-word-the-page = HTTP: ページに含まれるべき語 · DNS: 解決されるべきアドレス
app-checks-name-optional = 名前（任意）
app-checks-type-what-to-check-first = 先にチェック対象を入力してください。
app-checks-an-http-check-needs-a = HTTP チェックには http:// か https:// で始まる URL が必要です。
app-checks-a-tcp-check-is-host = TCP チェックは host:port の形式です（例: db.lan:5432）。
app-checks-added-checked-every-s-while = 「{ $name }」を追加しました: Crow が開いている間 { $every_secs } 秒ごとにチェックします。

## app-clankers
app-clankers-set-leave-blank-to-keep = 設定済み · そのままにするなら空欄、または新しいキーを貼り付け
app-clankers-e-g-sk-proj = 例: sk-proj-...
app-clankers-e-g-gpt-4o-mini = 例: gpt-4o-mini
app-clankers-model-cannot-be-empty = モデルは空にできません
app-clankers-the-vault-is-busy-try = ボールトが使用中です。もう一度お試しください
app-clankers-unknown-provider = 不明なプロバイダー
app-clankers-no-encryption-key = 暗号化キーがありません
app-clankers-not-saved = 保存されませんでした: { $e }
app-clankers-is-the-primary-clanker = { $name } がメインの Clanker です
app-clankers-is-the-backup-clanker = { $display_name } が予備の Clanker です
app-clankers-no-backup-clanker = 予備の Clanker なし
app-clankers-provider-call-stats-reset = プロバイダーの呼び出し統計をリセットしました
app-clankers-no-ai-provider-has-an = API キーを持つ AI プロバイダーがまだありません。上の ⚙ キーを編集 から追加してください。
app-clankers-via-the-backup-the-primary =
    予備の { $provider_label } による回答（メインが失敗: { $why }）
    {""}
    { $text }
app-clankers-via =
    { $provider_label } による回答
    {""}
    { $text }

## app-configs
app-configs-the-fleet = フリート
app-configs-group = グループ { $scope }
app-configs-type-a-key-or-pick = キーを入力するか、一覧から選んでください
app-configs-a-directive-needs-its-arguments = ディレクティブには引数が必要です
app-configs-could-not-be-parsed = { $selected } を解析できませんでした: { $e }
app-configs-no-config-file-selected = 設定ファイルが選択されていません。
app-configs-isnt-one-of-the-files = { $path } は、このサーバーで Crow が管理しているファイルではありません
app-configs-this-servers-configs-are-still = このサーバーの設定はまだ読み込み中です。何も書き込んでいません
app-configs-is-not-loaded = { $file } は読み込まれていません
app-configs-couldnt-re-read-it = { $file }: 再読み込みできませんでした: { $e }
app-configs-not-saved = { $file }: 保存されませんでした: { $e }
app-configs-filter-configs = 設定を絞り込み...
app-configs-reverted-firewall-rules-to-baseline = ファイアウォールのルールをベースラインに戻しました
app-configs-the-vault-is-busy = ボールトが使用中です
app-configs-no-snapshot-nothing-applied = スナップショットなし、何も適用していません: { $e }
app-configs-the-snapshot-failed-so-nothing = スナップショットに失敗したため、何も適用していません: { $e }
app-configs-type-exactly-to-stage-this = この変更をステージするには { $risk_confirm_keyword } と正確に入力してください。
app-configs-audit-commit-created = 監査コミットを作成しました: { $description }
app-configs-rolled-back-firewall-to-revision = ファイアウォールをリビジョン v{ $version } に戻しました
app-configs-this-machine = このマシン
app-configs-was-written-but-its-history = { $file } は書き込みましたが、履歴は記録されませんでした: { $e }
app-configs-baseline-not-pinned = ベースラインを固定できませんでした: { $e }
app-configs-baseline-not-unpinned = ベースラインの固定を解除できませんでした: { $e }
app-configs-couldnt-be-read-or-recorded = 読み取りも記録もできませんでした: { $e }

## app-containers
app-containers-done = 完了: { $summary }
app-containers-failed = { $summary } に失敗: { $e }
app-containers-no-ai-provider-has-a = キーを持つ AI プロバイダーがまだありません。設定 → Clanker（AI）で追加してください。
app-containers-every-network = すべてのネットワーク
app-containers-this-machine-only = このマシンのみ

## app-danger
app-danger-the-vault-is-busy = ボールトが使用中です
app-danger-asking-to = { $label } に { $verb } を依頼中…
app-danger-type-exactly-to-confirm = 確認するには { $keyword } と正確に入力してください。
app-danger-no-active-server = アクティブなサーバーがありません
app-danger-stopped-lab-container-s-failed = ラボコンテナーを { $ok } 個停止、{ $failed } 個失敗
app-danger-unknown-action = 不明な操作

## app-drift
app-drift-a-file-sshd-config-or = 任意のサーバーのファイル（sshd_config）または設定（PermitRootLogin）…
app-drift-couldnt-read-on-a-server = あるサーバーで { $failed } を読み取れませんでした: { $failed2 }
app-drift-the-baselines-content-wasnt-kept = ベースラインの内容は保存されていません（ハッシュのみ）。送信するものがありません
app-drift-not-reachable-right-now = 現在接続できません
app-drift-put-back-to-the-baseline = { $path } を { $e } のベースラインに戻す
app-drift-bring-back = 戻す { $path } · { $n } { $n ->
       *[other] 台
    }
app-drift-the-vault-is-busy = ボールトが使用中です
app-drift-crow-has-no-copy-of = そのサーバーのコピーは Crow にありません
app-drift-only-the-hash-of-that = そのコピーはハッシュしか保存されていないため、どこにも送信できませんでした。ボールトを開いてもう一度読み取ってください
app-drift-s-is-now-the-baseline = { $name } の { $path } が { $app } のベースラインになりました。今後ほかのサーバーはこれと比較されます。
app-drift-not-accepted = 承認されませんでした: { $e }

## app-dr
app-dr-plan-name-e-g-orders = プラン名（例: 注文データベース）
app-dr-rto-minutes = RTO（分）
app-dr-rpo-minutes = RPO（分）
app-dr-drill-notes-e-g-restored = 訓練メモ（例: 昨夜のダンプから Multipass VM に復元）

app-dr-what-matters-one-per-line =
    重要なもの（1 行に 1 つ）:
    /var/lib/postgresql
    /etc/nginx
    注文データベース
app-dr-backups-one-per-line-path =
    バックアップ（1 行に 1 つ、パスまたはグロブ | タイマー）:
    /var/backups/db/*.sql.gz | pg-dump.timer
app-dr-rebuild-1-launch-a-fresh =
    ## 再構築
    1. 新しいサーバーを起動（サーバーを追加）
    2. 復元: zcat /var/backups/db/latest.sql.gz | psql orders
    3. ベースラインを送信（ドリフト → 戻す）
    4. サイトを確認（チェック）
app-dr-give-the-plan-a-name = プランに名前を付け、対象を選んでください。
app-dr-saved = 保存しました。
app-dr-save-the-plan-first = 先にプランを保存してください。
app-dr-not-reachable-right-now-nothing = 現在接続できません: 何も確認できませんでした
app-dr-drill-started-follow-the-runbook = 訓練を開始しました: 手順書に従って（Multipass VM か作業用サーバーに復元）、終わったら 訓練を終了 を押してください。
app-dr-drill-recorded-min-the-rto = 訓練を記録しました: { $minutes } 分、RTO { $rto_minutes } 分{ $met ->
        [yes] 以内
       *[no] 超過
    }。
app-dr-group = グループ { $g }
app-dr-plan-written-to-keep-a = プランを { $path } に書き出しました。Crow が動かないときにも読める場所にコピーを置いてください。
app-dr-couldnt-write = { $path } に書き込めませんでした: { $e }

## app-files
app-files-chmod = chmod { $after } { $path }: { $e }
app-files-that-name-has-control-characters = その名前には制御文字が含まれています。ターミナルから自分で開いてください。

## app-firewall
app-firewall-8080-or-a-range-8000 = 8080、または範囲 8000:8100
app-firewall-anywhere-or-an-address-cidr = どこからでも、またはアドレス / CIDR
app-firewall-why-this-rule-exists = このルールがある理由
app-firewall-reading-s-firewall = { $name } のファイアウォールを読み取り中…
app-firewall-ran = 実行: { $summary }
app-firewall-failed = 失敗: { $summary } — { $e }
app-firewall-type-exactly-to-apply-this = この変更を適用するには { $risk_confirm_keyword } と正確に入力してください。
app-firewall-not-applied-no-firewall-crow = 適用されませんでした: このホストには Crow が変更できるファイアウォールが見つかりません。
app-firewall-not-applied-crow-reads-but = 適用されませんでした: Crow は { $label } を読み取りますが、書き込みはしません。
app-firewall-not-applied = 適用されませんでした: { $e }

## app-fleet-run
app-fleet-run-the-baselines-content-wasnt-kept = { $path }: ベースラインの内容は保存されていません（ハッシュのみ）。送信するものがありません
app-fleet-run-push-baseline = ベースライン { $path } を送信
app-fleet-run-rolling-reboot = 順次再起動 · { $n } { $n ->
       *[other] 台
    }
app-fleet-run-the-machine-crow-runs-on = Crow が動いているマシン: 自分で再起動してください
app-fleet-run-not-reachable-right-now-so = 現在接続できないため、Crow は復帰を確認できません
app-fleet-run-reboot-via = { $label } 経由で再起動
app-fleet-run-restart-the-lab-container = ラボコンテナーを再起動
app-fleet-run-the-machine-crow-runs-on-2 = Crow が動いているマシン
app-fleet-run-a-lab-container-reached-without = SSH を使わずに接続するラボコンテナー
app-fleet-run-not-reachable-right-now-the = 現在接続できません: 新しい鍵は現在の接続で取得します
app-fleet-run-new-host-keys-re-pin = 新しいホスト鍵、再固定、新規ログインで確認
app-fleet-run-not-pinned = 未固定
app-fleet-run-other-machines = ほかのマシン
app-fleet-run-anyone-else-who-sshes-to = これらのサーバーに SSH 接続するほかの人には、ホスト鍵の変更が表示され、再確認が必要になります
app-fleet-run-rotate-host-keys = ホスト鍵を更新 · { $n } { $n ->
       *[other] 台
    }
app-fleet-run-the-machine-crow-runs-on-3 = Crow が動いているマシン（あなた自身のログイン）
app-fleet-run-not-reachable-right-now = 現在接続できません
app-fleet-run-end-every-ssh-login-but = Crow 以外のすべての SSH ログインを終了
app-fleet-run-no-key-is-removed-anyone = 鍵は削除されません: 鍵がまだ有効な人は再びログインできます（そのためには鍵を更新または削除してください）
app-fleet-run-revoke-sessions = セッションを無効化 · { $n } { $n ->
       *[other] 台
    }
app-fleet-run-crow-has-only-the-public = Crow はこの鍵の公開部分しか持っていないため、ログインに使えません
app-fleet-run-no-server-logs-in-with = { $name } でログインするサーバーはありません。更新するものはありません。
app-fleet-run-couldnt-generate-the-new-key = 新しい鍵を生成できませんでした: { $e }
app-fleet-run-new-key-written-to-if = 新しい鍵を { $dir } に書き込みました。キャンセルしても未使用のまま残ります
app-fleet-run-install-new-key-verify-switch = 新しい鍵を設置、確認、切り替え、古い鍵を削除
app-fleet-run-install-key-verify-switch = 鍵を設置、確認、切り替え
app-fleet-run-switched-old-key-removed = 切り替え済み。古い鍵は削除しました
app-fleet-run-type-exactly-to-start = 開始するには { $keyword } と正確に入力してください。
app-fleet-run-approved-by = { $by } が承認しました。
app-fleet-run-crow = crow: { $title } — { $summary }
app-fleet-run-pinned-was = { $r } を固定（以前: { $before }）

## app-groups
app-groups-group-name-e-g-workers = グループ名（例: workers、edge、data）

## app-hardening
app-hardening-why-this-risk-is-acceptable = このリスクを許容する理由（許容するには必須）
app-hardening-write-why-the-risk-is = 先にリスクを許容する理由を書いてください。理由はレポートと監査ログに残ります。
app-hardening-accepted-its-no-longer-flagged = 許容しました: マップにもここにも表示されなくなり、レポートにはあなたの理由とともに記載されます。
app-hardening-accepted-as-a-risk = リスクとして許容
app-hardening-not-an-ssh-server-crow = SSH サーバーではありません（Crow はすべての修正を新規の SSH ログインで確認します）
app-hardening-not-reachable-right-now = 現在接続できません
app-hardening-neither-ufw-nor-firewalld-is = ufw も firewalld もインストールされていません: どちらかをインストールしてから（apt install ufw、dnf install firewalld）修正してください
app-hardening-crow-logs-in-as-root = Crow は鍵で root としてログインしています: prohibit-password が維持できる最も安全な設定です
app-hardening-report-written-to = レポートを { $path } に書き出しました
app-hardening-couldnt-write-the-report-to = レポートを { $path } に書き出せませんでした: { $e }

## app-history-view
app-history-view-disk = ディスク { $mount }

## app-incidents
app-incidents-on = { $name } の { $detail }
app-incidents-days = { $secs } 日
app-incidents-hours = { $secs } 時間
app-incidents-no-ai-provider-has-a = キーを持つ AI プロバイダーがまだありません。設定 → Clanker（AI）で追加してください。
app-incidents-timeline-written-to = タイムラインを { $path } に書き出しました
app-incidents-couldnt-write = { $path } に書き込めませんでした: { $e }
app-incidents-the-last = { $name }、直近 { $secs }

## app-journal
app-journal-no-ai-provider-has-an = API キーを持つ AI プロバイダーがありません。設定 → Clanker で追加してください。
app-journal-there-are-no-log-lines = 説明する対象のログ行が画面にありません。
app-journal-and-worse = { $label } 以上
app-journal-answered-by-the-backup-the =
    予備の { $provider_label } が回答しました: メインが失敗しました（{ $why }）。
    {""}
    { $text }
app-journal-nothing-to-export = 書き出すものがありません
app-journal-copied = { $lines } { $lines ->
       *[other] 行
    }をコピーしました

## app-keys
app-keys-enrolled-into-group = 「{ $name }」をグループ「{ $target_group }」に登録しました
app-keys-removed-key-from-memory-file = 鍵をメモリーから削除しました（ファイルは残ります）

app-keys-key-name-cannot-be-empty = 鍵の名前は空にできません
app-keys-group-name-cannot-be-empty = グループ名は空にできません
app-keys-path-cannot-be-empty = パスは空にできません
app-keys-directory-does-not-exist = フォルダーがありません: { $expanded }
app-keys-e-g-id-ed25519-bastion = 例: id_ed25519_bastion
app-keys-e-g-nelson-crow = 例: nelson@crow
app-keys-e-g-staging-fleet-edge = 例: ステージングフリート、エッジ踏み台
app-keys-e-g-work-keys-or = 例: ~/work-keys や /etc/ssh
app-keys-enter-key-name = 鍵の名前…

## app-lab
app-lab-crows-key = Crow の鍵: { $e }
app-lab-multipass = Multipass: { $e }
app-lab-couldnt-save-in-the-vault = { $name } をボールトに保存できませんでした: { $e }
app-lab-is-in-the-fleet = { $name } はフリートにあります（{ $host }）
app-lab-launching = { $name } を起動中
app-lab-importing = { $name } を取り込み中
app-lab-deleted = { $vm } を削除しました
app-lab-done = { $vm }: { $label } 完了

## app-logsearch
app-logsearch-text-or-regex-e-g = テキストまたは正規表現（例: Out of memory|segfault）
app-logsearch-unit-optional-e-g-nginx = ユニット（任意、例: nginx.service）
app-logsearch-a-name-to-save-it = 保存する名前
app-logsearch-no-reachable-server-in = { $label } に接続できるサーバーがありません。
app-logsearch-couldnt-read-its-journal-no = ジャーナルを読み取れませんでした（journalctl がないか、接続できません）
app-logsearch-name-the-search-to-save = 保存するには検索に名前を付けてください。
app-logsearch-a-saved-search-needs-text = 保存する検索にはテキスト、ユニット、優先度のいずれかが必要です。何にでも一致する検索は常に通知してしまいます。
app-logsearch-saved-switch-alert-on-to = 「{ $name }」を保存しました。一致したときに知らせるには 通知 をオンにしてください。
app-logsearch-no-ai-provider-has-a = キーを持つ AI プロバイダーがまだありません。設定 → Clanker（AI）で追加してください。

## app
app-no-server-is-enrolled-yet = まだサーバーが登録されていません。
app-detecting = 検出中...

## app-notify
app-notify-blank-if-the-server-needs = サーバーがログイン不要なら空欄
app-notify-set-leave-blank-to-keep = 設定済み · そのままにするなら空欄
app-notify-the-smtp-password-or-app = SMTP パスワードまたはアプリパスワード
app-notify-you-example-com-oncall-example = you@example.com, oncall@example.com
app-notify-e-g-22-07-or = 例: 22-07、または空欄
app-notify-the-form-isnt-open = フォームが開いていません
app-notify-the-port-is-a-number = ポートは 465 や 587 のような数値です
app-notify-the-digest-interval-is-a = まとめの間隔は分単位の数値です
app-notify-theres-a-username-but-no = ユーザー名はありますがパスワードがありません
app-notify-the-vault-is-busy-try = ボールトが使用中です。もう一度お試しください
app-notify-the-vault-is-locked = ボールトはロックされています
app-notify-saved-alerts-are-mailed-while = 保存しました。Crow の実行中はアラートをメールで送ります。
app-notify-not-saved = 保存されませんでした: { $e }
app-notify-the-saved-password-cant-be = ボールトがロックされている間は保存したパスワードを読み取れません
app-notify-sent-to-check-the-inbox = { $to } に送信しました。受信トレイ（と迷惑メール）を確認してください。
app-notify-nothing-was-sent = 何も送信されませんでした
app-notify-the-smtp-password-cant-be = SMTP パスワードを読み取れません（ボールトがロックされていませんか？）

## app-onboard
app-onboard-e-g-10-0-4 = 例: 10.0.4.32 や prod-db.internal
app-onboard-e-g-prod-db-01 = 例: prod-db-01
app-onboard-e-g-postgres-primary = 例: postgres, primary
app-onboard-test-node = テストノード · { $distro }
app-onboard-the-servers-address-is-missing = サーバーのアドレスがありません。
app-onboard-that-isnt-an-address-a = アドレスではありません。空白のないホスト名か IP を入力してください。
app-onboard-the-user-to-log-in = ログインするユーザーがありません。
app-onboard-pick-the-key-crow-logs = Crow がログインに使う鍵を選ぶか、パスワードを一度だけ使ってください。
app-onboard-type-the-servers-password-crow = サーバーのパスワードを入力してください。Crow は鍵の設置に一度だけ使います。
app-onboard-trust-the-servers-host-key = 先にサーバーのホスト鍵を信頼してください（または再確認）。
app-onboard-give-it-a-name = 名前を付けてください。
app-onboard-is-already-the-name-of = { $name } はすでに別のサーバーの名前です。
app-onboard-no-home-directory = ホームディレクトリがありません
app-onboard-no-free-crow-ed25519-file = ~/.ssh に空いている crow_ed25519 ファイル名がありません
app-onboard-saving-crows-key-in-the = Crow の鍵をボールトに保存中: { $e }
app-onboard-enter-the-servers-password-on = 手順 2（認証情報）でサーバーのパスワードを入力してください
app-onboard-the-bastion = 踏み台
app-onboard-cant-go-through-the-bastion = 踏み台を経由できません: { $e }
app-onboard-the-host-key-changed-remove = ホスト鍵が変わりました。変更が想定どおりなら、~/.ssh/known_hosts の古い項目を自分で削除してください。
app-onboard-no-host-key-was-fetched = サーバーからホスト鍵を取得できませんでした。もう一度調べてください。
app-onboard-couldnt-write-ssh-known-hosts = ~/.ssh/known_hosts に書き込めませんでした: { $e }
app-onboard-server-enrolled-into-fleet = サーバー「{ $name }」をフリートに登録しました
app-onboard-password-login = パスワードログイン: { $why }
app-onboard-logged-in-with-the-password = パスワードで一度ログインし、Crow の鍵を設置しました（{ $fingerprint }）
app-onboard-key-only-from-now-on = 今後は鍵のみ
app-onboard-password-login-2 = パスワードログイン: { $e }

## app-overview
app-overview-search-services = サービスを検索…  ( / )
app-overview-search-command-user-pid = コマンド、ユーザー、PID を検索…  ( / )
app-overview-search-name-group-shell = 名前、グループ、シェルを検索…  ( / )
app-overview-search-port-address-comment = ポート、アドレス、コメントを検索…  ( / )
app-overview-couldnt-read-updates = 更新を読み取れませんでした: { $e }
app-overview-failed = { $failed } 失敗
app-overview-on = オン
app-overview-off = オフ
app-overview-established-listening = 確立 { $established }、待ち受け { $listening }
app-overview-no-ai-provider-has-a = キーを持つ AI プロバイダーがまだありません。設定 → Clanker（AI）で追加してください。

## app-palette
app-palette-page = ページ
app-palette-this-server = このサーバー
app-palette-danger-zone = 危険ゾーン
app-palette-offline-unreachable-down = オフライン 接続不可 停止 offline unreachable down
app-palette-server = サーバー
app-palette-terminal = ターミナル
app-palette-terminal-on = { $name } のターミナル
app-palette-place = 場所
app-palette-fleet = フリート
app-palette-fleet-map = フリートマップ
app-palette-fleet-setup-policies = グループとポリシー
app-palette-patching = パッチ適用
app-palette-rollouts = ロールアウト
app-palette-drift-config-search = ドリフトと設定検索
app-palette-audit-log = 監査ログ
app-palette-settings = 設定
app-palette-ssh-keys = SSH 鍵
app-palette-fleet-config = フリート設定
app-palette-action = 操作
app-palette-add-a-server = サーバーを追加
app-palette-check-posture-now = 今すぐ態勢を確認
app-palette-reconnect-to-this-server = このサーバーに再接続
app-palette-lock-the-vault = ボールトをロック
app-palette-take-a-snapshot = スナップショットを作成
app-palette-security-stance-2fa = セキュリティ方針（2FA）
app-palette-about-crow = Crow について
app-palette-key = 鍵
app-palette-config = 設定
app-palette-service = サービス
app-palette-process = プロセス
app-palette-pid = PID { $pid } · { $user }
app-palette-search-servers-pages-config-files = サーバー、ページ、設定ファイル、操作、鍵を検索…

## app-password-login

app-password-login-crow-doesnt-connect-to-this = Crow はまだ自分の鍵でこのサーバーに接続していません。先に鍵ログインを設定してから（サーバーを追加 → Crow の鍵を設置）、パスワードをオフにしてください。
app-password-login-password-login-is-already-off = ✓ このサーバーではパスワードログインはすでにオフです。
app-password-login-this-sshd-config-doesnt-include = この sshd_config は sshd_config.d を読み込まないため、Crow のドロップインは効きません。sshd_config シートで PasswordAuthentication を no にしてから sshd を再読み込みしてください。
app-password-login-couldnt-read-sshds-settings = sshd の設定を読み取れませんでした: { $e }

## app-patching
app-patching-every-server = すべてのサーバー
app-patching-group = グループ { $g }
app-patching-any-time-e-g-sun = いつでも · 例: Sun 02-05
app-patching-the-machine-crow-runs-on = Crow が動いているマシン: 自分でパッチを当ててください
app-patching-not-reachable-right-now = 現在接続できません
app-patching-its-updates-havent-been-read = 更新がまだ読み取られていません: 先に 更新を確認 を押してください
app-patching-couldnt-read-its-updates = 更新を読み取れませんでした: { $e }
app-patching-no-pending = 保留中の { $label } はありません
app-patching-cant-be-rebooted-from-here = ここから再起動できない（{ $why }）ため、再起動を伴う実行から外しました
app-patching-then-reboot-if-needed = {" "}· 必要なら再起動
app-patching-rebooted = { $done }。再起動: { $m }

## app-people
app-people-failed-exit = `{ $cloned }` が失敗しました（終了コード { $status }）: { $stderr }
app-people-a-keys-fingerprint-sha256-or = 鍵のフィンガープリント（SHA256:…）またはコメント
app-people-username-e-g-alice = ユーザー名（例: alice）
app-people-their-public-key-ssh-ed25519 = その人の公開鍵: ssh-ed25519 AAAA… alice@laptop
app-people-revoke-keys-lock-and-expire = 鍵を無効化し、ロックして期限切れにする
app-people-revoke-keys-delete-the-account = 鍵を無効化し、アカウントを削除（ホームは残す）
app-people-text = { $user }: { $verb }（{ $count } { $count ->
       *[other] 手順
    }）
app-people-remove-the-key-from = { $count } { $count ->
       *[other] 個
    }のアカウントから鍵を削除
app-people-fill-in-the-username-and = 先にユーザー名と公開鍵を入力してください。
app-people-no-server-in-has-been = { $label } のサーバーはまだ読み取られていません: 先に 再読み取り を押してください。
app-people-create-add-their-key = { $user } を作成{ $sudo ->
        [yes] （sudo 付き）
       *[no] {""}
    }し、鍵を追加
app-people-add-the-key-to = { $user } に鍵を追加

## app-plugins
app-plugins-local-vms-containers = ローカル VM とコンテナー
app-plugins-multipass-vms = Multipass VM
app-plugins-local-lab-containers = ローカルのラボコンテナー
app-plugins-is-on = オン
app-plugins-is-off = オフ
app-plugins-mails = { $s } に送信
app-plugins-not-set-up-yet = 未設定
app-user-plugins-couldnt-create = { $path } を作成できませんでした: { $e }
app-user-plugins-changed-while-reviewing = 確認中に { $name } が変更されました。もう一度確認してください。
app-user-plugins-accepted = { $name } を承認しました。ファイルは固定されています。
app-user-plugins-refused = { $name } を拒否しました。
app-user-plugins-revoked = { $name } の承認を取り消しました。
app-user-plugins-state-new = 新規
app-user-plugins-state-accepted = 承認済み
app-user-plugins-state-changed = 変更あり · 再確認
app-user-plugins-state-refused = 拒否
app-user-plugins-state-invalid = 読み込めません
app-user-plugins-not-accepted-now = 承認後にこのプラグインのファイルが変更されたため、無効になっています。設定 → プラグインで確認してください。
app-user-plugins-later-stage = サーバーを変更するアクションや、このマシンで実行するアクションは、今後のバージョンで使えるようになります。
app-user-plugins-needs-settings = 設定が必要です: { $names }
app-user-plugins-needs-input = { $name } の値が必要です
app-user-plugins-dry-run-first = まずこの値でドライランを実行してください。
app-user-plugins-dry-run-still-going = ドライランはまだ実行中です。
app-user-plugins-dry-run-failed = ドライランが失敗しました。実行する前に出力を確認してください。
app-user-plugins-type-keyword = 実行するには { $keyword } と入力してください。
app-user-plugins-stopped = 停止
app-user-plugins-needs-install = まずこのプラグインをインストールしてください (インストール)。このファイルでセットアップがまだ実行されていません。
app-user-plugins-missing-here = このマシンに必要: { $names }
app-user-plugin-settings-secret-saved = 保存済み · 入力すると置き換えます
app-user-plugin-settings-needed = 必須
app-user-plugin-settings-vault-locked = シークレットを保存するには保管庫のロックを解除してください。
app-user-plugin-settings-saved = { $name }: 設定を保存しました。
app-user-plugin-fleet-done = 完了
app-user-plugin-fleet-failed = 失敗
app-user-plugin-fleet-no-ssh = ラボコンテナー: このマシンでのアクションには SSH が必要です
app-user-plugin-fleet-no-servers = 実行するサーバーがありません。対象を変更してください。
app-user-plugin-fleet-no-result = プラグインはこのサーバーの結果を書きませんでした
app-user-plugin-fleet-title = { $label } · { $plugin }

## app-providers
app-providers-set-type-to-replace = 設定済み · 入力すると置き換え
app-providers-not-set = 未設定
app-providers-default = 既定: { $d }
app-providers-the-vault-is-busy-try = ボールトが使用中です。もう一度お試しください
app-providers-the-vault-is-locked-or = ボールトがロックされているかパスワードがないため、プロバイダーのシークレットを読み取れません
app-providers-no-provider-accounts-yet-set = プロバイダーのアカウントがまだありません。設定 → プロバイダーで設定してください。
app-providers-no-encryption-key = 暗号化キーがありません
app-providers-this-provider-cant-list-instances = このプロバイダーはインスタンスを一覧できません
app-providers-no-longer-listed-by = { $m }（{ $name } の一覧にもうありません）
app-providers-now-lists-for-it-crow = { $m }: { $name } の一覧では { $addrs } になっていますが、Crow は古いアドレスに接続しています

## app-recovery
app-recovery-checking-the-host = ホストを確認中…
app-recovery-connecting = 接続中…
app-recovery-is-reachable-again = { $name } に再び接続できます
app-recovery-type-exactly-to-trust-the = 新しい鍵を信頼するには { $name } と正確に入力してください。
app-recovery-no-key-was-fetched-from = サーバーから鍵を取得できませんでした。信頼するものがありません。
app-recovery-known-hosts-wasnt-changed = known_hosts は変更されませんでした: { $e }

## app-region
app-region-located = { $found } 台の位置を特定
app-region-by-geoip = { $by_geoip } 台は GeoIP で
app-region-without-cloud-metadata = { $unknown } 台はクラウドのメタデータなし
app-region-unreachable = { $failed } 台に接続不可
app-region-asking-server-s = { $count } 台のサーバーに問い合わせ中…
app-region-geoip-unavailable = { $note } · GeoIP を使用できません: { $e }

## app-render
app-render-as = { $name } として · { $name2 } · { $session_label }
app-render-no-active-server-selected = アクティブなサーバーが選択されていません
app-render-select-a-server-tab-or = サーバーのタブを選ぶか、新しい Linux ホストを登録して、サービスと設定を確認してください。
app-render-go-to-fleet-overview = ⬢ フリートの概要へ
app-render-enroll-new-server = + 新しいサーバーを登録
app-render-not-built-yet = { $active_view } · 未実装
app-render-fleet = フリート
app-render-lab-container = ラボコンテナー
app-render-ssh-connecting = ssh · 接続中
app-render-ssh-connected = ssh · 接続済み
app-render-ssh = ssh · { $label }

## app-rollouts
app-rollouts-the-machine-crow-runs-on = Crow が動いているマシン: 自分で実行してください
app-rollouts-not-reachable-right-now = 現在接続できません
app-rollouts-the-files-new-content = ファイルの新しい内容
app-rollouts-that-baselines-content-wasnt-kept = そのベースラインの内容は保存されていない（ハッシュのみ）ため、ロールアウトできません。
app-rollouts-write-the-command-to-run = 先に実行するコマンドを書いてください。
app-rollouts-as-root = root として:{" "}
app-rollouts-rollout-command = ロールアウト · コマンド
app-rollouts-the-files-path-must-be = ファイルのパスは /etc/nginx/conf.d/app.conf のような絶対パスにしてください。
app-rollouts-write-the-files-new-content = 先にファイルの新しい内容を書いてください（サーバーのコピーやベースラインから始めることもできます）。
app-rollouts-reading-each-servers-current-file = プレビューのために各サーバーの現在のファイルを読み取り中です。読み終わったらもう一度計画してください。
app-rollouts-already-identical = すでに同じ
app-rollouts-write = { $path } を書き込み（+{ $add } −{ $del }）
app-rollouts-create = { $path } を作成
app-rollouts-couldnt-read-its-copy = コピーを読み取れませんでした: { $e }
app-rollouts-its-copy-hasnt-been-read = コピーはまだ読み取られていません
app-rollouts-rollout = ロールアウト · { $path }
app-rollouts-remove-it-didnt-exist = { $path } を削除（元は存在しなかった）
app-rollouts-put-back-as-it-was = { $path } を元に戻す
app-rollouts-roll-back = 戻す { $path } · { $n } { $n ->
       *[other] 台
    }

## app-secrets
app-secrets-the-key-stored-there-is = そこに保存された鍵は壊れています
app-secrets-not-saved-set-up-the = 保存されませんでした: 先にボールトのパスワードを設定してから、もう一度保存してください
app-secrets-not-saved = 保存されませんでした
app-secrets-the-os-keyring-no-longer = OS のキーリングに Crow の暗号化キーがなくなっていたため、保存済みのシークレット { $lost } 件を読み取れず、消去しました。もう一度入力してください。
app-secrets-the-vault-is-busy = ボールトが使用中です
app-secrets-moved-ai-api-key-s = AI の API キー { $n } 件を平文から暗号化ボールトに移しました。
app-secrets-couldnt-move-ai-api-keys = AI の API キーをボールトに移せませんでした: { $e }
app-secrets-the-os-keyring-no-longer-2 = OS のキーリングに Crow の暗号化キーがないため、保存したプロバイダーのトークンや AI のキーを読み取れません。もう一度入力してください。

app-secrets-ai-api-keys-are-still = AI の API キーはまだ平文で保存されています: { $why }。
app-secrets-ai-api-keys-are-still-2 = OS のキーリングが使えないため、AI の API キーはまだ平文で保存されています。暗号化するにはボールトのパスワードをオンにしてください。

## app-server-facts
app-server-facts-reconnecting-to = { $name } に再接続中…
app-server-facts-is-connected-again = { $name } に再接続しました
app-server-facts-couldnt-reconnect-to = { $name } に再接続できませんでした: { $e }

## app-session
app-session-enter-master-password = マスターパスワードを入力…
app-session-password-cannot-be-empty = パスワードは空にできません
app-session-6-digit-2fa-code-is = 6 桁の 2FA コードが必要です
app-session-this-computer = このコンピューター
app-session-enter-master-password-min-8 = マスターパスワードを入力（8 文字以上）…
app-session-re-type-password = パスワードを再入力…
app-session-6-digit-code-e-g = 6 桁のコード（例: 123456）
app-session-password-must-be-at-least = パスワードは 8 文字以上にしてください
app-session-passwords-do-not-match = パスワードが一致しません
app-session-please-enter-the-6-digit = 認証アプリの 6 桁の確認コードを入力してください
app-session-invalid-6-digit-code-check = 6 桁のコードが正しくありません。認証アプリを確認して、もう一度お試しください

## app-settings
app-settings-not-changed = 変更されませんでした: { $message }
app-settings-not-changed-2 = 変更されませんでした: { $e }
app-settings-value = 値…

## app-team
app-team-not-allowed = 許可されていません: { $why }。
app-team-passphrase = パスフレーズ
app-team-name-e-g-ana = 名前（例: ana）
app-team-their-passphrase-8-characters-they = その人のパスフレーズ（8 文字以上。後で変更できます）
app-team-a-second-members-passphrase = 2 人目のメンバーのパスフレーズ
app-team-signed-in-as = { $name }（{ $name2 }）としてサインインしました。
app-team-that-passphrase-isnt-right = パスフレーズが違います。
app-team-give-the-member-a-name = ほかの人と重ならない名前をメンバーに付けてください。
app-team-team-mode-is-on-is = チームモードがオンになりました: { $name } が最初の管理者としてサインインしています。今後 Crow は起動時に誰が操作しているかを尋ねます。
app-team-added-as = { $name } を { $name2 } として追加しました。
app-team-thats-the-last-admin-make = 最後の管理者です: 先にほかの誰かを管理者にしてください。
app-team-no-members-left-team-mode = メンバーがいなくなりました: チームモードはオフになり、Crow は再び 1 人用になります。
app-team-needs-a-second-members-approval = 2 人目のメンバーの承認が必要です: フリート実行を許可された別の人がパスフレーズを入力します

## app-terminal
app-terminal-shell = シェル { $i }
app-terminal-follows-the-shells-title-when = 空欄ならシェルのタイトルに従う
app-terminal-couldnt-start-a-terminal = ターミナルを開始できませんでした: { $e }

## app-updates
app-updates-downloading-and-verifying-crow = Crow { $version } をダウンロードして検証中…
app-updates-couldnt-start-the-new-version = 新しいバージョンを起動できませんでした: { $e }

## app-users
app-users-not-applied = 適用されませんでした: { $e }
app-users-ran = 実行: { $summary }
app-users-failed = 失敗: { $summary } — { $e }
app-users-not-applied-the-passwords-dont = 適用されませんでした: パスワードが一致しません
app-users-new-password = 新しいパスワード
app-users-again = もう一度
app-users-e-g-alice-deploy = 例: alice、deploy
app-users-e-g-alice-wonderland-optional = 例: Alice Wonderland（任意）
app-users-password = パスワード
app-users-not-applied-crow-logs-in = 適用されませんでした: Crow はこのサーバーに { $user } としてログインしています。削除すると Crow（と Crow 経由のあなた）が締め出されます。先にサーバーを別のアカウントに切り替えてください。
app-users-not-applied-thats-the-key = 適用されませんでした: Crow がこのサーバーへのログインに使っている鍵です。無効化すると Crow が締め出されます。先にサーバーを別の鍵に切り替えてから（鍵 → 切り替え）、この鍵を無効化してください。

## app-vault-manage
app-vault-manage-current-password = 現在のパスワード
app-vault-manage-2fa-code = 2FA コード
app-vault-manage-new-password-8-characters = 新しいパスワード（8 文字以上）
app-vault-manage-new-password-again = 新しいパスワード（確認）
app-vault-manage-the-new-passwords-dont-match = 新しいパスワードが一致しません
app-vault-manage-password-changed-your-secrets-and = パスワードを変更しました。シークレットと 2FA はそのままです。
app-vault-manage-tick-i-understand-first = 先に「理解しました」にチェックを入れてください。
app-vault-manage-crow-is-open-now-no = Crow は現在オープンです: パスワードなしで、鍵は OS のキーリングが保持します。
app-vault-manage-still-locked-couldnt-hand-the = まだロック中です: 鍵を OS のキーリングに渡せませんでした（{ $e }）。

## about
about-debug-build = デバッグビルド
about-release-build = リリースビルド
about-about-crow = Crow について
about-crow = Crow
about-native-linux-server-manager-over = SSH で使うネイティブな Linux サーバー管理ツール
about-website = ウェブサイト
about-source = ソース
about-community = コミュニティ
about-build = ビルド
about-platform = プラットフォーム
about-profile = プロファイル
about-ui-toolkit = UI ツールキット
about-gpui-kit-0-6-1 = GPUI Kit 0.6.1 · WGPU
about-config-engine = 設定エンジン
about-crow-config-core-schema-ir = crow-config-core · スキーマ IR
about-vault = ボールト
about-argon2id-chacha20-poly1305-totp = Argon2id · ChaCha20-Poly1305 · TOTP
about-credits = クレジット
about-icons = アイコン
about-tabler-icons-mit = Tabler Icons · MIT
about-flags = 国旗
about-flag-icons-mit = flag-icons · MIT
about-distro-logos = ディストリビューションのロゴ
about-dashboard-icons-by-homarr-labs = homarr-labs の Dashboard Icons · Apache-2.0
about-geoip-data = GeoIP データ
about-font = フォント
about-jetbrains-mono-ofl-1-1 = JetBrains Mono · OFL-1.1
about-license = ライセンス
about-aeupl-1-2-free-for = AEUPL-1.2。非商用であれば無料で利用できます。リポジトリの LICENSE を参照してください。
about-copied-system-info = ✓ システム情報をコピーしました
about-copy-system-info = システム情報をコピー
about-ok = OK
about-pre-release = {" "}（プレリリース）
about-release-notes = リリースノート
about-skip-this-version = このバージョンをスキップ
about-verify-install = 検証してインストール
about-restart-now = 今すぐ再起動
about-crow-doesnt-download-anything-unless = 設定 → 一般 でインストールをオンにしない限り、Crow は何もダウンロードしません。
about-crow-is-available = CROW { $version }{ $r } が利用可能

## danger-zone
danger-zone-simulated-signal-sent-to-crow = { $name } に { $action } シグナルを送ったことにしました（模擬）— Crow はまだ実際の電源操作を行いません
danger-zone-this-host-isnt-running-ufw = このホストでは ufw が動いていません — 消去するものはありません。
danger-zone-ufw-rules-flushed-on = { $label } の ufw ルールを消去しました
danger-zone-power = 電源
danger-zone-through-works-even-when-ssh = { $name } 経由: SSH が落ちていても使えます。
danger-zone-power-off = 電源オフ
danger-zone-simulated-crow-doesnt-send-real = 模擬: Crow はまだホストに実際の電源シグナルを送りません。実際に電源を制御するにはプロバイダーを連携してください。
danger-zone-reboot-45s-downtime = 再起動 · 45 秒の停止
danger-zone-snapshots = スナップショット…
danger-zone-snapshots-2 = スナップショット

danger-zone-taken-and-listed-at = { $name } で作成・一覧します。
danger-zone-flush-firewall = ファイアウォールを消去
danger-zone-resets-ufw-every-rule-is = ufw をリセットします: すべてのルールを削除し、ファイアウォールを無効にします。
danger-zone-flush-firewall-ufw = ファイアウォールを消去（UFW）
danger-zone-rotate-host-keys = ホスト鍵を更新
danger-zone-not-built-yet = 未実装です。
danger-zone-not-yet-built = 未実装
danger-zone-lab-containers = ラボコンテナー
danger-zone-stops-every-lab-node-crow = このマシンで Crow が作成したラボノードだけをすべて停止します。
danger-zone-kill-all-lab-containers = すべてのラボコンテナーを強制終了
danger-zone-archive-server = サーバーをアーカイブ
danger-zone-removes-it-from-the-fleet = フリートから外します。データは削除日まで保持され、復元できます。
danger-zone-archive-server-2 = サーバーをアーカイブ
danger-zone-danger-zone = 危険ゾーン
danger-zone-type-the-actions-keyword-to = 確認するには操作のキーワードを入力 · ワンクリックで破壊的な操作はしません
danger-zone-type-to-confirm = 確認するには { $keyword } と入力:
danger-zone-execute = 実行 ⏎
danger-zone-cancel = キャンセル

## distro-icon
distro-icon-alma-linux = alma linux
distro-icon-red-hat = red hat
distro-icon-arch-linux = arch linux
distro-icon-azure-linux = azure linux
distro-icon-mac-os = mac os
distro-icon-rhel = RHEL

## identity-bar
identity-bar-local = ローカル
identity-bar-lab-container = ラボコンテナー
identity-bar-connecting = 接続中…
identity-bar-connected = 接続済み
identity-bar-via-unreachable = { $name } 経由（接続不可）
identity-bar-via = { $name } 経由
identity-bar-unverified = 未検証
identity-bar-group = グループ
identity-bar-none = なし ▾
identity-bar-bastion = 踏み台 ✓
identity-bar-bastion-no = 踏み台: いいえ
identity-bar-search = 検索
identity-bar-reconnect = 再接続
identity-bar-start-vm = VM を起動
identity-bar-stop-vm = VM を停止
identity-bar-restart-vm = VM を再起動
identity-bar-crow-reaches-through-which-is = Crow は { $name2 } 経由で { $name } に接続しますが、{ $name2 } 自体に接続できません: 先に { $name2 } を直してください。{ $detail }
identity-bar-crow-cant-read-right-now = Crow は現在 { $name } を読み取れません — 接続できるまで下のページは空のままです。{ $detail }
identity-bar-what-happened = 何が起きたか →
identity-bar-set-up-key-login = 鍵ログインを設定 →
identity-bar-region-unknown = リージョン: 不明 ▾
identity-bar-region-unknown-2 = { $region_provider } · リージョン不明 ▾
identity-bar-set-by-hand = { $place }（手動設定）▾
identity-bar-by-ip = { $place }（IP から）▾

## palette
palette-recent-first = { $scope_label } · 新しい順
palette-text = { $scope_label } · { $count } { $count ->
       *[other] 件
    }
palette-nothing-matches-try-a-server = 一致するものがありません。サーバー名、ページ（ログ、ファイアウォール）、設定ファイル、操作で試してください。
palette-navigate = ↑↓ 移動
palette-open = ⏎ 開く
palette-into-a-server = ⇥ サーバーの中へ
palette-back-out = ⌫ 戻る
palette-esc-close = esc 閉じる

## password-login
password-login-reading-sshds-settings = sshd の設定を読み取り中…
password-login-working-checking-a-fresh-key = 処理中: 新規の鍵ログインを確認し、ドロップインを書き込み、検証、再読み込み、再確認しています…
password-login-done-keep-a-key-for = 完了しました。このサーバーの鍵を安全な場所に保管してください: もうパスワードではログインできません。
password-login-password-login-is-still-on = パスワードログインはまだオンです。
password-login-password-login-is-on-anyone = パスワードログインがオンです。パスワードを推測したり盗んだりした人は誰でも入れますし、ボットは絶えず試しています。オフにすると SSH 鍵だけが使えます。
password-login-what-crow-will-do-stopping = Crow が行うこと（最初の問題で停止して元に戻します）:
password-login-1-check-that-a-fresh = 1. 新規の鍵ログインが使えることを確認（Crow が保持している接続ではなく、新しい接続で）。
password-login-2-write-passwordauthentication-no-kbdinteractiveauthentication = 2. { $dropin } を書き込む（PasswordAuthentication no、KbdInteractiveAuthentication no）。ほかのドロップインより先に読まれるため、クラウドイメージ独自の設定に上書きされません。
password-login-3-validate-with-sshd-t = 3. sshd -t で検証し、sshd -T でパスワードが本当にオフになったことを確認。
password-login-4-reload-sshd-open-sessions = 4. sshd を再読み込み（開いているセッションは維持）し、もう一度新規の鍵ログインを確認。失敗したらファイルを削除して再読み込み。
password-login-these-accounts-have-a-password = 次のアカウントにはパスワードがあり、それによる SSH ログインができなくなります: { $pre }。先にそれぞれに鍵があることを確認してください。
password-login-i-understand = 理解しました
password-login-turn-off-ssh-password-login = SSH のパスワードログインをオフにする · { $server_name }
password-login-turn-off-password-login = パスワードログインをオフにする
password-login-cancel = キャンセル
password-login-close = 閉じる

## recovery-panel
recovery-panel-checking = 確認中
recovery-panel-what-to-do = 対処方法
recovery-panel-trust-new-key = 新しい鍵を信頼
recovery-panel-install-key-with-password = パスワードで鍵を設置
recovery-panel-try-again = 再試行
recovery-panel-close = 閉じる

## sidebar
sidebar-overview = 概要
sidebar-history = 履歴
sidebar-services = サービス
sidebar-processes = プロセス
sidebar-sockets = ソケット
sidebar-containers = コンテナー
sidebar-config = 設定ファイル
sidebar-logs = ログ
sidebar-files = ファイル
sidebar-cron = Cron
sidebar-users = ユーザー
sidebar-firewall = ファイアウォール
sidebar-terminal = ターミナル
sidebar-collapse = 折りたたむ
sidebar-danger-zone = 危険ゾーン
sidebar-plugins = プラグイン

## snapshot-offer
snapshot-offer-snapshot-the-whole-server-at = 先に { $provider } でサーバー全体のスナップショットを作成しますか？
snapshot-offer-youre-about-to-apply-a = { $file }（{ $description }）を適用しようとしています。ここで誤ると SSH から締め出され、Crow のファイルバックアップではサーバーに届かず元に戻せなくなります。{ $provider } のスナップショットなら外部からマシン全体を復元できます。変更は { $provider } がスナップショットを受け付けるまで待ち、失敗した場合は何も適用しません。
snapshot-offer-remember-my-choice-for-these = この種の変更では選択を記憶する
snapshot-offer-taking-snapshot = スナップショットを作成中…
snapshot-offer-snapshot-first-then-apply = 先にスナップショット、その後に適用
snapshot-offer-apply-without = スナップショットなしで適用
snapshot-offer-cancel = キャンセル

## snapshots-panel
snapshots-panel-asking = { $provider } に問い合わせ中…
snapshots-panel-has-no-snapshots-of-yet = { $provider } には { $server } のスナップショットがまだありません。
snapshots-panel-automatic = （自動）
snapshots-panel-snapshots = スナップショット · { $server }
snapshots-panel-at = { $provider }
snapshots-panel-restore-a-snapshot-from-the = スナップショットはプロバイダー自身のコンソールから復元してください: 復元するとサーバーのディスクが置き換わり、Crow はそれを代行しません。

## stance
stance-2fa = 2FA
stance-text = · { $weak } { $weak ->
       *[other] 件の問題
    }
stance-security-stance = セキュリティ方針
stance-no-vault-password-key-held = ボールトのパスワードなし · 鍵は OS のキーリングが保持
stance-vault-password-2fa-key-locked = ボールトのパスワード + 2FA · 鍵はパスワードでロック
stance-weakening-it-right-now = 今まさに弱めています

stance-what-it-protects = 守るもの
stance-what-youre-exposed-to = さらされるもの
stance-what-it-costs = 代償
stance-nothing-is-usable-without-your = パスワードと 2FA コードがなければ何も使えません。その代わり毎回ロック解除が必要で、どちらかを失うとシークレットも失います。
stance-no-unlocking-at-the-price = ロック解除は不要です。その代わり、あなたとして動くものはすべてのシークレットを読めます。
stance-switch-to-locked = ロックに切り替える →
stance-lock-now = 今すぐロック
stance-vault-security-settings = ボールトとセキュリティの設定
stance-no-password = パスワードなし
stance-password-2fa = パスワード + 2FA
stance-protects = 守るもの
stance-exposed-to = さらされるもの
stance-costs = 代償
stance-stay-open-and-save = オープンのまま保存
stance-i-understand-while-im-logged = 理解しました: ログインしている間は、私として動くものはこれらのシークレットを読めます。
stance-go-locked-set-a-password = ロックにする: パスワードを設定 →
stance-where-should-this-secret-stand = このシークレットをどう守りますか？
stance-crow-is-about-to-store = Crow が初めてシークレットを保存しようとしています。どちらの方針でも暗号化されますが、誰が取り出せるか、何をあきらめるかが違います。意図して選んでください。あとから方針バッジでロックに切り替えられます。
stance-cancel-dont-save = キャンセル（保存しない）
stance-the-other-stance = もう一方の方針、{ $label }: { $other }

## stat-strip
stat-strip-vcpu-peak = { $vcpu_count } vCPU · ピーク { $m }%
stat-strip-inodes-iowait = inode { $pct_or_dash } · iowait { $pct_or_dash2 }
stat-strip-mb-s = ↓ { $rx_mbps } · ↑ { $tx_mbps } Mb/s
stat-strip-turbo-buffer = ターボバッファー
stat-strip-s-lag-local-memory-cache = · -{ $lag_secs } 秒の遅延 · ローカルのメモリーキャッシュ
stat-strip-ahead-in-s = この先 · { $description } · { $lead_seconds } 秒後
stat-strip-next-s-nothing-unusual = この先 { $lag_secs } 秒: 異常なし
stat-strip-cpu-load = CPU 負荷
stat-strip-memory = メモリー
stat-strip-disk-nvme = ディスク { $disk_mount }（NVME）
stat-strip-load-avg = ロードアベレージ
stat-strip-uptime = 稼働時間
stat-strip-sampled = 取得 { $last_sample_ts }
stat-strip-network = ネットワーク
stat-strip-mb-s-2 = Mb/s

## status-bar
status-bar-no-open-alerts = 未解決のアラートなし
status-bar-1-open-alert = 未解決のアラート 1 件
status-bar-open-alerts = 未解決のアラート { $n } 件
status-bar-agentless-ssh = エージェントレス · ssh
status-bar-v-available = v{ $v } が利用可能 ↗

## titlebar
titlebar-crow = CROW
titlebar-fleet = フリート
titlebar-agentless-ssh = { $server_count } { $server_count ->
       *[other] 台
    } · エージェントレス ssh
titlebar-srv = SRV
titlebar-utc = UTC
titlebar-you = あなた
titlebar-overview = { $name } · 概要
titlebar-server-overview = サーバーの概要
titlebar-fleet-overview = フリートの概要
titlebar-enroll-new-server = 新しいサーバーを登録…
titlebar-this-server = このサーバー
titlebar-managed-configs = 管理対象の設定
titlebar-systemd-journal-logs = systemd ジャーナルのログ
titlebar-application = アプリケーション
titlebar-command-palette = コマンドパレット…
titlebar-settings = 設定
titlebar-about-crow = Crow について…
titlebar-session = セッション
titlebar-lock-vault-disconnect = ボールトをロックして切断
titlebar-quit-crow = Crow を終了

## audit-model
audit-model-by = { $text } · { $a }
audit-model-enrolled = 登録 { $login_user }@{ $host }:{ $port }
audit-model-saved = 保存 { $target }
audit-model-restored-an-earlier-version-of = { $target } の以前のバージョンを復元
audit-model-baseline-for = { $target } のベースライン: { $before_state }
audit-model-killed-process = プロセス { $target } を終了
audit-model-purged-stored-data-for = { $target } の保存データを削除
audit-model-turned-off-ssh-password-login = SSH のパスワードログインをオフに
audit-model-trusted-a-new-host-key = { $target } の新しいホスト鍵を信頼
audit-model-opened-a-terminal = ターミナルを開いた
audit-model-at-the-provider = プロバイダーで { $k }
audit-model-rolling-reboot = 順次再起動: { $before_state }
audit-model-ufw = ufw { $target }
audit-model-user = ユーザー { $target }

## audit
audit-audit-log = 監査ログ
audit-failed-every-server = { $count } { $count ->
       *[other] 件の変更
    } · { $failed } 件失敗 · 全サーバー
audit-server = サーバー
audit-kind = 種類
audit-nothing-recorded-yet = まだ何も記録されていません。

## config-cron-editor
config-cron-editor-every-minute = 毎分（* * * * *）
config-cron-editor-every-minutes = { $step } 分ごと
config-cron-editor-every-hour-on-the-hour = 毎時 0 分（0 * * * *）
config-cron-editor-every-hour-at-minute = 毎時 { $m } 分
config-cron-editor-every-day-at = 毎日 { $h }:{ $m }
config-cron-editor-sunday = 日曜日
config-cron-editor-monday = 月曜日
config-cron-editor-tuesday = 火曜日
config-cron-editor-wednesday = 水曜日
config-cron-editor-thursday = 木曜日
config-cron-editor-friday = 金曜日
config-cron-editor-saturday = 土曜日
config-cron-editor-weekday-mon-fri = 平日（月〜金）
config-cron-editor-weekend-sat-sun = 週末（土・日）
config-cron-editor-weekly-on-at = 毎週{ $day_name } { $h }:{ $m }
config-cron-editor-monthly-on-day-at = 毎月 { $dom } 日 { $h }:{ $m }
config-cron-editor-schedule = スケジュール: { $m } { $h } { $dom } { $mon } { $dow }
config-cron-editor-active-schedules = 有効なスケジュール { $active_count } 件
config-cron-editor-unsaved-edits = 未保存の編集
config-cron-editor-in-sync = 同期済み
config-cron-editor-new-cron-job = + 新しい cron ジョブ
config-cron-editor-revert = 元に戻す
config-cron-editor-stage-apply = ステージして適用
config-cron-editor-history = 履歴 · { $rev_count }
config-cron-editor-each-job-runs-as-its = 各ジョブはその「ユーザー」として実行 · 保存後 cron は /etc/crontab を自動で読み直します
config-cron-editor-no-jobs-in-etc-crontab = /etc/crontab にジョブがありません。+ 新しい cron ジョブ で追加できます。
config-cron-editor-on = 有効
config-cron-editor-schedule-2 = スケジュール
config-cron-editor-user = ユーザー
config-cron-editor-command = コマンド
config-cron-editor-every-15m = 15 分ごと
config-cron-editor-hourly = 毎時
config-cron-editor-daily-3-am = 毎日（午前 3 時）
config-cron-editor-daily-midnight = 毎日（午前 0 時）
config-cron-editor-weekly-sun = 毎週（日）
config-cron-editor-monthly-1st = 毎月（1 日）
config-cron-editor-workdays-9-am = 平日（午前 9 時）
config-cron-editor-quick-schedule-presets = よく使うスケジュール
config-cron-editor-5-field-schedule-expression = 5 フィールドのスケジュール式
config-cron-editor-minute-0-59 = 分（0-59）
config-cron-editor-hour-0-23 = 時（0-23）
config-cron-editor-day-of-month-1-31 = 日（1-31）
config-cron-editor-month-1-12 = 月（1-12）
config-cron-editor-day-of-week-0-7 = 曜日（0-7）
config-cron-editor-execution-user = 実行ユーザー
config-cron-editor-shell-command = シェルコマンド
config-cron-editor-any = {"*"}（任意）

## config-journald-editor
config-journald-editor-crash-safe = クラッシュに強い
config-journald-editor-volatile-risk = 揮発性のリスク
config-journald-editor-unsaved-edits = 未保存の編集
config-journald-editor-in-sync = 同期済み

config-journald-editor-usage = 使用量: { $disk_usage_display } / { $max_use_display }
config-journald-editor-revert = 元に戻す
config-journald-editor-stage-apply-config = 設定をステージして適用
config-journald-editor-history = 履歴 · { $rev_count }
config-journald-editor-why-default-linux-journal-configs = Linux の既定のジャーナル設定が本番で失敗する理由
config-journald-editor-by-default-systemd-journal-uses = 既定では、systemd のジャーナルはコンパイル時の暗黙の上限と Storage=auto を使います。負荷の高いサーバーや障害後の再起動では、ログが 30 分でローテートされたり完全に消えたりします。Crow-Config は明示的な上限を設定し、クラッシュの証拠を失わないようにします。
config-journald-editor-storage = 保存先
config-journald-editor-controls-where-journal-files-are = ジャーナルファイルの保存先を決めます。Volatile はログを RAM（/run）に置き、再起動で完全に消えます。Persistent は /var/log/journal/ への書き込みを保証します。
config-journald-editor-persistent-rec = 永続（推奨）
config-journald-editor-survives-reboots = 再起動後も残る
config-journald-editor-volatile = 揮発
config-journald-editor-ram-only-wiped-on-reboot = RAM のみ · 再起動で消える
config-journald-editor-auto = 自動
config-journald-editor-default-fallback = 既定の動作
config-journald-editor-systemmaxuse = SystemMaxUse
config-journald-editor-the-maximum-disk-space-journals = 永続ストレージでジャーナルが使えるディスク容量の上限です。超えると systemd は最も古いアーカイブから削除します。
config-journald-editor-4g-rec = 4G（推奨）
config-journald-editor-custom = カスタム
config-journald-editor-maxretentionsec = MaxRetentionSec
config-journald-editor-the-maximum-time-to-store = ジャーナルの項目を保存する最長期間です。ログが少ないときに古いアーカイブがいつまでも残るのを防ぎます。
config-journald-editor-30day-rec = 30day（推奨）
config-journald-editor-0-unlimited = 0（無制限）
config-journald-editor-systemkeepfree = SystemKeepFree
config-journald-editor-guarantees-that-at-least-this = ルートファイルシステムに少なくともこれだけの空きを残し、ログの洪水によるディスク満杯のクラッシュを防ぎます。
config-journald-editor-1-0-gb = 1.0 GB
config-journald-editor-2-0-gb-rec = 2.0 GB（推奨）
config-journald-editor-5-0-gb = 5.0 GB
config-journald-editor-ratelimitburst-ratelimitintervalsec = RateLimitBurst / RateLimitIntervalSec
config-journald-editor-suppresses-runaway-spam-loops-from = 暴走したメッセージのループがジャーナルをあふれさせ、システムを詰まらせるのを抑えます（既定は 30 秒あたり 10,000 件）。
config-journald-editor-entries-s = { $rate_limit_burst } 件 / { $rate_limit_interval_sec } 秒
config-journald-editor-anti-flood-enabled = ✓ 洪水対策が有効

## config-managed-files
config-managed-files-config-files = 設定ファイル
config-managed-files-unverified-distro-generic-scan-only = ⚠ 未検証のディストリビューション — 汎用スキャンのみのため、見つからない設定があるかもしれません。
config-managed-files-crow-ui-mapped = ● CROW の画面で編集可
config-managed-files-edited = 編集済み
config-managed-files-crow-ui = CROW 画面
config-managed-files-system-configs-raw = ○ システムの設定（テキスト）
config-managed-files-ro = 読取専用
config-managed-files-raw = テキスト
config-managed-files-schema-packs-active = 有効なスキーマパック
config-managed-files-systemd-postgres-openssh-ufw = systemd · postgres · openssh · ufw

## config-pending-diff-rail
config-pending-diff-rail-save-failed = 保存に失敗 · { $err }
config-pending-diff-rail-read-only = 読み取り専用 · { $reason }
config-pending-diff-rail-history = 履歴 · { $err }
config-pending-diff-rail-restore-v = v{ $v } を復元しますか？
config-pending-diff-rail-same-as-host = ホストと同じ
config-pending-diff-rail-pending-diff = 保留中の差分
config-pending-diff-rail-unified = unified · { $sel_file }
config-pending-diff-rail-author = 作成者:
config-pending-diff-rail-on-host = --- ホスト上/{ $sel_file }
config-pending-diff-rail-back-to-pending-diff = ← 保留中の差分に戻る
config-pending-diff-rail-baseline = --- ベースライン/{ $sel_file }
config-pending-diff-rail-staged-unsaved = +++ ステージ済み/{ $sel_file }（未保存）
config-pending-diff-rail-no-unsaved-edits = 未保存の編集はありません
config-pending-diff-rail-on-save = 保存時
config-pending-diff-rail-version-history-revisions = バージョン履歴（{ $count } リビジョン）
config-pending-diff-rail-on-host-not-recorded = ホスト上: 未記録
config-pending-diff-rail-on-host-v = ホスト上: v{ $active_rev }
config-pending-diff-rail-hash-only = ハッシュのみ
config-pending-diff-rail-restore = 復元
config-pending-diff-rail-current = 現在
config-pending-diff-rail-by = 作成者
config-pending-diff-rail-baseline-2 = ベースライン
config-pending-diff-rail-pin-as-baseline = ベースラインに固定
config-pending-diff-rail-revert = 元に戻す
config-pending-diff-rail-stage-apply = ステージして適用
config-pending-diff-rail-written-atomically-temp-file-then = アトミックに書き込み: 一時ファイルに書いてから名前を変更
config-pending-diff-rail-structured-edits-keep-comments-and = 構造化された編集はコメントとレイアウトを保持
config-pending-diff-rail-no-host-validator-for-this = この形式にはホスト上の検証ツールがありません
config-pending-diff-rail-checked-on-the-host-first = 先にホスト上で確認: { $cmds }
config-pending-diff-rail-plain-text-nothing-checks-it = プレーンテキスト: 書き込む前に何も確認しません
config-pending-diff-rail-not-kept-in-history-crow = 履歴に残りません: Crow はこのファイルをホストから読み取っていません
config-pending-diff-rail-every-version-kept-in-history = すべてのバージョンを暗号化して履歴に保存
config-pending-diff-rail-history-keeps-hashes-only-no = 履歴はハッシュのみ: 暗号化キーがありません
config-pending-diff-rail-history-not-loaded-yet = 履歴はまだ読み込まれていません
config-pending-diff-rail-none-pinned-pin-as-baseline = 固定なし。リビジョンで ベースラインに固定 を押すと、{ $scope_label } の正しい { $filename } になります。異なるサーバーはドリフトとして表示されます。
config-pending-diff-rail-matches-the-baseline = ベースラインと一致
config-pending-diff-rail-same-settings-as-the-baseline = 設定はベースラインと同じで、コメントやレイアウトだけが異なります
config-pending-diff-rail-drifted-from-the-baseline = ベースラインからドリフト
config-pending-diff-rail-more = … ほか { $more } 件
config-pending-diff-rail-pinned-from-for-by = { $set_by } が { $from_server } から { $scope_label } 用に固定
config-pending-diff-rail-load-baseline-into-editor = ベースラインをエディターに読み込む
config-pending-diff-rail-unpin = 固定を解除

## config-raw-editor
config-raw-editor-traditional-editor = 通常のエディター
config-raw-editor-edited = 編集済み（+{ $add_count } -{ $del_count }）
config-raw-editor-v-baseline = v{ $active_revision }（ベースライン）
config-raw-editor-author = 作成者:
config-raw-editor-history = 履歴 · { $count }
config-raw-editor-structured-view = 構造化ビュー
config-raw-editor-revert-baseline = ベースラインに戻す
config-raw-editor-stage-version = バージョンをステージ
config-raw-editor-lines = { $line_count } 行
config-raw-editor-bytes = { $count } バイト
config-raw-editor-utf-8 = UTF-8
config-raw-editor-unix-lf = UNIX（LF）
config-raw-editor-revisions = リビジョン: { $count }

config-raw-editor-read-only = 読み取り専用
config-raw-editor-unsaved-staged-changes = ● 未保存のステージ済み変更
config-raw-editor-clean-baseline = ✓ ベースラインのまま

## config-sshd-sheet
config-sshd-sheet-some-files-this-config-includes = この設定が読み込むファイルの一部を読み取れなかったため、以下の値は sshd が実際に使う値と違う可能性があります:
config-sshd-sheet-other-directives = その他のディレクティブ
config-sshd-sheet-match-in = MATCH { $scope } · { $path } 内
config-sshd-sheet-match-applies-only-when-this = MATCH { $scope } · 一致したときのみ適用
config-sshd-sheet-nothing-to-fix = 修正するものはありません
config-sshd-sheet-1-to-fix = 修正 1 件
config-sshd-sheet-to-fix = 修正 { $n } 件
config-sshd-sheet-security-posture = セキュリティ態勢
config-sshd-sheet-fine = 問題なし { $count } 件
config-sshd-sheet-set-in = { $src } で設定
config-sshd-sheet-sshds-default = sshd の既定値
config-sshd-sheet-set = { $v } に設定
config-sshd-sheet-turn-off-safely = 安全にオフにする…
config-sshd-sheet-not-set = 未設定
config-sshd-sheet-line = { $l } 行目
config-sshd-sheet-ignored-sets-it-first = { $at } · 無視: { $by } が先に設定
config-sshd-sheet-ignored-set-earlier = { $at } · 無視、先に設定済み
config-sshd-sheet-back-to-sshds-default-removes = sshd の既定値に戻す（行を削除）
config-sshd-sheet-remove-the-line = 行を削除
config-sshd-sheet-default = 既定

## config-state
config-state-differs-from-the-baseline-only = ベースラインと異なります（ハッシュのみ保存）
config-state-was-not-found-on-this = このサーバーに { $file } が見つかりませんでした
config-state-new-job = 新しいジョブ

## config-structured-editor
config-structured-editor-recommended = 推奨
config-structured-editor-caution = 注意
config-structured-editor-weak = 弱い
config-structured-editor-never-on-prod = 本番では禁止
config-structured-editor-deny = 拒否
config-structured-editor-lets-run = { $field } に { $risk } の実行を許可
config-structured-editor-crow-ui = CROW 画面 · { $plugin_name } · { $kind }
config-structured-editor-edited = 編集済み（+{ $add_count } -{ $del_count }）
config-structured-editor-v-baseline = v{ $active_revision }（ベースライン）
config-structured-editor-text-view = テキスト表示
config-structured-editor-history = 履歴
config-structured-editor-revert = 元に戻す
config-structured-editor-stage-version = バージョンをステージ
config-structured-editor-rows-are-evaluated-top-to = 行は上から順に評価されます
config-structured-editor-1-value-here-isnt = ここの値 1 件は
config-structured-editor-values-here-arent = ここの値 { $not_running } 件は
config-structured-editor-not-running-yet-what-the = 未反映 · { $what }カーネルが現在使っている値と違います。起動時か sysctl --system で反映されます
config-structured-editor-managed-by-it-rewrites-this = { $m } が管理 · このファイルを書き換えるため、ここでの編集は上書きされます。DNS の設定は { $m2 } で変更してください
config-structured-editor-read-only = 読み取り専用 · { $r }
config-structured-editor-edit-rejected = 編集を拒否 · { $e }
config-structured-editor-line = 行
config-structured-editor-add-directive = + ディレクティブを追加
config-structured-editor-no-entries-only-comments-or = 項目なし — コメントか空行だけです。
config-structured-editor-move-up-rules-are-read = 上へ移動: ルールは上から順に読まれます
config-structured-editor-scoped-to-match-read-only = Match { $s } に限定 — crow-config が Match ブロックを扱えるまで読み取り専用（ERR-12）
config-structured-editor-add-row = + 行を追加
config-structured-editor-add = 追加
config-structured-editor-added-at-the-end-of = このブロックの末尾に追加します。新しいブロックはテキスト表示で入力してください。
config-structured-editor-added-at-the-end-of-2 = ファイルの末尾に追加します。
config-structured-editor-added-at-the-end-of-3 = このセクションの末尾に追加します。新しいセクションはテキスト表示で入力してください。
config-structured-editor-this-change-introduces-values-rated = この変更は「本番では禁止」の値を含みます:
config-structured-editor-type-to-stage-it = ステージするには { $risk_confirm_keyword } と入力:
config-structured-editor-stage-anyway = それでもステージ
config-structured-editor-cancel = キャンセル
config-structured-editor-order-matters = 順序が重要 · { $ir }

## containers
containers-running = 実行中: { $b }…
containers-reading-the-servers-containers = サーバーのコンテナーを読み取り中…
containers-couldnt-read-the-server = サーバーを読み取れませんでした: { $e }
containers-containers = コンテナー · { $server_name }
containers-running-2 = { $running }/{ $count } 実行中
containers-reading = 読み取り中…
containers-refresh = 更新
containers-fix-it-on-the-server = サーバー側で直してください
containers-neither-docker-nor-podman-is = このサーバーには Docker も Podman もインストールされていません。
containers-no-container-runtime-crow-can = ここには Crow が使えるコンテナーランタイムがありません。
containers-no-containers-on-this-server = このサーバーにコンテナーはありません。
containers-standalone = 単体
containers-not-in-a-compose-stack = compose スタック外 { $count } 個
containers-up = 起動
containers-restart = 再起動
containers-down = 停止
containers-running-3 = { $running }/{ $count } 実行中
containers-create-and-start-every-service = このスタックのすべてのサービスを作成して起動しますか（イメージを取得する場合があります）？
containers-restart-every-container-in-this = このスタックのすべてのコンテナーを再起動しますか？それぞれ一時的に停止します。
containers-stop-and-remove-this-stacks = このスタックのコンテナーとネットワークを停止して削除しますか？名前付きボリュームは残ります。
containers-cancel = キャンセル
containers-name = 名前
containers-image = イメージ
containers-state = 状態
containers-published-ports = 公開ポート
containers-restarts = 再起動回数
containers-runtime = ランタイム
containers-all-nets = 全ネット
containers-root = {" "}· root
containers-no-output-yet = まだ出力はありません。
containers-reading-logs = ログを読み取り中…
containers-reading-2 = 読み取り中…
containers-start = 開始
containers-stop = 停止
containers-start-this-container = このコンテナーを開始しますか？
containers-stop-this-container-what-it = このコンテナーを停止しますか？再び開始するまで、提供しているものは止まります。
containers-restart-this-container-its-down = このコンテナーを再起動しますか？一時的に停止します。
containers-logs = ログ
containers-inspect = 詳細
containers-newest-first = 新しい順
containers-following = 追跡中
containers-follow = 追跡
containers-environment-values-hidden = 環境変数の値は非表示

## files-collector
files-collector-binary-file-nothing-to-show = バイナリファイル: テキストとして表示できるものはありません
files-collector-invalid-folder-name = 無効なフォルダー名

## files-models
files-models-kb = { $size_bytes } KB
files-models-mb = { $size_bytes } MB
files-models-gb = { $size_bytes } GB

## files
files-files = ファイル
files-up = 上へ
files-folder = + フォルダー
files-name = 名前
files-size = サイズ
files-perms = 権限
files-owner = 所有者
files-group = グループ
files-modified = 更新日時
files-empty-directory = 空のフォルダー
files-create = 作成 ⏎
files-cancel = キャンセル
files-edit-in-a-terminal-nano = ターミナルで編集（nano）
files-delete-empty-directory-this-cannot = 空のフォルダー { $del_name } を削除しますか？元に戻せません。

files-delete-this-cannot-be-undone = { $del_name } を削除しますか？元に戻せません。
files-cancel-2 = キャンセル
files-delete = 削除 ⏎
files-others = その他
files-everyone-else = それ以外の全員
files-setuid = SETUID
files-runs-as-its-owner = 所有者として実行
files-setgid = SETGID
files-new-files-get-this-group = 新しいファイルがこのグループになる
files-runs-as-its-group = グループとして実行
files-sticky = STICKY
files-only-owners-delete-inside = 中身は所有者だけが削除できる
files-anyone-on-the-server-can = サーバー上の誰でもこのファイルを変更できます
files-applying = 適用中…
files-chmod = chmod { $mode }
files-reading = 読み取り中…
files-first-kb-lines = 先頭 { $collector } KB · { $lines } 行
files-text = { $lines } { $lines ->
       *[other] 行
    }
files-edit-in-terminal = ターミナルで編集
files-reading-the-file = ファイルを読み取り中…

## firewall-detector
firewall-detector-ufw-is-active-but-its = ufw は有効ですが、状態の出力を解析できませんでした。
firewall-detector-ufw-is-installed-but-inactive = UFW はインストールされていますが無効です。システムの netfilter によるパケットフィルタリングは無効です。
firewall-detector-ufw-is-installed-but-crow = ufw はインストールされていますが、Crow は root としてルールを読み取れませんでした: { $e }
firewall-detector-firewalld-is-installed-but-not = firewalld はインストールされていますが動いていないため、何もフィルタリングしていません。
firewall-detector-firewalld-is-running-but-crow = firewalld は動いていますが、Crow は root としてゾーンを読み取れませんでした: { $e }
firewall-detector-nftables-iptables-are-here-but = nftables/iptables はありますが、Crow は root としてルールセットを読み取れませんでした: { $e }
firewall-detector-no-firewall-tooling-ufw-firewalld = このホストにはファイアウォールのツール（ufw、firewalld、nftables、iptables）が見つかりませんでした。

## firewalld
firewalld-from = { $z } から
firewalld-on = { $z } 上
firewalld-every-interface-not-in-another = ほかのゾーンにないすべてのインターフェイス
firewalld-zone-accepts-all-traffic = ゾーン { $name } はすべての通信を許可（{ $binding }）
firewalld-service-ports-not-known-zone = サービス { $svc }（ポート不明）· ゾーン { $name }
firewalld-service-zone = サービス { $svc } · ゾーン { $name }
firewalld-port-zone = ポート · ゾーン { $name }
firewalld-rich-rule-zone = リッチルール · ゾーン { $name }
firewalld-rich-rule-service-zone = リッチルール、サービス { $svc } · ゾーン { $name }
firewalld-rich-rule-not-interpreted = リッチルール（未解釈）: { $raw_rule }
firewalld-the-running-firewall-differs-from = 動作中のファイアウォールが保存済み（permanent）の設定と異なります: 再読み込みや再起動で実行時のみの変更は失われます。

## firewall-lockout
firewall-lockout-crow-cant-see-which-rules = この後どのルールが適用されるか Crow には分からないため、接続に使っている SSH ポート { $port }/tcp が許可されたままか確認できません。
firewall-lockout-after-this-port-tcp-crows = この後、ポート { $port }/tcp（Crow の SSH 接続）は { $value } からしか許可されません。Crow がそこから接続していなければ、次のログインは拒否されます。
firewall-lockout-this-stops-port-tcp-the = これにより、Crow が接続に使っている SSH ポート { $port }/tcp が許可されなくなります。Crow を含め、次の SSH ログインは拒否されます。

## firewall-models
firewall-models-ufw-uncomplicated-firewall = UFW（Uncomplicated Firewall）
firewall-models-unmanaged-none-detected = 管理外 / 検出なし
firewall-models-in = 受信
firewall-models-out = 送信
firewall-models-fwd = 転送
firewall-models-allow = 許可
firewall-models-deny = 拒否
firewall-models-reject = 拒絶
firewall-models-limit = 制限
firewall-models-tcp = TCP
firewall-models-udp = UDP
firewall-models-tcp-udp = TCP+UDP
firewall-models-any = すべて
firewall-models-tcp-udp-2 = { $port }/tcp,udp
firewall-models-firewall-allowed = ファイアウォール: 許可
firewall-models-firewall-blocked = ファイアウォール: ブロック
firewall-models-firewall-allow-default = ファイアウォール: 許可（既定）
firewall-models-firewall-no-inbound-rule = ファイアウォール: 受信ルールなし
firewall-models-firewall-inactive = ファイアウォール: 無効
firewall-models-firewall-not-read = ファイアウォール: 未読み取り

## firewall
firewall-firewall-is-currently-disabled-by = ファイアウォールは管理者によって無効にされています。
firewall-all = すべて
firewall-allow = 許可
firewall-deny = 拒否
firewall-limit = 制限
firewall-firewall = ファイアウォール
firewall-on = { $short_name } · 有効 · { $total_rules } { $total_rules ->
       *[other] 件のルール
    }
firewall-stage = ステージ +{ $add_count } −{ $del_count }
firewall-staged-firewall-rule-changes = ステージしたファイアウォールのルール変更
firewall-audit = 監査
firewall-rule-table = ルール表
firewall-raw-ruleset = 生のルールセット
firewall-reload = 再読み込み
firewall-disable = 無効にする
firewall-add-rule = + ルールを追加
firewall-defaults = 既定
firewall-quick-open = クイック開放
firewall-action = 動作
firewall-dir = 方向
firewall-port = ポート
firewall-from = 送信元
firewall-to = 宛先
firewall-comment = コメント
firewall-no-rules-yet-everything-follows = ルールはまだありません: すべて上の既定に従います
firewall-no-rules-match-try-all = 一致するルールがありません: すべて を選ぶか、検索を消してください
firewall-running = 実行中: { $p } …
firewall-read-only-crow-reads-the = 読み取り専用: Crow は { $label } のルールセット（入力チェーンとそのジャンプ先）を読み取りますが、生のルールセットは書き込みません。
firewall-read-only-crow-reads-the-2 = 読み取り専用: Crow は { $label }（動作中の設定）を読み取りますが、まだ変更はしません。編集はそのツールで行ってください。
firewall-changes-go-to-firewallds-running = 変更は firewalld の動作中の設定と保存済み（permanent）の設定の両方に入るため、すぐに反映され、再読み込みでも残ります。
firewall-this-could-lock-you-out = 締め出される可能性があります
firewall-type-confirm-to-apply-it = それでも適用するには CONFIRM と入力:
firewall-apply-anyway = それでも適用
firewall-cancel = キャンセル

## firewall-new-rule-modal
firewall-new-rule-modal-anywhere = どこからでも
firewall-new-rule-modal-will-run-running-saved-config = 実行内容（動作中 + 保存済みの設定）
firewall-new-rule-modal-not-valid-for-firewalld = FIREWALLD では無効
firewall-new-rule-modal-will-run = 実行内容
firewall-new-rule-modal-add-rule = ルールを追加
firewall-new-rule-modal-action = 動作
firewall-new-rule-modal-allow = 許可
firewall-new-rule-modal-deny = 拒否
firewall-new-rule-modal-reject = 拒絶
firewall-new-rule-modal-limit = 制限
firewall-new-rule-modal-protocol = プロトコル
firewall-new-rule-modal-tcp = TCP
firewall-new-rule-modal-udp = UDP
firewall-new-rule-modal-tcp-udp = TCP+UDP
firewall-new-rule-modal-any = すべて
firewall-new-rule-modal-port = ポート
firewall-new-rule-modal-from-2 = 送信元
firewall-new-rule-modal-comment-2 = コメント
firewall-new-rule-modal-enter-adds-it-esc-or = Enter で追加 · Esc か外側のクリックでキャンセル
firewall-new-rule-modal-cancel = キャンセル

## firewall-non-operational
firewall-non-operational-no-firewall-running = ファイアウォールが動いていません
firewall-non-operational-why = 理由
firewall-non-operational-firewall-tools-on-the-host = ホスト上のファイアウォールのツール
firewall-non-operational-no-standard-packet-filter-binaries = 標準的なパケットフィルターのバイナリ（ufw、firewalld、iptables）が見つかりません
firewall-non-operational-to-turn-it-on = 有効にするには:

firewall-non-operational-to-set-one-up-ufw = 設定するには（ufw）:
firewall-non-operational-turn-the-firewall-on = ファイアウォールを有効にする

## fleet-archived
fleet-archived-active-fleet = 稼働中のフリート（{ $active_n }）
fleet-archived-archived = アーカイブ済み（{ $archived_n }）
fleet-archived-danger-zone = 危険ゾーン
fleet-archived-host = ホスト
fleet-archived-address = アドレス
fleet-archived-env = 環境
fleet-archived-role = 役割
fleet-archived-archived-2 = アーカイブ日
fleet-archived-stored-data = 保存データ
fleet-archived-restore = 復元
fleet-archived-no-archived-servers = アーカイブ済みのサーバーはありません
fleet-archived-archiving-takes-a-server-out = アーカイブはサーバーを削除せずにフリートから外します。いきなり削除されるものは何もありません。
fleet-archived-purge-log = 削除ログ
fleet-archived-record-s-crows-own-audit = { $count } 件の記録 · Crow 自身の監査証跡
fleet-archived-nothing-has-been-purged-yet = まだ何も削除されていません。
fleet-archived-archive = { $name } をアーカイブ
fleet-archived-leaves-the-active-fleet-crow = { $name } は稼働中のフリートから外れます。Crow はポーリングをやめ、SSH 接続を閉じます。
fleet-archived-its-stored-data-change-history = 保存データ（変更履歴と鍵の関連付け）は { $purge_due } に削除されます。
fleet-archived-the-server-record-itself-is = サーバーの記録そのものは削除されません: アドレス、ポート、ユーザー、鍵、踏み台、環境、役割、グループ、タグ、ホスト鍵のフィンガープリント、OS の情報はすべて残ります。削除が実行されるまではアーカイブ済みタブから復元でき、その後は履歴なしで戻ります。
fleet-archived-cancel = キャンセル
fleet-archived-archive-server = サーバーをアーカイブ

## fleet-certificates
fleet-certificates-certificates = 証明書
fleet-certificates-served-expiring-within-days-not = 使用中 { $total } · { $cert_warn_days } 日以内に期限切れ { $soon } · 自動更新されないサーバー { $broken } { $broken ->
       *[other] 台
    }
fleet-certificates-read-again = 再読み取り
fleet-certificates-reading = { $count } 台を読み取り中…
fleet-certificates-renew-runs-certbots-dry-run = 更新 は、まず certbot のドライランを行い、次に更新し、nginx か Apache を確認して再読み込み（壊れた設定では行いません）、最後に証明書を読み直して新しい期限を確認します。certbot は期限の 30 日前以内でしか更新せず、Let's Encrypt は証明書の発行回数を制限しています。
fleet-certificates-no-renewal-tool = 更新ツールなし
fleet-certificates-couldnt-read = 読み取れませんでした: { $e }
fleet-certificates-reading-2 = 読み取り中…
fleet-certificates-renews-on-its-own = 自動で更新
fleet-certificates-no-certificates-found-lets-encrypts = 証明書が見つかりません（Let's Encrypt の live フォルダー、nginx と Apache の設定）。
fleet-certificates-no-name = （名前なし）
fleet-certificates-expired-days-ago = { $days } 日前に期限切れ
fleet-certificates-days-left = 残り { $days } 日
fleet-certificates-renew = 更新
fleet-certificates-not-certbot-acme-sh = certbot/acme.sh ではない
fleet-certificates-nothing-read-yet-read-again = まだ何も読み取っていません: 再読み取り で接続できるすべてのサーバーを読み取ります。

## fleet-checks
fleet-checks-checks-from-outside = 外部からのチェック
fleet-checks-checks-failing = チェック { $count } 件 · 失敗 { $failing } 件
fleet-checks-run-from-this-machine-only = このマシンから、Crow が開いている間だけ実行
fleet-checks-http-s = HTTP(S)
fleet-checks-tcp-port = TCP ポート
fleet-checks-dns-name = DNS 名
fleet-checks-every = 間隔
fleet-checks-1-min = 1 分
fleet-checks-5-min = 5 分
fleet-checks-attach-to = 関連付け先
fleet-checks-no-server = サーバーなし
fleet-checks-add-check = チェックを追加
fleet-checks-an-attached-checks-alerts-land = 関連付けたチェックのアラートはそのサーバーに届きます（ページ、マップ、メール）。チェックは 2 回続けて失敗するとアラートになり、最初の成功で解決します。
fleet-checks-on = {" "}· { $s }
fleet-checks-cert-days = {" "}· 証明書 { $d } 日
fleet-checks-checking = 確認中…
fleet-checks-not-checked-yet = 未確認
fleet-checks-alerts-on = アラート オン
fleet-checks-alerts-off = アラート オフ
fleet-checks-no-checks-yet-add-a = チェックはまだありません: 上でウェブサイト、ポート、DNS 名を追加してください。
fleet-checks-every-s = { $target } · { $every_secs } 秒ごと{ $server }

## fleet-drift
fleet-drift-not-yet = まだ
fleet-drift-just-now = たった今
fleet-drift-min-ago = { $s } 分前
fleet-drift-h-ago = { $s } 時間前
fleet-drift-search-every-servers-configs = 全サーバーの設定を検索
fleet-drift-from-what-crow-has-read = Crow が読み取った内容から検索。サーバーには接続しません
fleet-drift-nothing-found-crow-searches-the = 見つかりませんでした。Crow は読み取ったファイルを検索します: サーバーの設定画面を開くとファイルを読み取ります。
fleet-drift-first-matches = 最初の { $n } 件
fleet-drift-text = { $n } { $n ->
       *[other] 件一致
    }
fleet-drift-the-file = そのファイル
fleet-drift-drift-from-baselines = ベースラインからのドリフト
fleet-drift-drifted-files-re-read-every = ドリフト { $drifted } 件 · Crow の実行中は 15 分ごとにファイルを再読み取り · 確認 { $ago }
fleet-drift-check-now = 今すぐ確認
fleet-drift-reading = { $count } 台を読み取り中…
fleet-drift-no-baselines-pinned-yet-pin = 固定されたベースラインはまだありません。サーバーの設定画面（ファイルの履歴 → ベースラインに固定）で固定すると、そのグループまたはフリートのすべてのサーバーがそれと比較されます。
fleet-drift-drifted = ドリフト { $n_drift } 件
fleet-drift-all-match = すべて一致
fleet-drift-bring-all-back = { $n_drift } 件すべてを戻す
fleet-drift-matches-comments-spacing-differ = 一致（コメントや空白のみ違う）
fleet-drift-text-2 = { $count } { $count ->
       *[other] 件の差分
    }: { $cloned }
fleet-drift-vs-baseline = { $scope } のベースラインと比較
fleet-drift-what-changed = 変更点
fleet-drift-baseline-vs-server = { $name } · { $path } · ベースライン（−）とサーバー（+）
fleet-drift-crow-kept-only-hashes-of = Crow はどちらか一方のハッシュしか保存していません: ファイルの内容はボールトの鍵で封印されますが、読み取ったときに鍵がなかった（または現在ボールトがロックされている）ためです。違うことは分かりますが、どう違うかは分からず、ベースラインの送信やこのコピーの固定もできません。ボールトを設定してから（設定 → ボールトとセキュリティ）、今すぐ確認 を押してください。
fleet-drift-bring-back-writes-the-baseline = 戻す はこのサーバーのコピーをベースラインで上書きします（検証ツールで確認し、履歴に保存）。承認 はこのコピーをベースラインにします: 範囲内のほかのすべてのサーバーがこれと比較されます。

## fleet-dr
fleet-dr-plans = プラン
fleet-dr-new = + 新規
fleet-dr-no-plans-yet-a-plan = プランはまだありません。プランには、サーバーで何が重要か、バックアップの保存先、どれだけ早く復旧すべきか、再構築の方法を書きます。
fleet-dr-drill-min = {" "}· 訓練 { $minutes } 分{ $d }
fleet-dr-name = 名前
fleet-dr-rto-min = RTO（分）
fleet-dr-rpo-min = RPO（分）
fleet-dr-for = 対象
fleet-dr-group = グループ { $g }
fleet-dr-what-matters = 重要なもの
fleet-dr-backups-path-or-glob-timer = バックアップ（パスまたはグロブ | タイマー）
fleet-dr-runbook-markdown-with-commands = 手順書（Markdown、コマンド付き）
fleet-dr-save = 保存
fleet-dr-checking = 確認中…
fleet-dr-check-against-the-servers = サーバーと照合

fleet-dr-export-md = 書き出し（.md）
fleet-dr-delete-plan = プランを削除
fleet-dr-drills = 訓練
fleet-dr-rebuild-it-for-real-somewhere = 害のない場所（Multipass VM や作業用サーバー）で、手順書に従って実際に再構築してください。Crow が RTO と比べて時間を計ります。
fleet-dr-running-for-min = { $chrono } 分経過
fleet-dr-finish-drill = 訓練を終了
fleet-dr-start-a-drill = 訓練を開始
fleet-dr-met-the-rto = RTO を達成
fleet-dr-missed-the-rto = RTO を未達
fleet-dr-min = { $when } · { $minutes } 分 · { $d }{ $d2 }

## fleet-group-modal
fleet-group-modal-server = サーバー
fleet-group-modal-group = グループ · { $server_name }
fleet-group-modal-move-this-server-to-a = このサーバーを移すグループ:
fleet-group-modal-no-group = グループなし

## fleet-hardening
fleet-hardening-not-yet = まだ
fleet-hardening-min-ago = { $chrono } 分前
fleet-hardening-posture = 態勢
fleet-hardening-open-critical-accepted = 未対応 { $open } 件（重大 { $crit } 件）· 許容 { $accepted } 件
fleet-hardening-read-as-root-where-crow = 読み取り { $checked }、可能な場合は root として（sshd -T、ファイアウォールの状態、ss）
fleet-hardening-reading = 読み取り中…
fleet-hardening-check-now = 今すぐ確認
fleet-hardening-export-report-md = レポートを書き出し（.md）
fleet-hardening-reason-to-accept = 許容する理由
fleet-hardening-fix-everywhere = すべてで修正:
fleet-hardening-every-fix-first-proves-a = 各修正は、まず新規の鍵ログインが使えることを確かめてから 1 つだけ変更し、反映されたことを確認し（sshd -t、sshd -T、ファイアウォールの状態）、新規ログインがまだ使えることを確かめます。失敗したものは元に戻します。Crow が root でログインするサーバーで PermitRootLogin no にすることはなく、ファイアウォールは常に SSH ポートを許可します。
fleet-hardening-crow-logs-in-as = Crow は { $user } でログイン
fleet-hardening-not-read-unreachable-or-not = 未読み取り（接続不可、またはまだ確認していない）
fleet-hardening-nothing-found = 問題なし
fleet-hardening-text = { $count } { $count ->
       *[other] 件の指摘
    }
fleet-hardening-accepted-by = { $by } が許容: { $reason }
fleet-hardening-fix = 修正: { $f }

## fleet-import
fleet-import-import-from-providers = プロバイダーから取り込み
fleet-import-asking-account-s = { $loading } 個のアカウントに問い合わせ中…
fleet-import-instance-s-not-in-the = インスタンス { $count } 個 · フリート外 { $importable } 個
fleet-import-instances-already-in-the-fleet = すでにフリートにあるインスタンスは IP で照合して関連付けるため、リージョンはプロバイダーから取得します。取り込み を押すと入力済みの サーバーを追加 が開き、ホスト鍵とログインは通常どおり確認されます。
fleet-import-linked-enrolled-server-s-to = 登録済みのサーバー { $linked } 台をインスタンスに関連付けました。
fleet-import-no-address = アドレスなし
fleet-import-in-fleet-as = フリート上の名前: { $server }
fleet-import-import = 取り込み →

## fleet-incidents
fleet-incidents-from-an-alert-last-7 = アラートから（過去 7 日）
fleet-incidents-no-alerts-in-the-last = 過去 7 日間にアラートはありません。
fleet-incidents-resolved = {" "}· 解決済み
fleet-incidents-or-a-window-on-a = またはサーバーの期間を指定
fleet-incidents-pick-an-alert-or-a = アラートか期間を選んでください: タイムラインはその前後のアラート、Crow 自身の変更、失敗したチェック、エラーログ（err 以上）をまとめます。
fleet-incidents-reading-logs = {" "}· ログを読み取り中…
fleet-incidents-summarizing = 要約中…
fleet-incidents-summarize-clanker = 要約（CLANKER）
fleet-incidents-export-md = 書き出し（.md）
fleet-incidents-via-a-likely-sequence-an = { $via } による · ありそうな経緯であり AI の解釈です: タイムラインと照らし合わせてください
fleet-incidents-nothing-in-this-window = この期間には何もありません。
fleet-incidents-events = { $when } → { $when2 } · { $count } 件のイベント{ $st }

## fleet-lab-modal
fleet-lab-modal-local-vms-containers = ローカル VM とコンテナー
fleet-lab-modal-multipass-vms-and-lab-containers = このマシン上の Multipass VM とラボコンテナー（Crow に登録済み）
fleet-lab-modal-multipass-vms = MULTIPASS VM
fleet-lab-modal-ubuntu-vms-on-this-machine = このマシン上の Ubuntu VM: 起動、取り込み、電源操作
fleet-lab-modal-local-lab-containers = ローカルのラボコンテナー
fleet-lab-modal-lab-containers-on-this-machine = このマシン上のラボコンテナー（テストノードとして Crow に登録）
fleet-lab-modal-checking = 確認中…
fleet-lab-modal-not-installed = 未インストール
fleet-lab-modal-discovered-local-test-nodes = 見つかったローカルのテストノード
fleet-lab-modal-detected = { $count } 個検出
fleet-lab-modal-refresh = 更新 🔄
fleet-lab-modal-no-local-test-nodes-found = ローカルのテストノードは見つかりません
fleet-lab-modal-use-the-form-below-to = 下のフォームで新しい Ubuntu か Debian のテストノードを作成できます
fleet-lab-modal-close = 閉じる
fleet-lab-modal-endpoint = { $distro_display } · エンドポイント: { $endpoint_display }
fleet-lab-modal-stop = ■ 停止
fleet-lab-modal-start = ▶ 開始
fleet-lab-modal-enrolled = ✓ 登録済み
fleet-lab-modal-enroll-in-crow = + CROW に登録
fleet-lab-modal-checking-2 = 確認中…
fleet-lab-modal-not-installed-2 = 未インストール
fleet-lab-modal-daemon-not-running = デーモンが動いていません
fleet-lab-modal-needs-a-fix = { $daemon } · 修正が必要
fleet-lab-modal-ready = 準備完了 · { $daemon }
fleet-lab-modal-ubuntu-vms-on-this-machine-2 = このマシン上の Ubuntu VM。Crow は自分の鍵で起動し、Multipass 経由でホスト鍵を固定して、SSH サーバーとしてフリートに追加します。
fleet-lab-modal-no-vms-yet-launch-one = VM はまだありません: 下で起動してください。
fleet-lab-modal-copy = コピー
fleet-lab-modal-command-copied = コマンドをコピーしました
fleet-lab-modal-in-fleet = フリートにあり
fleet-lab-modal-import-to-crow = CROW に取り込み
fleet-lab-modal-no-address = アドレスなし
fleet-lab-modal-its-fleet-entry-goes-to = {" "}フリートの項目はアーカイブに移ります。
fleet-lab-modal-delete-for-good = 完全に削除
fleet-lab-modal-cancel = キャンセル
fleet-lab-modal-launch-a-vm = VM を起動
fleet-lab-modal-cpu = CPU
fleet-lab-modal-memory = メモリー
fleet-lab-modal-gb = { $m } GB
fleet-lab-modal-disk = ディスク
fleet-lab-modal-gb-2 = { $d } GB
fleet-lab-modal-newest-lts = 最新の LTS
fleet-lab-modal-launch-add-to-fleet = 起動してフリートに追加
fleet-lab-modal-delete-and-its-disk-for = { $name } とそのディスクを完全に削除しますか？{ $in_fleet }

## fleet-lab-state
fleet-lab-state-starting = { $name } を開始中…
fleet-lab-state-stopping = { $name } を停止中…
fleet-lab-state-restarting = { $name } を再起動中…
fleet-lab-state-suspending = { $name } を一時停止中…
fleet-lab-state-deleting-and-its-disk = { $name } とそのディスクを削除中…

## fleet-logsearch
fleet-logsearch-search = 検索
fleet-logsearch-priority = 優先度
fleet-logsearch-when = 期間
fleet-logsearch-at-most = 最大
fleet-logsearch-where = 対象
fleet-logsearch-every-server = すべてのサーバー
fleet-logsearch-group = グループ { $g }
fleet-logsearch-search-2 = 検索 ⏎
fleet-logsearch-searching = { $count } 台を検索中…
fleet-logsearch-save-as = 名前を付けて保存
fleet-logsearch-save = 保存
fleet-logsearch-saved = 保存済み
fleet-logsearch-alert = 通知 { $level }
fleet-logsearch-alert-off = 通知 オフ
fleet-logsearch-in = {" "}（{ $unit }）
fleet-logsearch-prio = {" "}· 優先度 ≤{ $p }
fleet-logsearch-group-2 = グループ { $group }

fleet-logsearch-an-alerting-search-runs-every = 通知付きの検索は、Crow が開いている間、10 分ごとに直近 15 分を対象サーバーで実行します。一致しなくなるとアラートは解決します（Email プラグインでメール送信）。
fleet-logsearch-lines = { $count } 行
fleet-logsearch-limit = {" "}（上限）
fleet-logsearch-summarizing = 要約中…
fleet-logsearch-summarize-clanker = 要約（CLANKER）
fleet-logsearch-via-only-the-matching-lines = { $via } による · 送信したのは一致した行のみ · AI の解釈です: 確認してください

## fleet-overview
fleet-overview-none-enrolled = 未登録
fleet-overview-1-group = 1 グループ
fleet-overview-groups = { $n } グループ
fleet-overview-1-region = 1 リージョン · { $group_note }
fleet-overview-regions = { $n } リージョン · { $group_note }
fleet-overview-avg-1m-load-vcpu = 1 分平均の負荷 · { $total_vcpu } vCPU
fleet-overview-no-metrics-yet = メトリクスなし
fleet-overview-not-read-yet = 未読み取り
fleet-overview-past-the-rotation-policy = 更新期限超過 { $overdue } · { $note }
fleet-overview-not-read-yet-2 = { $note } · 未読み取り { $unknown }
fleet-overview-not-computed-yet = 未計算
fleet-overview-no-baselines-pinned = 固定されたベースラインなし
fleet-overview-match-baselines-as-last-read = { $count } { $count ->
       *[other] 件のファイル
    }がベースラインと一致 · 最終読み取り時点
fleet-overview-on-as-last-read = { $count } { $count ->
       *[other] 台
    } · 最終読み取り時点
fleet-overview-servers = サーバー
fleet-overview-open-alerts = 未解決のアラート
fleet-overview-crit-warn = 重大 { $crit } · 警告 { $warn }
fleet-overview-config-drift = 設定のドリフト
fleet-overview-fleet-load = フリートの負荷
fleet-overview-oldest-host-key = 最古のホスト鍵
fleet-overview-connected = 接続済み
fleet-overview-reachable-over-their-transport = それぞれの経路で接続可能
fleet-overview-launch-vm = + VM を起動
fleet-overview-local-lab = ローカルラボ
fleet-overview-ok = OK
fleet-overview-checking = 確認中
fleet-overview-via = { $name } 経由の { $host }
fleet-overview-no-servers-enrolled = 登録済みのサーバーなし
fleet-overview-all-reachable = { $connected_count } 台すべて接続可能
fleet-overview-checking-2 = { $n } { $n ->
       *[other] 台
    }を確認中
fleet-overview-acknowledged = { $message } · 確認済み
fleet-overview-crow-wasnt-running-from-to = { $fmt } から { $fmt2 } まで Crow は動いていませんでした: その間は何も監視していません。
fleet-overview-all = すべて
fleet-overview-production = 本番
fleet-overview-staging = ステージング
fleet-overview-dev-lab = 開発 / ラボ
fleet-overview-by-region = リージョン別（{ $regions }）
fleet-overview-by-region-2 = リージョン別
fleet-overview-by-group = グループ別（{ $groups_count }）
fleet-overview-by-group-2 = グループ別
fleet-overview-hosts-sort-health-click-to = { $server_count } 台 · 状態順 ↓ · クリックで開く
fleet-overview-of-hosts-click-to-open = { $server_count } 台中 { $visible_count } 台 · クリックで開く
fleet-overview-unknown = 不明
fleet-overview-detecting = 検出中…
fleet-overview-detect-regions = リージョンを検出 ↻
fleet-overview-group = グループ:
fleet-overview-all-2 = すべて（{ $total_count }）
fleet-overview-no-group = グループなし
fleet-overview-create = 作成
fleet-overview-cancel = キャンセル
fleet-overview-new-group = + 新しいグループ
fleet-overview-host = ホスト
fleet-overview-address = アドレス
fleet-overview-os = OS
fleet-overview-role = 役割
fleet-overview-env = 環境
fleet-overview-cpu = CPU
fleet-overview-mem = メモリー
fleet-overview-disk = ディスク
fleet-overview-uptime = 稼働時間
fleet-overview-alerts = アラート
fleet-overview-no-servers-enrolled-2 = 登録済みのサーバーはありません
fleet-overview-connect-remote-linux-servers-via = SSH でリモートの Linux サーバーに接続するか、ローカルに隔離されたラボノードを作成してください。
fleet-overview-add-remote-server-ssh = + リモートサーバーを追加（SSH）
fleet-overview-fleet-alerts = フリートのアラート
fleet-overview-open = 未解決 { $count }
fleet-overview-ack-all = すべて確認
fleet-overview-activity = アクティビティ
fleet-overview-audit-log-all-hosts = 監査ログ · 全ホスト →
fleet-overview-no-recent-fleet-activity-recorded = 最近のフリートのアクティビティは記録されていません
fleet-overview-dismiss = 閉じる
fleet-overview-rolling-reboot = 順次再起動
fleet-overview-push-baseline-to-all = ベースラインを全台に送信
fleet-overview-revoke-sessions = セッションを無効化
fleet-overview-rotate-host-keys = ホスト鍵を更新
fleet-overview-fleet-wide = フリート全体
fleet-overview-one-host-at-a-time = 1 台ずつ · 最初の失敗で停止
fleet-overview-rolling-reboot-2 = 順次再起動
fleet-overview-reboots-every-reachable-server-in = 接続できるすべてのサーバーを順に再起動し、それぞれが起動し直してから次へ進みます。
fleet-overview-push-baseline = ベースラインを送信
fleet-overview-pushes-each-config-baseline-to = 各設定のベースラインを、コピーがドリフトしたすべてのサーバーに送信します。
fleet-overview-rotate-all-host-keys = すべてのホスト鍵を更新
fleet-overview-new-ssh-host-keys-on = すべてのサーバーに新しい SSH ホスト鍵を作り、known_hosts に固定し直して新規ログインで確認します。失敗すれば元に戻します。ほかのマシンからは鍵が変わったように見えます。
fleet-overview-revoke-all-sessions = すべてのセッションを無効化
fleet-overview-ends-every-ssh-login-on = すべてのサーバーで Crow 自身以外の SSH ログインをすべて終了します。コンソールのログインと鍵はそのままです: 鍵がまだ有効な人は再びログインできます。

## fleet-patching
fleet-patching-just-now = たった今
fleet-patching-min-ago = { $s } 分前
fleet-patching-h-ago = { $s } 時間前
fleet-patching-d-ago = { $s } 日前
fleet-patching-of-behind-on-security-updates = { $count } 台中 { $behind } 台がセキュリティ更新に遅れ · 再起動待ち { $reboots } 台
fleet-patching-every-server = すべてのサーバー
fleet-patching-group = グループ { $g }
fleet-patching-then-reboot-servers-that-need = その後、必要なサーバーを再起動（踏み台の後ろのサーバーを踏み台より先に）し、サービスの復帰を待つ
fleet-patching-each-run-lists-the-exact = 各実行はサーバーごとに正確なパッケージを一覧し、何かを実行する前に単語の入力を求めます。1 台ずつ実行し、最初の失敗で停止します。一覧にあるパッケージだけを更新します（apt --only-upgrade、dnf upgrade、apk add --upgrade）。グループの時間帯外のサーバー、接続できないサーバー、Crow が動いているマシンは理由とともに除外します。
fleet-patching-whos-behind = 遅れているサーバー
fleet-patching-read-every-6-h-while = 読み取り { $ago } · Crow の実行中は 6 時間ごと
fleet-patching-reading = { $count } 台を読み取り中…
fleet-patching-refresh = 更新を確認
fleet-patching-server = サーバー
fleet-patching-group-2 = グループ
fleet-patching-via = 経由
fleet-patching-security = セキュリティ
fleet-patching-all = すべて
fleet-patching-kernel = カーネル
fleet-patching-read = 読み取り
fleet-patching-reading-2 = 読み取り中…
fleet-patching-not-read-yet = 未読み取り
fleet-patching-couldnt-read = 読み取れませんでした: { $error }
fleet-patching-needed = 必要
fleet-patching-maintenance-windows = メンテナンス時間帯
fleet-patching-per-group-local-time-a = グループごと、現地時刻 · 実行は時間帯外のサーバーを除外 · 空欄: いつでも
fleet-patching-no-groups-yet-windows-are = グループはまだありません: 時間帯はグループごとに設定します（フリート → グループ別）。

fleet-patching-open-now = 現在時間帯内
fleet-patching-closed-now = 現在時間帯外
fleet-patching-any-time = いつでも
fleet-patching-examples-sun-02-05-mon = 例: Sun 02-05 · Mon-Fri 22-06（深夜 0 時をまたぐ分は開始した日に属します）· Sat,Sun 01-04 · daily 03-05。Enter で保存。

## fleet-people
fleet-people-people = ユーザー
fleet-people-accounts-keys-servers-read = アカウント { $count } · 鍵 { $key_rows_count } · 読み取り済みサーバー { $scans_count }
fleet-people-read-again = 再読み取り
fleet-people-reading = { $count } 台を読み取り中…
fleet-people-couldnt-read = 読み取れませんでした: { $failed }
fleet-people-who-can-log-in-where = 誰がどこにログインできるか
fleet-people-text = { $count } { $count ->
       *[other] 台
    }
fleet-people-offboard-everywhere-their-keys-are = すべてのサーバーで退職処理: まず鍵を無効化し、次にアカウントをロックして期限切れにする（元に戻せる）か削除します（ホームは残す）。Crow がこのアカウントでログインするサーバーは除外します。
fleet-people-find-a-key-everywhere = 鍵を全サーバーで検索
fleet-people-every-key-in-any-authorized = すべての authorized_keys にあるすべての鍵。Crow の鍵なら名前付き
fleet-people-crow = {" "}（Crow）
fleet-people-skips-crows-own-logins = （Crow 自身のログインは除外）
fleet-people-an-account-with-their-key = グループのサーバーに、その人の鍵を持つアカウント（なければ作成）
fleet-people-every-server = すべてのサーバー
fleet-people-group = グループ { $g }
fleet-people-with-sudo = sudo 付き

## fleet-rollouts
fleet-rollouts-one-change-across-many-servers = 1 つの変更を多数のサーバーへ: まずカナリア（確認できるよう実行を一時停止）、次に残りを並行して動くバッチで。最初の失敗でそのサーバーの出力とともにすべて停止します。
fleet-rollouts-what = 内容
fleet-rollouts-a-command = コマンド
fleet-rollouts-a-config-file = 設定ファイル
fleet-rollouts-path-enter-reads-every-servers = パス（Enter で各サーバーのコピーを読み取り）
fleet-rollouts-read = 読み取り
fleet-rollouts-from-a-baseline = ベースラインから:
fleet-rollouts-new-content = 新しい内容
fleet-rollouts-each-server-checks-it-with = 各サーバーはまずその形式の検証ツール（sshd -t、nginx -t、visudo、fstab…）で確認します。拒否された内容は書き込みません。すべての書き込みはそのサーバーの設定履歴に残ります。
fleet-rollouts-where = 対象
fleet-rollouts-every-server = すべてのサーバー
fleet-rollouts-group = グループ { $g }
fleet-rollouts-left-out-unreachable-or-the = {" "}· { $count } 台を除外（接続不可、または Crow が動いているマシン）
fleet-rollouts-canary-runs-first-alone = カナリア · 最初に単独で実行
fleet-rollouts-no-canary = カナリアなし
fleet-rollouts-then-at-a-time = その後、同時に
fleet-rollouts-preview-the-run = 実行内容をプレビュー →
fleet-rollouts-a-command-has-no-preview = コマンドにはプレビューがありません: まずカナリアで試してください。
fleet-rollouts-what-changes-per-server = サーバーごとの変更点
fleet-rollouts-type-the-path-and-press = パスを入力して Enter を押すと、各サーバーのコピーを読み取ります。
fleet-rollouts-new-file = 新しいファイル
fleet-rollouts-reading = 読み取り中…
fleet-rollouts-identical-this-server-is-left = 同一: このサーバーは除外します。
fleet-rollouts-no-such-file-there-yet = そのファイルはまだありません: 作成されます。
fleet-rollouts-couldnt-read-it = 読み取れませんでした: { $e }
fleet-rollouts-reading-2 = 読み取り中…
fleet-rollouts-start-from-this-servers-copy = このサーバーのコピーから始める
fleet-rollouts-text = { $count } { $count ->
       *[other] 台
    }{ $left_out }

## fleet-run-panel
fleet-run-panel-nothing-to-run = 実行するものはありません。
fleet-run-panel-pausing-after-the-canary = 、カナリアの後で一時停止
fleet-run-panel-one-host-at-a-time = { $count } { $count ->
       *[other] 手順
    }、1 台ずつ。最初の失敗で実行を停止します。
fleet-run-panel-the-canary-is-done-check = カナリアが完了しました。確認してから 残りを続行 するか、ここで停止 してください。
fleet-run-panel-stopping-after-the-current-host = 現在のホストの後で停止します…
fleet-run-panel-running = 実行中…
fleet-run-panel-not-run = 未実行
fleet-run-panel-left-out = 除外 · { $why }
fleet-run-panel-type-to-start = 開始するには { $keyword } と入力
fleet-run-panel-continue-with-the-rest = 残りを続行
fleet-run-panel-stop-here = ここで停止
fleet-run-panel-stop-after-this-host = このホストの後で停止
fleet-run-panel-close = 閉じる
fleet-run-panel-cancel = キャンセル
fleet-run-panel-in-the-first-failure-stops = { $count } { $count ->
       *[other] 手順
    }、{ $groups_count } { $groups_count ->
       *[other] 段階
    }{ $run }。最初の失敗で実行を停止します。

## fleet-run
fleet-run-done = 完了 { $count }
fleet-run-already-fine = 対応済み { $skipped }
fleet-run-stopped-at = { $failed } で停止
fleet-run-stopped-on-request = 要求により停止
fleet-run-not-run = 未実行 { $not_run }

## fleet-setup
fleet-setup-default-unassigned = 既定 / 未割り当て
fleet-setup-close-esc = 閉じる esc
fleet-setup-enrolled-servers = 登録済みサーバー:
fleet-setup-fleet-groups = フリートのグループ:
fleet-setup-unique-tags = タグの種類:
fleet-setup-host-keys-pinned = 固定済みホスト鍵:
fleet-setup-none-enrolled = 未登録
fleet-setup-fleet-topology-group-assignments = グループの割り当て
fleet-setup-manage-new-group = + 管理 / 新しいグループ
fleet-setup-no-servers-currently-enrolled-in = 現在フリートに登録されているサーバーはありません。
fleet-setup-text = { $member_count } { $member_count ->
       *[other] 台
    }
fleet-setup-change-group = グループを変更 ▾
fleet-setup-host-keys = ホスト鍵
fleet-setup-crow-pins-each-servers-host = Crow は登録時に各サーバーのホスト鍵のフィンガープリントを固定し、変わった場合は接続を拒否します。経過年数は各サーバー上の鍵ファイルの日付から求めます。更新ポリシーはまだありません。
fleet-setup-fingerprint-pinned = フィンガープリント固定済み
fleet-setup-not-pinned = 未固定
fleet-setup-keys-under-a-year-old = 1 年未満の鍵
fleet-setup-keys-1-2-years-old = 1〜2 年の鍵
fleet-setup-keys-over-2-years-old = 2 年を超える鍵
fleet-setup-key-age-not-read-yet = 鍵の経過年数は未読み取り
fleet-setup-fleet-tag-taxonomy = フリートのタグ分類
fleet-setup-no-custom-tags-defined-assign = 独自のタグはありません。登録時やサーバー設定でノードにタグを付けて、インフラを分類してください。
fleet-setup-agentless-execution-guarantee = エージェントレス実行の保証
fleet-setup-crow-talks-plain-ssh-and = Crow は素の SSH で通信し、サーバーに何もインストールしません。フリート全体の操作はまだありませんが、実装されたら 1 台ずつ実行し、最初の失敗で停止します。
fleet-setup-text-2 = { $count } { $count ->
       *[other] 台
    }
fleet-setup-groups-policies = グループとポリシー
fleet-setup-patching = パッチ適用
fleet-setup-rollouts = ロールアウト
fleet-setup-drift-search = ドリフトと検索

fleet-setup-hardening = ハードニング
fleet-setup-certificates = 証明書
fleet-setup-people = ユーザーと鍵
fleet-setup-log-search = ログ検索
fleet-setup-checks = チェック
fleet-setup-incidents = インシデント
fleet-setup-dr-plans = DR プラン
fleet-setup-team = チーム
fleet-setup-audit-log = 監査ログ
fleet-setup-fleet-map = フリートマップ
fleet-setup-nav-change = 変更
fleet-setup-nav-security = セキュリティ
fleet-setup-nav-watch = 監視
fleet-setup-nav-organisation = 組織
fleet-setup-nav-plugins = プラグイン
fleet-setup-fleet-configuration = フリート設定
fleet-setup-fleet-configuration-about = フリートの運用方法と、サーバーに起きることを決める設定
fleet-setup-topology-dr = 構成と DR
fleet-setup-topology-dr-about = フリートの現状と、災害からの復旧
titlebar-topology-dr = 構成と DR
titlebar-fleet-configuration = フリート設定
app-palette-dr-plans = DR プラン

## fleet-state
fleet-state-no-data = データなし
fleet-state-connected-but-the-metrics-probe = 接続済みですが、メトリクスの取得に失敗しました
fleet-state-stopped = 停止
fleet-state-lab-container-isnt-answering = ラボコンテナーが応答しません
fleet-state-the-metrics-probe-failed-on = このマシンでメトリクスの取得に失敗しました

## fleet-team
fleet-team-team = チーム
fleet-team-team-mode-on = { $count } { $count ->
       *[other] 人
    } · チームモード オン
fleet-team-off-crow-is-single-user = オフ: Crow は 1 人用です
fleet-team-sign-out-switch = サインアウト / 切り替え
fleet-team-who-uses-this-crow-with = この Crow を誰が、どの役割で、どのサーバーに使うか。各メンバーは自分のパスフレーズでサインインし、サーバーを変更するすべての操作（サービス操作、設定の書き込み、ユーザーとファイアウォール、フリート実行）で役割が適用され、すべての変更記録に名前が残ります。メンバーはこのマシンのボールトに保存されます。複数のマシンで共有するには Crow アカウントが必要です（今後対応）。
fleet-team-you = {" "}（あなた）
fleet-team-fleet-runs-need-a-second = フリート実行には 2 人目のメンバーの承認が必要（実行時にその人のパスフレーズを入力）
fleet-team-add-a-member = メンバーを追加
fleet-team-start-a-team-the-first = チームを始める: 最初のメンバーは管理者（あなた）
fleet-team-role = 役割
fleet-team-custom = カスタム
fleet-team-can = 権限
fleet-team-on = 対象
fleet-team-group = グループ { $g }
fleet-team-env = 環境 { $e }
fleet-team-none-picked-the-whole-fleet = （未選択: フリート全体。環境がグループより優先）
fleet-team-add-member = メンバーを追加
fleet-team-start-the-team = チームを始める
fleet-team-whos-at-the-keyboard = 操作しているのは誰ですか？
fleet-team-this-crow-has-a-team = この Crow にはチームがあります: 役割を適用し、変更にあなたの名前を残すためにサインインしてください。
fleet-team-sign-in = サインイン

## history
history-at = { $fmt } 時点
history-cpu = CPU
history-memory = メモリー
history-load-1m = 負荷（1 分）
history-history = 履歴
history-one-sample-a-minute-while = Crow が開いている間は 1 分に 1 回取得 · { $readout }
history-now = { $chrono } → 現在
history-crow-wasnt-running-nothing-watched = Crow は動いていませんでした: 監視なし
history-server-unreachable = サーバーに接続不可
history-alert-open-warn = アラート発生中（警告）
history-alert-open-crit = アラート発生中（重大）

## lock
lock-crow-vault-locked = CROW のボールトはロック中
lock-password-2fa = パスワード + 2FA
lock-store-config-crow-crow-db = 保存先: ~/.config/crow/crow.db
lock-2fa-mandatory = 2FA: 必須
lock-master-password = マスターパスワード
lock-hide = 隠す
lock-show = 表示
lock-2fa-totp-code = 2FA の TOTP コード
lock-6-digits = 6 桁
lock-tab-to-switch-to-unlock = Tab で切り替え · ↵ でロック解除
lock-unlock-vault = ボールトのロックを解除

## lock-setup
lock-setup-security-notice-enabling-password-logon = セキュリティのお知らせ · パスワードログオンを有効にする
lock-setup-mandatory-2fa = 2FA 必須
lock-setup-you-are-about-to-activate = Crow のローカル暗号化ボールトの保護を有効にしようとしています。続ける前に、次の重要なセキュリティ上のルールを確認してください:
lock-setup-1-password-means-password-and = 1. パスワードとはパスワードと 2FA のこと
lock-setup-there-is-no-password-only = パスワードだけのログオンはありません。Crow はロック解除のたびに二要素認証（RFC 6238 TOTP）を求めます。次の手順で認証アプリ（1Password、Google Authenticator、YubiKey）を連携します。
lock-setup-2-what-the-password-protects = 2. パスワードが守るもの
lock-setup-crow-encrypts-its-secrets-provider = Crow はシークレット（プロバイダーのトークン、AI の API キー）を ChaCha20-Poly1305 で暗号化します。パスワードを設定すると、その鍵は OS のキーリングではなくパスワード（Argon2id、64 MB、3 回）でロックされ、暗号化されないままディスクに書かれることはなく、ロック時に RAM から消去されます。~/.config/crow/crow.db のそのほかの部分（サーバー一覧、設定、変更履歴）は暗号化されません。SSH 鍵は ~/.ssh のままです。
lock-setup-3-no-cloud-backup-or = 3. クラウドへのバックアップもパスワードのリセットもなし
lock-setup-this-is-a-local-only = これはテレメトリーのない、ローカル専用の 1 人用システムです。マスターパスワードか 2FA の認証アプリをなくすと、Crow が保存したシークレット（プロバイダーのトークン、AI のキー）は復元できず、もう一度入力する必要があります。サーバー一覧と設定はそのまま読めます。
lock-setup-cancel = キャンセル
lock-setup-i-understand-setup-password-2fa = 理解しました · パスワードと 2FA を設定 →
lock-setup-configure-password-mandatory-2fa = パスワードと必須の 2FA を設定
lock-setup-step-2-of-2 = 手順 2/2
lock-setup-master-password = マスターパスワード
lock-setup-hide-password = パスワードを隠す
lock-setup-show-password = パスワードを表示
lock-setup-confirm-master-password = マスターパスワード（確認）
lock-setup-mandatory-two-factor-authentication-totp = 必須の二要素認証（TOTP）
lock-setup-required = 必須
lock-setup-scan-the-code-with-your = 認証アプリ（1Password、Google Authenticator、Aegis…）でコードを読み取るか、このキーを入力してください:
lock-setup-copied = ✓ コピーしました
lock-setup-copy-key = キーをコピー
lock-setup-enter-6-digit-code-to = 連携を確認するため 6 桁のコードを入力:
lock-setup-back = ← 戻る
lock-setup-activate-password-2fa = パスワードと 2FA を有効にする

## logs
logs-systemd-journal-logs = systemd ジャーナルのログ
logs-entries = { $filtered_count }/{ $total_count } 件
logs-volatile-lost-on-reboot = ⚠ 揮発 — 再起動で消える
logs-no-age-limit = 期間の上限なし
logs-no-size-cap = サイズの上限なし
logs-persistent = { $days } · { $size } 永続
logs-explain = ✦ 説明
logs-boundaries = ⚙ 上限: { $bound_text }
logs-live-tail-on = ライブ表示 オン
logs-paused = 一時停止中
logs-export = 書き出し ⧉
logs-show-all = すべて表示
logs-clear = クリア
logs-range = 期間
logs-boot = 起動
logs-dedupe-repeats = 重複をまとめる
logs-quick = クイック
logs-load-older = ↑ さらに古い { $limit } 件を読み込む
logs-all = すべて
logs-err = ERR+
logs-warn = WARN+
logs-info = INFO+
logs-kernel = カーネル
logs-crow-actions = ✓ CROW の操作
logs-crow-actions-2 = CROW の操作
logs-pid = PID { $fpid }
logs-terminate-pid = PID { $fpid } を終了しますか？
logs-kill-sigterm = 終了（SIGTERM）⏎
logs-cancel = キャンセル
logs-kill-pid = ⏻ PID を終了
logs-timestamp = 日時
logs-level = レベル
logs-unit-identifier = ユニット / 識別子
logs-pid-2 = PID
logs-message = メッセージ
logs-no-journal-log-entries-matched = フィルターに一致するジャーナルのログはありません
logs-try-clearing-the-search-query = 検索を消すか、レベルで「すべて」を選んでください
logs-structured-systemd-journal-fields = systemd ジャーナルの構造化フィールド
logs-copy-message = メッセージをコピー
logs-copy-json = JSON をコピー
logs-asking-about-lines = { $lines_sent } 行について { $provider } に問い合わせ中…
logs-lines-sent = { $provider } · { $lines_sent } 行を送信
logs-get-a-plain-english-reading = 画面上のログ行をわかりやすく説明します: 何が起きているか、何がおかしそうか、考えられる原因、次に確認すること。

logs-explain-logs = ✦ ログを説明
logs-working = 処理中…
logs-no-provider = プロバイダーなし
logs-sends-the-log-lines-on = 画面上のログ行（ホスト名ではありません）を { $p } に送ります。クリックするまで何も送りません。
logs-add-an-api-key-for = 有効にするには、設定 → Clanker でプロバイダーの API キーを追加してください。

## logs-retention-modal
logs-retention-modal-systemd-journal-retention-boundaries = systemd ジャーナルの保持と上限
logs-retention-modal-eliminate-30-minute-silent-log = 30 分で黙って消えるログのローテーションをなくし、クラッシュによる消失を防ぐ
logs-retention-modal-critical-storage-is-volatile-auto = ⚠ 重大: 保存先が揮発 / 自動です
logs-retention-modal-if-this-host-crashes-or = このホストがクラッシュしたり電源が落ちたりすると、ログは RAM から完全に消えます。
logs-retention-modal-lock-to-disk = ディスクに固定 🔒
logs-retention-modal-crash-safe-persistent-storage-active = クラッシュに強い永続保存が有効
logs-retention-modal-active-journals-persist-in-var = 現在のジャーナルは /var/log/journal/ に保存され、クラッシュや再起動を越えて残ります。
logs-retention-modal-current-disk-usage = 現在のディスク使用量
logs-retention-modal-active-files = 使用中のファイル { $active_files_count } 個
logs-retention-modal-retention-horizon = 保持期間
logs-retention-modal-hours = { $retention_horizon_hours } 時間
logs-retention-modal-oldest = 最古: { $oldest_timestamp }
logs-retention-modal-daily-write-rate = 1 日の書き込み量
logs-retention-modal-mb-day = { $daily_burn_rate_mb } MB/日
logs-retention-modal-days-at-current-rate = 現在のペースで約 { $estimated_retained_days } 日
logs-retention-modal-allocation = 割り当て: { $disk_usage_display } / { $max_use_display }（{ $usage_pct }%）
logs-retention-modal-systemmaxuse-boundary = SystemMaxUse の上限
logs-retention-modal-storage-persistence-storage = 保存の永続性（Storage）
logs-retention-modal-controls-whether-logs-survive-host = ホストの再起動や停電の後もログが残るかを決めます
logs-retention-modal-persistent-rec = 永続（推奨）
logs-retention-modal-volatile-ram-only = 揮発（RAM のみ）
logs-retention-modal-auto-fragile = 自動（不安定）
logs-retention-modal-max-disk-quota-systemmaxuse = ディスク容量の上限（SystemMaxUse）
logs-retention-modal-hard-ceiling-before-older-journals = 古いジャーナルがローテートされる前の絶対的な上限
logs-retention-modal-500-mb = 500 MB
logs-retention-modal-2-0-gb = 2.0 GB
logs-retention-modal-4-0-gb-rec = 4.0 GB（推奨）
logs-retention-modal-8-0-gb = 8.0 GB
logs-retention-modal-16-gb = 16 GB
logs-retention-modal-custom = カスタム
logs-retention-modal-time-horizon-maxretentionsec = 保持期間（MaxRetentionSec）
logs-retention-modal-maximum-age-before-logs-are = ログが削除されるまでの最長期間
logs-retention-modal-7-days = 7 日
logs-retention-modal-14-days = 14 日
logs-retention-modal-30-days-rec = 30 日（推奨）
logs-retention-modal-90-days = 90 日
logs-retention-modal-180-days = 180 日
logs-retention-modal-unlimited = 無制限
logs-retention-modal-safety-disk-reserve-systemkeepfree = ディスクの安全な空き容量（SystemKeepFree）
logs-retention-modal-guarantees-disk-space-stays-free = ルートディスクが満杯でクラッシュしないよう、空き容量を確保します
logs-retention-modal-1-0-gb = 1.0 GB
logs-retention-modal-2-0-gb-rec = 2.0 GB（推奨）
logs-retention-modal-5-0-gb = 5.0 GB
logs-retention-modal-edit-in-crow-config = CROW-CONFIG で編集 ↗
logs-retention-modal-apply-to-etc-systemd-journald = /etc/systemd/journald.conf に適用

## logs-state
logs-state-oom-kills = OOM による強制終了
logs-state-segfaults = セグフォルト
logs-state-firewall-blocks = ファイアウォールのブロック
logs-state-disk-errors = ディスクエラー
logs-state-ssh-login-failures = SSH ログインの失敗

## onboard
onboard-connect = 接続
onboard-verify = 確認
onboard-name = 名前
onboard-check-connection = 接続を確認 →
onboard-next-name-it = 次へ: 名前を付ける →
onboard-add-server = サーバーを追加 ⏎
onboard-add-server-2 = サーバーを追加
onboard-cancel-esc = キャンセル esc
onboard-back = ← 戻る
onboard-where-is-it-and-how = どこにあり、Crow はどうログインしますか？
onboard-crow-connects-over-ssh-and = Crow は SSH で接続し、何もインストールしません。次の手順で、何かを送る前にサーバーのホスト鍵を確認します。
onboard-or-a-lab-container-on = またはこのマシン上のラボコンテナー
onboard-address = アドレス
onboard-port = ポート
onboard-user = ユーザー
onboard-reach-it = 接続経路
onboard-directly = 直接
onboard-via = { $route } 経由
onboard-only-reachable-through-another-server = 別のサーバー経由でしか届きませんか？先にそのサーバーを踏み台にしてください: ヘッダーの「踏み台」スイッチです。
onboard-every-connection-goes-through-the = すべての接続は踏み台を経由し、踏み台にはその踏み台自身の鍵でログインします。踏み台からこのサーバーへの届き方はネットワーク次第で、Crow に必要なのは踏み台から見たアドレスだけです。
onboard-pick-a-bastion-when-this = このサーバーに踏み台経由でしか届かない場合は踏み台を選んでください。
onboard-log-in-with = ログイン方法
onboard-a-key-in-crow = Crow の鍵
onboard-a-key-from-settings-keys = 設定 → 鍵 にある、すでにサーバーに設置された鍵。
onboard-a-password-once = パスワードを一度だけ
onboard-crow-logs-in-once-installs = Crow が一度だけログインして自分の鍵を設置し、パスワードは忘れます。
onboard-my-ssh-agent = 自分の SSH エージェント
onboard-whatever-your-agent-and-ssh = エージェントと ~/.ssh/config が使うもの。
onboard-no-keys-in-crow-yet = Crow にはまだ鍵がありません: 「パスワードを一度だけ」を使うか、設定 → 鍵と更新 で追加してください。
onboard-used-for-one-login-only = ホスト鍵を確認した後の 1 回のログインにだけ使います。保存しません。
onboard-crow-wont-know-which-key = Crow にはどの鍵か分からないため、更新や監査ができません。
onboard-checking = 確認中…
onboard-not-checked-yet = まだ確認していません。
onboard-no-answer = 応答なし
onboard-logging-in-and-reading-the = ログインしてサーバーを読み取り中…
onboard-reached-host-key-trusted-logged = 接続し、ホスト鍵を信頼し、ログインしました。
onboard-the-host-key-is-trusted = ホスト鍵は信頼済みですが、ログインできませんでした: 下のログを見てください。
onboard-the-host-key-changed-since = ホスト鍵が前回信頼したときから変わっています。
onboard-reached-check-the-host-key = 接続しました。下のホスト鍵を確認してから信頼してください。
onboard-through = { $v } 経由
onboard-checking-2 = 確認中…
onboard-check-again = 再確認
onboard-host-key-trusted = ホスト鍵 · 信頼済み
onboard-host-key-changed-not-trusted = ホスト鍵 · 変更あり、未信頼
onboard-host-key-new-to-this = ホスト鍵 · このマシンでは初めて
onboard-compare-it-with-the-servers = サーバー自身の鍵（例えばコンソールで）と比べてください: ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub

onboard-it-matches-trust-it = ✓ 一致しました: 信頼する
onboard-possibly-a-reinstalled-server-possibly = サーバーが再インストールされたのかもしれませんし、誰かが間に入っているのかもしれません。変更が想定どおりなら、~/.ssh/known_hosts の古い項目を自分で削除してから再確認してください。
onboard-log = ログ
onboard-distro = ディストリビューション
onboard-kernel = カーネル
onboard-arch = アーキテクチャ
onboard-memory = メモリー
onboard-disk = ディスク
onboard-init = INIT
onboard-firewall = ファイアウォール
onboard-open-ports = 開いているポート
onboard-time-sync = 時刻同期
onboard-what-crow-read = CROW が読み取った内容
onboard-config-tools = 設定ツール
onboard-crow-knows-debian-and-red = Crow は Debian 系と Red Hat 系の設定の配置を知っています。このディストリビューションでは設定の検出でファイルを見落とすことがあります。
onboard-your-ssh-agent = 自分の SSH エージェント
onboard-password-once = パスワードを一度だけ
onboard-how-should-it-show-in = フリートにどう表示しますか？
onboard-all-of-this-can-be = どれも後で変更できます。
onboard-name-2 = 名前
onboard-environment = 環境
onboard-group = グループ
onboard-tags = タグ
onboard-bastion = 踏み台
onboard-other-servers-can-be-added = これを経由してほかのサーバーを追加できます。
onboard-through-2 = { $v } 経由 ·{" "}
onboard-group-2 = {" "}· グループ { $group }
onboard-crow-couldnt-log-in-yet = Crow はまだログインできていません: サーバーは追加されますが、ログインできるまで接続不可と表示されます。
onboard-couldnt-reach-it = 接続できませんでした: { $r }
onboard-logs-in-with = { $via }{ $login } でログイン{ $state }{ $state2 }

## onboard-probe
onboard-probe-is-not-a-valid-host = { $host } は有効なホストではありません
onboard-probe-could-not-resolve = { $host } を名前解決できませんでした
onboard-probe-tcp = tcp { $host }:{ $port }: { $e }
onboard-probe-tcp-connect-failed = { $host }:{ $port } への TCP 接続に失敗: { $e }
onboard-probe-tcp-connect = { $host }:{ $port } に TCP 接続
onboard-probe-ssh-banner = SSH バナー { $banner }
onboard-probe-no-ssh-banner-is-this = SSH バナーなし — SSH のポートですか？
onboard-probe-host-key = ホスト鍵 { $host_key_fingerprint }
onboard-probe-could-not-fetch-the-servers = サーバーのホスト鍵を取得できませんでした（ssh-keyscan）
onboard-probe-couldnt-make-a-scratch-known = 一時的な known_hosts を作れませんでした: { $e }
onboard-probe-no-answer = 応答なし
onboard-probe-let-crow-in-but-couldnt = { $via } は Crow を入れましたが { $host }:{ $port } に届かなかったか、{ $via } が Crow を拒否しました
onboard-probe-through = { $via } 経由: { $why }
onboard-probe-reached-through = { $via } 経由で { $host }:{ $port } に到達
onboard-probe-ssh-banner-2 = SSH バナー { $b }
onboard-probe-through-2 = { $via } 経由
onboard-probe-host-key-matches-ssh-known = ホスト鍵は ~/.ssh/known_hosts と一致
onboard-probe-host-key-changed-known-hosts = ホスト鍵が変更されています: known_hosts にはこのホストの別の鍵があります — 中間者攻撃の可能性があるため信頼しません
onboard-probe-host-key-not-in-known = ホスト鍵が known_hosts にありません — フィンガープリントを確認してから受け入れてください
onboard-probe-no-home-directory = ホームディレクトリがありません
onboard-probe-could-not-read-facts-over = SSH で情報を読み取れませんでした: { $e }
onboard-probe-privileges = 権限: { $v }
onboard-probe-unknown-location = 場所不明
onboard-probe-region = リージョン { $place } · { $provider } { $code }
onboard-probe-cloud-metadata = クラウドのメタデータ
onboard-probe-ufw-installed = ufw インストール済み
onboard-probe-read-os-release-uname-meminfo = os-release、uname、meminfo、df、待ち受けポートを読み取り

## overview-collector
overview-collector-unit-on = { $label } のユニット { $unit } を { $action }
overview-collector-sent-signal-to-pid-on = { $label } の PID { $pid } にシグナル { $signal } を送信

## overview-dashboard
overview-dashboard-open = 開く →
overview-dashboard-services = サービス
overview-dashboard-active = 稼働
overview-dashboard-failed = 失敗
overview-dashboard-degraded = 劣化
overview-dashboard-inactive = 停止
overview-dashboard-needs-attention = 要対応
overview-dashboard-no-failed-or-degraded-units = 失敗や劣化したユニットはありません。
overview-dashboard-cpu = CPU { $cpu }%
overview-dashboard-processes = プロセス
overview-dashboard-total = 合計
overview-dashboard-zombie = ゾンビ
overview-dashboard-blocked-d = ブロック（D）
overview-dashboard-top-cpu = CPU 上位
overview-dashboard-no-processes-sampled-yet = まだプロセスを取得していません。
overview-dashboard-top-memory = メモリー上位
overview-dashboard-sockets = ソケット
overview-dashboard-listening = 待ち受け
overview-dashboard-all-interfaces = 全インターフェイス
overview-dashboard-interface = インターフェイス
overview-dashboard-local-only = ローカルのみ
overview-dashboard-listening-ports-widest-exposure-first = 待ち受けポート · 公開範囲の広い順
overview-dashboard-no-listening-sockets-sampled-yet = まだ待ち受けソケットを取得していません。
overview-dashboard-updates-security = 更新とセキュリティ
overview-dashboard-checked = 確認 { $checked_label }
overview-dashboard-checking = 確認中…
overview-dashboard-refresh = 更新 ↻
overview-dashboard-reading-the-package-manager-and = パッケージマネージャーを読み取り、CVE を調べています…
overview-dashboard-not-checked-yet = まだ確認していません。
overview-dashboard-pending = 保留中
overview-dashboard-security = セキュリティ
overview-dashboard-cves-fixed-by-updating = 更新で修正される CVE
overview-dashboard-apply-security = セキュリティ { $security } 件を適用
overview-dashboard-apply-all = { $count } 件すべて適用
overview-dashboard-kernel = カーネル
overview-dashboard-running = 実行中 { $u }
overview-dashboard-update-pending = { $kind ->
        [security] セキュリティ
       *[kernel] カーネル
    }の更新が保留中: { $change }
overview-dashboard-reboot-required-to-finish-installed = インストール済みの更新を完了するには再起動が必要です
overview-dashboard-package-lists-refreshed-h-ago = パッケージリストは { $age_h } 時間前に更新
overview-dashboard-package-lists-are-days-old = パッケージリストは { $h } 日前のものです — 最新の結果には `apt update` を実行してください
overview-dashboard-cves-fixed-by-pending-updates = 保留中の更新で修正される CVE
overview-dashboard-none-found-for-the-pending = 保留中の更新については見つかりませんでした。
overview-dashboard-more = ほか { $fixed } 件
overview-dashboard-known-cves-in-running-software = 実行中のソフトウェアの既知の CVE { $no_fix_yet } 件にはまだ修正がありません
overview-dashboard-cves = CVE
overview-dashboard-pending-updates-security-first = 保留中の更新 · セキュリティ優先
overview-dashboard-everything-is-up-to-date = すべて最新です。
overview-dashboard-more-2 = ほか { $pending } 件
overview-dashboard-cve-data-osv-dev-crow = CVE データ: OSV.dev · Crow が送るのはパッケージ名とバージョンだけで、ホスト名は送りません

## overview-log-tail
overview-log-tail-log-tail = ログの末尾
overview-log-tail-expand = 拡大 ↗
overview-log-tail-error = エラー { $err_count }
overview-log-tail-warn = 警告 { $warn_count }
overview-log-tail-info = 情報 { $info_count }
overview-log-tail-stream-active = ストリーム中
overview-log-tail-this-server = このサーバー
overview-log-tail-socket-log-tail = ソケットのログ
overview-log-tail-pid = PID { $p }
overview-log-tail-all-host-network-sockets-click = ホストのすべてのネットワークソケット · 行をクリックで追跡
overview-log-tail-filter-socket = フィルター: ソケット ✓

overview-log-tail-filter-all = フィルター: すべて
overview-log-tail-time = 時刻
overview-log-tail-level = レベル
overview-log-tail-unit = ユニット
overview-log-tail-pid-2 = PID
overview-log-tail-message = メッセージ
overview-log-tail-lines-live-stream = { $count } 行 · ライブ

## overview-models
overview-models-sleep = スリープ
overview-models-disk = ディスク
overview-models-zombie = ゾンビ
overview-models-idle = アイドル
overview-models-loopback = ループバック
overview-models-private-net = プライベート網
overview-models-public-internet = インターネット

## overview-service-inspector
overview-service-inspector-service-manager = サービス管理
overview-service-inspector-select-a-service-on-the = 左でサービスを選んで管理します
overview-service-inspector-pid = PID
overview-service-inspector-cpu = CPU
overview-service-inspector-mem = メモリー
overview-service-inspector-rss = RSS
overview-service-inspector-uptime = 稼働時間
overview-service-inspector-did-not-reach-the-expected = 前回の操作の後、{ $unit } は想定した状態になりませんでした。
overview-service-inspector-retry = 再試行
overview-service-inspector-lifecycle = ライフサイクル
overview-service-inspector-view-logs-for-this-unit = このユニットのログを見る
overview-service-inspector-configuration = 設定
overview-service-inspector-edit = { $file } を編集
overview-service-inspector-open-in-crow-config-structured = crow-config で開く — 構造化と検証付きの編集
overview-service-inspector-recent-actions = 最近の操作
overview-service-inspector-ok = OK
overview-service-inspector-failed = 失敗
overview-service-inspector-pending = 保留中
overview-service-inspector-start = ▶ 開始
overview-service-inspector-reload = ⟳ 再読み込み
overview-service-inspector-restart = ↻ 再起動
overview-service-inspector-stop = ■ 停止
overview-service-inspector-no-active-connections-will-be = 切断されるアクティブな接続はありません。
overview-service-inspector-established-and-listening-will-be = 確立済みの接続 { $established } { $established ->
       *[other] 件
    }と待ち受けソケット { $listening } { $listening ->
       *[other] 個
    }が切断されます。
overview-service-inspector-checking-active-connections = アクティブな接続を確認中…
overview-service-inspector-cancel = キャンセル
overview-service-inspector-asking-the-clanker-what-this = このサービスが何かを Clanker に問い合わせ中…
overview-service-inspector-low-risk = 低リスク
overview-service-inspector-medium-risk = 中リスク
overview-service-inspector-high-risk = 高リスク
overview-service-inspector-critical = 重大
overview-service-inspector-to-stop-or-restart = 停止・再起動すると: { $reason }
overview-service-inspector-via-the-backup-the-primary = 予備の { $via } による · メインが失敗: { $why }
overview-service-inspector-via-an-ais-reading-check = { $via } による · AI の解釈です: 実行する前に確認してください

## overview-services-table
overview-services-table-table = 表
overview-services-table-map = マップ
overview-services-table-log-drawer-active = ログ欄（表示中）
overview-services-table-log-drawer = ログ欄
overview-services-table-sort-port = ポート順 ↑
overview-services-table-sort-status-name = 状態 · 名前順
overview-services-table-sort-cpu = CPU 順 ↓
overview-services-table-no-processes-match-try-all = 一致するプロセスがありません — すべて を選ぶか、検索を消してください
overview-services-table-no-services-match-try-all = 一致するサービスがありません — すべて を選ぶか、検索を消してください
overview-services-table-of = { $total } 件中 { $first }–{ $last }
overview-services-table-prev = ‹ 前へ
overview-services-table-next = 次へ ›
overview-services-table-group-by-status = 状態でまとめる
overview-services-table-group-by-name = 名前でまとめる
overview-services-table-processes = プロセス { $total }
overview-services-table-running = 実行中 { $running }
overview-services-table-high-cpu = 高 CPU { $high_cpu }
overview-services-table-sockets = ソケット { $total }
overview-services-table-listening = 待ち受け { $listen }
overview-services-table-established = 確立 { $estab }
overview-services-table-units = ユニット { $total }
overview-services-table-active = 稼働 { $active }
overview-services-table-degraded = 劣化 { $degraded }
overview-services-table-failed = 失敗 { $failed }
overview-services-table-pid = PID
overview-services-table-user = ユーザー
overview-services-table-cpu = CPU %
overview-services-table-mem = メモリー %
overview-services-table-rss = RSS
overview-services-table-stat = 状態
overview-services-table-time = 時間
overview-services-table-command = コマンド
overview-services-table-actions = 操作
overview-services-table-proto = プロトコル
overview-services-table-local-endpoint-firewall = ローカル側 / ファイアウォール
overview-services-table-peer-endpoint = 相手側
overview-services-table-state = 状態
overview-services-table-process-service = プロセス / サービス
overview-services-table-unit = ユニット
overview-services-table-uptime = 稼働時間
overview-services-table-no-systemd-service-units-detected = アクティブなホストで systemd のサービスユニットが見つかりません
overview-services-table-no-active-connections-will-be = 切断されるアクティブな接続はありません。
overview-services-table-established-and-listening-will-be = 確立済みの接続 { $established } { $established ->
       *[other] 件
    }と待ち受けソケット { $listening } { $listening ->
       *[other] 個
    }が切断されます。
overview-services-table-checking-active-connections = アクティブな接続を確認中…
overview-services-table-restart = { $name } を再起動しますか？{ $blast_radius_text }
overview-services-table-cancel = キャンセル{" "}
overview-services-table-esc = esc
overview-services-table-restart-2 = 再起動 ⏎
overview-services-table-no-processes-detected-on-active = アクティブなホストでプロセスが見つかりません
overview-services-table-terminate-pid-send-sigterm = { $comm }（PID { $pid }）を終了しますか？SIGTERM を送ります。
overview-services-table-kill-sigterm = 終了（SIGTERM）⏎
overview-services-table-no-open-network-sockets-detected = アクティブなホストで開いているネットワークソケットが見つかりません
overview-services-table-allow = 許可 #{ $rule_number }
overview-services-table-deny = 拒否 #{ $rule_number }
overview-services-table-default-allow = 既定で許可
overview-services-table-no-rule = ルールなし
overview-services-table-fw-off = FW オフ
overview-services-table-fw = FW ?

## overview-sockets-map
overview-sockets-map-gb = { $bytes } GB
overview-sockets-map-mb = { $bytes } MB
overview-sockets-map-kb = { $bytes } KB
overview-sockets-map-incoming-peers = 受信側の相手
overview-sockets-map-remote-clients-initiating-traffic-into = 待ち受け中のサービスに通信してくるリモートのクライアント
overview-sockets-map-local-processes-daemons = ローカルのプロセスとデーモン
overview-sockets-map-host-processes-maintaining-open-sockets = ソケットを開いているホストのプロセス
overview-sockets-map-outgoing-peers = 送信側の相手
overview-sockets-map-external-endpoints-reached-by-host = ホストのプロセスが接続している外部のエンドポイント
overview-sockets-map-hide-127-0-0-1 = 127.0.0.1 を隠す

overview-sockets-map-focus = 注目: { $p_name }
overview-sockets-map-showing-of-established-connections = 確立済みの接続 { $total_conns } 件中 { $shown_conns } 件を表示
overview-sockets-map-no-connections-in-this-category = この分類の接続はありません。
overview-sockets-map-ports = { $count } { $count ->
       *[other] 件の接続
    } · ポート: { $ports_summary }
overview-sockets-map-no-active-process-connections = アクティブなプロセスの接続はありません。
overview-sockets-map-pid = PID { $p }
overview-sockets-map-proc-unprivileged = proc 非特権
overview-sockets-map-in = ← 受信 { $in_count }
overview-sockets-map-out = → 送信 { $out_count }
overview-sockets-map-protocol-state-direction = プロトコル: { $protocol } · 状態: { $state } · 方向: { $direction }
overview-sockets-map-process-pid = プロセス: { $process }（PID { $pid }）
overview-sockets-map-view-in-processes = プロセスで見る →
overview-sockets-map-view-in-firewall = ファイアウォールで見る →

## overview-state
overview-state-all-traffic = すべての通信
overview-state-incoming = 受信
overview-state-outgoing = 送信
overview-state-live = 稼働中
overview-state-failed = 失敗
overview-state-inactive = 停止
overview-state-all = すべて
overview-state-apps = アプリ
overview-state-root = ROOT
overview-state-cpu-1 = CPU > 1%
overview-state-kernel = カーネル

## overview-summary
overview-summary-all-interfaces = 全インターフェイス
overview-summary-interface = インターフェイス
overview-summary-local = ローカル
overview-summary-failed = 失敗
overview-summary-degraded = 劣化

## settings-clankers
settings-clankers-clankers-llm-providers-keys = CLANKER · LLM プロバイダーとキー
settings-clankers-configured = { $configured_count }/{ $count } 設定済み
settings-clankers-30d-cumulative-calls = 30 日間の呼び出し:
settings-clankers-calls = { $total_calls_30d } 回
settings-clankers-primary-backup = メイン · 予備:
settings-clankers-no-backup = 予備なし
settings-clankers-ai-usability-engine-log-clarification = AI 補助エンジン（ログの説明とやさしい解説）
settings-clankers-configure-api-keys-for-llm = LLM プロバイダーの API キーを設定します。Crow はあなたが頼んだときだけ呼び出し、ジャーナルの行をわかりやすく説明させます。キーは Crow のボールトに暗号化して保存され（その鍵は OS のキーリングが保持するか、ボールトのパスワードがオンならそれでロック）、コマンドラインではなく curl の標準入力で送られます。送ったログ行はそのプロバイダーに届きます。
settings-clankers-keys-cant-be-saved-right = 現在キーを保存できません: { $why }。
settings-clankers-supported-clanker-providers-30-day = 対応している CLANKER プロバイダーと 30 日間の利用状況
settings-clankers-set-ends = 設定済み · 末尾 …{ $tail }
settings-clankers-no-key-configured = キー未設定
settings-clankers-primary = ★ メイン
settings-clankers-backup = ↺ 予備
settings-clankers-ready = 準備完了
settings-clankers-not-setup = 未設定
settings-clankers-api-key = API キー:
settings-clankers-model = モデル:
settings-clankers-endpoint = エンドポイント:
settings-clankers-30-day-activity = 30 日間の利用:
settings-clankers-calls-2 = { $calls_30d } 回
settings-clankers-last-used = 最終利用: { $ts }
settings-clankers-never-used = 未使用
settings-clankers-edit-key = ⚙ キーを編集
settings-clankers-testing = テスト中…
settings-clankers-test-key = ⚡ キーをテスト
settings-clankers-reset = リセット
settings-clankers-remove-backup = 予備を外す
settings-clankers-make-backup = 予備にする
settings-clankers-make-primary = メインにする
settings-clankers-ai-usability-sandbox-wtf-is = AI お試し: 「このログは一体何を言いたいの？」
settings-clankers-eli5-assistant-preview = やさしい解説アシスタントのプレビュー
settings-clankers-load-presets = 例を読み込む:
settings-clankers-oom-killer = OOM Killer
settings-clankers-segfault-in-libc = libc のセグフォルト
settings-clankers-systemd-failure = systemd の失敗
settings-clankers-sample-raw-journal-kernel-panic = ジャーナル / カーネルパニックのログ行の例:
settings-clankers-asking = 問い合わせ中…
settings-clankers-translate-log-eli5 = ログを解説 ↵
settings-clankers-asking-the-provider = プロバイダーに問い合わせ中…
settings-clankers-models-offered-for-this-key = このキーで使えるモデル { $count } 個
settings-clankers-load-models = モデルを読み込む
settings-clankers-configure-clanker-api-key = { $p_name } の Clanker API キーを設定
settings-clankers-api-key-stored-encrypted-in = API キー（Crow のボールトに暗号化して保存し、二度と表示しません）:
settings-clankers-custom-api-endpoint-proxy-optional = 独自の API エンドポイント / プロキシ（任意）:
settings-clankers-save-config = 設定を保存 ↵

## settings-config-table
settings-config-table-yes = はい
settings-config-table-no = いいえ
settings-config-table-ms = { $n } ms
settings-config-table-empty = （空）
settings-config-table-none = （なし）
settings-config-table-enable-setting = 設定を有効にする
settings-config-table-disable-setting = 設定を無効にする
settings-config-table-aggressive-timeout = 短めのタイムアウト
settings-config-table-standard-recommended = 標準（推奨）
settings-config-table-tolerant-of-high-latency-links = 遅延の大きい回線でも可
settings-config-table-slow-satellite-dialup = 低速な衛星 / ダイヤルアップ
settings-config-table-aggressive-ping = 頻繁な ping
settings-config-table-relaxed-heartbeat = ゆるやかな死活確認
settings-config-table-off = オフ
settings-config-table-disable-keepalive-ping = keepalive の ping を無効にする
settings-config-table-2-attempts = 2 回
settings-config-table-fast-dead-peer-detection = 相手の停止をすばやく検出
settings-config-table-3-attempts = 3 回
settings-config-table-5-attempts = 5 回
settings-config-table-tolerate-transient-packet-drop = 一時的なパケットロスを許容
settings-config-table-10-attempts = 10 回
settings-config-table-very-tolerant = かなり寛容
settings-config-table-1s-high-resolution = 1 秒（高解像度）
settings-config-table-higher-local-remote-cpu-use = ローカル/リモートの CPU 使用が増える
settings-config-table-2s-default = 2 秒（既定）
settings-config-table-smooth-streaming-low-load = なめらかで低負荷
settings-config-table-5s-battery-eco = 5 秒（省電力）
settings-config-table-low-background-overhead = バックグラウンドの負荷が小さい
settings-config-table-10s-minimal = 10 秒（最小）
settings-config-table-minimal-polling-footprint = ポーリングを最小限に
settings-config-table-chacha20-poly1305-fast-modern = chacha20-poly1305（高速でモダン）
settings-config-table-hardware-agnostic-aead = ハードウェアに依存しない AEAD
settings-config-table-aes128-gcm-aes-ni-accelerated = aes128-gcm（AES-NI で高速化）
settings-config-table-fast-on-modern-intel-amd = 最近の Intel/AMD で高速
settings-config-table-aes256-gcm-maximum-depth = aes256-gcm（最大の強度）
settings-config-table-high-security-environments = 高セキュリティ環境向け
settings-config-table-aes256-gcm-only = AES256-GCM のみ
settings-config-table-default-ed25519-key = 既定の Ed25519 鍵

settings-config-table-dedicated-bastion-key = 踏み台専用の鍵
settings-config-table-legacy-rsa-identity = 古い RSA の鍵
settings-config-table-risk = リスク: { $risk }
settings-config-table-preparing = 準備中…
settings-config-table-fleet-page-background = フリートページの背景
settings-config-table-a-picture-behind-the-server = サーバー一覧の背後に置く画像。表が読みやすいよう、薄くぼかして表示します。PNG、JPEG、WebP。Crow は自分用のコピーを保存します。
settings-config-table-change-picture = 画像を変更…
settings-config-table-choose-picture = 画像を選択…
settings-config-table-remove = 削除
settings-config-table-general = 一般
settings-config-table-general-application-behavior = {"["}general] · アプリの動作
settings-config-table-connection-ssh = 接続と SSH
settings-config-table-connection-applies-to-every-host = {"["}connection] · 上書きしない限りすべてのホストに適用
settings-config-table-keys-rotation = 鍵と更新
settings-config-table-keys-key-distribution-policies = {"["}keys] · 鍵の配布とポリシー
settings-config-table-vault-security = ボールトとセキュリティ
settings-config-table-vault-encrypted-secrets-who-can = {"["}vault] · 暗号化したシークレットと Crow を開ける人
settings-config-table-servers-archives = サーバーとアーカイブ
settings-config-table-servers-archiving-and-how-long = {"["}servers] · アーカイブと保存データの保持期間
settings-config-table-ui-components-lab = UI 部品ラボ
settings-config-table-lab-gpui-component-testbed-sandbox = {"["}lab] · gpui-component の試験場
settings-config-table-clankers-ai-usability = CLANKER（AI 補助）
settings-config-table-clankers-api-keys-log-eli5 = {"["}clankers] · API キーとログ解説
settings-config-table-providers = プロバイダー
settings-config-table-providers-cloud-accounts-your-servers = {"["}providers] · サーバーが動くクラウドのアカウント
settings-config-table-plugins = プラグイン
settings-config-table-plugins-integrations-off-until-you = {"["}plugins] · 連携機能。オンにするまでは無効
settings-config-table-personalisation = カスタマイズ
settings-config-table-appearance-make-crow-yours-saved = {"["}appearance] · Crow を自分好みに · 変更するとすぐ保存
settings-config-table-password-logon-is-not-configured = パスワードログオンは未設定
settings-config-table-disabled-default = 無効（既定）
settings-config-table-without-a-password-anyone-using = パスワードがないと、あなたのアカウントでコンピューターを使う人は誰でも Crow を開けます。シークレット（プロバイダーのトークン、AI のキー）は暗号化されたままで、その鍵は OS のキーリングが保持します。二要素認証（RFC 6238 TOTP）付きのマスターパスワードを設定すると、Crow は開く前に尋ね、シークレットの鍵をそのパスワードでロックします。
settings-config-table-set-password-2fa = パスワードと 2FA を設定 →
settings-config-table-password-2fa-vault-configured = パスワード + 2FA のボールト設定済み
settings-config-table-locked-stance = ロックの方針
settings-config-table-database-and-keys-are-secured = データベースと鍵は保護されています。ロック解除にはパスワードと 6 桁の TOTP コードが必要です。設定した時間が過ぎると自動でロックされます。
settings-config-table-lock-vault-now-l = 今すぐボールトをロック（⇧⌘L）
settings-config-table-its = その設定は
settings-config-table-theyre = それらの設定は
settings-config-table-changed = 変更あり
settings-config-table-or-enter-custom-value = または独自の値を入力:
settings-config-table-apply = 適用
settings-config-table-reset-section = セクションをリセット
settings-config-table-edit-config-toml = config.toml を編集 ⌘/
settings-config-table-save-s-pending = 保存 ⌘S（保留 { $total_changed } 件）
settings-config-table-save-s = 保存 ⌘S
settings-config-table-more-in-this-section-not = このセクションのほか { $hidden_settings } { $hidden_settings ->
       *[other] 件
    }の設定はまだ何にもつながっていないため、動くふりをせず{ $hidden_settings2 }非表示にしています。

## settings-diff-rail
settings-diff-rail-in-sync = 同期済み
settings-diff-rail-pending-diff = 保留中の差分
settings-diff-rail-configuration-in-sync = 設定は同期済み
settings-diff-rail-working-copy-matches-config-crow = 作業コピーは ~/.config/crow/config.toml と一致
settings-diff-rail-affects-live-sessions = 接続中のセッションに影響
settings-diff-rail-pending-ssh-connection-changes-take = 保留中の SSH 接続の変更は再接続時に反映されます。現在のセッションは今のパラメーターのままです。
settings-diff-rail-keychain = キーチェーン
settings-diff-rail-no-keys-enrolled-yet = 登録済みの鍵はまだありません。

## settings-email-form
settings-email-form-just-now = たった今
settings-email-form-min-ago = { $secs } 分前
settings-email-form-h-ago = { $secs } 時間前
settings-email-form-d-ago = { $secs } 日前
settings-email-form-waiting-retried-every-5-min = {" "}· 待機中 { $count } 件、5 分ごとに再試行
settings-email-form-in-the-next-digest = {" "}· 次のまとめに { $count } 件
settings-email-form-in-the-next-digest-2 = 次のまとめに { $count } 件
settings-email-form-security = セキュリティ
settings-email-form-smtp-server = SMTP サーバー
settings-email-form-port = ポート
settings-email-form-no-encryption-the-password-and = 暗号化なし: パスワードとメールが平文でネットワークを流れます。このマシンか自分のネットワーク内の中継サーバー向けに限ってください。
settings-email-form-username = ユーザー名
settings-email-form-password = パスワード
settings-email-form-from = 送信元
settings-email-form-to-comma-separated = 宛先（カンマ区切り）
settings-email-form-critical-alerts = 重大なアラート
settings-email-form-warnings = 警告
settings-email-form-digest-every-minutes = まとめの間隔（分）
settings-email-form-quiet-hours-digests-wait = 静かな時間帯（まとめは待機）
settings-email-form-resolved-alerts-send-nothing-alerts = 解決したアラートは何も送りません。オンにした時点ですでに発生中のアラートは送りません。Crow にはエージェントがないため、メールは Crow の実行中にしか送られません。
settings-email-form-save = 保存
settings-email-form-sending = 送信中…
settings-email-form-send-test = テスト送信
settings-email-form-test-failed = テストに失敗: { $m }
settings-email-form-last-send-failed = 前回の送信に失敗: { $e }{ $d }
settings-email-form-last-mail-sent = 最後のメール送信 { $ago }{ $d }

## settings-keys
settings-keys-scan-idle = スキャン待機中
settings-keys-all = すべて（{ $count }）
settings-keys-group = + グループ
settings-keys-scan = スキャン ⟳
settings-keys-path = + パス
settings-keys-generate-key = + 鍵を生成
settings-keys-auto-monitored-paths = 自動で監視するパス
settings-keys-unenrolled-ssh-discovered-on-disk = ディスク上で未登録の SSH 鍵を { $count } { $count ->
       *[other] 個
    }発見
settings-keys-found-in-scan-paths-click = スキャンしたパスで発見 · 取り込み をクリックして登録
settings-keys-fp = FP: { $fingerprint }
settings-keys-import-to = + { $target_grp } に取り込み
settings-keys-enrolled-keys-in-memory = メモリー内の登録済みの鍵（{ $count }）
settings-keys-filtered-by-group = グループで絞り込み: { $g }
settings-keys-all-groups = すべてのグループ

settings-keys-persisted-in-config-crow-crow = ~/.config/crow/crow.db に保存
settings-keys-no-enrolled-keys-matching-this = このフィルターに一致する登録済みの鍵はありません
settings-keys-scan-ssh-to-import-existing = ~/.ssh をスキャンして既存の鍵を取り込むか、「+ 鍵を生成」で作成してください。
settings-keys-private-key-linked = ● 秘密鍵あり
settings-keys-public-key-only = ○ 公開鍵のみ
settings-keys-reachable-servers = 接続できるサーバー:
settings-keys-attach-host = + ホストを関連付け
settings-keys-sha256-fingerprint = SHA256 フィンガープリント:
settings-keys-path-2 = パス: { $path }
settings-keys-comment = コメント: { $c }
settings-keys-copied-fingerprint-to-clipboard = フィンガープリントをクリップボードにコピーしました
settings-keys-copy-fingerprint = フィンガープリントをコピー
settings-keys-copied-openssh-public-key-to = OpenSSH の公開鍵をクリップボードにコピーしました
settings-keys-copy-public-key = 公開鍵をコピー
settings-keys-enrolled = 登録: { $key }
settings-keys-edit-attach-servers = 編集 / サーバーを関連付け ✎
settings-keys-switch-attached-servers-to-it = 関連付けたサーバーをこの鍵に切り替え
settings-keys-rotate = 更新
settings-keys-remove-from-memory = メモリーから削除 ✕
settings-keys-scan-paths = スキャンするパス
settings-keys-add-path = + パスを追加
settings-keys-default = 既定
settings-keys-group-taxonomy = グループの分類
settings-keys-new-group = + 新しいグループ
settings-keys-text = { $count } { $count ->
       *[other] 個の鍵
    }
settings-keys-key-rotation-audit = 鍵の更新と監査
settings-keys-private-keys-stay-on-this = 秘密鍵はこのマシンに残り、サーバーに送るのは公開鍵だけです。更新 は新しい鍵を作り、古い鍵を使うすべてのサーバーに設置し、ログインできることを確かめてから古い鍵を削除します（1 台ずつ）。自動更新のポリシーはありません。
settings-keys-keypair-generated-and-enrolled = ✓ 鍵ペアを生成して登録しました
settings-keys-generate-new-openssh-keypair = + 新しい OpenSSH 鍵ペアを生成
settings-keys-esc = esc
settings-keys-private-key-written-to-disk =
    秘密鍵をパーミッション 0600 でディスクに書き込みました:
    { $priv_path }
settings-keys-fingerprint = フィンガープリント: { $fp }
settings-keys-copied-public-key-to-clipboard = 公開鍵をクリップボードにコピーしました
settings-keys-key-name = 鍵の名前:
settings-keys-algorithm = アルゴリズム:
settings-keys-ed25519-recommended = Ed25519（推奨）
settings-keys-rsa-4096-bit = RSA 4096 ビット
settings-keys-comment-optional = コメント（任意）:
settings-keys-save-directory = 保存先フォルダー:
settings-keys-tab-to-switch-fields-enter = Tab で項目を移動 · Enter で生成
settings-keys-cancel = キャンセル
settings-keys-generate-keypair = 鍵ペアを生成 ↵
settings-keys-add-ssh-key-scan-directory = SSH 鍵のスキャンフォルダーを追加
settings-keys-enter-directory-path-e-g = フォルダーのパスを入力（例: ~/work-keys や /etc/ssh）:
settings-keys-add-scan = 追加してスキャン ↵
settings-keys-create-new-key-group = 新しい鍵のグループを作成
settings-keys-enter-group-name-e-g = グループ名を入力（例: ステージングフリート、エッジ踏み台）:
settings-keys-create-group = グループを作成 ↵
settings-keys-edit-key-attached-servers = 鍵と関連付けたサーバーを編集
settings-keys-name = 名前:
settings-keys-group-assignment = グループの割り当て:
settings-keys-attach-to-fleet-servers = フリートのサーバーに関連付け:
settings-keys-save-changes = 変更を保存 ↵

## settings-keys-state
settings-keys-state-scanned-found-new = { $count } { $count ->
       *[other] 個
    }のパスをスキャン · { $discovered_count } { $discovered_count ->
       *[other] 個
    }の鍵を発見（新規 { $new_count }）

## settings-lab
settings-lab-type-something-here-to-test = ここに入力して gpui-component を試す...
settings-lab-type-to-reveal-clear-button = 入力するとクリアボタンが表示されます...
settings-lab-search-fleet-by-tag-or = タグやリージョンでフリートを検索...
settings-lab-enter-sensitive-secret = 機密情報を入力...
settings-lab-fleet-deployment-manifest-env-production =
    # フリートのデプロイマニフェスト
    env: production
    replicas: 4
    region: us-east-1
    auto_drain: true
settings-lab-ui-components-lab = UI 部品ラボ
settings-lab-gpui-component-transition-testbed = · gpui-component 移行の試験場
settings-lab-isolated-sandbox = 隔離されたサンドボックス
settings-lab-evaluation-transition-sandbox = 評価と移行のサンドボックス
settings-lab-this-page-runs-gpui-component = このページは Crow の中で gpui-component のウィジェットを直接動かし、既存の画面を移行する前に、プラットフォームネイティブのテキスト選択、カーソルの点滅、クリップボードのショートカット、元に戻す/やり直し、HarfBuzz による文字の整形を試します。
settings-lab-1-primary-native-gpui-input = 1. 基本のネイティブ GPUI 入力
settings-lab-native-gpui-component-input = ネイティブの GPUI-COMPONENT 入力
settings-lab-rope-buffer-shaping = Rope バッファー + 整形
settings-lab-native-mouse-drag-selection-cmd =
    ✓ ネイティブなマウスドラッグでの選択
    ✓ Cmd+A、Cmd+C、Cmd+V、Cmd+Z（元に戻す！）
    ✓ 単語単位の移動（Alt+矢印）と行端への移動（Cmd+矢印）
settings-lab-2-input-variants-capabilities = 2. 入力の種類と機能
settings-lab-password-input-with-native-mask = パスワード入力（ネイティブのマスク切り替えとクリアボタン付き）:
settings-lab-search-filter-input-cleanable = 検索 / フィルター入力（クリア可）:
settings-lab-prefix-suffix-adornments = 前後の飾り:
settings-lab-port-22 = ポート 22
settings-lab-3-6-digit-otp-authenticator = 3. 6 桁の OTP 認証入力
settings-lab-grouped-otp-cells-split-3 = グループ化した OTP セル（3-3 に分割、自動で進む・戻る）:
settings-lab-direct-replacement-candidate-for-vault = ボールトのロック画面と設定画面をそのまま置き換える候補です。
settings-lab-4-multi-line-textarea = 4. 複数行のテキストエリア
settings-lab-multi-line-editor-with-word = 折り返しと複数行選択に対応した複数行エディター:
settings-lab-5-gpui-component-button-matrix = 5. GPUI-COMPONENT のボタン一覧
settings-lab-primary-action = メインの操作
settings-lab-secondary = サブ
settings-lab-ghost = ゴースト
settings-lab-destructive = 破壊的

## settings
settings-general = 一般
settings-connection-ssh = 接続と SSH
settings-keys-rotation = 鍵と更新
settings-vault-security = ボールトとセキュリティ
settings-servers-archives = サーバーとアーカイブ
settings-ui-components-lab = UI 部品ラボ
settings-clankers-ai = Clanker（AI）
settings-providers = プロバイダー
settings-plugins = プラグイン
settings-personalisation = カスタマイズ
settings-in-sync = { $path_str } · 同期済み
settings-pending = { $path_str } · 保留中の変更 { $total_changed } { $total_changed ->
       *[other] 件
    }
settings-no-servers = サーバーなし
settings-1-server = 1 台
settings-servers = { $n } 台
settings-settings = 設定
settings-close-esc = 閉じる esc

## settings-plugins
settings-plugins-local-virtual-machines = ローカルの仮想マシン
settings-plugins-containers = コンテナー
settings-plugins-cloud-providers = クラウドプロバイダー
settings-plugins-notifications = 通知
settings-plugins-plugins-what-crow-connects-to = プラグイン · CROW の接続先

settings-plugins-on = { $on }/{ $count } オン
settings-plugins-re-check = 再確認
settings-plugins-crow-does-nothing-with-these = オンにするまで、Crow はこれらで何もしません。オンにしたプラグインは、持ち込む機能をしかるべき場所に追加します（Multipass はフリートに + VM を起動 を、クラウドプロバイダーは取り込み・電源操作・スナップショットを追加）。オフのものは調べも表示もしません。
settings-plugins-checking = 確認中…
settings-plugins-not-installed-on-this-machine = このマシンにインストールされていません
settings-plugins-not-in-this-build = このビルドには含まれていません
settings-plugins-not-checked-yet = 未確認
settings-plugins-no-account-yet-set-one = アカウントなし · プロバイダーで設定 →
settings-plugins-1-account-providers = 1 アカウント · プロバイダー →
settings-plugins-accounts-providers = { $n } アカウント · プロバイダー →
settings-user-plugins-installed = インストール済みプラグイン
settings-user-plugins-open-folder = フォルダを開く
settings-user-plugins-reading = 読み込み中…
settings-user-plugins-reload = 再読み込み
settings-user-plugins-about = あなたや他の人が書いたプラグインです。1 つのプラグインに 1 つのフォルダ。Crow はすべてのファイルを確認し、承認する前にプラグインが実行できる内容を表示します。承認するまで何も実行されません。ファイルが変更されると、再確認するまでプラグインは無効になります。
settings-user-plugins-reading-folder = プラグインフォルダを読み込み中…
settings-user-plugins-none = プラグインはまだありません。プラグインのフォルダをプラグインフォルダに置いて「再読み込み」を押してください。
settings-user-plugins-by = 作者: { $author }
settings-user-plugins-close = 閉じる
settings-user-plugins-why = 理由
settings-user-plugins-review = 確認
settings-user-plugins-secret-from-vault = シークレット (保管庫から)
settings-user-plugins-effect-read = 読み取り
settings-user-plugins-effect-change = 変更
settings-user-plugins-effect-danger = 危険
settings-user-plugins-on-each-server-root = 各サーバーで root として
settings-user-plugins-on-each-server = 各サーバーで
settings-user-plugins-on-this-machine = このマシンで
settings-user-plugins-dry-run-first = 先にドライラン: { $action }
settings-user-plugins-typed-keyword = 入力するキーワード: { $keyword }
settings-user-plugins-login-check = Crow は前後でログインを確認します
settings-user-plugins-input = { $key } を入力
settings-user-plugins-needs-local = このマシンに必要
settings-user-plugins-needs-server = サーバーに必要
settings-user-plugins-needs-os = OS
settings-user-plugins-setup = セットアップ · 一度だけ、このマシンで
settings-user-plugins-actions = アクション
settings-user-plugins-settings = 設定
settings-user-plugins-setting-secret = { $label } (シークレット)
settings-user-plugins-page-server = 各サーバーのページ「{ $label }」
settings-user-plugins-page-fleet = フリート設定のページ「{ $label }」
settings-user-plugins-badge = バッジ ({ $every } ごと)
settings-user-plugins-shows = 表示
settings-user-plugins-file-binary = { $path } (バイナリ)
settings-user-plugins-meta = Crow { $crow } 用 · { $files } 個のファイル · { $license }
settings-user-plugins-changed-since = 承認後にファイルが変更されたため、このプラグインは無効です。現在の内容を確認してください。
settings-user-plugins-warnings = 警告
settings-user-plugins-refused-heading = 拒否 · Crow はこのプラグインを読み込みません
settings-user-plugins-files = ファイル
settings-user-plugins-confirm-accept = 確定: 承認
settings-user-plugins-accept = 承認
settings-user-plugins-refuse = 拒否
settings-user-plugins-revoke = 承認を取り消す
settings-user-plugins-confirm-explain = 承認すると、このプラグインは上のコマンドをサーバーで実行できるようになります (記載があれば root として)。実行するのは、あなたかフリート実行が開始したときです。承認するには「確定: 承認」を押してください。
settings-user-plugins-install = インストール
settings-user-plugins-reinstall = 再インストール
settings-user-plugins-installing = インストール中…
settings-user-plugins-setup-heading = このマシンでのセットアップ
settings-user-plugins-installed-for-these = このファイルでインストール済み。
settings-user-plugins-not-installed = 未インストール: このマシンでのアクションはインストールを待っています。
settings-user-plugins-settings-form = このプラグインの設定
settings-user-plugins-save = 保存
plugin-page-just-now = たった今
plugin-page-minutes-ago = { $n } 分前
plugin-page-hours-ago = { $n } 時間前
plugin-page-not-read-yet = まだ読み込んでいません。「更新」を押してください。
plugin-page-reading = 読み込み中…
plugin-page-exit = 終了コード { $code }
plugin-page-unreadable = 出力を読み取れません: { $why }
plugin-page-no-rows = 表示するものはありません。
plugin-page-runs = このサーバーでの実行
plugin-page-no-runs = まだ実行されていません。
plugin-page-running = { $label }…
plugin-page-truncated = 出力は 1 MB で切り詰められました。
plugin-page-from = { $name } { $version } · プラグイン
plugin-page-refresh = 更新
plugin-page-not-accepted = このプラグインは承認されていないか、承認後にファイルが変更されました。設定 → プラグインで確認してください。
plugin-page-dry-run-chip = ドライラン
plugin-page-stopped-chip = 停止
plugin-page-running-now = 実行中…
plugin-page-stopping = 停止中…
plugin-page-stop = 停止
plugin-page-danger-title = { $label }: アクセスを失ったりデータを失ったりする可能性があります
plugin-page-change-title = { $label }: サーバーを変更します
plugin-page-dry-run = ドライラン · { $label }
plugin-page-dry-run-needed = 先に実行してください。「実行」は同じ値を使います。
plugin-page-type-to-confirm = 確認するには { $keyword } と入力
plugin-page-run = 実行 · { $label }
plugin-page-cancel = キャンセル
plugin-page-needs-install = このプラグインはこのマシンで実行するため、先にセットアップが必要です。
plugin-fleet-where = 対象
plugin-fleet-count = { $count } 台のサーバー
plugin-local-lab-container = { $name } はラボコンテナーです。このマシンでのアクションは SSH でサーバーに接続しますが、このコンテナーにはありません。

## settings-providers
settings-providers-providers-where-your-servers-live = プロバイダー · サーバーの所在地
settings-providers-set-up = { $configured }/{ $count } 設定済み
settings-providers-import-instances = インスタンスを取り込み
settings-providers-tokens-cant-be-saved-right = 現在トークンを保存できません: { $why }。
settings-providers-no-cloud-provider-plugin-is = クラウドプロバイダーのプラグインがオンになっていません。アカウントを接続するには、設定 → プラグイン で Linode か UpCloud をオンにしてください。
settings-providers-connect-the-cloud-accounts-your = サーバーが動いているクラウドのアカウントを接続します。Crow はそこからインスタンスを読み取り、プロバイダーが対応していれば SSH の外から電源操作やスナップショットができます。トークンは Crow のボールトに暗号化して保存され、その鍵は OS のキーリング（キーチェーン、Secret Service）が保持するか、ボールトのパスワードがあればそれでロックされます。プロバイダーへはコマンドラインではなく curl の標準入力で送ります。
settings-providers-vault-security = ボールトとセキュリティ →
settings-providers-not-set-up = 未設定
settings-providers-testing = テスト中…
settings-providers-connected = 接続済み · { $msg }
settings-providers-saved-not-tested-yet = 保存済み · 未テスト
settings-providers-checked = 確認 { $short_time }
settings-providers-edit = 編集
settings-providers-set-up-2 = 設定
settings-providers-test-connection = 接続をテスト
settings-providers-click-again-to-remove = もう一度クリックで削除
settings-providers-remove = 削除
settings-providers-crow-is-open-anyone-at = Crow はオープンです: このセッションにいる人は誰でも、このトークンでサーバーの電源を切ったりスナップショットを作ったりできます。方針バッジを確認してください。
settings-providers-not-saved = 保存されませんでした: { $e }
settings-providers-secret-vault = シークレット · ボールト
settings-providers-clear = 消去
settings-providers-save-test = 保存してテスト
settings-providers-cancel = キャンセル

## settings-vault-manage
settings-vault-manage-password = パスワード
settings-vault-manage-change-password = パスワードを変更…
settings-vault-manage-go-back-to-open = オープンに戻す…
settings-vault-manage-going-back-to-open-means = オープンに戻すと:
settings-vault-manage-i-understand-without-the-password = 理解しました: パスワードがないと、私として動くものは Crow のシークレットを読めます。
settings-vault-manage-change-password-2 = パスワードを変更
settings-vault-manage-go-open = オープンにする
settings-vault-manage-cancel = キャンセル

## terminal
terminal-ctrl-shift-t = Ctrl+Shift+T
terminal-ctrl-shift-d = Ctrl+Shift+D
terminal-ctrl-shift-s = Ctrl+Shift+S
terminal-ctrl-shift-w = Ctrl+Shift+W
terminal-enter-renames-esc-cancels-empty = Enter で名前を変更 · Esc でキャンセル · 空欄ならシェルに従う
terminal-press-enter-to-start-a = { $why } · Enter で新しく開始
terminal-starting = 開始中…
terminal-lines-back-type-to-return = ↑ { $scrolled } 行前 · 入力すると戻ります
terminal-no-terminals-open-opens-one = 開いているターミナルはありません。{ $shortcut } で開きます。

## topology
topology-crow = CROW
topology-server = サーバー
topology-multipass-vm = MULTIPASS VM
topology-lab-container = ラボコンテナー
topology-container = コンテナー
topology-ssh-key = SSH 鍵
topology-checking-posture = 態勢を確認中…
topology-posture-not-checked-yet = 態勢は未確認
topology-just-now = たった今
topology-min = { $mins } 分
topology-all = すべて
topology-at-risk-only-crit-warn = リスクのみ · 重大 { $crit } · 警告 { $warn }
topology-check-posture = 態勢を確認
topology-reset-view = 表示をリセット
topology-no-servers-yet-enroll-one = サーバーはまだありません: 1 台登録するか、ローカルラボと VM から Multipass VM を起動してください。
topology-ssh = SSH
topology-via-jump-host = 踏み台経由
topology-key-unlocks = 鍵で接続
topology-critical-risk = 重大なリスク
topology-text = { $count } { $count ->
       *[other] 件のリスク
    }
topology-no-risks-found = リスクなし
topology-risks = リスク
topology-env-group = 環境 · グループ
topology-host-key = ホスト鍵
topology-not-pinned = 未固定
topology-cpu-mem-disk = CPU { $cpu_pct }% · メモリー { $mem_pct }% · ディスク { $disk_pct }%
topology-posture = 態勢
topology-not-checked-yet-check-posture = 未確認（態勢を確認 で接続できるサーバーを読み取ります）。
topology-couldnt-read-needs-root = 読み取れません（root が必要）
topology-root-login = root ログイン
topology-open-to-all = すべてに公開
topology-key = 鍵
topology-private-key = 秘密鍵
topology-not-on-this-machine = このマシンにない
topology-connected-to = 接続先
topology-via-jump = 踏み台経由
topology-open-server = サーバーを開く
topology-terminal = ターミナル
topology-posture-checked-ago-servers = 態勢の確認 { $mins } 前 · { $count } 台

## users-host-data
users-host-data-cant-read-etc-passwd = /etc/passwd を読み取れません: { $e }
users-host-data-unnamed-key = 名前のない鍵
users-host-data-is-not-a-valid-name = { $name } は有効な { $kind } 名ではありません
users-host-data-the-password-is-empty = パスワードが空です
users-host-data-the-password-cant-contain-line = パスワードに改行は使えません
users-host-data-the-password-is-longer-than = パスワードが 256 文字を超えています
users-host-data-is-not-an-absolute-shell = { $shell } はシェルの絶対パスではありません
users-host-data-the-full-name-cant-contain = フルネームに ':' や改行は使えません
users-host-data-the-root-account-cant-be = root アカウントは削除できません
users-host-data-not-a-valid-openssh-public = 有効な OpenSSH 公開鍵ではありません

## users-models
users-models-active = 有効
users-models-locked = ロック
users-models-system = システム
users-models-all = すべて
users-models-login = ログイン可
users-models-sudo = SUDO

## users
users-users = ユーザー
users-new-user = + 新しいユーザー
users-user = ユーザー
users-uid = UID
users-groups = グループ
users-shell = シェル
users-keys = 鍵
users-status = 状態
users-couldnt-read-accounts = アカウントを読み取れませんでした: { $e }
users-reading-accounts = アカウントを読み取り中…
users-no-accounts-match-try-all = 一致するアカウントがありません — すべて を選ぶか、検索を消してください
users-select-an-account-to-see = アカウントを選ぶと詳細を表示します。
users-running = 実行中: { $p } …
users-sudo = SUDO
users-root = ROOT
users-account = アカウント
users-uid-gid = uid / gid
users-can-log-in = ログイン可
users-no-password-locked = 不可 — パスワードがロック中

users-no-no-login-shell = 不可 — ログインシェルなし
users-primary = { $g }（プライマリー）
users-remove-from-no-sudo = { $group } から外す（SUDO なし）
users-make-admin-add-to-sudo = 管理者にする: { $group } に追加（SUDO）
users-authorized-ssh-keys = 許可された SSH 鍵
users-no-keys-in-ssh-authorized = ~/.ssh/authorized_keys に鍵はありません。
users-add-key-from-vault = + ボールトから鍵を追加
users-login-shell = ログインシェル
users-password = パスワード
users-locked-no-password-login-setting = ロック中 — パスワードログイン不可（設定するとロック解除）
users-set-can-log-in-with = 設定済み（パスワードでログイン可）
users-set-with-chpasswd-over-stdin = 標準入力経由の chpasswd で設定します。Crow は保存しません。
users-set-password = パスワードを設定
users-cancel = キャンセル
users-set-password-2 = パスワードを設定…
users-actions = 操作
users-unlock-password = パスワードのロックを解除
users-lock-password = パスワードをロック
users-view-passwd-entry = passwd の項目を見る
users-click-again-to-delete = もう一度クリックで { $username } を削除
users-delete-user = ユーザーを削除

## users-new-user-modal
users-new-user-modal-provision-new-system-user = 新しいシステムユーザーを作成
users-new-user-modal-login-username-posix = ログインユーザー名（POSIX）
users-new-user-modal-full-name-gecos-metadata = フルネーム（GECOS メタデータ）
users-new-user-modal-password-optional = パスワード（任意）
users-new-user-modal-the-passwords-dont-match = パスワードが一致しません。
users-new-user-modal-leave-empty-for-key-only = 鍵だけでログインするなら空欄にしてください（後で詳細画面から鍵を追加）。
users-new-user-modal-set-with-chpasswd-over-stdin = 標準入力経由の chpasswd で設定します。Crow は保存しません。
users-new-user-modal-login-shell = ログインシェル
users-new-user-modal-initial-privileges = 初期の権限
users-new-user-modal-admin-group = ✓ 管理者（{ $g } グループ）
users-new-user-modal-standard-user-click-to-add = 一般ユーザー · クリックで { $g } に追加
users-new-user-modal-supplementary-groups-usermod-ag = 追加のグループ（usermod -aG）
users-new-user-modal-create-user-home-directory-home = ホームディレクトリ（/home/<ユーザー名>）を雛形のドットファイル付きで作成
users-new-user-modal-enabled = 有効
users-new-user-modal-disabled = 無効
users-new-user-modal-provision-account = アカウントを作成

## users-passwd-inspector
users-passwd-inspector-return-to-users = ← ユーザーに戻る
users-passwd-inspector-account = アカウント: { $uname }
users-passwd-inspector-open-in-config-editor = 設定エディターで開く
users-passwd-inspector-raw-7-field-posix-passwd = 7 フィールドの POSIX passwd レコード（生）
users-passwd-inspector-delimiter-colon = 区切り文字: ':'（コロン）
users-passwd-inspector-exploded-field-anatomy-system-semantics = 各フィールドの構造と意味
users-passwd-inspector-switch-passwd-record-to-inspect = 確認する passwd レコードを切り替え
users-passwd-inspector-login-username = ログインユーザー名
users-passwd-inspector-password-placeholder = パスワードの印
users-passwd-inspector-indicates-account-password-is-locked = '!' はアカウントのパスワードがロックされていることを示します。PAM/パスワードでのログインは拒否されます。
users-passwd-inspector-x-indicates-hashed-passwords-and = 'x' はパスワードのハッシュとソルトが /etc/shadow（root のみ、0640）に分けて保存されていることを示します。
users-passwd-inspector-user-identifier-uid = ユーザー ID（UID）
users-passwd-inspector-uid-0-superuser-root-with = UID 0: Linux カーネルの権限に制限のないスーパーユーザー（root）。
users-passwd-inspector-system-service-uid-reserved-for = システムサービスの UID: 隔離されたバックグラウンドのデーモンやシステムサービス用に予約されています。
users-passwd-inspector-interactive-login-uid-standard-regular = 対話ログインの UID: 一般の人のアカウントは UID 1000 以上から始まります。
users-passwd-inspector-primary-group-gid = プライマリーグループ（GID）
users-passwd-inspector-primary-numeric-group-id-applied = このユーザーが作るファイルやプロセスに付くプライマリーグループの数値 ID です。/etc/group で定義されます。
users-passwd-inspector-gecos-metadata = GECOS メタデータ
users-passwd-inspector-empty = （空）
users-passwd-inspector-general-electric-comprehensive-operating-supervisor = General Electric Comprehensive Operating Supervisor から引き継いだフィールドです。フルネーム、所属、電話番号を入れます。
users-passwd-inspector-home-directory = ホームディレクトリ
users-passwd-inspector-initial-working-directory-set-upon = ログインセッションの開始時の作業ディレクトリです。~/.ssh、~/.config、ドットファイルが置かれます。
users-passwd-inspector-login-shell = ログインシェル
users-passwd-inspector-prohibits-interactive-terminal-access-daemons = 対話的なターミナルの利用を禁止します。デーモンやサービスアカウントは多層防御のために nologin を使います。
users-passwd-inspector-command-interpreter-spawned-upon-login = ログインセッションで起動されるコマンドインタープリター（例: /bin/bash、/bin/zsh、/usr/bin/fish）。
users-passwd-inspector-portion = 部分 #{ $num }

## users-ssh-attach-modal
users-ssh-attach-modal-ssh-keychain-attachment = SSH 鍵の関連付け · { $uname }
users-ssh-attach-modal-attached = { $count } 個関連付け済み
users-ssh-attach-modal-target-file = 対象ファイル: { $target_auth_path }
users-ssh-attach-modal-permissions-0600-secure = パーミッション: 0600（安全）
users-ssh-attach-modal-currently-authorized-keys-for-user = このユーザーに現在許可されている鍵
users-ssh-attach-modal-no-ssh-keys-currently-installed = このユーザーに設置された SSH 鍵はありません。下で関連付けてください。
users-ssh-attach-modal-revoke = 無効化
users-ssh-attach-modal-select-from-crow-ssh-keychain = CROW の SSH 鍵から選択
users-ssh-attach-modal-enrolled-vault-keys = ボールトの登録済みの鍵 { $count } 個
users-ssh-attach-modal-no-enrolled-keys-found-in = Crow のボールトに登録済みの鍵はありません。設定 > SSH 鍵 で生成するかスキャンしてください。
users-ssh-attach-modal-attached-2 = ✓ 関連付け済み
users-ssh-attach-modal-attach-key = + 鍵を関連付け

## host-diagnose
host-diagnose-asks-to-approve-each-use = { $agent }（鍵を使うたびに承認を求める）
host-diagnose-is-the-ssh-agent-here = ここでは { $agent } が SSH エージェントで、新しい接続のたびに承認を求めます。Crow はバックグラウンドで接続するため、誰も承認しないとタイムアウトや拒否のように見えます。{ $agent } に求められたら承認するか、Crow の承認を記憶するよう { $agent } を設定してください。
host-diagnose-for-no-prompts-at-all = 承認をまったく求められないようにするには、このサーバーを Crow 自身の鍵に切り替えてください（鍵 → サーバーを鍵に切り替え）。すると Crow は { $agent } を使いません。
host-diagnose-ssh-said = ssh の出力
host-diagnose-jump-host = 踏み台 { $j }
host-diagnose-missing-on-this-machine = { $path }（このマシンにない）
host-diagnose-your-ssh-agent-ssh-config = SSH エージェント / ~/.ssh/config
host-diagnose-connected = 接続済み
host-diagnose-crow-reaches = Crow は { $addr } に接続できます。
host-diagnose-key-tried = 試した鍵
host-diagnose-server-accepts = サーバーが受け付ける方式
host-diagnose-the-key-file-isnt-on = 鍵ファイル { $path } はもうこのマシンにありません: 復元するか、このサーバーを別の鍵に切り替えてください。
host-diagnose-the-server-doesnt-accept-that = サーバーは { $login_user } のその鍵を受け付けません: そのサーバーの ~{ $login_user }/.ssh/authorized_keys にないか、sshd がファイルを無視しています（パーミッション、AuthorizedKeysFile）。

host-diagnose-the-server-doesnt-offer-key = サーバーは鍵ログインをまったく提供していません（PubkeyAuthentication no）: Crow は鍵でしかログインしません。
host-diagnose-it-accepts-a-password-log = パスワードを受け付けます: 一度パスワードでログインすると Crow が鍵を設置し、パスワードは保存しません。
host-diagnose-login-refused = ログイン拒否
host-diagnose-answered-but-refused-crows-login = { $addr } は応答しましたが、Crow のログインを拒否しました。
host-diagnose-presents-now = 現在の鍵
host-diagnose-cant-fetch-through-a-jump = 踏み台経由では取得できません
host-diagnose-host-key-changed = ホスト鍵の変更
host-diagnose-presents-a-different-host-key = { $addr } は Crow が信頼していたものと違うホスト鍵を提示しています。サーバーが再インストールされたか鍵が作り直されたか、あるいはあなたとサーバーの間で何かがなりすましています。
host-diagnose-dont-trust-the-new-key = 変わった理由が分かるまで新しい鍵を信頼しないでください。
host-diagnose-check-it-on-the-server = サーバー自身（コンソール、プロバイダーのウェブシェル）で確認してください: ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub を実行し、「現在の鍵」と比べます。
host-diagnose-host-key-not-trusted = ホスト鍵が未信頼
host-diagnose-ssh-known-hosts-has-no = ~/.ssh/known_hosts に { $addr } の鍵がないため、ssh は接続を拒否します（厳密な確認）。
host-diagnose-compare-the-fingerprint-with-the = 信頼する前に、フィンガープリントをサーバーのコンソールのものと比べてください。
host-diagnose-this-server-is-behind-a = このサーバーは踏み台の後ろにあるため、Crow は鍵を直接取得できません: サーバー上で確認してから、自分で known_hosts に追加してください。
host-diagnose-once-it-checks-out-trust = 確認できたら: 新しい鍵を信頼します（確認のためサーバー名を入力）。
host-diagnose-jump-host-failed = 踏み台で失敗
host-diagnose-the-connection-through-failed-before = { $j } を経由した接続が { $addr } に届く前に失敗しました。
host-diagnose-open-in-crow-if-its = Crow で { $j } を開いてください: そこにも接続できなければ、先にそちらを直してください。
host-diagnose-check-the-jump-host-can = 踏み台からこのサーバーのアドレスとポートに届くか確認してください。
host-diagnose-name-not-found = 名前が見つかりません
host-diagnose-doesnt-resolve-to-an-address = このマシンからは { $host } をアドレスに解決できません。
host-diagnose-check-the-spelling-or-use = 綴りを確認するか、IP アドレスを使ってください。
host-diagnose-a-vpn-or-private-dns = 内部の名前には VPN やプライベート DNS が必要なことがあります。
host-diagnose-connection-refused = 接続拒否
host-diagnose-answered-but-nothing-accepts-connections = { $host } は応答しましたが、ポート { $server } で接続を受け付けるものがありません。
host-diagnose-a-firewall-rejecting-the-port = ファイアウォールがポートを拒否している場合も同じに見えます。
host-diagnose-timed-out = タイムアウト
host-diagnose-didnt-answer-at-all = { $host } はまったく応答しませんでした。
host-diagnose-the-server-may-be-off = サーバーが停止しているか再起動中かもしれません。プロバイダーで確認してください。
host-diagnose-a-firewall-dropping-the-port = ファイアウォールがポートを破棄している場合や、アドレスが違う場合も同じに見えます。
host-diagnose-no-route = 経路なし
host-diagnose-this-machine-has-no-route = このマシンから { $host } への経路がありません。
host-diagnose-check-your-network-vpn-or = ネットワーク、VPN、アドレスを確認してください。
host-diagnose-unreachable = 接続不可
host-diagnose-crow-couldnt-open-a-connection = Crow は { $addr } への接続を開けませんでした。
host-diagnose-try-again-if-it-persists = もう一度試してください。続くようならターミナルから ssh で接続して、エラーの全文を確認してください。
host-diagnose-direct-probe = 直接の確認
host-diagnose-no-ssh-banner = SSH バナーなし
host-diagnose-no-answer = 応答なし
host-diagnose-port-open-ms = ポート開放（{ $p } ms）· { $p2 }

## host-ssh
host-ssh-connected = 接続済み
host-ssh-auth-failed = 認証失敗
host-ssh-host-key = ホスト鍵
host-ssh-unreachable = 接続不可
host-ssh-1password = 1Password
host-ssh-secretive = Secretive
host-ssh-login-failed = ログイン失敗
host-ssh-isnt-a-private-folder-of = { $dir } はこのユーザーの非公開フォルダーではありません
host-ssh-the-bastions-loop-back-to = 踏み台が { $id } に戻って循環しています: どれかの設定を直してください
host-ssh-more-than-bastions-deep = 踏み台の深さが { $max_hops } を超えています
host-ssh-its-bastion-is-no-longer = 踏み台（{ $id }）はもうフリートにありません
host-ssh-bastion-has-an-invalid-address = 踏み台 { $name } のアドレスが無効です
host-ssh-bastion-logs-in-with-a = 踏み台 { $name } はパスワードでログインします。Crow は鍵で踏み台を経由します
host-ssh-bastion-s-key-has-no = 踏み台 { $name } の鍵に秘密鍵ファイルがありません
host-ssh-no-bastion = 踏み台なし
host-ssh-holds-a-which-ssh-would = { $bad } に '%' が含まれています。ssh はこれをトークンとして解釈します
host-ssh-the-server-directory-is-busy = サーバー一覧が使用中です
host-ssh-the-bastion = 踏み台
host-ssh-bastion-cant-reach = 踏み台 { $via } から { $target } に届きません: { $why }
host-ssh-bastion-refused-crows-key = 踏み台 { $name } が Crow の鍵を拒否しました: { $line }
host-ssh-bastion = 踏み台 { $name }: { $line }
host-ssh-this-server-is-set-to = このサーバーはパスワードログインに設定されています。Crow は鍵で接続し、その鍵はパスワードで一度だけ設置します。
host-ssh-the-enrolled-key-for-this = このサーバーに登録した鍵には秘密鍵ファイルがありません
host-ssh-invalid-host-address = 無効なホストアドレス { $host }
host-ssh-via = {" "}{ $name } 経由
host-ssh-bastion-2 = 踏み台 { $name }:{" "}
host-ssh-bastion-refused-crows-key-2 = 踏み台 { $name } が Crow の鍵を拒否しました: { $d }
host-ssh-ssh-failed = ssh が失敗

## security-hardening
security-hardening-root-can-log-in-over = root が SSH でパスワードログインできる（PermitRootLogin yes）
security-hardening-root-can-log-in-over-2 = root が SSH で鍵ログインできる（PermitRootLogin { $v }）
security-hardening-no-firewall-filters-incoming-traffic = 受信をフィルタリングするファイアウォールがない
security-hardening-no-firewall-and-open-to = ファイアウォールがなく、{ $count ->
       *[other] ポート
    } { $p } がすべてのネットワークに公開されている
security-hardening-permitrootlogin-prohibit-password-crow-logs = PermitRootLogin prohibit-password（Crow は鍵で root としてログインするため「no」ではなく）
security-hardening-permitrootlogin-prohibit-password = PermitRootLogin prohibit-password
security-hardening-permitrootlogin-no = PermitRootLogin no
security-hardening-passwordauthentication-no-once-a-key = PasswordAuthentication no（鍵ログインを確認してから）
security-hardening-and-the-listening = {" "}と待ち受け中の{ $count ->
       *[other] ポート
    } { $p }
security-hardening-crow-logs-in-to-this = ✕ Crow はこのサーバーに root としてログインしています: PermitRootLogin no にすると Crow が締め出されます。何も変更していません。
security-hardening-a-fresh-key-login-works = ✓ 新規の鍵ログインが使えます。
security-hardening-this-sshd-config-doesnt-include = ✕ この sshd_config は sshd_config.d を読み込まないため、ドロップインは効きません。何も変更していません。代わりに sshd_config シートで PermitRootLogin を設定してください。
security-hardening-and-reloaded-sshd = {" "}して sshd を再読み込み
security-hardening-undo-didnt-finish-remove-by = ⚠ 元に戻しきれませんでした: { $root_dropin } を手で削除して sshd を再読み込みしてください。

security-hardening-wrote = ✓ { $root_dropin } を書き込みました。
security-hardening-sshd-rejected-the-change = ✕ sshd が変更を拒否しました: { $e }
security-hardening-sshd-t-accepts-it = ✓ sshd -t が受け付けました。
security-hardening-sshd-will-use-permitrootlogin = ✓ sshd は PermitRootLogin { $value } を使います。
security-hardening-something-earlier-in-the-config = ✕ 設定の前のほうで、まだ何かが PermitRootLogin を設定しています（{ $other }）。
security-hardening-couldnt-check-the-result = ✕ 結果を確認できませんでした: { $e }
security-hardening-couldnt-reload-sshd = ✕ sshd を再読み込みできませんでした: { $e }
security-hardening-reloaded-sshd = ✓ sshd を再読み込みしました。
security-hardening-a-fresh-key-login-failed = ✕ 再読み込み後の新規の鍵ログインに失敗しました（{ $e }）。
security-hardening-a-fresh-key-login-still = ✓ 新規の鍵ログインはまだ使えます。PermitRootLogin は { $value } です。
security-hardening-a-fresh-key-login-doesnt-2 = ✕ 新規の鍵ログインが使えません（{ $e }）。何も変更していません。
security-hardening-neither-ufw-nor-firewalld-is = ✕ ufw も firewalld もインストールされていません。どちらかをインストールして（apt install ufw、dnf install firewalld）もう一度試してください。何も変更していません。
security-hardening-undone = ↺ 元に戻しました: { $off }。
security-hardening-undo-didnt-finish-run-by = ⚠ 元に戻しきれませんでした: { $off } を手で実行してください。
security-hardening-a-fresh-key-login-failed-2 = ✕ ファイアウォールを有効にした状態で新規の鍵ログインに失敗しました（{ $e }）。
security-hardening-a-fresh-key-login-still-2 = ✓ 新規の鍵ログインはまだ使えます。{ $tool } は有効で、{ $ports } を許可しています。
security-hardening-turn-a-firewall-on-that = SSH{ $p } を許可するファイアウォールを有効にする
security-hardening-undone-removed = ↺ 元に戻しました: { $root_dropin } を削除{ $reload }。

## security-stance
security-stance-open = オープン
security-stance-locked = ロック
security-stance-secrets-provider-tokens-ai-keys = シークレット（プロバイダーのトークン、AI のキー）は crow.db の中で暗号化されるため、ファイルのコピーやバックアップから漏れることはありません。
security-stance-tokens-never-appear-on-a = トークンがコマンドラインやログに現れることはありません。
security-stance-everything-open-protects = オープンが守るものすべて。
security-stance-nobody-can-open-crow-or = パスワードと 2FA コードがなければ、誰も（あなたとして動くプログラムでさえ）Crow を開いたりシークレットを使ったりできません。
security-stance-the-key-is-wiped-from = Crow がロックされると鍵はメモリーから消去されます。
security-stance-anyone-at-your-unlocked-computer = ロックされていないあなたのコンピューターの前にいる人は誰でも、Crow を開いてその中身をすべて使えます。
security-stance-any-program-running-as-you = あなたとして動くプログラムはどれも、ログイン中なら（ログインでキーリングが解錠されるため）OS のキーリングに Crow の鍵を求め、すべてのシークレットを読めます。
security-stance-provider-tokens-can-power-servers = プロバイダーのトークンでサーバーの電源を切ったりスナップショットを作ったりできます。あなたのセッションを持つ人はその力を持ちます。
security-stance-crows-ssh-keys-in-ssh = ~/.ssh にある Crow の SSH 鍵は Crow が暗号化しないファイルです: パスフレーズがない限り、あなたとして動くものはそれを使ってサーバーに入れます。
security-stance-while-crow-is-unlocked-anyone = Crow のロックが解除されている間は、自動ロックされるまで、コンピューターの前にいる人は誰でも使えます。
security-stance-crows-ssh-keys-in-ssh-2 = ~/.ssh にある Crow の SSH 鍵は Crow が暗号化しないファイルです: パスフレーズがない限り、ロックの有無にかかわらず、あなたとして動くものはそれを使えます。
security-stance-your-server-list-settings-and = サーバー一覧、設定、変更履歴はどちらの方針でも暗号化されません。
security-stance-nothing-to-type-nothing-to = 入力するものも、失うものもありません。その手軽さが上のリスクです。
security-stance-you-unlock-crow-with-your = 起動するたびと自動ロックの後に、パスワードと 2FA コードで Crow のロックを解除します。
security-stance-lose-the-password-or-the = パスワードか認証アプリをなくすと、保存したシークレットは永久に失われます。リセットも復旧もありません。
security-stance-ai-api-keys-are-still = AI の API キーがまだ平文で保存されています（古いバージョンの Crow から）。暗号化キーが使えるようになり次第、ボールトに移します。
security-stance-no-os-keyring-is-available = OS のキーリングが使えないため（{ $why }）、Crow はシークレットを保存できません。保存するにはロックに切り替えてください。
security-stance-provider-can-power-servers-or = プロバイダーのアカウント { $action_capable_accounts } { $action_capable_accounts ->
       *[other] 件
    }でサーバーの電源操作やスナップショットができ、このセッションにいる誰でも使えます。
security-stance-auto-lock-is-off-once = 自動ロックはオフです: 一度ロックを解除すると、ロックするか終了するまで解除されたままです。
security-stance-auto-lock-is-minutes-an = 自動ロックは { $auto_lock_minutes } 分です: 放置されたロック解除中の Crow はその間使えます。

## security-sshd-passwords
security-sshd-passwords-a-fresh-key-login-works = ✓ 新規の鍵ログインが使えます。
security-sshd-passwords-password-login-is-already-off = ✓ パスワードログインはすでにオフです。何もすることはありません。
security-sshd-passwords-this-sshd-config-doesnt-include = ✕ この sshd_config は sshd_config.d を読み込まないため、ドロップインは効きません。何も変更していません。代わりに sshd_config シートで PasswordAuthentication を no にしてください。
security-sshd-passwords-and-reloaded-sshd = {" "}して sshd を再読み込み
security-sshd-passwords-undo-didnt-finish-remove-by = ⚠ 元に戻しきれませんでした: { $dropin } を手で削除して sshd を再読み込みしてください。
security-sshd-passwords-wrote = ✓ { $dropin } を書き込みました。
security-sshd-passwords-sshd-rejected-the-change = ✕ sshd が変更を拒否しました: { $e }
security-sshd-passwords-sshd-t-accepts-it = ✓ sshd -t が受け付けました。
security-sshd-passwords-sshd-will-refuse-passwords = ✓ sshd はパスワードを拒否します。
security-sshd-passwords-set-in = {" "}（設定箇所: { $who }）
security-sshd-passwords-couldnt-check-the-result = ✕ 結果を確認できませんでした: { $e }
security-sshd-passwords-couldnt-reload-sshd = ✕ sshd を再読み込みできませんでした: { $e }
security-sshd-passwords-reloaded-sshd = ✓ sshd を再読み込みしました。
security-sshd-passwords-a-fresh-key-login-failed = ✕ 再読み込み後の新規の鍵ログインに失敗しました（{ $e }）。
security-sshd-passwords-a-fresh-key-login-still = ✓ 新規の鍵ログインはまだ使えます。パスワードログインはオフです。
security-sshd-passwords-undone-removed = ↺ 元に戻しました: { $dropin } を削除{ $reload }。
security-sshd-passwords-something-else-still-turns-password = ✕ ほかの何かがまだパスワードログインをオンにしています{ $who }。

## security-host-keys
security-host-keys-this-server-uses-host-certificates = このサーバーはホスト証明書を使っています。署名する CA で更新してください
security-host-keys-couldnt-read-sshds-settings = sshd の設定を読み取れませんでした: { $e }
security-host-keys-couldnt-generate-new-keys = 新しい鍵を生成できませんでした: { $e }
security-host-keys-none-of-sshds-host-keys = sshd のホスト鍵がディスク上に 1 つもありません
security-host-keys-the-server-returned-a-public = サーバーが Crow の読めない公開鍵を返しました。何も変更していません
security-host-keys-the-new-keys-werent-put = 新しい鍵を配置できませんでした: { $e }
security-host-keys-rolled-back-the-old-host = { $why }。元に戻しました: 古いホスト鍵に戻り、known_hosts も元のとおりです。

security-host-keys-server = サーバー: { $e }
security-host-keys-known-hosts-couldnt-be-updated = known_hosts を更新できませんでした（{ $e }）
security-host-keys-a-fresh-login-with-the = 新しい鍵での新規ログインに失敗しました（{ $e }）
security-host-keys-rollback-didnt-finish-the-old = { $why }。元に戻しきれませんでした（{ $r }{ $l }）。古い鍵はサーバーの { $backup } に残っています。

## security-osv
security-osv-osv-answered-with-something-unexpected = OSV が想定外の内容を返しました: { $e }
security-osv-an-unknown-os = 不明な OS
security-osv-cve-lookup-covers-debian-and = CVE の検索は Debian と Ubuntu が対象です。このサーバーは { $report } です

## security-sessions
security-sessions-crow-couldnt-find-its-own = sshd の接続の中から Crow 自身の接続を見つけられなかったため、何も終了していません
security-sessions-no-other-ssh-logins = ほかの SSH ログインはありません
security-sessions-ended-other-ssh = ほかの SSH ログインを { $count } { $count ->
       *[other] 件
    }終了しました: { $who }

## lab-multipass
lab-multipass-install-multipass-with-homebrew = Homebrew で Multipass をインストール
lab-multipass-install-multipass-from-canonical = Canonical から Multipass をインストール
lab-multipass-multipass-ships-as-a-snap = Multipass は snap で配布されています。システムイメージに snapd を追加してから再起動してください
lab-multipass-multipass-ships-as-a-snap-2 = Multipass は snap で配布されています。パッケージマネージャーで snapd をインストールしてください
lab-multipass-start-snapd = snapd を開始
lab-multipass-install-multipass = Multipass をインストール
lab-multipass-let-multipasss-dns-server-find = Multipass の DNS サーバーが nobody ユーザーを見つけられるようにする（このシステムではシステムユーザーは /usr/lib にあります）
lab-multipass-let-vms-get-an-address = VM がアドレスを取れるようにする: firewalld が Multipass のプライベートブリッジで DHCP と DNS をブロックしています
lab-multipass-restart-the-multipass-daemon = Multipass のデーモンを再起動
lab-multipass-the-daemon-didnt-answer = デーモンが応答しませんでした
lab-multipass-a-vm-name-is-a = { $name }: VM の名前は英字で始まり、英数字とハイフンが続きます（ハイフンで終わらない）
lab-multipass-a-vm-needs-at-least = VM には少なくとも CPU 1 個、メモリー 1 GB、ディスク 5 GB が必要です
lab-multipass-isnt-an-image-name-multipass = { $image } は Multipass が知っているイメージ名ではありません
lab-multipass-crows-public-key-isnt-a = Crow の公開鍵が有効な OpenSSH 鍵ではありません
lab-multipass-couldnt-pass-the-cloud-init = cloud-init を渡せませんでした: { $e }
lab-multipass-unexpected-multipass-list-output = `multipass list` の出力が想定外です: { $e }
lab-multipass-unexpected-multipass-info-output = `multipass info` の出力が想定外です: { $e }
lab-multipass-has-no-ssh-host-keys = { $name } にはまだ SSH ホスト鍵がありません。まだ起動中ですか？
lab-multipass-not-a-valid-openssh-public = 有効な OpenSSH 公開鍵ではありません
lab-multipass-start = 開始
lab-multipass-stop = 停止
lab-multipass-restart = 再起動
lab-multipass-suspend = 一時停止
lab-multipass-delete-for-good = 完全に削除
lab-multipass-not-running-yet = まだ起動していません
lab-multipass-no-address-yet = まだアドレスがありません
lab-multipass-state = 状態 { $state }
lab-multipass-didnt-come-up-in-s = { $name } は { $as_secs } 秒以内に起動しませんでした（{ $last }）
lab-multipass-no-host-keys-to-pin = 固定するホスト鍵がありません
lab-multipass-couldnt-write-ssh-known-hosts = ~/.ssh/known_hosts に書き込めませんでした: { $e }
lab-multipass-waiting-for-an-address-and = アドレスとホスト鍵を待機中
lab-multipass-pinning-host-keys-for = { $address } のホスト鍵を固定中
lab-multipass-logging-in-as-ubuntu-with = Crow の鍵で ubuntu としてログイン中
lab-multipass-reading-its-facts = 情報を読み取り中
lab-multipass-adding-crows-key-to = { $name } に Crow の鍵を追加中

## journal
journal-emerg = EMERG
journal-alert = ALERT
journal-notice = NOTICE
journal-debug = DEBUG
journal-live = ライブ
journal-all-time = 全期間
journal-this-boot = 今回の起動
journal-previous-boot = 前回の起動
journal-just-now = たった今
journal-s-ago = { $diff } 秒前
journal-m-ago = { $diff } 分前
journal-h-ago = { $diff } 時間前
journal-d-ago = { $diff } 日前

## dr
dr-min = { $s } 分
dr-days = { $s } 日
dr-no-backup-found-at = { $path } にバックアップが見つかりません
dr-last-ran = 、最終実行 { $last }
dr-not-found = 見つかりません
dr-exists = { $p } があります
dr-isnt-there-the-plan-may = { $p } がありません: プランが古いかもしれません
dr-config-pinned-as-baselines-rebuildable = 設定ファイル { $baselines } { $baselines ->
       *[other] 件
    }をベースラインとして固定（Crow から再構築可能）
dr-no-config-file-pinned-as = ベースラインとして固定した設定ファイルがありません: 再構築に必要なものを固定してください
dr-can-snapshot-it-danger-zone = { $p } でスナップショットを作成できます（危険ゾーン → スナップショット）
dr-the-plan-names-no-backups = プランにバックアップが指定されていません
dr-is-active = { $t } は有効です{ $last }
dr-isnt-running-backups-wont-be = { $t } は動いていません（{ $state }）: バックアップは作られません

## patching
patching-security-updates = セキュリティ更新
patching-all-updates = すべての更新
patching-s-package-index-was-never = { $manager } のパッケージ一覧が一度もダウンロードされていないため、保留中の更新は分かりません: そのサーバーで `{ $manager } update` を実行してください
patching-nothing-to-upgrade = 更新するものはありません
patching-isnt-a-package-name = { $bad } はパッケージ名ではありません
patching-no-package-manager-crow-knows = Crow が知っているパッケージマネージャー（apt、dnf、apk）がありません
patching-isnt-a-package-manager-crow = { $other } は Crow がパッチに使えるパッケージマネージャーではありません
patching-upgraded = 更新 { $count }
patching-still-pending = 、まだ保留 { $count }（{ $still_pending }）
patching-reboot-required = {" "}· 再起動が必要
patching-failed-exit = { $manager } が失敗しました（終了コード { $status }）: { $last_lines }
patching-a-window-looks-like-sun = 時間帯は「Sun 02-05」や「Mon-Fri 22-06」のような形式です
patching-isnt-a-day = { $a } は曜日ではありません
patching-isnt-a-day-2 = { $b } は曜日ではありません
patching-isnt-a-day-3 = { $part } は曜日ではありません
patching-no-days-in-the-window = 時間帯に曜日がありません
patching-hours-look-like-02-05 = 時刻は 02-05 のような形式です
patching-hours-are-0-to-24 = 時刻は 0〜24 です
patching-the-windows-start-and-end = 時間帯の開始と終了が同じ時刻です
patching-outside-s-maintenance-window = { $group } のメンテナンス時間帯外（{ $describe }）
patching-no-systemd-to-wait-for = 待つ systemd がありません
patching-services-up = サービス稼働（{ $state }）
patching-came-back-but-systemd-was = 復帰しましたが、{ $as_secs } 秒後も systemd が起動処理中でした
patching-came-back-but-failed = 復帰しましたが、{ $count ->
       *[other] ユニット
    }が失敗しました: { $units }

## certs
certs-certbots-timer-isnt-running-certificates = certbot のタイマーが動いていません（{ $t }）: 証明書は自動で更新されません
certs-certbots-last-renewal-failed = certbot の前回の更新に失敗しました: { $last }
certs-exit = 終了コード { $status }: { $stderr }
certs-isnt-a-certificate-name = ✕ { $name } は証明書の名前ではありません。
certs-dry-run-passed = ✓ ドライランに成功: { $tail }
certs-the-dry-run-failed-so = ✕ ドライランに失敗したため、何も更新していません: { $e }
certs-not-due-yet-certbot-renews = = まだ時期ではありません: certbot は期限の 30 日前以内に更新し、Let's Encrypt は証明書の発行回数を制限しています。

certs-renewed = ✓ 更新しました: { $tail }
certs-renewal-failed = ✕ 更新に失敗しました: { $e }
certs-isnt-a-domain = ✕ { $domain } はドメインではありません。
certs-acme-sh-has-no-dry = · acme.sh にはドライランがありません: 直接更新します。
certs-not-due-yet-acme-sh = = まだ時期ではありません（acme.sh は発行から 60 日後に更新）。
certs-nginx-config-checked-and-reloaded = ✓ nginx の設定を確認して再読み込みしました。
certs-nginx-wasnt-reloaded-it-still = ✕ nginx を再読み込みできませんでした（{ $e }）: 再読み込みするまで古い証明書を使い続けます。
certs-apache-config-checked-and-reloaded = ✓ Apache の設定を確認して再読み込みしました。
certs-apache-wasnt-reloaded-it-still = ✕ Apache を再読み込みできませんでした（{ $e }）: 再読み込みするまで古い証明書を使い続けます。
certs-the-certificate-now-expires = ✓ 証明書の期限は { $expiry } になりました。
certs-couldnt-re-read-the-certificate = · 新しい期限を確認するための証明書の読み直しができませんでした。

## keys
keys-ed25519-recommended = Ed25519（推奨）
keys-rsa-4096-bit = RSA 4096 ビット
keys-failed-to-create-destination-directory = 保存先フォルダー「{ $target_dir }」を作成できませんでした: { $e }
keys-failed-to-generate-ed25519-key = Ed25519 鍵を生成できませんでした: { $e }
keys-failed-to-generate-rsa-4096 = RSA 4096 鍵を生成できませんでした: { $e }
keys-failed-to-format-public-key = 公開鍵を整形できませんでした: { $e }
keys-failed-to-format-private-key = 秘密鍵を整形できませんでした: { $e }
keys-the-keys-file-name-cant = 鍵のファイル名に / は使えず、. や .. にもできません
keys-public-key-file-already-exists = 公開鍵ファイルが「{ $pub_path }」にすでにあります
keys-private-key-file-already-exists = 秘密鍵ファイルが「{ $priv_path }」にすでにあります
keys-failed-to-write-private-key = 秘密鍵を「{ $priv_path }」に書き込めませんでした: { $e }
keys-failed-to-write-public-key = 公開鍵を「{ $pub_path }」に書き込めませんでした: { $e }

## keys-deploy
keys-deploy-the-new-key-isnt-an = 新しい鍵は OpenSSH の公開鍵ではありません
keys-deploy-has-no-home-directory-on = { $user } にはサーバー上のホームディレクトリがありません
keys-deploy-the-new-key-didnt-log = 新しい鍵でログインできませんでした（{ $e }）。鍵は再び削除し、何も切り替えていません

## host-bootstrap
host-bootstrap-is-not-a-valid-host = { $host } は有効なホストではありません
host-bootstrap-cant-locate-the-crow-binary = askpass 用の Crow の実行ファイルが見つかりません: { $e }
host-bootstrap-couldnt-run-ssh = ssh を実行できませんでした: { $version }
host-bootstrap-askpass-socket = askpass のソケット: { $e }
host-bootstrap-the-key-install-script-didnt = 鍵の設置スクリプトが完了しませんでした: { $stderr }
host-bootstrap-installing-the-key-failed = 鍵の設置に失敗しました: { $stderr }
host-bootstrap-timed-out-logging-in-with = パスワードでのログインがタイムアウトしました

## topology
topology-its-health-check-fails = ヘルスチェックが失敗しています
topology-stopped-with-an-error = エラーで停止: { $status }
topology-restarted-times = { $n } 回再起動
topology-port-published-to-every-network = ポート { $host_port }（→ { $container }）がすべてのネットワークに公開
topology-dsa-key-broken-and-refused = DSA 鍵: 脆弱で、現在の OpenSSH では拒否されます
topology-rsa-bit-under-the-3072 = RSA { $bits } ビット: 現在の推奨である 3072 ビット未満
topology-crow-2 = Crow
topology-this-machine = このマシン
topology-host-key-not-pinned-crow = ホスト鍵が未固定: このサーバーがどの鍵を持つべきか Crow は記録していません
topology-crow-logs-in-with-a = Crow は鍵ではなくパスワードでログインしています
topology-jump-host-for-servers-if = { $n } 台の踏み台: 停止すると Crow はそれらに届きません
topology-servers = サーバー
topology-days-old-past-the-day = { $days } 日経過: { $key_max_age_days } 日ごとの更新の目安を超過
topology-no-private-key-file-recorded = このマシンに秘密鍵ファイルの記録がありません
topology-keys = 鍵

## team
team-restart-services = サービスの再起動
team-edit-configs = 設定の編集
team-fleet-runs = フリート実行
team-users-firewall = ユーザーとファイアウォール
team-ssh-keys = SSH 鍵
team-vault-settings = ボールトと設定
team-viewer = 閲覧者
team-operator = 運用者
team-admin = 管理者
team-the-whole-fleet = フリート全体
team-text = { $count ->
       *[other] グループ
    } { $g }
team-sign-in-first-this-crow = 先にサインインしてください: この Crow にはチームがあります
team-is-a-needs-a-role = { $name } は{ $name2 }です: { $label } には { $label } を許可する役割が必要です
team-isnt-in-s-scope = { $name } は { $name2 } の範囲（{ $describe }）に含まれません
team-a-passphrase-needs-at-least = パスフレーズは 8 文字以上必要です

## update
update-this-crow-runs-from-rather = この Crow はインストールされた Crow.app ではなく { $exe } から動いているため、置き換えません
update-no-home-directory = ホームディレクトリがありません
update-this-crow-runs-from-rather-2 = この Crow はインストールされた { $installed } ではなく { $exe } から動いているため、置き換えません（ソースからビルドし直してください）
update-theres-no-release-build-for = このプラットフォーム向けのリリースビルドはありません
update-has-no = { $tag } には { $name } がありません
update-has-no-provenance-intoto-jsonl = { $tag } には来歴情報（.intoto.jsonl）がないため、検証できません
update-installing-updates-needs-slsa-verifier = 更新のインストールには、リリースの署名付き来歴を確認する slsa-verifier（github.com/slsa-framework/slsa-verifier）が必要ですが、PATH にありません。何もインストールしていません
update-verification-failed = 検証に失敗しました
update-crow-is-installed-verified-restart = Crow { $version } をインストールしました（検証済み）。使うには Crow を再起動してください。
update-couldnt-move-the-old-crow = 古い Crow.app を退避できませんでした: { $e }
update-the-release-didnt-verify-so = リリースの検証に失敗したため、何もインストールしていません: { $from_utf8_lossy }

## providers
providers-isnt-compiled-into-this-build = { $plugin } はこのビルドに含まれていません
providers-required = { $names } が必要です{ $count ->
       *[other] {""}
    }
providers-provider-secrets-are-only-stored = プロバイダーのシークレットは暗号化してしか保存しませんが、暗号化キーがありません
providers-the-stored-isnt-text = 保存された { $field } はテキストではありません
providers-cant-power-instances = { $name } はインスタンスの電源を操作できません
providers-accepted-boot = { $name } が受け付けました: { $id } を起動
providers-accepted-reboot = { $name } が受け付けました: { $id } を再起動
providers-accepted-shut-down = { $name } が受け付けました: { $id } をシャットダウン
providers-cant-take-snapshots = { $name } はスナップショットを作成できません
providers-snapshot-started-for = { $id2 } の { $name } スナップショット { $id }（{ $status }）を開始しました

## incident
incident-alert = アラート
incident-resolved = 解決
incident-change = 変更
incident-check = チェック
incident-log = ログ

## config-push
config-push-removed-it-didnt-exist-before = 削除しました（元は存在しなかった）
config-push-removed-with-none-of-it = { $created_dirs } とともに削除しました（どれも元は存在しなかった）
config-push-couldnt-read = { $path } を読み取れませんでした: { $e }
config-push-already-matches-the-baseline = すでにベースラインと一致
config-push-not-written = 書き込みませんでした: { $e }
config-push-not-written-the-hosts-validator = 書き込みませんでした: ホストの検証ツールがベースラインを拒否しました: { $e }

config-push-the-vault-is-busy = ボールトが使用中です
config-push-written = 書き込みました（{ $differences } { $differences ->
       *[other] 件の差分
    }）
config-push-history-not-recorded = { $what }。履歴は記録されませんでした: { $e }

## config-drift
config-drift-missing = 不足: { $r }
config-drift-extra = 余分: { $r }
config-drift-same-lines-in-a-different = 同じ行が別の順序で並んでいます（このファイルでは順序が重要）
config-drift-differs-from-the-baseline-only = ベースラインと異なります（ハッシュしか保存していないため、どう違うかは分かりません）
config-drift-group = { $scope } グループ
config-drift-more = {" "}（ほか { $d } 件）
config-drift-drifted-from-the-baseline = { $path } が { $scope } のベースラインからドリフトしました: { $first }{ $more }

## metrics-certs
metrics-certs-certificate-for = { $name } の証明書
metrics-certs-expired-day-ago = { $d } { $d ->
       *[other] 日
    }前に期限切れ
metrics-certs-expires-today = 今日期限切れ
metrics-certs-expires-tomorrow = 明日期限切れ
metrics-certs-expires-in-days = { $d } 日後に期限切れ

## host-reboot
host-reboot-not-rebooted-couldnt-read-the = 再起動していません: 先に起動の目印を読み取れませんでした: { $e }
host-reboot-back-after-s = { $as_secs } 秒後に復帰
host-reboot-the-reboot-request-failed-and = 再起動の要求に失敗し（{ $e }）、サーバーは再起動しませんでした
host-reboot-not-back-after-s = { $as_secs } 秒後も復帰せず

## rollout
rollout-couldnt-run-it-exit = 実行できませんでした（終了コード { $status }）: { $last_lines }
rollout-done-no-output = 完了（出力なし）
rollout-ended-without-an-exit-code = 終了コードなしで終わりました: { $tail }
rollout-exit = 終了コード { $n }: { $tail }

## people
people-no-comment = コメントなし
people-crow-logs-in-to-as = Crow は { $server } に { $user } としてログインしています: 退職処理すると Crow が締め出されます
people-root-isnt-offboarded-turn-off = root は退職処理の対象外です: 代わりに ハードニング で root の SSH ログインをオフにしてください
people-no-account-on = { $server } に { $user } のアカウントはありません
people-its-the-key-crow-logs = Crow が { $server } へのログインに使っている鍵です: 先に Crow を別の鍵に切り替えてください（鍵 → 切り替え）
people-not-on = { $server } にはありません
people-that-isnt-an-openssh-public = OpenSSH の公開鍵ではありません（ssh-ed25519 AAAA… コメント）

## terminal-session
terminal-session-session-ended = セッション終了
terminal-session-session-ended-exit-status = セッション終了（終了ステータス { $code }）
terminal-session-session-ended-killed-by-a = セッション終了（シグナルで強制終了）

## geoip
geoip-the-geoip-database-couldnt-be = GeoIP データベースを読み取れませんでした: { $e }
geoip-the-geoip-database-has-no = GeoIP データベースに国のデータがありません
geoip-no-data-directory = データフォルダーがありません
geoip-bad-database-path = データベースのパスが正しくありません
geoip-the-geoip-database-couldnt-be-2 = GeoIP データベースをダウンロードできませんでした（{ $last }）

## appearance
appearance-isnt-a-picture-crow-can = { $source } は Crow が読める画像ではありません: { $e }
appearance-cant-create = { $dir } を作成できません: { $e }
appearance-cant-save = { $dest } を保存できません: { $e }
appearance-cant-read = { $stored } を読み取れません: { $e }
appearance-cant-save-2 = { $cached } を保存できません: { $e }

## vault
vault-the-vault-is-locked = ボールトはロックされています
vault-still-reading-the-os-keyring = OS のキーリングをまだ読み取り中です。少し待ってからもう一度お試しください
vault-no-os-keyring-is-available = 暗号化キーを保持できる OS のキーリングがありません（{ $why }）。代わりにボールトとセキュリティでボールトのパスワードをオンにしてください
vault-db-lock-poisoned = DB のロックが壊れています

## security-hardening
security-hardening-isnt-a-setting-crow-applies = ✕ { $value } は Crow が適用する設定ではありません。
security-hardening-couldnt-read-sshd-config-nothing = ✕ sshd_config を読み取れませんでした: { $e }。何も変更していません。
security-hardening-couldnt-write-nothing-was-changed = ✕ { $root_dropin } を書き込めませんでした: { $e }。何も変更していません。

## security-sshd-passwords
security-sshd-passwords-a-fresh-key-login-doesnt-2 = ✕ 新規の鍵ログインが使えません（{ $e }）。何も変更していません: 今パスワードをオフにすると締め出されるおそれがあります。
security-sshd-passwords-couldnt-read-sshds-settings-nothing = ✕ sshd の設定を読み取れませんでした: { $e }。何も変更していません。
security-sshd-passwords-couldnt-create-etc-ssh-sshd-2 = ✕ /etc/ssh/sshd_config.d を作成できませんでした: { $e }。何も変更していません。
security-sshd-passwords-couldnt-write-nothing-was-changed = ✕ { $dropin } を書き込めませんでした: { $e }。何も変更していません。

## host-diagnose
host-diagnose-sshd-may-be-stopped-or = sshd が停止しているか、別のポートで待ち受けているのかもしれません。

## security-hardening
security-hardening-sshd-accepts-passwords-passwordauthentication-yes = sshd がパスワードを受け付ける（PasswordAuthentication yes）

## lab-multipass
lab-multipass-multipass-launch-didnt-finish-in = multipass launch が { $launch } 分以内に終わりませんでした
lab-multipass-multipass-launch-failed = multipass launch に失敗しました（{ $status }）
lab-multipass-multipass-failed = multipass が失敗しました

## dr
dr-last-backup-is-old-more = 最新のバックアップ { $name } は { $age } 前のもの: RPO の { $rpo_minutes } 分を超えています
dr-last-backup-is-old = 最新のバックアップ { $name } は { $age } 前のもの

## certs
certs-certbot-is-installed-but-nothing = certbot はインストールされていますが、実行するもの（タイマーも cron も）がありません: 証明書は自動で更新されません
certs-acme-sh-is-installed-but = acme.sh はインストールされていますが、cron ジョブがありません: 証明書は自動で更新されません

## team
team-env = 環境 { $e }

## Settings → Personalisation → Language
app-language-couldnt-load = 言語パック { $code } を読み込めなかったため、Crow は英語で表示しています: { $e }
app-language-couldnt-create = { $path } を作成できませんでした: { $e }
app-language-template-written = 英語のテンプレートを { $path } に書き出しました。ファイル名を言語のコード（es.ftl、ja.ftl）に変え、各 = の後の文を翻訳してから 再読み込み を押してください。
app-language-template-not-written = テンプレートを書き出せませんでした: { $e }
settings-language-title = 言語
settings-language-about = Crow の画面の言語です。言語は自分で作るかダウンロードする .ftl ファイルで、{ $folder } に置きます。どれも英語と照合され、足りないものや誤りのあるものは英語で表示されます。
settings-language-open-folder = 言語フォルダーを開く
settings-language-export-template = 英語のテンプレートを書き出す
settings-language-reload = 再読み込み
settings-language-complete = 完全
settings-language-coverage = { $percent }% · 不足 { $missing }
settings-language-by = 作成: { $author }
settings-language-refused = { $count } { $count ->
       *[other] 件
    }のメッセージを採用しませんでした（構文エラー { $errors } 件）。英語で表示します:
settings-language-more = …ほか { $count } 件

## Converted by hand
fleet-rollouts-as-root-sudo = root として（sudo -n 経由）
users-new-user-modal-executes-under-sudo = sudo で useradd と usermod -aG を実行します
fleet-overview-host-count = · { $count } { $count ->
       *[other] 台
    }
settings-clankers-all-time = 通算 { $count }
logs-explain-lines = { $count } { $count ->
       *[other] 行
    }を説明

app-configs-value-placeholder = 値
fleet-alert-lines-down-now = { $reason } · 次のチェックでも停止していればアラート
fleet-alert-lines-now = 今
fleet-alert-lines-resolved = 解決 · { $detail }（継続 { $lasted }）
config-crawler-could-not-read-from = { $host } から { $name } を読み取れませんでした: { $e }
config-crawler-could-not-read = { $host } から { $name } を読み取れませんでした
config-pending-diff-rail-clean = 変更なし
app-overview-search-journal = ジャーナルを grep（正規表現、小文字なら大文字小文字を区別しない）…  ( / )

## Danger Zone
danger-zone-what-poweroff = サーバーの電源を切る
danger-zone-what-reboot = サーバーを再起動する
danger-zone-what-boot = サーバーを起動する
danger-zone-what-snapshot = サーバーのスナップショットを作成する
danger-zone-what-flush = ファイアウォールのルールをすべて削除し、無効にする
danger-zone-what-kill = Crow が作成したラボコンテナーをすべて停止する
danger-zone-what-other = この操作を実行する

## Settings rows
setting-general-refresh-interval-label = 更新間隔
setting-general-refresh-interval-help = サービスとテレメトリーをバックグラウンドで取得する間隔（秒）。
setting-connection-control-master-label = 接続の共有
setting-connection-control-master-help = ホストごとに 1 本の TCP 接続をすべてのパネルで共有します。ハンドシェイクが約 14 回から 1 回に減ります。
setting-connection-keepalive-interval-label = キープアライブの間隔
setting-connection-keepalive-interval-help = キープアライブの確認の間隔（秒）。短いほど停止したホストを早く検出しますが、通信が増えます。
setting-connection-connect-timeout-label = 接続のタイムアウト
setting-connection-connect-timeout-help = TCP のハンドシェイクをこの時間で打ち切ります。
setting-security-auto-lock-minutes-label = 自動ロック（分）
setting-security-auto-lock-minutes-help = 操作がない状態が続いたらボールトをロックし、復号した鍵を RAM から消去します。
setting-appearance-fleet-background-label = フリートの背景
setting-appearance-fleet-background-help = フリートのサーバー一覧の背後に置く画像（~/.config/crow/backgrounds にコピーを保存）。空欄でなし。
setting-appearance-fleet-background-opacity-label = フリートの背景の不透明度
setting-appearance-fleet-background-opacity-help = 画像をどれだけ透かすか。低いほど表が読みやすくなります。
setting-appearance-fleet-background-opacity-option-5 = 5% · ほのかに
setting-appearance-fleet-background-opacity-option-10 = 10% · 控えめ
setting-appearance-fleet-background-opacity-option-15 = 15% · はっきり
setting-appearance-fleet-background-opacity-option-25 = 25% · 強め
setting-appearance-fleet-background-opacity-option-35 = 35% · 大胆
setting-appearance-fleet-background-blur-label = フリートの背景のぼかし
setting-appearance-fleet-background-blur-help = 画像をぼかして、その上の行が読みやすいようにします。
setting-appearance-fleet-background-blur-option-0 = なし
setting-appearance-fleet-background-blur-option-4 = 弱
setting-appearance-fleet-background-blur-option-8 = やわらか
setting-appearance-fleet-background-blur-option-16 = 強
setting-appearance-fleet-background-blur-option-32 = すりガラス
setting-servers-archive-purge-days-label = アーカイブの削除までの日数
setting-servers-archive-purge-days-help = アーカイブしたサーバーの保存データを Crow が削除するまでの期間です。サーバーの記録そのものは削除しません。
setting-servers-archive-purge-days-option-30 = 30 日
setting-servers-archive-purge-days-option-60 = 60 日
setting-servers-archive-purge-days-option-90 = 90 日 · 既定
setting-servers-archive-purge-days-option-180 = 180 日
setting-servers-archive-purge-days-option-never = 削除しない
setting-appearance-terminal-font-size-label = ターミナルの文字サイズ
setting-appearance-terminal-font-size-help = サーバーのターミナルの文字サイズ（ポイント）。
setting-appearance-start-maximized-label = 最大化して起動
setting-appearance-start-maximized-help = Crow を最大化して開きます。次回の起動から反映されます。
setting-appearance-language-label = 言語
setting-appearance-language-help = Crow の画面の言語: en、または言語フォルダーにあるパックのコード（es.ftl なら es）。
setting-servers-host-key-max-age-days-label = ホスト鍵の最長使用日数
setting-servers-host-key-max-age-days-help = サーバーの SSH ホスト鍵がどれだけ古くなったら、フリートのアラートで更新時期を知らせるか（フリート → 危険ゾーン → すべてのホスト鍵を更新）。
setting-servers-host-key-max-age-days-option-180 = 180 日
setting-servers-host-key-max-age-days-option-365 = 1 年 · 既定
setting-servers-host-key-max-age-days-option-730 = 2 年
setting-servers-host-key-max-age-days-option-never = 知らせない
setting-servers-geoip-regions-label = GeoIP によるリージョン
setting-servers-geoip-regions-help = クラウドのメタデータがないサーバーの国を、公開 IP からこのマシンで DB-IP の無料の国別データベース（db-ip.com から月に 1 回ほどダウンロード。サーバーのアドレスはどこにも送りません）を引いて調べます。IP 位置情報は DB-IP（db-ip.com）、CC BY 4.0。
setting-general-update-check-label = 更新の確認
setting-general-update-check-help = 1 日に 1 回 GitHub で新しい Crow を確認し、バージョンの横に表示します。リリースの一覧を取得するだけで、あなたやサーバーについては何も送りません。
setting-general-update-prereleases-label = プレリリースも対象
setting-general-update-prereleases-help = プレリリース（ベータ）も提示します。プレリリース自体のビルドには常に新しいものを提示します。
setting-general-update-auto-install-label = 更新を自動でインストール
setting-general-update-auto-install-help = 新しいバージョンを自動でダウンロードしてインストールします。ただし slsa-verifier（要インストール）でリリースの署名付き来歴を確認した後に限ります。Crow は再起動を提案するだけで、自分から再起動はしません。

## Plugins
plugin-multipass-name = Multipass
plugin-multipass-about = このマシン上の Ubuntu VM: Crow の鍵で起動するとフリートに加わります。取り込み、電源操作、削除ができます。
plugin-podman-name = Podman
plugin-podman-about = このマシン上のラボコンテナー。テストノードとして登録します。
plugin-distrobox-name = Distrobox
plugin-distrobox-about = Distrobox の環境（中身は Podman か Docker）をラボコンテナーとして。
plugin-docker-name = Docker
plugin-docker-about = Docker で動くラボコンテナー。
plugin-containers-name = サーバー上のコンテナー
plugin-containers-about = サーバー上の Docker と Podman を SSH で: コンテナー、compose スタック、公開ポート、ログ、開始/停止/再起動、そして魔法の杖。各サーバーのサイドバーにコンテナーを追加します。
plugin-linode-name = Linode
plugin-linode-about = Linode のインスタンス: 取り込み、リージョンの読み取り、電源操作、スナップショット。
plugin-upcloud-name = UpCloud
plugin-upcloud-about = UpCloud のサーバー: 取り込み、ゾーンの読み取り、電源操作、スナップショット。
plugin-email-name = Email
plugin-email-about = 自分の SMTP サーバーでアラートをメール送信: 重大なものはすぐに、警告はまとめて。Crow の実行中に送ります。

## Product names (shown as they are)
lab-distrobox = Distrobox
lab-podman = Podman
lab-multipass = Multipass
lab-docker = Docker
containers-docker = Docker
containers-podman = Podman

## Tests
i18n-test-args = { $server }: { $count ->
   *[other] サーバー { $count } 台
}
i18n-test-plain = こんにちは


## Audit log: what a fleet run's records mean
audit-model-offboarded-on = { $target } で退職処理
audit-model-key-revoked-on = { $target } で鍵を無効化
audit-model-account-set-up-on = { $target } でアカウントを作成
audit-model-certificate-renewed-on = { $target } で証明書を更新
audit-model-hardening-fix-on = { $target } でハードニングの修正
audit-model-risk-accepted-on = { $target } でリスクを許容
audit-model-command-run-on = { $target } でコマンドを実行
audit-model-config-rolled-out-to = { $target } に設定をロールアウト
audit-model-rollout-undone-on = { $target } でロールアウトを取り消し
audit-model-updates-applied-on = { $target } で更新を適用
audit-model-host-keys-rotated-on = { $target } でホスト鍵を更新
audit-model-sessions-ended-on = { $target } で SSH セッションを終了
audit-model-brought-back-on = { $target } をベースラインに戻した
audit-model-permissions-changed = 権限を変更: { $target }
audit-model-plugin-accepted = プラグイン { $target } を承認
audit-model-plugin-refused = プラグイン { $target } を拒否
audit-model-plugin-revoked = プラグイン { $target } の承認を取り消し
audit-model-plugin-ran = { $target }: { $action } を実行
identity-bar-kernel = カーネル
identity-bar-role = 役割

## Audit log filters
audit-all = すべて
audit-kind-config = 設定
audit-kind-service = サービス
audit-kind-process = プロセス
audit-kind-firewall = ファイアウォール
audit-kind-user = ユーザー
audit-kind-power = 電源
audit-kind-ssh = ssh
audit-kind-enroll = 登録
audit-kind-crow = crow

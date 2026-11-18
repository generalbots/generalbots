# Transactional email text (#2094). Subject and body only; MIME and envelope stay untouched.

email-greeting = { $name } 님께
email-signoff = 감사합니다.
email-team = General Bots 팀
email-welcome-subject = General Bots Cloud에 오신 것을 환영합니다 — { $plan } 플랜
email-welcome-body = General Bots Cloud에 오신 것을 환영합니다! { $plan } 플랜이 활성화되었습니다. { $url } 에서 봇을 만들고 워크스페이스를 관리할 수 있습니다.
email-invoice-subject = 청구서 #{ $invoice } — General Bots Cloud
email-invoice-body = 새 청구서가 발행되었습니다. 청구서: { $invoice }. 금액: { $amount } { $currency }. 자세한 내용은 { $url } 에서 확인하세요.
email-payment-success-subject = 결제가 확인되었습니다 — General Bots Cloud
email-payment-success-body = { $amount } { $currency } 결제가 확인되었습니다. 플랜: { $plan }. 구독해 주셔서 감사합니다!
email-subscription-activated-subject = 구독이 활성화되었습니다 — General Bots Cloud
email-subscription-activated-body = { $plan } 구독이 활성화되었습니다. { $url } 에서 관리할 수 있습니다.
email-recurring-charge-subject = 정기 결제 — 청구서 #{ $invoice }
email-recurring-charge-body = 정기 구독이 청구되었습니다. 청구서: { $invoice }. 금액: { $amount } { $currency }. 자세한 내용은 { $url } 에서 확인하세요.
email-payment-failed-subject = 결제 실패 — 조치가 필요합니다
email-payment-failed-body = { $amount } { $currency } 결제에 실패했습니다. 청구서: { $invoice }. { $url } 에서 결제 수단을 업데이트해 주세요.
email-subscription-cancelled-subject = 구독이 취소되었습니다 — General Bots Cloud
email-subscription-cancelled-body = { $plan } 구독이 취소되었습니다. 다시 활성화하려면 { $url } 을(를) 방문하세요.
email-grace-warning-subject = 결제 연체 — 서비스 정지 예정 안내
email-grace-warning-body = 결제가 연체되었습니다. { $days } 일 후 서비스가 정지됩니다. 즉시 { $url } 에서 결제 수단을 업데이트해 주세요.
email-invitation-subject = { $org } 초대
email-invitation-body = { $org } 에 참여하도록 초대되었습니다. { $url } 에서 초대를 수락하세요. 이 초대는 { $expires } 에 만료됩니다.
email-quota-limit-reached = { $metric } 할당량 한도에 도달했습니다.
email-quota-threshold = { $metric } 할당량의 { $current }%를 사용했습니다(임계값: { $threshold }%).
email-metric-messages = 메시지
email-metric-storage = 스토리지
email-metric-api-calls = API 호출
email-metric-bots = 봇
email-metric-users = 사용자
email-metric-kb-documents = 지식 베이스 문서
email-metric-apps = 앱
email-metric-emails-per-day = 일일 이메일
email-metric-builds-per-day = 일일 빌드
email-metric-uploads-per-hour = 시간당 업로드

# Sharing and mention notifications (#2094)
email-share-subject = { $sender } 님이 "{ $file }" 파일을 공유했습니다
email-share-download = { $file } 다운로드
email-share-link-expires = 이 링크는 { $date } 에 만료됩니다. 언제든지 취소할 수 있습니다.
email-share-link-no-expiry = 이 링크는 만료되지 않습니다. 언제든지 취소할 수 있습니다.
email-mention-subject = { $author } 님이 댓글에서 회원님을 언급했습니다
email-mention-body = { $author } 님이 { $resource_type } ({ $resource_id }) 에서 회원님을 언급했습니다. "{ $preview }" 앱을 열어 확인하고 답변하세요.

# semantic i18n key (#2542)
mail-new-message = 새 메시지

# semantic i18n key (#2542)
mail-cc-bcc = 참조/숨은참조

# semantic i18n key (#2542)
mail-subject-label = 제목:

# semantic i18n key (#2542)
mail-track-opens = 열람 추적

# semantic i18n key (#2542)
mail-request-read-receipt = 읽음 확인 요청

# semantic i18n key (#2542)
mail-priority-normal = 보통 우선순위

# semantic i18n key (#2542)
mail-priority-high = 높은 우선순위

# semantic i18n key (#2542)
mail-priority-low = 낮은 우선순위

# semantic i18n key (#2542)
mail-dialog-templates-title = 이메일 템플릿

# semantic i18n key (#2542)
mail-dialog-new-template = + 새 템플릿

# semantic i18n key (#2542)
mail-dialog-signatures-title = 이메일 서명

# semantic i18n key (#2542)
mail-dialog-new-signature = + 새 서명

# semantic i18n key (#2542)
mail-dialog-rules-title = 이메일 규칙

# semantic i18n key (#2542)
mail-dialog-new-rule = + 새 규칙

# semantic i18n key (#2542)
mail-dialog-automatic-replies-title = 자동 회신

# semantic i18n key (#2542)
mail-dialog-send-automatic-replies = 자동 회신 보내기

# semantic i18n key (#2542)
mail-dialog-start-date = 시작 날짜

# semantic i18n key (#2542)
mail-dialog-end-date = 종료 날짜

# semantic i18n key (#2542)
mail-dialog-reply-only-org = 조직 내 사용자에게만 보내기

# semantic i18n key (#2542)
mail-dialog-add-account-title = 이메일 계정 추가

# semantic i18n key (#2542)
mail-dialog-account-name = 계정 이름

# semantic i18n key (#2542)
mail-dialog-imap-server = IMAP 서버

# semantic i18n key (#2542)
mail-dialog-imap-port = IMAP 포트

# semantic i18n key (#2542)
mail-dialog-smtp-server = SMTP 서버

# semantic i18n key (#2542)
mail-dialog-smtp-port = SMTP 포트

# semantic i18n key (#2542)
mail-dialog-use-ssl-tls = SSL/TLS 사용

# semantic i18n key (#2542)
mail-dialog-add-account-action = 계정 추가

# semantic i18n key (#2542)
mail-dialog-schedule-send-title = 전송 예약

# semantic i18n key (#2542)
mail-dialog-schedule-action = 예약

# semantic i18n key (#2542)
mail-manage-labels-title = 라벨 관리

# semantic i18n key (#2542)
mail-label-name = 라벨 이름

# semantic i18n key (#2542)
mail-label-color = 색상

# semantic i18n key (#2542)
mail-new-signature-title = 새 이메일 서명

# semantic i18n key (#2542)
mail-signature-name = 서명 이름

# semantic i18n key (#2542)
mail-html-content = HTML 콘텐츠

# semantic i18n key (#2542)
mail-new-rule-title = 새 이메일 규칙

# semantic i18n key (#2542)
mail-rule-name = 규칙 이름

# semantic i18n key (#2542)
mail-condition-sender-contains = 조건(보낸 사람 포함)

# semantic i18n key (#2542)
mail-action-star = 별표 표시

# semantic i18n key (#2542)
mail-new-template-title = 새 이메일 템플릿

# semantic i18n key (#2542)
mail-template-name = 템플릿 이름

# semantic i18n key (#2542)
mail-body-html = 본문 HTML

# semantic i18n key (#2542)
mail-placeholder-label-name = 예: 업무
mail-placeholder-signature-name = 예: 업무용 서명
mail-placeholder-signature-content = 감사합니다.<br><b>이름</b>

# semantic i18n key (#2542)
mail-placeholder-rule-name = 예: 뉴스레터 보관
mail-placeholder-rule-condition = 예: newsletter
mail-placeholder-template-name = 예: 환영 이메일
mail-placeholder-template-subject = 기본 환영 제목
mail-placeholder-template-body = 안녕하세요 {{name}}님, 플랫폼에 오신 것을 환영합니다!
mail-placeholder-autoreply-subject = 회신 제목
mail-placeholder-autoreply-message = 자동 회신 메시지...

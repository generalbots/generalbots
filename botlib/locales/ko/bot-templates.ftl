bot-greeting-default = 안녕하세요! 오늘은 무엇을 도와드릴까요?
bot-greeting-named = 안녕하세요, { $name }! 오늘은 무엇을 도와드릴까요?
bot-goodbye = 안녕히 가세요! 좋은 하루 보내세요!
bot-help-prompt = 제가 도와드릴 수 있는 부분은 다음과 같습니다: { $topics }. 무엇을 알고 싶나요?
bot-thank-you = 메시지를 보내주셔서 감사합니다. 오늘은 어떻게 도와드릴까요?
bot-echo-intro = 에코봇: 당신이 말하는 모든 것을 반복하겠습니다. 종료하려면 'quit'을 입력하세요.
bot-you-said = 당신은 다음과 같이 말했습니다: { $message }
bot-thinking = 생각해 보도록 할게요...
bot-processing = 요청을 처리 중입니다...
bot-error-occurred = 죄송합니다. 문제가 발생했습니다. 다시 시도해 주세요.
bot-not-understood = 나는 그것을 이해하지 못했습니다. 다시 말해주시겠어요?
bot-confirm-action = 계속하시겠습니까?
bot-action-cancelled = 작업이 취소되었습니다.
bot-action-completed = 완료!

bot-lead-welcome = 환영! 시작할 수 있도록 도와드리겠습니다.
bot-lead-ask-name = 이름이 뭐에요?
bot-lead-ask-email = 이메일은요?
bot-lead-ask-company = 당신은 어느 회사 출신인가요?
bot-lead-ask-phone = 당신의 전화번호는 무엇입니까?
bot-lead-hot = 엄청난! 영업팀이 곧 연락을 드릴 것입니다.
bot-lead-nurture = 관심을 가져주셔서 감사합니다! 몇 가지 자료를 보내드리겠습니다.
bot-lead-score = 귀하의 리드 점수는 100점 만점에 { $score }입니다.
bot-lead-saved = 귀하의 정보가 성공적으로 저장되었습니다.

bot-schedule-created = 예약된 작업 실행 중: { $name }
bot-schedule-next = 다음 실행은 { $datetime }로 예정되어 있습니다.
bot-schedule-cancelled = 일정이 취소되었습니다.
bot-schedule-paused = 일정이 일시중지되었습니다.
bot-schedule-resumed = 일정이 재개되었습니다.

bot-monitor-alert = 경고: { $subject }가 변경되었습니다.
bot-monitor-threshold = { $metric } 임계값을 초과했습니다: { $value }
bot-monitor-recovered = { $subject }가 정상으로 돌아왔습니다.
bot-monitor-status = 현재 상태: { $status }

bot-order-welcome = 우리 가게에 오신 것을 환영합니다! 어떻게 도와드릴까요?
bot-order-track = 내 주문 추적
bot-order-browse = 제품 찾아보기
bot-order-support = 지원팀에 문의
bot-order-enter-id = 주문 번호를 입력하세요:
bot-order-status = 주문 상태: { $status }
bot-order-shipped = 귀하의 주문이 배송되었습니다! 추적 번호: { $tracking }
bot-order-delivered = 귀하의 주문이 배송되었습니다.
bot-order-processing = 귀하의 주문을 처리 중입니다.
bot-order-cancelled = 주문이 취소되었습니다.
bot-order-ticket = 지원 티켓 생성됨: #{ $ticket }
bot-order-products-available = 사용 가능한 제품은 다음과 같습니다.
bot-order-product-item = { $name } - { $price }
bot-order-cart-added = 장바구니에 { $product }을 추가했습니다.
bot-order-cart-total = 장바구니 총액은 { $total }입니다.
bot-order-checkout = 결제 진행 중...

bot-hr-welcome = HR 보조원이 여기 있습니다. 어떻게 도와드릴까요?
bot-hr-request-leave = 휴가 요청
bot-hr-check-balance = 잔액 확인
bot-hr-view-policies = 정책 보기
bot-hr-leave-type = 어떤 종류의 휴가인가요? (휴가/병/개인)
bot-hr-start-date = 시작일? (YYYY-MM-DD)
bot-hr-end-date = 종료일? (YYYY-MM-DD)
bot-hr-leave-submitted = 요청을 제출했습니다. 관리자가 이를 검토할 것입니다.
bot-hr-leave-approved = 귀하의 휴가 요청이 승인되었습니다.
bot-hr-leave-rejected = 귀하의 휴가 요청이 거부되었습니다.
bot-hr-leave-pending = 귀하의 휴가 요청이 승인 대기 중입니다.
bot-hr-balance-title = 휴가 잔액:
bot-hr-vacation-days = 휴가: { $days }일
bot-hr-sick-days = 병가: { $days }일
bot-hr-personal-days = 개인: { $days }일
bot-hr-policy-found = 요청하신 정책 정보는 다음과 같습니다.
bot-hr-policy-not-found = 정책을 찾을 수 없습니다. 정책명을 확인해주세요.

bot-health-welcome = 저희 건강관리센터에 오신 것을 환영합니다. 어떻게 도와드릴까요?
bot-health-book = 약속 예약
bot-health-cancel = 약속 취소
bot-health-view = 내 약속 보기
bot-health-reschedule = 약속 일정 변경
bot-health-type = 어떤 종류의 약속인가요? (일반/전문가/실험실)
bot-health-doctor = 어떤 의사를 선호하시나요?
bot-health-date = 어떤 날짜가 당신에게 가장 적합합니까?
bot-health-time = 몇 시를 선호하시나요?
bot-health-confirmed = { $doctor }와의 { $datetime } 약속이 확정되었습니다.
bot-health-cancelled = 약속이 취소되었습니다.
bot-health-rescheduled = 약속이 { $datetime }로 변경되었습니다.
bot-health-reminder = 알림: { $datetime }에 약속이 있습니다.
bot-health-no-appointments = 예정된 약속이 없습니다.
bot-health-appointments-list = 예정된 약속:

bot-support-welcome = 오늘은 무엇을 도와드릴까요?
bot-support-describe = 문제를 설명해 주세요.
bot-support-category = 귀하의 문제를 가장 잘 설명하는 카테고리는 무엇입니까?
bot-support-priority = 이 문제는 얼마나 긴급한가요?
bot-support-ticket-created = 지원 티켓 #{ $ticket }이 생성되었습니다.
bot-support-ticket-status = 티켓 #{ $ticket } 상태: { $status }
bot-support-ticket-updated = 티켓이 업데이트되었습니다.
bot-support-ticket-resolved = 귀하의 티켓이 해결되었습니다. 추가 지원이 필요한 경우 알려주시기 바랍니다.
bot-support-transfer = 상담사에게 연결하는 중...
bot-support-wait-time = 예상 대기 시간: { $minutes }분.
bot-support-agent-joined = { $name } 요원이 대화에 참여했습니다.

bot-survey-intro = 우리는 귀하의 의견을 듣고 싶습니다!
bot-survey-question = { $question }
bot-survey-scale = 1~10점 중 { $subject }을 어떻게 평가하시겠습니까?
bot-survey-open = 추가 의견이 있으면 공유해 주세요.
bot-survey-thanks = 피드백을 보내주셔서 감사합니다!
bot-survey-completed = 설문조사가 성공적으로 완료되었습니다.
bot-survey-skip = 원한다면 이 질문을 건너뛸 수 있습니다.

bot-notification-new-message = { $sender }에서 새 메시지가 왔습니다.
bot-notification-task-due = "{ $task }" 작업이 { $when } 마감입니다.
bot-notification-reminder = 알림: { $message }
bot-notification-update = 업데이트: { $message }
bot-notification-alert = 경고: { $message }

bot-command-help = 사용 가능한 명령:
bot-command-unknown = 알 수 없는 명령입니다. 사용 가능한 명령을 보려면 'help'를 입력하세요.
bot-command-invalid = 잘못된 명령 구문입니다. 사용법: { $usage }

bot-transfer-to-human = 귀하를 상담원에게 연결하는 중입니다. 기다려 주세요...
bot-transfer-complete = 이제 { $agent }와 연결되었습니다.
bot-transfer-unavailable = 현재 상담원이 없습니다. 나중에 다시 시도해 주세요.
bot-transfer-queue-position = 당신은 대기열의 { $position }번입니다.

bot-auth-login-prompt = 계속하려면 자격 증명을 입력하세요.
bot-auth-login-success = 성공적으로 로그인되었습니다.
bot-auth-login-failed = 로그인에 실패했습니다. 자격 증명을 확인하세요.
bot-auth-logout-success = 로그아웃되었습니다.
bot-auth-session-expired = 세션이 만료되었습니다. 다시 로그인해주세요.

bot-file-upload-prompt = 파일을 업로드해주세요.
bot-file-upload-success = '{ $filename }' 파일이 성공적으로 업로드되었습니다.
bot-file-upload-failed = 파일을 업로드하지 못했습니다. 다시 시도해 주세요.
bot-file-download-ready = 파일을 다운로드할 준비가 되었습니다.
bot-file-processing = 파일을 처리하는 중...

bot-payment-amount = 총 금액은 { $amount }입니다.
bot-payment-method = 결제수단을 선택해주세요.
bot-payment-processing = 결제 처리 중...
bot-payment-success = 결제 성공! 거래 ID: { $transactionId }
bot-payment-failed = 결제에 실패했습니다. 다시 시도하거나 다른 결제 수단을 사용해 보세요.
bot-payment-refund = { $amount } 환불이 처리되었습니다.

bot-subscription-active = 귀하의 구독은 { $endDate }까지 유효합니다.
bot-subscription-expired = 구독이 만료되었습니다.
bot-subscription-renew = 구독을 갱신하시겠습니까?
bot-subscription-upgraded = 구독이 { $plan }로 업그레이드되었습니다.
bot-subscription-cancelled = 구독이 취소되었습니다.

bot-feedback-positive = 긍정적인 피드백을 보내주셔서 감사합니다!
bot-feedback-negative = 그 소식을 들으니 안타깝습니다. 어떻게 개선할 수 있나요?
bot-feedback-rating = 이 상호작용을 5점 만점에 { $rating }로 평가하셨습니다.

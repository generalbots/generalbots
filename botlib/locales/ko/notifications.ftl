notification-title-new-message = 새 메시지
notification-title-task-due = 작업 기한
notification-title-task-assigned = 할당된 작업
notification-title-task-completed = 작업 완료
notification-title-meeting-reminder = 회의 알림
notification-title-meeting-started = 회의가 시작되었습니다
notification-title-file-shared = 파일 공유
notification-title-file-uploaded = 파일이 업로드됨
notification-title-comment-added = 새 댓글
notification-title-mention = 당신이 언급되었습니다
notification-title-system = 시스템 알림
notification-title-security = 보안 경고
notification-title-update = 업데이트 가능
notification-title-error = 오류가 발생했습니다
notification-title-success = 성공
notification-title-warning = 경고
notification-title-info = 정보

notification-message-new = { $sender } 님으로부터 새 메시지가 왔습니다.
notification-message-unread = 귀하는 { $count ->
    [one] { $count } unread message
   *[other] { $count } unread messages
}
notification-task-due-soon = '{ $task }' 작업이 { $time }에 마감됩니다.
notification-task-due-today = '{ $task }' 작업이 오늘 마감입니다
notification-task-due-overdue = '{ $task }' 작업이 { $time }까지 기한이 지났습니다.
notification-task-assigned-to-you = 귀하는 "{ $task }" 작업에 할당되었습니다
notification-task-assigned-by = { $assigner }가 귀하를 "{ $task }"에 할당했습니다
notification-task-completed-by = { $user } 완료된 작업 "{ $task }"
notification-task-status-changed = 작업 "{ $task }" 상태가 { $status }로 변경되었습니다.

notification-meeting-in-minutes = { $minutes }분 후에 '{ $meeting }' 회의가 시작됩니다.
notification-meeting-starting-now = '{ $meeting }' 회의가 지금 시작됩니다
notification-meeting-cancelled = '{ $meeting }' 회의가 취소되었습니다.
notification-meeting-rescheduled = '{ $meeting }' 회의 일정이 { $datetime }로 변경되었습니다.
notification-meeting-invite = { $inviter } 님이 '{ $meeting }'에 귀하를 초대했습니다.
notification-meeting-response = { $user } { $response } 회의 초대

notification-file-shared-with-you = { $sharer }님이 '{ $filename }'을(를) 공유했습니다.
notification-file-uploaded-by = { $uploader } 업로드된 "{ $filename }"
notification-file-modified = "{ $filename }"이(가) { $user }에 의해 수정되었습니다.
notification-file-deleted = '{ $filename }'이 { $user }에 의해 삭제되었습니다.
notification-file-download-ready = '{ $filename }' 파일을 다운로드할 준비가 되었습니다.
notification-file-upload-complete = '{ $filename }' 업로드가 성공적으로 완료되었습니다.
notification-file-upload-failed = '{ $filename }' 업로드 실패

notification-comment-on-task = { $user }가 "{ $task }" 작업에 댓글을 달았습니다.
notification-comment-on-file = { $user }이 "{ $filename }"에 댓글을 달았습니다.
notification-comment-reply = { $user }이 귀하의 댓글에 답글을 달았습니다.
notification-mention-in-comment = { $user }가 댓글에서 귀하를 멘션했습니다.
notification-mention-in-chat = { $user }이 { $channel }에서 당신을 멘션했습니다.

notification-login-new-device = { $location }의 { $device }에서 새로운 로그인이 감지되었습니다.
notification-login-failed = 귀하의 계정에 로그인 시도가 실패했습니다
notification-password-changed = 비밀번호가 성공적으로 변경되었습니다
notification-password-expiring = 귀하의 비밀번호는 { $days }일 후에 만료됩니다.
notification-session-expired = 세션이 만료되었습니다
notification-account-locked = 귀하의 계정이 잠겼습니다.
notification-two-factor-enabled = 이중 인증이 활성화되었습니다
notification-two-factor-disabled = 이중 인증이 비활성화되었습니다.

notification-subscription-expiring = 구독이 { $days }일 후에 만료됩니다.
notification-subscription-expired = 구독이 만료되었습니다
notification-subscription-renewed = 구독이 { $date }까지 갱신되었습니다.
notification-payment-successful = { $amount } 결제가 완료되었습니다.
notification-payment-failed = { $amount } 결제 실패
notification-invoice-ready = { $period }에 대한 인보이스가 준비되었습니다.

notification-bot-response = { $bot }명이 귀하의 문의에 응답했습니다.
notification-bot-error = { $bot } 오류가 발생했습니다.
notification-bot-offline = { $bot }는 현재 오프라인 상태입니다
notification-bot-online = { $bot } 현재 온라인 상태입니다
notification-bot-updated = { $bot }가 업데이트되었습니다

notification-system-maintenance = { $datetime }에 시스템 점검 예정
notification-system-update = 시스템 업데이트 가능: { $version }
notification-system-restored = 시스템이 복원되었습니다.
notification-system-degraded = 시스템 성능이 저하되고 있습니다

notification-action-view = 보기
notification-action-dismiss = 닫기
notification-action-mark-read = 읽음으로 표시
notification-action-mark-all-read = 모두 읽음으로 표시
notification-action-settings = 알림 설정
notification-action-reply = 답장하다
notification-action-open = 열기
notification-action-join = 가입
notification-action-accept = 수락
notification-action-decline = 거절하다

notification-time-just-now = 지금 막
notification-time-minutes = { $개수 ->
    [one] { $count } minute ago
   *[other] { $count } minutes ago
}
notification-time-hours = { $개수 ->
    [one] { $count } hour ago
   *[other] { $count } hours ago
}
notification-time-days = { $개수 ->
    [one] { $count } day ago
   *[other] { $count } days ago
}
notification-time-weeks = { $개수 ->
    [one] { $count } week ago
   *[other] { $count } weeks ago
}

notification-preference-all = 모든 알림
notification-preference-important = 중요한 것만
notification-preference-none = 없음
notification-preference-email = 이메일 알림
notification-preference-push = 푸시 알림
notification-preference-in-app = 인앱 알림
notification-preference-sound = 소리 활성화됨
notification-preference-vibration = 진동 활성화됨

notification-empty = 알림 없음
notification-empty-description = 여러분 모두 따라잡혔어요!
notification-load-more = 더 로드하기
notification-clear-all = 모든 알림 지우기
notification-filter-all = 모두
notification-filter-unread = 읽지 않음
notification-filter-mentions = 언급
notification-filter-tasks = 작업
notification-filter-messages = 메시지
notification-filter-system = 시스템

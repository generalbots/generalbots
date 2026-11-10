# =============================================================================
# General Bots — ko — Attendant (support queue)
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# =============================================================================
# ATTENDANT / CRM SCREENS
# =============================================================================
# -----------------------------------------------------------------------------
# CRM Disabled State
# -----------------------------------------------------------------------------
attendant-crm-disabled = CRM 기능이 활성화되지 않음
attendant-crm-disabled-desc = Attendant Console에서는 이 봇에 대해 CRM 기능을 활성화해야 합니다. 이를 통해 인간 에이전트는 봇에서 전송된 대화를 수신하고 응답할 수 있습니다.
attendant-crm-enable-instruction = CRM 기능을 활성화하려면 봇의
attendant-crm-config-file = 구성.csv
attendant-crm-create-attendant = 그런 다음
attendant-crm-attendant-file = 수행자.csv
attendant-crm-configure-team = 팀 구성을 위한 파일

# -----------------------------------------------------------------------------
# Queue Sidebar
# -----------------------------------------------------------------------------
attendant-title = 교환원 콘솔
attendant-status-online = 온라인
attendant-status-busy = 바쁨
attendant-status-away = 떨어져
attendant-status-offline = 오프라인
attendant-status-ready = 온라인 - 대화 준비 완료
attendant-status-busy-msg = 다른 용무 중 - 대화 처리 중
attendant-status-away-msg = 어웨이 - 곧 돌아올 예정
attendant-status-offline-msg = 오프라인 - 사용할 수 없음

# -----------------------------------------------------------------------------
# Queue Stats
# -----------------------------------------------------------------------------
attendant-waiting = 대기 중
attendant-active = 활성
attendant-resolved = 해결됨
attendant-mine = 광산

# -----------------------------------------------------------------------------
# Queue Filters
# -----------------------------------------------------------------------------
attendant-filter-all = 모두
attendant-filter-waiting = 대기 중
attendant-filter-mine = 광산
attendant-filter-priority = 우선순위

# -----------------------------------------------------------------------------
# Conversation List
# -----------------------------------------------------------------------------
attendant-no-conversations = 대기열에 대화가 없습니다.
attendant-new-conversations-appear = 여기에 새 대화가 표시됩니다.
attendant-unread = 읽지 않음
attendant-typing = 타이핑 중...
attendant-select-conversation = 대화를 선택하세요
attendant-select-conversation-desc = 응답을 시작하려면 대기열에서 대화를 선택하세요.

# -----------------------------------------------------------------------------
# Channel Tags
# -----------------------------------------------------------------------------
attendant-channel-whatsapp = 왓츠앱
attendant-channel-teams = 팀
attendant-channel-instagram = 인스타그램
attendant-channel-web = 웹
attendant-channel-telegram = 텔레그램
attendant-channel-email = 이메일

# -----------------------------------------------------------------------------
# Priority Tags
# -----------------------------------------------------------------------------
attendant-priority-urgent = 긴급
attendant-priority-high = 높음
attendant-priority-normal = 보통

# -----------------------------------------------------------------------------
# Chat Area
# -----------------------------------------------------------------------------
attendant-message-placeholder = 메시지를 입력하세요...
attendant-send = 보내기
attendant-attach-file = 파일첨부
attendant-insert-emoji = 이모티콘 삽입
attendant-quick-responses = 빠른 응답
attendant-transfer = 환승
attendant-resolve = 해결
attendant-more-actions = 추가 작업

# -----------------------------------------------------------------------------
# Quick Responses
# -----------------------------------------------------------------------------
attendant-quick-greeting = 안녕하세요! 오늘은 무엇을 도와드릴까요?
attendant-quick-thanks = 양해해 주셔서 감사합니다.
attendant-quick-checking = 제가 확인해 보겠습니다.
attendant-quick-moment = 잠시만 기다려주세요.

# -----------------------------------------------------------------------------
# Transfer Modal
# -----------------------------------------------------------------------------
attendant-transfer-title = 대화 전송
attendant-transfer-to = 다음으로 환승
attendant-transfer-reason = 이유(선택사항)
attendant-transfer-reason-placeholder = 이 대화를 전달하는 이유는 무엇인가요?
attendant-transfer-cancel = 취소
attendant-transfer-confirm = 환승

# -----------------------------------------------------------------------------
# AI Insights Sidebar
# -----------------------------------------------------------------------------
attendant-ai-insights = AI 인사이트
attendant-ai-summary = 대화 요약
attendant-ai-sentiment = 고객 감정
attendant-sentiment-positive = 긍정적
attendant-sentiment-neutral = 중립
attendant-sentiment-negative = 네거티브
attendant-smart-replies = 스마트 답장
attendant-confidence = 자신감
attendant-source = 소스

# -----------------------------------------------------------------------------
# Customer Details
# -----------------------------------------------------------------------------
attendant-customer-details = 고객 세부정보
attendant-customer-name = 이름
attendant-customer-email = 이메일
attendant-customer-phone = 전화
attendant-customer-location = 위치
attendant-customer-tags = 태그

# -----------------------------------------------------------------------------
# Conversation History
# -----------------------------------------------------------------------------
attendant-history = 역사
attendant-history-resolved = 해결됨
attendant-history-transferred = 전송됨
attendant-history-abandoned = 버려진
attendant-view-history = 전체 기록 보기

# -----------------------------------------------------------------------------
# Toast Messages
# -----------------------------------------------------------------------------
attendant-toast-transferred = 대화가 성공적으로 전송되었습니다.
attendant-toast-resolved = 해결된 것으로 표시된 대화
attendant-toast-assigned = 나에게 할당된 대화
attendant-toast-error = 오류가 발생했습니다
attendant-toast-connection-lost = 연결이 끊겼습니다. 다시 연결하는 중...
attendant-toast-connection-restored = 연결이 복원되었습니다.

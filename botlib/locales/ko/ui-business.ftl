# =============================================================================
# General Bots — ko — Billing, products and tickets
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Analytics (additional UI keys)
# -----------------------------------------------------------------------------
analytics-dashboard-title = 분석 대시보드
analytics-last-hour = 지난 시간
analytics-last-6h = 지난 6시간
analytics-last-24h = 지난 24시간
analytics-last-7d = 지난 7일
analytics-last-30d = 지난 30일

# =============================================================================
# Billing
# =============================================================================
# -----------------------------------------------------------------------------
# Billing Navigation & General
# -----------------------------------------------------------------------------
billing-title = 청구
billing-invoices = 송장
billing-payments = 결제
billing-quotes = 인용문
billing-dashboard = 대시보드

# -----------------------------------------------------------------------------
# Billing Entities
# -----------------------------------------------------------------------------
billing-invoice = 송장
billing-invoice-desc = 고객에게 청구
billing-payment = 결제
billing-payment-desc = 결제 완료
billing-quote = 견적
billing-quote-desc = 가격 견적

# -----------------------------------------------------------------------------
# Billing Status
# -----------------------------------------------------------------------------
billing-due-date = 만기일
billing-overdue = 연체
billing-paid = 유료
billing-pending = 보류 중
billing-draft = 초안
billing-sent = 보냄
billing-partial = 부분
billing-cancelled = 취소됨

# -----------------------------------------------------------------------------
# Billing Actions
# -----------------------------------------------------------------------------
billing-new-invoice = 새 송장
billing-new-quote = 새로운 견적
billing-new-payment = 새로운 결제
billing-send-invoice = 송장 보내기
billing-record-payment = 기록 지불
billing-mark-paid = 유료로 표시
billing-void = 공허

# -----------------------------------------------------------------------------
# Billing Fields
# -----------------------------------------------------------------------------
billing-amount = 금액
billing-tax = 세금
billing-subtotal = 소계
billing-total = 합계
billing-discount = 할인
billing-line-items = 광고 항목
billing-add-item = 항목 추가
billing-remove-item = 항목 제거
billing-customer = 고객
billing-issue-date = 발행일
billing-payment-terms = 지불 조건
billing-notes = 메모
billing-invoice-number = 송장 번호
billing-quote-number = 견적 번호

# -----------------------------------------------------------------------------
# Billing Reports
# -----------------------------------------------------------------------------
billing-revenue = 수익
billing-outstanding = 뛰어난
billing-this-month = 이번 달
billing-last-month = 지난달
billing-total-paid = 총 지불액
billing-total-overdue = 총 연체
billing-subtitle = 송장, 결제 및 견적
billing-revenue-month = 이번 달 수익
billing-total-revenue = 총 수익
billing-paid-month = 이번 달에 지급됨

# -----------------------------------------------------------------------------
# Billing Empty States
# -----------------------------------------------------------------------------
billing-no-invoices = 인보이스를 찾을 수 없습니다.
billing-no-payments = 결제를 찾을 수 없습니다.
billing-no-quotes = 견적을 찾을 수 없습니다.

# =============================================================================
# Products
# =============================================================================
# -----------------------------------------------------------------------------
# Products Navigation & General
# -----------------------------------------------------------------------------
products-title = 제품
products-catalog = 카탈로그
products-services = 서비스
products-price-lists = 가격표
products-inventory = 인벤토리

# -----------------------------------------------------------------------------
# Products Entities
# -----------------------------------------------------------------------------
products-product = 제품
products-product-desc = 실제 또는 디지털 제품
products-service = 서비스
products-service-desc = 서비스 제공
products-price-list = 가격표
products-price-list-desc = 가격 책정 계층

# -----------------------------------------------------------------------------
# Products Actions
# -----------------------------------------------------------------------------
products-new-product = 신제품
products-new-service = 새로운 서비스
products-new-price-list = 새로운 가격표
products-new-pricelist = 새로운 가격표
products-edit-product = 제품 편집
products-duplicate = 중복

# -----------------------------------------------------------------------------
# Products Fields
# -----------------------------------------------------------------------------
products-sku = SKU
products-category = 카테고리
products-price = 가격
products-unit = 단위
products-stock = 주식
products-cost = 비용
products-margin = 여백
products-barcode = 바코드

# -----------------------------------------------------------------------------
# Products Status
# -----------------------------------------------------------------------------
products-in-stock = 재고 있음
products-out-of-stock = 품절
products-low-stock = 재고 부족
products-active = 활성
products-inactive = 비활성
products-featured = 추천
products-archived = 보관됨

# -----------------------------------------------------------------------------
# Products Stats & Metrics
# -----------------------------------------------------------------------------
products-subtitle = 제품, 서비스, 가격 관리
products-items = 제품
products-pricelists = 가격표
products-total-products = 총 제품
products-total-services = 종합 서비스

# -----------------------------------------------------------------------------
# Products Empty States
# -----------------------------------------------------------------------------
products-no-products = 제품을 찾을 수 없습니다
products-no-services = 서비스를 찾을 수 없습니다.
products-no-price-lists = 가격표를 찾을 수 없습니다.

# =============================================================================
# Tickets (Support Cases)
# =============================================================================
# -----------------------------------------------------------------------------
# Tickets Navigation & General
# -----------------------------------------------------------------------------
tickets-title = 티켓
tickets-cases = 사례
tickets-open = 열기
tickets-closed = 휴무
tickets-all = 모든 티켓
tickets-my-tickets = 내 티켓

# -----------------------------------------------------------------------------
# Tickets Entities
# -----------------------------------------------------------------------------
tickets-case = 케이스
tickets-case-desc = 지원 티켓
tickets-resolution = 해상도
tickets-resolution-desc = AI가 제안하는 솔루션

# -----------------------------------------------------------------------------
# Tickets Priority
# -----------------------------------------------------------------------------
tickets-priority = 우선순위
tickets-priority-low = 낮음
tickets-priority-medium = 중간
tickets-priority-high = 높음
tickets-priority-urgent = 긴급

# -----------------------------------------------------------------------------
# Tickets Status
# -----------------------------------------------------------------------------
tickets-status = 상태
tickets-status-new = 새로운
tickets-status-open = 열기
tickets-status-pending = 보류 중
tickets-status-resolved = 해결됨
tickets-status-closed = 휴무
tickets-status-on-hold = 보류 중

# -----------------------------------------------------------------------------
# Tickets Actions
# -----------------------------------------------------------------------------
tickets-new-ticket = 신규 티켓
tickets-assign = 할당
tickets-reassign = 재할당
tickets-escalate = 에스컬레이션
tickets-resolve = 해결
tickets-reopen = 재개설
tickets-close = 닫기
tickets-merge = 병합

# -----------------------------------------------------------------------------
# Tickets Fields
# -----------------------------------------------------------------------------
tickets-subject = 주제
tickets-description = 설명
tickets-category = 카테고리
tickets-assigned = 할당 대상
tickets-unassigned = 할당되지 않음
tickets-created = 생성됨
tickets-updated = 업데이트됨
tickets-response-time = 응답 시간
tickets-resolution-time = 해결 시간
tickets-customer = 고객
tickets-internal-notes = 내부 메모
tickets-attachments = 첨부파일

# -----------------------------------------------------------------------------
# Tickets AI Features
# -----------------------------------------------------------------------------
tickets-ai-suggestion = AI 제안
tickets-apply-suggestion = 제안 적용
tickets-ai-summary = AI 요약
tickets-similar-tickets = 유사한 티켓
tickets-suggested-articles = 추천 기사

# -----------------------------------------------------------------------------
# Tickets Empty States
# -----------------------------------------------------------------------------
tickets-no-tickets = 티켓을 찾을 수 없습니다
tickets-no-open = 오픈 티켓 없음
tickets-no-closed = 마감된 티켓 없음

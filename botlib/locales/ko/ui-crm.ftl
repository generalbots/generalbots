# =============================================================================
# General Bots — ko — CRM
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# =============================================================================
# CRM
# =============================================================================
# -----------------------------------------------------------------------------
# CRM Navigation & General
# -----------------------------------------------------------------------------
crm-title = CRM
crm-pipeline = 파이프라인
crm-leads = 리드
crm-opportunities = 기회
crm-accounts = 계정
crm-contacts = 연락처
crm-activities = 활동

# -----------------------------------------------------------------------------
# CRM Entities
# -----------------------------------------------------------------------------
crm-lead = 리드
crm-lead-desc = 자격이 없는 잠재고객
crm-opportunity = 기회
crm-opportunity-desc = 적격 판매 기회
crm-account = 계정
crm-account-desc = 회사 또는 조직
crm-contact = 연락처
crm-contact-desc = 계좌에 있는 사람
crm-activity = 활동
crm-activity-desc = 작업, 전화 또는 이메일

# -----------------------------------------------------------------------------
# CRM Actions
# -----------------------------------------------------------------------------
crm-qualify = 자격
crm-convert = 변환
crm-won = 원
crm-lost = 분실
crm-new-lead = 새로운 리드
crm-new-opportunity = 새로운 기회
crm-new-account = 새 계정
crm-new-contact = 새 연락처

# -----------------------------------------------------------------------------
# CRM Fields
# -----------------------------------------------------------------------------
crm-stage = 무대
crm-value = 가치
crm-probability = 확률
crm-close-date = 마감일
crm-company = 회사
crm-phone = 전화
crm-email = 이메일
crm-source = 소스
crm-owner = 소유자

# -----------------------------------------------------------------------------
# CRM Pipeline Stages
# -----------------------------------------------------------------------------
crm-pipeline-new = 새로운
crm-pipeline-contacted = 연락함
crm-pipeline-qualified = 자격을 갖춘
crm-pipeline-proposal = 제안
crm-pipeline-negotiation = 협상
crm-pipeline-closed-won = 마감 수주
crm-pipeline-closed-lost = 마감 분실

# -----------------------------------------------------------------------------
# CRM Stats & Metrics
# -----------------------------------------------------------------------------
crm-subtitle = 리드, 기회, 고객 관리
crm-stage-lead = 리드
crm-stage-qualified = 자격을 갖춘
crm-stage-proposal = 제안
crm-stage-negotiation = 협상
crm-stage-won = 원
crm-stage-lost = 분실
crm-conversion-rate = 전환율
crm-pipeline-value = 파이프라인 가치
crm-avg-deal = 평균 거래 규모
crm-won-month = 이번 달 승리

# -----------------------------------------------------------------------------
# CRM Empty States
# -----------------------------------------------------------------------------
crm-no-leads = 리드를 찾을 수 없습니다
crm-no-opportunities = 추천이 없습니다.
crm-no-accounts = 계정을 찾을 수 없습니다
crm-no-contacts = 연락처를 찾을 수 없습니다.
crm-drag-hint = 카드를 드래그하여 스테이지 변경

# CRM / Mail / Campaigns integration keys
crm-compose-email = 이메일 작성
crm-send-email = 이메일 보내기
mail-snooze = 스누즈
mail-snooze-later-today = 오늘 늦게(오후 6시)
mail-snooze-tomorrow = 내일 (오전 8시)
mail-snooze-next-week = 다음 주 (월 오전 8시)
mail-crm-log = CRM에 로그인
mail-crm-create-lead = 리드 생성
mail-add-to-list = 목록에 추가
campaign-send-email = 이메일 보내기
crm-add-lead = Add Lead
crm-audit-title = Audit trail
crm-campaigns = Campaigns
crm-col-account = Account
crm-col-actions = Actions
crm-col-city = City
crm-col-close = Expected Close
crm-col-close-date = Expected Close
crm-col-contact = Contact
crm-col-contacts = Deals
crm-col-customer = Customer
crm-col-delivery = Delivery
crm-col-email = Email
crm-col-industry = Industry
crm-col-items = Items
crm-col-name = Name
crm-col-number = Number
crm-col-owner = Owner
crm-col-phone = Phone
crm-col-probability = Probability
crm-col-revenue = Annual Revenue
crm-col-source = Source
crm-col-stage = Stage
crm-col-status = Status
crm-col-title = Title
crm-col-total = Total
crm-col-valid-until = Valid until
crm-col-value = Value
crm-create-lead = Create Contact
crm-currency = Currency
crm-deals = Deals
crm-estimated-value = Estimated Value
crm-filter-all = All
crm-filter-lost = Lost
crm-filter-negotiation = Negotiation
crm-filter-new = New
crm-filter-proposal = Proposal
crm-filter-qualified = Qualified
crm-filter-won = Won
crm-first-name = First Name
crm-job-title = Job Title
crm-last-name = Last Name
crm-loading = Loading…
crm-new = New
crm-new-deal = New Deal
crm-notes = Notes
crm-orders = Orders
crm-quotes = Quotes & Orders
crm-search-placeholder-deals = Search deals, accounts...
crm-source-cold-call = Cold Call
crm-source-email = Email Campaign
crm-source-event = Event/Conference
crm-source-linkedin = LinkedIn
crm-source-other = Other
crm-source-partner = Partner
crm-source-referral = Referral
crm-source-website = Website
crm-this-month = Won This Month
crm-total-value = Total Pipeline Value

# semantic i18n key (#2542)
crm-filter-opportunities-placeholder = 기회 필터…
crm-filter-deals-placeholder = 거래 필터…
crm-filter-accounts-placeholder = 계정 필터…
crm-filter-contacts-placeholder = 연락처 필터…
crm-filter-campaigns-placeholder = 캠페인 필터…

# semantic i18n key (#2542)
crm-placeholder-email = hong@example.co.kr
crm-placeholder-deal-lost-reason = 거래가 실패한 시점/사유
crm-placeholder-first-name = 길동
crm-placeholder-last-name = 홍
crm-placeholder-phone = +82 10-1234-5678
crm-placeholder-company-name = 회사명
crm-placeholder-job-title = CEO, 관리자 등
crm-placeholder-amount = 0.00
crm-placeholder-lead-lost-reason = 이 리드가 중단된 이유
crm-placeholder-lead-notes = 이 리드에 대한 추가 정보...
crm-placeholder-website = https://

# semantic i18n key (#2542)
crm-placeholder-search-contacts = 연락처 검색...

# -----------------------------------------------------------------------------
# #2328 — loading / empty / error state copy
# -----------------------------------------------------------------------------
crm-opps-empty-title = 아직 영업 기회가 없습니다
crm-opps-empty-desc = 영업 기회는 가능성 있는 판매를 첫 접촉부터 성사까지 추적합니다.
crm-opps-empty-action = 영업 기회 만들기
crm-deals-empty-title = 아직 거래가 없습니다
crm-deals-empty-desc = 모든 단계의 거래가 금액 및 담당자와 함께 여기에 표시됩니다.
crm-deals-empty-action = 거래 추가
crm-accounts-empty-title = 아직 계정이 없습니다
crm-accounts-empty-desc = 계정은 판매 대상 기업입니다. 연락처와 거래가 그 아래에 묶입니다.
crm-accounts-empty-action = 계정 추가
crm-contacts-empty-title = 아직 연락처가 없습니다
crm-contacts-empty-desc = 대화하는 사람이 여기에 있습니다. 이름, 이메일, 전화번호, 소속 계정입니다.
crm-contacts-empty-action = 연락처 추가
crm-campaigns-empty-title = 아직 캠페인이 없습니다
crm-campaigns-empty-desc = 캠페인은 잠재 고객을 파이프라인으로 가져오는 활동을 묶습니다.
crm-campaigns-empty-action = 캠페인 만들기

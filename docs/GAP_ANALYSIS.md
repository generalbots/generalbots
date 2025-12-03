# Documentation vs Source Code Gap Analysis

> Generated analysis comparing `botserver/src/` with `botserver/docs/`

## Summary

| Category | Documented | Implemented | Gap |
|----------|------------|-------------|-----|
| BASIC Keywords | ~65 | ~80+ | ~15 undocumented |
| Source Modules | 18 | 24 | 6 undocumented |
| Suite Apps | 14 | 14 | ✅ Complete |
| REST APIs | 22 | 22 | ✅ Complete |

---

## 1. Undocumented BASIC Keywords

The following keywords exist in `src/basic/keywords/` but lack dedicated documentation pages:

### High Priority (Commonly Used)

| Keyword | Source File | Description |
|---------|-------------|-------------|
| `QR CODE` | `qrcode.rs` | Generates QR code images from data |
| `SEND SMS` | `sms.rs` | Sends SMS messages via Twilio/AWS SNS/Vonage |
| `PLAY` | `play.rs` | Opens content projector for videos, images, docs |
| `REMEMBER` | `remember.rs` | Stores user memories with expiration |
| `BOOK` | `book.rs` | Schedules calendar meetings/appointments |
| `WEATHER` | `weather.rs` | Gets weather data (API documented, keyword not) |

### Medium Priority (Advanced Features)

| Keyword | Source File | Description |
|---------|-------------|-------------|
| `A2A` | `a2a_protocol.rs` | Agent-to-Agent communication protocol |
| `ADD BOT` | `add_bot.rs` | Dynamically adds bots to session |
| `ADD MEMBER` | `add_member.rs` | Adds members to groups/teams |
| `ADD SUGGESTION` | `add_suggestion.rs` | Adds response suggestions |
| `HUMAN APPROVAL` | `human_approval.rs` | Human-in-the-loop workflow |
| `MODEL ROUTE` | `model_routing.rs` | Routes requests to different LLM models |
| `SEND TEMPLATE` | `send_template.rs` | Sends WhatsApp/channel templates |
| `SET USER` | `set_user.rs` | Sets current user context |

### Low Priority (Internal/Advanced)

| Keyword | Source File | Description |
|---------|-------------|-------------|
| `EPISODIC MEMORY` | `episodic_memory.rs` | Long-term episodic memory storage |
| `KNOWLEDGE GRAPH` | `knowledge_graph.rs` | Knowledge graph operations |
| `LLM` | `llm_keyword.rs` | Direct LLM invocation |
| `MULTIMODAL` | `multimodal.rs` | Image/audio processing |
| `PROCEDURE` | `procedures.rs` | BASIC procedure definitions |
| `ON FORM SUBMIT` | `on_form_submit.rs` | Form submission handlers |
| `IMPORT/EXPORT` | `import_export.rs` | Data import/export operations |

---

## 2. Undocumented Source Modules

### Modules Without Dedicated Documentation

| Module | Path | Purpose | Priority |
|--------|------|---------|----------|
| `attendance` | `src/attendance/` | Queue management for human attendants | Medium |
| `timeseries` | `src/timeseries/` | InfluxDB 3 integration for metrics | Medium |
| `weba` | `src/weba/` | Placeholder for web app features | Low |
| `nvidia` | `src/nvidia/` | GPU acceleration (partially documented) | Low |
| `multimodal` | `src/multimodal/` | Image/video processing | Medium |
| `console` | `src/console/` | Admin console backend | Low |

### Modules With Partial Documentation

| Module | Missing Docs |
|--------|--------------|
| `llm` | LLM keyword syntax, model routing details |
| `calendar` | CalDAV integration details, recurrence rules |
| `meet` | WebRTC/LiveKit integration details |

---

## 3. Documentation Accuracy Issues

### Incorrect or Outdated References

1. **keyword-remember.md** - Referenced but file doesn't exist in SUMMARY.md
2. **keyword-book.md** - Referenced in keyword-create-task.md but no file exists
3. **keyword-weather.md** - API documented but keyword syntax not documented

### Missing from SUMMARY.md

These keyword files exist but aren't linked in SUMMARY.md:

- `keyword-synchronize.md`
- `keyword-reference-complete.md`
- Several template files

---

## 4. API Endpoint Gaps

### Suite App Backend APIs (Recently Implemented)

| App | Endpoints | Status |
|-----|-----------|--------|
| Analytics | 12 endpoints | ✅ Implemented |
| Paper | 20+ endpoints | ✅ Implemented |
| Research | 8 endpoints | ✅ Implemented |
| Sources | 7 endpoints | ✅ Implemented |
| Designer | 5 endpoints | ✅ Implemented |

### Undocumented Internal APIs

| API | Path | Purpose |
|-----|------|---------|
| Queue API | `/api/queue/*` | Human attendant queue management |
| TimeSeries API | N/A | Metrics ingestion (internal only) |

---

## 5. Recommended Documentation Additions

### Immediate Priority

1. **Create `keyword-qrcode.md`**
   ```basic
   ' Generate QR code
   path = QR CODE "https://example.com"
   SEND FILE path
   
   ' With custom size
   path = QR CODE "data", 512
   ```

2. **Create `keyword-sms.md`**
   ```basic
   ' Send SMS
   SEND SMS "+1234567890", "Hello!"
   
   ' With provider
   SEND SMS phone, message, "twilio"
   ```

3. **Create `keyword-play.md`**
   ```basic
   ' Play video
   PLAY "video.mp4"
   
   ' With options
   PLAY "presentation.pptx" WITH OPTIONS "fullscreen"
   ```

4. **Create `keyword-remember.md`**
   ```basic
   ' Remember with expiration
   REMEMBER "user_preference", "dark_mode", "30 days"
   
   ' Recall later
   pref = RECALL "user_preference"
   ```

5. **Create `keyword-book.md`**
   ```basic
   ' Book a meeting
   BOOK "Team Standup" WITH user1, user2 AT "2025-01-20 10:00" FOR 30
   ```

### Medium Priority

1. **Document TimeSeries module** - Add to appendix or chapter-11
2. **Document Attendance/Queue system** - Add to chapter-10 APIs
3. **Expand Multimodal docs** - Add keyword reference
4. **Create A2A Protocol guide** - Multi-agent communication

### Low Priority

1. Add advanced LLM routing documentation
2. Document internal console APIs
3. Add GPU acceleration tuning guide

---

## 6. Consistency Issues

### Naming Conventions

| Issue | Location | Fix |
|-------|----------|-----|
| `keyword-for-each.md` vs `for_next.rs` | Inconsistent naming | Document both FOR EACH and FOR/NEXT |
| `keyword-delete-http.md` vs `DELETE` | Overlap | Clarify HTTP DELETE vs data DELETE |

### Missing Cross-References

- Paper app docs don't reference .gbusers storage (FIXED)
- Calendar docs don't reference BOOK keyword
- Meet docs don't reference video/audio keywords

---

## 7. Action Items

### High Priority
- [ ] Create 5 missing keyword docs (QR CODE, SMS, PLAY, REMEMBER, BOOK)
- [ ] Add WEATHER keyword syntax to weather.md
- [ ] Fix broken references in existing docs

### Medium Priority
- [ ] Document attendance/queue module
- [ ] Add timeseries module to appendix
- [ ] Create A2A protocol guide
- [ ] Add multimodal keyword reference

### Low Priority
- [ ] Document internal console APIs
- [ ] Add advanced configuration examples
- [ ] Create video tutorials references

---

## 8. Verification Commands

```bash
# List all keyword files in src
ls botserver/src/basic/keywords/*.rs | wc -l

# List all keyword docs
ls botserver/docs/src/chapter-06-gbdialog/keyword-*.md | wc -l

# Find references to undocumented keywords
grep -r "QRCODE\|QR CODE\|SEND SMS\|PLAY\|REMEMBER\|BOOK" botserver/docs/

# Check for broken links in SUMMARY.md
grep -oP '\./[^)]+\.md' botserver/docs/src/SUMMARY.md | while read f; do
  [ ! -f "botserver/docs/src/$f" ] && echo "Missing: $f"
done
```

---

*Last updated: 2025-01-20*
*Analyzed modules: 24 source directories, 100+ documentation files*
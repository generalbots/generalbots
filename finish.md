# Remaining Work to Finish Crate Extraction

## Current State
- **botsettings**: 0 errors, 5 warnings (cfg features `rbac`/`mail` not declared)
- **botmeet**: 114 errors
- **botapi**: compiles clean
- **All other previously clean crates**: still 0/0

## 1. Fix botsettings warnings (5 min)
Add features to `botsettings/Cargo.toml`:
```toml
[features]
default = []
rbac = []
mail = ["lettre"]
```

## 2. Add voice session methods to ChannelAdapter trait (5 min)
File: `botlib/src/traits.rs` line 27 — add default methods:
```rust
fn start_voice_session(&self, session_id: &str, user_id: &str) -> botlib::traits::BoxFutureString {
    let _ = (session_id, user_id);
    Box::pin(async { Err("start_voice_session: not implemented".to_string()) })
}
fn stop_voice_session(&self, session_id: &str) -> botlib::traits::BoxFutureUnit {
    let _ = session_id;
    Box::pin(async { Err("stop_voice_session: not implemented".to_string()) })
}
```

## 3. Fix botmeet webinar_api/handlers.rs imports (2 min)
Add missing `get`, `post` imports:
```rust
use axum::routing::{get, post};  // already has axum imported, just missing routing
```

## 4. Fix botmeet recording.rs — webinar_types struct mismatches (30 min)
**The big one.** recording.rs references fields/variants that don't exist in webinar_types.rs.

### 4a. Add missing variants to enums in webinar_types.rs:
- `RecordingQuality`: add `Standard`, `AudioOnly`, `Ultra`
- `RecordingStatus`: add `Recording`, `Ready`
- `TranscriptionFormat`: add `PlainText`

### 4b. Add missing fields to structs in webinar_types.rs:
- `WebinarRecording`: add `file_url`, `download_url`, `download_count`, `view_count`, `file_size_bytes`, `processed_at`, `expires_at`
- `WebinarTranscription`: add `duration_seconds`, `word_count`, `speaker_count`, `confidence_score`, `srt_url`, `vtt_url`, `json_url`
- `TranscriptionSegment`: add `start_time_ms`, `end_time_ms`
- `TranscriptionWord`: add `start_time_ms`, `end_time_ms`

### 4c. Fix botschema import in recording.rs:
`use botschema::meeting_recordings;` — check if `meeting_recordings` module exists in botschema. If not, either add it or remove the import and use a local struct.

### 4d. Fix botcore model imports:
`botcore::shared::models::BotResponse` / `UserMessage` — check if these exist in botcore; if not, add or redirect.

## 5. Fix botmeet diesel query issues (10 min)
In `webinar_api/handlers.rs`:
- `.get_result::<RecordingRow>()` — needs `diesel::RunQueryDsl` trait import
- `.execute(&mut conn)` — needs `diesel::RunQueryDsl` trait import

## 6. cargo check -p botmeet → iterate until 0/0

## 7. cargo check -p botserver → fix re-export breakage → 0/0

## 8. Extract remaining modules (ordered by difficulty):
| Module | Target Crate | Est. | Status |
|--------|-------------|------|--------|
| console | botconsole | 1hr | queued |
| drive | botdrive | 1hr | queued |
| basic/compiler | botbasic_compiler | 1hr | queued |
| basic/keywords | botbasic_keywords | 2hr | queued (31K lines, 77 files) |
| core/ | botcore_modules | 3hr | queued (22K lines, many cross-deps) |
| main_module | botmain | 30min | queued |

## Total remaining: ~8-10hrs of focused work
- botmeet fixes: ~1hr
- botserver re-export verification: ~30min
- 6 remaining extractions: ~7-8hrs

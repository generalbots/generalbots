# Cargo Audit Migration Strategy - Task Breakdown

## Project Context
**Tauri Desktop Application** using GTK3 bindings for Linux support with 1143 total dependencies.

---

## CRITICAL: 1 Vulnerability (Fix Immediately)

### Task 1.1: Fix idna Punycode Vulnerability ⚠️ HIGH PRIORITY
**Issue**: RUSTSEC-2024-0421 - Accepts invalid Punycode labels  
**Status**: ✅ FIXED (Updated validator to 0.20)

### Task 2.1: Replace atty (Used by clap 2.34.0)
**Issue**: RUSTSEC-2024-0375 + RUSTSEC-2021-0145 (unmaintained + unsound)  
**Status**: ✅ FIXED (Replaced `ksni` with `tray-icon`)

### Task 2.2: Replace ansi_term (Used by clap 2.34.0)
**Issue**: RUSTSEC-2021-0139 (unmaintained)  
**Status**: ✅ FIXED (Replaced `ksni` with `tray-icon`)

### Task 2.3: Replace rustls-pemfile
**Issue**: RUSTSEC-2025-0134 (unmaintained)  
**Status**: ✅ FIXED (Updated axum-server to 0.8 and qdrant-client to 1.16)

### Task 2.4: Fix aws-smithy-runtime (Yanked Version)
**Issue**: Version 1.9.6 was yanked  
**Status**: ✅ FIXED (Updated aws-sdk-s3 to 1.120.0)

### Task 2.5: Replace fxhash
**Issue**: RUSTSEC-2025-0057 (unmaintained)  
**Current**: `fxhash 0.2.1`  
**Used by**: `selectors 0.24.0` → `kuchikiki` (speedreader fork) → Tauri
**Status**: ⏳ PENDING (Wait for upstream Tauri update)

### Task 2.6: Replace instant
**Issue**: RUSTSEC-2024-0384 (unmaintained)  
**Status**: ✅ FIXED (Updated rhai)

### Task 2.7: Replace lru (Unsound Iterator)
**Issue**: RUSTSEC-2026-0002 (unsound - violates Stacked Borrows)  
**Status**: ✅ FIXED (Updated ratatui to 0.30 and aws-sdk-s3 to 1.120.0)

---

## MEDIUM PRIORITY: Tauri/GTK Stack (Major Effort)

### Task 3.1: Evaluate GTK3 → Tauri Pure Approach
**Issue**: All GTK3 crates unmaintained (12 crates total)  
**Current**: Using Tauri with GTK3 Linux backend

**Strategic Question**: Do you actually need GTK3?

**Investigation Items**:
- [ ] Audit what GTK3 features you're using:
  - System tray? (ksni 0.2.2 uses it)
  - Native file dialogs? (rfd 0.15.4)
  - Native menus? (muda 0.17.1)
  - WebView? (wry uses webkit2gtk)
- [ ] Check if Tauri v2 can work without GTK3 on Linux
- [ ] Test if removing `ksni` and using Tauri's built-in tray works

**Decision Point**: 
- **If GTK3 is only for tray/dialogs**: Migrate to pure Tauri approach
- **If GTK3 is deeply integrated**: Plan GTK4 migration

**Estimated effort**: 4-8 hours investigation

---

### Task 3.2: Option A - Migrate to Tauri Pure (Recommended)
**If Task 3.1 shows GTK3 isn't essential**

**Action Items**:
- [ ] Replace `ksni` with Tauri's `tauri-plugin-tray` or `tray-icon`
- [ ] Remove direct GTK dependencies from Cargo.toml
- [ ] Update Tauri config to use modern Linux backend
- [ ] Test on: Ubuntu 22.04+, Fedora, Arch
- [ ] Verify all system integrations work

**Benefits**:
- Removes 12 unmaintained crates
- Lighter dependency tree
- Better cross-platform consistency

**Estimated effort**: 1-2 days

---

### Task 3.3: Option B - Migrate to GTK4 (If GTK Required)
**If Task 3.1 shows GTK3 is essential**

**Action Items**:
- [ ] Create migration branch
- [ ] Update Cargo.toml GTK dependencies:
  ```toml
  # Remove:
  gtk = "0.18"
  gdk = "0.18"
  
  # Add:
  gtk4 = "0.9"
  gdk4 = "0.9"
  ```
- [ ] Rewrite GTK code following [gtk-rs migration guide](https://gtk-rs.org/gtk4-rs/stable/latest/book/migration/)
- [ ] Key API changes:
  - `gtk::Window` → `gtk4::Window`
  - Event handling completely redesigned
  - Widget hierarchy changes
  - CSS theming changes
- [ ] Test thoroughly on all Linux distros

**Estimated effort**: 1-2 weeks (significant API changes)

---

## LOW PRIORITY: Transitive Dependencies

### Task 4.1: Replace proc-macro-error
**Issue**: RUSTSEC-2024-0370 (unmaintained)  
**Current**: `proc-macro-error 1.0.4`  
**Used by**: `validator_derive` and `gtk3-macros` and `glib-macros`

**Action Items**:
- [ ] Update `validator` crate (may have migrated to `proc-macro-error2`)
- [ ] GTK macros will be fixed by Task 3.2 or 3.3
- [ ] Run `cargo update -p validator`

**Estimated effort**: 30 minutes (bundled with Task 1.1)

---

### Task 4.2: Replace paste
**Issue**: RUSTSEC-2024-0436 (unmaintained, no vulnerabilities)  
**Current**: `paste 1.0.15`  
**Used by**: `tikv-jemalloc-ctl`, `rav1e`, `ratatui`

**Action Items**:
- [ ] Low priority - no security issues
- [ ] Will likely be fixed by updating parent crates
- [ ] Monitor for updates when updating other deps

**Estimated effort**: Passive (wait for upstream)

---

### Task 4.3: Replace UNIC crates
**Issue**: All unmaintained (5 crates)  
**Current**: Used by `urlpattern 0.3.0` → `tauri-utils`

**Action Items**:
- [ ] Update Tauri to latest version
- [ ] Check if Tauri has migrated to `unicode-*` crates
- [ ] Run `cargo update -p tauri -p tauri-utils`

**Estimated effort**: 30 minutes (bundled with Tauri updates)

---

### Task 4.4: Fix glib Unsoundness
**Issue**: RUSTSEC-2024-0429 (unsound iterator)
**Current**: `glib 0.18.5` (part of GTK3 stack)
**Status**: 🛑 Transitive / Accepted Risk (Requires GTK4 migration)

**Action Items**:
- [ ] Document as accepted transitive risk until Tauri migrates to GTK4

**Estimated effort**: N/A (Waiting for upstream)

---

## Recommended Migration Order

### Phase 1: Critical Fixes (Week 1)
1. ✅ Task 1.1 - Fix idna vulnerability
2. ✅ Task 2.4 - Fix AWS yanked version
3. ✅ Task 2.3 - Update rustls-pemfile
4. ✅ Task 2.6 - Update instant/rhai
5. ✅ Task 2.7 - Update lru

**Result**: No vulnerabilities, no yanked crates

---

### Phase 2: Direct Dependency Cleanup (Week 2)
6. ✅ Task 3.1 - Evaluate GTK3 usage (Determined ksni was main usage, replaced)
7. ✅ Task 2.1/2.2 - Fix atty/ansi_term via clap (Removed ksni)
8. ⏳ Task 2.5 - Fix fxhash (Waiting for upstream Tauri update, currently on v2)

**Result**: All direct unmaintained crates addressed

---

### Phase 3: GTK Migration (Weeks 3-4)
9. 🛑 Task 3.1/3.2/3.3 - GTK Migration halted.
   - **Reason**: GTK3 is a hard dependency of Tauri on Linux (via `wry` -> `webkit2gtk`).
   - **Decision**: Accept the ~11-12 transitive GTK3 warnings as they are unavoidable without changing frameworks.
   - **Action**: Suppress warnings if possible, otherwise document as known transitive issues.

10. ✅ Task 4.1 - Update validator/proc-macro-error (Verified validator 0.20)
11. ✅ Task 4.3 - Update UNIC crates via Tauri (Verified Tauri v2)

**Result**: All actionable warnings addressed. GTK3 warnings acknowledged as transitive/upstream.

---

## Testing Checklist

After each phase, verify:

- [ ] `cargo audit` shows 0 vulnerabilities, 0 actionable warnings (GTK3 warnings accepted)
- [ ] `cargo build --release` succeeds
- [ ] `cargo test` passes
- [ ] Manual testing:
  - [ ] botapp launches and renders correctly
  - [ ] System tray works (Linux)
  - [ ] File dialogs work
  - [ ] Web view renders content
  - [ ] HTTP/gRPC endpoints respond (botserver)
  - [ ] S3 operations work (botserver)
  - [ ] Database connections work
  - [ ] Scripting engine works (botserver)

---

## Quick Commands Reference

```bash
# Phase 1 - Critical fixes
cargo update -p validator          # Task 1.1
cargo update -p aws-config -p aws-sdk-s3 -p aws-sdk-sts  # Task 2.4
cargo update -p tonic -p axum-server  # Task 2.3
cargo update -p rhai                # Task 2.6
cargo update -p ratatui -p aws-sdk-s3  # Task 2.7

# Phase 2 - Direct deps
cargo update -p dbus-codegen        # Task 2.1 (if possible)
cargo update -p tauri -p wry        # Task 2.5

# Verify after each update
cargo audit
cargo build --release
cargo test
```

---

## Risk Assessment

| Task | Risk Level | Breaking Changes | Rollback Difficulty |
|------|-----------|------------------|---------------------|
| 1.1 idna | Low | None expected | Easy |
| 2.1 atty/clap | Medium | Possible CLI changes | Medium |
| 2.3 rustls | Low | Internal only | Easy |
| 2.4 AWS | Low | None expected | Easy |
| 2.5 fxhash | Medium | Depends on upstream | Hard (may need fork) |
| 3.2 Tauri Pure | Medium | API changes | Medium |
| 3.3 GTK4 | **High** | **Major API rewrite** | **Hard** |

---

## Estimated Total Effort

- **Phase 1 (Critical)**: 2-4 hours
- **Phase 2 (Cleanup)**: 4-8 hours
- **Phase 3 Option A (Tauri Pure)**: 1-2 days
- **Phase 3 Option B (GTK4)**: 1-2 weeks

**Recommended**: Start Phase 1 immediately, then do Task 3.1 investigation before committing to Option A or B.

---

## Success Criteria

✅ **Complete when**:
- `cargo audit` returns: `Success! 0 vulnerabilities found` (ignoring transitive GTK warnings)
- All direct dependencies are maintained and secure
- All automated tests pass
- Manual testing confirms no regressions
- Application runs on target Linux distributions

---

## Notes

- Most issues are **transitive dependencies** - updating direct deps often fixes them
- **GTK3 → GTK4** is the biggest effort but solves 12 warnings at once
- Consider **Tauri Pure** approach to avoid GUI framework entirely
- Some fixes (like fxhash) may require upstream updates - don't block on them
- Document any temporary workarounds for future reference
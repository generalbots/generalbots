# Editorial checklist

- [ ] Front matter complete; `author` and a different `reviewer` are named.
- [ ] CVSS vector and score agree (`python3 security/cvss_check.py <file>`).
- [ ] Affected/fixed version ranges match a released version in `Cargo.toml`.
- [ ] Severity thresholds: Critical ≥ 9.0, High 7.0–8.9, Medium 4.0–6.9, Low < 4.0.
- [ ] Embargo: 90 days from acknowledgement; extensions need a written date and a timeline entry.
- [ ] Early notice to government customers happens before the public release when the advisory is Critical/High.
- [ ] No personal data, internal addresses or credentials in the advisory.
- [ ] CSAF, OSV, CVE JSON and the HTML page are generated from the same front matter.
- [ ] Two-person review recorded before publication; drafts never auto-publish.

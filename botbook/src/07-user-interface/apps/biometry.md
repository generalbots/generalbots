# Biometry 🟡 PREVIEW

> **Preview application.** Usable today, not yet part of the supported surface.

Biometry collects identity verification into one console: document checks, liveness, signatures and the certificates that depend on them.

## What it does

| Tab | Purpose |
|---|---|
| **KYC** | Identity verification records and their state |
| **Liveness** | Active liveness sessions — proving a live person is present, not a photograph |
| **Signatures** | Documents awaiting signature, with a signature pad for capturing one |
| **Certificates** | Certificates issued from verified identities |
| **Audit** | The trail of what was checked, when |

## How it relates to the rest of the suite

| App | Concern |
|---|---|
| **Biometry** | Who the person is, and evidence of it |
| [KYC](./kyc.md) | The verification workflow itself |
| [Compliance](./compliance.md) | Controls, evidence and audit |
| [Signatures](./kyc.md) | Document signing |

## Opening it

Biometry is a **preview** application. Turn on the **Preview** switch in the left sidebar, then open **Biometry** from the app menu.

## Limits

- Liveness and document checks require the corresponding backend services to be configured; the tabs report what is available rather than hiding unavailable ones.
- Biometric data is sensitive. Treat the audit trail as a regulated record and check your retention obligations before enabling it.

## See Also

- [KYC](./kyc.md) - Know-your-customer workflows
- [Anti-Fraud](./fraud.md) - Detection and governance
- [Compliance](./compliance.md) - Controls and evidence
- [Apps overview](./README.md) - Stability classification for the whole suite

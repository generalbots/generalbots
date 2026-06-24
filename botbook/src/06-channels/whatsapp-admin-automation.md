# WhatsApp Admin Automation 🟡 BETA

> **Automated WhatsApp Business Account provisioning and management**

## Automation Ceiling

WhatsApp Business API provisioning has one unavoidable human step — the initial OTP SMS/voice verification. Everything else is fully automatable.

| Step | Automatable | Method |
|------|-------------|--------|
| Meta Business Manager creation | ❌ No | Requires human identity verification |
| Initial Meta account | ❌ No | ToS requires a human actor |
| OTP (SMS/voice) verification | ⚠️ Once | Can be internalized with SIM infrastructure |
| WABA creation (BSP path) | ✅ Yes | Partner-initiated endpoint |
| Phone number addition | ✅ Yes | `add_phone_numbers` endpoint |
| 2SV PIN setup | ✅ Yes | Two-Step Verification API |
| Final registration | ✅ Yes | Register API |
| Webhook configuration | ✅ Yes | Graph API subscription endpoints |
| Message template creation | ✅ Yes | Template Management API |
| System user token | ✅ Yes (one-time) | Long-lived, non-expiring tokens |
| Credit line assignment | ✅ Yes | Credit Line API (BSPs) |

## Implementation Strategy

### BSP Path (maximum automation)

1. Obtain BSP approval from Meta
2. Build a pre-verified number pool
3. Use Partner-Initiated WABA Creation API
4. Automate: WABA → number → register → webhook → template

### Standard Business Path

1. Integrate Embedded Signup flow into onboarding UI
2. Build server-side token exchange endpoint
3. Wire post-signup automation hooks (webhooks, templates)

## API Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/{api-version}/{waba-id}/phone_numbers` | POST | Add phone number |
| `/{api-version}/{phone-number-id}/register` | POST | Register number |
| `/{api-version}/{phone-number-id}/two_step_verification` | POST | Set 2SV PIN |
| `/{api-version}/{app-id}/subscriptions` | POST | Configure webhooks |
| `/{api-version}/{waba-id}/message_templates` | POST | Create templates |
| `/{api-version}/{waba-id}/credit_lines` | POST | Assign credit |

## Key Concepts

### System Users

System Users generate permanent, non-expiring access tokens essential for production automation. Standard user tokens expire and require refresh.

### BSP (Business Solution Provider)

BSPs can achieve near-zero human interaction by combining Partner-Initiated WABA Creation with a pre-verified number pool. The OTP is the only truly unavoidable step.

### Webhooks

Webhooks replace polling and enable reactive, event-driven architecture for message status updates, incoming messages, and template quality reviews.

## Configuration

In `config.csv`:
```csv
whatsapp-bsp-mode,true
whatsapp-system-user-id,<id>
whatsapp-system-user-token,<permanent-token>
whatsapp-preverified-pool,./pools/numbers.json
```

## References

- [WhatsApp Quick Start](./whatsapp-quick-start.md)
- [WhatsApp Webhooks](./whatsapp-webhooks.md)
- [WhatsApp Channel Config](../10-configuration-deployment/whatsapp-channel.md)

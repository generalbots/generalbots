# Skills Marketplace

A public catalog of installable skill packages (`.gbskill`: manifest + `.bas`
scripts + prompts + declared permissions).

- Browsing requires no authentication: `GET /api/marketplace/skills`,
  `GET /api/marketplace/skills/{slug}`.
- Publishing requires an administrator or the owning organization.
- Installation copies the package into the target bot's Drive bucket and
  records the install; uninstall removes only the added files.
- Ten starter skills are seeded automatically on first boot.

The storefront page is available anonymously at `/marketplace`; installing
prompts for sign-in and shows the permission consent card before applying.

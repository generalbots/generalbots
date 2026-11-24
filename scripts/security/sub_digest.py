#!/usr/bin/env python3
"""Tracking-free advisory email digest, built from the canonical bundle (#2719).

The message is plain text: no remote images, no open/click beacons, no CSS
fetches and no third-party assets. Recipients only get advisory IDs, titles,
severity, dates and links back to the bulletin, plus a signed one-click
unsubscribe link. Locale strings live here, never in a hand-maintained feed.

Usage: `python3 scripts/security/sub_digest.py --self-test`
"""

from __future__ import annotations

import sys

from advisory_source import advisories_for_product, load_bundle, public_base
from sub_crypto import (
    UNSUBSCRIBE_TTL_DAYS,
    SubscriptionError,
    check_secret,
    make_token,
)
from sub_subscribers import check_locale, list_headers, normalize_email

TEXT = {
    "en": ("Security advisory digest", "Advisories published", "Severity", "Published", "Upgrade", "Unsubscribe"),
    "pt-BR": ("Resumo de avisos de segurança", "Avisos publicados", "Gravidade", "Publicado", "Atualização", "Cancelar assinatura"),
    "es": ("Resumen de avisos de seguridad", "Avisos publicados", "Gravedad", "Publicado", "Actualización", "Cancelar suscripción"),
    "fr": ("Résumé des avis de sécurité", "Avis publiés", "Gravité", "Publié", "Correction", "Se désabonner"),
    "de": ("Sicherheitshinweise-Übersicht", "Veröffentlichte Hinweise", "Schweregrad", "Veröffentlicht", "Behebung", "Abmelden"),
    "ja": ("セキュリティ勧告ダイジェスト", "公開された勧告", "重大度", "公開日", "修正", "購読解除"),
    "ko": ("보안 권고 요약", "게시된 권고", "심각도", "게시일", "수정", "구독 취소"),
    "zh-CN": ("安全公告摘要", "已发布公告", "严重性", "发布日期", "修复", "取消订阅"),
}
MAX_ADVISORIES = 50


def render_digest(bundle: dict, locale: str, unsubscribe_link: str, base: str) -> str:
    """Render a deterministic, pixel-free plain-text digest."""
    check_locale(locale)
    subject, heading, severity_label, published_label, upgrade_label, unsubscribe_label = TEXT[locale]
    records = advisories_for_product(bundle, None)[:MAX_ADVISORIES]
    lines = [subject, "=" * len(subject), "", heading, ""]
    for record in records:
        lines.append(f"- {record['id']}: {record['title']}")
        lines.append(f"  {severity_label}: {record['severity']} · {published_label}: {record['published']}")
        remediation = record.get("remediation") or {}
        if isinstance(remediation, dict) and remediation.get("recommended_version"):
            lines.append(f"  {upgrade_label}: {remediation['recommended_version']}")
        lines.append(f"  {base}/advisories/{record['id']}")
        summary = record.get("summary")
        if summary:
            lines.append(f"  {summary}")
        lines.append("")
    lines.extend([unsubscribe_label, unsubscribe_link, ""])
    return "\n".join(lines)


def digest_message(
    bundle: dict,
    email: str,
    secret: str,
    *,
    locale: str = "en",
    base: str | None = None,
) -> dict[str, str]:
    """Build subject, body and RFC 8058 headers for one active subscriber."""
    check_secret(secret)
    address = normalize_email(email)
    origin = base or public_base()
    token = make_token(secret, "unsubscribe", address, UNSUBSCRIBE_TTL_DAYS)
    link = f"{origin}/unsubscribe?t={token}"
    subject = TEXT[check_locale(locale)][0]
    headers = list_headers(origin, secret, address)
    return {
        "to": address,
        "subject": subject,
        "text": render_digest(bundle, locale, link, origin),
        **headers,
    }


def self_test() -> int:
    secret = "d" * 32
    try:
        bundle = load_bundle()
        for locale in TEXT:
            message = digest_message(bundle, "ops@example.org", secret, locale=locale)
            body = message["text"]
            if "<img" in body.lower() or "tracking" in body.lower() or "<html" in body.lower():
                raise SubscriptionError(f"{locale} digest contains a tracking vector")
            if "unsubscribe?t=" not in body or message["subject"] != TEXT[locale][0]:
                raise SubscriptionError(f"{locale} digest is missing required content")
            for record in bundle["advisories"]:
                if record["id"] not in body:
                    raise SubscriptionError(f"{locale} digest dropped {record['id']}")
        # Removing an advisory from the canonical bundle changes every channel.
        trimmed = {"advisories": bundle["advisories"][:-1]}
        shorter = render_digest(trimmed, "en", "https://example.org/u", public_base())
        full = render_digest(bundle, "en", "https://example.org/u", public_base())
        if shorter == full:
            raise SubscriptionError("digest is not derived from the canonical bundle")
    except (SubscriptionError, ValueError, KeyError) as error:
        print(f"sub_digest self-test FAILED: {error}", file=sys.stderr)
        return 1
    print(f"sub_digest self-test: ok ({len(TEXT)} locales, no tracking)")
    return 0


if __name__ == "__main__":
    raise SystemExit(self_test())

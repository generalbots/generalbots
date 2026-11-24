#!/usr/bin/env python3
"""Offline static audit of the enterprise-sharing / object-ACL surface (#2452).

Reads Rust source only. It never starts a server, contacts the network or drives
a browser, so it cannot produce a fake UAT screenshot. It asserts four
source-level invariants and then prints a documented GAP for the structurally
unreachable `docs_acl_denied` branch. The audit PASSES with the GAP present:
honesty about the untestable branch is required, not a false green.

Usage:
    python3 scripts/check_sharing_acl_audit.py [SOURCE_ROOT]

SOURCE_ROOT defaults to the repository root (parent of scripts/). The override
exists so the audit can be run against a throwaway /tmp copy in non-vacuity
probes.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else Path(__file__).resolve().parents[1]

DOCS_SRC = "botserver/crates/botdocs/src"
DOC_STORAGE = f"{DOCS_SRC}/storage_drive.rs"
DOC_AST = f"{DOCS_SRC}/handlers_api/ast.rs"
ACL_MODEL = f"{DOCS_SRC}/access_control/model.rs"
DECISION = f"{DOCS_SRC}/access_control/decision.rs"
SHARED_LINK = f"{DOCS_SRC}/shared_link.rs"
COLLAB_ROUTES = "botserver/src/main_module/routes/collab_routes"
COLLAB_DTO = f"{COLLAB_ROUTES}/dto.rs"
COLLAB_PERMISSIONS = f"{COLLAB_ROUTES}/permissions.rs"
OFFICE_OWNER = "botserver/src/security/office_owner.rs"


class AuditFailure(Exception):
    """A named, source-backed assertion failure."""

    def __init__(self, assertion: str, detail: str) -> None:
        super().__init__(f"{assertion}: {detail}")
        self.assertion = assertion
        self.detail = detail


def require(condition: bool, assertion: str, detail: str) -> None:
    if not condition:
        raise AuditFailure(assertion, detail)


def source(rel: str) -> str:
    path = ROOT / rel
    try:
        return path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError) as exc:
        raise AuditFailure("source-readable", f"cannot read {rel}: {exc}") from exc


def line_of(text: str, pattern: str) -> int | None:
    for number, line in enumerate(text.splitlines(), 1):
        if re.search(pattern, line):
            return number
    return None


def lines_of(text: str, pattern: str) -> list[int]:
    return [n for n, line in enumerate(text.splitlines(), 1) if re.search(pattern, line)]


def struct_block(text: str, name: str, assertion: str) -> str:
    match = re.search(r"pub struct " + re.escape(name) + r"\s*\{(.*?)\n\}", text, re.S)
    require(match is not None, assertion, f"struct {name} not found")
    return match.group(1)


def check_owner_derivation() -> str:
    """Object reads stamp owner_id from the verified requester, never from JSON."""
    assertion = "object-read-owner-from-verified-requester"
    text = source(DOC_STORAGE)
    stamps = lines_of(text, r"\bowner_id:\s*user_identifier\.to_string\(\),")
    require(
        len(stamps) >= 2,
        assertion,
        f"expected >=2 owner_id assignments from user_identifier, found {stamps}",
    )
    for number in lines_of(text, r"\bowner_id:"):
        line = text.splitlines()[number - 1]
        require(
            re.search(r"\b(?:req|body|payload|request)\.", line) is None,
            assertion,
            f"owner_id taken from a request field at {DOC_STORAGE}:{number}: {line.strip()}",
        )
    require(
        re.search(r"get_user_docs_path\(user_identifier\)", text) is not None,
        assertion,
        "storage namespace root is not derived from user_identifier",
    )
    require(
        re.search(r"pub async fn load_document_from_drive\([^)]*user_identifier: &str", text, re.S)
        is not None,
        assertion,
        "load_document_from_drive signature lost user_identifier",
    )

    callers: list[str] = []
    for path in sorted((ROOT / DOCS_SRC).rglob("*.rs")):
        rel = path.relative_to(ROOT).as_posix()
        content = source(rel)
        for match in re.finditer(r"load_document_from_drive\(", content):
            prefix = content[max(0, match.start() - 40) : match.start()]
            if re.search(r"\bfn\s+$", prefix):
                continue  # the function definition, not a call
            number = content.count("\n", 0, match.start()) + 1
            window = content[match.start() : match.start() + 160]
            argument = re.search(
                r"load_document_from_drive\(\s*&?state\s*,\s*&?([A-Za-z_]\w*)", window
            )
            require(
                argument is not None,
                assertion,
                f"cannot resolve the requester argument at {rel}:{number}",
            )
            name = argument.group(1)
            require(
                name in ("user_id", "owner_id"),
                assertion,
                f"object read at {rel}:{number} passes '{name}' as requester, expected user_id/owner_id",
            )
            require(
                re.search(r"\b(?:req|body|payload)\.(?:owner_id|user_id)\b", window) is None,
                assertion,
                f"request-body identity used as requester at {rel}:{number}",
            )
            if name == "owner_id":
                require(
                    "shared_link.rs" in rel and re.search(r"resolve_owner\(", content) is not None,
                    assertion,
                    f"owner_id at {rel}:{number} is not server-resolved from the link row",
                )
            else:
                has_verified_source = (
                    re.search(r"user\.id\(\)", content) is not None
                    or re.search(r"user_id\s*:\s*&str", content) is not None
                )
                require(
                    has_verified_source,
                    assertion,
                    f"{rel} has no verified user_id source for the read at :{number}",
                )
            callers.append(f"{rel}:{number}")
    require(
        len(callers) >= 10,
        assertion,
        f"expected >=10 object read call sites, found {len(callers)}",
    )
    return (
        f"{len(stamps)} owner_id stamps from user_identifier; {len(callers)} read call sites "
        f"pass a verified id (storage_drive.rs:{stamps[0]},{stamps[1]})"
    )


def check_owner_bypass_first() -> str:
    """The owner allow must be decided before any cross-tenant/role/link deny."""
    assertion = "owner-bypass-precedes-role-deny"
    text = source(DECISION)
    guard = line_of(text, r"principal\.member_id\(\) == Some\(acl\.owner\.as_str\(\)\)")
    allow = line_of(text, r"return Decision::allow\(Reason::Owner\)")
    require(
        guard is not None and allow is not None,
        assertion,
        "owner-bypass guard/allow not found in decide()",
    )
    denies: dict[str, int] = {}
    for label, pattern in (
        ("cross-tenant", r"Reason::CrossTenant"),
        ("no-grant", r"Reason::NoGrant"),
        ("insufficient-role", r"Reason::InsufficientRole"),
        ("no-identity", r"Reason::NoIdentity"),
        ("link-token-mismatch", r"Reason::LinkTokenMismatch"),
    ):
        number = line_of(text, pattern)
        require(number is not None, assertion, f"deny branch {label} not found")
        denies[label] = number
    helper = line_of(text, r"fn decide_link\(")
    require(
        helper is not None and guard < helper,
        assertion,
        "owner bypass is not in the top-level decide()",
    )
    first_deny = min(denies.values())
    require(
        guard < allow < first_deny,
        assertion,
        f"owner bypass at decision.rs:{guard}/{allow} must precede first deny at "
        f":{first_deny} ({denies})",
    )
    rendered = ", ".join(f"{label}={number}" for label, number in denies.items())
    return f"decision.rs:{guard} guard -> :{allow} allow(Owner) before first deny at :{first_deny} ({rendered})"


def check_sharing_json_inputs() -> str:
    """JSON role fields are constrained; no JSON field sets an object owner."""
    assertion = "no-untrusted-role-or-owner-in-sharing-json"
    dto = source(COLLAB_DTO)
    perm = source(COLLAB_PERMISSIONS)
    owner_source = source(OFFICE_OWNER)
    grant = struct_block(dto, "GrantBody", assertion)
    transfer = struct_block(dto, "TransferBody", assertion)
    link = struct_block(dto, "LinkBody", assertion)

    require(
        "owner_id" not in link,
        assertion,
        "LinkBody accepts a user-supplied owner_id",
    )
    require(
        re.search(r"\brole:\s*String", grant) is not None
        and re.search(r"\brole:\s*String", link) is not None,
        assertion,
        "expected role fields in GrantBody/LinkBody not found",
    )
    editable = re.search(r"fn valid_editable_role\(role: &str\) -> bool\s*\{(.*?)\}", dto, re.S)
    require(editable is not None, assertion, "valid_editable_role not found")
    body = editable.group(1)
    require(
        re.search(r'"viewer"\s*\|\s*"commenter"\s*\|\s*"editor"', body) is not None
        and '"owner"' not in body,
        assertion,
        "valid_editable_role must allow only viewer|commenter|editor (never owner)",
    )
    validations = lines_of(perm, r"valid_editable_role\(&req\.role\)")
    require(
        len(validations) >= 2,
        assertion,
        f"expected >=2 handler-side role validations, found {validations}",
    )
    require(
        re.search(r"let owner_id = link_owner_id\(&user, &req\.resource_type\)", perm) is not None,
        assertion,
        "link owner is not derived from the authenticated user",
    )
    require(
        re.search(r"verified_owner_id\(user\)", perm) is not None,
        assertion,
        "link_owner_id does not call the verified owner resolver",
    )
    require(
        re.search(r"user\.user_id\.to_string\(\)", owner_source) is not None,
        assertion,
        "verified_owner_id does not use the authenticated user id",
    )
    for path in sorted((ROOT / COLLAB_ROUTES).rglob("*.rs")):
        rel = path.relative_to(ROOT).as_posix()
        for number in lines_of(source(rel), r"\breq\.owner_id\b"):
            raise AuditFailure(
                assertion, f"request-body owner_id accepted at {rel}:{number}"
            )

    start = perm.find("pub async fn transfer_ownership")
    require(start != -1, assertion, "transfer_ownership handler not found")
    end = perm.find("\npub async fn ", start + 1)
    segment = perm[start : end if end != -1 else len(perm)]
    segment_start_line = perm[:start].count("\n") + 1
    guard = line_of(segment, r"if !owner_or_admin\(")
    insert = line_of(segment, r"INSERT INTO resource_permissions")
    require(
        guard is not None and insert is not None and guard < insert,
        assertion,
        "transfer_ownership must gate new_owner_id with owner_or_admin before the INSERT",
    )

    def loc(text: str, pattern: str) -> int:
        return line_of(text, pattern) or 0

    exceptions = [
        f"EXCEPTION {COLLAB_DTO}:{loc(dto, 'pub struct GrantBody')} GrantBody.role and "
        f"{COLLAB_DTO}:{loc(dto, 'pub struct LinkBody')} LinkBody.role are accepted from JSON "
        f"but restricted to viewer|commenter|editor by {COLLAB_DTO}:"
        f"{loc(dto, 'fn valid_editable_role')} and enforced at "
        f"{COLLAB_PERMISSIONS}:{validations[0]},{validations[1]}",
        f"EXCEPTION {COLLAB_DTO}:{loc(dto, 'pub struct TransferBody')} TransferBody.new_owner_id "
        f"is accepted from JSON but only after the owner-or-admin gate at "
        f"{COLLAB_PERMISSIONS}:{segment_start_line + guard - 1}, before the INSERT at "
        f"{COLLAB_PERMISSIONS}:{segment_start_line + insert - 1}",
        f"EXCEPTION {SHARED_LINK}:{line_of(source(SHARED_LINK), 'resolve_owner\\(') or 0} "
        "anonymous link reads use the owner recorded server-side at link creation "
        "(minted from the verified user), never a request-body field",
    ]
    return (
        "no req.owner_id in collab handlers; link owner derived from verified user; "
        "documented exceptions:\n       " + "\n       ".join(exceptions)
    )


def check_link_credential_order() -> str:
    """A share-link token is checked before any document content is returned."""
    assertion = "link-credential-verified-before-content"
    text = source(SHARED_LINK)
    shape = line_of(text, r"is_valid_token\(&token\)")
    load = line_of(text, r"load_link\(&state, &token\)")
    expiry = line_of(text, r"expires_at < chrono::Utc::now\(\)")
    read = line_of(text, r"load_document_from_drive\(&state, &owner_id")
    content = line_of(text, r"content:\s*document\.content")
    positions = (
        ("token shape check", shape),
        ("link lookup", load),
        ("expiry check", expiry),
        ("document read", read),
        ("content in response", content),
    )
    for label, number in positions:
        require(number is not None, assertion, f"{label} not found in {SHARED_LINK}")
    require(
        shape < load < expiry < read < content,
        assertion,
        f"link credential ordering violated: {positions}",
    )
    require(
        re.search(r"FROM resource_links WHERE token = \$1", text) is not None,
        assertion,
        "load_link does not bind the presented token to the lookup",
    )
    require(
        re.search(r"\.ok_or_else\(not_found\)", text) is not None,
        assertion,
        "an unknown token does not fail closed with 404",
    )
    decision_text = source(DECISION)
    equal = line_of(decision_text, r"constant_time_eq\(&credential\.token, &link\.token\)")
    grant = line_of(decision_text, r"Reason::LinkGrant\(link\.role\)")
    require(
        equal is not None and grant is not None and equal < grant,
        assertion,
        "decide_link grants a link role before verifying the token",
    )
    return (
        f"shared_link.rs:{shape} shape -> :{load} lookup -> :{expiry} expiry -> :{read} read "
        f"-> :{content} content; decision.rs:{equal} token equality before :{grant} LinkGrant"
    )


def gap_docs_acl_denied() -> str:
    """Document the structurally unreachable denial with live file:line evidence."""
    assertion = "gap-evidence-present"
    ast = source(DOC_AST)
    storage = source(DOC_STORAGE)
    model = source(ACL_MODEL)
    decision_text = source(DECISION)
    evidence = (
        ("acl denial response", line_of(ast, r'"code": "docs_acl_denied"')),
        ("denial call site", line_of(ast, r"acl_denied\(decision\.reason\(\)\)")),
        ("loader call", line_of(ast, r"load_document_from_drive\(&state, &user_id, &doc_id\)")),
        ("verified user id", line_of(ast, r"let user_id = user\.id\(\)\.to_string\(\)")),
        ("acl owner wiring", line_of(ast, r"acl_from_permissions\(&state, &doc_id, &doc\.owner_id")),
        ("Acl::new owner field", line_of(model, r"owner:\s*owner\.into\(\)")),
        ("owner stamp", line_of(storage, r"owner_id:\s*user_identifier\.to_string\(\)")),
        ("same-id principal", line_of(ast, r"Principal::member\(user_id\.clone\(\)")),
        ("owner bypass", line_of(decision_text, r"principal\.member_id\(\) == Some\(acl\.owner\.as_str\(\)\)")),
    )
    for label, number in evidence:
        require(number is not None, assertion, f"evidence '{label}' is missing from the tree")
    labels = {label: number for label, number in evidence}
    return (
        f"docs_acl_denied (ast.rs:{labels['acl denial response']}) is structurally unreachable "
        f"in handle_docs_ast: storage_drive.rs:{labels['owner stamp']} stamps owner_id from the "
        f"same verified requester passed at ast.rs:{labels['loader call']} "
        f"(user.id() at ast.rs:{labels['verified user id']}); ast.rs:"
        f"{labels['acl owner wiring']} installs that id as acl.owner (model.rs:"
        f"{labels['Acl::new owner field']}); ast.rs:{labels['same-id principal']} builds a "
        f"principal with the identical id, so decision.rs:{labels['owner bypass']} always "
        f"returns allow(Reason::Owner) and ast.rs:{labels['denial call site']} is never reached. "
        "Real browser ACL-denied UAT on this read path remains BLOCKED until a read path can "
        "load another owner's object."
    )


def main() -> int:
    checks = (
        ("object-read-owner-from-verified-requester", check_owner_derivation),
        ("owner-bypass-precedes-role-deny", check_owner_bypass_first),
        ("no-untrusted-role-or-owner-in-sharing-json", check_sharing_json_inputs),
        ("link-credential-verified-before-content", check_link_credential_order),
    )
    failures: list[AuditFailure] = []
    for name, check in checks:
        try:
            detail = check()
        except AuditFailure as exc:
            failures.append(exc)
            print(f"[FAIL] {exc.assertion} — {exc.detail}")
        else:
            print(f"[PASS] {name} — {detail}")
    try:
        gap = gap_docs_acl_denied()
    except AuditFailure as exc:
        failures.append(exc)
        print(f"[FAIL] {exc.assertion} — {exc.detail}")
    else:
        print(f"[GAP ] docs-acl-denied-unreachable — {gap}")
    if failures:
        print(f"RESULT: FAIL ({len(failures)} violated assertion(s))")
        return 1
    print(
        f"RESULT: PASS ({len(checks)}/{len(checks)} assertions) — 1 documented GAP, "
        "no browser evidence claimed"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())

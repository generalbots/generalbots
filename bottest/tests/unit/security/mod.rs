//! Security rules that hold without a database or a network (#2074).
//!
//! Every module here drives the same library function the production handler
//! calls, so the accept/reject boundary is exercised without a server:
//!
//! * `jwt_verification` — a Bearer token picks an identity only after its
//!   HS256 signature and expiry are verified (#2112);
//! * `webhook_signatures` — an anonymous WhatsApp/Teams POST proves its origin
//!   or is refused, and an unconfigured secret fails closed (#2113);
//! * `totp_encoding` — enrollment and verification share one base32 secret, so
//!   a code from the QR code is accepted (#2114);
//! * `sheet_access` — a workbook ACL is a pure function of owner and grants.

mod jwt_verification;
mod sheet_access;
mod totp_encoding;
mod webhook_signatures;

//! Long-lived two-way sessions with a program inside the agent workspace,
//! for agent CLIs that ask the server for permission over their own stdio
//! (a JSON-RPC app server, ACP, or a stream-json control channel).
//!
//! - [`DuplexSession`]: one child process with piped stdin, stdout and stderr.
//!   stdout arrives as ordered lines (capped per line), stderr is kept as a
//!   bounded tail, the whole session has a deadline, and every way out
//!   (close, drop, deadline, oversize line) ends with the process gone:
//!   stdin closes, the peer gets a grace period, then its tree is killed.
//!   [`crate::vm::exec_in_vm_duplex`] opens one inside a user's container.
//! - [`RpcClient`]: line-delimited JSON-RPC 2.0 on top of a session.
//! - Raw line mode is the session itself: [`DuplexSession::send_json`] and
//!   [`DuplexSession::recv`].
//!
//! Nothing here logs line contents or stderr.

mod rpc;
mod rpc_route;
mod session;
mod supervise;
mod types;

pub use rpc::{
    Incoming, RpcClient, RpcError, RpcInbox, RpcOptions, ServerRequest, INTERNAL_ERROR,
    METHOD_NOT_FOUND,
};
pub use session::{DuplexEvents, DuplexSender, DuplexSession};
pub use types::{
    DuplexError, DuplexEvent, DuplexOptions, EndReason, SessionEnd, DEFAULT_CLOSE_GRACE,
    DEFAULT_MAX_LINE_BYTES, DEFAULT_SESSION_DEADLINE,
};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod rpc_tests;

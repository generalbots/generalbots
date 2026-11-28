//! Which address the HTTP API listens on.
//!
//! A desktop edition is one person's computer: only the shell on the same
//! machine talks to the server, so it listens on loopback. The peer network
//! does not change that: peers are resolved through the public relay over an
//! outbound connection, and `/api/peer/*` is served by this same listener.
//! Setting `BOTSERVER_HOST` to a non-loopback address opens a desktop build to
//! the network on purpose (sign-in is required in that mode).
//!
//! #2477 — the General Bots OS appliance (shell mode) stays on loopback unless
//! the operator sets `BOTSERVER_HOST` explicitly *and* opts in with
//! `GB_ALLOW_LAN=1`.
//!
//! The same decision feeds `config.server.host`, so log lines, span names and
//! loopback URLs built from the config name the address actually bound.

use std::net::IpAddr;

const LOOPBACK_V4: IpAddr = IpAddr::V4(std::net::Ipv4Addr::LOCALHOST);
const ANY_V4: IpAddr = IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED);

pub(crate) struct BindInputs<'a> {
    pub no_security: bool,
    pub desktop_edition: bool,
    pub shell_edition: bool,
    pub host: &'a str,
    pub lan_opt_in: bool,
}

pub(crate) fn is_loopback_host(host: &str) -> bool {
    matches!(host.trim(), "127.0.0.1" | "localhost" | "::1" | "[::1]")
}

pub(crate) fn decide_bind_ip(inputs: &BindInputs<'_>) -> IpAddr {
    let host = inputs.host.trim();
    let host_is_default = host.is_empty() || is_loopback_host(host);
    let loopback = inputs.no_security
        || ((inputs.desktop_edition || inputs.shell_edition) && host_is_default)
        || (inputs.shell_edition && !inputs.lan_opt_in);
    if loopback {
        LOOPBACK_V4
    } else {
        ANY_V4
    }
}

/// `no_security` as the product file sets it, read the same way as the
/// security setup does (file first, then the loaded global).
pub(crate) fn product_no_security() -> bool {
    botcore::product::ProductConfig::load()
        .map(|product| product.no_security)
        .unwrap_or_else(|_| {
            botcore::product::PRODUCT_CONFIG
                .read()
                .map(|product| product.no_security)
                .unwrap_or(false)
        })
}

pub(crate) fn configured_host() -> String {
    std::env::var("BOTSERVER_HOST").unwrap_or_default()
}

/// The address this process listens on, from the environment and the product.
pub(crate) fn effective_bind_ip(no_security: bool) -> IpAddr {
    let host = configured_host();
    let desktop_edition = botcore::product::PRODUCT_CONFIG
        .read()
        .map(|product| product.edition.as_deref() == Some("desktop"))
        .unwrap_or(false);
    let lan_opt_in = std::env::var("GB_ALLOW_LAN")
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    decide_bind_ip(&BindInputs {
        no_security,
        desktop_edition,
        shell_edition: botcore::product::is_shell_mode(),
        host: &host,
        lan_opt_in,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs(host: &str) -> BindInputs<'_> {
        BindInputs {
            no_security: false,
            desktop_edition: false,
            shell_edition: false,
            host,
            lan_opt_in: false,
        }
    }

    #[test]
    fn a_desktop_build_without_a_host_listens_on_loopback() {
        let decided = decide_bind_ip(&BindInputs { desktop_edition: true, ..inputs("") });
        assert_eq!(decided, LOOPBACK_V4);
    }

    #[test]
    fn a_desktop_build_with_a_loopback_host_listens_on_loopback() {
        for host in ["127.0.0.1", "localhost", "::1", "[::1]", " 127.0.0.1 "] {
            let decided = decide_bind_ip(&BindInputs { desktop_edition: true, ..inputs(host) });
            assert_eq!(decided, LOOPBACK_V4, "host {host:?}");
        }
    }

    #[test]
    fn a_desktop_build_opens_to_the_network_only_with_an_explicit_host() {
        let decided = decide_bind_ip(&BindInputs { desktop_edition: true, ..inputs("0.0.0.0") });
        assert_eq!(decided, ANY_V4);
    }

    #[test]
    fn no_security_always_stays_on_loopback() {
        let decided = decide_bind_ip(&BindInputs {
            no_security: true,
            lan_opt_in: true,
            ..inputs("0.0.0.0")
        });
        assert_eq!(decided, LOOPBACK_V4);
    }

    #[test]
    fn shell_mode_needs_a_host_and_the_lan_opt_in() {
        let without_opt_in = decide_bind_ip(&BindInputs { shell_edition: true, ..inputs("0.0.0.0") });
        assert_eq!(without_opt_in, LOOPBACK_V4);
        let with_opt_in = decide_bind_ip(&BindInputs {
            shell_edition: true,
            lan_opt_in: true,
            ..inputs("0.0.0.0")
        });
        assert_eq!(with_opt_in, ANY_V4);
        let opt_in_without_host = decide_bind_ip(&BindInputs {
            shell_edition: true,
            lan_opt_in: true,
            ..inputs("")
        });
        assert_eq!(opt_in_without_host, LOOPBACK_V4);
    }

    #[test]
    fn a_server_build_keeps_listening_on_every_interface() {
        assert_eq!(decide_bind_ip(&inputs("")), ANY_V4);
    }
}

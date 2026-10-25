//! Server addresses: reading what the user types, recognising Tailscale,
//! and the plain-HTTP rule (roadmap §14.6, D-S6): `http://` only to
//! loopback, private LAN ranges and Tailscale; anything else needs
//! `https://`. Hostnames are judged by what they resolve to.

use std::net::IpAddr;

use url::{Host, Url};

/// Blotter Sync's default port.
pub const DEFAULT_PORT: u16 = 7458;

fn in_v4(ip: std::net::Ipv4Addr, net: [u8; 4], bits: u32) -> bool {
    let mask = u32::MAX.checked_shl(32 - bits).unwrap_or(0);
    (u32::from(ip) & mask) == (u32::from_be_bytes(net) & mask)
}

/// Tailscale's address ranges: 100.64.0.0/10 and fd7a:115c:a1e0::/48.
fn is_tailscale_ip(ip: IpAddr) -> bool {
    match ip.to_canonical() {
        IpAddr::V4(v4) => in_v4(v4, [100, 64, 0, 0], 10),
        IpAddr::V6(v6) => v6.segments()[..3] == [0xfd7a, 0x115c, 0xa1e0],
    }
}

/// A Tailscale address: a `*.ts.net` name or an address in its ranges.
pub fn is_tailscale(address: &str) -> bool {
    let Ok(url) = Url::parse(address) else {
        return false;
    };
    match url.host() {
        Some(Host::Domain(d)) => d.to_ascii_lowercase().ends_with(".ts.net"),
        Some(Host::Ipv4(ip)) => is_tailscale_ip(IpAddr::V4(ip)),
        Some(Host::Ipv6(ip)) => is_tailscale_ip(IpAddr::V6(ip)),
        None => false,
    }
}

/// Whether plain HTTP may go to this address: loopback, private,
/// link-local, IPv6 unique-local, or Tailscale.
pub fn plain_http_ok(ip: IpAddr) -> bool {
    match ip.to_canonical() {
        IpAddr::V4(v4) => {
            v4.is_loopback() || v4.is_private() || v4.is_link_local() || is_tailscale_ip(ip)
        }
        IpAddr::V6(v6) => v6.is_loopback() || v6.is_unique_local() || v6.is_unicast_link_local(),
    }
}

fn insecure(host: &str) -> String {
    format!(
        "Plain http:// only works for addresses on your network or Tailscale. \
         Use https:// for {host}."
    )
}

/// Whether `authority` (`host:port`, `[v6]:port`) spells out a port, so an
/// explicit `:80` isn't mistaken for no port.
fn has_port(authority: &str) -> bool {
    authority
        .rsplit_once(':')
        .is_some_and(|(_, p)| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
}

/// Reads an address as typed (`192.168.1.10`, `nas.local:7458`,
/// `https://sync.example.com/`) into the form the link stores. No scheme
/// means plain http, and plain http without a port means Blotter Sync's
/// default. A path, query or login is refused, as is plain http to a public
/// IP address; names are judged when they're used ([`check`]).
pub fn normalize(input: &str) -> Result<String, String> {
    let text = input.trim();
    if text.is_empty() {
        return Err("Enter the server's address.".into());
    }
    let full = if text.contains("://") {
        text.to_owned()
    } else {
        format!("http://{text}")
    };
    let bad = || format!("“{text}” isn't an address Blotter can use.");
    let mut url = Url::parse(&full).map_err(|_| bad())?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(format!("Use http:// or https:// for {text}."));
    }
    if url.host().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
    {
        return Err(bad());
    }
    if url.scheme() == "http" {
        let authority = full
            .split_once("://")
            .map_or("", |(_, rest)| rest.split('/').next().unwrap_or(""));
        if !has_port(authority) {
            url.set_port(Some(DEFAULT_PORT)).map_err(|()| bad())?;
        }
        let ip = match url.host() {
            Some(Host::Ipv4(ip)) => Some(IpAddr::V4(ip)),
            Some(Host::Ipv6(ip)) => Some(IpAddr::V6(ip)),
            _ => None,
        };
        if ip.is_some_and(|ip| !plain_http_ok(ip)) {
            return Err(insecure(url.host_str().unwrap_or(text)));
        }
    }
    Ok(url.as_str().trim_end_matches('/').to_owned())
}

/// Refuses a plain `http://` address that is, or resolves to, anything
/// outside the ranges [`plain_http_ok`] allows.
pub async fn check(address: &str) -> Result<(), String> {
    let url = Url::parse(address).map_err(|_| format!("“{address}” isn't an address."))?;
    if url.scheme() == "https" {
        return Ok(());
    }
    let port = url.port_or_known_default().unwrap_or(DEFAULT_PORT);
    let ips: Vec<IpAddr> = match url.host() {
        Some(Host::Ipv4(ip)) => vec![IpAddr::V4(ip)],
        Some(Host::Ipv6(ip)) => vec![IpAddr::V6(ip)],
        Some(Host::Domain(d)) => tokio::net::lookup_host((d, port))
            .await
            .map_err(|e| format!("Couldn't look up {d}: {e}"))?
            .map(|a| a.ip())
            .collect(),
        None => Vec::new(),
    };
    if ips.is_empty() || !ips.into_iter().all(plain_http_ok) {
        return Err(insecure(url.host_str().unwrap_or(address)));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addresses_read_as_typed() {
        for (typed, want) in [
            ("192.168.1.10", "http://192.168.1.10:7458"),
            (" nas.local:8080 ", "http://nas.local:8080"),
            ("http://192.168.1.10:80/", "http://192.168.1.10"),
            ("HTTP://NAS.Local", "http://nas.local:7458"),
            ("https://sync.example.com/", "https://sync.example.com"),
            (
                "https://sync.example.com:8443",
                "https://sync.example.com:8443",
            ),
            ("[fd00::1]", "http://[fd00::1]:7458"),
            ("[fd00::1]:9000", "http://[fd00::1]:9000"),
            (
                "https://blotter.tail1234.ts.net",
                "https://blotter.tail1234.ts.net",
            ),
        ] {
            assert_eq!(normalize(typed).as_deref(), Ok(want), "{typed}");
        }
    }

    #[test]
    fn unusable_addresses_say_why() {
        assert_eq!(normalize("  "), Err("Enter the server's address.".into()));
        for bad in [
            "https://sync.example.com/blotter",
            "http://me:pw@nas.local",
            "nas.local?x=1",
            "http://",
        ] {
            let err = normalize(bad).unwrap_err();
            assert!(err.contains("isn't an address"), "{bad}: {err}");
        }
        assert!(
            normalize("ftp://nas.local")
                .unwrap_err()
                .contains("http://")
        );
        let err = normalize("203.0.113.5").unwrap_err();
        assert!(err.contains("Use https:// for 203.0.113.5"), "{err}");
        assert!(normalize("https://203.0.113.5").is_ok());
    }

    #[test]
    fn tailscale_is_recognised() {
        assert!(is_tailscale("https://blotter.tail1234.ts.net"));
        assert!(is_tailscale("http://100.101.102.103:7458"));
        assert!(is_tailscale("http://[fd7a:115c:a1e0::1]:7458"));
        assert!(!is_tailscale("http://192.168.1.10:7458"));
        assert!(
            !is_tailscale("http://100.128.0.1:7458"),
            "outside 100.64/10"
        );
        assert!(!is_tailscale("https://sync.example.com"));
    }

    #[test]
    fn plain_http_only_stays_local() {
        for ok in [
            "127.0.0.1",
            "10.1.2.3",
            "172.20.0.1",
            "192.168.0.9",
            "169.254.1.1",
        ] {
            assert!(plain_http_ok(ok.parse().unwrap()), "{ok}");
        }
        for ok in [
            "::1",
            "fd00::1",
            "fe80::1",
            "100.64.0.1",
            "::ffff:192.168.1.2",
        ] {
            assert!(plain_http_ok(ok.parse().unwrap()), "{ok}");
        }
        for bad in [
            "8.8.8.8",
            "203.0.113.5",
            "2001:db8::1",
            "100.128.0.1",
            "::ffff:8.8.8.8",
        ] {
            assert!(!plain_http_ok(bad.parse().unwrap()), "{bad}");
        }
    }

    #[tokio::test]
    async fn the_rule_applies_to_plain_http_only() {
        assert!(check("http://192.168.1.10:7458").await.is_ok());
        assert!(check("https://203.0.113.5").await.is_ok());
        let err = check("http://203.0.113.5:7458").await.unwrap_err();
        assert!(err.contains("Use https:// for 203.0.113.5"), "{err}");
        assert!(check("http://localhost:7458").await.is_ok());
    }
}

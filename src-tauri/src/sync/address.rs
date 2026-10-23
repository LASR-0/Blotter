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
        let host = url.host_str().unwrap_or(address);
        return Err(format!(
            "Plain http:// only works for addresses on your network or Tailscale. \
             Use https:// for {host}."
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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

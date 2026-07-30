//! System DNS lookup and address formatting.

use std::collections::HashSet;
use std::io::Write;
use std::net::{SocketAddr, ToSocketAddrs};

use crate::error::ResolverError;

/// Metadata from a completed lookup.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct LookupSummary {
    /// Number of unique addresses written.
    pub addresses: u64,
}

/// Resolves `name` with the OS resolver and writes unique IP lines.
///
/// # Errors
///
/// Returns an error when resolution fails, yields no addresses, or output
/// cannot be written.
pub fn lookup_name<W: Write>(name: &str, writer: &mut W) -> Result<LookupSummary, ResolverError> {
    let socket_addrs = (name, 0_u16)
        .to_socket_addrs()
        .map_err(ResolverError::Lookup)?;
    write_addresses(socket_addrs, writer)
}

/// Deduplicates socket addresses and writes each unique IP on its own line.
///
/// # Errors
///
/// Returns [`ResolverError::NoAddresses`] when the iterator is empty, or
/// [`ResolverError::WriteOutput`] when writing fails.
pub fn write_addresses<I, W>(addrs: I, writer: &mut W) -> Result<LookupSummary, ResolverError>
where
    I: IntoIterator<Item = SocketAddr>,
    W: Write,
{
    let mut seen = HashSet::new();
    let mut addresses = 0_u64;

    for socket_addr in addrs {
        let ip = socket_addr.ip();
        if !seen.insert(ip) {
            continue;
        }
        writeln!(writer, "{ip}").map_err(ResolverError::WriteOutput)?;
        addresses += 1;
    }

    if addresses == 0 {
        return Err(ResolverError::NoAddresses);
    }

    Ok(LookupSummary { addresses })
}

/// Collects unique IP addresses from socket addresses in first-seen order.
#[cfg(test)]
fn unique_ips(addrs: impl IntoIterator<Item = SocketAddr>) -> Vec<std::net::IpAddr> {
    let mut seen = HashSet::new();
    let mut ips = Vec::new();
    for socket_addr in addrs {
        let ip = socket_addr.ip();
        if seen.insert(ip) {
            ips.push(ip);
        }
    }
    ips
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

    use super::{unique_ips, write_addresses};
    use crate::error::ResolverError;

    #[test]
    fn deduplicates_and_preserves_order() {
        let addrs = [
            SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            SocketAddr::from((Ipv6Addr::LOCALHOST, 0)),
            SocketAddr::from((Ipv4Addr::LOCALHOST, 80)),
        ];

        assert_eq!(
            unique_ips(addrs),
            [
                IpAddr::V4(Ipv4Addr::LOCALHOST),
                IpAddr::V6(Ipv6Addr::LOCALHOST),
            ]
        );

        let mut output = Vec::new();
        let summary = write_addresses(addrs, &mut output).expect("addresses should write");
        assert_eq!(summary.addresses, 2);
        assert_eq!(String::from_utf8(output).unwrap(), "127.0.0.1\n::1\n");
    }

    #[test]
    fn empty_iterator_is_no_addresses() {
        let error = write_addresses([], &mut Vec::new()).expect_err("empty should fail");
        assert!(matches!(error, ResolverError::NoAddresses));
    }
}

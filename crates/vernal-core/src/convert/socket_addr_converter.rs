//! 网络地址转换器。
//!
//! 对标 Spring 的 `StringToInetAddressConverter` 与 `StringToSocketAddressConverter`。
//! 支持 `SocketAddr` / `SocketAddrV4` / `SocketAddrV6` 的字符串解析。
//!
//! 仅依赖 std,无需任何外部 crate。

use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};

use super::{ConversionError, Convertible};

/// 网络地址转换器。
///
/// 对标 Spring `StringToInetAddressConverter` / `StringToSocketAddressConverter`。
pub struct SocketAddrConverter;

impl Convertible for SocketAddr {
    fn from_str_value(value: &str) -> Result<Self, ConversionError> {
        value
            .parse()
            .map_err(|e: std::net::AddrParseError| ConversionError {
                value: value.to_string(),
                target_type: "SocketAddr",
                reason: format!("网络地址解析失败: {e}"),
            })
    }
}

impl Convertible for SocketAddrV4 {
    fn from_str_value(value: &str) -> Result<Self, ConversionError> {
        value
            .parse()
            .map_err(|e: std::net::AddrParseError| ConversionError {
                value: value.to_string(),
                target_type: "SocketAddrV4",
                reason: format!("IPv4 地址解析失败: {e}"),
            })
    }
}

impl Convertible for SocketAddrV6 {
    fn from_str_value(value: &str) -> Result<Self, ConversionError> {
        value
            .parse()
            .map_err(|e: std::net::AddrParseError| ConversionError {
                value: value.to_string(),
                target_type: "SocketAddrV6",
                reason: format!("IPv6 地址解析失败: {e}"),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn parses_ipv4_socket_addr() {
        let addr = SocketAddr::from_str_value("127.0.0.1:8080").unwrap();
        assert_eq!(addr.ip(), Ipv4Addr::LOCALHOST);
        assert_eq!(addr.port(), 8080);
    }

    #[test]
    fn parses_ipv4_socket_addr_v4() {
        let addr = SocketAddrV4::from_str_value("0.0.0.0:443").unwrap();
        assert_eq!(*addr.ip(), Ipv4Addr::UNSPECIFIED);
        assert_eq!(addr.port(), 443);
    }

    #[test]
    fn parses_ipv6_socket_addr() {
        let addr = SocketAddrV6::from_str_value("[::1]:8080").unwrap();
        assert_eq!(*addr.ip(), Ipv6Addr::LOCALHOST);
        assert_eq!(addr.port(), 8080);
    }

    #[test]
    fn parses_ipv6_full_address() {
        let addr = SocketAddr::from_str_value("[2001:db8::1]:443").unwrap();
        assert_eq!(addr.port(), 443);
    }

    #[test]
    fn rejects_missing_port() {
        let err = SocketAddr::from_str_value("127.0.0.1").unwrap_err();
        assert_eq!(err.target_type, "SocketAddr");
    }

    #[test]
    fn rejects_invalid_ip() {
        let err = SocketAddr::from_str_value("999.999.999.999:80").unwrap_err();
        assert_eq!(err.target_type, "SocketAddr");
        assert!(err.reason.contains("解析失败"));
    }

    #[test]
    fn rejects_empty_string() {
        let err = SocketAddr::from_str_value("").unwrap_err();
        assert_eq!(err.target_type, "SocketAddr");
    }

    #[test]
    fn socket_addr_via_conversion_service() {
        let addr: SocketAddr =
            super::super::ConversionService::convert("192.168.1.1:8080").unwrap();
        assert_eq!(addr.port(), 8080);
    }

    /// `SocketAddrV4` 错误消息标识具体类型
    #[test]
    fn socket_addr_v4_error_mentions_ipv4_kind() {
        let err = SocketAddrV4::from_str_value("not-a-v4-addr:80").unwrap_err();
        assert_eq!(err.target_type, "SocketAddrV4");
        assert!(err.reason.contains("IPv4"));
    }

    /// `SocketAddrV6` 错误消息标识具体类型
    #[test]
    fn socket_addr_v6_error_mentions_ipv6_kind() {
        let err = SocketAddrV6::from_str_value("not-a-v6-addr:80").unwrap_err();
        assert_eq!(err.target_type, "SocketAddrV6");
        assert!(err.reason.contains("IPv6"));
    }

    /// `SocketAddrV4`: 完整 IPv4 地址 + 端口
    #[test]
    fn socket_addr_v4_parses_full_address() {
        let addr = SocketAddrV4::from_str_value("192.168.1.1:8080").unwrap();
        assert_eq!(addr.port(), 8080);
        assert_eq!(*addr.ip(), Ipv4Addr::new(192, 168, 1, 1));
    }
}

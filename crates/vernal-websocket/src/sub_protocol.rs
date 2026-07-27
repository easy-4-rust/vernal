//! WebSocket 子协议协商。

/// 子协议能力提供 trait。
pub trait SubProtocolCapable {
    /// 返回按优先级排列的支持协议。
    fn supported_protocols(&self) -> &[String];
}

/// 在客户端请求和服务端支持列表之间进行确定性协商。
#[must_use]
pub fn negotiate_subprotocol(requested: &[&str], supported: &[String]) -> Option<String> {
    requested
        .iter()
        .find(|requested_protocol| {
            supported
                .iter()
                .any(|candidate| candidate == **requested_protocol)
        })
        .map(|protocol| (*protocol).to_owned())
}

/// 检查单个子协议名称是否合法。
#[must_use]
pub fn is_valid_subprotocol(protocol: &str) -> bool {
    !protocol.is_empty()
        && protocol.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(
                    byte,
                    b'!' | b'#'
                        | b'$'
                        | b'%'
                        | b'&'
                        | b'\''
                        | b'*'
                        | b'+'
                        | b'-'
                        | b'.'
                        | b'^'
                        | b'_'
                        | b'`'
                        | b'|'
                        | b'~'
                )
        })
}

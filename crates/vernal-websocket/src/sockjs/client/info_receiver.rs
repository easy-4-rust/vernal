//! 对应 Java 类：org.springframework.web.socket.sockjs.client.InfoReceiver
//!
//! 从 SockJS 服务端获取 `/info` JSON 响应的 SPI。

use std::{future::Future, pin::Pin};

/// info 响应 future。
pub type InfoFuture<'a> = Pin<Box<dyn Future<Output = Result<ServerInfo, String>> + Send + 'a>>;

/// SockJS 服务端 `/info` 响应。
#[derive(Debug, Clone)]
pub struct ServerInfo {
    /// 是否支持 WebSocket。
    pub websocket: bool,
    /// 是否需要 cookie（JSESSIONID）。
    pub cookie_needed: bool,
    /// 服务端支持的 transport。
    pub entropy: u32,
    /// 原始 JSON。
    pub raw: String,
}

/// InfoReceiver SPI。
pub trait InfoReceiver: Send + Sync {
    /// 执行 `/info` 请求。
    fn fetch_info<'a>(&'a self, info_url: &'a str) -> InfoFuture<'a>;
}

/// 从 JSON 字符串解析 ServerInfo。
///
/// # Errors
///
/// 当 JSON 无效时返回错误。
pub fn parse_info_json(json: &str) -> Result<ServerInfo, String> {
    let value: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    Ok(ServerInfo {
        websocket: value
            .get("websocket")
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
        cookie_needed: value
            .get("cookie_needed")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        entropy: value
            .get("entropy")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .unwrap_or(0),
        raw: json.to_string(),
    })
}

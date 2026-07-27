//! WebSocket 关闭状态。

use std::fmt;

/// WebSocket 关闭原因允许的最大 UTF-8 字节数。
pub const MAX_CLOSE_REASON_BYTES: usize = 123;

/// RFC 6455 关闭码。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CloseCode {
    /// 正常关闭。
    Normal,
    /// 端点离开。
    GoingAway,
    /// 协议错误。
    ProtocolError,
    /// 不支持的数据类型。
    UnsupportedData,
    /// 未提供状态码。
    NoStatus,
    /// 异常关闭，仅用于本地描述。
    Abnormal,
    /// 文本数据不是有效 UTF-8。
    InvalidPayload,
    /// 策略拒绝。
    PolicyViolation,
    /// 消息过大。
    MessageTooBig,
    /// 缺少扩展。
    MandatoryExtension,
    /// 服务端错误。
    ServerError,
    /// TLS 握手失败，仅用于本地描述。
    TlsHandshake,
    /// 应用自定义关闭码。
    Custom(u16),
}

impl CloseCode {
    /// 返回线上的数值关闭码。
    #[must_use]
    pub const fn as_u16(self) -> u16 {
        match self {
            Self::Normal => 1000,
            Self::GoingAway => 1001,
            Self::ProtocolError => 1002,
            Self::UnsupportedData => 1003,
            Self::NoStatus => 1005,
            Self::Abnormal => 1006,
            Self::InvalidPayload => 1007,
            Self::PolicyViolation => 1008,
            Self::MessageTooBig => 1009,
            Self::MandatoryExtension => 1010,
            Self::ServerError => 1011,
            Self::TlsHandshake => 1015,
            Self::Custom(code) => code,
        }
    }

    /// 从数值关闭码创建关闭码。
    #[must_use]
    pub const fn from_u16(code: u16) -> Self {
        match code {
            1000 => Self::Normal,
            1001 => Self::GoingAway,
            1002 => Self::ProtocolError,
            1003 => Self::UnsupportedData,
            1005 => Self::NoStatus,
            1006 => Self::Abnormal,
            1007 => Self::InvalidPayload,
            1008 => Self::PolicyViolation,
            1009 => Self::MessageTooBig,
            1010 => Self::MandatoryExtension,
            1011 => Self::ServerError,
            1015 => Self::TlsHandshake,
            other => Self::Custom(other),
        }
    }

    /// 返回该关闭码是否允许通过网络发送。
    #[must_use]
    pub const fn is_wire_valid(self) -> bool {
        let code = self.as_u16();
        code >= 1000 && code < 5000 && code != 1004 && code != 1005 && code != 1006 && code != 1015
    }
}

/// WebSocket 关闭状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloseStatus {
    code: CloseCode,
    reason: String,
}

impl CloseStatus {
    /// 创建关闭状态。
    ///
    /// # Errors
    ///
    /// 当状态码不能在线上传输，或原因超过 123 个 UTF-8 字节时返回错误。
    pub fn new(code: CloseCode, reason: impl Into<String>) -> Result<Self, CloseStatusError> {
        if !code.is_wire_valid() {
            return Err(CloseStatusError::InvalidCode(code.as_u16()));
        }
        let reason = reason.into();
        if reason.len() > MAX_CLOSE_REASON_BYTES {
            return Err(CloseStatusError::ReasonTooLong(reason.len()));
        }
        Ok(Self { code, reason })
    }

    /// 创建正常关闭状态。
    #[must_use]
    pub fn normal() -> Self {
        Self {
            code: CloseCode::Normal,
            reason: String::new(),
        }
    }

    /// 返回关闭码。
    #[must_use]
    pub const fn code(&self) -> CloseCode {
        self.code
    }

    /// 返回关闭原因。
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

impl fmt::Display for CloseStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.reason.is_empty() {
            write!(formatter, "{}", self.code.as_u16())
        } else {
            write!(formatter, "{} {}", self.code.as_u16(), self.reason)
        }
    }
}

/// 关闭状态校验错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloseStatusError {
    /// 状态码不能在线上传输。
    InvalidCode(u16),
    /// 关闭原因过长。
    ReasonTooLong(usize),
}

impl fmt::Display for CloseStatusError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCode(code) => write!(formatter, "关闭码 {code} 不能在线上传输"),
            Self::ReasonTooLong(length) => write!(
                formatter,
                "关闭原因长度 {length} 超过 {MAX_CLOSE_REASON_BYTES} 字节"
            ),
        }
    }
}

impl std::error::Error for CloseStatusError {}

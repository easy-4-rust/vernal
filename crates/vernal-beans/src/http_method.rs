//! HttpMethod — HTTP 方法。
/// HTTP 方法枚举。
#[derive(Clone, Debug, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
    Trace,
}

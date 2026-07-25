//! 测试用非法租户级消费者对象。

use std::sync::Arc;

use super::request_service::RequestService;

/// 故意让父作用域依赖子作用域，用于验证生命周期收窄保护。
pub struct TenantConsumer {
    /// 不应被成功注入的短生命周期请求服务。
    pub _request: Arc<RequestService>,
}

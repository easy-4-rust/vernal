//! 测试用请求级服务对象。

use std::sync::Arc;

use super::tenant_value::TenantValue;

/// 验证短生命周期组件可以安全依赖父作用域组件。
pub struct RequestService {
    /// 从父租户 Scope 解析的共享组件。
    pub tenant: Arc<TenantValue>,
}

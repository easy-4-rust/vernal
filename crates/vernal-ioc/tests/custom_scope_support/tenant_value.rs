//! 测试用租户级组件对象。

/// 被请求级组件依赖的租户共享值。
pub struct TenantValue {
    /// 用于断言依赖注入结果的静态名称。
    pub name: &'static str,
}

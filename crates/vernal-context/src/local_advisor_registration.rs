//! 应用本地切面顾问登记对象。

use vernal_aop::LocalAdvisor;

use crate::managed_local_advisor::ManagedLocalAdvisor;

/// 保留直接 Local Advisor 与 `IoC` 管理 Local Advisor 的统一登记顺序。
///
/// 该声明只存在于应用构建阶段；目录封存后不参与 Worker-local 调用热路径。
pub(crate) enum LocalAdvisorRegistration {
    /// 调用方已经构造完成的本地 Advisor。
    Instance(LocalAdvisor),
    /// 需要从当前应用 Container 解析拦截器的本地 Advisor。
    Component(ManagedLocalAdvisor),
}

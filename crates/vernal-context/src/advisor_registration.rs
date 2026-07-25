//! 应用切面顾问登记对象。

use vernal_aop::Advisor;

use crate::managed_advisor::ManagedAdvisor;

/// 保留直接 Advisor 与 `IoC` 管理 Advisor 的统一登记顺序。
///
/// 相同 `order` 的拦截器依赖稳定注册顺序决定嵌套关系，因此不能把两类声明分别
/// 存储后再拼接。该枚举只存在于应用构建阶段，计划封存后不会进入运行时热路径。
pub(crate) enum AdvisorRegistration {
    /// 调用方已经构造完成的普通 Advisor。
    Instance(Advisor),
    /// 需要从当前应用 Container 解析拦截器的 Advisor。
    Component(ManagedAdvisor),
}

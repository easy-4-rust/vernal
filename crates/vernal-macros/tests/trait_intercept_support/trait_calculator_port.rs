//! Trait 默认方法织入合同测试端口对象。

use vernal_aop::InvocationError;

/// 由具体 AOP 组件继承默认业务实现的异步端口。
///
/// 宏只为被拦截方法添加最小 `Self: AopComponent` 边界，Trait 本身不需要继承
/// Vernal 运行时契约。最终实现对象仍提供当前 `ApplicationContext` 的计划目录和
/// 取消令牌，宏不会通过全局表寻找实例。
pub(crate) trait TraitCalculatorPort {
    /// 使用 Trait 默认方法执行一次泛型回显。
    #[vernal_macros::intercept(
        component = "TraitCalculatorPort",
        tags = ["trait", "generic"]
    )]
    async fn trait_echo<T>(&self, value: T) -> Result<T, InvocationError>
    where
        T: Send + Sync + 'static,
    {
        tokio::task::yield_now().await;
        Ok(value)
    }

    /// 不声明 component 时，Operation 身份使用最终实现类型而不是 Trait 类型。
    #[vernal_macros::intercept(tags = ["trait-default-identity"])]
    async fn implementor_name(&self) -> Result<&'static str, InvocationError> {
        Ok(std::any::type_name::<Self>())
    }
}

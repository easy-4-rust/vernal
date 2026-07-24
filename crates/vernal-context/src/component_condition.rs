//! 组件装配条件契约。

use vernal_core::BoxError;

use crate::ApplicationEnvironment;

/// 在应用构建阶段决定一组组件是否进入 `IoC` 依赖图。
///
/// 条件只读取已经冻结的 [`ApplicationEnvironment`]，不能访问容器、组件实例或
/// 运行期请求，因此不会形成隐式 Service Locator。实现必须是确定性的：相同环境
/// 应返回相同结果。失败应通过 `Result` 返回，Vernal 会将其包装成脱敏构建错误。
pub trait ComponentCondition: Send + Sync + 'static {
    /// 返回适合进入启动诊断的稳定条件类型名。
    ///
    /// 名称应是 `profile.any`、`property.equals` 一类静态代码，不能包含属性值、
    /// Token、连接串或动态错误正文。
    fn name(&self) -> &'static str;

    /// 判断条件在当前应用环境中是否命中。
    ///
    /// # Errors
    ///
    /// 属性来源不可用、占位符解析失败或自定义条件无法完成判断时返回错误。
    fn matches(&self, environment: &ApplicationEnvironment) -> Result<bool, BoxError>;
}

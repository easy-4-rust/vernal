//! 应用属性来源合同对象。

use crate::EnvironmentError;

/// 向 [`crate::ApplicationEnvironment`] 提供字符串属性的框架中立端口。
///
/// Vernal 只定义来源优先级与解析语义，不直接依赖 TOML、YAML、环境变量、Nacos
/// 或 Hutool `.setting`。适配器可以实现本 Trait，把已加载的数据或动态配置中心
/// 暴露为普通 Context 组件。实现不得在读取失败时 panic，应返回
/// [`EnvironmentError::PropertySource`]。
pub trait PropertySource: Send + Sync + 'static {
    /// 返回稳定且不含凭证的来源名称。
    ///
    /// 名称参与重复检测和错误诊断，但不会被当作属性键。
    fn name(&self) -> &str;

    /// 按精确属性键读取原始字符串值。
    ///
    /// 返回 `Ok(None)` 表示当前来源没有该键，Environment 会继续检查低优先级
    /// 来源。实现不应自行展开 `${...}`，统一展开由 Environment 完成。
    ///
    /// # Errors
    ///
    /// 文件、远端配置中心或其他底层来源读取失败时返回 [`EnvironmentError`]。
    fn get(&self, key: &str) -> Result<Option<String>, EnvironmentError>;
}

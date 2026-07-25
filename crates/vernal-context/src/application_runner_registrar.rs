//! 应用 Runner 注册回调类型。

use crate::ApplicationContextBuilder;

/// 在依赖图冻结后，把已声明的一次性 Runner 登记到 Context 建造器。
///
/// 回调只保存具体组件类型和可选限定符，不提前解析实例；最终解析、作用域校验和
/// 依赖顺序仍由 [`ApplicationContextBuilder`] 使用同一个应用 Container 完成。
pub(crate) type ApplicationRunnerRegistrar =
    dyn FnOnce(&mut ApplicationContextBuilder) + Send + Sync + 'static;

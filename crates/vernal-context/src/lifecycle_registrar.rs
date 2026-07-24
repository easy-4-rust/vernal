//! 生命周期组件注册回调类型。

use crate::ApplicationContextBuilder;

/// 在依赖图冻结后，把已命中的生命周期组件登记到 Context 建造器。
///
/// 回调只保存组件类型和可选限定符，不提前解析组件实例；真正的解析与排序仍由
/// [`ApplicationContextBuilder`] 依据 `IoC` 构建计划完成。
pub(crate) type LifecycleRegistrar =
    dyn FnOnce(&mut ApplicationContextBuilder) + Send + Sync + 'static;

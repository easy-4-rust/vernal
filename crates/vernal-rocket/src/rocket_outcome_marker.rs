//! Rocket Handler Outcome 类型擦除标记对象。

/// 表示原生 Outcome 已保存在当前借用型目标中。
///
/// 标记本身不携带 Response 或 Data，避免带请求生命周期的 Rocket 类型逃逸到
/// `'static` 的 AOP 返回值；实际 Outcome 只能由同一次 Handler 调用取回。
pub(crate) struct RocketOutcomeMarker;

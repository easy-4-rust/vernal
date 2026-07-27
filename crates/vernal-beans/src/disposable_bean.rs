//! DisposableBean — Spring 风格的销毁回调接口。
//!
//! 对应 Java 类：`org.springframework.beans.factory.DisposableBean`。
//!
//! Bean 实现此接口后，容器在销毁 Bean 时回调 `destroy`。

use crate::aware::Aware;

/// Spring 风格的销毁回调接口。
///
/// 对应 Spring 的 `DisposableBean.destroy()`。
///
/// 容器关闭或单例 Bean 被销毁时，按逆拓扑序调用此方法。
///
/// ## 执行顺序
///
/// 1. `DestructionAwareBeanPostProcessor.postProcessBeforeDestruction`
/// 2. **`DisposableBean.destroy`** ← 此接口
/// 3. 自定义 `destroy-method`
pub trait DisposableBean: Aware {
    /// 销毁 Bean 时调用。
    ///
    /// 对应 Spring 的 `DisposableBean.destroy()`。
    ///
    /// # 错误
    ///
    /// 返回 `Err` 时容器会记录警告但继续销毁其他 Bean。
    fn destroy(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

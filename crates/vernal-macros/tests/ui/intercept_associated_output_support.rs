//! AOP 关联输出正向编译合同。

/// 把输入值投影成一个由实现类型决定的关联输出。
pub trait AssociatedProjection {
    type Output;

    fn project(self) -> Self::Output;
}

impl AssociatedProjection for String {
    type Output = usize;

    fn project(self) -> Self::Output {
        self.len()
    }
}

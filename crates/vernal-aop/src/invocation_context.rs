//! 调用扩展上下文对象。

use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use tokio::sync::RwLock;

/// 在异步拦截器链中传递强类型扩展数据。
///
/// 上下文使用 Tokio 异步读写锁，拦截器可以跨 `.await` 安全共享认证主体、
/// Trace 信息、幂等键或 Web 请求快照。扩展值必须满足 `Send + Sync + 'static`，
/// 借用型框架请求应先转换成 owned snapshot 再放入上下文。
#[derive(Default)]
pub struct InvocationContext {
    values: RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
}

impl InvocationContext {
    /// 创建空调用上下文。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 写入或替换一种强类型扩展值，并返回旧值。
    pub async fn insert<T>(&self, value: T) -> Option<T>
    where
        T: Any + Send + Sync,
    {
        self.values
            .write()
            .await
            .insert(TypeId::of::<T>(), Box::new(value))
            .and_then(|old| old.downcast::<T>().ok())
            .map(|old| *old)
    }

    /// 克隆读取一种强类型扩展值。
    ///
    /// 返回克隆值可以确保锁守卫不会跨越调用方后续的 `.await`。
    pub async fn get<T>(&self) -> Option<T>
    where
        T: Any + Clone + Send + Sync,
    {
        self.values
            .read()
            .await
            .get(&TypeId::of::<T>())
            .and_then(|value| value.downcast_ref::<T>())
            .cloned()
    }

    /// 移除并返回一种强类型扩展值。
    pub async fn remove<T>(&self) -> Option<T>
    where
        T: Any + Send + Sync,
    {
        self.values
            .write()
            .await
            .remove(&TypeId::of::<T>())
            .and_then(|value| value.downcast::<T>().ok())
            .map(|value| *value)
    }

    /// 判断指定扩展类型是否存在。
    pub async fn contains<T>(&self) -> bool
    where
        T: Any + Send + Sync,
    {
        self.values.read().await.contains_key(&TypeId::of::<T>())
    }
}

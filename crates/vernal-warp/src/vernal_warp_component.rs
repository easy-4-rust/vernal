//! Warp `IoC` 组件 Filter 对象。

use std::{any::Any, marker::PhantomData, ops::Deref, sync::Arc};

use warp::{Filter, Rejection};

use crate::{VernalWarpContext, WarpRejection};

/// 从当前 `ApplicationContext` 解析类型化组件。
pub struct VernalWarpComponent<T> {
    component: Arc<T>,
    marker: PhantomData<T>,
}

impl<T> VernalWarpComponent<T> {
    /// 创建组件 Filter。
    #[must_use]
    pub fn filter() -> impl Filter<Extract = (Self,), Error = Rejection> + Clone
    where
        T: Any + Send + Sync,
    {
        VernalWarpContext::filter().and_then(|context: VernalWarpContext| async move {
            context
                .0
                .container()
                .resolve::<T>()
                .map(|component| Self {
                    component,
                    marker: PhantomData,
                })
                .map_err(|error| warp::reject::custom(WarpRejection::component_resolution(error)))
        })
    }

    /// 取得共享组件所有权。
    #[must_use]
    pub fn into_inner(self) -> Arc<T> {
        self.component
    }
}

impl<T> Deref for VernalWarpComponent<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.component
    }
}

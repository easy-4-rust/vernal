//! Rocket Route 集合严格 AOP 装配扩展对象。

use std::mem;

use rocket::{Route, route::dummy_handler};

use crate::vernal_rocket_handler::VernalRocketHandler;

/// 为 `routes![...]` 生成的 Route 集合批量织入 Vernal 严格 AOP。
pub trait VernalRocketRoutesExt {
    /// 替换每条 Route 的 Handler，同时保留其全部原生路由元数据。
    ///
    /// 返回值可直接传给 `Rocket::mount`。应用仍需安装
    /// [`crate::VernalRocketFairing`]，由 Fairing 注册 Context 并管理请求 Scope。
    /// 每个集合只应调用一次本方法，避免重复织入同一 Handler。
    #[must_use]
    fn with_vernal_aop(self) -> Self;
}

impl VernalRocketRoutesExt for Vec<Route> {
    fn with_vernal_aop(mut self) -> Self {
        for route in &mut self {
            // Route 的 handler 字段公开，但没有 take API；用 Rocket 自带的隐藏
            // dummy handler 临时占位，可以在不重建 Route 的情况下保留 Name、
            // Method、URI、Rank、Format 与宏发现的 Sentinel。
            let inner = mem::replace(&mut route.handler, Box::new(dummy_handler));
            route.handler = Box::new(VernalRocketHandler::new(inner));
        }
        self
    }
}

//! Salvo 匹配路由元数据转换对象。

use salvo::Request;
use vernal_web::RouteMetadata;

use crate::SalvoAopError;

/// 从 Salvo `matched-path` 生成低基数 Vernal 操作身份。
pub(crate) struct SalvoRouteMetadata;

impl SalvoRouteMetadata {
    /// 读取已经完成路由匹配的模板，并结合真实 HTTP 方法创建路由元数据。
    ///
    /// # Errors
    ///
    /// 非根请求缺少匹配模板时返回 fail-closed 错误，绝不回退到包含用户输入的
    /// 原始 URI。
    pub(crate) fn capture(request: &Request) -> Result<RouteMetadata, SalvoAopError> {
        let matched = request.matched_path().trim_matches('/');
        let path_pattern = if matched.is_empty() {
            if request.uri().path() == "/" {
                String::from("/")
            } else {
                return Err(SalvoAopError::MissingRouteMetadata);
            }
        } else {
            format!("/{matched}")
        };
        Ok(RouteMetadata::new(
            path_pattern.clone(),
            request.method().as_str(),
            path_pattern,
        ))
    }
}

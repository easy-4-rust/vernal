//! NamespaceHandlerResolver — 命名空间处理器解析器接口。
//!
//! 对应 Java 类：`org.springframework.beans.factory.xml.NamespaceHandlerResolver`。
//!
//! 根据命名空间 URI 查找对应的 `NamespaceHandler`。

use crate::factory::xml::namespace_handler::NamespaceHandler;

/// 命名空间处理器解析器接口。
///
/// 对应 Spring 的 `NamespaceHandlerResolver`。
///
/// 根据命名空间 URI 解析对应的 `NamespaceHandler`。
/// Spring 默认使用 `DefaultNamespaceHandlerResolver`，
/// 通过 `META-INF/spring.handlers` 文件查找处理器。
pub trait NamespaceHandlerResolver: Send + Sync {
    /// 根据命名空间 URI 解析处理器。
    ///
    /// # 参数
    /// - `namespace_uri` — 命名空间 URI
    ///
    /// # 返回
    /// 对应的命名空间处理器。
    fn resolve(
        &self,
        namespace_uri: &str,
    ) -> Result<Box<dyn NamespaceHandler>, Box<dyn std::error::Error + Send + Sync>>;
}

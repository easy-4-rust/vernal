//! 对标 `org.springframework.transaction.aspectj` 包。
//!
//! 提供 AspectJ 事务管理切面的 Rust 等价实现，覆盖：
//! - `AbstractTransactionAspect`：事务切面抽象层
//! - `AnnotationTransactionAspect`：`@Transactional` 注解驱动
//! - `JtaAnnotationTransactionAspect`：JTA 1.2 `@Transactional` 驱动
//! - `AspectJTransactionManagementConfiguration`：Spring @Configuration 等价
//! - `AspectJJtaTransactionManagementConfiguration`：JTA @Configuration 等价
//! - `Propagation` / `Isolation` / `TransactionAttribute`：事务属性模型
//! - `TransactionAspectSupport`：事务核心执行引擎
//! - `Rethrower`：checked 异常透传辅助

mod abstract_transaction_aspect;
mod annotation_transaction_aspect;
mod aspect_j_jta_transaction_management_configuration;
mod aspect_j_transaction_management_configuration;
mod isolation;
mod jta_annotation_transaction_aspect;
mod propagation;
mod rethrower;
mod transaction_aspect_support;
mod transaction_attribute;
mod transaction_attribute_source;

pub use abstract_transaction_aspect::AbstractTransactionAspect;
pub use annotation_transaction_aspect::AnnotationTransactionAspect;
pub use aspect_j_jta_transaction_management_configuration::AspectJJtaTransactionManagementConfiguration;
pub use aspect_j_transaction_management_configuration::AspectJTransactionManagementConfiguration;
pub use isolation::Isolation;
pub use jta_annotation_transaction_aspect::JtaAnnotationTransactionAspect;
pub use propagation::Propagation;
pub use rethrower::Rethrower;
pub use transaction_aspect_support::TransactionAspectSupport;
pub use transaction_attribute::TransactionAttribute;
pub use transaction_attribute_source::TransactionAttributeSource;

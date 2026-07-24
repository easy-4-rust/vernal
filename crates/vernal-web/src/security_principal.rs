//! 安全主体对象。

use std::sync::Arc;

/// 由 Sa-Token-Rust 等安全 Bridge 写入请求上下文的只读主体。
///
/// Vernal 只携带身份与角色，不解释登录、Session、权限或踢人下线语义。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SecurityPrincipal {
    subject: Arc<str>,
    roles: Arc<[Arc<str>]>,
}

impl SecurityPrincipal {
    /// 创建安全主体。
    #[must_use]
    pub fn new<I, S>(subject: impl Into<Arc<str>>, roles: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<Arc<str>>,
    {
        Self {
            subject: subject.into(),
            roles: roles.into_iter().map(Into::into).collect(),
        }
    }

    /// 返回主体标识。
    #[must_use]
    pub fn subject(&self) -> &str {
        &self.subject
    }

    /// 返回只读角色集合。
    #[must_use]
    pub fn roles(&self) -> &[Arc<str>] {
        &self.roles
    }

    /// 判断主体是否具有指定角色。
    #[must_use]
    pub fn has_role(&self, role: &str) -> bool {
        self.roles
            .iter()
            .any(|candidate| candidate.as_ref() == role)
    }
}

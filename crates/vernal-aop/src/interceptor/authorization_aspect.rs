//! 授权切面。
//!
//! 对应 aspect-rs：aspect-std/src/authorization.rs。
//! spring-aop 无直接对应。
//!
//! 移植自 aspect-rs 的 `AuthorizationAspect`（RBAC）+ `AllowlistAspect`（身份白名单），
//! 改造为 Tokio-first 异步。
//!
//! 提供两种互补的授权切面：
//! - [`AuthorizationAspect`] — 基于角色的访问控制（RBAC）
//! - [`AllowlistAspect`] — 基于身份的白名单

use std::collections::HashSet;
use std::sync::Arc;

use crate::{Interceptor, Invocation, InvocationError, InvocationFuture, InvocationResult, Next};

/// 授权模式。
///
/// 对应 aspect-rs `AuthMode`。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AuthMode {
    /// 要求所有指定角色。
    RequireAll,
    /// 要求任意一个指定角色。
    RequireAny,
}

/// 基于角色的访问控制切面。
///
/// 对应 aspect-rs `AuthorizationAspect`。
/// 在函数执行前检查调用者是否拥有所需角色。
///
/// # 用法
///
/// ```rust,ignore
/// use vernal_aop::AuthorizationAspect;
///
/// let auth = AuthorizationAspect::require_role("admin", || {
///     get_current_user_roles()
/// });
/// ```
#[derive(Clone)]
pub struct AuthorizationAspect {
    required_roles: Arc<HashSet<String>>,
    role_provider: Arc<dyn Fn() -> HashSet<String> + Send + Sync>,
    mode: AuthMode,
}

impl AuthorizationAspect {
    /// 创建要求特定角色的授权切面。
    ///
    /// # 参数
    /// * `role` - 所需角色
    /// * `role_provider` - 返回当前用户角色的函数
    pub fn require_role<F>(role: &str, role_provider: F) -> Self
    where
        F: Fn() -> HashSet<String> + Send + Sync + 'static,
    {
        let mut roles = HashSet::new();
        roles.insert(role.to_string());

        Self {
            required_roles: Arc::new(roles),
            role_provider: Arc::new(role_provider),
            mode: AuthMode::RequireAll,
        }
    }

    /// 创建要求多个角色的授权切面。
    ///
    /// # 参数
    /// * `roles` - 所需角色列表
    /// * `role_provider` - 返回当前用户角色的函数
    /// * `mode` - 要求所有角色还是任意一个
    pub fn require_roles<F>(roles: &[&str], role_provider: F, mode: AuthMode) -> Self
    where
        F: Fn() -> HashSet<String> + Send + Sync + 'static,
    {
        let role_set: HashSet<String> = roles.iter().map(|r| r.to_string()).collect();

        Self {
            required_roles: Arc::new(role_set),
            role_provider: Arc::new(role_provider),
            mode,
        }
    }

    /// 检查当前用户是否被授权。
    fn check_authorization(&self) -> Result<(), String> {
        let current_roles = (self.role_provider)();

        let authorized = match self.mode {
            AuthMode::RequireAll => self
                .required_roles
                .iter()
                .all(|r| current_roles.contains(r)),
            AuthMode::RequireAny => self
                .required_roles
                .iter()
                .any(|r| current_roles.contains(r)),
        };

        if authorized {
            Ok(())
        } else {
            let required: Vec<_> = self.required_roles.iter().cloned().collect();
            let mode_str = match self.mode {
                AuthMode::RequireAll => "all",
                AuthMode::RequireAny => "any",
            };
            Err(format!(
                "Access denied: requires {} of roles {:?}",
                mode_str, required
            ))
        }
    }
}

impl Interceptor for AuthorizationAspect {
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: Next<'a>,
    ) -> InvocationFuture<'a> {
        Box::pin(async move {
            self.check_authorization().map_err(|_| InvocationError::Cancelled)?;
            next.run(invocation).await
        })
    }
}

/// 基于身份的白名单切面。
///
/// 对应 aspect-rs `AllowlistAspect`。
/// 当提供的身份字符串存在于配置的白名单中时放行调用。
///
/// 支持通配符 `"*"` 和可选的规范化钩子。
///
/// # 用法
///
/// ```rust,ignore
/// use vernal_aop::AllowlistAspect;
///
/// let policy = AllowlistAspect::new(["user_a", "user_b"]);
/// assert!(policy.is_allowed("user_a"));
/// assert!(!policy.is_allowed("intruder"));
///
/// // 通配符
/// let open = AllowlistAspect::new(["*"]);
/// assert!(open.is_allowed("anyone"));
/// ```
#[derive(Clone)]
pub struct AllowlistAspect {
    inner: Arc<AllowlistInner>,
}

struct AllowlistInner {
    entries: tokio::sync::RwLock<Vec<String>>,
    normalizer: Option<Arc<dyn Fn(&str) -> String + Send + Sync>>,
    matcher: Option<Arc<dyn Fn(&str, &str) -> bool + Send + Sync>>,
}

impl Clone for AllowlistInner {
    fn clone(&self) -> Self {
        // 注意：clone 时 entries 需要同步读取，使用 try_read
        // 如果锁被持有，则使用空 Vec
        let entries = self
            .entries
            .try_read()
            .map(|guard| guard.clone())
            .unwrap_or_default();
        Self {
            entries: tokio::sync::RwLock::new(entries),
            normalizer: None,
            matcher: None,
        }
    }
}

impl AllowlistAspect {
    /// 从初始条目集合构建白名单切面。
    ///
    /// 通配符 `"*"` 允许所有人。空列表拒绝所有人（fail-closed）。
    pub fn new<I, S>(entries: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            inner: Arc::new(AllowlistInner {
                entries: tokio::sync::RwLock::new(entries.into_iter().map(Into::into).collect()),
                normalizer: None,
                matcher: None,
            }),
        }
    }

    /// 附加规范化函数，应用于白名单条目和查询身份后再比较。
    pub fn with_normalizer<F>(self, normalizer: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        let inner = Arc::new(AllowlistInner {
            entries: tokio::sync::RwLock::new(
                self.inner.entries.try_read().map(|g| g.clone()).unwrap_or_default(),
            ),
            normalizer: Some(Arc::new(normalizer)),
            matcher: self.inner.matcher.clone(),
        });
        Self { inner }
    }

    /// 附加自定义匹配谓词 `(entry, identity) -> bool`。
    pub fn with_matcher<F>(self, matcher: F) -> Self
    where
        F: Fn(&str, &str) -> bool + Send + Sync + 'static,
    {
        let inner = Arc::new(AllowlistInner {
            entries: tokio::sync::RwLock::new(
                self.inner.entries.try_read().map(|g| g.clone()).unwrap_or_default(),
            ),
            normalizer: None,
            matcher: Some(Arc::new(matcher)),
        });
        Self { inner }
    }

    /// 检查身份是否被白名单允许。
    pub async fn is_allowed(&self, identity: &str) -> bool {
        let entries = self.inner.entries.read().await;
        if entries.is_empty() {
            return false;
        }
        if let Some(matcher) = self.inner.matcher.as_ref() {
            for entry in entries.iter() {
                if entry == "*" {
                    return true;
                }
                if matcher(entry, identity) {
                    return true;
                }
            }
            return false;
        }
        let norm = self.inner.normalizer.as_ref();
        let needle = norm.as_ref().map(|f| f(identity));
        for entry in entries.iter() {
            if entry == "*" {
                return true;
            }
            match (&needle, norm) {
                (Some(needle), Some(f)) => {
                    if f(entry) == *needle {
                        return true;
                    }
                }
                _ => {
                    if entry == identity {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// 追加身份到白名单（如果不存在）。
    pub async fn add(&self, identity: impl Into<String>) {
        let identity = identity.into();
        let mut entries = self.inner.entries.write().await;
        if !entries.iter().any(|e| e == &identity) {
            entries.push(identity);
        }
    }

    /// 原子替换整个白名单。
    pub async fn set<I, S>(&self, entries: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        *self.inner.entries.write().await = entries.into_iter().map(Into::into).collect();
    }

    /// 快照当前白名单。
    pub async fn snapshot(&self) -> Vec<String> {
        self.inner.entries.read().await.clone()
    }

    /// 当前条目数量。
    pub async fn len(&self) -> usize {
        self.inner.entries.read().await.len()
    }

    /// 白名单是否为空。
    pub async fn is_empty(&self) -> bool {
        self.inner.entries.read().await.is_empty()
    }
}

/// 每次调用的上下文。
#[derive(Clone, Debug)]
pub struct AllowlistCall {
    /// 被检查的身份。
    pub identity: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_roles(roles: Vec<&str>) -> HashSet<String> {
        roles.into_iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn require_role_success() {
        let auth = AuthorizationAspect::require_role("admin", || mock_roles(vec!["admin"]));
        assert!(auth.check_authorization().is_ok());
    }

    #[test]
    fn require_role_failure() {
        let auth = AuthorizationAspect::require_role("admin", || mock_roles(vec!["user"]));
        assert!(auth.check_authorization().is_err());
    }

    #[test]
    fn require_all_success() {
        let auth = AuthorizationAspect::require_roles(
            &["admin", "moderator"],
            || mock_roles(vec!["admin", "moderator", "user"]),
            AuthMode::RequireAll,
        );
        assert!(auth.check_authorization().is_ok());
    }

    #[test]
    fn require_all_failure() {
        let auth = AuthorizationAspect::require_roles(
            &["admin", "moderator"],
            || mock_roles(vec!["admin"]),
            AuthMode::RequireAll,
        );
        assert!(auth.check_authorization().is_err());
    }

    #[test]
    fn require_any_success() {
        let auth = AuthorizationAspect::require_roles(
            &["admin", "moderator"],
            || mock_roles(vec!["moderator"]),
            AuthMode::RequireAny,
        );
        assert!(auth.check_authorization().is_ok());
    }

    #[test]
    fn require_any_failure() {
        let auth = AuthorizationAspect::require_roles(
            &["admin", "moderator"],
            || mock_roles(vec!["user"]),
            AuthMode::RequireAny,
        );
        assert!(auth.check_authorization().is_err());
    }

    #[test]
    fn empty_roles() {
        let auth = AuthorizationAspect::require_role("admin", || mock_roles(vec![]));
        assert!(auth.check_authorization().is_err());
    }

    // AllowlistAspect tests

    #[tokio::test]
    async fn empty_list_denies_everyone() {
        let policy = AllowlistAspect::new(Vec::<String>::new());
        assert!(!policy.is_allowed("alice").await);
        assert!(!policy.is_allowed("").await);
    }

    #[tokio::test]
    async fn wildcard_admits_everyone() {
        let policy = AllowlistAspect::new(["*".to_string()]);
        assert!(policy.is_allowed("alice").await);
        assert!(policy.is_allowed("anyone").await);
        assert!(policy.is_allowed("").await);
    }

    #[tokio::test]
    async fn specific_identity() {
        let policy = AllowlistAspect::new(["alice".to_string(), "bob".to_string()]);
        assert!(policy.is_allowed("alice").await);
        assert!(policy.is_allowed("bob").await);
        assert!(!policy.is_allowed("charlie").await);
    }

    #[tokio::test]
    async fn wildcard_mixed_with_specific() {
        let policy = AllowlistAspect::new(["alice".to_string(), "*".to_string()]);
        assert!(policy.is_allowed("alice").await);
        assert!(policy.is_allowed("charlie").await);
    }

    #[tokio::test]
    async fn normalizer_case_insensitive() {
        let policy = AllowlistAspect::new(["Alice@Example.com".to_string()])
            .with_normalizer(|s| s.to_ascii_lowercase());
        assert!(policy.is_allowed("alice@example.com").await);
        assert!(policy.is_allowed("ALICE@EXAMPLE.COM").await);
        assert!(!policy.is_allowed("bob@example.com").await);
    }

    #[tokio::test]
    async fn add_appends_unique() {
        let policy = AllowlistAspect::new(["alice".to_string()]);
        policy.add("bob").await;
        policy.add("bob").await;
        assert_eq!(policy.len().await, 2);
        assert!(policy.is_allowed("bob").await);
    }

    #[tokio::test]
    async fn set_replaces_atomically() {
        let policy = AllowlistAspect::new(["alice".to_string()]);
        policy.set(["bob".to_string(), "carol".to_string()]).await;
        assert!(!policy.is_allowed("alice").await);
        assert!(policy.is_allowed("bob").await);
        assert!(policy.is_allowed("carol").await);
        assert_eq!(policy.len().await, 2);
    }

    #[tokio::test]
    async fn snapshot_returns_owned_copy() {
        let policy = AllowlistAspect::new(["alice".to_string(), "bob".to_string()]);
        let snap = policy.snapshot().await;
        assert_eq!(snap, vec!["alice".to_string(), "bob".to_string()]);
        policy.add("carol").await;
        assert_eq!(snap.len(), 2);
    }

    #[tokio::test]
    async fn clone_shares_underlying_list() {
        let policy_a = AllowlistAspect::new(["alice".to_string()]);
        let policy_b = policy_a.clone();
        policy_b.add("bob").await;
        assert!(policy_a.is_allowed("bob").await);
    }
}

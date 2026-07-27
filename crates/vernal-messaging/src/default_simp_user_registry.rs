//! SIMP 用户/会话/订阅注册表。对标 Spring `SimpUserRegistry`/`SimpUser`/`SimpSession`/`SimpSubscription`。

use std::collections::BTreeMap;
use std::sync::Arc;

use tokio::sync::RwLock;

/// SIMP 订阅。对标 Spring `SimpSubscription`。
#[derive(Debug, Clone)]
pub struct SimpSubscription {
    /// 订阅 ID。
    pub id: String,
    /// 目的地。
    pub destination: String,
    /// 所属 session id。
    pub session_id: String,
}

/// SIMP session。对标 Spring `SimpSession`。
#[derive(Debug, Clone, Default)]
pub struct SimpSession {
    /// session ID。
    pub id: String,
    /// user name。
    pub user: Option<String>,
    /// 订阅列表。
    pub subscriptions: BTreeMap<String, SimpSubscription>,
}

/// SIMP user。对标 Spring `SimpUser`。
#[derive(Debug, Clone, Default)]
pub struct SimpUser {
    /// user name。
    pub name: String,
    /// 该 user 的所有 session。
    pub sessions: BTreeMap<String, SimpSession>,
}

impl SimpUser {
    /// 是否有 session。
    #[must_use]
    pub fn has_sessions(&self) -> bool {
        !self.sessions.is_empty()
    }
}

/// 默认 SIMP 用户注册表。对标 Spring `DefaultSimpUserRegistry`。
#[derive(Debug, Default)]
pub struct DefaultSimpUserRegistry {
    users: RwLock<BTreeMap<String, SimpUser>>,
}

impl DefaultSimpUserRegistry {
    /// 创建空注册表。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册或更新 session。
    pub async fn register_session(&self, user: Option<String>, session_id: impl Into<String>) {
        let session_id = session_id.into();
        let mut users = self.users.write().await;
        if let Some(user_name) = user {
            let user_entry = users.entry(user_name.clone()).or_insert(SimpUser {
                name: user_name.clone(),
                sessions: BTreeMap::new(),
            });
            user_entry.sessions.insert(
                session_id.clone(),
                SimpSession {
                    id: session_id,
                    user: Some(user_name),
                    subscriptions: BTreeMap::new(),
                },
            );
        }
    }

    /// 移除 session。
    pub async fn remove_session(&self, session_id: &str) {
        let mut users = self.users.write().await;
        for user in users.values_mut() {
            user.sessions.remove(session_id);
        }
    }

    /// 添加订阅。
    pub async fn add_subscription(&self, session_id: &str, subscription: SimpSubscription) {
        let mut users = self.users.write().await;
        for user in users.values_mut() {
            if let Some(session) = user.sessions.get_mut(session_id) {
                session
                    .subscriptions
                    .insert(subscription.id.clone(), subscription);
                break;
            }
        }
    }

    /// 移除订阅。
    pub async fn remove_subscription(&self, session_id: &str, subscription_id: &str) {
        let mut users = self.users.write().await;
        for user in users.values_mut() {
            if let Some(session) = user.sessions.get_mut(session_id) {
                session.subscriptions.remove(subscription_id);
                break;
            }
        }
    }

    /// 返回 user 快照。
    pub async fn get_user(&self, name: &str) -> Option<SimpUser> {
        self.users.read().await.get(name).cloned()
    }

    /// 返回所有 user 名称。
    pub async fn user_names(&self) -> Vec<String> {
        self.users.read().await.keys().cloned().collect()
    }

    /// 返回 session 快照。
    pub async fn get_session(&self, session_id: &str) -> Option<SimpSession> {
        let users = self.users.read().await;
        for user in users.values() {
            if let Some(session) = user.sessions.get(session_id) {
                return Some(session.clone());
            }
        }
        None
    }

    /// 在线 user 数。
    pub async fn user_count(&self) -> usize {
        self.users.read().await.len()
    }

    /// 在线 session 数。
    pub async fn session_count(&self) -> usize {
        self.users
            .read()
            .await
            .values()
            .map(|user| user.sessions.len())
            .sum()
    }
}

/// 共享句柄。
pub type SharedSimpUserRegistry = Arc<DefaultSimpUserRegistry>;

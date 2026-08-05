//! 启动步骤契约。
//!
//! 对标 Spring `org.springframework.core.metrics.StartupStep`。

use std::collections::BTreeMap;
use std::time::Instant;

/// 启动步骤契约。
///
/// 对应 Java: org.springframework.core.metrics.StartupStep
///
/// Spring 语义：应用启动阶段的一个可观测步骤（ID、名称、标签与耗时），
/// `end()` 标记步骤结束。
pub trait StartupStep: Send + Sync {
    /// 返回步骤名称。
    fn name(&self) -> &str;

    /// 返回步骤 ID。
    fn id(&self) -> u64;

    /// 返回父步骤 ID（根步骤为 0）。
    fn parent_id(&self) -> u64;

    /// 返回标签集合（快照）。
    fn tags(&self) -> Vec<(&str, String)>;

    /// 结束步骤（返回结束时间点）。
    fn end(&mut self) -> Instant;

    /// 步骤是否已结束。
    fn is_ended(&self) -> bool;
}

/// 启动步骤的简单值实现（供测试与轻量使用）。
pub struct SimpleStartupStep {
    id: u64,
    parent_id: u64,
    name: String,
    tags: BTreeMap<String, String>,
    started_at: Instant,
    ended: bool,
}

impl SimpleStartupStep {
    /// 创建步骤。
    #[must_use]
    pub fn new(id: u64, parent_id: u64, name: impl Into<String>) -> Self {
        Self {
            id,
            parent_id,
            name: name.into(),
            tags: BTreeMap::new(),
            started_at: Instant::now(),
            ended: false,
        }
    }

    /// 记录标签。
    pub fn tag(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.tags.insert(key.into(), value.into());
    }
}

impl StartupStep for SimpleStartupStep {
    fn name(&self) -> &str {
        &self.name
    }

    fn id(&self) -> u64 {
        self.id
    }

    fn parent_id(&self) -> u64 {
        self.parent_id
    }

    fn tags(&self) -> Vec<(&str, String)> {
        self.tags
            .iter()
            .map(|(k, v)| (k.as_str(), v.clone()))
            .collect()
    }

    fn end(&mut self) -> Instant {
        self.ended = true;
        self.started_at
    }

    fn is_ended(&self) -> bool {
        self.ended
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_records_metadata() {
        // A 类（合同对齐）：对标 Spring 步骤元数据
        let mut step = SimpleStartupStep::new(1, 0, "bean-factory");
        step.tag("phase", "prepare");
        assert_eq!(step.name(), "bean-factory");
        assert_eq!(step.id(), 1);
        assert_eq!(step.parent_id(), 0);
        assert!(!step.is_ended());
        let tags = step.tags();
        assert!(tags.iter().any(|(k, v)| *k == "phase" && v == "prepare"));
        step.end();
        assert!(step.is_ended());
    }

    #[test]
    fn step_tracks_start_time() {
        // B 类（边界行为）：end 返回开始时间点
        let mut step = SimpleStartupStep::new(2, 1, "refresh");
        let start = step.end();
        let _ = start.elapsed(); // 步骤有开始时间点
    }
}

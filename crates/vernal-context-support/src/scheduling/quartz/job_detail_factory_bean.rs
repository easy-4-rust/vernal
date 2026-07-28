//! 任务详情工厂 — 对标 `org.springframework.scheduling.quartz.JobDetailFactoryBean`。

/// 任务详情。
#[derive(Debug, Clone)]
pub struct JobDetail {
    /// 任务名称
    pub name: String,
    /// 任务组
    pub group: String,
    /// 任务描述
    pub description: Option<String>,
    /// 是否持久化
    pub durability: bool,
    /// 是否请求恢复
    pub requests_recovery: bool,
}

/// 任务详情工厂 Bean。
///
/// 对标 Spring 的 `JobDetailFactoryBean`，创建 `JobDetail` 实例。
pub struct JobDetailFactoryBean {
    name: String,
    group: String,
    description: Option<String>,
    job_class_name: String,
}

impl JobDetailFactoryBean {
    /// 创建任务详情工厂。
    pub fn new(name: String, job_class_name: String) -> Self {
        Self {
            name,
            group: String::from("DEFAULT"),
            description: None,
            job_class_name,
        }
    }

    /// 设置任务组。
    pub fn set_group(&mut self, group: String) {
        self.group = group;
    }

    /// 设置任务描述。
    pub fn set_description(&mut self, description: String) {
        self.description = Some(description);
    }

    /// 创建任务详情。
    pub fn job_detail(&self) -> JobDetail {
        JobDetail {
            name: self.name.clone(),
            group: self.group.clone(),
            description: self.description.clone(),
            durability: true,
            requests_recovery: false,
        }
    }
}

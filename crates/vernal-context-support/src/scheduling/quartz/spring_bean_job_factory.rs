//! Spring Bean 任务工厂 — 对标 `SpringBeanJobFactory`。

/// Spring Bean 任务工厂。
pub struct SpringBeanJobFactory {
    ignored_unknown_properties: Vec<String>,
}
impl SpringBeanJobFactory {
    /// 创建 Spring Bean 任务工厂。
    pub fn new() -> Self {
        Self {
            ignored_unknown_properties: Vec::new(),
        }
    }
    /// 设置忽略的未知属性列表。
    pub fn set_ignored_unknown_properties(&mut self, props: Vec<String>) {
        self.ignored_unknown_properties = props;
    }
    /// 获取忽略的未知属性列表。
    pub fn ignored_unknown_properties(&self) -> &[String] {
        &self.ignored_unknown_properties
    }
}
impl Default for SpringBeanJobFactory {
    fn default() -> Self {
        Self::new()
    }
}

//! BeanDefinitionVisitor — 对应 Spring `org.springframework.beans.factory.config.BeanDefinitionVisitor`。
//!
//! Bean 定义访问者。

/// Bean 定义访问者。
///
/// 对应 Java 类：`org.springframework.beans.factory.config.BeanDefinitionVisitor`。
///
/// 用于遍历和修改 Bean 定义中的值。
pub struct BeanDefinitionVisitor {
    visited: Vec<String>,
}

impl BeanDefinitionVisitor {
    /// 创建一个新的实例。
    pub fn new() -> Self {
        Self { visited: Vec::new() }
    }

    /// 访问 Bean 定义。
    pub fn visit(&mut self, bean_name: &str) {
        self.visited.push(bean_name.to_string());
    }

    /// 获取已访问的 Bean 名称。
    pub fn visited(&self) -> &[String] {
        &self.visited
    }

    /// 检查是否已访问。
    pub fn has_visited(&self, bean_name: &str) -> bool {
        self.visited.iter().any(|n| n == bean_name)
    }
}

impl Default for BeanDefinitionVisitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visitor() {
        let mut visitor = BeanDefinitionVisitor::new();
        assert!(!visitor.has_visited("myBean"));

        visitor.visit("myBean");
        assert!(visitor.has_visited("myBean"));
        assert_eq!(visitor.visited().len(), 1);
    }
}

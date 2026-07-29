//! AnnotationProcessor — Spring 风格的注解处理器 trait。
//!
//! 对应 Java 类：`org.springframework.beans.factory.annotation.AnnotationProcessor` 概念。
//!
//! 处理采集到的注解元数据，执行诸如注册 Bean 定义、生成代理等动作。

use std::sync::Arc;

use crate::annotation_metadata::AnnotationMetadata;

/// Spring 风格的注解处理器 trait。
///
/// 对应 Spring 注解处理流水线中的处理器概念。
///
/// 处理器消费 `AnnotationMetadata`，执行相应的副作用
/// （例如注册 Bean、配置环境等）。
pub trait AnnotationProcessor: Send + Sync {
    /// 处理给定的注解元数据。
    ///
    /// # 参数
    ///
    /// * `annotation_metadata` — 待处理的注解元数据
    ///
    /// # 返回
    ///
    /// 处理过程中发生的错误（若有）。
    fn process(
        &self,
        annotation_metadata: &dyn AnnotationMetadata,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 此处理器关注的注解类型名集合。
    ///
    /// 默认返回空切片，表示处理所有注解。
    fn supported_annotation_types(&self) -> &[String] {
        &[]
    }

    /// 是否关注给定的注解类型。
    ///
    /// 默认实现：当 `supported_annotation_types` 为空时关注所有类型，
    /// 否则只关注列表中的类型。
    fn supports(&self, annotation_type: &str) -> bool {
        let supported = self.supported_annotation_types();
        if supported.is_empty() {
            return true;
        }
        supported.iter().any(|t| t == annotation_type)
    }
}

/// 复合注解处理器：按顺序执行多个子处理器。
pub struct CompositeAnnotationProcessor {
    processors: Vec<Arc<dyn AnnotationProcessor>>,
}

impl std::fmt::Debug for CompositeAnnotationProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompositeAnnotationProcessor")
            .field("processor_count", &self.processors.len())
            .finish()
    }
}

impl CompositeAnnotationProcessor {
    /// 创建空的复合处理器。
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
        }
    }

    /// 添加一个子处理器。
    pub fn add(&mut self, processor: Arc<dyn AnnotationProcessor>) {
        self.processors.push(processor);
    }

    /// 子处理器数量。
    pub fn len(&self) -> usize {
        self.processors.len()
    }

    /// 是否没有子处理器。
    pub fn is_empty(&self) -> bool {
        self.processors.is_empty()
    }
}

impl Default for CompositeAnnotationProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl AnnotationProcessor for CompositeAnnotationProcessor {
    fn process(
        &self,
        annotation_metadata: &dyn AnnotationMetadata,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for processor in &self.processors {
            for descriptor in annotation_metadata.annotations() {
                if processor.supports(descriptor.annotation_type()) {
                    processor.process(annotation_metadata)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::annotation_metadata::AnnotationDescriptor;
    use std::sync::Mutex;

    struct CountingProcessor {
        supported: Vec<String>,
        calls: Mutex<usize>,
    }

    impl AnnotationProcessor for CountingProcessor {
        fn process(
            &self,
            _annotation_metadata: &dyn AnnotationMetadata,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            *self.calls.lock().unwrap() += 1;
            Ok(())
        }

        fn supported_annotation_types(&self) -> &[String] {
            &self.supported
        }
    }

    struct StubMetadata {
        annotations: Vec<AnnotationDescriptor>,
    }
    impl AnnotationMetadata for StubMetadata {
        fn annotations(&self) -> &[AnnotationDescriptor] {
            &self.annotations
        }
    }

    #[test]
    fn test_supports() {
        let proc = CountingProcessor {
            supported: vec!["Component".to_owned()],
            calls: Mutex::new(0),
        };
        assert!(proc.supports("Component"));
        assert!(!proc.supports("Service"));
    }

    #[test]
    fn test_composite_invokes_matching() {
        let counter = Arc::new(CountingProcessor {
            supported: vec!["Component".to_owned()],
            calls: Mutex::new(0),
        });
        let mut composite = CompositeAnnotationProcessor::new();
        composite.add(counter.clone());

        let meta = StubMetadata {
            annotations: vec![AnnotationDescriptor::new_class("Component")],
        };
        composite.process(&meta).unwrap();
        assert_eq!(*counter.calls.lock().unwrap(), 1);
    }
}

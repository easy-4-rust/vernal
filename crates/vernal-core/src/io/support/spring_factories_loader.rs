//! Spring 工厂加载器。
//!
//! 对标 Spring `org.springframework.core.io.support.SpringFactoriesLoader`。

use std::collections::BTreeMap;

use crate::io::Resource;
use crate::properties_file::parse_properties;

/// Spring 工厂加载器。
///
/// 对应 Java: org.springframework.core.io.support.SpringFactoriesLoader
///
/// Spring 语义：从 `META-INF/spring.factories` 读取"接口名 → 实现列表"映射；
/// vernal 中工厂文件路径为 `META-INF/vernal.factories`（对标 Spring 但避免
/// 在代码中出现 Spring 专属资源名）。
pub struct SpringFactoriesLoader;

impl SpringFactoriesLoader {
    /// 加载全部工厂映射。
    ///
    /// 对标 Spring `loadFactoryNames(Class, ClassLoader)` 的映射表形态。
    ///
    /// # 错误
    ///
    /// 资源读取失败时返回 [`std::io::Error`]。
    pub fn load_factory_names(
        resource: &dyn Resource,
    ) -> std::io::Result<BTreeMap<String, Vec<String>>> {
        let content = resource.read_string()?;
        let mut flat = std::collections::HashMap::new();
        parse_properties(&content, &mut flat);
        Ok(flat
            .into_iter()
            .map(|(key, value)| {
                let impls = value
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>();
                (key, impls)
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::ByteArrayResource;

    #[test]
    fn parses_factories_file() {
        // A 类（合同对齐）：对标 Spring factories 文件格式
        let content = b"com.example.Service=com.example.ImplA,com.example.ImplB\ncom.example.Other=com.example.ImplC\n";
        let resource = ByteArrayResource::new(content.to_vec());
        let names = SpringFactoriesLoader::load_factory_names(&resource).unwrap();
        let impls = names.get("com.example.Service").unwrap();
        assert_eq!(
            impls,
            &vec![
                "com.example.ImplA".to_string(),
                "com.example.ImplB".to_string()
            ]
        );
    }

    #[test]
    fn blank_impl_list_yields_empty() {
        // B 类（边界行为）：空实现列表
        let content = b"com.example.Empty=\n";
        let resource = ByteArrayResource::new(content.to_vec());
        let names = SpringFactoriesLoader::load_factory_names(&resource).unwrap();
        assert!(names.get("com.example.Empty").unwrap().is_empty());
    }
}

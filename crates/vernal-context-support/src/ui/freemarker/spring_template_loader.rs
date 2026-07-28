//! Spring 模板加载器 — 对标 `org.springframework.ui.freemarker.SpringTemplateLoader`。

use std::path::PathBuf;

/// Spring 模板加载器。
///
/// 对标 Spring 的 `SpringTemplateLoader`，从 Spring 资源路径加载模板。
pub struct SpringTemplateLoader {
    /// 基础路径
    base_path: PathBuf,
    /// 编码
    encoding: String,
}

impl SpringTemplateLoader {
    /// 创建模板加载器。
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            base_path,
            encoding: "UTF-8".to_string(),
        }
    }

    /// 设置编码。
    pub fn set_encoding(&mut self, encoding: impl Into<String>) {
        self.encoding = encoding.into();
    }

    /// 获取模板文件的完整路径。
    pub fn get_template_path(&self, template_name: &str) -> PathBuf {
        self.base_path.join(template_name)
    }

    /// 获取基础路径。
    pub fn base_path(&self) -> &PathBuf {
        &self.base_path
    }

    /// 获取编码。
    pub fn encoding(&self) -> &str {
        &self.encoding
    }
}

impl std::fmt::Debug for SpringTemplateLoader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpringTemplateLoader")
            .field("base_path", &self.base_path)
            .field("encoding", &self.encoding)
            .finish()
    }
}

//! FreeMarker 配置工厂 — 对标 `org.springframework.ui.freemarker.FreeMarkerConfigurationFactory`。
//!
//! 使用 `tera::Tera` 作为底层模板引擎实现。

use std::collections::HashMap;
use std::path::PathBuf;

/// FreeMarker 配置工厂。
///
/// 对标 Spring 的 `FreeMarkerConfigurationFactory`，创建 `tera::Tera` 配置实例。
///
/// # Spring 方法映射
///
/// | Spring 方法 | Rust 方法 | 说明 |
/// |---|---|---|
/// | `setConfigLocation(Resource)` | `set_config_location()` | 设置配置文件位置 |
/// | `setTemplateLoaderPath(String)` | `set_template_loader_path()` | 设置模板加载路径 |
/// | `setFreemarkerSettings(Properties)` | `set_freemarker_settings()` | 设置 FreeMarker 配置 |
/// | `createConfiguration()` | `create_configuration()` | 创建模板引擎配置 |
pub struct FreeMarkerConfigurationFactory {
    /// 配置文件位置
    config_location: Option<PathBuf>,
    /// 模板加载路径
    template_loader_paths: Vec<String>,
    /// FreeMarker 设置
    freemarker_settings: HashMap<String, String>,
}

impl FreeMarkerConfigurationFactory {
    /// 创建配置工厂。
    pub fn new() -> Self {
        Self {
            config_location: None,
            template_loader_paths: Vec::new(),
            freemarker_settings: HashMap::new(),
        }
    }

    /// 设置配置文件位置。
    pub fn set_config_location(&mut self, path: PathBuf) {
        self.config_location = Some(path);
    }

    /// 设置模板加载路径。
    pub fn set_template_loader_path(&mut self, path: impl Into<String>) {
        self.template_loader_paths.push(path.into());
    }

    /// 设置 FreeMarker 配置。
    pub fn set_freemarker_settings(&mut self, settings: HashMap<String, String>) {
        self.freemarker_settings.extend(settings);
    }

    /// 创建 Tera 模板引擎实例。
    pub fn create_configuration(&self) -> Result<tera::Tera, String> {
        let base_dir = self
            .config_location
            .as_ref()
            .and_then(|p| p.parent())
            .unwrap_or(std::path::Path::new("."));

        // 构建模板目录路径
        let template_dir = if self.template_loader_paths.is_empty() {
            base_dir.to_string_lossy().to_string()
        } else {
            self.template_loader_paths
                .first()
                .cloned()
                .unwrap_or_default()
        };

        // 使用 glob 模式加载模板
        let pattern = format!("{}/**/*", template_dir);
        let tera = tera::Tera::new(&pattern).map_err(|e| format!("创建 Tera 实例失败：{e}"))?;

        Ok(tera)
    }
}

impl Default for FreeMarkerConfigurationFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for FreeMarkerConfigurationFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FreeMarkerConfigurationFactory")
            .field("template_loader_paths", &self.template_loader_paths)
            .finish()
    }
}

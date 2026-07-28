//! FreeMarker 模板工具 — 对标 `org.springframework.ui.freemarker.FreeMarkerTemplateUtils`。

use std::collections::HashMap;

/// FreeMarker 模板工具。
///
/// 对标 Spring 的 `FreeMarkerTemplateUtils`，提供模板渲染的便捷方法。
pub struct FreeMarkerTemplateUtils;

impl FreeMarkerTemplateUtils {
    /// 渲染模板并返回结果字符串。
    ///
    /// 对标 `processTemplateIntoString`。
    pub fn process_template_into_string(
        tera: &tera::Tera,
        template_name: &str,
        model: &HashMap<String, tera::Value>,
    ) -> Result<String, String> {
        let context = tera::Context::from_serialize(model)
            .map_err(|e| format!("序列化模板上下文失败：{e}"))?;
        tera.render(template_name, &context)
            .map_err(|e| format!("模板渲染失败（{template_name}）：{e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_rendering() {
        let mut tera = tera::Tera::default();
        tera.add_raw_template("test.html", "Hello, {{ name }}!")
            .unwrap();

        let mut model = HashMap::new();
        model.insert("name".to_string(), tera::Value::String("World".to_string()));

        let result =
            FreeMarkerTemplateUtils::process_template_into_string(&tera, "test.html", &model)
                .unwrap();
        assert_eq!(result, "Hello, World!");
    }
}

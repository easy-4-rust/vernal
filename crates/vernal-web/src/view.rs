//! Spring Web MVC 视图层合同：`Model` / `RenderedView` / `View` / `ViewResolver`。
//!
//! 对标 `org.springframework.ui.Model`、`org.springframework.web.servlet.View`
//! 与 `org.springframework.web.servlet.ViewResolver`。视图引擎（如
//! thymeleaf-rust 的 `ThymeleafViewResolver`）在本合同之上提供具体实现；
//! 渲染产出框架中立的 [`RenderedView`]，由 Web 运行时（vernal-axum /
//! vernal-actix-web 等）或视图引擎适配层组装为各自的响应类型。
//!
//! 不依赖 `vernal-http`（后者依赖本 crate）：状态/Header/Body 用 `http`
//! 与 `bytes` 底层类型表达，避免环。

use indexmap::IndexMap;
use std::any::Any;
use std::fmt::Display;
use std::sync::Arc;

use bytes::Bytes;
use http::{HeaderMap, StatusCode};

/// 处理器与视图之间传递的属性集。
///
/// 对标 `org.springframework.ui.Model`：属性按插入顺序保留、值为类型擦除的
/// 共享对象（Spring 侧为 `Object`，此处为 `Arc<dyn Any + Send + Sync>`），
/// 由具体 `View` 实现负责把值转换为模板变量。
#[derive(Default)]
pub struct Model {
    attributes: IndexMap<String, Option<Arc<dyn Any + Send + Sync>>>,
}

impl Model {
    /// 创建空模型。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加或替换一个属性（对标 `Model#addAttribute`）。
    ///
    /// # 参数
    /// - `name`：属性名。
    /// - `value`：类型擦除的共享值；`None` 对应 Java null 值语义。
    pub fn add_attribute(
        &mut self,
        name: impl Into<String>,
        value: Option<Arc<dyn Any + Send + Sync>>,
    ) {
        self.attributes.insert(name.into(), value);
    }

    /// 读取属性（对标 `Model#getAttribute`；不存在返回 `None`）。
    pub fn get_attribute(&self, name: &str) -> Option<&Option<Arc<dyn Any + Send + Sync>>> {
        self.attributes.get(name)
    }

    /// 是否包含属性（对标 `Model#containsAttribute`）。
    #[must_use]
    pub fn contains_attribute(&self, name: &str) -> bool {
        self.attributes.contains_key(name)
    }

    /// 属性数量。
    #[must_use]
    pub fn attribute_count(&self) -> usize {
        self.attributes.len()
    }

    /// 按插入顺序返回全部属性名（对标 `Model#asMap` 的键序语义）。
    #[must_use]
    pub fn attribute_names(&self) -> Vec<&str> {
        self.attributes.keys().map(String::as_str).collect()
    }

    /// 消费模型并返回内部属性映射（供视图实现批量转换）。
    #[must_use]
    pub fn into_attributes(self) -> IndexMap<String, Option<Arc<dyn Any + Send + Sync>>> {
        self.attributes
    }
}

/// 渲染视图失败的中立错误。
#[derive(Debug)]
pub struct ViewError {
    message: String,
}

impl ViewError {
    /// 创建携带诊断消息的视图错误。
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// 返回诊断消息。
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Display for ViewError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ViewError {}

/// 视图渲染产出的中立结果。
///
/// 状态、Header 与（全量）Body 用 `http`/`bytes` 底层类型表达；流式 Body
/// 场景由视图引擎在自身类型上扩展（本合同预留全量渲染主流形态，与
/// Spring `View#render` 写完整响应的语义一致）。
pub struct RenderedView {
    status: StatusCode,
    headers: HeaderMap,
    body: Bytes,
}

impl RenderedView {
    /// 组装渲染结果（默认 200）。
    #[must_use]
    pub fn new(status: StatusCode, headers: HeaderMap, body: Bytes) -> Self {
        Self {
            status,
            headers,
            body,
        }
    }

    /// 状态码。
    #[must_use]
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// Header 集。
    #[must_use]
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// 全量 Body。
    #[must_use]
    pub fn body(&self) -> &Bytes {
        &self.body
    }

    /// 消费为各部分（供适配层组装具体响应类型）。
    #[must_use]
    pub fn into_parts(self) -> (StatusCode, HeaderMap, Bytes) {
        (self.status, self.headers, self.body)
    }
}

/// 渲染为中立结果的视图。
///
/// 对标 `org.springframework.web.servlet.View#render`；实现负责把
/// [`Model`] 属性与 `locale` 文本转换为模板变量并产出 [`RenderedView`]。
pub trait View: Send + Sync {
    /// 渲染视图。
    ///
    /// # 参数
    /// - `model`：处理器输出的属性集。
    /// - `locale`：BCP-47 语言标签（如 `zh-CN`）；`None` 由实现决定默认。
    ///
    /// # 返回
    /// 渲染完成的框架中立结果。
    fn render(&self, model: &Model, locale: Option<&str>) -> Result<RenderedView, ViewError>;
}

/// 按视图名解析视图。
///
/// 对标 `org.springframework.web.servlet.ViewResolver#resolveViewName`；
/// 典型实现持前缀/后缀（Spring `spring.thymeleaf.prefix/suffix`）把视图名
/// 映射为模板资源名。
pub trait ViewResolver: Send + Sync {
    /// 解析视图名。
    ///
    /// # 参数
    /// - `view_name`：处理器返回的逻辑视图名（如 `home`）。
    /// - `locale`：请求协商的语言标签。
    ///
    /// # 返回
    /// 可渲染的视图；无法解析时返回 `None`（Spring 语义：交给链上下一个
    /// resolver）。
    fn resolve_view_name(&self, view_name: &str, locale: Option<&str>) -> Option<Arc<dyn View>>;
}

#[cfg(test)]
mod tests {
    use super::{Model, RenderedView, View, ViewError, ViewResolver};
    use bytes::Bytes;
    use http::{HeaderMap, StatusCode};
    use std::any::Any;
    use std::sync::Arc;

    #[test]
    fn model_preserves_insertion_order_and_null_semantics() {
        let mut model = Model::new();
        model.add_attribute(
            "first",
            Some(Arc::new("one".to_owned()) as Arc<dyn Any + Send + Sync>),
        );
        model.add_attribute("nullish", None);
        model.add_attribute(
            "second",
            Some(Arc::new(2_i64) as Arc<dyn Any + Send + Sync>),
        );

        assert_eq!(model.attribute_names(), ["first", "nullish", "second"]);
        assert!(model.contains_attribute("nullish"));
        assert!(model.get_attribute("nullish").is_some_and(Option::is_none));
        assert!(
            model
                .get_attribute("second")
                .and_then(Option::as_ref)
                .is_some_and(|value| value.downcast_ref::<i64>() == Some(&2))
        );
        model.add_attribute("first", None);
        assert_eq!(model.attribute_count(), 3, "同名覆盖不增计数");
    }

    struct ProbeView;

    impl View for ProbeView {
        fn render(&self, model: &Model, locale: Option<&str>) -> Result<RenderedView, ViewError> {
            let name = model
                .get_attribute("name")
                .and_then(Option::as_ref)
                .and_then(|value| value.downcast_ref::<String>())
                .cloned()
                .unwrap_or_default();
            let body = format!("hello {name} @{locale:?}");
            Ok(RenderedView::new(
                StatusCode::OK,
                HeaderMap::new(),
                Bytes::from(body),
            ))
        }
    }

    struct ProbeResolver;

    impl ViewResolver for ProbeResolver {
        fn resolve_view_name(
            &self,
            view_name: &str,
            _locale: Option<&str>,
        ) -> Option<Arc<dyn View>> {
            (view_name == "home").then(|| Arc::new(ProbeView) as Arc<dyn View>)
        }
    }

    #[test]
    fn resolver_returns_none_for_unknown_and_renders_for_known() {
        let resolver = ProbeResolver;
        assert!(resolver.resolve_view_name("missing", None).is_none());

        let view = resolver
            .resolve_view_name("home", Some("zh-CN"))
            .expect("home");
        let mut model = Model::new();
        model.add_attribute(
            "name",
            Some(Arc::new("vernal".to_owned()) as Arc<dyn Any + Send + Sync>),
        );
        let rendered = view.render(&model, Some("zh-CN")).expect("render");
        assert_eq!(rendered.status(), StatusCode::OK);
        assert_eq!(rendered.body(), "hello vernal @Some(\"zh-CN\")");
    }
}

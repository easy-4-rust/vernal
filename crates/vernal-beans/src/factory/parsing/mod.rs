//! factory::parsing — 对应 Spring beans.factory.parsing 包。
//!
//! 提供 Bean 定义解析所需的基础设施：位置信息、问题报告、源提取、
//! 事件监听、解析状态跟踪，以及各种解析中间表示（组件定义、别名、
//! 导入、默认值、条目等）。

// ── 异常 ────────────────────────────────────────────────────────

/// Spring BeanDefinitionParsingException 类。
pub mod bean_definition_parsing_exception;

// ── 基础设施 ───────────────────────────────────────────────────

/// 源文件位置信息。
pub mod location;

/// Bean 定义解析问题。
pub mod problem;

/// 问题报告器 trait。
pub mod problem_reporter;

/// 快速失败的问题报告器。
pub mod fail_fast_problem_reporter;

/// 源对象提取器 trait。
pub mod source_extractor;

/// 空源提取器（始终返回 None）。
pub mod null_source_extractor;

/// 直通源提取器（原样返回元数据）。
pub mod pass_through_source_extractor;

// ── 读取器上下文与事件 ──────────────────────────────────────────

/// 读取器事件监听器 trait。
pub mod reader_event_listener;

/// 空的读取器事件监听器。
pub mod empty_reader_event_listener;

/// 读取器上下文（封装报告器、监听器和提取器）。
pub mod reader_context;

/// 解析状态栈。
pub mod parse_state;

// ── 组件定义 ────────────────────────────────────────────────────

/// 不可变组件定义对象（工厂级别，此处为便捷访问）。
pub mod component_definition;

/// 抽象组件定义基类。
pub mod abstract_component_definition;

/// 基于 Bean 定义的组件定义。
pub mod bean_component_definition;

/// 组合组件定义（包含多个子组件）。
pub mod composite_component_definition;

// ── 解析定义 ────────────────────────────────────────────────────

/// Bean 别名定义。
pub mod alias_definition;

/// 导入定义。
pub mod import_definition;

/// 默认值定义 trait 和实现。
pub mod defaults_definition;

// ── 解析条目 ────────────────────────────────────────────────────

/// Bean 条目（解析阶段的 Bean 摘要）。
pub mod bean_entry;

/// 构造器参数条目。
pub mod constructor_argument_entry;

/// 属性注入条目。
pub mod property_entry;

/// 限定符条目。
pub mod qualifier_entry;
